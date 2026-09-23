//! **What the three export windows remember between jobs** —
//! `OPERATOR_REQUESTS.md` **O196**, the operator's words of 2026-09-13:
//!
//! > *"the export windows forget everything. every time I export a dxf I have
//! > to set it up again."*
//!
//! This file is the **writing** half, shared by all three windows. The reading
//! half lives in each window's own `open`, which seeds every remembered field
//! from what it is handed.
//!
//! # Why one file for three windows rather than three `remembered.rs`
//!
//! [`crate::dialogs::print::remembered`] is the precedent and it is one window,
//! so it put its argument beside its projection. Here the projection is three
//! different struct literals — an image format is nothing like a DXF unit — but
//! **the argument is identical three times**, and an argument written out three
//! times is an argument that will be corrected in one of them.
//!
//! So the split is by *what varies*:
//!
//! | half | where | why |
//! |---|---|---|
//! | `habits()` — the projection | each window's own file | it reads that window's private fields, and the membership judgement is about *those controls* |
//! | `remember_*()` — the write | here | the no-op guard, the swallowed failure, the position of the call and the trace's shape are one decision made once |
//!
//! # The three properties this file is responsible for
//!
//! ## 1. Nothing is written when nothing changed
//!
//! Exporting the same page twice with the same answers is the commonest export
//! there is, and rewriting the whole preferences file on each one buys nothing.
//! The comparison is a plain `!=` on the group struct, which is why
//! [`crate::app::prefs::ExportImagePrefs`] and its two siblings derive
//! `PartialEq`.
//!
//! ⚠ The comparison is **per group**, not on the whole of
//! [`crate::app::prefs::ExportPrefs`]. An operator who exports a DXF and then
//! an image must not have the image write suppressed because the DXF group is
//! unchanged — and, in the other direction, a DXF export must not rewrite the
//! file merely because the image group differs from what it was at startup.
//!
//! ## 2. The failure is swallowed
//!
//! [`crate::app::actions::prefs`] states the rule, as the fourth of the four
//! properties every preference verb shares: *"one discrete operator decision is
//! one write, and losing a preference across a restart does not justify a modal
//! in front of somebody who is"* — here — *about to export*. A read-only
//! `userdata` folder must not turn an export into an error dialog; the export
//! is the operator's actual errand and it proceeds unchanged.
//!
//! ## 3. The trace spells values as TOKENS, never `{:?}`
//!
//! This project's standing lesson, learned on the print window: never
//! `Debug`-format a field a machine reads. A `{:?}` on a payload-carrying
//! variant prints the payload too, so a driven check grepping `scale=custom`
//! misses `scale=Custom(2.5)` **while quoting the truth in its own failure
//! message** — a confident false negative that reads as an application defect.
//!
//! Every value below goes through the same `*_key` function the preferences
//! file itself uses, so the token a driven check reads and the token on disk
//! cannot drift.
//!
//! # ★ Where these are CALLED, which is the part that is a decision
//!
//! At the **Export press**, immediately before the `Action` is pushed — never
//! when the window closes.
//!
//! Closing without exporting is how a person says *"not this"*: they opened the
//! window, changed the resolution, thought better of it, and cancelled.
//! Persisting on close would make that abandoned configuration the state the
//! next export opens in, which is the opposite of what cancelling means.
//!
//! That is [`crate::dialogs::print::PrintDialog::remember`]'s ruling, applied
//! unchanged, and it is stated here as well because the three call sites are in
//! three other files and a rule visible only from the print window is a rule
//! the next export window will not find.

use crate::app::prefs::exporting;
use crate::app::prefs::{ExportDxfPrefs, ExportImagePrefs, ExportTextPrefs, Prefs};

/// Persist the Export-image window's habits.
///
/// Takes the group **by value** rather than by reference: the caller has just
/// built it out of its own fields and has no further use for it, and a move is
/// what makes the assignment below a store rather than a clone.
pub(super) fn remember_image(habits: ExportImagePrefs, prefs: &mut Prefs) {
    if prefs.export.image == habits {
        return;
    }
    prefs.export.image = habits;
    let saved = prefs.save();
    let image = &prefs.export.image;
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "export-image-remembered saved={} format={} scope={} dpi={} transparent={} quality={} keep_text={}",
            saved.is_ok(),
            exporting::image_format_key(image.format),
            // The same reduction the preferences file performs, and it must be
            // the same one: a check that reads this line and then reads the
            // file would otherwise see `Typed` disclosed here and `current` on
            // disk, and conclude the write had gone wrong.
            exporting::page_scope_key_or(image.scope, ExportImagePrefs::default().scope),
            image.dpi,
            u8::from(image.transparent),
            image.quality,
            u8::from(image.keep_text),
        )
    });
}

/// Persist the Export-text window's habits.
pub(super) fn remember_text(habits: ExportTextPrefs, prefs: &mut Prefs) {
    if prefs.export.text == habits {
        return;
    }
    prefs.export.text = habits;
    let saved = prefs.save();
    let text = &prefs.export.text;
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "export-text-remembered saved={} scope={} separator={} endings={} bom={}",
            saved.is_ok(),
            exporting::page_scope_key_or(text.scope, ExportTextPrefs::default().scope),
            exporting::separator_key(text.separator),
            exporting::line_endings_key(text.line_endings),
            u8::from(text.byte_order_mark),
        )
    });
}

/// Persist the Export-DXF window's habits.
///
/// ⚠ **No `scale=` here, and its absence is the design rather than an
/// oversight.** The DXF scale is derived per open from the page's own
/// dimension groups and is deliberately not a remembered preference — see
/// [`crate::app::prefs::ExportDxfPrefs`] for the argument. A trace key naming
/// a value this function does not store would be the first place somebody
/// looked when the scale failed to survive a restart, and it would tell them
/// the opposite of the truth.
pub(super) fn remember_dxf(habits: ExportDxfPrefs, prefs: &mut Prefs) {
    if prefs.export.dxf == habits {
        return;
    }
    prefs.export.dxf = habits;
    let saved = prefs.save();
    let dxf = &prefs.export.dxf;
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "export-dxf-remembered saved={} units={} arcs={} text={}",
            saved.is_ok(),
            exporting::dxf_units_key(dxf.units),
            u8::from(dxf.fit_arcs),
            exporting::dxf_text_key(dxf.text),
        )
    });
}
