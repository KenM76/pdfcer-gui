//! # `dialogs::insert_where` — where inserted pages land
//!
//! The four radios both insert dialogs ask, and their mapping onto the
//! engine's `InsertPosition`. One copy, so "before" and "after" cannot mean
//! different things in the two dialogs.

use egui::Ui;
use pdfcer_core::pageops::InsertPosition;

use crate::text::pages as t;

/// Where the pages land, as the radios offer it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Where {
    /// Before the page the operator is looking at.
    BeforeCurrent,
    /// After it. **The default**, because it is what "insert here" means to
    /// somebody who navigated to a sheet first.
    AfterCurrent,
    /// Before every existing page.
    Start,
    /// After every existing page.
    End,
}

impl Where {
    /// The engine's position, with `current` the 0-based page the dialog was
    /// opened on.
    pub(crate) const fn position(self, current: usize) -> InsertPosition {
        match self {
            Self::BeforeCurrent => InsertPosition::Before(current),
            Self::AfterCurrent => InsertPosition::After(current),
            Self::Start => InsertPosition::Start,
            Self::End => InsertPosition::End,
        }
    }

    /// Draw the heading and the four radios. Each radio's rect is published
    /// as `{region_prefix}.where.{n}`, `n` in drawing order (after, before,
    /// start, end).
    ///
    /// The current page is named in the labels rather than left as "here",
    /// because the dialog is centred over a document the operator may have
    /// scrolled: the number is what makes the choice checkable.
    pub(crate) fn radios(&mut self, ui: &mut Ui, current: usize, region_prefix: &str) {
        ui.label(t::insert_where_heading());
        let page = current.saturating_add(1);
        for (n, (option, label)) in [
            (Self::AfterCurrent, t::insert_after_page(page)),
            (Self::BeforeCurrent, t::insert_before_page(page)),
            (Self::Start, t::insert_at_start().to_owned()),
            (Self::End, t::insert_at_end().to_owned()),
        ]
        .into_iter()
        .enumerate()
        {
            let radio = ui.radio(*self == option, label);
            crate::diag::ui_rect(&format!("{region_prefix}.where.{n}"), radio.rect);
            if radio.clicked() {
                *self = option;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The four radios produce the engine's four positions, and the two that
    /// need a page carry the RIGHT one.
    #[test]
    fn each_position_maps_to_the_engines_own() {
        assert_eq!(Where::AfterCurrent.position(6), InsertPosition::After(6));
        assert_eq!(Where::BeforeCurrent.position(6), InsertPosition::Before(6));
        assert_eq!(Where::Start.position(6), InsertPosition::Start);
        assert_eq!(Where::End.position(6), InsertPosition::End);
    }
}
