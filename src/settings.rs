use log::info;
use platform_dirs::AppDirs;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub(crate) struct SavedSettings {
    pub(crate) save_directory: Option<String>,
    pub(crate) api_key: Option<String>,
    #[serde(default)]
    pub(crate) ignore_downloaded: bool,
}

impl SavedSettings {
    pub(crate) async fn save_settings(settings: SavedSettings) -> () {
        let app_dirs = AppDirs::new(Some("wall-a-bunga"), true).unwrap();
        tokio::fs::create_dir_all(app_dirs.config_dir.clone())
            .await
            .expect("Failed to create all directories");
        let config_file = app_dirs.config_dir.join("config.json");
        let mut file = tokio::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(config_file.clone())
            .await
            .expect(&format!(
                "Failed to create or open config file at {:?}",
                config_file
            ));
        file.write_all(
            serde_json::to_string(&settings)
                .expect("Failed to serialize config")
                .as_bytes(),
        )
        .await
        .expect("Don't fail saving this plz");
        info!("Saved settings to {:?}", config_file);
    }

    // Function left sync intentionally
    pub(crate) fn load_settings() -> Option<Self> {
        let app_dirs = AppDirs::new(Some("wall-a-bunga"), true).unwrap();
        let config_file = app_dirs.config_dir.join("config.json");
        let json = std::fs::read_to_string(config_file.clone()).ok()?;
        info!("Loaded settings from {:?} with json {}", config_file, json);
        serde_json::from_str(&json).ok()
    }
}
