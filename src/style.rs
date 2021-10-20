use crate::font_awesome::FAIcon;
use crate::gui::WallpaperMessage;
use font_awesome_as_a_crate::Type;
use iced::{button, Row};
use iced::{Button, Length, Text};

pub(crate) fn make_button<'a>(
    state: &'a mut button::State,
    text: &str,
) -> Button<'a, WallpaperMessage> {
    Button::new(state, Text::new(text).size(21))
        .padding(10)
        .style(button_style::Button::Primary)
        .height(Length::Shrink)
        .width(Length::Shrink)
}

pub(crate) fn make_button_fa<'a>(
    state: &'a mut button::State,
    text: &str,
    fa_icon: &str,
) -> Button<'a, WallpaperMessage> {
    Button::new(
        state,
        Row::new()
            .push(
                FAIcon::new(Type::Solid, fa_icon)
                    .svg()
                    .height(Length::Units(16)),
            )
            .push(Text::new(text).size(18)),
    )
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
        Downloaded,
        Inactive,
        Downloading,
        Failed,
    }

    impl button::StyleSheet for Button {
        fn active(&self) -> button::Style {
            button::Style {
                background: Some(Background::Color(match self {
                    Button::Primary => Color::from_rgb(0.87, 0.42, 0.11),
                    Button::Downloaded => Color::from_rgb(0.467, 0.867, 0.467),
                    Button::Inactive => Color::from_rgb(0.3, 0.3, 0.3),
                    Button::Downloading => Color::from_rgb(0.992, 0.992, 0.588),
                    Button::Failed => Color::from_rgb(1.0, 0.0, 0.0),
                })),
                border_radius: 12.0,
                shadow_offset: Vector::new(1.0, 1.0),
                text_color: Color::WHITE,
                ..button::Style::default()
            }
        }
    }
}
