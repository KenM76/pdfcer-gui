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

use egui::Ui;
use pdfcer_core::pageops::SeparationPolicy;
use pdfcer_core::settings::MissingAppearanceState;

use super::{Draft, widgets};
use crate::text::settings as t;

/// What happens when only some plates of a separated page survive an edit.
///
/// # The one setting in this group that changes SAVED BYTES
///
/// Its radius line says *"Affects the file you save"* and it is the only
/// unqualified such line in the window outside the *Saving files* group. That
/// matters for the same reason the whole radius discipline does: a choice whose
/// consequence is a different file on disk is a different kind of decision from
/// one that changes a preview, and an operator must be able to tell from the
/// words which they are making.
///
/// # Why the enum comes from `pageops` and not from `settings`
///
/// `SeparationPolicy` is the only one of the thirteen whose type lives outside
/// the settings module, because it is a parameter to a page operation that
/// happens to be configurable rather than a configuration that happens to be
/// consumed. Importing it from where it lives keeps that legible; re-exporting
/// it through `settings` would suggest the settings module owns the behaviour,
/// which it deliberately does not — *"this module reads and writes; it does not
/// define."*
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
///
/// # ★ The guess disclosure here is inverted, and deliberately
///
/// Every other setting's default note says whether *that default* is a guess.
/// This one says that **the other two options are** — because the shipped
/// default is a refusal to guess, and the thing the operator needs to know is
/// that picking either alternative means accepting an invented answer.
///
/// Making one of them the default would be exactly the *sneaky* failure the
/// disclosure rule forbids: the operator would see a plausible appearance — a
/// ticked box, an "off" stamp — **with no indication that pdfcer chose it**. The
/// spec notes are explicit on the point: *do NOT silently pick first, `Off`, or
/// `On`.* Offering them as opt-ins is legitimate; shipping one is not.
///
/// Whatever is chosen, the case stays **counted**, and pdfcer never writes an
/// `/AS` to repair the file — the document is malformed and pdfcer's job is to
/// say so, not to quietly make it conforming.
///
/// # What this setting is NOT about
///
/// A **single-entry** appearance subdictionary is not covered and never was:
/// with one entry there are no alternatives, so painting it is not a guess.
/// Only the multi-entry case — where Table 164 makes `/AS` *required*, so the
/// file is malformed — reaches this setting.
///
/// # `FirstEntry` means the producer's first, not an alphabetical invention
///
/// It uses the dictionary's own iteration order, which pdfcer preserves from the
/// file. That is worth knowing because it makes the option *slightly* less
/// arbitrary than it sounds — and only slightly, which is why the note still
/// says nothing guarantees the order is meaningful.
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
