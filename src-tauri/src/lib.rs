//! FrameXShot — Linux backend

mod clipboard;
mod commands;
mod image;
mod ocr;
mod screenshot;
mod utils;

use commands::{
    capture_all_monitors, capture_once, capture_region, capture_screen_for_selector, copy_image_file_to_clipboard,
    crop_and_save_region, get_desktop_directory, get_mouse_position, get_temp_directory, move_window_to_active_space,
    native_capture_fullscreen, native_capture_interactive, native_capture_ocr_region,
    native_capture_window, play_screenshot_sound, read_file_as_base64, render_image_with_effects_rust, save_edited_image,
    select_folder_dialog, show_quick_overlay,
};

use tauri::{Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

fn show_main_window(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    } else {
        let window = WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
            .title("FrameXShot")
            .inner_size(1200.0, 800.0)
            .min_inner_size(800.0, 600.0)
            .center()
            .resizable(true)
            .decorations(false)
            .build()?;

        let window_clone = window.clone();
        window.on_window_event(move |event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if let Err(e) = window_clone.hide() {
                    eprintln!("Failed to hide window: {}", e);
                }
                api.prevent_close();
            }
        });
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(target_os = "linux")]
    {
        if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
            std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = show_main_window(app);
        }))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--hidden"]),
        ))
        .setup(|app| {
            use tauri::menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem};

            // Enable autostart by default
            {
                use tauri_plugin_autostart::ManagerExt;
                let autostart_manager = app.autolaunch();
                if !autostart_manager.is_enabled().unwrap_or(false) {
                    let _ = autostart_manager.enable();
                }
            }

            // Check CLI arguments for Hyprland / Wayland native keybindings
            let args: Vec<String> = std::env::args().collect();
            let app_handle = app.handle().clone();

            let is_hidden = args.iter().any(|arg| arg == "--hidden");
            let is_cli_capture = args.iter().any(|arg| {
                arg == "--capture-region" || arg == "-r" ||
                arg == "--capture-screen" || arg == "-s" ||
                arg == "--capture-window" || arg == "-w" ||
                arg == "--capture-ocr"    || arg == "-o"
            });

            if args.iter().any(|arg| arg == "--capture-region" || arg == "-r") {
                let _ = show_main_window(&app_handle);
                let _ = app_handle.emit("capture-triggered", ());
            } else if args.iter().any(|arg| arg == "--capture-screen" || arg == "-s") {
                let _ = show_main_window(&app_handle);
                let _ = app_handle.emit("capture-fullscreen", ());
            } else if args.iter().any(|arg| arg == "--capture-window" || arg == "-w") {
                let _ = show_main_window(&app_handle);
                let _ = app_handle.emit("capture-window", ());
            } else if args.iter().any(|arg| arg == "--capture-ocr" || arg == "-o") {
                let _ = show_main_window(&app_handle);
                let _ = app_handle.emit("capture-ocr", ());
            }

            // Create main window — visible unless --hidden or CLI capture is passed
            let window =
                WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
                    .title("FrameXShot")
                    .inner_size(1200.0, 800.0)
                    .min_inner_size(800.0, 600.0)
                    .center()
                    .resizable(true)
                    .decorations(false)
                    .visible(!is_hidden && !is_cli_capture)
                    .build()?;

            let window_clone = window.clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    if let Err(e) = window_clone.hide() {
                        eprintln!("Failed to hide window: {}", e);
                    }
                    api.prevent_close();
                }
            });

            // Tray menu
            let open_item = MenuItemBuilder::with_id("open", "Open FrameXShot").build(app)?;
            let capture_region_item =
                MenuItemBuilder::with_id("capture_region", "Capture Region").build(app)?;
            let capture_screen_item =
                MenuItemBuilder::with_id("capture_screen", "Capture Screen").build(app)?;
            let capture_window_item =
                MenuItemBuilder::with_id("capture_window", "Capture Window").build(app)?;
            let capture_ocr_item =
                MenuItemBuilder::with_id("capture_ocr", "OCR Region").build(app)?;
            let preferences_item = MenuItemBuilder::with_id("preferences", "Preferences...")
                .accelerator("CommandOrControl+,")
                .build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "Quit")
                .accelerator("CommandOrControl+Q")
                .build(app)?;

            let menu = MenuBuilder::new(app)
                .items(&[
                    &open_item,
                    &PredefinedMenuItem::separator(app)?,
                    &capture_region_item,
                    &capture_screen_item,
                    &capture_window_item,
                    &capture_ocr_item,
                    &PredefinedMenuItem::separator(app)?,
                    &preferences_item,
                    &PredefinedMenuItem::separator(app)?,
                    &quit_item,
                ])
                .build()?;

            let _tray = tauri::tray::TrayIconBuilder::new()
                .menu(&menu)
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("FrameXShot")
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    "open" => {
                        if let Err(e) = show_main_window(app) {
                            eprintln!("Failed to show window: {}", e);
                        }
                    }
                    "capture_region" => {
                        if let Err(e) = show_main_window(app) {
                            eprintln!("Failed to show window: {}", e);
                        }
                        let _ = app.emit("capture-triggered", ());
                    }
                    "capture_screen" => {
                        if let Err(e) = show_main_window(app) {
                            eprintln!("Failed to show window: {}", e);
                        }
                        let _ = app.emit("capture-fullscreen", ());
                    }
                    "capture_window" => {
                        if let Err(e) = show_main_window(app) {
                            eprintln!("Failed to show window: {}", e);
                        }
                        let _ = app.emit("capture-window", ());
                    }
                    "capture_ocr" => {
                        if let Err(e) = show_main_window(app) {
                            eprintln!("Failed to show window: {}", e);
                        }
                        let _ = app.emit("capture-ocr", ());
                    }
                    "preferences" => {
                        if let Err(e) = show_main_window(app) {
                            eprintln!("Failed to show window: {}", e);
                        } else {
                            let _ = app.emit("open-preferences", ());
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            capture_once,
            capture_all_monitors,
            capture_region,
            save_edited_image,
            render_image_with_effects_rust,
            get_desktop_directory,
            get_temp_directory,
            native_capture_interactive,
            native_capture_fullscreen,
            native_capture_window,
            native_capture_ocr_region,
            play_screenshot_sound,
            get_mouse_position,
            move_window_to_active_space,
            copy_image_file_to_clipboard,
            show_quick_overlay,
            select_folder_dialog,
            read_file_as_base64,
            capture_screen_for_selector,
            crop_and_save_region
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
