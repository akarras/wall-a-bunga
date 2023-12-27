use crate::gui::WallpaperMessage;
use crate::style::{inactive_style, make_button};
use iced::widget::Row;
use std::collections::HashSet;
use std::sync::OnceLock;
use wallapi::types::XYCombo;

#[derive(Debug, Clone)]
pub(crate) struct RatioMenu {
    options: Vec<(XYCombo, &'static str)>,
}

impl Default for RatioMenu {
    fn default() -> Self {
        static LOCK: OnceLock<Vec<(XYCombo, &str)>> = OnceLock::new();
        let options = LOCK.get_or_init(|| {
            wallapi::types::ASPECT_RATIOS
                .iter()
                .map(|ratio| {
                    let s: &'static str = Box::new(ratio.to_string()).leak();
                    (*ratio, s)
                })
                .collect()
        });
        Self {
            options: options.clone(),
        }
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
        &self,
        selected_ratios: &Option<HashSet<XYCombo>>,
    ) -> Row<WallpaperMessage> {
        self.options.iter().fold(Row::new(), |row, (ratio, label)| {
            row.push(
                make_button(label)
                    .style(inactive_style(get_is_toggled(ratio, selected_ratios)))
                    .on_press(WallpaperMessage::AspectRatioSelected(*ratio)),
            )
        })
    }
}
