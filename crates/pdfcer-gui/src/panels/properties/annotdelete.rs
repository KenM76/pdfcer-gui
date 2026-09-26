//! # `panels::properties::annotdelete` — whether the selected annotation can
//! be deleted, and what would go with it
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/properties/annotdelete.md`.

use egui::Ui;
use pdfcer_core::object::ObjId;

use crate::app::state::OpenDoc;
use crate::text::markup as t;
use crate::text::markup::AnnotDeleteRefusal;

/// The section's rect, for `ui-verify`.
const REGION: &str = "properties.annot_delete"; // ui-text-exempt: trace region name, never displayed
/// The **refusal** sentence's own rect, published only when a gate refuses.
const REGION_REFUSED: &str = "properties.annot_delete.refused"; // ui-text-exempt: trace region name, never displayed
/// The **collateral** sentence's rect, published only when there is collateral.
const REGION_COLLATERAL: &str = "properties.annot_delete.collateral"; // ui-text-exempt: trace region name, never displayed
/// The per-frame census of what the gate answered and what the preview found.
const TRACE: &str = "annot-delete-gates"; // ui-text-exempt: diagnostic trace name, never displayed

/// **Why the selected annotation cannot be deleted**, or `None` if it can.
#[must_use]
pub fn gate(
    doc: &OpenDoc,
    target: &crate::canvas::selection::annot::AnnotTarget,
) -> Option<Refusal> {
    if target.locked {
        return Some(Refusal::Locked);
    }
    doc.session
        .annotation_deletion_refusal()
        .as_ref()
        .map(refusal_for)
        .map(Refusal::Document)
}

/// **Would a delete of whatever is selected right now be refused?**
///
/// The `&OpenDoc` convenience over [`gate`], for the two callers that have a
/// document and no annotation target in hand: `crate::app::conditions`, which
/// publishes `selection.delete_permitted`, and `crate::canvas::interact`, which
/// fills in `canvas::keys::Keys::annot_delete_refused`.
///
/// **`false` when nothing is selected, and when what is selected is not an
/// annotation.** This answers *would the engine refuse?* and not *is there
/// anything to delete?* — the second question is `selection.actionable`'s, and
/// conflating them here would make a content selection or an empty one look
/// like a refusal. The safe direction is stated at length on
/// `crate::app::conditions`' publication site: a control drawn where it refuses
/// is the defect being fixed, and a control withheld where it would have worked
/// is a worse one, because the operator has no gesture left that reports it.
///
/// It exists so that [`gate`]'s three-check ladder has exactly one spelling.
/// `crate::canvas::interact` calls [`refuses`] in **one line** by deliberate
/// necessity — that file sits ON R2's 1,500-line ceiling — and the argument that
/// would otherwise have been a comment there is here instead, which is where a
/// rule with four readers belongs anyway.
///
/// # WHY THIS IS NOT THE FUNCTION `canvas::interact` MAY CALL
///
/// It reads `doc.selection`, and **inside a canvas frame `doc.selection` is
/// empty**. `canvas::interact` opens with
///
/// ```ignore
/// let mut selection = std::mem::take(&mut doc.selection);
/// ```
///
///
/// What that looked like from a chair is the R83 defect the gate was written to
/// close, intact: on a certified drawing the Properties panel drew *"this
/// document carries a certification signature…"* beside the selected comment,
/// the operator pressed Delete, `AnnotAction::Delete` was raised anyway,
/// `EditSession::delete_annotation` refused it, `actions::apply::vector_edit`'s
/// `Err` arm wrote `delete-annotation-refused` to the trace **and said nothing
/// to the operator**, and `actions::annots::delete` cleared the selection
/// regardless — taking the panel sentence that explained the refusal away with
/// it. Three visible controls' worth of gate, and the keyboard walked straight
/// past it.
///
/// It was invisible to every unit test in the crate, because a unit test builds
/// an `OpenDoc` and asks the question with the selection **on** it — which is
/// the state this function documents and the state the caller was not in. Only
/// driving the running binary could see it, and `ui-verify`'s `annot_delete_gate`
/// phase D did, on its first real run.
///
/// ⇒ The remedy is structural rather than a comment: [`refuses`] takes the
/// selection it is to ask about **by argument**, so a caller holding a detached
/// one cannot silently ask about the wrong one. This wrapper stays for the
/// callers that genuinely hold an intact document — `crate::app::conditions`
/// runs in the panel pass, outside `interact`'s take — and is one line so that
/// the ladder still has exactly one spelling.
#[must_use]
pub fn refuses_selected(doc: &OpenDoc) -> bool {
    refuses(doc, &doc.selection)
}

/// [`refuses_selected`], asking about **the selection handed in** rather than
/// the one on the document.
#[must_use]
pub fn refuses(doc: &OpenDoc, selection: &crate::canvas::selection::SelectionState) -> bool {
    selection
        .annot()
        .is_some_and(|selected| gate(doc, &selected.target).is_some())
}

/// What [`gate`] found, when it found something.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    /// §12.5.3 Table 165 bit 8 is set on this annotation.
    Locked,
    /// The document itself refuses, for the reason carried.
    Document(AnnotDeleteRefusal),
}

impl Refusal {
    /// The sentence.
    ///
    #[must_use]
    pub(crate) const fn line(self) -> &'static str {
        match self {
            Self::Locked => t::annot_delete_locked(),
            Self::Document(why) => why.line(),
        }
    }
}

/// Which sentence an `EditError` from `annotation_deletion_refusal` earns.
fn refusal_for(error: &pdfcer_core::edit::EditError) -> AnnotDeleteRefusal {
    use pdfcer_core::edit::EditError;
    match error {
        EditError::DocumentEncrypted => AnnotDeleteRefusal::Encrypted,
        EditError::CertificationForbidsChange { .. } => AnnotDeleteRefusal::Certified,
        _ => AnnotDeleteRefusal::Other,
    }
}

/// **The memoised answer to *what would go with this delete?***
#[derive(Debug, Default)]
pub struct DeletionPreview {
    /// What [`Self::line`] describes, or `None` before anything has been asked.
    stamp: Option<(ObjId, u64)>,
    /// The collateral sentence for that stamp, if there was any collateral.
    line: Option<String>,
}

impl DeletionPreview {
    /// The collateral sentence for `id` at this document's current epoch,
    /// asking the engine only when the stamp has moved.
    fn line(&mut self, doc: &OpenDoc, id: ObjId) -> Option<&str> {
        let stamp = (id, doc.edit_epoch);
        if self.stamp != Some(stamp) {
            self.stamp = Some(stamp);
            self.line = doc
                .session
                .annotation_deletion_preview(id)
                .ok()
                .and_then(|preview| {
                    t::deletion_would_take(
                        preview.popup_removed,
                        preview.parent_popup_cleared,
                        preview.replies_orphaned,
                        preview.group_members_promoted,
                    )
                });
        }
        self.line.as_deref()
    }
}

/// **Draw what is true about deleting the selected annotation, or nothing.**
pub fn section(ui: &mut Ui, doc: &OpenDoc, memo: &mut DeletionPreview) -> bool {
    let Some(selection) = doc.selection.annot() else {
        return false;
    };
    let target = &selection.target;

    // R83 — ASKED HERE, BEFORE ANYTHING IS DRAWN, THROUGH THE SAME FUNCTION
    // `crate::app::conditions` ASKS. See [`gate`] on why there is one
    // derivation and not two.
    //
    // A **pure query**: `annotation_deletion_refusal` reads the signature census
    // and the trailer and mutates nothing, so it is safe every frame, and the
    // engine says so in as many words.
    let refusal = gate(doc, target);
    let collateral = if refusal.is_some() {
        // Not asked when a gate refuses, and this is a correctness point rather
        // than an optimisation: `annotation_deletion_preview` raises the SAME
        // refusals, so on a refused document it would return `Err` and the memo
        // would cache `None` under this epoch. Harmless today; a trap the first
        // time somebody reads a cached `None` as "no collateral" rather than as
        // "not asked".
        None
    } else {
        memo.line(doc, target.id).map(str::to_owned)
    };

    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI.
        //
        // Written EVERY frame this section runs, refused or not and collateral
        // or not — which is what makes the two regions above readable as
        // evidence rather than as noise. See `REGION_REFUSED`.
        format!(
            "{TRACE} id={} locked={} refused={} collateral={}",
            target.id.num,
            u8::from(target.locked),
            u8::from(refusal.is_some()),
            u8::from(collateral.is_some()),
        )
    });

    let Some(text) = refusal.map(Refusal::line).map(str::to_owned).or(collateral) else {
        return false;
    };
    let refused = refusal.is_some();

    // No `.strong()` — R84 / DEFECTS.md D11: no theme this project ships renders
    // it legibly on a panel. And no warning tint: every one of these sentences
    // is a fact about the **document**, and warning styling would make a
    // property of the operator's file read as a pdfcer failure — which is the
    // ruling `super`'s own header makes about its disclosure heading.
    ui.label(text);
    crate::diag::ui_rect(
        if refused {
            REGION_REFUSED
        } else {
            REGION_COLLATERAL
        },
        ui.min_rect(),
    );
    // `ui.min_rect()`, AFTER drawing. `max_rect` is the space a `Ui` is
    // *allowed* to use, not the space it took; published before anything is
    // drawn it names a different panel entirely, and `ui-verify` scrolls **at** a
    // region — so a wheel event aimed at that centre lands somewhere else and a
    // check hunting for controls below the fold reports them missing. The full
    // account is on `crate::panels::properties::formfield::section`, which is
    // where that was measured.
    crate::diag::ui_rect(REGION, ui.min_rect());
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfcer_core::edit::EditError;

    /// **Every refusal the query documents earns its own sentence**, and
    /// none of them falls through to the catch-all.
    #[test]
    fn each_documented_refusal_earns_its_own_sentence() {
        for (error, expected) in [
            (EditError::DocumentEncrypted, AnnotDeleteRefusal::Encrypted),
            (
                EditError::CertificationForbidsChange { permission: 2 },
                AnnotDeleteRefusal::Certified,
            ),
            (
                EditError::CertificationForbidsChange { permission: 1 },
                AnnotDeleteRefusal::Certified,
            ),
        ] {
            assert_eq!(
                refusal_for(&error),
                expected,
                "`{error}` must not fall through to the catch-all"
            );
        }
        assert_eq!(
            refusal_for(&EditError::ObjectNumbersExhausted),
            AnnotDeleteRefusal::Other,
            "and something the query does not document must land in `Other` \
             rather than borrowing a sentence about a signature"
        );
    }

    /// **The three sentences are three different sentences.**
    #[test]
    fn the_refusals_are_told_apart_by_their_words() {
        let lines = [
            AnnotDeleteRefusal::Encrypted.line(),
            AnnotDeleteRefusal::Certified.line(),
            AnnotDeleteRefusal::Other.line(),
        ];
        for (i, a) in lines.iter().enumerate() {
            for b in lines.iter().skip(i + 1) {
                assert_ne!(a, b);
            }
        }
        assert!(AnnotDeleteRefusal::Encrypted.line().contains("encrypted"));
        assert!(AnnotDeleteRefusal::Certified.line().contains("signature"));
    }

    /// **`Locked` beats the document's own refusal when both are true.**
    #[test]
    fn the_locked_sentence_is_the_one_that_leaves_a_next_step() {
        assert_ne!(
            Refusal::Locked.line(),
            Refusal::Document(AnnotDeleteRefusal::Certified).line()
        );
        assert_eq!(Refusal::Locked.line(), t::annot_delete_locked());
    }
}

#[cfg(test)]
mod fixtures {
    use super::*;
    use crate::app::state::open_local_fixture;

    /// **Two pages, one enforced certification (`/Perms /DocMDP`, `/P 2`), one
    /// `/Square` markup with a pop-up and a reply.** Built by
    /// `tools/gen-certified-fixture.py`, whose header carries why no existing
    /// fixture could drive this — `signed-two-pages.pdf` is deliberately an
    /// *approval* signature, so the gate is open on it.
    const CERTIFIED: &str = "certified-comments.pdf";
    /// The **same document with the certification removed**, and nothing else
    /// changed. See the generator's header on why the pair is one document
    /// rather than two: any difference the tests below see is caused by
    /// `/Perms`, because nothing else differs.
    const ORDINARY: &str = "threaded-comments.pdf";
    /// The `/Square` under test — object 20 in both fixtures, by construction.
    const SQUARE: u32 = 20;

    /// A target naming the fixture's square, unlocked.
    fn square_target() -> crate::canvas::selection::annot::AnnotTarget {
        crate::canvas::selection::annot::AnnotTarget {
            page: 0,
            id: ObjId::new(SQUARE, 0),
            kind: crate::canvas::selection::annot::AnnotKind::Markup,
            subtype: "Square".to_owned(),
            locked: false,
        }
    }

    /// **An enforced certification withholds the Delete control, and the
    /// sentence names the signature.**
    #[test]
    fn a_certified_document_withholds_the_delete_control() {
        let doc = open_local_fixture(CERTIFIED);
        let refusal = gate(&doc, &square_target()).expect(
            "an enforced /Perms /DocMDP at /P 2 must refuse an annotation delete — if this \
             is None the fixture has lost its certification, not the gate its nerve",
        );
        assert_eq!(refusal, Refusal::Document(AnnotDeleteRefusal::Certified));
        assert!(
            refusal.line().contains("signature"),
            "the sentence must name what is actually stopping the delete: an operator \
             sent looking for an encryption setting on a certified file finds nothing"
        );
        assert!(
            refuses_selected(&doc) == doc.selection.annot().is_some(),
            "with nothing selected the convenience form answers `false` — it asks \
             *would the engine refuse?*, not *is the document certified?*"
        );
    }

    /// **The gate answers about the selection it is HANDED, not the one on
    /// the document** — the regression test for the defect of 2026-08-29.
    #[test]
    fn the_gate_reads_the_selection_it_is_given() {
        let doc = open_local_fixture(CERTIFIED);
        assert!(
            doc.selection.annot().is_none(),
            "the premise: a freshly-opened document has selected nothing, which is also \
             what `canvas::interact` leaves behind on the document for a whole frame"
        );

        let mut detached = crate::canvas::selection::SelectionState::default();
        detached.select_annot(crate::canvas::selection::AnnotSelection {
            target: square_target(),
            outline: egui::Rect::from_min_max(egui::pos2(120.0, 142.0), egui::pos2(320.0, 282.0)),
            oriented: None,
        });

        assert!(
            refuses(&doc, &detached),
            "the certification refuses this delete, and the selection naming the annotation \
             is the DETACHED one — a gate that reads `doc.selection` instead answers `false` \
             here, which is the exact state the Delete key shipped in"
        );
        assert!(
            !refuses(&doc, &crate::canvas::selection::SelectionState::default()),
            "and the other direction, so the assertion above is not satisfied by a gate that \
             refuses unconditionally: with nothing selected there is no delete to refuse"
        );
        assert!(
            !refuses_selected(&doc),
            "the convenience form still asks about the DOCUMENT's selection, which is empty \
             — the two forms are the same ladder asked about two different selections, and \
             that difference is the whole point of the pair"
        );
    }

    /// **The same document without the certification offers the control**,
    /// which is what makes the test above evidence rather than a tautology.
    #[test]
    fn the_same_document_without_the_certification_offers_it() {
        let doc = open_local_fixture(ORDINARY);
        assert_eq!(
            gate(&doc, &square_target()),
            None,
            "an approval signature is not an enforced certification: \
             `forbids_structural_change` is `perms_enforced && signatures > 0`, and this \
             file has the signature without the /Perms entry"
        );
    }

    /// **The collateral is stated before the click, with both clauses.**
    #[test]
    fn the_collateral_names_the_popup_and_the_orphaned_reply() {
        let doc = open_local_fixture(ORDINARY);
        let mut memo = DeletionPreview::default();
        let line = memo
            .line(&doc, ObjId::new(SQUARE, 0))
            .expect("a square with a pop-up and a reply has collateral to disclose")
            .to_owned();
        assert!(line.contains("pop-up"), "{line}");
        assert!(line.contains("1 reply will be left"), "{line}");
        assert!(
            !line.contains("grouped"),
            "the fixture carries no /RT /Group subordinate, so a promotion clause \
             would mean the counts are being read from the wrong field: {line}"
        );
    }

    /// **The memo answers from the stamp on the second call.**
    #[test]
    fn the_second_frame_costs_no_engine_call() {
        let doc = open_local_fixture(ORDINARY);
        let mut memo = DeletionPreview::default();
        let id = ObjId::new(SQUARE, 0);
        assert!(memo.line(&doc, id).is_some());
        // ui-text-exempt: a test poison, never rendered.
        memo.line = Some("poisoned".to_owned());
        assert_eq!(
            memo.line(&doc, id),
            Some("poisoned"),
            "the stamp has not moved, so nothing may re-ask the engine"
        );
    }

    /// **A moved epoch re-asks**, which is the other half of the stamp.
    #[test]
    fn a_moved_epoch_re_asks_the_engine() {
        let mut doc = open_local_fixture(ORDINARY);
        let mut memo = DeletionPreview::default();
        let id = ObjId::new(SQUARE, 0);
        assert!(memo.line(&doc, id).is_some());
        // ui-text-exempt: a test poison, never rendered.
        memo.line = Some("poisoned".to_owned());
        doc.edit_epoch = doc.edit_epoch.wrapping_add(1);
        assert_ne!(
            memo.line(&doc, id),
            Some("poisoned"),
            "an edit happened, so the collateral is a fact about a document \
             revision that is no longer on screen"
        );
    }
}
