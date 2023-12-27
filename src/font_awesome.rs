use font_awesome_as_a_crate::Type;
use iced::widget::svg::Handle;
use iced::widget::Svg;
use iced::Length;

pub struct FAIcon {
    icon_handle: Handle,
}

impl FAIcon {
    /// Creates a new font awesome icon, panics if the icon can not be found
    pub fn new(fa_type: Type, icon_name: &str) -> Self {
        let svg_str = font_awesome_as_a_crate::svg(fa_type, icon_name)
            .unwrap()
            // this replace hack helps turn all the icons white.
            .replace("<path", "<path fill=\"white\"");
        let svg = svg_str.as_bytes().to_vec();
        let handle = iced::widget::svg::Handle::from_memory(svg);
        Self {
            icon_handle: handle,
        }
    }

    pub fn svg(&self) -> Svg {
        Svg::new(self.icon_handle.clone())
            .width(Length::Shrink)
            .height(Length::Shrink)
    }
}
