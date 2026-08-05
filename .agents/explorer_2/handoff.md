# Handoff Report — Explorer 2 (Vite & Tauri Bundling Specialist)

**Type**: Soft Handoff  
**Working Directory**: `/home/sahilcodex/Documents/bettershot-linux-main/.agents/explorer_2`  
**Target Audience**: Orchestrator / Implementer  

---

## 1. Observation

1. **Vite Base Path (`vite.config.ts:10`)**:
   - `vite.config.ts` line 10 specifies `base: "./"`.
   - `vite.config.ts` lines 15–19 specify `resolve.alias: { "@": path.resolve(__dirname, "./src") }`.
   - `vite.config.ts` lines 20–40 specify target `es2020` and manual vendor chunk splits.

2. **Tauri Build & Window Configuration (`src-tauri/tauri.conf.json`)**:
   - Line 10: `"frontendDist": "../dist"`.
   - Line 13: `"windows": []`.
   - Windows are created programmatically in Rust:
     - `main` window: `src-tauri/src/lib.rs` (lines 25–32 & 106–116) via `WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))`.
     - `quick-overlay` window: `src-tauri/src/commands.rs` (lines 515–526) via `WebviewWindowBuilder::new(&app, "quick-overlay", WebviewUrl::App("index.html?overlay=1".into()))`.
   - Both window labels (`"main"`, `"quick-overlay"`) are explicitly listed in `src-tauri/capabilities/default.json` (line 5) and `src-tauri/capabilities/desktop.json` (lines 8–11).

3. **Content Security Policy Defect (`src-tauri/tauri.conf.json:15`)**:
   - Current CSP string:
     `"csp": "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' asset: https://asset.localhost data: blob:; font-src 'self' data:; connect-src 'self' ipc: http://ipc.localhost; frame-src 'none'; object-src 'none'; base-uri 'self'; media-src 'none'; prefetch-src 'self'; form-action 'none'"`
   - `img-src` allows `asset:` and `https://asset.localhost`, but omits `http://asset.localhost`.
   - In Tauri 2 on Linux, `convertFileSrc()` generates asset URLs using `http://asset.localhost/...` or `asset://...`.

4. **Static Asset Bundling & Migration (`src/lib/asset-registry.ts` & `src/assets/`)**:
   - `src/lib/asset-registry.ts` lines 10–62 contain explicit ES module `import` statements for all 50+ background images, Mac assets, and gradient webp/jpg files in `src/assets/`.
   - Lines 145–166: `resolveBackgroundPath` converts storable asset IDs (e.g. `bg-18`, `gradient-1`) or data URLs into Vite-resolved module URLs.
   - Lines 187–214: `migrateStoredValue` automatically migrates legacy disk paths (e.g., `/src/assets/mesh/mesh1.webp`) loaded from `settings.json` to canonical asset IDs (`gradient-1`).

5. **Dynamic Thumbnail Generation (`src/lib/thumbnail-utils.ts:8-38`)**:
   - Thumbnails are generated on-the-fly using HTML5 Canvas `toDataURL("image/jpeg", 0.7)` and cached in memory. Output data URIs (`data:image/jpeg;base64,...`) are permitted by `img-src data:` in the CSP.

---

## 2. Logic Chain

1. **Vite relative asset paths (`base: "./"`)**:
   - Setting `base: "./"` ensures that when Vite compiles `index.html` and JS/CSS bundles, all asset URLs (scripts, stylesheets, imported images) use relative paths (e.g., `./assets/index-B_...js`).
   - When loaded inside an AppImage container via custom URI schemes (`tauri://localhost/index.html` or file/custom protocol), relative pathing prevents 404 errors caused by absolute path assumptions (`/assets/...`).

2. **Window setup & capability matching**:
   - `"windows": []` in `tauri.conf.json` does not cause runtime window creation errors because `main` and `quick-overlay` windows are created dynamically via Rust `WebviewWindowBuilder`.
   - The window labels `"main"` and `"quick-overlay"` match the target arrays in `src-tauri/capabilities/default.json` and `desktop.json`, ensuring IPC commands and plugin capabilities are properly bound.

3. **CSP Vulnerability Identification**:
   - In `convertFileSrc(imagePath)` calls in `ImageEditor.tsx:221`, `RegionSelector.tsx:68`, `QuickOverlay.tsx:147`, and `auto-process.ts:173`, Tauri converts local file paths into asset protocol URLs.
   - On Linux WebKit2GTK, these URLs take the form `http://asset.localhost/<path>`.
   - Because `img-src` in `tauri.conf.json:15` only lists `https://asset.localhost` (HTTPS scheme), WebKit2GTK's CSP engine will block local screenshot preview image requests with a CSP violation error unless `http://asset.localhost` is added.

4. **Asset Registry Integrity**:
   - Explicit ES imports in `asset-registry.ts` ensure Vite includes all images in the `dist/assets/` output folder with hash suffixes.
   - The migration logic in `asset-registry.ts` prevents broken image references when loading legacy user settings referencing unhashed `/src/assets/...` paths.

---

## 3. Caveats

- **Unbuilt Binary State**: No `npm run build` or `npm run tauri build` command was executed during this read-only investigation.
- **Protocol Protocol Variability**: Different WebKit2GTK / Tauri versions on Linux may use either `http://asset.localhost` or `asset://localhost`. Adding both `http://asset.localhost` and `asset:` (or `https://asset.localhost`) guarantees coverage across all Linux distros.

---

## 4. Conclusion

- **Vite and Tauri asset configurations are structurally sound**, with correct relative base pathing (`base: "./"`), correct `frontendDist` (`"../dist"`), complete static asset imports in `asset-registry.ts`, and proper capability bindings.
- **One Critical Actionable Bug Identified**: `src-tauri/tauri.conf.json` CSP must be updated to include `http://asset.localhost` under `img-src` and `https://ipc.localhost` under `connect-src` to avoid WebKit2GTK image loading failures in AppImage production builds.

---

## 5. Verification Method

1. **Inspect CSP Modification**:
   - Check `src-tauri/tauri.conf.json` line 15 to confirm `img-src` contains `http://asset.localhost`.
2. **Build and Test Execution**:
   - Run `npm run build` (or `npm run tauri build`).
   - Launch the production AppImage executable.
   - Verify that captured screenshots load in `ImageEditor` and `QuickOverlay` without WebKit browser console errors regarding CSP violations (`Refused to load the image 'http://asset.localhost/...' because it violates the following Content Security Policy directive...`).

---

## Remaining Work (For Implementer / Next Stage)

- Apply the CSP fix to `src-tauri/tauri.conf.json`.
- Proceed with building and testing the production AppImage release.
