//! # `panels::properties::annotdelete` — whether the selected annotation can
//! be deleted, and what would go with it
//!
//! Two engine queries, one section, and both of them were named nowhere in this
//! shell until 2026-08-29:
//!
//! | query | what it answers | what this section does with it |
//! |---|---|---|
//! | `EditSession::annotation_deletion_refusal` | *would `delete_annotation` refuse right now?* | withholds every Delete control and puts a sentence in their place |
//! | `EditSession::annotation_deletion_preview` | *what else would go with it?* | states the collateral **before** the press |
//!
//! ## ★★★ The defect the first query closes, and it is the day-before defect
//! wearing a different `/Subtype`
//!
//! On 2026-08-28 an audit of `EditSession`'s public surface found that
//! **`deletion_refusal` — the FORMS one — was consulted by nothing.** It
//! appeared in this crate only inside three comments in `crate::panels::forms`,
//! arguing correctly about which query *Flatten* should ask, while Rename,
//! Delete field and Delete this box asked none. On an ordinary certified
//! fillable form all three were drawn live and every press returned a refusal to
//! the trace and nothing at all to the operator.
//! `crate::panels::properties::formfield` is the fix and its header carries the
//! finding.
//!
//! **The annotation half was the same defect, one file along, and it was still
//! open.** `annotation_deletion_refusal` is `&self`, side-effect-free, and its
//! own doc comment names this call site by rule number — *"safe to call every
//! frame from a UI (R83: ask before offering the control)"*. Nothing called it.
//! So on a certified or encrypted drawing:
//!
//! * the **Format tab's Delete** was drawn and enabled,
//! * the **canvas right-click's Delete** was drawn and enabled,
//! * the **Delete key** raised the action,
//!
//! and all three ended in `crate::app::actions::apply::vector_edit`'s `Err`
//! arm — which wrote one line to the trace and, by that arm's own recorded
//! decision, said **nothing to the operator**. Three visible controls, silently
//! inert. That is the failure this project is named after. (Since O116,
//! 2026-09-04, that arm words an un-categorised decline. It ends the silence
//! and names no annotation, no kind and no cause, which is why this file's
//! sentences are still owed.)
//!
//! ⇒ ★★ The generalisation, and it is the audit's rather than this file's: **a
//! query the engine wrote for a shell is not consumed by being read.** Both of
//! these carry doctests spelling out the call site, and both sat unused for the
//! whole life of the crate. The instrument that found them was
//! `tools/verb-coverage.py` — asking what the engine offers — not a re-reading
//! of this shell.
//!
//! ## ★★★ Where a gate refuses, the control is NOT DRAWN and a sentence takes
//! its place (R9)
//!
//! The established shape, set by the forms fix and followed here rather than
//! re-invented:
//!
//! - Greying is for a capability that is **temporarily** unavailable, and is
//!   always explained on hover. A certification signature is not temporary and
//!   cannot be argued out of; nor is `/Encrypt`; nor is §12.5.3 bit 8.
//! - A permanently-refused capability renders **nothing**, or a sentence saying
//!   where the thing actually lives. There is no elsewhere here, so it is the
//!   sentence.
//! - **A sentence rather than a silence.** A panel that quietly omits half its
//!   controls looks half-drawn, and an operator who finds Delete missing with no
//!   explanation has found a broken program rather than a protected document.
//!
//! The withholding of the ribbon and menu controls is not done here — it cannot
//! be, because a manifest item is drawn by `egui-shell` and this crate may not
//! reach into it. It is done by the condition
//! `crate::app::conditions`' `selection.delete_permitted`, which `format.delete`
//! carries as its `visible_when` on the Format tab and on both canvas menus.
//! **One question, two consumers**: the condition and this sentence are derived
//! from the same three facts in the same order, and [`gate`] is the one function
//! that derives them.
//!
//! ## ★★ Why the SENTENCE lives in a panel and not in the status bar's decline
//!
//! `crate::app::status::decline` is this shell's worded-decline surface and it
//! would have been the reflexive choice. It is the wrong one here, and the
//! module's own header says why in the general case:
//!
//! > A decline must be **repeatable** … and a decline changes no document, so
//! > the epoch never moves.
//!
//! A decline is a report that *a gesture just failed*. What this section states
//! is not a gesture's outcome at all — it is a **standing property of the open
//! document**, true from the moment it was opened until it is closed, and true
//! whether or not the operator has pressed anything. A sentence that arrives
//! only after a press, and retires when the next command runs, would deliver
//! that fact at the one moment R83 exists to get ahead of.
//!
//! ⇒ So it sits in the panel that describes what is selected, permanently, and
//! the press it prevents is a press the operator never makes.
//!
//! ## ⚠ The cost of the preview, and what was chosen
//!
//! `annotation_deletion_preview` is `&self` and mutates nothing, but it is **not
//! free**: it locates the annotation, walks the page's whole `/Annots` array to
//! find `/IRT` referrers, and validates the `/Popup`. That is O(annotations) per
//! call. The old shell computed exactly this and gated it on **hover**, one row
//! at a time, because its Comments panel would otherwise have paid
//! O(rows × walk) every frame.
//!
//! This section does better than hover, and the reason it can is that **it is
//! not a list**. There is one selected annotation, so the worst case is one call
//! per frame rather than one per row — and even that is not paid, because the
//! answer is memoised in [`DeletionPreview`] on `(annotation id, edit epoch)`.
//! Both halves of that key are load-bearing:
//!
//! * the **id** changes when the operator selects something else, which is the
//!   only other thing that can change the answer;
//! * the **epoch** changes on every accepted edit, which is what makes a reply
//!   added or removed since the last frame visible here.
//!
//! ⇒ In steady state — an annotation selected, nothing being edited — the cost
//! is one `Option` comparison per frame and no engine call at all. A hover gate
//! would have been cheaper only in the frames where the answer is not wanted,
//! and it would have hidden the fact behind a gesture the operator has no reason
//! to make.
//!
//! ## What this section does NOT do
//!
//! **It raises no action and offers no control.** It reads and it draws
//! sentences; `super::body`'s contract is `&OpenDoc` shared, so that is a
//! compile-time fact rather than a convention. The Delete controls stay where
//! they are — the Format tab, the canvas menus, the Delete key — and this
//! section only decides what is *said* about them.
//!
//! **It does not run the delegated route's gate.** `annotation_deletion_preview`
//! reports a `/Redact` mark or a ce dimension with its
//! `AnnotationDeletionRoute` and **zeroed counts**, and the engine states
//! plainly that it does not run the destination verb's certification gate for
//! those. Because this section says nothing at all when every count is zero, no
//! false claim is made on that path: a delegated target produces silence here,
//! not a promise that the delete would work. The engine's own note says the
//! honest fix is for those verbs to grow refusal queries of their own, and
//! *"inventing a half-answer here would be worse than a stated gap."*

use egui::Ui;
use pdfcer_core::object::ObjId;

use crate::app::state::OpenDoc;
use crate::text::markup as t;
use crate::text::markup::AnnotDeleteRefusal;

/// The section's rect, for `ui-verify`.
///
/// ★ A published region name is a cross-repo stability contract: the harness
/// asserts on it by string, so renaming one turns a check into a skip rather
/// than a failure.
const REGION: &str = "properties.annot_delete"; // ui-text-exempt: trace region name, never displayed
/// The **refusal** sentence's own rect, published only when a gate refuses.
///
/// ★★★ "Only when refused" is the whole value of it. A driven check asserting
/// that a certified document withholds Delete reads two things: this region
/// **present**, and `properties.format.delete` — the ribbon control's region —
/// **absent**. An absence is admissible evidence only because [`TRACE`] is
/// written on every frame this section runs either way, so a check can tell
/// *"the control was withheld"* from *"the panel never drew"*. The harness's
/// own rule 4 states the same obligation from the other side.
const REGION_REFUSED: &str = "properties.annot_delete.refused"; // ui-text-exempt: trace region name, never displayed
/// The **collateral** sentence's rect, published only when there is collateral.
const REGION_COLLATERAL: &str = "properties.annot_delete.collateral"; // ui-text-exempt: trace region name, never displayed
/// The per-frame census of what the gate answered and what the preview found.
///
/// ★ The name carries a verb suffix — `annot-delete-…` rather than
/// `delete-annotation-…` — because `tools/gates/check-trace-names.py` forbids a
/// module's own summary line from sharing its first token with a `vector_edit`
/// funnel label, and `delete-annotation` is exactly such a label. A harness
/// asking `last("delete-annotation")` would otherwise get the funnel's line,
/// which carries `page`, `n`, `epoch` and `disclosures` and none of the keys
/// this line exists to publish. That failure has happened three times on this
/// project and it presents as a confident false negative.
const TRACE: &str = "annot-delete-gates"; // ui-text-exempt: diagnostic trace name, never displayed

/// **Why the selected annotation cannot be deleted**, or `None` if it can.
///
/// # ★★★ The one derivation, because there are two consumers
///
/// This function is asked by [`section`], which draws the sentence, and by
/// `crate::app::conditions`, which publishes `selection.delete_permitted` and
/// thereby decides whether `format.delete` is drawn at all. **They must not ask
/// different questions.** A control withheld by one rule while a panel explains
/// a different one is the shape the forms audit found in miniature — three
/// comments arguing about which query to ask, and no call.
///
/// # The order of the three checks, and why it is this order
///
/// 1. **`locked`** first — §12.5.3 Table 165 bit 8, *the file says the user
///    interface may not change this annotation's properties*. It is a fact about
///    **this annotation** rather than about the document, so it is the most
///    specific answer available and the one that tells the operator the most.
///    It is read off `AnnotTarget::locked`, which
///    `crate::canvas::selection::annot` carries on the target for precisely this
///    reason: *"so a surface can omit the controls it governs rather than offer
///    them and let the engine refuse."*
/// 2. **`annotation_deletion_refusal`** second — the document-wide gates,
///    `/Encrypt` then the certification permission, in the engine's own order.
/// 3. Nothing else. There is no third source, and a check invented here would be
///    a second implementation of a rule the engine owns.
///
/// ⇒ ★★ Locked is checked **first even though the engine would refuse a
/// certified document anyway**, because the two sentences are not
/// interchangeable: an operator told *"this comment is marked as not to be
/// changed"* can go and look at that comment, and an operator told *"the
/// document is certified"* cannot do anything about one annotation. The more
/// actionable fact wins when both are true.
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
/// ★★ **`false` when nothing is selected, and when what is selected is not an
/// annotation.** This answers *would the engine refuse?* and not *is there
/// anything to delete?* — the second question is `selection.actionable`'s, and
/// conflating them here would make a content selection or an empty one look
/// like a refusal. The safe direction is stated at length on
/// `crate::app::conditions`' publication site: a control drawn where it refuses
/// is the defect being fixed, and a control withheld where it would have worked
/// is a worse one, because the operator has no gesture left that reports it.
///
/// ★ It exists so that [`gate`]'s three-check ladder has exactly one spelling.
/// `crate::canvas::interact` calls [`refuses`] in **one line** by deliberate
/// necessity — that file sits ON R2's 1,500-line ceiling — and the argument that
/// would otherwise have been a comment there is here instead, which is where a
/// rule with four readers belongs anyway.
///
/// # ★★★ WHY THIS IS NOT THE FUNCTION `canvas::interact` MAY CALL
///
/// It reads `doc.selection`, and **inside a canvas frame `doc.selection` is
/// empty**. `canvas::interact` opens with
///
/// ```ignore
/// let mut selection = std::mem::take(&mut doc.selection);
/// ```
///
/// and puts it back a thousand lines later, so every line between those two
/// sees a `SelectionState::default()` on the document. From 2026-08-28 to
/// 2026-08-29 this function was what filled in
/// `canvas::keys::Keys::annot_delete_refused`, which meant that flag was
/// **`false` on every frame of the program's life, on every document**:
/// `doc.selection.annot()` answered `None`, `is_some_and` short-circuited, and
/// the Delete key's annotation rung never declined.
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
///
/// The form `crate::canvas::interact` must use, and the reason is on
/// [`refuses_selected`] at length: inside a canvas frame the document's own
/// selection has been moved out into a local, so a query that reaches for
/// `doc.selection` is asking about an empty one and answers `false` — *"the
/// delete is permitted"* — for every document there is.
///
/// `doc` is still needed and is still the whole document: `gate`'s second and
/// third checks are `EditSession::annotation_deletion_refusal`, which is a fact
/// about the **file** (its `/Encrypt`, its `/Perms /DocMDP`) and not about
/// anything selected. Only the first check — §12.5.3 Table 165 bit 8 — is
/// per-annotation, and it is read off the target the selection carries.
#[must_use]
pub fn refuses(doc: &OpenDoc, selection: &crate::canvas::selection::SelectionState) -> bool {
    selection
        .annot()
        .is_some_and(|selected| gate(doc, &selected.target).is_some())
}

/// What [`gate`] found, when it found something.
///
/// Two variants rather than one enum with a `Locked` arm folded into
/// [`AnnotDeleteRefusal`], because the two come from **different sources and
/// have different scopes**: one is a bit on the selected annotation's flags word
/// and the other is the engine's verdict on the whole document. Folding them
/// would put a per-annotation fact into a type whose every other member is
/// derived from an `EditError`, and the mapping function below would have to
/// answer for a variant no `EditError` produces.
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
    /// ★ `pub(crate)` since 2026-08-29, when `Ctrl+X` turned out to be a fourth
    /// door onto this verb and needed the same words on the status row that this
    /// panel puts beside the selection. **One catalog, two surfaces** — the
    /// alternative is a second wording of one fact, and this project's record of
    /// that is `UnembedBlocker::reason` delegating to `Removability::reason`
    /// precisely so a font that refuses in two places refuses for the same
    /// stated reason.
    #[must_use]
    pub(crate) const fn line(self) -> &'static str {
        match self {
            Self::Locked => t::annot_delete_locked(),
            Self::Document(why) => why.line(),
        }
    }
}

/// Which sentence an `EditError` from `annotation_deletion_refusal` earns.
///
/// # ★★ A total match with a named catch-all, not a `_ =>` with a guess
///
/// Every variant the query's own documentation names has an arm, **in the order
/// the engine checks them**, so this function and
/// `EditSession::annotation_deletion_refusal`'s two-line body can be read side
/// by side. That is the shape `crate::app::actions::xobject::refusal_for` uses
/// for `unshare_form`, and the failure it prevents is the one that is invisible
/// in a diff: somebody adds an arm, mistypes a variant name, the compiler is
/// happy because `_ =>` catches it, and the operator meets *"pdfcer cannot delete
/// comments from this document"* where they should have met the sentence about
/// the signature.
///
/// ★ A free function rather than a `From` impl, for the same reason that one is:
/// a `From` invites the mapping to be reused for another verb's errors and it is
/// not reusable. `DocumentEncrypted` earns *this* sentence because the subject
/// is an annotation; the same variant out of `unshare_form` earns a sentence
/// about a drawing.
fn refusal_for(error: &pdfcer_core::edit::EditError) -> AnnotDeleteRefusal {
    use pdfcer_core::edit::EditError;
    match error {
        EditError::DocumentEncrypted => AnnotDeleteRefusal::Encrypted,
        EditError::CertificationForbidsChange { .. } => AnnotDeleteRefusal::Certified,
        _ => AnnotDeleteRefusal::Other,
    }
}

/// **The memoised answer to *what would go with this delete?***
///
/// See the module header's cost section for why this is memoised rather than
/// hover-gated. The stamp is `(annotation id, edit epoch)`; the payload is the
/// finished sentence, or `None` for the ordinary case where there is nothing to
/// say.
///
/// ★ The **sentence** is cached rather than the engine's `AnnotationDeletion`
/// record, deliberately. Caching the record would leave the wording to be
/// re-derived every frame, which is cheap but pointless, and it would put a
/// `pdfcer_core` type in `crate::panels::PanelsState` — a struct whose whole
/// purpose is *the operator's own state*. What is stored here is what is drawn.
///
/// ★★ A **failed** preview caches as `None` too, and that is not a bug being
/// papered over. `annotation_deletion_preview` returns `Err` for exactly the
/// refusals [`gate`] has already asked about, plus a target that has gone; in
/// every one of those cases the section either drew a refusal sentence instead
/// or is describing an annotation that no longer exists, and a second sentence
/// about a failed preview would be noise on top of an answer already given.
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
    ///
    /// `&mut self` and a borrowed return, so the string is neither cloned per
    /// frame nor re-derived — the same shape
    /// `crate::panels::properties::text::TextStyleDraft::sync` uses for the far
    /// more expensive run inspection next door, and for the same reason.
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
///
/// Returns whether it drew, so [`super::body_sections`] knows the panel is
/// already saying something — the same contract its five sibling sections have,
/// and for the same reason: *"nothing is selected"* under a section describing
/// the thing that is selected would be the panel contradicting itself.
///
/// ## The three outcomes
///
/// | state | drawn |
/// |---|---|
/// | nothing selected, or a content selection | nothing, and `false` |
/// | a gate refuses | the refusal sentence, and `true` |
/// | the delete would work and carry collateral | the collateral sentence, and `true` |
/// | the delete would work and carry none | nothing, and `false` — the overwhelmingly common case |
///
/// ★ The last row is R9 rather than an omission. A line reading *"deleting this
/// affects nothing else"* on every selection would be read three times and
/// skipped for ever after, which is precisely what makes the row above it
/// invisible when it matters.
pub fn section(ui: &mut Ui, doc: &OpenDoc, memo: &mut DeletionPreview) -> bool {
    let Some(selection) = doc.selection.annot() else {
        return false;
    };
    let target = &selection.target;

    // ★★★ R83 — ASKED HERE, BEFORE ANYTHING IS DRAWN, THROUGH THE SAME FUNCTION
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
        // ★ Written EVERY frame this section runs, refused or not and collateral
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
    // ★★★ `ui.min_rect()`, AFTER drawing. `max_rect` is the space a `Ui` is
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

    /// ★★★ **Every refusal the query documents earns its own sentence**, and
    /// none of them falls through to the catch-all.
    ///
    /// The failure this pins is the one that is invisible in a diff: somebody
    /// adds an arm to [`refusal_for`], mistypes a variant name, and the compiler
    /// is happy because `_ =>` catches it. The operator then meets *"pdfcer
    /// cannot delete comments or markup from this document"* where they should
    /// have met the sentence naming the certification signature — which is the
    /// difference between a dead end and an explanation.
    ///
    /// The same test guards `crate::app::actions::xobject::refusal_for` for
    /// `unshare_form`, and it is the same test because it is the same hazard.
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

    /// ★★ **The three sentences are three different sentences.**
    ///
    /// An encrypted drawing and a certified one look identical on the canvas.
    /// If these collapsed to one wording the enum would be decoration, and an
    /// operator would be sent hunting for a signature that is not in their file.
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

    /// ★★★ **`Locked` beats the document's own refusal when both are true.**
    ///
    /// Asserted through [`Refusal::line`] rather than by re-reading the branch,
    /// because what is being pinned is the *operator's* outcome: the more
    /// actionable of two true facts is the one that gets said. An operator told
    /// their comment is marked unchangeable can go and look at that comment; an
    /// operator told the document is certified can do nothing about one
    /// annotation.
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
    ///
    /// Named by object number rather than found by a walk, deliberately. The
    /// fixture is authored by this repository byte for byte, so the number is a
    /// fact about a file in this tree rather than an assumption about a
    /// document somebody else produced — and a test that *searched* for "the
    /// square" would pass on a fixture that had lost its pop-up, which is half
    /// of what these assert.
    const SQUARE: u32 = 20;

    /// A target naming the fixture's square, unlocked.
    ///
    /// ★ `locked: false` matters. [`gate`] checks §12.5.3 bit 8 **first**, so a
    /// locked target would be refused by the older half of the ladder and the
    /// certification assertion below would pass without the certification being
    /// consulted at all.
    fn square_target() -> crate::canvas::selection::annot::AnnotTarget {
        crate::canvas::selection::annot::AnnotTarget {
            page: 0,
            id: ObjId::new(SQUARE, 0),
            kind: crate::canvas::selection::annot::AnnotKind::Markup,
            subtype: "Square".to_owned(),
            locked: false,
        }
    }

    /// ★★★ **An enforced certification withholds the Delete control, and the
    /// sentence names the signature.**
    ///
    /// The end-to-end assertion for R83 on this surface: the engine's query is
    /// asked, its `EditError` is mapped, and the operator-facing sentence is the
    /// one about a certification rather than the catch-all. Every link in that
    /// chain is exercised on a real file rather than on a constructed error.
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

    /// ★★★ **The gate answers about the selection it is HANDED, not the one on
    /// the document** — the regression test for the defect of 2026-08-29.
    ///
    /// # What went wrong, in one sentence
    ///
    /// `canvas::interact` moves the selection off the document for the length of
    /// a canvas frame (`std::mem::take(&mut doc.selection)`), and it filled in
    /// `canvas::keys::Keys::annot_delete_refused` by calling
    /// [`refuses_selected`] — which reads `doc.selection`. So the flag was
    /// `false` on every frame, on every document, and the Delete key's
    /// annotation rung never declined once: it raised the action, the engine
    /// refused it into `vector_edit`'s silent `Err` arm, and the selection was
    /// cleared anyway, removing the panel sentence that explained the refusal.
    ///
    /// # ★★ Why THIS shape of test, and why the old one could not have caught it
    ///
    /// The assertion in `a_certified_document_withholds_the_delete_control`
    /// above is `refuses_selected(&doc) == doc.selection.annot().is_some()`,
    /// which on a freshly-opened fixture is `false == false` — true of the fixed
    /// build and true of the broken one. Every unit test in `canvas::keys`
    /// likewise sets `annot_delete_refused` **by hand**, so none of them was
    /// ever downstream of the call that was wrong.
    ///
    /// What this test asserts is the property the caller actually needs: that
    /// the two arguments are independent, by putting the annotation in a
    /// **detached** selection and leaving `doc.selection` empty — which is
    /// precisely the state `canvas::interact` is in when it asks. A build that
    /// reaches for `doc.selection` answers `false` here and fails.
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

    /// ★★★ **The same document without the certification offers the control**,
    /// which is what makes the test above evidence rather than a tautology.
    ///
    /// A `gate` that refused unconditionally would satisfy the certified
    /// assertion perfectly. The two fixtures differ in exactly one dictionary,
    /// so this pins that the difference the gate reacts to is that dictionary
    /// and not the presence of a signature, or of an annotation, or of a
    /// pop-up.
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

    /// ★★★ **The collateral is stated before the click, with both clauses.**
    ///
    /// `annotation_deletion_preview` on the square must find the `/Popup`
    /// companion (§12.5.6.14 makes taking it a `shall`) and the one `/IRT`
    /// referrer that Table 170's default `/RT` of `R` classifies as a **reply**.
    /// Two clauses rather than one, because one would not prove the joining is
    /// right.
    ///
    /// ★ Asserted through [`DeletionPreview::line`] — the memo — rather than by
    /// calling the engine directly, so what is pinned is the string an operator
    /// would read on the frame the annotation is selected, stamp and all.
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

    /// ★★ **The memo answers from the stamp on the second call.**
    ///
    /// The whole cost argument rests on this: `annotation_deletion_preview`
    /// walks the page's `/Annots` looking for `/IRT` referrers, and the panel
    /// would otherwise pay that every frame the annotation stayed selected. A
    /// stamp hit is asserted by *poisoning the payload* and reading it back —
    /// if the engine had been re-asked, the real sentence would have returned
    /// and the poison would be gone.
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

    /// ★★ **A moved epoch re-asks**, which is the other half of the stamp.
    ///
    /// Without the epoch term the panel would show the collateral of a document
    /// state that no longer exists — a reply deleted a moment ago would go on
    /// being counted, which is exactly the failure that makes a properties
    /// panel untrustworthy. Asserted the same way round: the poison must be
    /// gone.
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
