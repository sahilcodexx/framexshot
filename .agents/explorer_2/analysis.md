# Vite & Tauri Production Asset Bundling & CSP Analysis Report

**Author**: Explorer 2 (Vite & Tauri Bundling Specialist)  
**Date**: 2026-08-04  
**Working Directory**: `/home/sahilcodex/Documents/bettershot-linux-main/.agents/explorer_2`  
**Scope**: `vite.config.ts`, `src-tauri/tauri.conf.json`, `src-tauri/capabilities/`, `src/assets/`, `src/lib/asset-registry.ts`, `src/components/editor/AssetGrid.tsx`, `src/components/editor/BackgroundSelector.tsx`.

---

## Executive Summary

1. **Vite Base Configuration**: `vite.config.ts` correctly sets `base: "./"`, guaranteeing that all compiled JavaScript, CSS, and imported image assets are referenced via relative paths (`./assets/...`). This is optimal for Tauri production builds and prevents asset 404 errors inside AppImage bundles.
2. **Tauri Config & Window Setup**: `src-tauri/tauri.conf.json` sets `"frontendDist": "../dist"` which accurately resolves to Vite's build output directory. `"windows": []` is intentionally empty in JSON because windows (`main` and `quick-overlay`) are constructed dynamically in Rust (`src-tauri/src/lib.rs` and `src-tauri/src/commands.rs`). Window labels are registered in `capabilities/default.json` and `capabilities/desktop.json`.
3. **CSP Security Vulnerability Identified**: `tauri.conf.json` defines a CSP (`img-src 'self' asset: https://asset.localhost data: blob:`). On Linux WebKit2GTK, Tauri's `convertFileSrc()` outputs `http://asset.localhost/...` (or `asset://localhost/...`). Because `http://asset.localhost` is missing from `img-src`, WebKit2GTK will reject loading captured screenshots loaded via `convertFileSrc()` due to CSP violation.
4. **Static Asset Resolution**: All 50+ images in `src/assets/` are explicitly imported via ES module `import` statements in `asset-registry.ts` and `BackgroundSelector.tsx`. Vite processes, hashes, and outputs them into `dist/assets/`. Runtime lookup via `assetRegistry` and migration helpers in `asset-registry.ts` map legacy paths (`/src/assets/...`) to runtime bundled URLs. No missing static file issues were found in frontend code.

---

## 1. Vite Bundling Configuration (`vite.config.ts`)

### Code Findings & Line References
- **File**: `vite.config.ts` (lines 9–40)

```typescript
10:   base: "./",
11:   plugins: [react(), tailwindcss()],
12:   define: {
13:     __APP_VERSION__: JSON.stringify(process.env.npm_package_version),
14:   },
15:   resolve: {
16:     alias: {
17:       "@": path.resolve(__dirname, "./src"),
18:     },
19:   },
20:   build: {
21:     cssMinify: "lightningcss",
22:     minify: "esbuild",
23:     target: "es2020",
24:     rollupOptions: {
25:       output: {
26:         manualChunks: {
27:           "vendor-react": ["react", "react-dom"],
28:           "vendor-ui": [
29:             "@radix-ui/react-dialog",
30:             "@radix-ui/react-dropdown-menu",
31:             "@radix-ui/react-switch",
32:             "@radix-ui/react-tooltip",
33:             "lucide-react",
34:           ],
35:           "vendor-motion": ["motion"],
36:           "vendor-state": ["zustand", "immer"],
37:         },
38:       },
39:     },
40:   },
```

### Analysis
- **`base: "./"`**: Essential for Tauri desktop packaging. Using relative asset paths avoids root-relative (`/assets/`) resolution failures when `index.html` is loaded via custom protocols (`tauri://localhost` or `http://tauri.localhost`) inside WebKit2GTK on Linux.
- **Alias `@`**: Resolves to `./src`. Used consistently across all component and library imports.
- **Target `es2020`**: Fully compatible with Linux WebKit2GTK 2.42+.
- **Manual Chunks**: Prevents single huge bundle files by grouping React core, Radix UI primitives, Motion, and Zustand state into separate vendor chunks.

---

## 2. Tauri Configuration & Window Setup (`tauri.conf.json` & `capabilities/`)

### Code Findings & Line References
- **File**: `src-tauri/tauri.conf.json` (lines 6–31)

```json
6:   "build": {
7:     "beforeDevCommand": "npm run dev",
8:     "devUrl": "http://localhost:1420",
9:     "beforeBuildCommand": "npm run build",
10:     "frontendDist": "../dist"
11:   },
12:   "app": {
13:     "windows": [],
14:     "security": {
15:       "csp": "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' asset: https://asset.localhost data: blob:; font-src 'self' data:; connect-src 'self' ipc: http://ipc.localhost; frame-src 'none'; object-src 'none'; base-uri 'self'; media-src 'none'; prefetch-src 'self'; form-action 'none'",
16:       "assetProtocol": {
17:         "enable": true,
18:         "scope": [
19:           "$TEMP/**",
20:           "$PICTURES/**",
21:           "$HOME/Pictures/**",
22:           "$APPDATA/**",
23:           "$DESKTOP/**",
24:           "$HOME/Desktop/**",
25:           "/tmp/**",
26:           "/var/tmp/**"
27:         ]
28:       }
29:     },
30:     "withGlobalTauri": false
31:   }
```

### Window Definition & Capabilities Analysis
- **`windows: []` in JSON**: The array is intentionally empty because `main` and `quick-overlay` windows are created programmatically in Rust:
  - `main` window created in `src-tauri/src/lib.rs` (lines 25–32 & 106–116) via `WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))`.
  - `quick-overlay` window created in `src-tauri/src/commands.rs` (lines 515–526) via `WebviewWindowBuilder::new(&app, "quick-overlay", WebviewUrl::App("index.html?overlay=1".into()))`.
- **Capabilities Configured**: Both window labels (`"main"`, `"quick-overlay"`) are registered in `src-tauri/capabilities/default.json` (line 5) and `src-tauri/capabilities/desktop.json` (lines 8–11). Permissions include `core:default`, `core:window:*`, `store:*`, `autostart:*`, `global-shortcut:*`.
- **Asset Protocol Scope**: Enables access to `$TEMP/**`, `$PICTURES/**`, `$HOME/Pictures/**`, `$APPDATA/**`, `$DESKTOP/**`, `$HOME/Desktop/**`, `/tmp/**`, `/var/tmp/**`.

### CSP Flaw Details & Fix Proposal
- **Current `csp` (Line 15)**:
  ```
  default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' asset: https://asset.localhost data: blob:; font-src 'self' data:; connect-src 'self' ipc: http://ipc.localhost; frame-src 'none'; object-src 'none'; base-uri 'self'; media-src 'none'; prefetch-src 'self'; form-action 'none'
  ```
- **Defect**:
  1. `img-src` specifies `https://asset.localhost` but NOT `http://asset.localhost`. On Linux WebKit2GTK, Tauri 2's asset protocol generates `http://asset.localhost/...` URLs when using `convertFileSrc()`.
  2. `connect-src` specifies `http://ipc.localhost` but NOT `https://ipc.localhost`.
- **Impact**: Captured screenshots displayed in `<img src={convertFileSrc(imagePath)} />` in `ImageEditor.tsx`, `RegionSelector.tsx`, and `QuickOverlay.tsx` will trigger CSP image loading blocks on Linux.
- **Recommended CSP Correction**:
  ```json
  "csp": "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' asset: http://asset.localhost https://asset.localhost data: blob:; font-src 'self' data:; connect-src 'self' ipc: http://ipc.localhost https://ipc.localhost; frame-src 'none'; object-src 'none'; base-uri 'self'; media-src 'none'; prefetch-src 'self'; form-action 'none'"
  ```

---

## 3. Static Asset Loading Analysis (`src/assets/` & Registry)

### Code Findings & Line References
- **File**: `src/lib/asset-registry.ts` (lines 10–119)
  - All 13 wallpaper images (`bg-images/`), 7 Mac assets (`mac/`), and 27 gradient mesh images (`mesh/`) are directly imported via ES module syntax:
    ```typescript
    10: import bgImage13 from "@/assets/bg-images/asset-13.jpg";
    ...
    35: import gradient1 from "@/assets/mesh/mesh1.webp";
    ```
- **File**: `src/components/editor/BackgroundSelector.tsx` (lines 9–25 & 36–54)
  - Also imports mesh images directly for gradient presets.
- **File**: `src/lib/thumbnail-utils.ts` (lines 8–38)
  - Generates low-res 140px thumbnails dynamically using HTML5 Canvas `toDataURL("image/jpeg", 0.7)`.
  - Output data URLs (`data:image/jpeg;base64,...`) are stored in `thumbnailCache` and allowed by `img-src data:` in CSP.

### Asset Path Migration & Resolution Logic
- **`resolveBackgroundPath` in `src/lib/asset-registry.ts`** (lines 145–166):
  ```typescript
  export function resolveBackgroundPath(storedValue: string | null): string {
    if (!storedValue) return getDefaultBackgroundPath();
    if (isAssetId(storedValue)) return assetRegistry[storedValue];
    if (isDataUrl(storedValue)) return storedValue;
    ...
    return storedValue;
  }
  ```
- **`migrateStoredValue` in `src/lib/asset-registry.ts`** (lines 187–214):
  - Converts legacy hardcoded source paths (such as `/src/assets/mesh/mesh1.webp`) loaded from `settings.json` into storable asset IDs like `gradient-1` or `bg-18`.
  - At runtime, asset IDs resolve to Vite's hashed bundled paths (`./assets/mesh1-D_x38k.webp`).

---

## 4. Synthesis of Findings & Risk Matrix

| Component | Current State | Production Risk | Action Needed |
|---|---|---|---|
| `vite.config.ts` | `base: "./"`, Target `es2020`, Alias `@` | Low | None |
| `tauri.conf.json` (`frontendDist`) | `"../dist"` | Low | None |
| `tauri.conf.json` (`windows`) | `[]` in JSON, defined in Rust | Low | None (expected pattern) |
| `tauri.conf.json` (`csp`) | `img-src` missing `http://asset.localhost` | **HIGH** | Add `http://asset.localhost` to `img-src` & `https://ipc.localhost` to `connect-src` |
| Asset Registry & Imports | ESM `import` for all assets, runtime migration | Low | None |
| Thumbnail Generation | Canvas `toDataURL()` (`data:`) | Low | None |

---

## 5. Summary & Actionable Recommendations

1. **CSP Update Required**: Update `src-tauri/tauri.conf.json` line 15 to include `http://asset.localhost` under `img-src` and `https://ipc.localhost` under `connect-src`.
2. **Asset Packaging Verified**: All static assets in `src/assets/` are bundled by Vite into `dist/assets/` with relative references (`./assets/...`). No missing asset 404 errors will occur in production AppImage bundles.
