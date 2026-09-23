//! **What the Print window remembers between jobs** — `OPERATOR_REQUESTS.md`
//! **O166**, the operator's words of 2026-09-10: *"the printer dialogue box
//! needs to remember our last settings."*
//!
//! [`PrintDialog::habits`] reduces the open dialog to the subset of its state
//! that would still be the right answer for a **different document**;
//! [`PrintDialog::remember`] writes that subset to the preferences file. The
//! reading half is in [`super::PrintDialog::open`], which seeds every one of
//! those fields from what it is handed.
//!
//! # ★★ The second direction, added at O185
//!
//! [`PrintDialog::restore`] writes [`super::PrintDialog::opened_with`] back --
//! the settings the window opened with -- and it exists because Cancel has to
//! *undo*, not merely *decline*. The reason it has anything to undo is on that
//! field: a Print the driver refuses saves before it spools and leaves the
//! window open, so the preferences can genuinely hold values this window put
//! there and the operator then rejected.
//!
//! Both directions are one private writer, [`PrintDialog::store`], which is
//! also the sole emitter of the `print-remembered` trace. That is the point of
//! the split rather than a tidiness argument: two copies of
//! compare/assign/save/trace would be two places for the disclosure to drift,
//! and the failure mode of a drifted disclosure is a trace that reports the
//! wrong direction while looking entirely well-formed.
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
            poster: self.poster,
            lines: self.lines,
        }
    }

    /// Write [`Self::habits`] into the preferences file.
    ///
    /// Returns whether the preferences **now hold** those settings — which is
    /// `true` both when they were written and when they already matched, and
    /// `false` only when a write was attempted and the disk refused it. See
    /// [`Self::store`] for why this is the right question for *this* direction
    /// and the wrong one for the other.
    ///
    /// # The failure is swallowed, and that matches every other preference
    ///
    /// [`crate::app::actions::prefs`] states the rule this follows, as the
    /// fourth of the four properties every preference verb shares: *"one discrete
    /// operator decision is one write, and losing a preference across a restart
    /// does not justify a modal in front of somebody who is"* — here — *about
    /// to print*. A read-only `userdata` folder must not turn a print into an
    /// error dialog; the job is the operator's actual errand and it proceeds
    /// unchanged.
    ///
    /// ★ The swallowed failure is nevertheless **reported**, off-canvas, by the
    /// trace this returns into and by `print-dismissed saved=` at the window's
    /// single return. Rule 4 is *fuzzy, never sneaky*: not raising a modal is a
    /// decision about interruption, not a licence to be silent.
    ///
    /// # Nothing is written when nothing changed
    ///
    /// Reprinting the same job with the same answers is the commonest print
    /// there is, and rewriting the whole preferences file on each one buys
    /// nothing. The comparison is a plain `!=` on
    /// [`crate::app::prefs::PrintPrefs`], which is why that type derives
    /// `PartialEq`.
    pub(super) fn remember(&self, prefs: &mut crate::app::prefs::Prefs) -> bool {
        self.store(prefs, self.habits(), "habits").stored
    }

    /// Put the preferences back to [`super::PrintDialog::opened_with`] — the
    /// undoing half of `OPERATOR_REQUESTS.md` **O185**.
    ///
    /// # ★★ It returns both facts, because Cancel's disclosure needs both
    ///
    /// [`Written::changed`] answers the operator's question — *were my changes
    /// undone?* — and is `false` for a Cancel on a window nobody touched, which
    /// is the commonest Cancel there is. Reporting `reverted=true` for that one
    /// would leave the trace unable to tell the two apart.
    ///
    /// [`Written::stored`] answers the other question, which is whether the disk
    /// took it, and it is what `print-dismissed saved=` reports. ⚠ The two are
    /// independent and neither implies the other: a revert that put the settings
    /// back in memory and could not write them is `changed = true, stored =
    /// false`, and that pair is the honest account — the operator's *session*
    /// genuinely has been reverted, and the next launch will not agree.
    ///
    /// # ★★★ Why this is not a `bool`, said here because it was one for an hour
    ///
    /// The single boolean was `changed`. That left [`super::dismissal`]'s Revert
    /// arm with nothing to report `saved=` from, so it passed a hard-coded
    /// `false` — while **this doc comment claimed, in the same commit, that a
    /// disk failure "is disclosed by the `saved=` field of the same trace
    /// line"**. It was not. That field read `false` on every revert, one that
    /// wrote successfully and one that could not write at all, and the line was
    /// well-formed either way.
    ///
    /// A comment promising a disclosure the code does not make is the defect this
    /// project has corrected more often than any other, and this instance is worth
    /// the paragraph because of *how* it survived: the comment reads as a
    /// description of the arm, the arm reads as an implementation of the comment,
    /// and neither reading opens the other file.
    pub(super) fn restore(&self, prefs: &mut crate::app::prefs::Prefs) -> Written {
        self.store(prefs, self.opened_with.clone(), "restored")
    }

    /// **The only writer of `prefs.print`, and the only emitter of the
    /// `print-remembered` trace.**
    ///
    /// `how` is the stable token the trace reports the direction under:
    /// `habits` for [`Self::remember`], `restored` for [`Self::restore`]. A
    /// driven check reading a `print-remembered` line without it could not tell
    /// a Keep from a Cancel, because the two are identical in every other field
    /// whenever the window opened on the settings it is being asked to keep.
    ///
    /// # ★★★ Why it returns two booleans rather than one
    ///
    /// Because the two callers are asking different questions and only one of
    /// them is about the write:
    ///
    /// | caller | the operator's question | field |
    /// |---|---|---|
    /// | [`Self::remember`] | *"are my settings kept?"* | `stored` |
    /// | [`Self::restore`] | *"were my changes undone?"* | `changed` |
    ///
    /// They disagree on exactly the case that matters: **nothing to do**. A
    /// Keep on settings already on disk kept them (`stored`), and a Cancel on
    /// an untouched window undid nothing (`!changed`). One boolean would have
    /// to pick one of those, and whichever it picked the other caller would be
    /// reporting the opposite of the truth — in a trace line, where nobody
    /// would see it, because a well-formed field reads as a measured one.
    fn store(
        &self,
        prefs: &mut crate::app::prefs::Prefs,
        wanted: crate::app::prefs::PrintPrefs,
        how: &'static str,
    ) -> Written {
        if prefs.print == wanted {
            return Written {
                changed: false,
                stored: true,
            };
        }
        prefs.print = wanted;
        let saved = prefs.save();
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "print-remembered how={} saved={} orientation={:?} duplex={:?} paper={} \
                 scale={} copies={} subset={:?}",
                how,
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
        Written {
            changed: true,
            stored: saved.is_ok(),
        }
    }
}

/// What a call to [`PrintDialog::store`] actually did.
///
/// Two independent facts that a single `bool` kept conflating. See `store`'s
/// own table for which caller reads which, and why picking one would make the
/// other caller's disclosure a quiet lie.
pub(super) struct Written {
    /// The preferences did not already hold the wanted value, so it was
    /// written over and the `print-remembered` trace was emitted.
    pub(super) changed: bool,
    /// The preferences now hold the wanted value. False only when a write was
    /// attempted and [`crate::app::prefs::Prefs::save`] refused — a read-only
    /// or missing `userdata` folder.
    pub(super) stored: bool,
}
