// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // ── Linux GDK backend override ──────────────────────────────────────
    // linuxdeploy-plugin-gtk unconditionally injects
    //   export GDK_BACKEND=x11
    // into every GTK AppImage's AppRun. On pure-Wayland setups (Hyprland,
    // sway, etc.) XWayland does not expose a working EGL display, so GTK's
    // X11 backend aborts during init with:
    //   "Could not create default EGL display: EGL_BAD_PARAMETER. Aborting..."
    //
    // We override the env var here, before any GTK / GDK / webkit2gtk
    // initialization happens. GDK reads GDK_BACKEND lazily on the first
    // display open, so a set_var in main() beats every caller downstream.
    //
    // Reference: tauri-apps/tauri#8541 — the bug the upstream workaround
    // was guarding against no longer reproduces on Tauri v2.
    #[cfg(target_os = "linux")]
    {
        if std::env::var_os("WAYLAND_DISPLAY").is_some() {
            // Respect an explicit user override, otherwise force Wayland.
            if std::env::var_os("GDK_BACKEND").is_none() {
                std::env::set_var("GDK_BACKEND", "wayland");
            }
        }
    }

    framexshot_lib::run()
}
