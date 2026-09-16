//! # `dialogs::settings::forms` — the order Tab visits a form in
//!
//! Two settings, both about one question: *"I pressed Tab and it went to the
//! wrong field."* That sentence is the filing rule for this group — an
//! operator who comes here has come from a keyboard, not from a menu, and
//! neither setting is discoverable from anywhere else in the window.
//!
//! ## Why they are their own group rather than joining Pages
//!
//! Every other group in this window answers *"what is in the document"* or
//! *"what does pdfcer do with it"*. These two answer *"what happens when I
//! press a key"*, and the only other settings of that shape — the mouse wheel,
//! the paste chords — live under **Drawing the page** because their values are
//! shell preferences stored in a different file. These are engine settings,
//! and filing them under a heading about drawing because that is where the
//! other keyboard settings ended up would be organising the window by our
//! storage rather than by the operator's question.
//!
//! ## The standard contradicts itself, and pdfcer says so either way
//!
//! `/Tabs /W` is described twice in ISO 32000-2, in two clauses that give
//! different answers and neither of which cites the other. `pdfcer-core`'s
//! `WidgetTabTail` carries the full reading; what matters here is that the
//! disclosure does not depend on this control. Whichever way it is set, a
//! sequence computed for a `/W` page names the reading that was applied.
//! Changing it therefore cannot make pdfcer quieter about the ambiguity — it
//! only chooses which side of it this machine takes.

use egui::Ui;
use pdfcer_core::settings::{MAX_TAB_ROW_TOLERANCE, MIN_TAB_ROW_TOLERANCE, WidgetTabTail};

use super::{Draft, widgets};
use crate::text::settings as t;

/// Which reading of `/Tabs /W` this machine follows.
pub fn tab_tail(ui: &mut Ui, draft: &mut Draft) {
    widgets::header(
        ui,
        t::tab_tail_title(),
        t::tab_tail_silence(),
        t::tab_tail_radius(),
    );
    widgets::option(
        ui,
        &mut draft.working.widget_tab_tail,
        WidgetTabTail::ArrayOrder,
        t::tab_tail_array_label(),
        Some(t::tab_tail_array_note()),
    );
    widgets::option(
        ui,
        &mut draft.working.widget_tab_tail,
        WidgetTabTail::RowOrder,
        t::tab_tail_row_label(),
        Some(t::tab_tail_row_note()),
    );
}

/// How far apart two fields' leading edges may be and still count as one row.
///
/// A slider rather than a typed number, and not because typing is hard: the
/// useful range is a single inch and the useful resolution is coarse, so the
/// control that shows the whole range at once is the one that answers the
/// question *"is a point enough?"* without the operator having to know what
/// the bounds are.
///
/// Linear, for [`super::measuring::parallel`]'s reason — half a point against
/// one point matters exactly as much as ten against twenty, because both
/// answer *how crooked may this form be before I stop calling it a row?*
pub fn row_tolerance(ui: &mut Ui, draft: &mut Draft) {
    widgets::header(
        ui,
        t::tab_tolerance_title(),
        t::tab_tolerance_silence(),
        t::tab_tolerance_radius(),
    );
    ui.add(
        egui::Slider::new(
            &mut draft.working.tab_row_tolerance,
            MIN_TAB_ROW_TOLERANCE..=MAX_TAB_ROW_TOLERANCE,
        )
        .suffix(t::point_suffix())
        .text(t::tab_tolerance_slider_label()),
    );
    ui.label(egui::RichText::new(t::tab_tolerance_note()).small().weak());
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfcer_core::settings::Settings;

    /// The line terminator a settings file uses.
    const NEWLINE: char = '\n';

    /// **The slider's range is the STORE's, and a hand-edited legal value
    /// survives opening this window.**
    ///
    /// The regression test for the silent-edit hazard every slider in this
    /// window carries: a control whose range is narrower than what the engine
    /// accepts rewrites a value the operator chose deliberately, the moment
    /// they open the window to look at something else. `egui::Slider` clamps
    /// on draw, so the narrowing would be silent and the value would be gone.
    #[test]
    fn the_slider_spans_everything_the_store_accepts() {
        for text in [
            "tab_row_tolerance = 0.0",
            "tab_row_tolerance = 72.0",
            "tab_row_tolerance = 2.5",
        ] {
            let mut notes = Vec::new();
            let parsed = Settings::parse(&format!("{text}{}", NEWLINE), &mut notes);
            assert!(
                notes.is_empty(),
                "the store clamped a value this window offers: {notes:?}"
            );
            assert!(
                (MIN_TAB_ROW_TOLERANCE..=MAX_TAB_ROW_TOLERANCE).contains(&parsed.tab_row_tolerance),
                "the store accepts {} and the slider does not",
                parsed.tab_row_tolerance
            );
        }
    }
}
