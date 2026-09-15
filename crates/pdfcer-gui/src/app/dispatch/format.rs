//! # `app::dispatch::format` — the contextual Format tab's command arms
//!
//! ## The seam
//!
//! The same one `dispatch::pages` took, one tab over. [`super`]'s subject is
//! *"a command id becomes an intent"* across the whole ribbon; this file's is
//! the **Format tab's** share of it. They change for different reasons: a new
//! tab or a new dispatch convention touches the parent, a new verb on the
//! thing-you-just-clicked touches this.
//!
//! Format is a small tab and this file grows with it: every `format.*` entry
//! in `manifest::PLANNED` lands here when it ships.
//!
//! ## ★ What the selection arms have in common, and it is not the tab
//!
//! They act on **the selection**, and each has to answer the same question
//! first: *which of the two index spaces is this?* A selection can name a page
//! object — an index into the page's own paint order, which every
//! `EditSession` verb accepts — or a **leaf**, an object painted from inside a
//! form XObject, whose token range indexes the form's content stream and which
//! no paint-order verb can address.
//!
//! Keeping them together is what makes their answers reviewable side by
//! side:
//!
//! | arm | what it does about a leaf |
//! |---|---|
//! | `format.delete` | raises nothing, and **says why** — an outline with an unexplained dead Delete reads as a broken program |
//! | `format.select_form` | selects the leaf's outermost enclosing form, which *is* an operand |
//! | `format.properties` | opens the panel, which describes either kind |
//!
//! ## ★★ Why the arms re-ask what `enabled_when` already asked
//!
//! Because `enabled_when` greys a ribbon item and **enforces nothing**. Every
//! non-ribbon route — the context menu, a chord, a future script — reaches the
//! dispatcher without consulting it. That was settled on this project after a
//! blanket guard at the top of `dispatch_command` was written and two tests
//! refused it, for making `Ctrl+Z` on an empty stack do nothing *and say
//! nothing*: greying is a hint, the worded decline is the answer, and only the
//! arms that would otherwise act unconditionally need the check — and they must
//! say why.

use crate::app::PdfcerApp;
use crate::app::actions::Action;
use crate::app::actions::textstyle::StyleChange;
use crate::app::state::Status;

/// Whether this file owns `id`.
///
/// `pub(crate)` rather than `pub(super)`, for the reason `dispatch::pages`
/// gives: `shell::commands::reach`'s `guard_claiming` calls it, because the
/// reachability checker must be able to EVALUATE every guard arm it finds — a
/// guard it cannot evaluate is a place commands could hide from the check that
/// exists to find them.
#[must_use]
pub(crate) fn handles(id: &str) -> bool {
    matches!(
        id,
        "format.delete"
            | "format.properties"
            | "format.select_form"
            // ★ The text-run re-aim — O188(A). It sits next to
            // `format.select_form` because it is the same act one rung down:
            // both RE-AIM the selection at something the click could not have
            // named on its own, and neither edits anything. `select_form`
            // ascends from a leaf to the form that paints it; this one descends
            // from a text block to the one line the pointer was over.
            //
            // ★★ It is the only arm in this file whose operand is not the
            // selection. It reads a pick parked in `egui::Memory` by the
            // right-click that opened the menu — which is why `dispatch`
            // takes an `egui::Context` at all — and that is not a shortcut:
            // no ribbon control can ask *"which line of this block
            // is the pointer on?"*, so the command has no ribbon home and is
            // registered `TAB_SCOPED` for exactly that reason.
            | "format.select_text_line"
            // ★ The form-XObject unshare. It sits with
            // `format.select_form` rather than with the Font group because it
            // asks the same first question every arm in this file has to ask —
            // *which of the two index spaces is this?* — and answers it the
            // same way: from a LEAF, which is the only operand either command
            // can be built from.
            | "format.unshare_form"
            // The Font group. All five, including the three whose
            // ribbon control is an `Item::Custom` — a custom control REPORTS
            // (it parks an operand and returns a token) and this file ACTS, so
            // every one of the five arrives here and none of them has a second
            // implementation inside the renderer.
            | "format.font"
            | "format.font_size"
            | "format.font_colour"
            | "format.bold"
            | "format.italic"
            // The Markup group. All six are drawn by an
            // `Item::Custom` — `app::markupband` REPORTS (it parks a
            // `(target, edit)` pair and returns a token) and this file ACTS, so
            // every one of them arrives here and none has a second
            // implementation inside the renderer.
            //
            // ★ `format.line_style` needs no machinery of its own here — it
            // parks a `MarkupEdit` like the rest of the group — which is what
            // that type's one-field-per-variant design buys.
            | "format.colour"
            | "format.fill"
            | "format.line_width"
            | "format.line_style"
            | "format.opacity"
            | "format.arrowheads"
    )
}

/// Turn one Format command into an intent.
///
/// `id` is guaranteed to be one [`handles`] claims — the caller's arm is
/// guarded on it — so the fall-through is unreachable and says so rather than
/// silently doing nothing.
pub(crate) fn dispatch(
    app: &mut PdfcerApp,
    ctx: &egui::Context,
    id: &str,
    actions: &mut Vec<Action>,
) {
    match id {
        // ★ The ribbon's Delete — the contextual Format tab's one command.
        //
        // The id is `format.delete`, not `edit.delete`: `RIBBON_IA.md`
        // §5.8 puts Delete on the **Format** tab, which is contextual and
        // appears only while something is selected, and
        // `shell::commands` registers exactly that id gated on
        // `selection.any`. There is no `edit.delete` in this build, and
        // adding an arm for one would be an arm no token can ever reach —
        // dead code wearing a design pattern, which is what the
        // no-placeholders invariant forbids.
        //
        // ★★ **The selection lives on `OpenDoc`, and this function's
        // `egui::Context` is not a licence to move it back into frame
        // memory.** The context is here so `format.select_text_line` can read
        // an operand the RIBBON could not have asked for — a pointer pick
        // parked by the right-click. State kept in `egui::Memory` is invisible
        // to everything outside a frame and no test can hold it.
        //
        // **The rule is not restated here.**
        // `crate::canvas::deleting::subject` decides what a Delete may act on,
        // for whichever rung the operator is at, and the canvas's Delete key
        // asks the same function. Two statements of a destructive rule is one
        // too many.
        //
        // An empty list raises nothing rather than an empty action the
        // engine would have to refuse. That is reachable in practice: the
        // Format tab is visible whenever *anything* is selected, including
        // at a rung whose delete verb does not exist yet.
        "format.delete" => {
            if let Status::Open(doc) = &app.status {
                // ★★★ **A FORM FIELD FIRST, and this arm must stay in step
                // with the Delete KEY.**
                //
                // `canvas::keys`' Delete ladder reaches a selected widget at
                // its first rung. An arm here that did not would make
                // Delete-the-key and Delete-the-command act on different
                // things, which is precisely what `app::keyboard`'s header
                // calls the defect the single dispatcher exists to make
                // impossible.
                //
                // ⇒ Nothing surfaces such a divergence while the only route to
                // the command is the Format tab, because the Format tab is not
                // drawn for a form selection. The `canvas.field` right-click is
                // the second door, and it is where the divergence shows up as
                // *"the menu's Delete does nothing"*.
                //
                // ★★ The guard is `edit_content`, matching `canvas::keys` and
                // matching where the selection is offered at all: `canvas::forms`
                // gives the selection surface to Edit and the fill surface to
                // Read and Review. One predicate per capability.
                //
                // ★ `DeleteWidget`, not `DeleteField` — this box, not every box
                // the field owns. A field with two widgets on two pages is one
                // field the operator can select from either place, and deleting
                // the whole field because they pointed at one of its boxes
                // would destroy work on a page they are not looking at. The
                // Properties panel offers both, labelled, which is the right
                // place for a choice.
                if app.capabilities().edit_content
                    && let Some(field) = &doc.selected_field
                {
                    // ★★★ R83 — ASKED HERE, THROUGH THE SAME FUNCTION THAT
                    // WITHHOLDS THE MENU ITEM AND DRAWS THE SENTENCE.
                    //
                    // Without it — and without a `visible_when` on the
                    // `canvas.field` menu that is this arm's only pointer
                    // route — an ordinary certified fillable form draws the
                    // item live and undimmed, the press reaches
                    // `delete_widget`, and the refusal lands in
                    // `actions::apply::vector_edit`'s `Err` arm and says
                    // nothing to the operator.
                    //
                    // `panels::properties::formfield::refuses_delete` is the
                    // one derivation, asked here so that this arm, the
                    // `canvas.field` menu (via
                    // `app::conditions`' `selection.delete_permitted`),
                    // `canvas::keys`' rung 0 and the Properties panel's
                    // sentence are **one question with four readers**. A
                    // control withheld by one rule while a panel explains a
                    // different one is the shape the forms audit found.
                    //
                    // ⇒ Reaching this branch refused means one of the two
                    // things the annotation arm below lists: a **chord** bound
                    // to `format.delete` (a chord consults no `visible_when`),
                    // or the condition having gone stale within a frame. The
                    // sentence for both is already on screen in the Properties
                    // panel — and, since the verb no longer clears the
                    // selection ahead of the engine, it stays there — so this
                    // declines silently to the trace rather than inventing a
                    // second wording.
                    if crate::panels::properties::formfield::refuses_delete(doc) {
                        crate::diag::trace(|| {
                            // ui-text-exempt: diagnostic trace, never displayed.
                            format!(
                                "command-declined id={id} reason=field-delete-refused field={}",
                                field.field
                            )
                        });
                        return;
                    }
                    actions.push(Action::Field(
                        crate::app::actions::forms::FieldAction::DeleteWidget {
                            field: field.field.clone(),
                            widget: field.widget,
                        },
                    ));
                    return;
                }
                // ★ An ANNOTATION first — not a tie-break: `SelectionState`
                // cannot hold both, so these are the two cases of one
                // question. Locked (§12.5.3 bit 8) does nothing rather than
                // raising an action the engine would refuse; the control
                // itself should be absent, which is the Format tab's work.
                if let Some(annot) = doc.selection.annot().filter(|_| {
                    // ★ `author_markup`, NOT `edit_content` — one predicate per
                    // capability, the rule `canvas::keys` states beside its own
                    // pair. **Review must keep this**: deleting a markup is
                    // exactly what Review is for, and a guard that reached for
                    // `edit_content` here would take the working verb away from
                    // the mode that owns it while leaving the broken one in
                    // Read. The two capabilities are separate questions and
                    // neither stands in for the other.
                    app.capabilities().author_markup
                }) {
                    // ★★★ R83 — ASKED HERE, THROUGH THE SAME FUNCTION THAT
                    // WITHHELD THE CONTROL.
                    //
                    // ⚠ §12.5.3 Table 165's `Locked` bit is only part of the
                    // answer: `/Encrypt` and a certification signature refuse a
                    // deletion too. A guard reading `locked` alone pushes the
                    // action on a certified drawing, `delete_annotation`
                    // refuses, and `actions::apply::vector_edit`'s `Err` arm
                    // writes one line to the trace and says **nothing to the
                    // operator**.
                    //
                    // `annotation_deletion_refusal` is the third part, and it
                    // is asked through `annotdelete::gate` rather than directly
                    // so that this arm, `canvas::keys`' Delete ladder,
                    // `app::conditions`' `selection.delete_permitted` and the
                    // Properties panel's sentence are **one derivation with four
                    // readers**. A control withheld by one rule while a panel
                    // explains a different one is the shape the forms audit
                    // found.
                    //
                    // ⇒ Reaching this branch at all now means one of two things,
                    // and neither is a state the operator can see: a **chord**
                    // bound to `format.delete` (a chord consults no
                    // `visible_when`), or the condition having gone stale within
                    // a frame. The sentence for both is already on screen in the
                    // Properties panel, which is why this declines silently to
                    // the trace rather than inventing a second wording.
                    match crate::panels::properties::annotdelete::gate(doc, &annot.target) {
                        Some(refusal) => crate::diag::trace(|| {
                            // ui-text-exempt: diagnostic trace, never displayed.
                            format!(
                                "command-declined id={id} reason=annot-delete-refused why={refusal:?}"
                            )
                        }),
                        None => actions.push(Action::Annot(
                            crate::app::actions::annot::AnnotAction::Delete {
                                page: annot.target.page,
                                id: annot.target.id,
                            },
                        )),
                    }
                } else if !app.capabilities().edit_content {
                    // ★★★ THE MODE, ASKED HERE, BECAUSE WITHOUT IT THE RIBBON
                    // DELETES PAGE CONTENT IN READ.
                    //
                    // `canvas::keys`' Delete-key path carries this guard and
                    // argues it at length. An arm here without it gives:
                    //
                    //   Read mode › click a picture › Format ▸ Delete
                    //     → `VectorAction::DeleteSelection`
                    //
                    // in the mode whose entire promise is that it authors
                    // nothing — the keyboard refusing and the button doing it.
                    //
                    // ★★ **A content selection IS reachable in Read**, so
                    // *"entering a mode without the capability clears the
                    // selection, and no gesture can build a new one"* is not a
                    // guard. `canvas::clicking`'s image arm runs precisely when
                    // `!caps.edit_content` — it exists so a reader can click a
                    // picture and copy it (O71) — so every condition built on
                    // `selection.any` can be set in Read. The control is not
                    // greyed there; it is **enabled**.
                    //
                    // ★ The compound is what makes this a data-loss defect
                    // rather than an untidy one: `format.select_form` re-aims
                    // the selection from one picture to the whole form
                    // XObject, and `format.delete` then takes the lot. Click a
                    // logo inside a title block in Read, and the title block
                    // goes.
                    //
                    // ⇒ The rule, which is why this comment is long: **a
                    // guard justified as "unreachable in practice" is a claim
                    // about a different file, and it decays without either
                    // file changing.** Write it anyway, in every route, and
                    // say why.
                    crate::diag::trace(|| {
                        // ui-text-exempt: diagnostic trace, never displayed in the UI
                        "format-delete-declined reason=mode-cannot-edit-content".to_owned()
                    });
                } else {
                    delete_the_selection(doc, actions);
                }
            }
        }

        // ★★★ **Select the form that contains what is selected.**
        //
        // The deliberate second act that pays for the deep hit test. A click
        // reaches inside a form XObject and the form itself is excluded from
        // the hit test outright, because a `/BBox` is a clipping extent and
        // not a claim about ink — include it and a page-sized form wins every
        // click at every point, which the operator reported as *"all I get is
        // the page selected"*.
        //
        // A form is nonetheless a legitimate thing to want: it is one page
        // object with an ordinary paint-order index, and moving a title
        // block is *the form*, not the two hundred objects inside it. This
        // is the route to it, and it is the only one on the canvas.
        //
        // # The FIRST leaf, not all of them
        //
        // A multi-selection can hold leaves from several different forms,
        // and there is no single container for such a set. Taking the
        // first — in `leaf_indices_on`'s ascending, deduplicated order, so
        // it is deterministic rather than click-ordered — makes the act
        // mean one thing always. `select_only` then replaces the selection
        // outright, which is the honest report: what you now have is the
        // form, and not the set you had before.
        // ★ Guarded with the other re-aims. Non-destructive on its own, but
        // it is the FIRST HALF of the compound that makes A18 a data-loss
        // defect: in Read, click a picture inside a title block,
        // `select_form` re-aims the selection from the one image to the whole
        // form XObject, and `format.delete` then takes the lot.
        "format.select_form" if !app.capabilities().edit_content => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "format-select-form-declined reason=mode-cannot-edit-content".to_owned()
            });
        }
        "format.select_form" => {
            if let Status::Open(doc) = &mut app.status {
                let page = doc.view.page_index;
                let container = doc
                    .selection
                    .leaf_indices_on(page)
                    .first()
                    .and_then(|&leaf| {
                        let target = crate::canvas::target::TargetId::Leaf(leaf as u64);
                        doc.page_objects()?.containing_form(page, target)
                    });
                match container {
                    Some(form) => {
                        doc.selection.select_only(page, form, "select-form");
                    }
                    // ★★★ **The sentence must say that NOTHING is inside a
                    // form, not that something is.**
                    //
                    // Nothing selected is drawn inside a form, or this page's
                    // model could not be read; `NoContainingForm` reports both
                    // honestly. A wording about a form-interior selection is
                    // the exact inverse of what happened here, and it would not
                    // even be visibly wrong: `Declined::still_true` retires
                    // that fact on `selection_in_form`, which is false whenever
                    // this arm runs, so the decline would be discarded on the
                    // frame it was written. `InsideFormRefusal`'s own doc
                    // carries the variant-per-fact rule.
                    None => crate::app::status::decline::record_inside_form(
                        crate::text::status::InsideFormRefusal::NoContainingForm,
                    ),
                }
            }
        }
        // ★★★ **Select just the line of text the pointer was over.**
        //
        // O188(A). The operator's words, `OPERATOR_REQUESTS.md`:
        // *"In text that is grouped together or whatever it is called, such
        // as in my title blocks, I would like a way to move the individual
        // text blocks within it around, and have the ability to delete
        // them"*.
        //
        // Deleting one line works. **Without this arm, reaching the rung it
        // works at has exactly one route and nothing in the program names
        // it**: arm the Points tool first (`A`), then single-click. A route he
        // can find only after already failing is not a route, which is the
        // whole of what this arm is for.
        //
        // # Where the operand comes from, and why it is not the selection
        //
        // Every other arm in this file acts on `doc.selection`. This one
        // cannot: the selection at the moment of the click is the whole text
        // block, and *which line* is a property of the POINTER, not of the
        // selection. `canvas::menus` parks the pick — page, object, run
        // index, and how many runs the object has — on the frame that opens
        // the context menu, because egui reopens that menu every frame while
        // the pointer travels onto it, and by the time this arm runs the
        // pointer is over a menu row and not over the text at all. The markup
        // node menu established the pattern; `canvas::runmenu` states the
        // argument in full.
        //
        // # What it deliberately does not do
        //
        // It does not edit, move, or delete anything, and it raises no
        // `Action`. Re-aiming the selection is not an edit, which is why a
        // menu row is allowed to do it where `DESIGNS.md` §6.2 forbids a menu
        // row that edits. Nothing is pushed, nothing is undoable, and Escape
        // ascends back to the whole block exactly as it does from any
        // Part-rung selection.
        //
        // # Why a decline raises nothing
        //
        // The same reason the markup node commands do. By the time this arm
        // runs the menu has closed, and a closed menu is not a surface an
        // explanation can arrive on. It is also not reachable in practice:
        // the row is ABSENT unless a pick was parked with a live run on the
        // current page, per R9 — not greyed, absent — so a decline here means
        // the model changed between the click and the press, and the honest
        // report of that is to leave the selection where the operator last
        // saw it. The trace still says so, for the harness.
        //
        // ★ Guarded on `edit_content` with the other re-aim, and for the
        // same measured reason: `select_form` + `delete` is a data-loss
        // compound in Read mode (A18). This one re-aims DOWN rather than up,
        // so the same compound narrows a delete rather than widening it —
        // but Read mode does not select parts of content at all, and an arm
        // that acts anyway is the divergence, not the risk it happens to
        // carry.
        "format.select_text_line" if !app.capabilities().edit_content => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "format-select-text-line-declined reason=mode-cannot-edit-content".to_owned()
            });
        }
        "format.select_text_line" => {
            if let Status::Open(doc) = &mut app.status {
                let page = doc.view.page_index;
                // ★ The `Ref` is taken and dropped inside this block, before
                // `doc` is borrowed mutably to move the selection. `resolve`
                // re-validates the parked pick against the CURRENT model —
                // the object is still a text run, the run index is still in
                // range, the page is still the one that was clicked — so a
                // reflow between the click and the press narrows the
                // selection to nothing rather than to somewhere the operator
                // never pointed.
                let resolved = {
                    let targets = doc.page_objects();
                    crate::canvas::runmenu::resolve(ctx, targets.as_deref(), page)
                };
                if let Some((object, run)) = resolved {
                    doc.selection
                        .select_part(page, object, run, "select-text-line");
                }
            }
        }
        // ★★★ **Give this page its own copy of the shared drawing.**
        //
        // The "option" half of `pdfcer-core`'s decision 076, and the only route
        // in this shell to `EditSession::unshare_form`.
        //
        // # What this arm computes, and why it is the dispatcher's job
        //
        // Two hops, both of which must happen on the near side of the action
        // funnel:
        //
        // 1. the selection's **first leaf** on this page — the same accessor,
        //    in the same ascending, de-duplicated order, that
        //    `format.select_form` above takes its container from, so the two
        //    commands cannot come to different answers about which form the
        //    operator means;
        // 2. that leaf's **outermost** enclosing form, as an `ObjId`, through
        //    `ObjectModelProvider::containing_form_object`.
        //
        // `containing_form` (paint order) and `containing_form_object`
        // (`ObjId`) are the two halves of one question, and this verb needs the
        // second: `unshare_form`'s signature is `(page_index, form: ObjId)`. Its
        // doc comment carries the argument for why an `ObjId` cannot serve the
        // *selection* act and a paint-order index cannot serve this one.
        //
        // ★★ **OUTERMOST, and passing the innermost would be a live defect.**
        // `FormLeaf::containment` is *"outermost first"*, so position 0 is the
        // form the PAGE invokes and the last entry is the form the object sits
        // directly inside. `unshare_form` refuses a nested invocation by name —
        // `FormNestedInAnotherForm` — because re-binding one means editing the
        // parent, whose blast radius depends on the document's nesting
        // structure. Handing it `parent()` would therefore produce a worded
        // refusal on every nested drawing, for an operand nobody chose.
        //
        // # The FIRST leaf, not all of them
        //
        // `format.select_form`'s reason, unchanged: a multi-selection can hold
        // leaves from several forms and there is no single container for such a
        // set. Taking the first in a deterministic order makes the act mean one
        // thing always.
        //
        // ⇒ And it is honest here in a way it would not be for a bulk verb: the
        // granularity of `unshare_form` is **one page, one form**, so "unshare
        // everything selected" is not a call the engine offers. See
        // `app::actions::xobject`'s header.
        //
        // # Why the arm re-asks what `enabled_when` already asked
        //
        // The file header's rule, applied: greying is a hint and enforces
        // nothing — the context menu, a chord and a future script all arrive
        // here without consulting it — so the arm asks again **and says why**.
        // The sentence is `UnshareRefusal::NothingInAForm`, which deliberately
        // is not `record_inside_form`'s: that one reports a verb refusing
        // BECAUSE the selection is in a form, and this one refuses because it is
        // not. Reusing it would state the exact inverse of what happened.
        // ★ Guarded — it WRITES TO THE DOCUMENT (`EditSession::unshare_form`)
        // and is reachable from Read by the same route as the re-aims above.
        "format.unshare_form" if !app.capabilities().edit_content => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "format-unshare-declined reason=mode-cannot-edit-content".to_owned()
            });
        }
        "format.unshare_form" => {
            if let Status::Open(doc) = &app.status {
                let page = doc.view.page_index;
                let form = doc
                    .selection
                    .leaf_indices_on(page)
                    .first()
                    .and_then(|&leaf| {
                        let target = crate::canvas::target::TargetId::Leaf(leaf as u64);
                        doc.page_objects()?.containing_form_object(page, target)
                    });
                match form {
                    Some(form) => actions.push(crate::app::actions::Action::XObject(
                        crate::app::actions::xobject::XObjectAction::Unshare { page, form },
                    )),
                    // Nothing selected is inside a form, the page's model has
                    // gone, or a leaf carried an empty containment chain (which
                    // the engine documents as impossible). All three mean the
                    // same thing to the operator — this verb had no operand —
                    // and the sentence tells them where to click to make one.
                    None => crate::app::status::decline::record_unshare(
                        crate::text::unshare::UnshareRefusal::NothingInAForm,
                    ),
                }
            }
        }
        "format.properties" => {
            actions.push(crate::app::actions::Action::Command(
                // ui-text-exempt: a registered command id, never displayed
                "file.properties".to_owned(),
            ));
        }
        // ★★★ The Font group. Five ids, two operand shapes, ONE derivation of
        // *which text*.
        //
        // Bold and Italic carry their operand in the button they are, so their
        // `StyleChange` is built here. The face chooser, the size field and the
        // colour swatch cannot — a `HandlerToken` has no room for
        // "Helvetica-Bold" — so `app::fontband` parks theirs on
        // `PdfcerApp::font_change` and this takes it.
        //
        // ★ **The page and the runs are derived here for all five**, through
        // `app::textoperand::resolve`, and that is the point of routing the custom
        // controls through a command at all. The alternative — the renderer
        // building a whole `Action::TextStyle` because it already has the
        // document in hand — would put the *"which runs does a restyle act
        // on?"* rule in two places, and the copy in the renderer would be the
        // one a chord never reached.
        "format.font" | "format.font_size" | "format.font_colour" | "format.bold"
        | "format.italic" => {
            // Built before the document is borrowed, because `take` needs
            // `&mut app` and the operand read needs `&app.status`.
            let change = match id {
                // ★ Bold and Italic are **buttons that apply, not switches
                // that reflect**, which is why each names one attribute and
                // sets the other false rather than toggling a remembered pair.
                // There is no "is this run bold" bit in a PDF — weight is a
                // property of the *face*, and a synthetic weight is a stroke
                // width in the content stream — so a toggle would be claiming
                // to have read a fact that is not recorded. The Properties
                // panel's twin controls make the same choice and its header
                // carries the full argument.
                "format.bold" => Some(StyleChange::Weight {
                    bold: true,
                    italic: false,
                }),
                "format.italic" => Some(StyleChange::Weight {
                    bold: false,
                    italic: true,
                }),
                _ => app.font_change.take(),
            };
            // ★ `None` here is not a defect and raises nothing. It is the
            // ordinary state of a custom control the operator hovered without
            // changing: the ribbon returns a token only when something was
            // invoked, but a token can also arrive from a chord bound to one
            // of the three custom-drawn ids, and a chord cannot park an
            // operand. Silence is the honest answer — there is no value to
            // apply and nothing was refused.
            let Some(change) = change else {
                return;
            };
            let Status::Open(doc) = &app.status else {
                return;
            };
            // ★ The **one** derivation of *which runs a restyle acts on*,
            // asked of `app::textoperand` so that this arm, the five commands'
            // `enabled_when` and both font surfaces cannot disagree. It answers
            // with a live swept range if there is one, and otherwise with the
            // single selected text object resolved through its byte span.
            //
            // ★★★ **The object rung is why the press costs something here and
            // nowhere else.** Resolving an object operand runs one page
            // extraction with provenance capture — 392 ms on the operator's
            // benchmark sheet — and this is the correct place to pay it: once
            // per gesture, on the frame the operator pressed something. The
            // condition that lit the control asked the cheap half; the draft
            // caches in `app::fontband` and `panels::properties::textobject`
            // answer the per-frame read-back. See `app::textoperand`'s header.
            //
            // ★ `runs` owns the staleness gate for the swept rung — a stale
            // run ordinal restyles the WRONG text, so the check lives with the
            // data rather than with each of its readers.
            //
            // A missing operand raises nothing and says nothing, and that is
            // deliberate rather than an omission: the ribbon control is greyed
            // on `selection.text_runs` in exactly this state, so an operator
            // cannot reach here by clicking. The route that can is a chord, and
            // a chord pressed with nothing selected is the operator asking a
            // question, not making a mistake worth a sentence in the status bar.
            let Some(operand) = crate::app::textoperand::resolve(doc) else {
                return;
            };
            actions.push(crate::app::actions::Action::TextStyle {
                page: operand.page,
                runs: operand.runs,
                change,
            });
        }
        // ★★★ The Markup group. ONE operand shape, and the operand arrives
        // WITH the token rather than being re-derived here.
        //
        // # Why this is the opposite of the Font arm above, deliberately
        //
        // The Font arm re-reads `doc.text_selection` for all five of its ids,
        // and its own note says why: *"which runs does a restyle act on?"* is a
        // **rule**, and a rule stated in the renderer as well as here is a rule
        // that diverges — with the renderer's copy being the one a chord never
        // reaches.
        //
        // This operand is not a rule. It is *the annotation the swatch was
        // showing the colour of*, which `app::markupband` had already read out
        // of the session in order to draw itself. Re-deriving it here would
        // open a window this surface cannot afford: the control reads
        // `doc.selection.annot()` while drawing, the dispatcher would read it
        // again after the frame's input has been applied, and a canvas click in
        // the same frame — deselecting, or selecting a different mark — would
        // send the operator's colour to a mark they were no longer looking at.
        // A restyle applied to the wrong annotation is a wrong document, and
        // `MarkupStyleChange` is not undoable in halves.
        //
        // ⇒ So the arm **verifies** rather than re-derives, which is the honest
        // middle: the parked target must still be what the selection names.
        //
        // ★ `None` raises nothing and is not a defect: it is what a chord
        // bound to one of these ids produces, because a chord cannot park an
        // operand. Silence is the honest answer — there is no value to apply
        // and nothing was refused.
        // ★ `format.line_style` needs no machinery of its own here: it parks a
        // `MarkupEdit` like the rest of the group, so it is the same operand
        // shape reaching the same verb. That is the property `MarkupEdit`'s
        // one-field-per-variant design buys — a new control is an enum variant
        // and an arm of this list, and nothing about the dispatch has to learn
        // what a dash is.
        "format.colour" | "format.fill" | "format.line_width" | "format.opacity"
        | "format.line_style" | "format.arrowheads" => {
            // Taken before the document is borrowed: `take` needs `&mut app`
            // and the check below needs `&app.status`.
            let Some((target, edit)) = app.markup_change.take() else {
                return;
            };
            // ★★ `author_markup`, NOT `edit_content` — one predicate per
            // capability, the rule `canvas::keys` states beside its own pair
            // and the Delete arm above repeats. **Review must keep this**:
            // restyling a mark is exactly what Review is for, and a guard
            // reaching for `edit_content` would take the working verb away from
            // the mode that owns it.
            //
            // ★ Re-asked here although `enabled_when` and `shown_when` both
            // carry it, for this file's standing reason: greying is a hint and
            // enforces nothing — a chord consults no condition at all.
            if !app.capabilities().author_markup {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "format-markup-style-declined reason=mode-cannot-author-markup".to_owned()
                });
                return;
            }
            let Status::Open(doc) = &app.status else {
                return;
            };
            // ★★★ The parked target must still be the selection, and the
            // §12.5.3 lock must still be clear.
            //
            // Both are re-asked rather than trusted, and neither is reachable
            // by clicking: the control was drawn from this very selection one
            // frame ago. What IS reachable is a chord, and a frame in which the
            // canvas moved the selection between the ribbon's draw and this
            // dispatch. The engine refuses a locked annotation by name, so
            // without this the refusal would arrive in
            // `actions::apply`'s `Err` arm and say nothing to the operator —
            // which is the silent-decline class this project was founded on.
            //
            // ★ It declines to the TRACE rather than to the status bar for
            // `format.delete`'s reason above: the sentence for a
            // locked mark is already on screen, in the Properties panel and on
            // the greyed control's own hover
            // (`text::panels::properties::markup_locked`), so inventing a
            // second wording here would be two statements of one refusal.
            let addressed = doc
                .selection
                .annot()
                .is_some_and(|a| a.target == target && !a.target.locked);
            if !addressed {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    format!(
                        "format-markup-style-declined id={id} reason=selection-moved-or-locked \
                         page={} annot={:?}",
                        target.page, target.id
                    )
                });
                return;
            }
            actions.push(crate::app::actions::Action::SetMarkupStyle {
                page: target.page,
                id: target.id,
                style: edit.into_style(),
            });
        }
        // ui-text-exempt: a panic message, read from a stack trace by whoever
        // adds an id to `handles` and forgets the arm. Never rendered.
        other => unreachable!("dispatch::format was handed `{other}`, which it does not claim"),
    }
}

/// **The ribbon's Delete, asked through the same function as the key.**
///
/// # ★★★ Why this is a function and not four lines inside the arm
///
/// Because the rule is destructive and there must be exactly one of it. Both
/// claimants — this command and the Delete **key** in `canvas::keys` — ask
/// `crate::canvas::deleting::subject`, which answers for whichever rung of the
/// ladder the operator is on.
///
/// Two hand-written copies of one destructive rule is exactly what
/// `deletable_objects_on`'s own header refuses, and the divergence is not
/// hypothetical: it has happened over form fields, where the key reached a
/// selected widget and this command did not, so Delete-the-key and
/// Delete-the-command acted on different things — which `app::keyboard`'s
/// header calls the defect the single dispatcher exists to make impossible.
///
/// **Both** callers therefore ask [`crate::canvas::deleting::subject`], and
/// the Part and Node rungs come with it: the ribbon's Delete removes one line,
/// one label or one corner point exactly as the key does, because there is
/// only one answer to ask for.
///
/// # ★★ The provider, and why it is read here rather than passed in
///
/// The dispatcher is not inside the canvas's frame — it runs from the command
/// funnel, after the ribbon or a menu has already closed — so it cannot inherit
/// the canvas's borrow the way `canvas::keys` does. (It takes an
/// `egui::Context` for a different arm's parked operand; a context is not a
/// frame and buys this one nothing.) `doc.page_objects()` is keyed on
/// `(page, edit_epoch)` and the canvas built it on the frame that drew the
/// selection outline the operator is looking at, so this is a cache read rather
/// than a second `decompose_page` — the same key, the same epoch, the same
/// `Ref`.
///
/// The `Ref` is taken and dropped inside the `map_or_else` below, before
/// anything is pushed, because `doc` is borrowed immutably for the whole arm and
/// holding a `RefCell` guard across an `actions.push` is how a re-entrant read
/// becomes a panic nobody can reproduce.
///
/// # What it deliberately does NOT do
///
/// It does not clear the selection, it does not take the erase preview, and it
/// does not word anything. The first two belong to `actions::vector::apply`
/// (which owns the four-step protocol and the O63 preview) and the third to
/// [`crate::text::deleting`]. This function's whole job is *ask, then raise*.
fn delete_the_selection(doc: &crate::app::state::OpenDoc, actions: &mut Vec<Action>) {
    let page = doc.view.page_index;
    let outcome = {
        let targets = doc.page_objects();
        crate::canvas::deleting::subject(&doc.selection, page, targets.as_deref())
    };
    match outcome {
        Ok(subject) => actions.push(crate::canvas::deleting::action(subject).into()),
        // ★ The identical channel the key uses, epoch and all — see
        // `crate::text::deleting::refusal` for which refusals speak and why the
        // rest are silent on purpose. Routing an empty operand list to the
        // inside-a-form decline instead leaves a Part or Node rung with no
        // trace line at all, and the command ends up quieter than the key.
        // ★ `true` for `model_attempted`, and it is a fact rather than a
        // convenience: this arm asks `doc.page_objects()` unconditionally just
        // above, so a `None` here can only mean the page would not decompose.
        // The key's answer can differ, because it inherits the canvas's
        // conditional borrow and this route never has one.
        Err(reason) => {
            crate::canvas::deleting::decline(&doc.selection, reason, doc.edit_epoch, true);
        }
    }
}
