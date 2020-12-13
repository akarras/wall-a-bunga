mod gui;
mod ratio_menu;
mod resolution_menu;
mod settings;
mod style;

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
        default_font: None,
        default_text_size: 18,
        antialiasing: false,
    });
}
