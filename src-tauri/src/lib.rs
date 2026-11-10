mod recording;
mod settings;

use recording::Recording;
use serde::{Deserialize, Serialize};
use settings::{Ratio, Settings};
use std::{sync::Mutex, thread, time::Duration};
use tauri::{
    image::Image,
    menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu},
    tray::TrayIconBuilder,
    AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, State, WebviewWindow, Wry,
};
use tauri_plugin_window_state::{AppHandleExt, StateFlags, WindowExt};

const MIN_SHORT_SIDE: f64 = 135.0;
const DEFAULT_AREA: f64 = 270.0 * 480.0;

struct AppState {
    settings: Mutex<Settings>,
    recording: Mutex<Option<Recording>>,
    ratio_items: Vec<(Ratio, CheckMenuItem<Wry>)>,
    camera_menu: Submenu<Wry>,
    mic_menu: Submenu<Wry>,
    toggle_item: MenuItem<Wry>,
    record_item: MenuItem<Wry>,
}

impl AppState {
    fn set_recording_menu(&self, recording: bool) -> tauri::Result<()> {
        self.record_item
            .set_text(if recording { "Berhenti Rekam" } else { "Mulai Rekam" })?;
        self.toggle_item.set_enabled(!recording)?;
        // The recorded canvas size is fixed when recording starts, so ratio and camera stay locked.
        for (_, item) in &self.ratio_items {
            item.set_enabled(!recording)?;
        }
        self.camera_menu.set_enabled(!recording)
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SettingsPayload {
    ratio: [f64; 2],
    camera_id: Option<String>,
    mic_id: Option<String>,
}

#[derive(Deserialize)]
struct DeviceInfo {
    id: String,
    label: String,
}

fn window_flags() -> StateFlags {
    StateFlags::SIZE | StateFlags::POSITION
}

fn min_size(ratio: Ratio) -> (f64, f64) {
    let (rw, rh) = ratio.dims();
    if rw < rh {
        (MIN_SHORT_SIDE, (MIN_SHORT_SIDE * rh / rw).round())
    } else {
        ((MIN_SHORT_SIDE * rw / rh).round(), MIN_SHORT_SIDE)
    }
}

fn size_for(ratio: Ratio, area: f64) -> (f64, f64) {
    let (rw, rh) = ratio.dims();
    let (min_w, min_h) = min_size(ratio);
    let width = (area * rw / rh).sqrt().max(min_w).round();
    (width, (width * rh / rw).round().max(min_h))
}

#[cfg(target_os = "macos")]
fn apply_native_aspect(window: &WebviewWindow, ratio: Ratio) -> tauri::Result<()> {
    use objc2_app_kit::NSWindow;
    use objc2_foundation::NSSize;

    let (rw, rh) = ratio.dims();
    // SAFETY: only called from the setup hook and menu handlers, which run on the
    // macOS main thread; the pointer refers to Tauri's live NSWindow.
    let ns_window = unsafe { &*(window.ns_window()? as *mut NSWindow) };
    ns_window.setContentAspectRatio(NSSize::new(rw, rh));
    Ok(())
}

#[cfg(target_os = "macos")]
fn configure_overlay(window: &WebviewWindow) -> tauri::Result<()> {
    use objc2_app_kit::{NSStatusWindowLevel, NSWindow, NSWindowCollectionBehavior};

    // SAFETY: called from the setup hook, which runs on the macOS main thread,
    // and the pointer refers to Tauri's live NSWindow for this window.
    let ns_window = unsafe { &*(window.ns_window()? as *mut NSWindow) };
    ns_window.setCollectionBehavior(
        NSWindowCollectionBehavior::CanJoinAllSpaces
            | NSWindowCollectionBehavior::FullScreenAuxiliary
            | NSWindowCollectionBehavior::Stationary,
    );
    ns_window.setLevel(NSStatusWindowLevel);
    Ok(())
}

/// Resizes the window to `ratio` with roughly `area` logical pixels, optionally keeping its center.
fn fit_window(
    window: &WebviewWindow,
    ratio: Ratio,
    area: Option<f64>,
    keep_center: bool,
) -> tauri::Result<()> {
    let scale = window.scale_factor()?;
    let old = window.inner_size()?.to_logical::<f64>(scale);
    let (width, height) = size_for(ratio, area.unwrap_or(old.width * old.height));
    let (min_w, min_h) = min_size(ratio);
    window.set_min_size(Some(LogicalSize::new(min_w, min_h)))?;
    #[cfg(target_os = "macos")]
    apply_native_aspect(window, ratio)?;

    if (old.width - width).abs() <= 1.0 && (old.height - height).abs() <= 1.0 {
        return Ok(());
    }
    if keep_center {
        let pos = window.outer_position()?.to_logical::<f64>(scale);
        window.set_position(LogicalPosition::new(
            pos.x + (old.width - width) / 2.0,
            pos.y + (old.height - height) / 2.0,
        ))?;
    }
    window.set_size(LogicalSize::new(width, height))
}

#[tauri::command]
fn get_settings(state: State<AppState>) -> SettingsPayload {
    let settings = state.settings.lock().unwrap();
    let (rw, rh) = settings.ratio.dims();
    SettingsPayload {
        ratio: [rw, rh],
        camera_id: settings.camera_id.clone(),
        mic_id: settings.mic_id.clone(),
    }
}

/// Rebuilds the "Kamera" (`kind = "camera"`) or "Mikrofon" (`kind = "mic"`) submenu.
#[tauri::command]
fn set_devices(
    app: AppHandle,
    state: State<AppState>,
    kind: String,
    devices: Vec<DeviceInfo>,
    selected: Option<String>,
) -> tauri::Result<()> {
    let is_mic = kind == "mic";
    let (menu, fallback_label) = if is_mic {
        (&state.mic_menu, "Mikrofon")
    } else {
        (&state.camera_menu, "Kamera")
    };
    while menu.remove_at(0)?.is_some() {}

    if is_mic {
        // WebKit hides microphone devices until mic permission is granted, so an
        // empty list only shows the default entry instead of "no microphone".
        let item = CheckMenuItem::with_id(&app, "mic:default", "Default Sistem", true, selected.is_none(), None::<&str>)?;
        menu.append(&item)?;
        if !devices.is_empty() {
            menu.append(&PredefinedMenuItem::separator(&app)?)?;
        }
    } else if devices.is_empty() {
        menu.append(&MenuItem::with_id(&app, "camera-none", "Tidak ada kamera", false, None::<&str>)?)?;
    }
    for (index, device) in devices.iter().enumerate() {
        let label = match device.label.trim() {
            "" => format!("{fallback_label} {}", index + 1),
            label => label.to_string(),
        };
        let checked = selected.as_deref() == Some(device.id.as_str());
        let item = CheckMenuItem::with_id(&app, format!("{kind}:{}", device.id), label, true, checked, None::<&str>)?;
        menu.append(&item)?;
    }

    if !is_mic && let Some(selected) = selected {
        let mut settings = state.settings.lock().unwrap();
        if settings.camera_id.as_deref() != Some(selected.as_str()) {
            settings.camera_id = Some(selected);
            settings.save(&app);
        }
    }
    Ok(())
}

fn check_only(menu: &Submenu<Wry>, id: &str) -> tauri::Result<()> {
    for item in menu.items()? {
        if let Some(check) = item.as_check_menuitem() {
            check.set_checked(check.id().0 == id)?;
        }
    }
    Ok(())
}

fn quit(app: &AppHandle) {
    let _ = app.save_window_state(window_flags());
    app.exit(0);
}

fn handle_menu(app: &AppHandle, event: &MenuEvent) -> tauri::Result<()> {
    let state = app.state::<AppState>();
    let Some(window) = app.get_webview_window("main") else { return Ok(()) };
    let id = event.id().0.as_str();

    if let Some(ratio) = id.strip_prefix("ratio:").and_then(Ratio::from_id) {
        for (item_ratio, item) in &state.ratio_items {
            item.set_checked(*item_ratio == ratio)?;
        }
        {
            let mut settings = state.settings.lock().unwrap();
            settings.ratio = ratio;
            settings.save(app);
        }
        fit_window(&window, ratio, None, true)?;
        let (rw, rh) = ratio.dims();
        window.emit("ratio-changed", [rw, rh])?;
    } else if let Some(camera_id) = id.strip_prefix("camera:") {
        check_only(&state.camera_menu, id)?;
        {
            let mut settings = state.settings.lock().unwrap();
            settings.camera_id = Some(camera_id.to_string());
            settings.save(app);
        }
        window.emit("select-camera", camera_id)?;
    } else if let Some(mic_id) = id.strip_prefix("mic:") {
        check_only(&state.mic_menu, id)?;
        let mic_id = (mic_id != "default").then(|| mic_id.to_string());
        {
            let mut settings = state.settings.lock().unwrap();
            settings.mic_id = mic_id.clone();
            settings.save(app);
        }
        window.emit("select-mic", mic_id)?;
    } else {
        match id {
            "record" => window.emit("recording-toggle", ())?,
            "toggle" => {
                let visible = window.is_visible()?;
                if visible {
                    window.hide()?;
                } else {
                    window.show()?;
                }
                state.toggle_item.set_text(if visible { "Tampilkan Overlay" } else { "Sembunyikan Overlay" })?;
                window.emit("overlay-visibility", !visible)?;
            }
            "reset" => {
                let ratio = state.settings.lock().unwrap().ratio;
                fit_window(&window, ratio, Some(DEFAULT_AREA), false)?;
                window.center()?;
            }
            "quit" if recording::is_recording(&state) => {
                window.emit("request-quit", ())?;
                let app = app.clone();
                thread::spawn(move || {
                    thread::sleep(Duration::from_secs(5));
                    quit(&app);
                });
            }
            "quit" => quit(app),
            _ => {}
        }
    }
    Ok(())
}

fn build_tray(app: &tauri::App, ratio: Ratio) -> tauri::Result<AppState> {
    let version = app.package_info().version.to_string();
    let about = MenuItem::with_id(app, "about", format!("InspirasiCam v{version}"), false, None::<&str>)?;
    let record_item = MenuItem::with_id(app, "record", "Mulai Rekam", true, None::<&str>)?;
    let toggle_item = MenuItem::with_id(app, "toggle", "Sembunyikan Overlay", true, None::<&str>)?;

    let ratio_menu = Submenu::with_id(app, "ratio", "Rasio", true)?;
    let mut ratio_items = Vec::new();
    for item_ratio in Ratio::ALL {
        let item = CheckMenuItem::with_id(
            app,
            format!("ratio:{}", item_ratio.id()),
            item_ratio.label(),
            true,
            item_ratio == ratio,
            None::<&str>,
        )?;
        ratio_menu.append(&item)?;
        ratio_items.push((item_ratio, item));
    }

    let camera_menu = Submenu::with_id(app, "camera", "Kamera", true)?;
    camera_menu.append(&MenuItem::with_id(app, "camera-loading", "Memuat kamera…", false, None::<&str>)?)?;
    let mic_menu = Submenu::with_id(app, "mic", "Mikrofon", true)?;
    mic_menu.append(&MenuItem::with_id(app, "mic-loading", "Memuat mikrofon…", false, None::<&str>)?)?;

    let reset = MenuItem::with_id(app, "reset", "Reset Posisi & Ukuran", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Keluar", true, Some("CmdOrCtrl+Q"))?;

    let menu = Menu::with_items(
        app,
        &[
            &about,
            &PredefinedMenuItem::separator(app)?,
            &record_item,
            &PredefinedMenuItem::separator(app)?,
            &toggle_item,
            &ratio_menu,
            &camera_menu,
            &mic_menu,
            &reset,
            &PredefinedMenuItem::separator(app)?,
            &quit,
        ],
    )?;

    TrayIconBuilder::with_id("main")
        .icon(Image::from_bytes(include_bytes!("../icons/tray.png"))?)
        .icon_as_template(true)
        .tooltip("InspirasiCam")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| {
            if let Err(error) = handle_menu(app, &event) {
                eprintln!("menu action failed: {error}");
            }
        })
        .build(app)?;

    Ok(AppState {
        settings: Mutex::new(Settings::default()),
        recording: Mutex::new(None),
        ratio_items,
        camera_menu,
        mic_menu,
        toggle_item,
        record_item,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_window_state::Builder::new()
                .with_state_flags(window_flags())
                .skip_initial_state("main")
                .build(),
        )
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![
            get_settings,
            set_devices,
            recording::recording_start,
            recording::recording_append,
            recording::recording_stop,
            recording::crop_recording,
            recording::recording_error,
        ])
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let settings = Settings::load(app.handle());
            let window = app
                .get_webview_window("main")
                .expect("main window not found");
            let _ = window.restore_state(window_flags());
            fit_window(&window, settings.ratio, None, false)?;
            #[cfg(target_os = "macos")]
            configure_overlay(&window)?;

            let state = build_tray(app, settings.ratio)?;
            *state.settings.lock().unwrap() = settings;
            app.manage(state);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
