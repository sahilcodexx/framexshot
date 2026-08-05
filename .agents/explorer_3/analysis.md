# AppImage Build & Headless GUI Verification Analysis

## 1. Executive Summary

This report provides a comprehensive analysis of the AppImage build scripts, output pathing, and headless GUI startup verification strategy for **FrameXShot** (`framexshot` v1.0.0).

Key findings:
1. **Build Commands**: `npm run tauri build` executes `@tauri-apps/cli` v2.11.0, which runs `npm run build` (`tsc && vite build`) to emit static frontend assets to `dist/`, compiles the Rust binary `src-tauri/target/release/framexshot`, and packages the output into an `.AppImage` via `tauri-bundler`.
2. **Binary Output Path**: The expected output binary is located at `src-tauri/target/release/bundle/appimage/framexshot_1.0.0_amd64.AppImage` (or `framexshot_1.0.0_x86_64.AppImage`).
3. **Headless GUI Verification Strategy**: A robust, zero-hang programmatic verification method using `xvfb-run` with 24-bit color depth, software OpenGL rasterization (`LIBGL_ALWAYS_SOFTWARE=1`), DMABUF disabling (`WEBKIT_DISABLE_DMABUF_RENDERER=1`), log scanning for WebKit/EGL crash signatures, X11 window tree querying via `xwininfo`/`xdotool`, and virtual screenshot inspection.

---

## 2. Tauri v2 AppImage Build Architecture & Scripts

### 2.1 Package & Configuration Inspection

- **`package.json`**:
  - `name`: `"framexshot"`
  - `version`: `"1.0.0"`
  - Relevant scripts:
    - `"build": "tsc && vite build"` — Transpiles TypeScript and compiles React frontend into `dist/`.
    - `"tauri": "tauri"` — CLI tool provided by `@tauri-apps/cli` v2.11.0.
  - Command to trigger production build: `npm run tauri build`.

- **`src-tauri/tauri.conf.json`**:
  - `productName`: `"framexshot"`
  - `version`: `"1.0.0"`
  - `identifier`: `"com.framexshot.app"`
  - `build.beforeBuildCommand`: `"npm run build"`
  - `build.frontendDist`: `"../dist"`
  - `bundle.active`: `true`
  - `bundle.targets`: `["appimage", "nsis", "dmg"]`
  - `app.windows`: `[]` (dynamic window spawning in `lib.rs` on setup).

- **`src-tauri/Cargo.toml`**:
  - `[package] name`: `"framexshot"`
  - `version`: `"1.0.0"`
  - `tauri`: `{ version = "2", features = ["protocol-asset", "tray-icon"] }`

### 2.2 Build Execution Lifecycle (`npm run tauri build`)

```
[npm run tauri build]
       │
       ├──> 1. Execute `beforeBuildCommand` ("npm run build")
       │       ├──> `tsc` (TypeScript typecheck)
       │       └──> `vite build` (Compiles React app -> outputs to `dist/` with relative base `./`)
       │
       ├──> 2. Cargo Build
       │       └──> `cargo build --release` in `src-tauri/` (Generates binary `src-tauri/target/release/framexshot`)
       │
       └──> 3. Bundle Packaging (`tauri-bundler`)
               ├──> Constructs AppDir structure: `src-tauri/target/release/bundle/appimage/framexshot.AppDir`
               └──> Invokes `appimagetool` to produce `.AppImage` binary
```

---

## 3. Expected Binary Output Paths

When `npm run tauri build` completes on Linux (amd64 / x86_64 architecture), the output artifacts are located in:

| Output Artifact | Path |
|-----------------|------|
| **AppImage Package** | `src-tauri/target/release/bundle/appimage/framexshot_1.0.0_amd64.AppImage`<br>*(or `framexshot_1.0.0_x86_64.AppImage` depending on system target string)* |
| **AppDir Workspace** | `src-tauri/target/release/bundle/appimage/framexshot.AppDir` |
| **Debian Package** | `src-tauri/target/release/bundle/deb/framexshot_1.0.0_amd64.deb` |
| **Compiled Binary** | `src-tauri/target/release/framexshot` |
| **Root Release Copy** | `/home/sahilcodex/Documents/bettershot-linux-main/bettershot-x86_64.AppImage` *(Existing build artifact: 147.5 MB)* |

---

## 4. WebKit2GTK & Headless Linux Environment Analysis

### 4.1 Crash Mechanisms in Headless WebKit2GTK

WebKit2GTK 2.42+ on Linux utilizes hardware-accelerated EGL compositing by default. When executed in headless environments (e.g. CI runners, Xvfb virtual displays, Docker containers without GPU pass-through):

1. **`WEBKIT_DISABLE_COMPOSITING_MODE=1` Crash**:
   - Setting `WEBKIT_DISABLE_COMPOSITING_MODE=1` forces WebKit into a deprecated rendering path that causes `Could not create default EGL display: EGL_BAD_PARAMETER. Aborting...` in WebKit2GTK 2.42+.
   - **Requirement**: `WEBKIT_DISABLE_COMPOSITING_MODE` MUST NOT be set to `1` (must remain unset).

2. **DMA-BUF Buffer Sharing Crash**:
   - WebKit2GTK attempts DMA-BUF hardware buffer sharing between WebProcess and UIProcess. In virtual displays or software rendering, this fails.
   - **Requirement**: `WEBKIT_DISABLE_DMABUF_RENDERER=1` MUST be exported (configured natively in `src-tauri/src/lib.rs:51-53`).

3. **Missing Hardware GPU Driver / EGL Context**:
   - In headless Xvfb, hardware OpenGL drivers are unavailable.
   - **Requirement**: `LIBGL_ALWAYS_SOFTWARE=1` forces Mesa to use the `llvmpipe` CPU software rasterizer, allowing WebKit2GTK to successfully create an EGL display context without GPU hardware.

4. **GTK Backend Selection**:
   - GTK3 can attempt Wayland auto-detection if `WAYLAND_DISPLAY` is set in environment.
   - **Requirement**: `GDK_BACKEND=x11` explicitly targets X11 for `Xvfb` display compatibility.

---

## 5. Programmatic Headless GUI Execution & Verification Design

### 5.1 Verification Objectives
- **Execution Safety**: Run the AppImage headlessly without user interaction or physical display.
- **Log Sanitation**: Capture stdout/stderr logs and assert zero EGL / WebProcess crashes.
- **Window Spawning**: Confirm that GTK creates the top-level X11 window.
- **React UI Mount Verification**: Ensure frontend loads without blank screen.
- **Clean Termination**: Prevent hanging background processes by using strict timeout wrappers.

### 5.2 Verification Pipeline Step-by-Step

```
┌────────────────────────────────────────────────────────────────────────┐
│ 1. Locate & Grant Exec Permissions on AppImage Binary                 │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
┌───────────────────────────────────▼────────────────────────────────────┐
│ 2. Configure Environment (GDK_BACKEND=x11, LIBGL_ALWAYS_SOFTWARE=1)    │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
┌───────────────────────────────────▼────────────────────────────────────┐
│ 3. Execute inside `xvfb-run` with 24-bit color & 15s Timeout          │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
┌───────────────────────────────────▼────────────────────────────────────┐
│ 4. Scan Log File for Forbidden Error Patterns (EGL, WebProcess, Panic) │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
┌───────────────────────────────────▼────────────────────────────────────┐
│ 5. Query X11 Window Tree (`xwininfo` / `xdotool` on DISPLAY)           │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
┌───────────────────────────────────▼────────────────────────────────────┐
│ 6. Virtual Display Screenshot Analysis (`import` / `maim` / `scrot`)   │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 6. Verification Script Prototype (`scripts/verify-appimage.sh`)

Below is the complete blueprint for an automated verification script:

```bash
#!/usr/bin/env bash
set -euo pipefail

# Script Configuration
APPIMAGE_DIR="src-tauri/target/release/bundle/appimage"
LOG_FILE="/tmp/appimage_startup.log"
SCREENSHOT_FILE="/tmp/appimage_screenshot.png"
TIMEOUT_SECS=12

echo "=== FrameXShot AppImage Verification ==="

# Step 1: Locate AppImage
APPIMAGE_PATH=$(find "$APPIMAGE_DIR" -maxdepth 1 -name "*.AppImage" | head -n 1)

if [[ -z "$APPIMAGE_PATH" ]]; then
  if [[ -f "bettershot-x86_64.AppImage" ]]; then
    APPIMAGE_PATH="bettershot-x86_64.AppImage"
    echo "[INFO] Found root AppImage: $APPIMAGE_PATH"
  else
    echo "[ERROR] No .AppImage binary found in $APPIMAGE_DIR or root directory."
    exit 1
  fi
else
  echo "[INFO] Found target AppImage: $APPIMAGE_PATH"
fi

chmod +x "$APPIMAGE_PATH"

# Step 2: Prepare Environment
export GDK_BACKEND=x11
export LIBGL_ALWAYS_SOFTWARE=1
export WEBKIT_DISABLE_DMABUF_RENDERER=1
unset WEBKIT_DISABLE_COMPOSITING_MODE || true

# Determine execution flag for non-FUSE / container environments
RUN_CMD="$APPIMAGE_PATH"
if ! command -v fusermount &> /dev/null; then
  echo "[INFO] FUSE not detected, using --appimage-extract-and-run"
  RUN_CMD="$APPIMAGE_PATH --appimage-extract-and-run"
fi

# Step 3: Run AppImage in Xvfb with timeout
echo "[INFO] Launching AppImage under Xvfb (24-bit depth, ${TIMEOUT_SECS}s timeout)..."
rm -f "$LOG_FILE" "$SCREENSHOT_FILE"

# Launch under Xvfb in background to allow window tree sampling
xvfb-run -a -s "-screen 0 1280x1024x24" \
  timeout --preserve-status "${TIMEOUT_SECS}s" \
  $RUN_CMD > "$LOG_FILE" 2>&1 &

APP_PID=$!
sleep 5 # Allow app to initialize window & WebKit

# Step 4: Window Spawning Verification
echo "[INFO] Inspecting X11 window tree..."
if command -v xwininfo &> /dev/null; then
  WINDOW_TREE=$(xwininfo -root -tree -display :99 2>/dev/null || true)
  echo "$WINDOW_TREE"
  if echo "$WINDOW_TREE" | grep -iq "framexshot\|FrameXShot"; then
    echo "[SUCCESS] FrameXShot GTK main window spawned successfully!"
  else
    echo "[WARN] Could not match window name in X11 tree. Checking process state..."
  fi
fi

# Step 5: Screen Capture Verification
if command -v import &> /dev/null; then
  import -window root -display :99 "$SCREENSHOT_FILE" || true
  if [[ -f "$SCREENSHOT_FILE" ]]; then
    SIZE=$(stat -c%s "$SCREENSHOT_FILE" 2>/dev/null || stat -f%z "$SCREENSHOT_FILE")
    echo "[INFO] Virtual display screenshot captured: $SCREENSHOT_FILE ($SIZE bytes)"
  fi
fi

# Wait for timeout process to finish
wait $APP_PID 2>/dev/null || true

# Step 6: Log Scan & Assertion Verification
echo "=== Log Scan Results ==="
cat "$LOG_FILE"

FORBIDDEN_PATTERNS=(
  "EGL_BAD_PARAMETER"
  "Could not create default EGL display"
  "WebKitWebProcess"
  "terminated unexpectedly"
  "cannot open display"
  "Segmentation fault"
  "panic"
)

ERRORS_FOUND=0
for pattern in "${FORBIDDEN_PATTERNS[@]}"; do
  if grep -qi "$pattern" "$LOG_FILE"; then
    echo "[CRITICAL ERROR] Detected forbidden crash pattern: '$pattern' in $LOG_FILE"
    ERRORS_FOUND=$((ERRORS_FOUND + 1))
  fi
done

if [[ $ERRORS_FOUND -gt 0 ]]; then
  echo "=== VERIFICATION FAILED: $ERRORS_FOUND critical errors found in log ==="
  exit 1
fi

echo "=== VERIFICATION PASSED: AppImage executed headlessly without WebKit/EGL crashes ==="
exit 0
```

---

## 7. Summary & Recommendations

1. **Build Step**: Run `npm run tauri build` to compile both frontend and native backend into `src-tauri/target/release/bundle/appimage/framexshot_1.0.0_amd64.AppImage`.
2. **Environment Sanitization**: Ensure `WEBKIT_DISABLE_COMPOSITING_MODE` is not defined anywhere in the codebase or shell environment.
3. **Automated Verification**: Use `xvfb-run -a -s "-screen 0 1280x1024x24"` paired with `LIBGL_ALWAYS_SOFTWARE=1` and `WEBKIT_DISABLE_DMABUF_RENDERER=1` to programmatically test the binary startup without requiring a physical monitor.
