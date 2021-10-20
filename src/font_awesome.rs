use font_awesome_as_a_crate::Type;
use iced::svg::Handle;
use iced::Svg;

pub struct FAIcon {
    icon_handle: Handle,
}

impl FAIcon {
    /// Creates a new font awesome icon, panics if the icon can not be found
    pub fn new(fa_type: Type, icon_name: &str) -> Self {
        let svg = font_awesome_as_a_crate::svg(fa_type, icon_name)
            .unwrap()
            .as_bytes()
            .to_vec();
        let handle = iced::svg::Handle::from_memory(svg);
        Self {
            icon_handle: handle,
        }
    }

    pub fn svg(&self) -> Svg {
        Svg::new(self.icon_handle.clone())
    }
}
