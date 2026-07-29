//! Tauri commands module — Linux implementation

use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

use crate::clipboard::{copy_image_to_clipboard, copy_text_to_clipboard};
use crate::image::{
    copy_screenshot_to_dir, crop_image, render_image_with_effects, save_base64_image, CropRegion,
    RenderSettings,
};
use crate::ocr::recognize_text_from_image;
use crate::screenshot::{capture_all_monitors as capture_monitors, capture_primary_monitor, MonitorShot};
use crate::utils::{generate_filename, get_desktop_path};

static SCREENCAPTURE_LOCK: Mutex<()> = Mutex::new(());

/// Detect whether we're running under Wayland or X11
fn is_wayland() -> bool {
    std::env::var("WAYLAND_DISPLAY").is_ok()
        || std::env::var("XDG_SESSION_TYPE")
            .map(|v| v.to_lowercase() == "wayland")
            .unwrap_or(false)
}

/// Check if a binary is available on PATH
fn has_binary(name: &str) -> bool {
    Command::new("which")
        .arg(name)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[tauri::command]
pub async fn move_window_to_active_space(_app_handle: AppHandle) -> Result<(), String> {
    // No-op on Linux — spaces/virtual desktops are managed by the compositor
    Ok(())
}

#[tauri::command]
pub async fn copy_image_file_to_clipboard(path: String) -> Result<(), String> {
    copy_image_to_clipboard(&path).map_err(|e| e.to_string())
}

/// Quick capture of primary monitor
#[tauri::command]
pub async fn capture_once(
    app_handle: AppHandle,
    save_dir: String,
    copy_to_clip: bool,
) -> Result<String, String> {
    let screenshot_path = capture_primary_monitor(app_handle).await?;
    let screenshot_path_str = screenshot_path.to_string_lossy().to_string();
    let saved_path = copy_screenshot_to_dir(&screenshot_path_str, &save_dir)?;
    if copy_to_clip {
        copy_image_to_clipboard(&saved_path)?;
    }
    Ok(saved_path)
}

/// Capture all monitors with geometry info
#[tauri::command]
pub async fn capture_all_monitors(
    _app_handle: AppHandle,
    save_dir: String,
) -> Result<Vec<MonitorShot>, String> {
    capture_monitors(&save_dir)
}

/// Crop a region from a screenshot
#[tauri::command]
pub async fn capture_region(
    screenshot_path: String,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    save_dir: String,
) -> Result<String, String> {
    let region = CropRegion { x, y, width, height };
    crop_image(&screenshot_path, region, &save_dir)
}

/// Render image with effects using Rust (optimized for blur)
#[tauri::command]
pub async fn render_image_with_effects_rust(
    image_path: String,
    settings: RenderSettings,
) -> Result<String, String> {
    render_image_with_effects(&image_path, settings)
}

/// Save an edited image from base64 data
#[tauri::command]
pub async fn save_edited_image(
    image_data: String,
    save_dir: String,
    copy_to_clip: bool,
) -> Result<String, String> {
    let saved_path = save_base64_image(&image_data, &save_dir, "framexshot")?;
    if copy_to_clip {
        copy_image_to_clipboard(&saved_path)?;
    }
    Ok(saved_path)
}

/// Get the user's Desktop directory path
#[tauri::command]
pub async fn get_desktop_directory() -> Result<String, String> {
    get_desktop_path()
}

/// Get the system temp directory path
#[tauri::command]
pub async fn get_temp_directory() -> Result<String, String> {
    let temp_dir = std::env::temp_dir();
    let canonical = temp_dir.canonicalize().unwrap_or(temp_dir);
    canonical
        .to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "Failed to convert temp directory path to string".to_string())
}

/// Capture screenshot using Linux native tools with interactive region selection.
/// Wayland: uses grim + slurp
/// X11: uses scrot -s  (fallback: gnome-screenshot -a or spectacle -r)
#[tauri::command]
pub async fn native_capture_interactive(save_dir: String) -> Result<String, String> {
    let _lock = SCREENCAPTURE_LOCK
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    let filename = generate_filename("screenshot", "png")?;
    let save_path = PathBuf::from(&save_dir);
    let screenshot_path = save_path.join(&filename);
    let path_str = screenshot_path.to_string_lossy().to_string();

    if is_wayland() {
        // Wayland: grim + slurp
        if has_binary("grim") && has_binary("slurp") {
            // Get region selection via slurp
            let slurp_output = Command::new("slurp")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .map_err(|e| format!("Failed to run slurp: {}", e))?;

            if !slurp_output.status.success() {
                return Err("Screenshot was cancelled or failed".to_string());
            }

            let region = String::from_utf8_lossy(&slurp_output.stdout)
                .trim()
                .to_string();

            if region.is_empty() {
                return Err("Screenshot was cancelled or failed".to_string());
            }

            let status = Command::new("grim")
                .arg("-g")
                .arg(&region)
                .arg(&path_str)
                .status()
                .map_err(|e| format!("Failed to run grim: {}", e))?;

            if !status.success() || !screenshot_path.exists() {
                return Err("Screenshot failed".to_string());
            }

            return Ok(path_str);
        }

        // Wayland fallback: gnome-screenshot
        if has_binary("gnome-screenshot") {
            let status = Command::new("gnome-screenshot")
                .arg("-a")
                .arg("-f")
                .arg(&path_str)
                .status()
                .map_err(|e| format!("Failed to run gnome-screenshot: {}", e))?;

            if !status.success() || !screenshot_path.exists() {
                return Err("Screenshot was cancelled or failed".to_string());
            }
            return Ok(path_str);
        }

        return Err("No supported screenshot tool found for Wayland. Please install grim and slurp.".to_string());
    }

    // X11 path
    if has_binary("scrot") {
        let status = Command::new("scrot")
            .arg("-s")
            .arg(&path_str)
            .status()
            .map_err(|e| format!("Failed to run scrot: {}", e))?;

        if !status.success() || !screenshot_path.exists() {
            return Err("Screenshot was cancelled or failed".to_string());
        }
        return Ok(path_str);
    }

    if has_binary("gnome-screenshot") {
        let status = Command::new("gnome-screenshot")
            .arg("-a")
            .arg("-f")
            .arg(&path_str)
            .status()
            .map_err(|e| format!("Failed to run gnome-screenshot: {}", e))?;

        if !status.success() || !screenshot_path.exists() {
            return Err("Screenshot was cancelled or failed".to_string());
        }
        return Ok(path_str);
    }

    if has_binary("spectacle") {
        let status = Command::new("spectacle")
            .arg("-r")
            .arg("-b")
            .arg("-n")
            .arg("-o")
            .arg(&path_str)
            .status()
            .map_err(|e| format!("Failed to run spectacle: {}", e))?;

        if !status.success() || !screenshot_path.exists() {
            return Err("Screenshot was cancelled or failed".to_string());
        }
        return Ok(path_str);
    }

    Err("No supported screenshot tool found. Please install scrot (X11) or grim+slurp (Wayland).".to_string())
}

/// Capture full screen
#[tauri::command]
pub async fn native_capture_fullscreen(save_dir: String) -> Result<String, String> {
    let _lock = SCREENCAPTURE_LOCK
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    let filename = generate_filename("screenshot", "png")?;
    let save_path = PathBuf::from(&save_dir);
    let screenshot_path = save_path.join(&filename);
    let path_str = screenshot_path.to_string_lossy().to_string();

    if is_wayland() {
        if has_binary("grim") {
            let status = Command::new("grim")
                .arg(&path_str)
                .status()
                .map_err(|e| format!("Failed to run grim: {}", e))?;

            if !status.success() || !screenshot_path.exists() {
                return Err("Screenshot failed".to_string());
            }
            return Ok(path_str);
        }

        if has_binary("gnome-screenshot") {
            let status = Command::new("gnome-screenshot")
                .arg("-f")
                .arg(&path_str)
                .status()
                .map_err(|e| format!("Failed to run gnome-screenshot: {}", e))?;

            if !status.success() || !screenshot_path.exists() {
                return Err("Screenshot failed".to_string());
            }
            return Ok(path_str);
        }

        return Err("No supported screenshot tool found for Wayland. Please install grim.".to_string());
    }

    // X11
    if has_binary("scrot") {
        let status = Command::new("scrot")
            .arg(&path_str)
            .status()
            .map_err(|e| format!("Failed to run scrot: {}", e))?;

        if !status.success() || !screenshot_path.exists() {
            return Err("Screenshot failed".to_string());
        }
        return Ok(path_str);
    }

    if has_binary("gnome-screenshot") {
        let status = Command::new("gnome-screenshot")
            .arg("-f")
            .arg(&path_str)
            .status()
            .map_err(|e| format!("Failed to run gnome-screenshot: {}", e))?;

        if !status.success() || !screenshot_path.exists() {
            return Err("Screenshot failed".to_string());
        }
        return Ok(path_str);
    }

    Err("No supported screenshot tool found. Please install scrot (X11) or grim (Wayland).".to_string())
}

/// Capture a specific window
#[tauri::command]
pub async fn native_capture_window(save_dir: String) -> Result<String, String> {
    let _lock = SCREENCAPTURE_LOCK
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    let filename = generate_filename("screenshot", "png")?;
    let save_path = PathBuf::from(&save_dir);
    let screenshot_path = save_path.join(&filename);
    let path_str = screenshot_path.to_string_lossy().to_string();

    // X11: scrot -s (click to select window)
    if !is_wayland() && has_binary("scrot") {
        let status = Command::new("scrot")
            .arg("-s")
            .arg(&path_str)
            .status()
            .map_err(|e| format!("Failed to run scrot: {}", e))?;

        if !status.success() || !screenshot_path.exists() {
            return Err("Screenshot was cancelled or failed".to_string());
        }
        return Ok(path_str);
    }

    // Wayland: no reliable window capture without compositor protocol support
    // Fall back to interactive region selection
    // Drop the lock BEFORE awaiting (std::sync::MutexGuard is not Send)
    drop(_lock);
    native_capture_interactive(save_dir).await
}

/// Play screenshot sound using paplay / aplay
#[tauri::command]
pub async fn play_screenshot_sound() -> Result<(), String> {
    std::thread::spawn(|| {
        // Try common Linux screenshot sounds
        let sound_paths = [
            "/usr/share/sounds/freedesktop/stereo/screen-capture.oga",
            "/usr/share/sounds/gnome/default/alerts/glass.ogg",
            "/usr/share/sounds/ubuntu/stereo/screen-capture.ogg",
        ];

        for sound_path in &sound_paths {
            if std::path::Path::new(sound_path).exists() {
                if Command::new("paplay")
                    .arg(sound_path)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false)
                {
                    return;
                }
                // fallback: aplay
                let _ = Command::new("aplay")
                    .arg(sound_path)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn();
                return;
            }
        }
    });
    Ok(())
}

/// Get the current mouse cursor position
#[tauri::command]
pub async fn get_mouse_position() -> Result<(f64, f64), String> {
    // Use xdotool on X11
    if !is_wayland() && has_binary("xdotool") {
        let output = Command::new("xdotool")
            .arg("getmouselocation")
            .arg("--shell")
            .output()
            .map_err(|e| format!("Failed to get mouse position: {}", e))?;

        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            let mut x = 0.0f64;
            let mut y = 0.0f64;
            for line in text.lines() {
                if let Some(val) = line.strip_prefix("X=") {
                    x = val.trim().parse().unwrap_or(0.0);
                }
                if let Some(val) = line.strip_prefix("Y=") {
                    y = val.trim().parse().unwrap_or(0.0);
                }
            }
            return Ok((x, y));
        }
    }

    // Wayland: no reliable cross-compositor way; return (0,0) — window will center
    Ok((0.0, 0.0))
}

/// Capture region and perform OCR, copying text to clipboard
#[tauri::command]
pub async fn native_capture_ocr_region(save_dir: String) -> Result<String, String> {
    let screenshot_path = {
        let _lock = SCREENCAPTURE_LOCK
            .lock()
            .map_err(|e| format!("Failed to acquire lock: {}", e))?;

        let filename = generate_filename("ocr_temp", "png")?;
        let save_path = PathBuf::from(&save_dir);
        let path = save_path.join(&filename);
        let path_str = path.to_string_lossy().to_string();

        // Reuse interactive capture logic (without the lock, already held)
        let captured = capture_interactive_inner(&path_str)?;
        if !std::path::Path::new(&captured).exists() {
            return Err("Screenshot was cancelled or failed".to_string());
        }
        captured
    };

    play_screenshot_sound().await.ok();

    let recognized_text = recognize_text_from_image(&screenshot_path)
        .map_err(|e| format!("OCR failed: {}", e))?;

    copy_text_to_clipboard(&recognized_text)
        .map_err(|e| format!("Failed to copy text to clipboard: {}", e))?;

    let _ = std::fs::remove_file(&screenshot_path);
    Ok(recognized_text)
}

/// Inner capture logic (no mutex, called when lock is already held)
fn capture_interactive_inner(path_str: &str) -> Result<String, String> {
    let screenshot_path = PathBuf::from(path_str);

    if is_wayland() && has_binary("grim") && has_binary("slurp") {
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
            .arg(path_str)
            .status()
            .map_err(|e| format!("Failed to run grim: {}", e))?;

        if !status.success() || !screenshot_path.exists() {
            return Err("Screenshot failed".to_string());
        }
        return Ok(path_str.to_string());
    }

    if has_binary("scrot") {
        let status = Command::new("scrot")
            .arg("-s")
            .arg(path_str)
            .status()
            .map_err(|e| format!("Failed to run scrot: {}", e))?;

        if !status.success() || !screenshot_path.exists() {
            return Err("Screenshot was cancelled or failed".to_string());
        }
        return Ok(path_str.to_string());
    }

    Err("No supported screenshot tool found.".to_string())
}

#[derive(Clone, serde::Serialize)]
pub struct OverlayPayload {
    pub path: String,
}

#[tauri::command]
pub async fn show_quick_overlay(
    app: tauri::AppHandle,
    screenshot_path: String,
    mouse_x: Option<f64>,
    mouse_y: Option<f64>,
) -> Result<(), String> {
    use tauri::Emitter;

    // Get or create the quick-overlay window
    let overlay = if let Some(win) = app.get_webview_window("quick-overlay") {
        win
    } else {
        tauri::WebviewWindowBuilder::new(
            &app,
            "quick-overlay",
            tauri::WebviewUrl::App("index.html?overlay=1".into()),
        )
        .title("FrameXShot – Quick Overlay")
        .inner_size(360.0, 240.0)
        .resizable(true)
        .decorations(true)
        .visible(false)
        .build()
        .map_err(|e| e.to_string())?
    };

    // Calculate position
    let overlay_width = 360.0;
    let overlay_height = 240.0;
    let margin = 16.0;

    let mut target_x = 0.0;
    let mut target_y = 0.0;

    if let Ok(monitors) = app.available_monitors() {
        let mut target_monitor = monitors.first().cloned();

        if let (Some(mx), Some(my)) = (mouse_x, mouse_y) {
            if let Some(m) = monitors.iter().find(|m| {
                let pos = m.position();
                let size = m.size();
                mx >= pos.x as f64
                    && mx < (pos.x as f64 + size.width as f64)
                    && my >= pos.y as f64
                    && my < (pos.y as f64 + size.height as f64)
            }) {
                target_monitor = Some(m.clone());
            }
        }

        if let Some(monitor) = target_monitor {
            let scale_factor = monitor.scale_factor();
            let physical_width = overlay_width * scale_factor;
            let physical_height = overlay_height * scale_factor;
            let physical_margin = margin * scale_factor;

            target_x = monitor.position().x as f64
                + monitor.size().width as f64
                - physical_width
                - physical_margin;

            target_y = monitor.position().y as f64
                + monitor.size().height as f64
                - physical_height
                - physical_margin;
        }
    }

    let _ = overlay.set_size(tauri::Size::Logical(tauri::LogicalSize::new(overlay_width, overlay_height)));
    let _ = overlay.set_position(tauri::Position::Physical(tauri::PhysicalPosition::new(target_x as i32, target_y as i32)));
    let _ = overlay.set_always_on_top(true);
    let _ = overlay.show();
    let _ = overlay.set_focus();

    // Emit event in case the window already exists and is listening
    let _ = app.emit("overlay-show-capture", OverlayPayload { path: screenshot_path });

    Ok(())
}

