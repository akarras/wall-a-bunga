use crate::gui::WallpaperMessage;
use crate::style::{inactive_style, make_button};
use iced::{button, Row};
use std::collections::HashSet;
use wallapi::types::XYCombo;

#[derive(Debug, Clone)]
pub(crate) struct RatioMenu {
    button_states: Vec<(XYCombo, button::State)>,
}

impl Default for RatioMenu {
    fn default() -> Self {
        let button_states = wallapi::types::ASPECT_RATIOS
            .iter()
            .map(|aspect_ratio| (aspect_ratio.clone(), button::State::new()))
            .collect();
        Self { button_states }
    }
}

fn get_is_toggled(option: &XYCombo, selections: &Option<HashSet<XYCombo>>) -> bool {
    match selections {
        None => false,
        Some(options) => options.contains(option),
    }
}

impl RatioMenu {
    pub(crate) fn build_ratio_row(
        &mut self,
        selected_ratios: &Option<HashSet<XYCombo>>,
    ) -> Row<WallpaperMessage> {
        self.button_states
            .iter_mut()
            .fold(Row::new(), |row, (ratio, state)| {
                row.push(
                    make_button(state, &ratio.to_string())
                        .style(inactive_style(get_is_toggled(ratio, &selected_ratios)))
                        .on_press(WallpaperMessage::AspectRatioSelected(ratio.clone())),
                )
            })
    }
}
