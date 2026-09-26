//! # `app::actions::customstamp` — placing one of the operator's OWN stamps
//!
//! `OPERATOR_REQUESTS.md` **O172**: *"add our own custom stamps and use them,
//! preferrably exactly the same way acrobat does."* This module is the second
//! half of that sentence. The first half — reading his stamps folder and
//! finding what is in it — is [`crate::stamps::library`]; the half that writes
//! a NEW collection for Acrobat to load is [`super::stamps`].
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/customstamp.md`.

use pdfcer_core::edit::EditError;
use pdfcer_core::page_tree::Rect;

use super::apply::vector_edit;
use crate::app::state::OpenDoc;
use crate::stamps::library::CustomStamp;
use crate::text::stamps as t;

/// **Place `stamp`'s artwork on `page`, inside `rect`.**
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
            // ⊗, not ⚑. Nothing was edited, and a refusal reported under
            // `⚑ About your last edit:` claims one was — see
            // `t::CustomStampUnavailable`'s header.
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
    // Unconditional here, where `textannot` excludes the sticky note. There
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
    // `vector_edit`, not `vector_edit_on_page`, and the choice is not
    // cosmetic. The narrow variant asserts that EVERY call of the verb changes
    // nothing a rasteriser would draw on any other sheet. This one imports an
    // object graph into the document's own object table — a resource closure
    // that may include fonts and images — and while today's engine attaches it
    // only to this page's new annotation, the assertion `vector_edit_on_page`
    // demands is about the verb rather than about this operand. It is not one
    // this shell can make on the engine's behalf.
    vector_edit(doc, "place-custom-stamp", page, 1, move |session| {
        // The one refusal on this route that has a REMEDY, and therefore
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
        // Recorded and then re-raised, deliberately. Swallowing it would
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

        // The measurement line. Its first token is `custom-stamp-placed`,
        // deliberately NOT `place-custom-stamp` — the funnel writes a line
        // under the bare label and `Trace::last(name)` matches on the first
        // token, so sharing the name would hand a driven check the funnel's
        // `page= n= epoch= disclosures=` when it asked for `scale-x=`.
        // `tools/gates/check-trace-names.py` enforces the suffix convention.
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
fn disclosures(placed: &pdfcer_core::edit::PlacedArtwork, dynamic: bool) -> Vec<String> {
    let mut said = Vec::new();
    if placed.distorted {
        said.push(t::placed_distorted(placed.scale_x, placed.scale_y));
    }
    if dynamic {
        said.push(t::placed_dynamic().to_owned());
    } else if placed.source_widgets_ignored > 0 {
        // The `else` is the whole rule. A dynamic stamp's widgets ARE its
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
    //! These do NOT place anything. The placement itself is the engine's
    //! verb and is tested there; what is this shell's own is the decision
    //! about which facts become sentences, and that decision is a pure
    //! function of a `PlacedArtwork` and one boolean. A test that opened a
    //! document to reach it would be measuring the engine.
    //!
    //! The rest of the route is measured by DRIVING it, in
    //! `tools/ui-verify/src/checks/custom_stamp.rs`
    //! (`custom_stamp_reaches_the_page`): it arms Markup ▸ Stamp, presses one
    //! of the operator's own stamps in the gallery, drags a rectangle of the
    //! wrong shape, and asserts on `custom-stamp-placed distorted=true` plus
    //! the disclosure region on the status bar. That check plants a stamp
    //! collection into a scratch `%APPDATA%`, so it is neither vacuous on a
    //! machine with no stamps nor dependent on his own folder.
    //!
    //! The two are not interchangeable. Everything above `place()` — the
    //! gallery, the selection, the fork in `apply` — is invisible to the tests
    //! in this module, which is why the driven check exists; and the driven
    //! check cannot enumerate the disclosure rules, which is why these do.
    //! R1 — a passing unit test is not a report of working software.

    use super::*;

    /// Build a `PlacedArtwork` with the fields these rules read.
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
