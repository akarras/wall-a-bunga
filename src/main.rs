mod download_manager;
mod gui;
mod settings;
mod style;
mod submenus;

use crate::settings::SavedSettings;
use gui::WallpaperUi;
use iced::{window, Application, Settings};

fn main() {
    pretty_env_logger::init();
    WallpaperUi::run(Settings {
        window: window::Settings {
            size: (1800, 800),
            min_size: None,
            max_size: None,
            ..Default::default()
        },
        flags: SavedSettings::load_settings(),
        ..Default::default()
    })
    .expect("Failed to launch UI");
}
