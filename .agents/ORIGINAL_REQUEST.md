# Original User Request

## Initial Request — 2026-08-04T17:09:55Z

You are the Project Orchestrator (teamwork_preview_orchestrator).
Your working directory is `/home/sahilcodex/Documents/bettershot-linux-main/.agents/orchestrator`.
The project root is `/home/sahilcodex/Documents/bettershot-linux-main`.
The original user request is located at `/home/sahilcodex/Documents/bettershot-linux-main/.agents/ORIGINAL_REQUEST.md`.

Mission & Requirements:
1. Fix WebKit Environment Variables & WebProcess Startup Crash:
   Remove `WEBKIT_DISABLE_COMPOSITING_MODE=1` from `src-tauri/src/lib.rs` (which causes `Could not create default EGL display: EGL_BAD_PARAMETER. Aborting...` in WebKit2GTK 2.42+). Configure fallback WebKit flags cleanly for Linux AppImage compatibility.
2. Verify Vite & Tauri Production Bundling Configuration:
   Ensure `vite.config.ts` asset base paths (`base: './'`), Tauri CSP settings in `tauri.conf.json`, and static assets resolve correctly inside the AppImage bundle without missing file errors.
3. Build and Test Production AppImage Release:
   Run `npm run tauri build` to generate the AppImage, execute the binary, and programmatically verify that the main window loads the React UI successfully without a blank screen or WebProcess crash.

Please create `plan.md` and `progress.md` in your working directory `/home/sahilcodex/Documents/bettershot-linux-main/.agents/orchestrator`, coordinate team execution, verify build and app startup, and report when finished.
