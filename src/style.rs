use crate::font_awesome::FAIcon;
use crate::gui::WallpaperMessage;
use font_awesome_as_a_crate::Type;
use iced::{
    theme,
    widget::{Button, Row, Space, Text},
    Length,
};

pub(crate) fn make_button(
    // state: &'a mut button::State,
    text: &str,
) -> Button<'_, WallpaperMessage> {
    Button::new(Text::new(text).size(21))
        .padding(10)
        .style(theme::Button::custom(button_style::Button::Primary))
        .height(Length::Shrink)
        .width(Length::Shrink)
}

pub(crate) fn make_button_fa<'a>(
    // state: &'a mut button::State,
    text: &'a str,
    fa_icon: &str,
) -> Button<'a, WallpaperMessage> {
    Button::new(
        // state,
        Row::new()
            .push(Text::new(text).size(21))
            .push(Space::new(Length::Fixed(5.0), Length::Shrink))
            .push(
                FAIcon::new(Type::Solid, fa_icon)
                    .svg()
                    .height(Length::Fixed(21.0))
                    .width(Length::Fixed(21.0)),
            ),
    )
    .padding(10)
    .style(theme::Button::custom(button_style::Button::Primary))
    .height(Length::Shrink)
    .width(Length::Shrink)
}

pub(crate) fn inactive_style(btn: bool) -> theme::Button {
    let custom_style = match btn {
        true => button_style::Button::Primary,
        false => button_style::Button::Inactive,
    };
    theme::Button::custom(custom_style)
}

pub mod pick_style {
    use iced::{overlay::menu, widget::pick_list, Background, BorderRadius, Color, Theme};

    #[derive(Default, Clone)]
    pub struct PickList;

    impl menu::StyleSheet for PickList {
        type Style = Theme;

        fn appearance(&self, _style: &Self::Style) -> menu::Appearance {
            menu::Appearance {
                text_color: Color::WHITE,
                background: Background::Color(Color::from_rgb(0.3, 0.3, 0.3)),
                border_width: 1.0,
                border_color: Color::from_rgb(0.3, 0.3, 0.3),
                selected_background: Color::from_rgb(0.3, 0.3, 0.3).into(),
                selected_text_color: Color::WHITE,
                border_radius: BorderRadius::from([1.0, 1.0, 1.0, 1.0]),
            }
        }
    }

    impl pick_list::StyleSheet for PickList {
        fn active(&self, _style: &Self::Style) -> pick_list::Appearance {
            pick_list::Appearance {
                text_color: Color::WHITE,
                placeholder_color: Color::WHITE,
                background: Color::from_rgb(0.3, 0.3, 0.3).into(),
                border_width: 1.0,
                border_color: Color {
                    a: 0.6,
                    ..Color::BLACK
                },
                border_radius: 10.0.into(),
                // icon_size: 0.5,
                handle_color: Color::from_rgb(0.6, 0.2, 0.1),
            }
        }

        fn hovered(&self, style: &Self::Style) -> pick_list::Appearance {
            let active = self.active(style);

            pick_list::Appearance {
                border_color: Color {
                    a: 0.9,
                    ..Color::BLACK
                },
                ..active
            }
        }

        type Style = Theme;
    }
}

pub mod button_style {
    use iced::{widget::button, Background, Color, Theme, Vector};

    #[derive(Copy, Clone)]
    pub enum Button {
        Primary,
        Downloaded,
        Inactive,
        Downloading,
        Failed,
    }

    impl Default for Button {
        fn default() -> Self {
            Self::Primary
        }
    }

    impl button::StyleSheet for Button {
        fn active(&self, _style: &Self::Style) -> button::Appearance {
            button::Appearance {
                background: Some(Background::Color(match self {
                    Button::Primary => Color::from_rgb(0.87, 0.42, 0.11),
                    Button::Downloaded => Color::from_rgb(0.467, 0.867, 0.467),
                    Button::Inactive => Color::from_rgb(0.3, 0.3, 0.3),
                    Button::Downloading => Color::from_rgb(0.992, 0.992, 0.588),
                    Button::Failed => Color::from_rgb(1.0, 0.0, 0.0),
                })),
                border_radius: 12.0.into(),
                shadow_offset: Vector::new(1.0, 1.0),
                text_color: Color::WHITE,
                ..button::Appearance::default()
            }
        }

        type Style = Theme;
    }
}
