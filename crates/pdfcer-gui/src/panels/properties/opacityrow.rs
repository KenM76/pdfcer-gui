//! # `panels::properties::opacityrow` — one 0–100 % opacity field, or the
//! sentence saying the file's opacity cannot be read
//!
//! An opacity the engine could not resolve (an `/ExtGState` that does not
//! resolve, or a `/ca` that is not a number) gets **no field**: a field must
//! open on some value, and one opened on a guess overwrites a value nobody saw.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/properties/opacityrow.md`.

use egui::Ui;
use pdfcer_gui_base::entry;

use crate::text::strokestyle as t;

/// One opacity across a selection.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum Alpha {
    /// Every member's opacity was read: the first member's, and whether any
    /// other differs.
    Read {
        /// 0.0 (invisible) to 1.0 (opaque).
        first: f64,
        /// Some member differs from `first`.
        mixed: bool,
    },
    /// At least one member's opacity could not be read.
    Unknown,
}

impl Alpha {
    /// The selection's opacity, `None` for an empty selection. `None` items
    /// are members whose opacity could not be read; one makes it
    /// [`Alpha::Unknown`].
    pub(super) fn of(values: impl IntoIterator<Item = Option<f64>>) -> Option<Self> {
        let mut it = values.into_iter();
        let Some(first) = it.next()? else {
            return Some(Self::Unknown);
        };
        let mut mixed = false;
        for v in it {
            match v {
                None => return Some(Self::Unknown),
                Some(v) => mixed |= (v - first).abs() > 1e-6,
            }
        }
        Some(Self::Read { first, mixed })
    }

    /// The trace token: `unknown`, `mixed`, or the value to three places.
    pub(super) fn token(self) -> String {
        match self {
            Self::Unknown => "unknown".to_owned(), // ui-text-exempt: trace token
            Self::Read { mixed: true, .. } => "mixed".to_owned(), // ui-text-exempt: trace token
            Self::Read { first, .. } => format!("{first:.3}"),
        }
    }
}

/// The row. `Some` (as 0..=1) once, when an edit ends on a new value. The
/// field publishes `region`; the unreadable sentence publishes
/// `{region}.unknown`, so a check can tell which state it is looking at.
pub(super) fn row(ui: &mut Ui, label: String, alpha: Alpha, region: &str) -> Option<f64> {
    let mut chosen = None;
    ui.horizontal(|ui| {
        ui.label(label);
        let Alpha::Read { first, mixed } = alpha else {
            let said = ui.label(egui::RichText::new(t::opacity_unknown()).small().weak());
            crate::diag::ui_rect_visible(&format!("{region}.unknown"), said.rect, ui.clip_rect());
            return;
        };
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let shown = (first * 100.0).round().clamp(0.0, 100.0) as u8;
        let mut percent = shown;
        let (widget, refusal) = entry::drag_value(ui, &mut percent, entry::Kind::Number(&["%"]));
        let response = refusal.show(
            ui.add(
                widget
                    .range(0..=100)
                    .speed(1.0)
                    .suffix(crate::text::panels::properties::markup_opacity_suffix()),
            ),
        );
        crate::diag::ui_rect_visible(region, response.rect, ui.clip_rect());
        if (response.drag_stopped() || response.lost_focus()) && percent != shown {
            chosen = Some(f64::from(percent) / 100.0);
        }
        if mixed {
            ui.label(egui::RichText::new(t::mixed()).small().weak());
        }
    });
    chosen
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_unreadable_member_makes_the_whole_selection_unknown() {
        assert_eq!(Alpha::of([Some(0.5), None]), Some(Alpha::Unknown));
        assert_eq!(Alpha::of([None, Some(0.5)]), Some(Alpha::Unknown));
    }

    /// The counterpart: if `Unknown` were returned unconditionally the test
    /// above would still pass.
    #[test]
    fn readable_members_read_as_their_value_and_mixed_only_when_they_differ() {
        assert_eq!(
            Alpha::of([Some(0.5), Some(0.5)]),
            Some(Alpha::Read {
                first: 0.5,
                mixed: false
            })
        );
        assert_eq!(
            Alpha::of([Some(0.5), Some(1.0)]),
            Some(Alpha::Read {
                first: 0.5,
                mixed: true
            })
        );
        assert_eq!(Alpha::of(std::iter::empty()), None);
    }

    #[test]
    fn tokens_name_each_state() {
        assert_eq!(Alpha::Unknown.token(), "unknown");
        assert_eq!(
            Alpha::of([Some(0.25)]).map(Alpha::token).as_deref(),
            Some("0.250")
        );
        assert_eq!(
            Alpha::of([Some(0.25), Some(1.0)])
                .map(Alpha::token)
                .as_deref(),
            Some("mixed")
        );
    }
}
