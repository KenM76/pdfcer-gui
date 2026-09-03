//! # `app::actions::textannot` — authoring the annotations that carry WORDS
//!
//! The sticky note, the text box and the stamp: the three markup kinds whose
//! gesture ends in a dialog the operator types into, rather than on a mouse
//! release. Split out of [`super::apply`] under **R2** on 2026-08-28, when
//! author-time opacity took that file past 1,500 lines for the sixth time.
//!
//! ## ★★ Why THIS arm, when four markup arms sit beside it
//!
//! Because it is the only one of the four that **composes** rather than routes.
//! `CommitMarkup` and `CommitTextMarkup` each build a spec and call the engine
//! — six lines apiece, and their own headers say the arm *"routes; it does not
//! compute"*. This one resolves an author from preferences, reads a clock,
//! builds a `MarkupNote`, builds a `MarkupOptions`, decides whether the text is
//! empty, and traces two separate facts about what it wrote. That is a subject,
//! not a routing table entry, and a subject is what R2 asks a file to be about.
//!
//! ⇒ Moving one of the six-line arms instead would have made the count work and
//! left the thing that grows in the file that keeps overflowing.
//!
//! ## What is deliberately NOT here
//!
//! **Editing** a note on an annotation that already exists —
//! `super::annots::set_note` and `clear_note`. Authoring and editing are
//! different subjects however much they share the noun, which is the same line
//! `super::annot`'s header draws for the whole annotation family.

use pdfcer_core::edit::MarkupOptions;

use super::apply::vector_edit;

use crate::app::prefs::Prefs;
use crate::app::state::OpenDoc;
use crate::canvas::textannot::TextAnnotKind;

/// **Author a text-bearing annotation** — a sticky note, a text box or a stamp
/// — signed, dated, and at the pen's opacity.
///
/// `ink` and `opacity` are read from the live pen by the caller rather than
/// carried on the action, and that is the OPPOSITE of `CommitMarkup`'s rule.
/// The difference is real: this is raised by a **dialog the operator has been
/// sitting in**, and applied on the frame they press Accept, so there is no
/// window across which the value could go stale. `CommitMarkup` is raised by a
/// gesture that finished frames before the queue drains.
/// **What the operator placed** — the four values that come off the action,
/// grouped so the function that consumes them takes four arguments rather than
/// nine.
///
/// ★ A struct rather than a longer parameter list, and not only to satisfy a
/// lint: `page`, `kind`, `rect` and `stamp` are **one thing the operator did**,
/// while `prefs` and the pen are settings that happen to be in scope. A
/// signature that mixed all six in a row would let a caller transpose two of
/// them silently, which on `(usize, …)` positions is the class of mistake that
/// compiles.
pub(super) struct Placement {
    /// The page it goes on.
    pub page: usize,
    /// Sticky note, text box or stamp.
    pub kind: TextAnnotKind,
    /// Where, in PDF user space.
    pub rect: pdfcer_core::page_tree::Rect,
    /// Which stamp face, for the stamp kind.
    pub stamp: pdfcer_core::annot_author::StampName,
}

pub(super) fn commit(
    doc: &mut OpenDoc,
    prefs: &Prefs,
    placed: &Placement,
    text: &str,
    ink: (f64, f64, f64),
    opacity: Option<f64>,
) {
    let Placement {
        page,
        kind,
        rect,
        stamp,
    } = *placed;
    // ★ The pen's ink, so a callout matches the comments beside it
    // and one Style group governs the whole markup family.
    //
    // Read here rather than carried on the action, which is the
    // OPPOSITE of `CommitMarkup`'s rule two arms up — and the
    // difference is real. That action is raised by a gesture that
    // completed frames before the queue drains, so the live pen may
    // have moved under it. This one is raised by a DIALOG the
    // operator has been sitting in, and is applied on the same
    // frame they pressed Accept. There is no window for the value
    // to go stale across.

    // ★★★ **The note the operator just typed, signed and dated.**
    //
    // `add_text_annotation_with` rather than the bare verb, and the
    // difference is three keys: `/Contents`, `/T` and `/M`.
    //
    // # Why the text is passed TWICE, which looks like a mistake
    //
    // The spec already carries it — a sticky's `/Contents` is what
    // its popup shows, a `/FreeText`'s is what is painted — and
    // `MarkupOptions::note` writes `/Contents` again over the top.
    // Identical bytes, so the file is unchanged by the duplication.
    //
    // ⇒ The note is passed anyway because **`/T` and `/M` are only
    // reachable through it.** The engine writes the three as a
    // group or not at all, so a shell that wanted an author had to
    // supply the text with it. Splitting them would be a change to
    // `pdfcer-core`, and asking for one to avoid re-passing a string
    // this frame already holds is not a case worth making.
    //
    // # ★★ The author is a PREFERENCE and may be empty
    //
    // Empty writes no `/T`, which is legal and is exactly what
    // every annotation this shell authored before today did. It is
    // not a defect to leave it unset — an anonymous comment is a
    // real choice — so there is no nag and no default guessed from
    // the OS user account.
    //
    // # ★ The date is UTC and may be absent
    //
    // `app::clock` carries the whole argument, including why a
    // local time labelled `Z` was the one option ruled out. `None`
    // means the system clock is before 1970, and omitting `/M`
    // beats writing a comment dated 1969.
    // ★ Builders, not a struct literal: `MarkupNote` is
    // `#[non_exhaustive]`, which is what keeps a future field a
    // non-breaking addition for us. `by` and `at` take the value,
    // so both are applied conditionally rather than passed as
    // `Option`.
    let mut note = pdfcer_core::edit::MarkupNote::new(text);
    let author = prefs.author_name.trim();
    if !author.is_empty() {
        note = note.by(author);
    }
    if let Some(stamp) = crate::app::clock::pdf_date_utc() {
        note = note.at(stamp);
    }
    let options = MarkupOptions {
        note: Some(note),
        // ★ The pen's opacity reaches the sticky note, the text box
        // and the stamp as well, and it has to: a stamp is the
        // markup most likely to be placed over drawing content, and
        // an operator who set the group's opacity and found it
        // applied to four kinds out of seven would be right to call
        // that broken. One control, one meaning, every kind.
        opacity,
    };
    if let Some(spec) = crate::canvas::textannot::spec(kind, rect, text, stamp, ink) {
        // ★★ The note's three keys, on the diagnostic channel and
        // NOT on the status line. An operator who typed a comment
        // does not need to be told their own name was written; a
        // driven check needs to know it, because `/T` and `/M` are
        // invisible on the page by construction — a sticky's words
        // live in a popup and its author lives nowhere at all
        // until a reviewer UI draws a column.
        //
        // ⇒ Without this line the feature has NO oracle short of
        // parsing the saved file. It is the same argument
        // `markup_move`'s `keys=` makes for the half of a move a
        // screenshot cannot see.
        let signed = !prefs.author_name.trim().is_empty();
        let dated = crate::app::clock::pdf_date_utc().is_some();
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "text-annot-note chars={} signed={signed} dated={dated}",
                text.chars().count()
            )
        });
        vector_edit(doc, "add-text-annot", page, 1, |session| {
            session
                .add_text_annotation_with(page, &spec, &options)
                .map(|_| Vec::new())
        });
    } else {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("text-annot-declined kind={kind:?} reason=no-text")
        });
    }
}
