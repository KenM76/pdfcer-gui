//! # `dialogs::settings::pages` — plates, and controls with no stated look
//!
//! Two settings that have little in common except where an operator would look
//! for them: both are about a document being *processed* rather than read.
//!
//! ## They are also two different shapes of ambiguity, and the copy keeps them apart
//!
//! - **Separations** is not a spec ambiguity at all. §14.11.4 is perfectly
//!   clear about the invariant; what it does not say is what an *editor* should
//!   do when an edit breaks it. It is a setting because all three answers are
//!   defensible for different workflows — **product policy**, not silence.
//! - **Missing appearance state** is a genuine silence, and a peculiar one: the
//!   file in question is *malformed*, and the standard states no recovery.
//!
//! Blurring the two would make the window's whole framing dishonest, since the
//! intro paragraph promises that everything below exists *because the standard
//! declines to have an opinion*. The separations silence line therefore says
//! "says nothing about what an editor should do" rather than "does not define",
//! which is the accurate sentence and reads no worse.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/settings/pages.md`.

use egui::Ui;
use pdfcer_core::pageops::SeparationPolicy;
use pdfcer_core::settings::MissingAppearanceState;

use super::{Draft, widgets};
use crate::text::settings as t;

/// What happens when only some plates of a separated page survive an edit.
pub fn separations(ui: &mut Ui, draft: &mut Draft) {
    widgets::header(
        ui,
        t::separations_title(),
        t::separations_silence(),
        t::separations_radius(),
    );
    widgets::option(
        ui,
        &mut draft.working.separations,
        SeparationPolicy::Repair,
        t::separations_repair_label(),
        Some(t::separations_repair_note()),
    );
    widgets::option(
        ui,
        &mut draft.working.separations,
        SeparationPolicy::Discard,
        t::separations_discard_label(),
        Some(t::separations_discard_note()),
    );
    widgets::option(
        ui,
        &mut draft.working.separations,
        SeparationPolicy::Refuse,
        t::separations_refuse_label(),
        Some(t::separations_refuse_note()),
    );
}

/// What to draw for a control that carries several appearances and names none.
pub fn missing_as(ui: &mut Ui, draft: &mut Draft) {
    widgets::header(
        ui,
        t::missing_as_title(),
        t::missing_as_silence(),
        t::missing_as_radius(),
    );
    widgets::option(
        ui,
        &mut draft.working.missing_as,
        MissingAppearanceState::PaintNothing,
        t::missing_as_nothing_label(),
        Some(t::missing_as_nothing_note()),
    );
    widgets::option(
        ui,
        &mut draft.working.missing_as,
        MissingAppearanceState::FirstEntry,
        t::missing_as_first_label(),
        Some(t::missing_as_first_note()),
    );
    widgets::option(
        ui,
        &mut draft.working.missing_as,
        MissingAppearanceState::OffElseNothing,
        t::missing_as_off_label(),
        Some(t::missing_as_off_note()),
    );
}
