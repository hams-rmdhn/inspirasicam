use tauri::{LogicalSize, Manager, WebviewWindow};
use tauri_plugin_window_state::{StateFlags, WindowExt};

const ASPECT_W: f64 = 9.0;
const ASPECT_H: f64 = 16.0;
const MIN_WIDTH: f64 = 135.0;

fn normalize_aspect_ratio(window: &WebviewWindow) -> tauri::Result<()> {
    let scale = window.scale_factor()?;
    let size = window.inner_size()?.to_logical::<f64>(scale);
    let width = size.width.max(MIN_WIDTH).round();
    let height = (width * ASPECT_H / ASPECT_W).round();
    if (size.width - width).abs() > 1.0 || (size.height - height).abs() > 1.0 {
        window.set_size(LogicalSize::new(width, height))?;
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn configure_overlay(window: &WebviewWindow) -> tauri::Result<()> {
    use objc2_app_kit::{NSStatusWindowLevel, NSWindow, NSWindowCollectionBehavior};
    use objc2_foundation::NSSize;

    // SAFETY: called from the setup hook, which runs on the macOS main thread,
    // and the pointer refers to Tauri's live NSWindow for this window.
    let ns_window = unsafe { &*(window.ns_window()? as *mut NSWindow) };
    ns_window.setContentAspectRatio(NSSize::new(ASPECT_W, ASPECT_H));
    ns_window.setCollectionBehavior(
        NSWindowCollectionBehavior::CanJoinAllSpaces
            | NSWindowCollectionBehavior::FullScreenAuxiliary
            | NSWindowCollectionBehavior::Stationary,
    );
    ns_window.setLevel(NSStatusWindowLevel);
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_window_state::Builder::new()
                .with_state_flags(StateFlags::SIZE | StateFlags::POSITION)
                .skip_initial_state("main")
                .build(),
        )
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let window = app
                .get_webview_window("main")
                .expect("main window not found");
            let _ = window.restore_state(StateFlags::SIZE | StateFlags::POSITION);
            normalize_aspect_ratio(&window)?;
            #[cfg(target_os = "macos")]
            configure_overlay(&window)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
