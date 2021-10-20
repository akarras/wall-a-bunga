mod download_manager;
mod gui;
mod settings;
mod style;
mod submenus;

use crate::settings::SavedSettings;
use gui::WallpaperUi;
use iced::{window, Application, Settings};

/// Hides the console that pops up when the Iced gui is started on Windows.
#[cfg(windows)]
fn hide_console_window() {
    use winapi::um::wincon::GetConsoleWindow;
    use winapi::um::winuser::{ShowWindow, SW_HIDE};

    let window = unsafe { GetConsoleWindow() };
    // https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-showwindow
    if !window.is_null() {
        unsafe {
            ShowWindow(window, SW_HIDE);
        }
    }
}

fn main() {
    pretty_env_logger::init();
    #[cfg(windows)]
    hide_console_window();
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
