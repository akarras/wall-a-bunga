use crate::gui::WallpaperMessage;
use crate::style::{inactive_style, make_button};
use crate::submenus::calculate_aspect_ratio;
use iced::widget::{Checkbox, Column, Row, Text};
use itertools::Itertools;
use std::collections::HashSet;
use std::sync::OnceLock;
use wallapi::types::XYCombo;

#[derive(Debug, Clone)]
pub(crate) struct ResolutionOptionsMenu {
    button_states: Vec<(XYCombo, &'static str)>,
    pub(crate) is_minimum_set: bool,
}

impl Default for ResolutionOptionsMenu {
    fn default() -> Self {
        static STATES: OnceLock<Vec<(XYCombo, &'static str)>> = OnceLock::new();
        let button_states = STATES
            .get_or_init(|| {
                wallapi::types::RESOLUTION_POSSIBILITIES
                    .into_iter()
                    .sorted_by(|a, b| {
                        let (bx, by) = calculate_aspect_ratio(b.x, b.y);
                        let (ax, ay) = calculate_aspect_ratio(a.x, a.y);
                        ax.cmp(&bx)
                            .then_with(|| ay.cmp(&by))
                            .then_with(|| a.x.cmp(&b.x))
                            .then_with(|| a.y.cmp(&b.y))
                    })
                    .map(|c| {
                        let s: &'static str = Box::new(c.to_string()).leak();
                        (c, s)
                    })
                    .collect()
            })
            .clone();
        Self {
            button_states,
            is_minimum_set: false,
        }
    }
}

impl ResolutionOptionsMenu {
    pub(crate) fn build_resolution_row(
        &self,
        selected_options: &Option<HashSet<XYCombo>>,
        minimum_resolution: &Option<XYCombo>,
    ) -> Row<WallpaperMessage> {
        let check_resolution_active_multi = |option: &XYCombo| -> bool {
            match selected_options {
                None => false,
                Some(options) => options.contains(option),
            }
        };

        let check_minimum_resolution_active = |button_option: &XYCombo| -> bool {
            minimum_resolution
                .as_ref()
                .map_or(false, |minimum| button_option.eq(minimum))
        };

        let is_minimum_resolution = self.is_minimum_set;

        self.button_states
            .iter()
            .group_by(|(res, _label)| calculate_aspect_ratio(res.x, res.y))
            .into_iter()
            .fold(Row::new(), |row, ((x, y), resolutions)| {
                row.push(resolutions.fold(
                    Column::new().push(Text::new(format!("{}:{}", x, y))),
                    |column, (res, label)| {
                        column.push(match is_minimum_resolution {
                            false => make_button(label)
                                .style(inactive_style(check_resolution_active_multi(res)))
                                .on_press(WallpaperMessage::ResolutionSelected(*res)),
                            true => make_button(label)
                                .style(inactive_style(check_minimum_resolution_active(res)))
                                .on_press(WallpaperMessage::SetMinimumResolution(*res)),
                        })
                    },
                ))
            })
            .push(Checkbox::new(
                "Minimum resolution",
                self.is_minimum_set,
                WallpaperMessage::ResolutionIsSingleTargetChanged,
            ))
    }
}
