use crate::gui::WallpaperMessage;
use iced::button;
use iced::{Button, Length, Text};

pub(crate) fn make_button<'a>(
    state: &'a mut button::State,
    text: &str,
) -> Button<'a, WallpaperMessage> {
    Button::new(state, Text::new(text).size(18))
        .padding(10)
        .style(button_style::Button::Primary)
        .height(Length::Shrink)
        .width(Length::Shrink)
}

pub(crate) fn inactive_style(btn: bool) -> button_style::Button {
    match btn {
        true => button_style::Button::Primary,
        false => button_style::Button::Inactive,
    }
}

pub mod button_style {
    use iced::{button, Background, Color, Vector};

    pub enum Button {
        Primary,
        Select,
        Inactive,
        Downloading,
    }

    impl button::StyleSheet for Button {
        fn active(&self) -> button::Style {
            button::Style {
                background: Some(Background::Color(match self {
                    Button::Primary => Color::from_rgb(0.11, 0.42, 0.87),
                    Button::Select => Color::from_rgb(0.1, 0.8, 0.2),
                    Button::Inactive => Color::from_rgb(0.1, 0.1, 0.1),
                    Button::Downloading => Color::from_rgb(1.0, 0.82, 0.863),
                })),
                border_radius: 12.0,
                shadow_offset: Vector::new(1.0, 1.0),
                text_color: Color::WHITE,
                ..button::Style::default()
            }
        }
    }
}
