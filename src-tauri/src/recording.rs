use crate::AppState;
use std::{
    fs::{self, File},
    io::{BufWriter, Write},
    path::PathBuf,
};
use tauri::{ipc::InvokeBody, AppHandle, Manager, State};
use tauri_plugin_notification::NotificationExt;

pub struct Recording {
    file: BufWriter<File>,
    path: PathBuf,
}

fn file_name(path: &std::path::Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn recording_path(app: &AppHandle, timestamp: &str) -> Result<PathBuf, String> {
    let valid = timestamp.len() == 19 && timestamp.chars().all(|c| c.is_ascii_digit() || c == '-');
    if !valid {
        return Err("timestamp tidak valid".into());
    }
    let dir = app.path().video_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let base = format!("Rekaman-{timestamp}");
    let mut path = dir.join(format!("{base}.mp4"));
    let mut suffix = 2;
    while path.exists() {
        path = dir.join(format!("{base}-{suffix}.mp4"));
        suffix += 1;
    }
    Ok(path)
}

fn notify(app: &AppHandle, title: &str, body: &str) {
    if let Err(error) = app.notification().builder().title(title).body(body).show() {
        eprintln!("failed to show notification: {error}");
    }
}

pub fn is_recording(state: &AppState) -> bool {
    state.recording.lock().unwrap().is_some()
}

#[tauri::command]
pub fn recording_start(app: AppHandle, state: State<AppState>, timestamp: String) -> Result<String, String> {
    let mut recording = state.recording.lock().unwrap();
    if recording.is_some() {
        return Err("Rekaman sedang berjalan".into());
    }
    let path = recording_path(&app, &timestamp)?;
    let file = File::create(&path).map_err(|e| e.to_string())?;
    let name = file_name(&path);
    *recording = Some(Recording { file: BufWriter::new(file), path });
    drop(recording);
    state.set_recording_menu(true).map_err(|e| e.to_string())?;
    Ok(name)
}

#[tauri::command]
pub async fn recording_append(state: State<'_, AppState>, request: tauri::ipc::Request<'_>) -> Result<(), String> {
    let InvokeBody::Raw(bytes) = request.body() else {
        return Err("payload rekaman harus berupa bytes".into());
    };
    let mut recording = state.recording.lock().unwrap();
    let recording = recording.as_mut().ok_or("Tidak ada rekaman aktif")?;
    recording.file.write_all(bytes).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn recording_stop(app: AppHandle, state: State<AppState>) -> Result<String, String> {
    let recording = state.recording.lock().unwrap().take();
    state.set_recording_menu(false).map_err(|e| e.to_string())?;
    let Some(mut recording) = recording else {
        return Err("Tidak ada rekaman aktif".into());
    };
    recording.file.flush().map_err(|e| e.to_string())?;
    let name = file_name(&recording.path);
    drop(recording);
    notify(&app, "Rekaman tersimpan", &format!("{name} di folder Movies"));
    Ok(name)
}

/// Crops the recorded file to the given aspect ratio using AVFoundation.
/// MediaRecorder always writes the camera's native frame; the ratio is applied
/// here so the file matches what the overlay shows.
#[tauri::command]
pub async fn crop_recording(app: AppHandle, name: String, rw: f64, rh: f64) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        #[cfg(target_os = "macos")]
        {
            crop_file(&app, &name, rw, rh)
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = (app, rw, rh);
            Ok(name)
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(target_os = "macos")]
fn crop_file(app: &AppHandle, name: &str, rw: f64, rh: f64) -> Result<String, String> {
    #![allow(deprecated)]
    use objc2_core_foundation::{CGAffineTransform, CGSize};
    use objc2::AllocAnyThread;
    use objc2_av_foundation::*;
    use objc2_core_media::{CMTime, CMTimeRange};
    use objc2_foundation::{NSArray, NSString, NSURL};

    if !name.starts_with("Rekaman-") || !name.ends_with(".mp4") || name.contains('/') || name.contains('\\') {
        return Err("nama file tidak valid".into());
    }
    let dir = app.path().video_dir().map_err(|e| e.to_string())?;
    let src = dir.join(name);
    let tmp = dir.join(format!(".{name}.crop.mp4"));

    let inner = || unsafe {
        let url = NSURL::fileURLWithPath(&NSString::from_str(&src.to_string_lossy()));
        let asset = AVURLAsset::URLAssetWithURL_options(&url, None);
        let tracks = asset.tracksWithMediaType(AVMediaTypeVideo.ok_or("no video media type")?);
        let track = tracks.firstObject().ok_or("tidak ada video track")?;
        let size = track.naturalSize();
        let target = rw / rh;
        let (cw, ch) = if size.width / size.height > target {
            (size.height * target, size.height)
        } else {
            (size.width, size.width / target)
        };
        if (size.width - cw).abs() < 2.0 && (size.height - ch).abs() < 2.0 {
            return Ok(false); // already matches the ratio, nothing to export
        }
        let rw_out = (cw / 2.0).floor() * 2.0;
        let rh_out = (ch / 2.0).floor() * 2.0;
        let sx = (size.width - cw) / 2.0;
        let sy = (size.height - ch) / 2.0;

        let composition = AVMutableVideoComposition::videoComposition();
        composition.setRenderSize(CGSize::new(rw_out, rh_out));
        let fps = track.nominalFrameRate();
        composition.setFrameDuration(CMTime::new(1, if fps > 0.0 { fps.max(1.0) as i32 } else { 30 }));

        let zero = CMTime::new(0, 600);
        let instruction = AVMutableVideoCompositionInstruction::videoCompositionInstruction();
        instruction.setTimeRange(CMTimeRange { start: zero, duration: asset.duration() });
        let layer = AVMutableVideoCompositionLayerInstruction::videoCompositionLayerInstructionWithAssetTrack(&track);
        layer.setTransform_atTime(
            CGAffineTransform { a: 1.0, b: 0.0, c: 0.0, d: 1.0, tx: -sx, ty: -sy },
            zero,
        );
        let layer_ref = &*(&*layer as *const AVMutableVideoCompositionLayerInstruction
            as *const AVVideoCompositionLayerInstruction);
        instruction.setLayerInstructions(&NSArray::arrayWithObject(layer_ref));
        let proto = objc2::runtime::ProtocolObject::<dyn AVVideoCompositionInstructionProtocol>::from_ref(&*instruction);
        composition.setInstructions(&NSArray::arrayWithObject(proto));

        let tmp_url = NSURL::fileURLWithPath(&NSString::from_str(&tmp.to_string_lossy()));
        let session = AVAssetExportSession::initWithAsset_presetName(
            AVAssetExportSession::alloc(),
            &asset,
            &NSString::from_str("AVAssetExportPresetHighestQuality"),
        )
        .ok_or("gagal membuat export session")?;
        session.setOutputURL(Some(&tmp_url));
        session.setOutputFileType(AVFileTypeMPEG4);
        let comp_ref = &*(&*composition as *const AVMutableVideoComposition as *const AVVideoComposition);
        session.setVideoComposition(Some(comp_ref));
        let done = block2::StackBlock::new(|| {});
        session.exportAsynchronouslyWithCompletionHandler(&*done);

        for _ in 0..2400 {
            match session.status() {
                AVAssetExportSessionStatus::Completed => return Ok(true),
                AVAssetExportSessionStatus::Failed | AVAssetExportSessionStatus::Cancelled => {
                    let message = session
                        .error()
                        .map(|e| e.localizedDescription().to_string())
                        .unwrap_or_else(|| "export dibatalkan".into());
                    return Err(message);
                }
                _ => std::thread::sleep(std::time::Duration::from_millis(50)),
            }
        }
        Err("export terlalu lama".into())
    };

    match inner() {
        // rename atomically replaces src, so no need to delete it first
        Ok(true) => {
            fs::rename(&tmp, &src).map_err(|e| e.to_string())?;
            Ok(name.to_string())
        }
        Ok(false) => Ok(name.to_string()),
        Err(error) => {
            let _ = fs::remove_file(&tmp);
            Err(error)
        }
    }
}

#[tauri::command]
pub fn recording_error(app: AppHandle, state: State<AppState>, message: String) -> Result<(), String> {
    if let Some(recording) = state.recording.lock().unwrap().take() {
        let path = recording.path.clone();
        drop(recording);
        if fs::metadata(&path).map(|m| m.len() == 0).unwrap_or(false) {
            let _ = fs::remove_file(&path);
        }
    }
    state.set_recording_menu(false).map_err(|e| e.to_string())?;
    notify(&app, "Gagal merekam", &message);
    Ok(())
}
