# Project Orchestration Plan: FrameXShot Linux AppImage Fix & Verification

## Objective
Fix WebKit environment variable startup crashes, verify Vite/Tauri production asset bundling, build the Linux AppImage release, and programmatically verify that the React UI loads successfully in the AppImage binary.

## Phases

### Phase 0: Initial Survey & Investigation
- Dispatch 3 parallel Explorer agents to survey the codebase, specifications, and build infrastructure.
  - **Explorer 1 (WebKit & WebProcess Crash)**: Survey `src-tauri/src/lib.rs`, `src-tauri/src/main.rs`, environment variable setups, WebKit2GTK 2.42+ EGL display / compositing flags, and fallback WebKit flags for Linux compatibility.
  - **Explorer 2 (Vite & Tauri Bundling)**: Survey `vite.config.ts`, `tauri.conf.json`, asset base paths (`base: './'`), CSP rules, static assets (`src/assets/`, `src/lib/asset-registry.ts`), and path resolution in production bundles.
  - **Explorer 3 (AppImage Build & Headless E2E Verification)**: Survey `package.json`, Tauri build scripts, AppImage output path (`src-tauri/target/release/bundle/appimage/`), X11/xvfb headless execution methods, and GUI startup verification techniques.

### Phase 1: Architecture & Feature Inventory (`PROJECT.md`)
- Aggregate Explorer reports into `PROJECT.md`.
- Establish Feature Inventory, Code Layout, and Milestone Decomposition.

### Phase 2: Milestone Execution Loop
- **Milestone 1: WebKit Env Vars & WebProcess Startup Crash Fix**
  - Dispatch Explorer -> Worker -> 2 Reviewers + 2 Challengers + 1 Forensic Auditor -> Gate.
  - Goal: Cleanly remove `WEBKIT_DISABLE_COMPOSITING_MODE=1` from `src-tauri/src/lib.rs` and configure clean fallback flags.
- **Milestone 2: Vite & Tauri Production Asset Resolution & CSP Configuration**
  - Dispatch Explorer -> Worker -> 2 Reviewers + 2 Challengers + 1 Forensic Auditor -> Gate.
  - Goal: Ensure `vite.config.ts` asset base paths, Tauri CSP settings, and static assets resolve cleanly inside the AppImage bundle.
- **Milestone 3: AppImage Build & Programmatic GUI Verification**
  - Dispatch Explorer -> Worker (runs `npm run tauri build` and executes programmatic verification script) -> 2 Reviewers + 2 Challengers + 1 Forensic Auditor -> Gate.
  - Goal: Produce `.AppImage` binary and programmatically verify main window loads React UI without WebProcess crash.

### Phase 3: Final Synthesis & Human Reporting
- Verify all audit clean verdicts and gate approvals.
- Prepare report for user.
