//! Cross-desktop capture fallback chains (X11 + Wayland)
//!
//! Every desktop ships a different set of screenshot tools. Instead of one
//! hard-coded path, we probe them in priority order and fall through until one
//! succeeds. Order matters:
//!
//!   Region:
//!     cosmic-screenshot (COSMIC) → spectacle (KDE) → grim+slurp (wlroots) →
//!     org.gnome.Shell.Screenshot SelectArea (GNOME ≥42, works in Flatpak) →
//!     gnome-screenshot (legacy GNOME) → maim/scrot (X11)
//!
//!   Fullscreen:
//!     cosmic-screenshot → spectacle → grim →
//!     org.gnome.Shell.Screenshot (GNOME) → xdg-desktop-portal Screenshot
//!     (universal — works on every desktop with a portal) →
//!     gnome-screenshot → scrot (X11)
//!
//! The D-Bus paths use `zbus`, which is pure-Rust and already in the
//! dependency tree via xcap, so nothing extra is compiled. They are the only
//! fallbacks that work inside the Flatpak sandbox (no grim/slurp/scrot there).

use std::path::Path;
use std::process::{Command, Stdio};

/// Check if we're on Wayland
pub fn is_wayland() -> bool {
    std::env::var("WAYLAND_DISPLAY").is_ok()
        || std::env::var("XDG_SESSION_TYPE")
            .map(|v| v.to_lowercase() == "wayland")
            .unwrap_or(false)
}

/// Check if a binary is on PATH
pub fn has_binary(name: &str) -> bool {
    Command::new("which")
        .arg(name)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Interactively capture a user-selected region into `path`.
pub fn capture_region(path: &Path) -> Result<(), String> {
    if is_wayland() {
        // COSMIC: cosmic-screenshot --interactive=true
        if has_binary("cosmic-screenshot") && cosmic_region(path).is_ok() {
            return Ok(());
        }
        // KDE Plasma: spectacle --region (works on Wayland and X11)
        if has_binary("spectacle") && spectacle_region(path).is_ok() {
            return Ok(());
        }
        // wlroots (sway/hyprland): grim + slurp
        if has_binary("grim") && has_binary("slurp") && grim_slurp_region(path).is_ok() {
            return Ok(());
        }
        // Modern GNOME (42+): interactive area picker over D-Bus
        #[cfg(target_os = "linux")]
        if gnome_shell_region(path).is_ok() {
            return Ok(());
        }
        // Legacy GNOME: gnome-screenshot -a
        if has_binary("gnome-screenshot") {
            let status = Command::new("gnome-screenshot")
                .arg("-a")
                .arg("-f")
                .arg(path)
                .status()
                .map_err(|e| format!("Failed to run gnome-screenshot: {}", e))?;
            if status.success() && path.exists() {
                return Ok(());
            }
        }
        return Err(
            "No screenshot tool available on this Wayland desktop. Install cosmic-screenshot (COSMIC), spectacle (KDE), or grim+slurp (Sway/Hyprland)."
                .to_string(),
        );
    }

    // X11
    if has_binary("spectacle") && spectacle_region(path).is_ok() {
        return Ok(());
    }
    if has_binary("maim") {
        let status = Command::new("maim")
            .arg("-s")
            .arg(path)
            .status()
            .map_err(|e| format!("Failed to run maim: {}", e))?;
        if status.success() && path.exists() {
            return Ok(());
        }
    }
    if has_binary("scrot") {
        let status = Command::new("scrot")
            .arg("-s")
            .arg(path)
            .status()
            .map_err(|e| format!("Failed to run scrot: {}", e))?;
        if status.success() && path.exists() {
            return Ok(());
        }
    }

    Err("No screenshot tool found on this desktop. Install scrot or maim (X11), or spectacle (KDE).".to_string())
}

/// Capture the full screen (or primary monitor) into `path`.
pub fn capture_fullscreen(path: &Path) -> Result<(), String> {
    if is_wayland() {
        if has_binary("cosmic-screenshot") && cosmic_fullscreen(path).is_ok() {
            return Ok(());
        }
        if has_binary("spectacle") && spectacle_fullscreen(path).is_ok() {
            return Ok(());
        }
        if has_binary("grim") {
            let status = Command::new("grim")
                .arg(path)
                .status()
                .map_err(|e| format!("Failed to run grim: {}", e))?;
            if status.success() && path.exists() {
                return Ok(());
            }
        }
        // Modern GNOME (42+): D-Bus Screenshot
        #[cfg(target_os = "linux")]
        if gnome_shell_fullscreen(path).is_ok() {
            return Ok(());
        }
        // Universal: xdg-desktop-portal (works on every desktop, incl. Flatpak)
        #[cfg(target_os = "linux")]
        if portal_fullscreen(path).is_ok() {
            return Ok(());
        }
        // Legacy GNOME
        if has_binary("gnome-screenshot") {
            let status = Command::new("gnome-screenshot")
                .arg("-f")
                .arg(path)
                .status()
                .map_err(|e| format!("Failed to run gnome-screenshot: {}", e))?;
            if status.success() && path.exists() {
                return Ok(());
            }
        }
        return Err(
            "No screenshot tool available on this Wayland desktop. Install cosmic-screenshot (COSMIC), spectacle (KDE), or grim (Sway/Hyprland)."
                .to_string(),
        );
    }

    // X11
    if has_binary("spectacle") && spectacle_fullscreen(path).is_ok() {
        return Ok(());
    }
    if has_binary("scrot") {
        let status = Command::new("scrot")
            .arg(path)
            .status()
            .map_err(|e| format!("Failed to run scrot: {}", e))?;
        if status.success() && path.exists() {
            return Ok(());
        }
    }
    // Universal portal also works on X11 desktops with xdg-desktop-portal
    #[cfg(target_os = "linux")]
    if portal_fullscreen(path).is_ok() {
        return Ok(());
    }
    if has_binary("gnome-screenshot") {
        let status = Command::new("gnome-screenshot")
            .arg("-f")
            .arg(path)
            .status()
            .map_err(|e| format!("Failed to run gnome-screenshot: {}", e))?;
        if status.success() && path.exists() {
            return Ok(());
        }
    }

    Err("No screenshot tool found on this desktop. Install scrot (X11) or spectacle (KDE).".to_string())
}

/// Capture the focused window into `path`.
pub fn capture_window(path: &Path) -> Result<(), String> {
    if is_wayland() {
        // COSMIC has no dedicated window mode — fall back to its region picker
        if has_binary("cosmic-screenshot") {
            return capture_region(path);
        }
        // KDE Plasma: spectacle --window
        if has_binary("spectacle") {
            let status = Command::new("spectacle")
                .arg("--window")
                .arg("-b")
                .arg("-n")
                .arg("-o")
                .arg(path)
                .status()
                .map_err(|e| format!("Failed to run spectacle: {}", e))?;
            if status.success() && path.exists() {
                return Ok(());
            }
        }
        // Modern GNOME (42+): D-Bus ScreenshotWindow
        #[cfg(target_os = "linux")]
        if gnome_shell_window(path).is_ok() {
            return Ok(());
        }
        // Legacy GNOME
        if has_binary("gnome-screenshot") {
            let status = Command::new("gnome-screenshot")
                .arg("-w")
                .arg("-f")
                .arg(path)
                .status()
                .map_err(|e| format!("Failed to run gnome-screenshot: {}", e))?;
            if status.success() && path.exists() {
                return Ok(());
            }
        }
        return Err(
            "No window capture tool available on this Wayland desktop. Install spectacle (KDE) or cosmic-screenshot (COSMIC)."
                .to_string(),
        );
    }

    // X11
    if has_binary("spectacle") {
        let status = Command::new("spectacle")
            .arg("--window")
            .arg("-b")
            .arg("-n")
            .arg("-o")
            .arg(path)
            .status()
            .map_err(|e| format!("Failed to run spectacle: {}", e))?;
        if status.success() && path.exists() {
            return Ok(());
        }
    }
    if has_binary("scrot") {
        let status = Command::new("scrot")
            .arg("-u")
            .arg(path)
            .status()
            .map_err(|e| format!("Failed to run scrot: {}", e))?;
        if status.success() && path.exists() {
            return Ok(());
        }
    }

    Err("No window capture tool found on this desktop. Install spectacle (KDE) or scrot (X11).".to_string())
}

// ── Per-tool helpers ───────────────────────────────────────────────────────

fn cosmic_region(path: &Path) -> Result<(), String> {
    let save_dir = path
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "/tmp".to_string());

    let output = Command::new("cosmic-screenshot")
        .arg("--interactive=true")
        .arg("--modal=false")
        .arg("--notify=false")
        .arg("-s")
        .arg(&save_dir)
        .output()
        .map_err(|e| format!("Failed to run cosmic-screenshot: {}", e))?;

    if !output.status.success() {
        return Err("cosmic-screenshot failed".to_string());
    }

    // cosmic-screenshot prints the saved path to stdout
    let out_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !out_path.is_empty() && Path::new(&out_path).exists() {
        // Move/copy into the requested path so the caller finds its file
        return std::fs::copy(&out_path, path)
            .map(|_| ())
            .map_err(|e| format!("Failed to copy cosmic-screenshot result: {}", e));
    }

    // Fall back to the most recent PNG in save_dir
    if let Ok(entries) = std::fs::read_dir(&save_dir) {
        let mut files: Vec<_> = entries
            .flatten()
            .filter(|e| e.path().extension().map(|x| x == "png").unwrap_or(false))
            .collect();
        files.sort_by_key(|e| e.metadata().and_then(|m| m.modified()).ok());
        if let Some(last) = files.last() {
            return std::fs::copy(last.path(), path)
                .map(|_| ())
                .map_err(|e| format!("Failed to copy cosmic-screenshot result: {}", e));
        }
    }

    Err("cosmic-screenshot completed but no file found".to_string())
}

fn cosmic_fullscreen(path: &Path) -> Result<(), String> {
    let save_dir = path
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "/tmp".to_string());

    let output = Command::new("cosmic-screenshot")
        .arg("--interactive=false")
        .arg("--notify=false")
        .arg("-s")
        .arg(&save_dir)
        .output()
        .map_err(|e| format!("Failed to run cosmic-screenshot: {}", e))?;

    if !output.status.success() {
        return Err("cosmic-screenshot failed".to_string());
    }

    // cosmic-screenshot prints the saved path to stdout
    let out_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !out_path.is_empty() && Path::new(&out_path).exists() {
        return std::fs::copy(&out_path, path)
            .map(|_| ())
            .map_err(|e| format!("Failed to copy cosmic-screenshot result: {}", e));
    }

    // Fall back to the most recent PNG in save_dir
    if let Ok(entries) = std::fs::read_dir(&save_dir) {
        let mut files: Vec<_> = entries
            .flatten()
            .filter(|e| e.path().extension().map(|x| x == "png").unwrap_or(false))
            .collect();
        files.sort_by_key(|e| e.metadata().and_then(|m| m.modified()).ok());
        if let Some(last) = files.last() {
            return std::fs::copy(last.path(), path)
                .map(|_| ())
                .map_err(|e| format!("Failed to copy cosmic-screenshot result: {}", e));
        }
    }

    Err("cosmic-screenshot completed but no file found".to_string())
}

fn spectacle_region(path: &Path) -> Result<(), String> {
    let status = Command::new("spectacle")
        .arg("--region")
        .arg("-b")
        .arg("-n")
        .arg("-o")
        .arg(path)
        .status()
        .map_err(|e| format!("Failed to run spectacle: {}", e))?;

    if status.success() && path.exists() {
        Ok(())
    } else {
        Err("spectacle was cancelled or failed".to_string())
    }
}

fn spectacle_fullscreen(path: &Path) -> Result<(), String> {
    let status = Command::new("spectacle")
        .arg("--fullscreen")
        .arg("-b")
        .arg("-n")
        .arg("-o")
        .arg(path)
        .status()
        .map_err(|e| format!("Failed to run spectacle: {}", e))?;

    if status.success() && path.exists() {
        Ok(())
    } else {
        Err("spectacle failed to capture the screen".to_string())
    }
}

fn grim_slurp_region(path: &Path) -> Result<(), String> {
    let slurp_output = Command::new("slurp")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("Failed to run slurp: {}", e))?;

    if !slurp_output.status.success() {
        return Err("Screenshot was cancelled or failed".to_string());
    }

    let region = String::from_utf8_lossy(&slurp_output.stdout).trim().to_string();
    if region.is_empty() {
        return Err("Screenshot was cancelled or failed".to_string());
    }

    let status = Command::new("grim")
        .arg("-g")
        .arg(&region)
        .arg(path)
        .status()
        .map_err(|e| format!("Failed to run grim: {}", e))?;

    if status.success() && path.exists() {
        Ok(())
    } else {
        Err("grim failed to capture the region".to_string())
    }
}

// ── D-Bus fallbacks (pure-Rust zbus, no external binaries) ─────────────────

/// Capture via the xdg-desktop-portal `org.freedesktop.portal.Screenshot`
/// interface (fullscreen only). Works on GNOME, KDE, COSMIC, wlroots and X11
/// — and inside the Flatpak sandbox, where none of the CLI tools exist.
#[cfg(target_os = "linux")]
fn portal_fullscreen(path: &Path) -> Result<(), String> {
    use std::collections::HashMap;
    use zbus::blocking::{Connection, Proxy};
    use zbus::zvariant::{OwnedObjectPath, OwnedValue, Value};

    let conn = Connection::session().map_err(|e| format!("D-Bus session unavailable: {}", e))?;
    let portal = Proxy::new(
        &conn,
        "org.freedesktop.portal.Desktop",
        "/org/freedesktop/portal/desktop",
        "org.freedesktop.portal.Screenshot",
    )
    .map_err(|e| format!("Cannot reach xdg-desktop-portal: {}", e))?;

    let uri = url::Url::from_file_path(path)
        .map_err(|_| "Invalid save path for portal capture".to_string())?
        .to_string();

    let mut options: HashMap<&str, Value> = HashMap::new();
    options.insert("interactive", Value::from(false));
    options.insert("modal", Value::from(false));
    options.insert("filename", Value::from(uri));

    let reply = portal
        .call_method("Screenshot", &options)
        .map_err(|e| format!("Portal screenshot call failed: {}", e))?;
    let request_path: OwnedObjectPath = reply
        .body()
        .deserialize()
        .map_err(|e| format!("Bad portal reply: {}", e))?;

    // Wait for the Response signal on the request object
    let request = Proxy::new(
        &conn,
        "org.freedesktop.portal.Desktop",
        request_path.as_str(),
        "org.freedesktop.portal.Request",
    )
    .map_err(|e| format!("Cannot watch portal request: {}", e))?;
    let mut signals = request
        .receive_signal("Response")
        .map_err(|e| format!("Cannot subscribe to portal response: {}", e))?;
    let response_msg = signals
        .next()
        .ok_or_else(|| "Portal closed without responding".to_string())?;
    let (response, _results): (u32, HashMap<String, OwnedValue>) = response_msg
        .body()
        .deserialize()
        .map_err(|e| format!("Bad portal response: {}", e))?;

    if response != 0 {
        return Err("Screenshot was cancelled or failed".to_string());
    }
    if !path.exists() {
        return Err("Portal reported success but no file was written".to_string());
    }
    Ok(())
}

/// GNOME Shell exposes an interactive area picker over D-Bus
/// (`org.gnome.Shell` / `/org/gnome/Shell/Screenshot`). This is the only
/// interactive region selection that works on modern GNOME (42+) Wayland and
/// inside the Flatpak sandbox. Returns Err when GNOME Shell is unreachable.
#[cfg(target_os = "linux")]
fn gnome_shell_region(path: &Path) -> Result<(), String> {
    use zbus::blocking::{Connection, Proxy};

    let conn = Connection::session().map_err(|e| format!("D-Bus session unavailable: {}", e))?;
    let shell = Proxy::new(
        &conn,
        "org.gnome.Shell",
        "/org/gnome/Shell/Screenshot",
        "org.gnome.Shell.Screenshot",
    )
    .map_err(|e| format!("GNOME Shell not reachable: {}", e))?;

    // Interactive rubber-band selection; returns logical coordinates
    let reply = shell
        .call_method("SelectArea", &())
        .map_err(|e| format!("GNOME Shell SelectArea failed: {}", e))?;
    let (x, y, width, height): (i32, i32, i32, i32) = reply
        .body()
        .deserialize()
        .map_err(|e| format!("Bad SelectArea reply: {}", e))?;

    if width <= 0 || height <= 0 {
        return Err("Screenshot was cancelled".to_string());
    }

    // Capture exactly the selected area
    let filename = path.to_string_lossy().to_string();
    let reply = shell
        .call_method("ScreenshotArea", &(x, y, width, height, false, filename))
        .map_err(|e| format!("GNOME Shell ScreenshotArea failed: {}", e))?;
    let (ok, _used): (bool, String) = reply
        .body()
        .deserialize()
        .map_err(|e| format!("Bad ScreenshotArea reply: {}", e))?;

    if !ok || !path.exists() {
        return Err("GNOME Shell failed to capture the area".to_string());
    }
    Ok(())
}

/// GNOME Shell fullscreen capture over D-Bus (GNOME 42+, sandbox-friendly).
#[cfg(target_os = "linux")]
fn gnome_shell_fullscreen(path: &Path) -> Result<(), String> {
    use zbus::blocking::{Connection, Proxy};

    let conn = Connection::session().map_err(|e| format!("D-Bus session unavailable: {}", e))?;
    let shell = Proxy::new(
        &conn,
        "org.gnome.Shell",
        "/org/gnome/Shell/Screenshot",
        "org.gnome.Shell.Screenshot",
    )
    .map_err(|e| format!("GNOME Shell not reachable: {}", e))?;

    let filename = path.to_string_lossy().to_string();
    let reply = shell
        .call_method("Screenshot", &(true, false, filename))
        .map_err(|e| format!("GNOME Shell Screenshot failed: {}", e))?;
    let (ok, _used): (bool, String) = reply
        .body()
        .deserialize()
        .map_err(|e| format!("Bad Screenshot reply: {}", e))?;

    if !ok || !path.exists() {
        return Err("GNOME Shell failed to capture the screen".to_string());
    }
    Ok(())
}

/// GNOME Shell focused-window capture over D-Bus (GNOME 42+).
#[cfg(target_os = "linux")]
fn gnome_shell_window(path: &Path) -> Result<(), String> {
    use zbus::blocking::{Connection, Proxy};

    let conn = Connection::session().map_err(|e| format!("D-Bus session unavailable: {}", e))?;
    let shell = Proxy::new(
        &conn,
        "org.gnome.Shell",
        "/org/gnome/Shell/Screenshot",
        "org.gnome.Shell.Screenshot",
    )
    .map_err(|e| format!("GNOME Shell not reachable: {}", e))?;

    let filename = path.to_string_lossy().to_string();
    let reply = shell
        .call_method("ScreenshotWindow", &(true, true, false, filename))
        .map_err(|e| format!("GNOME Shell ScreenshotWindow failed: {}", e))?;
    let (ok, _used): (bool, String) = reply
        .body()
        .deserialize()
        .map_err(|e| format!("Bad ScreenshotWindow reply: {}", e))?;

    if !ok || !path.exists() {
        return Err("GNOME Shell failed to capture the window".to_string());
    }
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn portal_fullscreen(_path: &Path) -> Result<(), String> {
    Err("portal capture is Linux-only".to_string())
}

#[cfg(not(target_os = "linux"))]
fn gnome_shell_region(_path: &Path) -> Result<(), String> {
    Err("GNOME Shell capture is Linux-only".to_string())
}

#[cfg(not(target_os = "linux"))]
fn gnome_shell_fullscreen(_path: &Path) -> Result<(), String> {
    Err("GNOME Shell capture is Linux-only".to_string())
}

#[cfg(not(target_os = "linux"))]
fn gnome_shell_window(_path: &Path) -> Result<(), String> {
    Err("GNOME Shell capture is Linux-only".to_string())
}
