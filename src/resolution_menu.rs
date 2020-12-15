use crate::gui::WallpaperMessage;
use crate::style::{inactive_style, make_button};
use iced::{button, Column, Row, Text};
use itertools::Itertools;
use std::collections::HashSet;
use wallapi::types::XYCombo;

#[derive(Debug, Clone)]
pub(crate) struct ResolutionOptionsMenu {
    button_states: Vec<(XYCombo, button::State)>,
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
        Self { button_states }
    }
}

fn calculate_aspect_ratio(x: i32, y: i32) -> (i32, i32) {
    let gcd = num::integer::gcd(y, x);
    (x / gcd, y / gcd)
}

impl ResolutionOptionsMenu {
    pub(crate) fn build_resolution_row(
        &mut self,
        selected_options: &Option<HashSet<XYCombo>>,
    ) -> Row<WallpaperMessage> {
        let get_is_toggled = |option: &XYCombo| -> bool {
            match selected_options {
                None => false,
                Some(options) => options.contains(option),
            }
        };
        self.button_states
            .iter_mut()
            .group_by(|(res, _)| calculate_aspect_ratio(res.x, res.y))
            .into_iter()
            .fold(Row::new(), |row, ((x, y), resolutions)| {
                row.push(resolutions.fold(
                    Column::new().push(Text::new(format!("{}:{}", x, y))),
                    |column, (res, btn_state)| {
                        column.push(
                            make_button(btn_state, &res.to_string())
                                .style(inactive_style(get_is_toggled(&res)))
                                .on_press(WallpaperMessage::ResolutionSelected(res.clone())),
                        )
                    },
                ))
            })
    }
}
