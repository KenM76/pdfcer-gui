//! # `dialogs::settings::saving` — three settings nobody can see
//!
//! All three change the **bytes pdfcer writes** and none of them changes
//! anything visible. That is stated in all three radius lines in nearly the same
//! words, and it is the whole reason they are grouped together rather than filed
//! with the settings whose effects an operator can look at.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/settings/saving.md`.

use egui::Ui;
use pdfcer_core::settings::{QuadPointOrder, TrailingEol, XrefEntryEol};

use super::{Draft, widgets};
use crate::text::settings as t;

/// How each cross-reference entry's line ends.
pub fn xref_entry_eol(ui: &mut Ui, draft: &mut Draft) {
    widgets::header(
        ui,
        t::xref_eol_title(),
        t::xref_eol_silence(),
        t::xref_eol_radius(),
    );
    widgets::option(
        ui,
        &mut draft.working.xref_entry_eol,
        XrefEntryEol::MatchSource,
        t::xref_eol_match_label(),
        Some(t::xref_eol_match_note()),
    );
    widgets::option(
        ui,
        &mut draft.working.xref_entry_eol,
        XrefEntryEol::SpaceLf,
        t::xref_eol_space_lf_label(),
        Some(t::xref_eol_space_lf_note()),
    );
    widgets::option(
        ui,
        &mut draft.working.xref_entry_eol,
        XrefEntryEol::SpaceCr,
        t::xref_eol_space_cr_label(),
        None,
    );
    widgets::option(
        ui,
        &mut draft.working.xref_entry_eol,
        XrefEntryEol::CrLf,
        t::xref_eol_cr_lf_label(),
        None,
    );
}

/// Whether one byte follows the end-of-file marker.
pub fn trailing_eol(ui: &mut Ui, draft: &mut Draft) {
    widgets::header(
        ui,
        t::trailing_eol_title(),
        t::trailing_eol_silence(),
        t::trailing_eol_radius(),
    );
    widgets::option(
        ui,
        &mut draft.working.trailing_eol,
        TrailingEol::Lf,
        t::trailing_eol_lf_label(),
        Some(t::trailing_eol_lf_note()),
    );
    widgets::option(
        ui,
        &mut draft.working.trailing_eol,
        TrailingEol::None,
        t::trailing_eol_none_label(),
        Some(t::trailing_eol_none_note()),
    );
}

/// Which corner order `/QuadPoints` gets — spec ambiguity `QP-A1`.
pub fn quad_point_order(ui: &mut Ui, draft: &mut Draft) {
    widgets::header(
        ui,
        t::quad_order_title(),
        t::quad_order_silence(),
        t::quad_order_radius(),
    );
    widgets::option(
        ui,
        &mut draft.working.quad_point_order,
        QuadPointOrder::ReadingOrder,
        t::quad_order_reading_label(),
        Some(t::quad_order_reading_note()),
    );
    widgets::option(
        ui,
        &mut draft.working.quad_point_order,
        QuadPointOrder::Counterclockwise,
        t::quad_order_ccw_label(),
        Some(t::quad_order_ccw_note()),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Only the three legal entry forms are offered.
    #[test]
    fn only_the_legal_entry_forms_are_offered() {
        // `resolve` needs a base document to observe; `MatchSource` against an
        // empty base falls back to `SpaceLf`, which is the documented
        // behaviour for a file with no such index.
        for form in [
            XrefEntryEol::MatchSource,
            XrefEntryEol::SpaceLf,
            XrefEntryEol::SpaceCr,
            XrefEntryEol::CrLf,
        ] {
            let bytes = form.resolve(&[]).bytes();
            assert_eq!(
                bytes.len(),
                2,
                "{form:?} does not encode as two bytes and would break the 20-byte entry"
            );
            assert!(
                matches!(bytes, [b' ', b'\n'] | [b' ', b'\r'] | [b'\r', b'\n']),
                "{form:?} encodes as {bytes:?}, which is not one of the three legal forms"
            );
        }
    }

    /// `MatchSource` with nothing to match falls back to a legal fixed form.
    #[test]
    fn matching_nothing_falls_back_to_the_documented_form() {
        assert_eq!(
            XrefEntryEol::MatchSource.resolve(&[]).bytes(),
            [b' ', b'\n']
        );
    }
}
