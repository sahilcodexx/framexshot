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
use crate::utils::{file_to_data_uri, generate_filename, get_desktop_path};

static PENDING_SCREENSHOT_B64: Mutex<Option<String>> = Mutex::new(None);

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
    tauri::async_runtime::spawn_blocking(move || {
        copy_image_to_clipboard(&path).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// Quick capture of primary monitor
#[tauri::command]
pub async fn capture_once(
    app_handle: AppHandle,
    save_dir: String,
    copy_to_clip: bool,
) -> Result<String, String> {
    let screenshot_path = capture_primary_monitor(app_handle).await?;
    tauri::async_runtime::spawn_blocking(move || {
        let screenshot_path_str = screenshot_path.to_string_lossy().to_string();
        let saved_path = copy_screenshot_to_dir(&screenshot_path_str, &save_dir)?;
        if copy_to_clip {
            copy_image_to_clipboard(&saved_path)?;
        }
        Ok(saved_path)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// Capture all monitors with geometry info
#[tauri::command]
pub async fn capture_all_monitors(
    _app_handle: AppHandle,
    save_dir: String,
) -> Result<Vec<MonitorShot>, String> {
    tauri::async_runtime::spawn_blocking(move || capture_monitors(&save_dir))
        .await
        .map_err(|e| format!("Task join error: {}", e))?
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
    tauri::async_runtime::spawn_blocking(move || {
        let region = CropRegion { x, y, width, height };
        crop_image(&screenshot_path, region, &save_dir)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// Render image with effects using Rust (optimized for blur)
#[tauri::command]
pub async fn render_image_with_effects_rust(
    image_path: String,
    settings: RenderSettings,
) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        render_image_with_effects(&image_path, settings)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// Save an edited image from base64 data
#[tauri::command]
pub async fn save_edited_image(
    image_data: String,
    save_dir: String,
    copy_to_clip: bool,
) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let saved_path = save_base64_image(&image_data, &save_dir, "framexshot")?;
        if copy_to_clip {
            copy_image_to_clipboard(&saved_path)?;
        }
        Ok(saved_path)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// Get the user's Desktop directory path
#[tauri::command]
pub async fn get_desktop_directory() -> Result<String, String> {
    get_desktop_path()
}

/// Get the system temp directory path
#[tauri::command]
pub async fn get_temp_directory() -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(|| {
        let _ = crate::utils::cleanup_temp_files();
        let temp_dir = std::env::temp_dir();
        let canonical = temp_dir.canonicalize().unwrap_or(temp_dir);
        canonical
            .to_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "Failed to convert temp directory path to string".to_string())
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// Capture screenshot using Linux native tools with interactive region selection.
/// COSMIC: cosmic-screenshot --interactive=true
/// Sway/Hyprland: grim + slurp
/// X11: scrot -s  (fallback: gnome-screenshot -a or spectacle -r)
#[tauri::command]
pub async fn native_capture_interactive(save_dir: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let _lock = SCREENCAPTURE_LOCK
            .lock()
            .map_err(|e| format!("Failed to acquire lock: {}", e))?;

        if is_wayland() {
            // COSMIC / portal-based Wayland: cosmic-screenshot
            let is_cosmic = std::env::var("XDG_CURRENT_DESKTOP")
                .map(|v| v.to_lowercase().contains("cosmic"))
                .unwrap_or(false)
                || has_binary("cosmic-screenshot");

            if is_cosmic && has_binary("cosmic-screenshot") {
                let start_time = std::time::SystemTime::now() - std::time::Duration::from_secs(30);
                let output = Command::new("cosmic-screenshot")
                    .arg("--interactive=true")
                    .arg("--modal=false")
                    .arg("--notify=false")
                    .arg("-s")
                    .arg(&save_dir)
                    .output()
                    .map_err(|e| format!("Failed to run cosmic-screenshot: {}", e))?;

                if output.status.success() {
                    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !path.is_empty() && std::path::Path::new(&path).exists() {
                        return Ok(path);
                    }

                    // 1. Search in save_dir, system temp, user Pictures & Pictures/Screenshots
                    let pictures_dir = dirs::picture_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
                    let pictures_screenshots = pictures_dir.join("Screenshots");
                    let search_dirs = vec![
                        PathBuf::from(&save_dir),
                        std::env::temp_dir(),
                        pictures_dir,
                        pictures_screenshots,
                    ];

                    for dir in search_dirs {
                        if let Ok(entries) = std::fs::read_dir(&dir) {
                            let mut files: Vec<_> = entries
                                .flatten()
                                .filter(|e| {
                                    let is_png = e.path().extension().map(|x| x == "png" || x == "jpg" || x == "jpeg").unwrap_or(false);
                                    let is_new = e.metadata().and_then(|m| m.modified()).map(|m| m >= start_time).unwrap_or(false);
                                    is_png && is_new
                                })
                                .collect();
                            files.sort_by_key(|e| e.metadata().and_then(|m| m.modified()).ok());
                            if let Some(last) = files.last() {
                                return Ok(last.path().to_string_lossy().into_owned());
                            }
                        }
                    }

                    // 2. If cosmic-screenshot interactive mode put the image on the clipboard, extract it to save_dir!
                    if has_binary("wl-paste") {
                        let clip_out = Command::new("wl-paste")
                            .arg("-t")
                            .arg("image/png")
                            .output();

                        if let Ok(clip) = clip_out {
                            if clip.status.success() && !clip.stdout.is_empty() {
                                let filename = generate_filename("screenshot", "png")?;
                                let screenshot_path = std::path::PathBuf::from(&save_dir).join(&filename);
                                if std::fs::write(&screenshot_path, &clip.stdout).is_ok() {
                                    return Ok(screenshot_path.to_string_lossy().into_owned());
                                }
                            }
                        }
                    }
                }
                
                // Return explicitly on COSMIC so it NEVER falls back to grim/slurp
                return Err("Screenshot was cancelled or failed".to_string());
            }

            // Wayland wlroots (sway/hyprland): grim + slurp
            if has_binary("grim") && has_binary("slurp") {
                let filename = generate_filename("screenshot", "png")?;
                let screenshot_path = std::path::PathBuf::from(&save_dir).join(&filename);
                let path_str = screenshot_path.to_string_lossy().to_string();

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
                let filename = generate_filename("screenshot", "png")?;
                let screenshot_path = std::path::PathBuf::from(&save_dir).join(&filename);
                let path_str = screenshot_path.to_string_lossy().to_string();

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

            return Err(
                "No supported screenshot tool found for Wayland. Install cosmic-screenshot (COSMIC) or grim+slurp (sway/hyprland)."
                    .to_string(),
            );
        }

        // X11 path
        let filename = generate_filename("screenshot", "png")?;
        let screenshot_path = std::path::PathBuf::from(&save_dir).join(&filename);
        let path_str = screenshot_path.to_string_lossy().to_string();

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

        Err("No supported screenshot tool found. Please install scrot (X11) or cosmic-screenshot/grim+slurp (Wayland).".to_string())
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// Capture full screen
/// COSMIC: cosmic-screenshot --interactive=false
/// Sway/Hyprland: grim
/// X11: scrot
#[tauri::command]
pub async fn native_capture_fullscreen(save_dir: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let _lock = SCREENCAPTURE_LOCK
            .lock()
            .map_err(|e| format!("Failed to acquire lock: {}", e))?;

        if is_wayland() {
            // COSMIC portal-based: cosmic-screenshot --interactive=false
            let is_cosmic = std::env::var("XDG_CURRENT_DESKTOP")
                .map(|v| v.to_lowercase().contains("cosmic"))
                .unwrap_or(false)
                || has_binary("cosmic-screenshot");

            if is_cosmic && has_binary("cosmic-screenshot") {
                let start_time = std::time::SystemTime::now() - std::time::Duration::from_secs(30);
                let output = Command::new("cosmic-screenshot")
                    .arg("--interactive=false")
                    .arg("--notify=false")
                    .arg("-s")
                    .arg(&save_dir)
                    .output()
                    .map_err(|e| format!("Failed to run cosmic-screenshot: {}", e))?;

                if output.status.success() {
                    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !path.is_empty() && std::path::Path::new(&path).exists() {
                        return Ok(path);
                    }
                    let pictures_dir = dirs::picture_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
                    let pictures_screenshots = pictures_dir.join("Screenshots");
                    let search_dirs = vec![
                        PathBuf::from(&save_dir),
                        std::env::temp_dir(),
                        pictures_dir,
                        pictures_screenshots,
                    ];

                    for dir in search_dirs {
                        if let Ok(entries) = std::fs::read_dir(&dir) {
                            let mut files: Vec<_> = entries
                                .flatten()
                                .filter(|e| {
                                    let is_png = e.path().extension().map(|x| x == "png" || x == "jpg" || x == "jpeg").unwrap_or(false);
                                    let is_new = e.metadata().and_then(|m| m.modified()).map(|m| m >= start_time).unwrap_or(false);
                                    is_png && is_new
                                })
                                .collect();
                            files.sort_by_key(|e| e.metadata().and_then(|m| m.modified()).ok());
                            if let Some(last) = files.last() {
                                return Ok(last.path().to_string_lossy().into_owned());
                            }
                        }
                    }
                }
                return Err("Screenshot was cancelled or failed".to_string());
            }

            // wlroots (sway/hyprland): grim
            if has_binary("grim") {
                let filename = generate_filename("screenshot", "png")?;
                let screenshot_path = std::path::PathBuf::from(&save_dir).join(&filename);
                let path_str = screenshot_path.to_string_lossy().to_string();

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
                let filename = generate_filename("screenshot", "png")?;
                let screenshot_path = std::path::PathBuf::from(&save_dir).join(&filename);
                let path_str = screenshot_path.to_string_lossy().to_string();

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

            return Err("No supported screenshot tool found for Wayland. Install cosmic-screenshot (COSMIC) or grim (sway/hyprland).".to_string());
        }

        // X11
        let filename = generate_filename("screenshot", "png")?;
        let screenshot_path = std::path::PathBuf::from(&save_dir).join(&filename);
        let path_str = screenshot_path.to_string_lossy().to_string();

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

        Err("No supported screenshot tool found. Please install scrot (X11) or cosmic-screenshot/grim (Wayland).".to_string())
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// Capture a specific window
#[tauri::command]
pub async fn native_capture_window(save_dir: String) -> Result<String, String> {
    if is_wayland() {
        if has_binary("cosmic-screenshot") {
            return native_capture_interactive(save_dir).await;
        }

        if has_binary("gnome-screenshot") {
            return tauri::async_runtime::spawn_blocking(move || {
                let _lock = SCREENCAPTURE_LOCK
                    .lock()
                    .map_err(|e| format!("Failed to acquire lock: {}", e))?;

                let filename = generate_filename("screenshot", "png")?;
                let save_path = PathBuf::from(&save_dir);
                let screenshot_path = save_path.join(&filename);
                let path_str = screenshot_path.to_string_lossy().to_string();

                let status = Command::new("gnome-screenshot")
                    .arg("-w")
                    .arg("-f")
                    .arg(&path_str)
                    .status()
                    .map_err(|e| format!("Failed to run gnome-screenshot: {}", e))?;

                if !status.success() || !screenshot_path.exists() {
                    return Err("Screenshot was cancelled or failed".to_string());
                }
                Ok(path_str)
            })
            .await
            .map_err(|e| format!("Task join error: {}", e))?;
        }
    }

    if !is_wayland() && has_binary("scrot") {
        return tauri::async_runtime::spawn_blocking(move || {
            let _lock = SCREENCAPTURE_LOCK
                .lock()
                .map_err(|e| format!("Failed to acquire lock: {}", e))?;

            let filename = generate_filename("screenshot", "png")?;
            let save_path = PathBuf::from(&save_dir);
            let screenshot_path = save_path.join(&filename);
            let path_str = screenshot_path.to_string_lossy().to_string();

            let status = Command::new("scrot")
                .arg("-u")
                .arg(&path_str)
                .status()
                .map_err(|e| format!("Failed to run scrot: {}", e))?;

            if !status.success() || !screenshot_path.exists() {
                return Err("Screenshot was cancelled or failed".to_string());
            }
            Ok(path_str)
        })
        .await
        .map_err(|e| format!("Task join error: {}", e))?;
    }

    // Fall back to interactive region selection
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
    let screenshot_path = tauri::async_runtime::spawn_blocking(move || {
        let _lock = SCREENCAPTURE_LOCK
            .lock()
            .map_err(|e| format!("Failed to acquire lock: {}", e))?;

        let filename = generate_filename("ocr_temp", "png")?;
        let save_path = PathBuf::from(&save_dir);
        let path = save_path.join(&filename);
        let path_str = path.to_string_lossy().to_string();

        let captured = capture_interactive_inner(&path_str)?;
        if !std::path::Path::new(&captured).exists() {
            return Err("Screenshot was cancelled or failed".to_string());
        }
        Ok::<String, String>(captured)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))??;

    play_screenshot_sound().await.ok();

    let path_clone = screenshot_path.clone();
    let recognized_text = tauri::async_runtime::spawn_blocking(move || {
        let recognized_text = recognize_text_from_image(&path_clone)
            .map_err(|e| format!("OCR failed: {}", e))?;

        copy_text_to_clipboard(&recognized_text)
            .map_err(|e| format!("Failed to copy text to clipboard: {}", e))?;

        let _ = std::fs::remove_file(&path_clone);
        Ok::<String, String>(recognized_text)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))??;

    Ok(recognized_text)
}

/// Inner capture logic (no mutex, called when lock is already held)
fn capture_interactive_inner(path_str: &str) -> Result<String, String> {
    let save_dir = std::path::Path::new(path_str)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "/tmp".to_string());

    if is_wayland() {
        // COSMIC portal-based: cosmic-screenshot
        if has_binary("cosmic-screenshot") {
            let output = Command::new("cosmic-screenshot")
                .arg("--interactive=true")
                .arg("--modal=false")
                .arg("--notify=false")
                .arg(format!("--save-dir={}", save_dir))
                .output()
                .map_err(|e| format!("Failed to run cosmic-screenshot: {}", e))?;

            if output.status.success() {
                let out_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !out_path.is_empty() && std::path::Path::new(&out_path).exists() {
                    return Ok(out_path);
                }
                // Find most recent PNG in save_dir
                if let Ok(entries) = std::fs::read_dir(&save_dir) {
                    let mut files: Vec<_> = entries
                        .flatten()
                        .filter(|e| e.path().extension().map(|x| x == "png").unwrap_or(false))
                        .collect();
                    files.sort_by_key(|e| e.metadata().and_then(|m| m.modified()).ok());
                    if let Some(last) = files.last() {
                        return Ok(last.path().to_string_lossy().into_owned());
                    }
                }
                return Err("cosmic-screenshot completed but no file found".to_string());
            }
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("cancel") || stderr.contains("dismiss") {
                return Err("Screenshot was cancelled or failed".to_string());
            }
        }

        // wlroots (sway/hyprland): grim + slurp
        if has_binary("grim") && has_binary("slurp") {
            let screenshot_path = std::path::PathBuf::from(path_str);

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

        return Err("No supported screenshot tool found for Wayland. Install cosmic-screenshot (COSMIC) or grim+slurp (sway/hyprland).".to_string());
    }

    // X11: scrot
    if has_binary("scrot") {
        let screenshot_path = std::path::PathBuf::from(path_str);

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

/// Native folder selection dialog command for Linux / macOS / Windows
#[tauri::command]
pub async fn select_folder_dialog(default_path: Option<String>) -> Result<Option<String>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        // 1. Try zenity --file-selection --directory if available on Linux GTK/GNOME/Hyprland
        if has_binary("zenity") {
            let mut cmd = Command::new("zenity");
            cmd.arg("--file-selection")
                .arg("--directory")
                .arg("--title=Select Save Directory");

            if let Some(ref path) = default_path {
                let trimmed = path.trim();
                if !trimmed.is_empty() {
                    cmd.arg(format!("--filename={}", trimmed));
                }
            }

            if let Ok(output) = cmd.output() {
                if output.status.success() {
                    let selected = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !selected.is_empty() {
                        return Ok(Some(selected));
                    }
                }
            }
        }

        // 2. Try kdialog --getexistingdirectory on KDE Plasma
        if has_binary("kdialog") {
            let mut cmd = Command::new("kdialog");
            cmd.arg("--getexistingdirectory");

            if let Some(ref path) = default_path {
                let trimmed = path.trim();
                if !trimmed.is_empty() {
                    cmd.arg(trimmed);
                }
            }

            if let Ok(output) = cmd.output() {
                if output.status.success() {
                    let selected = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !selected.is_empty() {
                        return Ok(Some(selected));
                    }
                }
            }
        }

        // 3. Try python3 / tkinter dialog fallback
        if has_binary("python3") {
            let initial_dir_py = default_path
                .as_ref()
                .map(|p| format!("initialdir='{}'", p.replace('\'', "\\'")))
                .unwrap_or_default();

            let script = format!(
                "import tkinter as tk, sys, os; from tkinter import filedialog; root = tk.Tk(); root.withdraw(); root.attributes('-topmost', True); path = filedialog.askdirectory({}); print(path if path else '')",
                initial_dir_py
            );

            if let Ok(output) = Command::new("python3").arg("-c").arg(script).output() {
                if output.status.success() {
                    let selected = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !selected.is_empty() {
                        return Ok(Some(selected));
                    }
                }
            }
        }

        Err("No native file dialog binary found (zenity, kdialog, or python3). Please install zenity.".to_string())
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// Read a file and return it as a base64 data URI.
/// Exposed as a command so the frontend can load arbitrary image files
/// without going through Tauri's asset protocol on Windows and Linux.
#[tauri::command]
pub async fn read_file_as_base64(path: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || file_to_data_uri(&path))
        .await
        .map_err(|e| format!("Task join error: {}", e))?
}

/// Returns a CLONE of the stored screenshot for the selector overlay to display.
#[tauri::command]
pub async fn capture_screen_for_selector(app_handle: AppHandle) -> Result<String, String> {
    {
        let lock = PENDING_SCREENSHOT_B64.lock().map_err(|e| format!("Mutex: {}", e))?;
        if let Some(ref data) = *lock {
            return Ok(data.clone());
        }
    }

    // Fallback if no stored screenshot
    let path = capture_primary_monitor(app_handle).await?;
    tauri::async_runtime::spawn_blocking(move || {
        let data_uri = file_to_data_uri(&path.to_string_lossy())?;
        let _ = std::fs::remove_file(&path);

        {
            let mut lock = PENDING_SCREENSHOT_B64.lock().map_err(|e| format!("Mutex: {}", e))?;
            *lock = Some(data_uri.clone());
        }

        Ok(data_uri)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// Crops the stored screenshot and saves to disk.
/// Returns a data URI for the editor to display directly.
#[tauri::command]
pub async fn crop_and_save_region(
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    save_dir: String,
) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        if width == 0 || height == 0 {
            return Err("Invalid region: width/height must be > 0".to_string());
        }

        let data_uri = {
            let mut lock = PENDING_SCREENSHOT_B64.lock().map_err(|e| format!("Mutex: {}", e))?;
            lock.take().ok_or("No pending screenshot — call interactive capture first")?
        };

        use base64::{engine::general_purpose, Engine as _};
        use crate::utils::ensure_dir;
        use std::io::Cursor;

        let raw = data_uri.splitn(2, ',').nth(1).ok_or("Malformed base64 data URI")?;
        let bytes = general_purpose::STANDARD.decode(raw).map_err(|e| format!("Base64 decode failed: {}", e))?;
        let img = image::load_from_memory(&bytes).map_err(|e| format!("Failed to decode image: {}", e))?;

        let iw = img.width();
        let ih = img.height();

        let cx = (x.max(0) as u32).min(iw.saturating_sub(1));
        let cy = (y.max(0) as u32).min(ih.saturating_sub(1));
        let cw = width.min(iw.saturating_sub(cx));
        let ch = height.min(ih.saturating_sub(cy));

        if cw == 0 || ch == 0 {
            return Err(format!("Region ({},{} {}x{}) is outside bounds ({}x{})", x, y, width, height, iw, ih));
        }

        let cropped = img.crop_imm(cx, cy, cw, ch);

        let dest_dir = PathBuf::from(&save_dir);
        ensure_dir(&dest_dir)?;
        let fname = generate_filename("screenshot", "png")?;
        let out = dest_dir.join(&fname);

        let mut bytes_buf = Vec::new();
        cropped.write_to(&mut Cursor::new(&mut bytes_buf), image::ImageFormat::Png)
            .map_err(|e| format!("Failed to encode cropped image: {}", e))?;

        std::fs::write(&out, &bytes_buf).map_err(|e| format!("Failed to save: {}", e))?;

        Ok(format!("data:image/png;base64,{}", general_purpose::STANDARD.encode(&bytes_buf)))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}
