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
// The stamp-fit sentences, shared verbatim with the properties panel's
// restyle route: one act described in one place, so the two surfaces
// cannot drift into wording the operator has to reconcile.
use crate::text::panels::textannotstyle as ts;
use pdfcer_core::annot_author::StampLabelFit;

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

/// **What the placing act tells the operator afterwards** — the surviving half
/// of R8b rule 4, read off `TextAnnotOutcome` (`pdfcer-core` `Pass 291.0`).
///
/// # ★★★ What is said, what is deliberately NOT, and why the silences are the argued half
///
/// Rule 4 has two clauses that pull in opposite directions and only one of
/// them is about drawing. The forbidden half — no badge, no tint, no dashed
/// outline, nothing that would make a screenshot of the editing canvas differ
/// from a screenshot of the same file saved and reopened — is honoured here by
/// construction: this function returns `String`s and touches no painter. The
/// owed half is *"an inference the operator cannot see still gets a sentence,
/// off-canvas"*, and that is what this decides, outcome by outcome.
///
/// # The three fields, and what each one is worth saying
///
/// | field | said? | why |
/// |---|---|---|
/// | `unencodable_chars` | **yes**, when non-zero | the `?` is visible; *that pdfcer put it there* is not |
/// | `stamp_label_fit` = `LabelShrunk` / `LabelClipped` | **yes** | the operator cannot tell a label drawn small from one they asked for small |
/// | `stamp_label_fit` = `BoxGrown` | **no** | a wider stamp is visible as itself, and the dialog said so before the drag |
/// | `stamp_label_fit` = `AsRequested` | **no** | nothing was decided for anyone |
/// | `applied_autosize` | **no** | always `None` here; see below |
///
/// ★★★ **`BoxGrown` is the interesting silence, and it is argued rather than
/// overlooked.** It is an inference — pdfcer chose a rectangle the operator
/// did not drag — so the reflex is to disclose it. Two things say not to. The
/// stamp is *visibly* wider than the box that was dragged, which is the test
/// rule 4 states: a screenshot of this canvas matches a screenshot of the
/// saved file, and the difference from the drag is on the screen in front of
/// them. And `text::textannot::stamp_size_bound` already tells them, in the
/// dialog, **before** they commit — so a sentence afterwards would be pdfcer
/// telling the operator a thing it had just told them, which is the exact
/// failure mode that teaches somebody to stop reading the status line. The
/// nagging in the old GUI is on the record as having cost real visibility
/// bugs; this is where that lesson is spent.
///
/// ★★ **The restyle route rules the opposite way on the same variant, and both
/// are right.** [`crate::app::actions::annots::set_text_annot_style`] does
/// disclose `BoxGrown`, because there the rectangle is **existing content
/// pdfcer changed** rather than a request in progress — a stamp that has sat
/// on the page for months, at a size the operator chose, whose right edge
/// moves because they typed a number into a properties field. Nothing
/// forewarned them, and nothing about the act said "and the box will grow".
/// ⇒ The question is never *"can they see it?"* — they can see it on both
/// routes — but *"did they author it, in the act they just performed?"* That
/// module's own comment carries the long form.
///
/// ⚠ **`LabelShrunk` and `LabelClipped` are unreachable from this route
/// today**, because the placing dialog offers no fit policy and every
/// `StampSize` variant carries the engine's default `GrowToText`. They are
/// handled anyway, and that is deliberate: the day a fit control appears on
/// this dialog — or the day the engine changes which policy `StampStyle`
/// defaults to — the disclosure must already be here, or the feature ships
/// silent. A branch that is currently dead is cheaper than a rule 4 breach
/// that is currently invisible.
///
/// ★ `StampLabelFit` is `#[non_exhaustive]`. A fourth outcome this build has
/// no words for still answers `is_inference()` and lands on the `_` arm, which
/// says something true and vague rather than nothing at all. Silence there
/// would be the worst of the three options: an inference that happened,
/// reported as though it had not.
///
/// ★★ `applied_autosize` is not read, and the engine says why in as many
/// words: it is the **variable-text** auto-size, `None` whenever `/DA` names
/// an explicit size, and a stamp's fitted size is always written as an
/// explicit size. It is `None` on every stamp, always — which is the measured
/// fact that produced `Pass 291.0` in the first place. A `/FreeText` this
/// shell authors never asks for `0 Tf`, so it is `None` there too. Reading it
/// would be a field that can only ever say nothing.
/// ★★★ **Takes the two facts rather than the `TextAnnotOutcome` that carries
/// them, and that is about being testable at all.**
///
/// `TextAnnotOutcome` is `#[non_exhaustive]`, so no code outside `pdfcer-core`
/// can build one — which would make every assertion below reachable only by
/// opening a document, authoring a real annotation and hoping the engine
/// produced the outcome the case needs. That is a test of the engine wearing a
/// test of this function's rules, and the rules are where the operator-visible
/// decisions live. `StampLabelFit`'s variants **are** constructible, so passing
/// the field lets each silence be asserted directly, including the ones that
/// are unreachable from today's placing dialog.
///
/// ★ The caller therefore does the unwrapping, in one place, in sight of the
/// engine call. That is the seam this project already uses for the same reason
/// elsewhere: the impure read stays where the session is, the decision stays
/// pure.
fn disclosures(unencodable_chars: usize, fit: Option<&StampLabelFit>) -> Vec<String> {
    let mut said = Vec::new();
    if let Some(line) = crate::text::textannot::placed_unencodable(unencodable_chars) {
        said.push(line);
    }
    // ★★ `is_inference()` first, which is the ENGINE's question rather than a
    // re-derivation of it, and then the match narrows to the two outcomes a
    // screenshot cannot show. Written as a guard plus a match rather than as
    // one match with three silent arms, so that a new variant added upstream
    // falls into `_` *inside* the inference branch and is spoken about,
    // instead of being swallowed by an `AsRequested`-shaped catch-all.
    if let Some(fit) = fit
        && fit.is_inference()
    {
        match fit {
            StampLabelFit::LabelShrunk { size, requested } => {
                said.push(ts::stamp_label_shrunk(*size, *requested));
            }
            StampLabelFit::LabelClipped { hidden_chars, .. } => {
                said.push(ts::stamp_label_clipped(*hidden_chars));
            }
            // Silent, and argued in this function's header: visible as itself,
            // and forewarned in the dialog before the drag was committed.
            StampLabelFit::BoxGrown { .. } => {}
            _ => said.push(ts::stamp_label_fit_unknown().to_owned()),
        }
    }
    said
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
            // ★★★ **`_reporting`, not `_with`, and the difference is three
            // disclosures this route dropped on the floor until 2026-09-10.**
            //
            // The three entry points do identical work, take identical
            // guards and leave one identical undo entry; they differ only in
            // what they hand back. `add_text_annotation_with` returns the new
            // object's id, which is the shape forty call sites in the engine
            // use, and it is the shape this shell reached for because it was
            // the one named in the example. `add_text_annotation_reporting`
            // returns a `TextAnnotOutcome`, and the engine's own doc says
            // what only this route can tell you.
            //
            // ⚠ It is the same class of mistake as taking
            // `..Default::default()` on a struct that grew a field: nothing
            // fails, nothing warns, and a capability is declined on the
            // operator's behalf without a word appearing anywhere. There is
            // no compiler between `_with` and `_reporting` — both compile,
            // both author the same annotation, and only one of them can say
            // what it did.
            //
            // ★ Costs nothing. `_with` is literally
            // `_reporting(...).map(|o| o.annot_id)`, so this is the same call
            // with the discard removed.
            let out = session.add_text_annotation_reporting(page, &spec, &options)?;
            let id = out.annot_id;
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
            Ok::<Vec<String>, pdfcer_core::edit::EditError>(disclosures(
                out.unencodable_chars,
                out.stamp_label_fit.as_ref(),
            ))
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

    /// ★★★ **The operator's own sentence, asked twice, and the half that
    /// was still open on 2026-09-10:** *"still can't adjust the size of a stamp
    /// on the canvas, or by entering a different size in the properties box."*
    ///
    /// The first half of that sentence — the canvas grips — was answered on
    /// 2026-09-09 by [`super::super::annots::resize`], and the test below this
    /// one asserts it. **The second half was not**, and the reason was a real
    /// gap rather than an oversight: until `pdfcer-core` `Pass 292.0` a stamp
    /// already on the page had a label size that could be neither read nor
    /// written. There was no verb to call.
    ///
    /// ★★ This test drives BOTH new verbs against a real document and asserts
    /// they agree with each other, which is the property a panel depends on
    /// and neither verb can guarantee alone. `set_text_annot_style` writing
    /// `/DA` is worth nothing if `stamp_label_parameters` reads a different
    /// number back — the properties spinner would then be seeded with a value
    /// the operator did not type, on the very next frame, and would look like
    /// the edit had failed.
    ///
    /// ⚠ Deliberately NOT a test of the widget. It asserts the round trip the
    /// widget sits on top of; whether a `DragValue` commits on `drag_stopped`
    /// is a driven-check question and this project's founding rule says a
    /// passing unit test is not a report of working software. That check is
    /// owed and is recorded as owed.
    #[test]
    fn a_stamp_already_on_the_page_takes_a_new_label_size_and_reads_it_back() {
        let mut doc = open_local_fixture("four-pages.pdf");
        let (id, _) = place_stamp(
            &mut doc,
            Rect {
                llx: 100.0,
                lly: 400.0,
                urx: 260.0,
                ury: 450.0,
            },
        );

        // ★★ The size the stamp starts at is DERIVED, not stated — the
        // gallery's default is `FitTheBox` — so this is the shape of stamp
        // every build before `Pass 287.0` produced and the majority of the
        // stamps on the operator's drawings. Starting from a stated size would
        // have tested the easy case.
        let before = doc
            .session
            .stamp_label_parameters(id)
            .expect("a stamp reports its label parameters")
            .expect("this stamp's appearance paints words");
        assert_eq!(before.label, "APPROVED");
        assert!(
            before.size > 0.0,
            "a size read off a baked appearance is still a size: {before:?}"
        );

        // ★★ Through the SHELL's function, not the engine's, so the undo
        // entry, the epoch bump and the texture drop are exercised with it.
        // `doc.session` is an `Arc` and cannot be borrowed mutably here at
        // all, which is the type system enforcing the same thing: every write
        // goes through the funnel.
        super::super::annots::set_text_annot_style(
            &mut doc,
            id,
            &pdfcer_core::edit::TextAnnotStyle {
                font_size: Some(30.0),
                stamp_fit: Some(pdfcer_core::annot_author::StampFit::GrowToText),
                color: None,
                icon: None,
            },
        );

        let after = doc
            .session
            .stamp_label_parameters(id)
            .expect("still readable")
            .expect("still painting words");
        assert!(
            (after.size - 30.0).abs() < 0.01,
            "the size he typed must be the size the file states, or the spinner reseeds itself \
             with a number he did not choose: asked 30, read {}",
            after.size
        );
        assert_eq!(
            after.label, "APPROVED",
            "and the WORDS must survive a size change untouched — a restyle that replaced the \
             label is the defect this shell already shipped once"
        );
    }

    /// ★★★ **What the placing act says, and — the harder half — what it deliberately does not.**
    ///
    /// Every one of these is a **silence** or a **sentence**, and the silences
    /// are the ones worth asserting: a sentence that goes missing is noticed
    /// the first time somebody uses the feature, while a sentence that appears
    /// where the header argued for silence is nagging — the failure mode the
    /// old GUI is on the record for, and the one that costs real visibility
    /// bugs rather than a shrug.
    mod placing_disclosures {
        use super::super::disclosures;
        use pdfcer_core::annot_author::StampLabelFit;

        /// The ordinary case says nothing at all.
        #[test]
        fn a_stamp_that_fitted_as_asked_is_not_talked_about() {
            let said = disclosures(0, Some(&StampLabelFit::AsRequested { size: 18.0 }));
            assert!(
                said.is_empty(),
                "reporting a label that fitted at the size the operator chose is reporting their \
                 own instruction back at them, and it teaches them to stop reading the line: \
                 {said:?}"
            );
        }

        /// ★★★ **The argued silence, and it is asserted rather than left to the comment above it.**
        ///
        /// `BoxGrown` answers `is_inference() == true`, so the reflex reading of
        /// R8b rule 4 says disclose it. The header argues the opposite on two
        /// grounds — it is visible on the canvas as itself, and the dialog said
        /// it would happen before the drag was committed — and an argument in a
        /// comment is one an editor can delete by agreeing with the reflex. This
        /// is the argument in a form that goes red.
        #[test]
        fn a_grown_box_is_silent_because_it_is_visible_and_was_forewarned() {
            let grown = StampLabelFit::BoxGrown {
                size: 24.0,
                width: 180.0,
            };
            assert!(
                grown.is_inference(),
                "the engine calls a grown box an inference, and this test's whole point is that \
                 this shell stays silent about one anyway — if that stops being true the silence \
                 below is no longer a decision, it is an accident"
            );
            let said = disclosures(0, Some(&grown));
            assert!(
                said.is_empty(),
                "a stamp the operator can SEE is wider than the box they dragged, having been \
                 told in the dialog that it would be, does not also get a sentence afterwards: \
                 {said:?}"
            );
        }

        /// The two a screenshot cannot show DO get a sentence.
        #[test]
        fn a_shrunk_or_clipped_label_is_always_disclosed() {
            let shrunk = disclosures(
                0,
                Some(&StampLabelFit::LabelShrunk {
                    size: 9.0,
                    requested: 24.0,
                }),
            );
            assert_eq!(shrunk.len(), 1, "one sentence, not none and not two");
            assert!(
                shrunk[0].contains('9') && shrunk[0].contains("24"),
                "the sentence has to carry BOTH numbers — the size drawn and the size asked for \
                 — because the operator cannot tell them apart by looking: {shrunk:?}"
            );

            let clipped = disclosures(
                0,
                Some(&StampLabelFit::LabelClipped {
                    size: 24.0,
                    hidden_chars: 3,
                    overflow: 40.0,
                }),
            );
            assert_eq!(clipped.len(), 1);
            assert!(
                clipped[0].contains('3'),
                "the count is the whole value of the sentence: {clipped:?}"
            );
        }

        /// ★★ **The gap this pass found**, and the reason it existed: the placing
        /// path called the entry point that returns an id and drops the outcome, so
        /// an annotation was substituted in silence while a form field with the same
        /// character had said so for months.
        #[test]
        fn an_unencodable_character_is_named_even_though_the_question_mark_is_visible() {
            let said = disclosures(2, None);
            assert_eq!(
                said.len(),
                1,
                "a `?` pdfcer substituted looks exactly like a `?` the operator typed, and only \
                 one of those is worth investigating: {said:?}"
            );
            assert!(
                said[0].contains('2'),
                "the count says how much of the mark to check: {said:?}"
            );
            assert!(
                disclosures(0, None).is_empty(),
                "and nothing is said when nothing was substituted"
            );
        }

        /// ★ Two facts, two sentences — not one summary.
        #[test]
        fn the_two_kinds_of_disclosure_are_independent() {
            let said = disclosures(
                1,
                Some(&StampLabelFit::LabelShrunk {
                    size: 8.0,
                    requested: 36.0,
                }),
            );
            assert_eq!(
                said.len(),
                2,
                "a substitution and a shrink are two separate facts about one mark, and a shell \
                 that reported only the first would leave the operator believing the size they \
                 typed is the size on the page: {said:?}"
            );
        }
    }
}
