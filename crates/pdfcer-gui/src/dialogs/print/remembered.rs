//! **What the Print window remembers between jobs** — `OPERATOR_REQUESTS.md`
//! **O166**, the operator's words of 2026-09-10: *"the printer dialogue box
//! needs to remember our last settings."*
//!
//! Two functions. [`PrintDialog::habits`] reduces the open dialog to the subset
//! of its state that would still be the right answer for a **different
//! document**; [`PrintDialog::remember`] writes that subset to the preferences
//! file. The reading half is in [`super::PrintDialog::open`], which seeds every
//! one of those fields from what it is handed.
//!
//! # Why this is its own file rather than part of [`super::commit`]
//!
//! `commit` is what calls `remember`, so proximity would be defensible. But the
//! subject here is not *printing* — it is a judgement about **which of this
//! window's twenty controls describe the operator rather than the document**,
//! and that judgement has a long argument attached to it. Splitting it out
//! under rule R2 was the opportunity to put it where its argument is.
//!
//! The rule itself, and the reasoning for every inclusion and every omission,
//! lives on [`crate::app::prefs::PrintPrefs`]. Read that first; this file is
//! only the projection.
//!
//! # The two properties that keep this honest
//!
//! - **Writing is compiler-enforced.** `habits` is a struct literal with no
//!   `..Default::default()`, so a field added to `PrintPrefs` stops this file
//!   building rather than being silently written out as its own default.
//! - **Reading is test-enforced.** Nothing about the *other* direction is
//!   visible to the compiler: a field that `PrintDialog::open` never mentions
//!   compiles perfectly and is simply inert. That gap is closed by
//!   `crate::app::prefs::printing::tests::every_remembered_field_is_read_back_by_the_print_dialog`,
//!   which reads the struct declaration out of the source rather than carrying
//!   a hand-written list of its own.

use super::PrintDialog;

impl PrintDialog {
    /// **This dialog's state, reduced to what a different document would still
    /// want** — the producing half of `OPERATOR_REQUESTS.md` **O166**.
    ///
    /// The membership rule and the argument for every inclusion and every
    /// omission live on [`crate::app::prefs::PrintPrefs`], which is the type
    /// this returns; this function is only the projection. It is written as one
    /// struct literal with no `..Default::default()` so that a field added to
    /// `PrintPrefs` is a **compile error here** rather than a preference that
    /// is written to disk as its default and never actually remembered.
    ///
    /// ⚠ [`Self::device`] is read rather than [`Self::effective_device`]. The
    /// operator's *choice* is what is remembered — "match the pages" — never
    /// the sheet O167's arithmetic resolved it to on this one document. See
    /// `effective_device`'s own note on why those two are kept apart.
    pub(super) fn habits(&self) -> crate::app::prefs::PrintPrefs {
        crate::app::prefs::PrintPrefs {
            printer: self.printers.get(self.selected).map(|p| p.name.clone()),
            orientation: self.device.orientation,
            duplex: self.device.duplex,
            pick_tray_by_page_size: self.device.pick_tray_by_page_size,
            // A hand-picked `Form(id)` is stored as "from the printer's own
            // settings" — `paper_key` does that reduction and argues it. It is
            // deliberate loss, not a gap.
            paper: self.device.paper,
            scale: self.scale,
            custom_percent: self.custom_percent,
            scope: self.scope,
            max_dpi: self.max_dpi,
            copies: self.copies,
            uncollated: self.uncollated,
            subset: self.subset,
            reverse: self.reverse,
        }
    }

    /// Write [`Self::habits`] into the preferences file.
    ///
    /// # The failure is swallowed, and that matches every other preference
    ///
    /// `Action::SetFindZoom` states the rule this follows: *"one discrete
    /// operator decision is one write, and losing a preference across a restart
    /// does not justify a modal in front of somebody who is"* — here — *about
    /// to print*. A read-only `userdata` folder must not turn a print into an
    /// error dialog; the job is the operator's actual errand and it proceeds
    /// unchanged.
    ///
    /// # Nothing is written when nothing changed
    ///
    /// Reprinting the same job with the same answers is the commonest print
    /// there is, and rewriting the whole preferences file on each one buys
    /// nothing. The comparison is a plain `!=` on
    /// [`crate::app::prefs::PrintPrefs`], which is why that type derives
    /// `PartialEq`.
    pub(super) fn remember(&self, prefs: &mut crate::app::prefs::Prefs) {
        let habits = self.habits();
        if prefs.print == habits {
            return;
        }
        prefs.print = habits;
        let saved = prefs.save();
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "print-remembered saved={} orientation={:?} duplex={:?} paper={} \
                 scale={} copies={} subset={:?}",
                saved.is_ok(),
                prefs.print.orientation,
                prefs.print.duplex,
                // ★ Stable lowercase tokens, never `{:?}`, for the two fields a
                // driven check reads back. This project's standing lesson:
                // never `Debug`-format a field a machine reads — a `{:?}` on a
                // payload-carrying variant prints the payload too, and a check
                // grepping `scale=custom` would miss `scale=Custom(2.5)` while
                // quoting the truth in its own failure message.
                crate::app::prefs::printing::paper_key(prefs.print.paper),
                crate::app::prefs::printing::scale_key(prefs.print.scale),
                prefs.print.copies,
                prefs.print.subset,
            )
        });
    }
}
