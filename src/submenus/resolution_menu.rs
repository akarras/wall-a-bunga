use crate::gui::WallpaperMessage;
use crate::style::{inactive_style, make_button};
use crate::submenus::calculate_aspect_ratio;
use iced::{button, Checkbox, Color, Column, Row, Text};
use itertools::Itertools;
use std::collections::HashSet;
use wallapi::types::XYCombo;

#[derive(Debug, Clone)]
pub(crate) struct ResolutionOptionsMenu {
    button_states: Vec<(XYCombo, button::State)>,
    pub(crate) is_minimum_set: bool,
}

impl Default for ResolutionOptionsMenu {
    fn default() -> Self {
        let button_states = wallapi::types::RESOLUTION_POSSIBILITIES
            .iter()
            .sorted_by(|a, b| {
                let (bx, by) = calculate_aspect_ratio(b.x, b.y);
                let (ax, ay) = calculate_aspect_ratio(a.x, a.y);
                ax.cmp(&bx)
                    .then_with(|| ay.cmp(&by))
                    .then_with(|| a.x.cmp(&b.x))
                    .then_with(|| a.y.cmp(&b.y))
            })
            .map(|resolution| (resolution.clone(), button::State::new()))
            .collect();
        Self {
            button_states,
            is_minimum_set: false,
        }
    }
}

impl ResolutionOptionsMenu {
    pub(crate) fn build_resolution_row(
        &mut self,
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
            .iter_mut()
            .group_by(|(res, _)| calculate_aspect_ratio(res.x, res.y))
            .into_iter()
            .fold(Row::new(), |row, ((x, y), resolutions)| {
                row.push(resolutions.fold(
                    Column::new().push(Text::new(format!("{}:{}", x, y)).color(Color::WHITE)),
                    |column, (res, btn_state)| {
                        column.push(match is_minimum_resolution {
                            false => make_button(btn_state, &res.to_string())
                                .style(inactive_style(check_resolution_active_multi(res)))
                                .on_press(WallpaperMessage::ResolutionSelected(res.clone())),
                            true => make_button(btn_state, &res.to_string())
                                .style(inactive_style(check_minimum_resolution_active(res)))
                                .on_press(WallpaperMessage::SetMinimumResolution(res.clone())),
                        })
                    },
                ))
            })
            .push(
                Checkbox::new(
                    self.is_minimum_set,
                    "Minimum resolution",
                    WallpaperMessage::ResolutionIsSingleTargetChanged,
                )
                .text_color(Color::WHITE),
            )
    }
}
