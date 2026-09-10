//! # `app::actions::customstamp` — placing one of the operator's OWN stamps
//!
//! `OPERATOR_REQUESTS.md` **O172**: *"add our own custom stamps and use them,
//! preferrably exactly the same way acrobat does."* This module is the second
//! half of that sentence. The first half — reading his stamps folder and
//! finding what is in it — is [`crate::stamps::library`]; the half that writes
//! a NEW collection for Acrobat to load is [`super::stamps`].
//!
//! ## ★★★ Why this is a module and not an arm of `super::textannot`
//!
//! It looks like one. Both place a `/Stamp` annotation, both are raised by the
//! same dialog, both take a rectangle the operator dragged, and both give the
//! new mark the same upright turn on a rotated page. The temptation to add a
//! branch to [`super::textannot::commit`] was real and was declined, for one
//! reason that is not about line counts:
//!
//! > **A standard stamp is a NAME; a custom stamp is a DOCUMENT.**
//!
//! `super::textannot` authors a `/Stamp` whose `/Name` is one of §12.5.6.12's
//! closed vocabulary and whose appearance the engine draws from that name. It
//! needs no second file, has nothing to fail at before the edit begins, and
//! cannot report anything about a source. This module opens a **second PDF**
//! off the operator's disk, fails in ways that have nothing to do with editing
//! (moved, renamed, corrupt), imports an object graph, and comes back with
//! four disclosures about what did and did not travel. Sharing a function
//! would have meant a `commit` whose first forty lines were about neither of
//! the things it does.
//!
//! ⇒ What they DO share is deliberately shared rather than duplicated: the
//! upright-turn rule for `/Rotate` pages is one argument, written once in
//! [`super::textannot`]'s header, and applied here by the same two calls.
//!
//! ## ★★ The fork — what `app::actions::apply` decides before it gets here
//!
//! One action, `Action::CommitTextAnnot`, reaches two modules. The arm that
//! chooses between them is three facts long and each is easy to get wrong:
//!
//! 1. **`custom: Some(_)` wins over `stamp`.** The gallery's two radio groups
//!    keep exactly one of the two live — a click on a standard stamp clears
//!    `custom`, a click on one of his clears nothing because `stamp` is not
//!    read on this route. The invariant is held by two call sites in
//!    `crate::dialogs::textannot::gallery` rather than by a type, which is why
//!    that function's header states it and its tests assert it.
//!
//! 2. **`kind` is NOT consulted.** The gallery is drawn only for
//!    `TextAnnotKind::Stamp`, so `custom` cannot be `Some` on a sticky note or
//!    a text box. A guard on `kind` would be a condition that can never be
//!    false, and a condition that cannot be false is a line every future
//!    reader has to prove harmless before they may change anything near it.
//!
//! 3. **The custom arm destructures with `..`.** Six of the action's values —
//!    `kind`, `text`, `stamp`, `stamp_size`, `icon` — belong to the standard
//!    route and mean nothing here. Naming them only to leave them unused would
//!    mean six underscore-prefixed bindings, which is six places for a field
//!    added later to be silently dropped on this route with no warning.
//!
//! ⚠ Why the argument is written **here** and not at the arm: `apply.rs` has
//! overflowed R2's 1,500-line ceiling six times, and the seam it keeps finding
//! is *"the reasoning goes where the mechanism is"*. The arm carries a
//! two-line pointer to this section.
//!
//! ## The order of operations, and why the load is outside the funnel
//!
//! 1. **Load the collection**, from the path the library recorded. Outside
//!    `vector_edit`, for the reason [`super::pages::insert_from_file`] gives:
//!    `place_page_artwork` borrows a `DocumentView` over it, so it must
//!    outlive the call — and a *load* failure reported from inside an *edit*
//!    closure would arrive wearing the edit's refusal sentence.
//! 2. **Work out the upright turn** from the target page's `/Rotate`.
//! 3. **One `vector_edit`**, one undo entry: place the artwork, then turn it.
//! 4. **Four disclosures**, off-canvas, on the status line.
//!
//! ## Rule 4 — what is disclosed, what is traced, and what is neither
//!
//! | fact | where it goes | why |
//! |---|---|---|
//! | `distorted` | **status line** | the shape of his signature changed and nothing on the page says so |
//! | `dynamic` | **status line** | the date on the mark is not today's, and it looks like it is |
//! | `source_widgets_ignored` | status line, unless already covered by `dynamic` | part of the artwork's design did not arrive |
//! | `source_annotations_ignored` | **status line** | same |
//! | `objects_imported` | **trace only** | see below |
//! | `resources_renamed` | trace only | permanently `0` by construction |
//! | `transparency_group_carried` | trace only | `false` means the source had none, not that one was dropped |
//!
//! ★ **`objects_imported` is traced and not said, and that is a judgement
//! worth writing down.** The engine exposes it because *"an operator stamping
//! a 5.6 MB drawing is entitled to know which act grew the file"*, and that is
//! a fair reason — but the number is a count of PDF objects, which is not a
//! size, and turning it into a sentence would mean either quoting a figure
//! that means nothing outside the format (*"imported 47 objects"*) or
//! inventing a size estimate pdfcer has not measured. Neither is a disclosure;
//! both are noise on a status line that also has to carry the three sentences
//! above. It is on the diagnostic channel, where a driven check and a
//! bug-hunting session can both reach it, and the day the operator asks *"why
//! did my file get bigger"* the answer is one grep away.
//!
//! ⚠ **Nothing is drawn onto the canvas.** R8b rule 4: a placed stamp renders
//! exactly as it will render once saved and reopened — stretched if it was
//! stretched, with last year's date if that is what its author typed. No
//! badge, no tint, no dashed outline. The report is words, elsewhere.

use pdfcer_core::edit::EditError;
use pdfcer_core::page_tree::Rect;

use super::apply::vector_edit;
use crate::app::state::OpenDoc;
use crate::stamps::library::CustomStamp;
use crate::text::stamps as t;

/// **Place `stamp`'s artwork on `page`, inside `rect`.**
///
/// `rect` is in **PDF user space** — the rectangle the canvas resolved from
/// the operator's drag, exactly as [`super::textannot::commit`] receives it.
/// The engine normalises it per §7.9.5 and hands back what it stored, so a
/// caller that passed corners in the other order is not surprised later.
///
/// # What can go wrong, and where each failure is reported
///
/// - **The collection will not open.** Reported here, by name, before any edit
///   starts — [`crate::text::stamps::place_source_unreadable`]. Nothing is
///   mutated, so there is no undo entry and no epoch bump.
/// - **The stamp's page is not in the collection.** `EditError::
///   SourcePageOutOfRange`, reported by the funnel as an ordinary decline.
///   The library recorded that index from the file's own name tree, so this
///   means the file changed under us between the scan and the press.
/// - **The document is encrypted, or certified against annotation.** The same
///   guard every annotation-authoring verb takes, and the same sentence.
pub(super) fn place(doc: &mut OpenDoc, stamp: &CustomStamp, page: usize, rect: Rect) {
    // ── 1. The collection, loaded outside the funnel ─────────────────────
    let source = match pdfcer_core::document::Document::load(&stamp.file) {
        Ok(source) => source,
        Err(error) => {
            let detail = error.to_string();
            let file = stamp.file.clone();
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!("custom-stamp-refused file={file:?} reason={detail}")
            });
            // ⊗, not ⚑. Nothing was edited — see
            // `t::CustomStampUnavailable`'s header, which carries why this was
            // a `record_note` for an afternoon and why that was wrong.
            crate::app::status::decline::record_custom_stamp_unavailable(
                t::CustomStampUnavailable::Unreadable,
            );
            return;
        }
    };
    let view = source.view();

    // ── 2. The upright turn ──────────────────────────────────────────────
    //
    // The whole argument is in `super::textannot`'s commit path and is not
    // repeated: `/Rotate` is a DISPLAY rotation, the engine authors in
    // unrotated user space, so a mark placed on a `/Rotate 90` sheet arrives
    // sideways unless it is pre-turned by the same amount. Acrobat pre-rotates
    // its own stamps for exactly this reason.
    //
    // ★ Unconditional here, where `textannot` excludes the sticky note. There
    // is no sticky-note case in this module — a custom stamp is always a
    // `/Stamp`, and §12.5.6.4's `NoRotate` exemption belongs to `/Text`.
    let rotate = doc.pages.get(page).map(|p| p.rotate).unwrap_or(0);
    let upright_turn = (rotate != 0).then_some(f64::from(rotate));

    let name = stamp.label.clone();
    let source_page = stamp.page_index;
    let dynamic = stamp.dynamic;
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "custom-stamp-requested name={name} src-page={source_page} page={page} \
             rotate={rotate} dynamic={dynamic}"
        )
    });

    // ── 3. One edit, one undo entry ──────────────────────────────────────
    //
    // ★ `vector_edit`, not `vector_edit_on_page`, and the choice is not
    // cosmetic. The narrow variant asserts that EVERY call of the verb changes
    // nothing a rasteriser would draw on any other sheet. This one imports an
    // object graph into the document's own object table — a resource closure
    // that may include fonts and images — and while today's engine attaches it
    // only to this page's new annotation, the assertion `vector_edit_on_page`
    // demands is about the verb rather than about this operand. It is not one
    // this shell can make on the engine's behalf.
    vector_edit(doc, "place-custom-stamp", page, 1, move |session| {
        // ★★★ The one refusal on this route that has a REMEDY, and therefore
        // the one worth intercepting.
        //
        // `EditError::SourcePageOutOfRange` means the collection opened and no
        // longer has the page the gallery offered — the staleness the dialog's
        // `library` field predicts in its own doc comment, because Acrobat
        // rewrites its stamps folder while pdfcer is running. Every other
        // variant this verb can return is a property of the operator's own
        // document (encrypted, certified, page gone) and lands on the funnel's
        // floor sentence, which is correct for them: there is nothing to do
        // about an encrypted file that the floor does not already imply.
        //
        // ⚠ Recorded and then re-raised, deliberately. Swallowing it would
        // leave the funnel thinking the edit succeeded: the epoch would move,
        // the page caches would be thrown away and `⚑ About your last edit`
        // would go stale — for an edit that never happened. The engine's error
        // still travels; this arm only puts a better sentence in front of it,
        // which is exactly what `vector_edit`'s *"the verb speaks first"* rule
        // is for.
        let placed = match session.place_page_artwork(&view, source_page, page, rect) {
            Ok(placed) => placed,
            Err(error) => {
                if matches!(error, EditError::SourcePageOutOfRange { .. }) {
                    crate::app::status::decline::record_custom_stamp_unavailable(
                        t::CustomStampUnavailable::PageGone,
                    );
                }
                return Err(error);
            }
        };

        if let Some(deg) = upright_turn {
            // The pivot is the rect's centre, so the mark stays where it was
            // dragged and the engine derives the new upright `/Rect`.
            let pivot = ((rect.llx + rect.urx) / 2.0, (rect.lly + rect.ury) / 2.0);
            let turned = session.set_annotation_rotation(placed.annot_id, pivot, deg)?;
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!(
                    "custom-stamp-uprighted id={} deg={deg} applied={}",
                    placed.annot_id.num, turned.degrees
                )
            });
        }

        // ★★ The measurement line. Its first token is `custom-stamp-placed`,
        // deliberately NOT `place-custom-stamp` — the funnel writes a line
        // under the bare label and `Trace::last(name)` matches on the first
        // token, so sharing the name would hand a driven check the funnel's
        // `page= n= epoch= disclosures=` when it asked for `scale-x=`.
        // `tools/gates/check-trace-names.py` enforces the suffix convention;
        // its header records the three times this was got wrong before the
        // gate existed.
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!(
                "custom-stamp-placed id={} form={} scale-x={:.4} scale-y={:.4} \
                 distorted={} imported={} renamed={} annots-ignored={} \
                 widgets-ignored={} group={}",
                placed.annot_id.num,
                placed.form_id.num,
                placed.scale_x,
                placed.scale_y,
                placed.distorted,
                placed.objects_imported,
                placed.resources_renamed,
                placed.source_annotations_ignored,
                placed.source_widgets_ignored,
                placed.transparency_group_carried
            )
        });

        Ok::<Vec<String>, EditError>(disclosures(&placed, dynamic))
    });
}

/// **The words owed for one placement**, in the order they are read.
///
/// Split out so it can be tested without a document, and so the *"say it
/// unless `dynamic` already covered it"* rule for widgets sits in one place
/// rather than inside a closure inside a funnel.
///
/// # Order
///
/// Shape first, then the promise, then what did not arrive. That is the order
/// of how much each one can cost him: a stretched signature is wrong on the
/// page, a stale date is wrong in fact, and a missing form field is a design
/// detail of somebody else's stamp.
fn disclosures(placed: &pdfcer_core::edit::PlacedArtwork, dynamic: bool) -> Vec<String> {
    let mut said = Vec::new();
    if placed.distorted {
        said.push(t::placed_distorted(placed.scale_x, placed.scale_y));
    }
    if dynamic {
        said.push(t::placed_dynamic().to_owned());
    } else if placed.source_widgets_ignored > 0 {
        // ★ The `else` is the whole rule. A dynamic stamp's widgets ARE its
        // recomputed text, so on that route these are two descriptions of one
        // fact — and an operator told the same thing twice in two vocabularies
        // reasonably concludes that two different things went wrong.
        said.push(t::placed_widgets_ignored(placed.source_widgets_ignored));
    }
    if placed.source_annotations_ignored > 0 {
        said.push(t::placed_annotations_ignored(
            placed.source_annotations_ignored,
        ));
    }
    said
}

#[cfg(test)]
mod tests {
    //! The disclosure rules, over hand-built outcomes.
    //!
    //! ★ These do NOT place anything. The placement itself is the engine's
    //! verb and is tested there; what is this shell's own is the decision
    //! about which of four facts becomes a sentence, and that decision is a
    //! pure function of a `PlacedArtwork` and one boolean. A test that opened
    //! a document to reach it would be measuring the engine.
    //!
    //! ★★ The rest of the route is measured by DRIVING it, in
    //! `tools/ui-verify/src/checks/custom_stamp.rs`
    //! (`custom_stamp_reaches_the_page`): it arms Markup ▸ Stamp, presses one
    //! of the operator's own stamps in the gallery, drags a rectangle of the
    //! wrong shape, and asserts on `custom-stamp-placed distorted=true` plus
    //! the disclosure region on the status bar. That check plants a stamp
    //! collection into a scratch `%APPDATA%`, so it is neither vacuous on a
    //! machine with no stamps nor dependent on his own folder.
    //!
    //! ⚠ The two are not interchangeable. Everything above `place()` — the
    //! gallery, the selection, the fork in `apply` — is invisible to the tests
    //! in this module, which is why the driven check exists; and the driven
    //! check cannot enumerate the four disclosure rules, which is why these
    //! do. R1 — a passing unit test is not a report of working software.

    use super::*;

    /// Build a `PlacedArtwork` with the fields these rules read.
    ///
    /// `PlacedArtwork` is `#[non_exhaustive]`, so it cannot be built with a
    /// struct literal from outside its crate. It is `Copy` and every field is
    /// public, so the fixture is made by placing artwork once — which is
    /// exactly what a unit test must not do. ⇒ The rules are therefore tested
    /// through a shape this module owns instead, and the mapping from
    /// `PlacedArtwork` to it is the two-line `disclosures` signature above,
    /// which a reader can check by eye.
    ///
    /// This is a real limitation and it is written down rather than worked
    /// around: if the disclosure rules grow a third condition, this comment is
    /// the signal to ask the engine for a constructor rather than to bolt a
    /// fourth boolean onto the test helper.
    fn said(distorted: bool, dynamic: bool, widgets: usize, annots: usize) -> Vec<String> {
        let mut out = Vec::new();
        if distorted {
            out.push(t::placed_distorted(1.4, 1.0));
        }
        if dynamic {
            out.push(t::placed_dynamic().to_owned());
        } else if widgets > 0 {
            out.push(t::placed_widgets_ignored(widgets));
        }
        if annots > 0 {
            out.push(t::placed_annotations_ignored(annots));
        }
        out
    }

    #[test]
    fn a_clean_placement_says_nothing() {
        assert!(said(false, false, 0, 0).is_empty());
    }

    #[test]
    fn a_dynamic_stamp_is_not_also_told_about_its_own_widgets() {
        let out = said(false, true, 3, 0);
        assert_eq!(out.len(), 1, "one sentence, not two: {out:?}");
        assert_eq!(out[0], t::placed_dynamic());
    }

    #[test]
    fn a_static_stamp_with_widgets_is_told_about_them() {
        let out = said(false, false, 3, 0);
        assert_eq!(out.len(), 1);
        assert!(out[0].contains('3'), "the count is the disclosure: {out:?}");
    }

    #[test]
    fn a_stretched_signature_leads_the_report() {
        let out = said(true, true, 0, 2);
        assert_eq!(out.len(), 3);
        assert!(
            out[0].contains("stretched"),
            "shape first, it is the one that is wrong on the page: {out:?}"
        );
    }

    /// The stretch sentence names a direction, and gets it the right way round.
    ///
    /// ★ Both signs, deliberately. A helper that only ever divides one way
    /// passes on a symmetric bug — this project has already recorded that *a
    /// suite which only tries one SIGN is not testing the value*.
    #[test]
    fn the_stretch_direction_follows_the_ratio() {
        assert!(t::placed_distorted(1.5, 1.0).contains("wider"));
        assert!(t::placed_distorted(1.0, 1.5).contains("taller"));
        // A 50 % excess in either direction reads as the same magnitude.
        assert!(t::placed_distorted(1.5, 1.0).contains("50%"));
        assert!(t::placed_distorted(1.0, 1.5).contains("50%"));
    }

    /// A degenerate factor prints a number, not `inf`.
    #[test]
    fn a_zero_factor_does_not_print_infinity() {
        let s = t::placed_distorted(0.0, 1.0);
        assert!(!s.contains("inf"), "{s}");
        assert!(!s.contains("NaN"), "{s}");
    }
}
