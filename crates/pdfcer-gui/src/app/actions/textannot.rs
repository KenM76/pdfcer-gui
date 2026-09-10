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
/// **What the operator placed** — the six values that come off the action,
/// grouped so the function that consumes them takes four arguments rather than
/// ten.
///
/// ★ A struct rather than a longer parameter list, and not only to satisfy a
/// lint: `page`, `kind`, `rect`, `stamp`, `stamp_size` and `icon` are **one
/// thing the operator did**, while `prefs` and the pen are settings that happen
/// to be in scope. A signature that mixed all eight in a row would let a caller
/// transpose two of them silently, which on `(usize, …)` positions is the class
/// of mistake that compiles.
///
/// ⚠ The doc above said **four values** and **six arguments** until
/// 2026-09-06, then five and seven; it now describes **six** and **eight**.
/// Corrected each time rather than left, because the sentence's whole point is
/// the count — a prose count that has drifted from the struct is worse than no
/// count, since it reads as a measurement.
pub(super) struct Placement {
    /// The page it goes on.
    pub page: usize,
    /// Sticky note, text box or stamp.
    pub kind: TextAnnotKind,
    /// Where, in PDF user space.
    pub rect: pdfcer_core::page_tree::Rect,
    /// Which stamp face, for the stamp kind.
    pub stamp: pdfcer_core::annot_author::StampName,
    /// ☑ **How big the stamp's label is drawn** (engine `Pass 287.0`), for
    /// the stamp kind.
    ///
    /// `stamp`'s and `icon`'s third sibling, and it arrived by the same route
    /// they did: dialog → `Action::CommitTextAnnot` → here →
    /// `crate::canvas::textannot::spec`. Ignored by the two kinds it does not
    /// belong to, and unconditional rather than `Option`al because a chooser
    /// always has a selection — here the selection that means *"let the box
    /// decide"*, which is a value, not an absence.
    pub stamp_size: crate::canvas::textannot::StampSize,
    /// ★ Which icon (`/Name`, §12.5.6.4 Table 172), for the sticky kind.
    ///
    /// The exact counterpart of `stamp` one field up, and it arrived by
    /// following that field's route: dialog → `Action::CommitTextAnnot` → here
    /// → `crate::canvas::textannot::spec`. Both are ignored by the two kinds
    /// they do not belong to, and both are unconditional rather than
    /// `Option`al because a chooser always has a selection.
    pub icon: pdfcer_core::annot_author::StickyIcon,
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
        stamp_size,
        ref icon,
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
    // ★★★ **…and "identical" is now enforced by the engine, on pain
    // of the whole gesture being refused.** As of `pdfcer-core`
    // `95a936e`, passing a `TextAnnotSpec::FreeText { text }` and a
    // `MarkupOptions::note` whose words differ is
    // `EditError::FreeTextNoteConflictsWithText` — refused before
    // anything is written, because for that one subtype the two
    // arguments are the same PDF key and there is no defensible way
    // to pick a winner. This shell reported the trap; this is the
    // side of it that has to stay clear of the muzzle.
    //
    // ⇒ Both strings therefore come from
    // `crate::canvas::textannot::painted_text`, ONE function, rather
    // than from one variable normalised in one of its two readers.
    // Before 2026-09-06 they did not: `spec` trimmed and
    // `MarkupNote::new` did not, so a text box typed with a trailing
    // space — a stray space bar, invisible in the box — would have
    // been refused outright, authoring nothing and showing the
    // operator only the generic decline sentence. The engine turned a
    // latent mess into a loud refusal, and the refusal found this.
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
    //
    // ★ **Shadowed, not a second name.** `text` the parameter is gone from
    // this scope after this line, so a later reader cannot reach the raw
    // string even by accident — the two callers below have nothing else to
    // pass. A `let words = …` beside a live `text` would have left the
    // mistake representable, and it is a mistake that compiles.
    let text = crate::canvas::textannot::painted_text(text);
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
        // ★ No dash on a note, a text box or a stamp, and that is a decision
        // rather than a default taken by omission. `MarkupOptions::dash`
        // arrived 2026-09-06 and applies to any mark; these three are the ones
        // whose border is a *container* for words rather than a drawn line, and
        // a dashed box around a comment reads as provisional — which R8b
        // forbids content from doing. The dashed control belongs to the shapes.
        dash: None,
        // ★ The pen's opacity reaches the sticky note, the text box
        // and the stamp as well, and it has to: a stamp is the
        // markup most likely to be placed over drawing content, and
        // an operator who set the group's opacity and found it
        // applied to four kinds out of seven would be right to call
        // that broken. One control, one meaning, every kind.
        opacity,
    };
    if let Some(spec) =
        crate::canvas::textannot::spec(kind, rect, text, stamp, icon, stamp_size, ink)
    {
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
        // ★★★ A ROTATED PAGE — 2026-09-09. The operator, on his 25-sheet
        // Ghostscript drawing whose every page carries `/Rotate 90`: *"when I
        // try to put a stamp on the drawing … the text comes out vertical, and
        // there is no control to set the angle or horizontal."*
        //
        // `/Rotate` is a DISPLAY rotation (Table 30): the page's content is
        // authored in unrotated user space and the reader turns the whole
        // sheet clockwise by that amount when it draws it. An annotation
        // appearance authored upright in user space is therefore turned with
        // it — a stamp reads sideways on every sheet of a landscape drawing
        // that was exported portrait. Acrobat pre-rotates its own stamps and
        // text boxes by the page's rotation for exactly this reason.
        //
        // The engine's authoring verbs do not consult `/Rotate` (they author
        // in user space, correctly); its rotation verb composes a rotation
        // into the appearance `/Matrix` (`set_annotation_rotation`, absolute,
        // degrees counter-clockwise in user space, about a pivot). So the two
        // are composed here, in ONE undo step: author, then turn the new mark
        // by +`rotate` about its own centre, which the display's clockwise
        // turn then cancels. A sticky is excluded — §12.5.6.4 makes it
        // `NoRotate`, so a reader draws its icon upright whatever the page
        // does, and rotating its appearance would be the one way to make it
        // come out sideways.
        //
        // ★ The pivot is the rect's centre, so the mark stays where the
        // operator dragged it; the engine derives the new upright `/Rect`.
        let rotate = doc.pages.get(page).map(|p| p.rotate).unwrap_or(0);
        let upright_turn = (rotate != 0 && kind != crate::canvas::textannot::TextAnnotKind::Sticky)
            .then_some(f64::from(rotate));
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!("text-annot-page-rotate page={page} rotate={rotate} turn={upright_turn:?}")
        });
        vector_edit(doc, "add-text-annot", page, 1, |session| {
            let id = session.add_text_annotation_with(page, &spec, &options)?;
            if let Some(deg) = upright_turn {
                let pivot = ((rect.llx + rect.urx) / 2.0, (rect.lly + rect.ury) / 2.0);
                let turned = session.set_annotation_rotation(id, pivot, deg)?;
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed
                    format!(
                        "text-annot-uprighted id={} deg={deg} applied={}",
                        id.num, turned.degrees
                    )
                });
            }
            Ok::<Vec<String>, pdfcer_core::edit::EditError>(Vec::new())
        });
    } else {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("text-annot-declined kind={kind:?} reason=no-text")
        });
    }
}

#[cfg(test)]
mod tests {
    //! The two stamp reports of 2026-09-09, each as a test that was RED on the
    //! code it corrects: *"the text comes out vertical"* on a `/Rotate 90`
    //! page, and *"still can't adjust the size of a stamp on the canvas, or by
    //! entering a different size in the properties box"*. Both routes end in
    //! [`super::super::annots::resize`] / [`super::commit`], so both are unit
    //! tests over those functions with a real document — no window, no
    //! pointer; the driven check is owed separately.

    use super::*;
    use crate::app::state::open_local_fixture;
    use crate::canvas::selection::{AnnotKind, AnnotSelection, AnnotTarget};
    use crate::canvas::textannot::TextAnnotKind;
    use pdfcer_core::annot::page_annotations;
    use pdfcer_core::annot_author::StampName;
    use pdfcer_core::page_tree::Rect;

    /// Place an `Approved` stamp at `rect` on page 0 and return its id and the
    /// `/Rect` the engine wrote.
    fn place_stamp(
        doc: &mut crate::app::state::OpenDoc,
        rect: Rect,
    ) -> (pdfcer_core::object::ObjId, Rect) {
        let placed = Placement {
            page: 0,
            kind: TextAnnotKind::Stamp,
            rect,
            stamp: StampName::Approved,
            // The gallery's own default, so these tests measure the stamp an
            // operator actually gets rather than one this module invented.
            stamp_size: crate::canvas::textannot::DEFAULT_STAMP_SIZE,
            icon: crate::canvas::textannot::DEFAULT_STICKY_ICON,
        };
        commit(
            doc,
            &Prefs::default(),
            &placed,
            "APPROVED",
            (0.8, 0.1, 0.1),
            None,
        );
        let page = doc.pages[0].clone();
        let found = page_annotations(&doc.session.graph(), page.id)
            .into_iter()
            .find(|a| a.subtype.as_slice() == b"Stamp")
            .expect("the stamp was authored");
        (
            found.id.expect("an indirect annotation"),
            found.rect.expect("a /Rect"),
        )
    }

    fn select(doc: &mut crate::app::state::OpenDoc, id: pdfcer_core::object::ObjId) {
        doc.selection.select_annot(AnnotSelection {
            target: AnnotTarget {
                page: 0,
                id,
                kind: AnnotKind::Markup,
                subtype: "Stamp".to_owned(),
                locked: false,
            },
            outline: egui::Rect::NOTHING,
            oriented: None,
        });
    }

    /// ★★★ **A stamp on a `/Rotate 90` page is authored turned by 90°, so the
    /// reader's clockwise display turn brings it upright.** RED before the
    /// fix: the appearance had no rotation and read sideways on every sheet
    /// of the operator's drawing set.
    #[test]
    fn a_stamp_on_a_rotated_page_is_authored_upright() {
        let mut doc = open_local_fixture("rotated-90.pdf");
        assert_eq!(
            doc.pages[0].rotate, 90,
            "the fixture is the operator's page shape"
        );
        let (id, _) = place_stamp(
            &mut doc,
            Rect {
                llx: 100.0,
                lly: 100.0,
                urx: 300.0,
                ury: 160.0,
            },
        );
        let view = doc.session.view();
        let oriented = crate::canvas::annotquad::oriented_by_id(&view, &doc.pages[0], id)
            .expect("the stamp has an appearance");
        let degrees = oriented
            .degrees
            .expect("the appearance matrix is a rotation");
        assert!(
            (degrees - 90.0).abs() < 0.5 || (degrees - 450.0).abs() < 0.5,
            "the stamp should be pre-turned by the page's rotation, got {degrees}"
        );
        assert!(doc.session.can_undo(), "author + turn is one undo step");
    }

    /// ★★★ **A stamp resizes — proportionally and not — with the default
    /// modifiers, from the same `resize` the canvas grips and the Properties
    /// width/height fields both raise.** RED before the fix: the engine
    /// refused the carried appearance as "foreign" because neither Tool-panel
    /// switch was set, and the sentence said pdfcer had not drawn it.
    #[test]
    fn a_stamp_resizes_with_the_default_modifiers() {
        let mut doc = open_local_fixture("four-pages.pdf");
        let (id, before) = place_stamp(
            &mut doc,
            Rect {
                llx: 100.0,
                lly: 100.0,
                urx: 300.0,
                ury: 160.0,
            },
        );
        select(&mut doc, id);
        // Proportional, from the lower-left anchor.
        super::super::annots::resize(
            &mut doc,
            id,
            (before.llx, before.lly),
            (1.5, 1.5),
            true,
            crate::canvas::scaling::Modifiers::default(),
        );
        let after = page_annotations(&doc.session.graph(), doc.pages[0].id)
            .into_iter()
            .find(|a| a.id == Some(id))
            .and_then(|a| a.rect)
            .expect("still there");
        assert!(
            ((after.urx - after.llx) - 1.5 * (before.urx - before.llx)).abs() < 0.5,
            "a proportional resize of a stamp must land: {before:?} -> {after:?}"
        );
        // Non-proportional — the case that needs `allow_appearance_distortion`,
        // and for a picture of text that IS the resize the operator asked for.
        super::super::annots::resize(
            &mut doc,
            id,
            (after.llx, after.lly),
            (2.0, 1.0),
            false,
            crate::canvas::scaling::Modifiers::default(),
        );
        let stretched = page_annotations(&doc.session.graph(), doc.pages[0].id)
            .into_iter()
            .find(|a| a.id == Some(id))
            .and_then(|a| a.rect)
            .expect("still there");
        assert!(
            ((stretched.urx - stretched.llx) - 2.0 * (after.urx - after.llx)).abs() < 0.5
                && ((stretched.ury - stretched.lly) - (after.ury - after.lly)).abs() < 0.5,
            "a non-proportional resize of a stamp must land too: {after:?} -> {stretched:?}"
        );
    }
}
