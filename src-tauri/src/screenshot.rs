//! Screenshot capture module — Linux implementation using xcap

use serde::Serialize;
use std::path::PathBuf;
use xcap::Monitor;

use crate::utils::{ensure_dir, generate_filename, generate_filename_with_id, AppResult};

/// Represents a captured monitor screenshot with geometry info
#[derive(Serialize, Clone, Debug)]
pub struct MonitorShot {
    pub id: u32,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub scale_factor: f32,
    pub path: String,
}

/// Capture screenshots of all available monitors
pub fn capture_all_monitors(save_dir: &str) -> AppResult<Vec<MonitorShot>> {
    let monitors = Monitor::all().map_err(|e| format!("Failed to get monitors: {}", e))?;

    if monitors.is_empty() {
        return Err("No monitors available".into());
    }

    let save_path = PathBuf::from(save_dir);
    ensure_dir(&save_path)?;

    let mut shots = Vec::with_capacity(monitors.len());
    for monitor in monitors {
        let shot = capture_single_monitor(&monitor, &save_path)?;
        shots.push(shot);
    }

    Ok(shots)
}

/// Capture a single monitor screenshot
fn capture_single_monitor(monitor: &Monitor, save_path: &PathBuf) -> AppResult<MonitorShot> {
    let monitor_id = monitor
        .id()
        .map_err(|e| format!("Failed to get monitor id: {}", e))?;

    let image = monitor
        .capture_image()
        .map_err(|e| format!("Failed to capture monitor {}: {}", monitor_id, e))?;

    let filename = generate_filename_with_id("monitor", monitor_id, "png")?;
    let screenshot_path = save_path.join(&filename);

    image
        .save(&screenshot_path)
        .map_err(|e| format!("Failed to save screenshot: {}", e))?;

    let x = monitor.x().map_err(|e| format!("Failed to get monitor x: {}", e))?;
    let y = monitor.y().map_err(|e| format!("Failed to get monitor y: {}", e))?;
    let width = monitor.width().map_err(|e| format!("Failed to get monitor width: {}", e))?;
    let height = monitor.height().map_err(|e| format!("Failed to get monitor height: {}", e))?;
    let scale_factor = monitor.scale_factor().map_err(|e| format!("Failed to get scale factor: {}", e))?;

    Ok(MonitorShot {
        id: monitor_id,
        x,
        y,
        width,
        height,
        scale_factor,
        path: screenshot_path.to_string_lossy().into_owned(),
    })
}

/// Capture primary monitor using xcap (works on Linux X11 and Wayland via xdpyinfo/pipewire)
pub async fn capture_primary_monitor(_app_handle: tauri::AppHandle) -> AppResult<PathBuf> {
    let monitors = Monitor::all().map_err(|e| format!("Failed to get monitors: {}", e))?;

    if monitors.is_empty() {
        return Err("No monitors available".into());
    }

    // Pick the monitor marked as primary, fallback to first
    let primary = monitors
        .iter()
        .find(|m| m.is_primary().unwrap_or(false))
        .or_else(|| monitors.first())
        .ok_or("No monitor found")?;

    let temp_dir = std::env::temp_dir();
    ensure_dir(&temp_dir)?;

    let image = primary
        .capture_image()
        .map_err(|e| format!("Failed to capture primary monitor: {}", e))?;

    let filename = generate_filename("screenshot", "png")?;
    let screenshot_path = temp_dir.join(filename);

    image
        .save(&screenshot_path)
        .map_err(|e| format!("Failed to save screenshot: {}", e))?;

    Ok(screenshot_path)
}

/// Capture primary monitor to save_dir (Windows/macOS fallback using xcap)
#[cfg(not(target_os = "linux"))]
pub fn capture_primary(save_dir: &str) -> AppResult<String> {
    use crate::utils::resolve_path;

    let monitors = Monitor::all().map_err(|e| format!("Failed to get monitors: {}", e))?;

    if monitors.is_empty() {
        return Err("No monitors available".into());
    }

    let primary = monitors
        .iter()
        .find(|m| m.is_primary().unwrap_or(false))
        .or_else(|| monitors.first())
        .ok_or("No monitor found")?;

    let save_path = PathBuf::from(save_dir);
    ensure_dir(&save_path)?;

    let image = primary
        .capture_image()
        .map_err(|e| format!("Failed to capture primary monitor: {}", e))?;

    let filename = generate_filename("screenshot", "png")?;
    let screenshot_path = save_path.join(&filename);

    image
        .save(&screenshot_path)
        .map_err(|e| format!("Failed to save screenshot: {}", e))?;

    Ok(resolve_path(&screenshot_path.to_string_lossy()))
}

