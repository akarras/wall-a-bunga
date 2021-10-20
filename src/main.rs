mod download_manager;
mod gui;
mod settings;
mod style;
mod submenus;

use crate::settings::SavedSettings;
use gui::WallpaperUi;
use iced::{window, Application, Settings};

#[cfg(windows)]
fn hide_console_window() {
    use std::ptr;
    use winapi::um::wincon::GetConsoleWindow;
    use winapi::um::winuser::{ShowWindow, SW_HIDE};

    let window = unsafe { GetConsoleWindow() };
    // https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-showwindow
    if window != ptr::null_mut() {
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
