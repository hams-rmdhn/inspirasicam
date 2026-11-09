use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use tauri::{AppHandle, Manager};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Ratio {
    #[serde(rename = "16:9")]
    Landscape,
    #[default]
    #[serde(rename = "9:16")]
    Portrait,
    #[serde(rename = "1:1")]
    Square,
    #[serde(rename = "4:3")]
    Classic,
}

impl Ratio {
    pub const ALL: [Ratio; 4] = [Ratio::Landscape, Ratio::Portrait, Ratio::Square, Ratio::Classic];

    pub fn dims(self) -> (f64, f64) {
        match self {
            Ratio::Landscape => (16.0, 9.0),
            Ratio::Portrait => (9.0, 16.0),
            Ratio::Square => (1.0, 1.0),
            Ratio::Classic => (4.0, 3.0),
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Ratio::Landscape => "16:9",
            Ratio::Portrait => "9:16",
            Ratio::Square => "1:1",
            Ratio::Classic => "4:3",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Ratio::Landscape => "16:9 Landscape",
            Ratio::Portrait => "9:16 Portrait",
            Ratio::Square => "1:1 Square",
            Ratio::Classic => "4:3 Classic",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|r| r.id() == id)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub ratio: Ratio,
    pub camera_id: Option<String>,
}

fn settings_path(app: &AppHandle) -> Option<PathBuf> {
    app.path()
        .app_config_dir()
        .ok()
        .map(|dir| dir.join("settings.json"))
}

impl Settings {
    pub fn load(app: &AppHandle) -> Self {
        settings_path(app)
            .and_then(|path| fs::read_to_string(path).ok())
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, app: &AppHandle) {
        let Some(path) = settings_path(app) else { return };
        let result = path
            .parent()
            .map_or(Ok(()), fs::create_dir_all)
            .and_then(|_| fs::write(&path, serde_json::to_string_pretty(self).unwrap_or_default()));
        if let Err(error) = result {
            eprintln!("failed to save settings: {error}");
        }
    }
}
