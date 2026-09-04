//! # `panels::forms` — filling this document's interactive form
//!
//! Salvaged from the old shell's `main.rs` (roughly lines 7843–9425, mapped
//! here by `SALVAGE.md`'s Class C table). **The filling half came across; the
//! authoring half did not** — see "What was deliberately left behind" below.
//!
//! ## What this panel is for
//!
//! The operator's goal for this build is *"replace Acrobat Reader first"*, and
//! **a reader fills forms; it does not create fields.** So this panel offers
//! text fields, check boxes, radio groups, choice lists, a reviewed reset, a
//! reviewed native recompute of script-driven fields, an appearance redraw and
//! a flatten. Nothing here adds, renames or moves a field.
//!
//! ★★ **One exception, added 2026-08-28, and it is stated rather than left to
//! be discovered:** the [`groups`] section removes a *field group* — a name the
//! form files fields under — and every field beneath it. It is here and not on
//! an authoring surface because a grouping node is reachable from nowhere else:
//! it has no widget to click on the page, no row in the fill list, and no entry
//! in the tab order, so the Properties pane can never be pointed at one. See
//! that module's header.
//!
//! ## ★ This is the first panel that changes the document
//!
//! Every other panel in [`crate::panels`] is a report, and two
//! ([`crate::panels::layers`], [`crate::panels::bookmarks`]) change what is
//! *drawn* without changing what would be *saved*. This one writes `/V`.
//!
//! It changes nothing about the discipline. The body is handed `&OpenDoc` — a
//! **shared** reference, so this is a compile-time fact and not a convention —
//! it reads, and it raises a [`crate::app::actions::Action`]. What is new is
//! only that the action reaches an `EditSession` verb at the far end; see
//! [`edit`] for the four-step mutation protocol that makes that safe, and for
//! why nothing travels back.
//!
//! ## Rule 4, and the two places this panel had to move a disclosure
//!
//! `D:\Dev\FeatureRequests\pdfce_FeatureRequests\README.md`'s rule 4 in one
//! clause: *"Disclosure lives off-canvas: a status line, a results panel, a
//! report after the command, a properties field."* **A panel is the right
//! home**, and this panel is nothing but disclosure and controls — it draws
//! not one pixel on the page, and it must not start to. The one-line test is
//! *would a screenshot of the editing canvas differ from a screenshot of the
//! same document saved and reopened?*
//!
//! That is worth stating twice here because the old shell's Forms panel did
//! draw on the canvas: hovering a row **highlighted the field's rectangle on
//! the page** (its "Pass 47.3"), through a `self.highlighted_field` the canvas
//! overlay read. It was answering a real question — *"which of these is the
//! one I am about to type into?"* — and the answer is welcome under rule 4's
//! fourth clause, which permits *"a snap indicator, a hover highlight, a
//! rubber-band, a selection handle — these are the cursor"*. It is still not
//! carried, and the reason has **changed**: the mechanism now exists (see
//! [`crate::canvas::forms`], which places every fillable widget in canvas
//! space and is `pub(crate)`), so what is missing is only the panel→canvas
//! channel for *which row is hovered*. Named rather than silently dropped, and
//! named as a *permitted* affordance so nobody later reads its absence as a
//! rule.
//!
//! ## ★ The page is now a second way in, and this panel is still the first
//!
//! [`crate::canvas::forms`] lets an operator click a field where it is drawn
//! and type into it — the gesture every reader has and this build did not.
//! Nothing about this panel changed to make room for it: the two surfaces share
//! [`rows::block_reason`], [`rows::commit`] and the whole [`edit::FormEdit`]
//! vocabulary, so there is one rule for what is fillable, one rule for when a
//! draft is written, and one place a form verb is called.
//!
//! What this panel gained is two obligations, both of them disclosure:
//!
//! 1. **[`fill_disclosure`]** — the two things a fill decides that the document
//!    cannot afterwards be asked (an auto-size pdfcer chose, characters it
//!    replaced). Those were previously discarded on the argument that
//!    everything is re-derivable next frame; that argument is true of six of
//!    `FillOutcome`'s eight facts and false of these two. See [`edit`]'s header.
//! 2. **[`canvas_routing`]** — which fields the page cannot be clicked for, and
//!    why. Without it the canvas silently shrinks the capability from the
//!    operator's point of view.
//!
//! **This panel remains the accessible surface, and that is not a courtesy.**
//! Its rows are real widgets with tab order, AccessKit exposure and `/TU`
//! labels; a box projected onto a page raster has none of those, because the
//! thing underneath it is a picture with no text alternative.
//!
//! Two disclosures the old shell reported **after** an edit are reported
//! **before** one here, and both moves are improvements rather than
//! translations:
//!
//! | Fact | Old shell | Here |
//! |---|---|---|
//! | this form carries an XFA packet, so a fill may not stick | a status note after each fill, from `FillOutcome::xfa_may_disagree` | one line above the list, from `AcroForm::xfa` — a property of the FILE, knowable before anything is typed |
//! | this check box has no appearance for the state you selected | a status note after the click | the control is **disabled**, because core would refuse the call — see [`rows`]' header for the defect this replaced |
//!
//! ## Two counts that are not the counts to display
//!
//! The README's third bite — *"a returned count is not always the count to
//! display"* — lands twice in this panel, and neither instance is the worked
//! example it uses:
//!
//! 1. **"N you can fill here"** is derived from what this panel will actually
//!    draw a live control for, **not** from `Field::is_fillable`. The model's
//!    predicate knows about read-only, signature and push-button fields; it
//!    does not know that a certification signature disables the whole document
//!    or that a rich-text field is offered a conversion rather than a box. See
//!    [`crate::text::forms::forms_field_count`].
//! 2. **`AcroForm::fields.len()` is not the number of fields in the file.** It
//!    excludes `inline_field_roots` — `/Fields` entries written as direct
//!    dictionaries, which Table 218 forbids and which have no object identity
//!    a fill could write to. Disclosed rather than silently absorbed, because
//!    an operator comparing pdfcer's count against another reader's needs to be
//!    able to find out why.
//!
//! ## JavaScript is never executed
//!
//! A standing project rule, not an unfinished feature. Script-driven fields
//! are **recognised** (`Field::has_additional_actions`, surfaced as a
//! disclosure above the list) and a whitelisted subset of Acrobat's built-in
//! calculations is **recomputed natively** by
//! `pdfcer_core::form_script::recompute` — arithmetic pdfcer reproduces itself,
//! never a script it ran.
//!
//! The Calculated Fields section carries that whole posture across from the
//! old shell, including its two rule-4 disclosures: a **derived evaluation
//! order** when the form fails to list its calculated fields in `/CO` (pdfcer
//! inferred something, and another reader may compute different values), and
//! **coerced operands** where a blank or non-numeric input counted as zero.
//! Skips are listed **before** the changes, because a field pdfcer declined to
//! compute is the thing an operator most needs to notice and a list of
//! successful changes above it reads as completeness.
//!
//! The section is collapsed by default and **never auto-runs**: merely opening
//! a form must not change a computed `/V`.
//!
//! ## What was deliberately left behind
//!
//! Roughly half the salvaged range, all of it `Edit ▸ Forms` **authoring**:
//! field creation, field deletion, widget deletion, field renaming with its
//! ancestor breadcrumb, and the grouping-node roster. Also the FDF/XFDF/CSV
//! import and export surface, which needs a file dialog this stage does not
//! have. Each answers to a different ribbon command and, in the deletion and
//! renaming cases, to a **different certification gate** — see
//! [`crate::text::forms::forms_structural_certification_disabled_tooltip`].
//! They land with the commands that name them.
//!
//! ## A note on very large forms
//!
//! `pdfcer_core::forms::MAX_FORM_FIELDS` is 500,000, and this panel lays out
//! every row inside one `ScrollArea` — so a pathological form would lay out
//! half a million rows per frame. Not addressed, and stated rather than
//! discovered: the fix is `ScrollArea::show_rows`, which needs a uniform row
//! height, which these rows do not have (a multiline text field, a radio
//! cluster and a one-line combo are three different heights). Every real form
//! measured in `pdfcer-core`'s corpus is under a thousand fields.

/// The verbs this panel can ask for, and the one place they are applied.
pub mod edit;
/// The names this form files its fields under, and the shell's only route to
/// deleting one. See that module's header for why a grouping node is reachable
/// from nowhere else in the shell, and for the two-press protocol its
/// invisibility forces.
mod groups;
/// One field, one row — the per-field controls.
pub mod rows;
/// ★★ **The panel→canvas channel** — which field the panel is pointing at, so
/// the canvas can spotlight it (`OPERATOR_REQUESTS.md` O98). This header has
/// named that gap since the panel was written, and named it as a PERMITTED
/// affordance under rule 4's fourth clause; the module is that channel and
/// nothing more.
pub mod spotlight;
/// The order this form is tabbed through, per page — a **read-only** second
/// list beside the fill list. See that module's header for what it is, why it
/// is a section rather than a panel, and why it offers no reorder affordance.
pub mod tab_order;

/// What an existing push button does, and how to change it. Its own module
/// because the reader has four states and each one permits a different control.
pub(crate) mod button;

use crate::app::actions::forms::FieldAction;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::PathBuf;

use pdfcer_core::forms::{AcroForm, Field};
use pdfcer_core::object::ObjId;

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::panels::PanelsState;
use crate::text::forms as t;

use self::edit::FormEdit;
use self::rows::RowContext;

/// The ribbon command that opens this panel.
///
/// Named here as well as (eventually) on `crate::panels::Panel` so this
/// module's own reachability test can assert it without waiting on the enum
/// variant. See [`tests::the_forms_command_is_reachable_from_the_ribbon`] for
/// what that test is defending against, and why a panel with no route from the
/// ribbon is a defect three panels in the old shell actually shipped.
pub const COMMAND_ID: &str = "view.panel_forms";

/// Draw the Forms panel.
///
/// The one entry point. Shape and signature match every other panel body — see
/// [`crate::panels::Panel::show`] — so wiring it is `Self::Forms =>
/// forms::body(ui, doc, state, actions)` and nothing else.
///
/// `state` is unused: this panel's only inter-frame state is the set of text
/// drafts, and that lives in [`FormsUi`] rather than on
/// [`crate::panels::PanelsState`]. The reason is boundary rather than
/// preference — `PanelsState` is defined in `crate::panels`' own `mod.rs`,
/// which this work may not extend — and [`FormsUi`]'s own header sets out both
/// why the chosen home is *sound* and what the better home would be.
pub fn body(ui: &mut egui::Ui, doc: &OpenDoc, _state: &mut PanelsState, actions: &mut Vec<Action>) {
    // Read the SESSION, not the file on disk. An operator who has already
    // filled three fields must see those three values; `EditSession::view` is
    // the base revision with every unsaved edit applied, which is the same
    // thing the canvas rasterizes.
    // ★★ **Put the spotlight out before the rows draw, so that a focused row can
    // light it again this frame** — `OPERATOR_REQUESTS.md` O98.
    //
    // Clear-then-set rather than tracking a transition: with no focused row the
    // clear stands and the canvas draws nothing, and the whole "which row lost
    // focus, and was it this one" question never has to be asked. It also means
    // a panel that is not drawn at all — hidden, or a different tab — writes
    // nothing, so hiding the panel puts the spotlight out by construction.
    crate::panels::forms::spotlight::clear(ui.ctx());

    let view = doc.session.view();
    // ★★ NEITHER of these returns early any more, and the reason is the
    // same one the Bookmarks panel's empty-outline return was removed for on
    // the same day.
    //
    // A document with no `/AcroForm` can still carry `/Widget` annotations, and
    // **pdfcer makes exactly that**: `insert_pages` copies everything reachable
    // from a page, `/Annots` reaches the widgets, `/AcroForm` is a catalog
    // entry that is never in the copied set. Insert a form's pages into a CAD
    // drawing and you get boxes that draw like fields, swallow every keystroke,
    // and belong to nothing.
    //
    // The Tab-order section below is the one surface that lists those widgets
    // and offers to register them. Returning here put it **behind a guard that
    // the very state it exists for cannot pass** — the panel said "this
    // document has no form" and offered nothing, in the one document that most
    // needed the remedy.
    //
    // Found by a driven run, not by reading: the check that inserts a form's
    // pages and then registers one of the orphans got as far as opening this
    // panel and stopped. Both sentences are still shown, because both are true
    // and an operator opening the panel on an ordinary drawing deserves to be
    // told why it is empty. What has changed is that they are no longer the
    // last thing the panel does.
    let form = pdfcer_core::forms::parse_acroform(&view);
    let fillable = match &form {
        None => {
            ui.label(t::forms_no_acroform());
            None
        }
        Some(f) if f.fields.is_empty() => {
            ui.label(t::forms_empty_acroform());
            None
        }
        Some(f) => Some(f),
    };
    let Some(form) = fillable else {
        // No fields to fill, and possibly widgets to register. Everything
        // between here and the Tab-order section is about filling, so it is
        // skipped rather than drawn empty — R9: an unavailable capability
        // renders nothing.
        //
        // ★★ WRAPPED IN A SCROLL AREA, which the filling path does not need
        // and this path does.
        //
        // The dock gives a panel body a fixed rectangle and no scrolling of its
        // own — `egui-shell`'s dock says so in as many words: *"any `ScrollArea`
        // a panel body creates inherits this"*, meaning the body is expected to
        // create one. On the filling path the field list's own scroll area is
        // that mechanism and it takes the rest of the pane.
        //
        // This path has no field list, so nothing was scrolling and the
        // Tab-order section's content simply ran past the bottom of the pane.
        // A driven run measured the panel body at y=466..770 with the Register
        // buttons laid out at y=773..797 — **outside the panel on both axes**,
        // drawn, published, and unreachable at any pane size, because there was
        // nothing to scroll.
        //
        // Fourth instance today of one shape: a control that must be reachable
        // placed where the container cannot show it. The other three were fixed
        // by moving the control; this one by giving the container the mechanism
        // it was assumed to have.
        egui::ScrollArea::vertical()
            .id_salt("pdfcer-forms-no-fields")
            .show(ui, |ui| {
                ui.separator();
                tab_order::section(ui, doc, &view, form.as_ref(), actions);
            });
        return;
    };

    // Asked ONCE, before any control is drawn, and applied to every one: a
    // certification signature forbids filling the whole DOCUMENT, not one
    // field, so per-control re-asking would repeat a signature census per
    // field and still say the same thing (R83 — know before you offer).
    let fill_refusal: Option<&'static str> = doc
        .session
        .fill_refusal()
        .map(|_| t::form_field_certification_disabled_tooltip());
    // ★ FLATTEN ASKS A DIFFERENT GATE, and the difference is not academic.
    //
    // Filling takes core's `/P`-aware gate; flattening removes the form, which
    // is a STRUCTURAL change and takes the strict one. On the ordinary
    // real-world shape — a certified fillable form at `/P 2` — filling is
    // permitted and flattening is refused, so reusing `fill_refusal` here
    // would render an enabled Flatten button whose every press errors.
    //
    // ★ This asks `flatten_refusal`, and the borrowed answer it replaced is
    // worth recording because the correction went both ways.
    //
    // This originally asked `deletion_refusal`, because core exposed no
    // flatten query and the two routed through what looked like the identical
    // check. It was named as a borrowed answer and reported as a boundary
    // finding rather than left silent — and the report was **half wrong**.
    //
    // `flatten_refusal` was added (pdfcer `fa243df`). But the accompanying
    // claim that `deletion_refusal` under-reported was rejected, correctly:
    // it predicts DELETION and matches `deletion_preflight` exactly. The
    // comparison was against flatten, which is a different operation. Acting
    // on it would have disabled a Delete control that would have worked — an
    // over-reporting refusal query is a different bug, not a safe one, and
    // there is now a test in core whose job is to stop exactly that.
    //
    // The two gates really do differ, just not the way it was reported:
    // deletion and flatten share the strict certification gate, and flatten
    // additionally CREATES page content, so it carries a suppression guard
    // deletion does not. Two checks of three — which works until it does not,
    // on documents that are not exotic.
    let structural_refusal: Option<&'static str> = doc
        .session
        .flatten_refusal()
        .map(|_| t::forms_structural_certification_disabled_tooltip());

    header(ui, form, fill_refusal);
    // ★ Directly under the header, above every control: what the LAST edit
    // decided on the operator's behalf, and which fields the page cannot be
    // clicked for. Both are answers to "why did that not happen where I
    // expected?", and both belong before the thing they are about rather than
    // after it — the same placement rule the document-wide disclosures follow.
    fill_disclosure(ui, doc);
    canvas_routing(ui, doc, fill_refusal);

    // Collected while `form` is borrowed, converted to actions at the end —
    // the actions-not-mutations discipline, and the same shape
    // `crate::panels::layers` uses for its checkbox.
    let mut edits: Vec<FormEdit> = Vec::new();

    calculated_fields(ui, &view, fill_refusal, &mut edits);
    reset_section(ui, doc, fill_refusal, &mut edits);
    whole_form_controls(ui, form, fill_refusal, structural_refusal, &mut edits);
    ui.separator();
    // ★ THE SECOND LIST, and it answers a different question from the one
    // below it — see [`tab_order`]'s header.
    //
    // It is placed BETWEEN the whole-form controls and the fill list, and the
    // placement is argued rather than incidental. Below the fill list it would
    // be unreachable: the fill list's `ScrollArea` takes the rest of the pane,
    // so anything after it is laid out past the bottom of a container that does
    // not scroll. Above the whole-form controls it would push Redraw and
    // Flatten down, and "a control that acts on everything below it belongs
    // above it" is why those two sit where they do.
    //
    // It is handed the SAME `view` and the SAME parsed `form` this body is
    // already drawing from, rather than re-deriving either: two parses of one
    // form per frame is a cost with no benefit, and a second parse could in
    // principle disagree with the one the rows above came from.
    //
    // It takes `actions` directly rather than the `edits` vector, because the
    // only thing it can raise is `Action::GoToPage` — navigation, not a form
    // verb, and `FormEdit` has no variant that could carry it.
    tab_order::section(ui, doc, &view, Some(form), actions);
    // ★★ THE THIRD LIST, and it is placed here for the two constraints
    // [`groups`]' header sets out.
    //
    // It must be **above** the fill list, because that list's own `ScrollArea`
    // takes the rest of the pane and the panel's top level does not scroll — so
    // anything after it is laid out past the bottom of a container with no way
    // to reach it. This panel has already shipped that defect once, measured in
    // a driven run at y=773 in a body ending at y=770.
    //
    // It sits beside Tab order rather than beside the fill list because the two
    // are the panel's **structural** surfaces: that one lists controls the form
    // does not claim and offers to register them, this one lists the names the
    // form files fields under and offers to remove them. The fill list is about
    // the form's contents; these two are about its shape.
    //
    // It is handed the SAME parsed `form` the rows above came from rather than
    // re-deriving it: two parses of one form per frame is a cost with no
    // benefit, and a second parse could in principle disagree with the first.
    //
    // It takes `actions` directly rather than the `edits` vector, because what
    // it raises is a `FieldAction` — a form verb with its own two-press
    // protocol — and `FormEdit` has no variant that could carry it.
    groups::section(ui, doc, form, actions);
    ui.separator();
    field_list(ui, doc, form, fill_refusal, &mut edits, actions);

    for e in edits {
        raise(actions, e);
    }
}

/// The count line and every document-wide disclosure, in the order they are
/// read.
///
/// **Above every control, without exception.** Each of these describes a
/// condition under which what the operator sees here and what a different
/// viewer shows can legitimately disagree, and a caveat below a list arrives
/// after the operator has already drawn a conclusion.
///
/// The order is by how much it changes what the operator should do:
/// the refusal first (nothing below it will work), then the two rendering
/// divergences, then the scripts, then the malformed entries.
fn header(ui: &mut egui::Ui, form: &AcroForm, fill_refusal: Option<&'static str>) {
    // The count this panel will actually offer — see this module's header, and
    // `text::forms::forms_field_count`'s, for why `Field::is_fillable` is the
    // wrong number to put here.
    let fillable = form.fields.iter().filter(|f| offers_a_control(f)).count();
    let fillable = if fill_refusal.is_some() { 0 } else { fillable };
    ui.label(t::forms_field_count(form.fields.len(), fillable));
    if fillable == 0 {
        ui.label(
            egui::RichText::new(t::forms_no_fillable_fields())
                .small()
                .weak(),
        );
    }

    if fill_refusal.is_some() {
        ui.colored_label(ui.visuals().warn_fg_color, t::forms_certification_note());
    }
    if form.need_appearances {
        ui.colored_label(ui.visuals().warn_fg_color, t::forms_need_appearances_note());
    }
    if form.xfa.is_present() {
        ui.colored_label(ui.visuals().warn_fg_color, t::forms_xfa_note());
    }
    let scripted = form
        .fields
        .iter()
        .filter(|f| f.has_additional_actions)
        .count();
    if scripted > 0 {
        ui.colored_label(
            ui.visuals().warn_fg_color,
            t::forms_javascript_note(scripted),
        );
    }
    if form.inline_field_roots > 0 {
        ui.colored_label(
            ui.visuals().warn_fg_color,
            t::forms_inline_field_roots_note(form.inline_field_roots),
        );
    }
}

/// **The off-canvas home for the two things a fill decides and the document
/// cannot afterwards be asked.**
///
/// Rule 4 in one clause: *"disclosure lives off-canvas — a status line, a
/// results panel, a report after the command, a properties field."* This is
/// the panel half of that, and it is the half this build can reach: a status
/// line would be better still for a fill made by clicking the **page**, when
/// this panel may not even be open, and wiring one is a change to
/// `crate::app::status` that this work does not own. Named rather than
/// silently absent — see [`edit::last_fill_disclosure`].
///
/// # Why the panel used to discard these, and why that argument does not
/// survive
///
/// [`edit`]'s header carries the original reasoning — *"every fact those notes
/// carried is derivable from the document the panel re-reads on the next
/// frame"* — and it is correct for six of `FillOutcome`'s eight facts. It is
/// **false** for the two below, and the falsity is exactly the point: an
/// auto-size pdfcer chose and a character pdfcer replaced both look, in the saved
/// file, precisely like an author's decision. Re-reading the field cannot
/// distinguish them, so there is nothing to derive and the note has to be
/// carried.
///
/// Shown only while it describes the revision on screen, which is what the
/// epoch comparison inside [`edit::last_fill_disclosure`] is for: an undo moves
/// the epoch past the disclosure and the sentence stops being drawn, with
/// nothing anywhere that has to remember to clear it.
fn fill_disclosure(ui: &mut egui::Ui, doc: &OpenDoc) {
    let Some(disclosure) = edit::last_fill_disclosure(doc.edit_epoch) else {
        return;
    };
    // Unencodable characters FIRST, because it is the more serious of the two:
    // an auto-size changes how the value looks, this changes what the value
    // *is*. Same ordering rule the recompute section uses when it lists skips
    // above changes.
    if disclosure.unencodable_chars > 0 {
        ui.colored_label(
            ui.visuals().warn_fg_color,
            t::forms_fill_unencodable_note(&disclosure.field, disclosure.unencodable_chars),
        );
    }
    if let Some(size) = disclosure.applied_autosize {
        ui.colored_label(
            ui.visuals().warn_fg_color,
            t::forms_fill_autosize_note(&disclosure.field, size),
        );
    }
}

/// **Where the fields that cannot be clicked on the page went.**
///
/// `crate::canvas::forms` lets the operator fill a form by clicking it. Five
/// reasons a field is not offered there are listed in that module's header, and
/// every one of them ends in *"fill it in the panel"* — so the panel is where
/// they have to be said, or the capability has silently shrunk from the
/// operator's point of view and they have no way to find out why.
///
/// # ★ The counts come from the canvas's own walk, not from a second one
///
/// [`crate::canvas::forms::placed`] is **the** classification of this form —
/// the one whose boxes the canvas hit-tests — and it hands back
/// [`crate::canvas::forms::boxes::Routing`] from the same pass. This panel reads that, and
/// deliberately does not repeat the rule.
///
/// The first draft did repeat it, and repeating it was wrong twice over. Once
/// for the ordinary reason: two statements of one rule drift, and a panel
/// promising "3 fields can only be filled here" over a canvas that declined
/// four is worse than no count. And once for a reason no review would have
/// caught — the re-derivation asked each widget's `/P` to work out which page
/// it was on, which is the silent defect
/// [`crate::canvas::forms::boxes::place`]'s ★ section describes and which **no test against
/// the fixture corpus can reach**.
///
/// It is also free: the walk is cached per `(document, revision)` and the
/// canvas has already paid for it this frame.
///
/// Conditional, so it is a signal rather than furniture: a form every one of
/// whose fields can be clicked says nothing at all.
fn canvas_routing(ui: &mut egui::Ui, doc: &OpenDoc, fill_refusal: Option<&'static str>) {
    if fill_refusal.is_some() {
        // Nothing can be filled anywhere, which the header has already said in
        // stronger words. A second sentence about *where* would be noise on
        // top of a refusal.
        return;
    }
    let routing = crate::canvas::forms::placed(ui.ctx(), doc).routing;

    if routing.undrawn > 0 {
        ui.label(
            egui::RichText::new(t::forms_canvas_undrawn_note(routing.undrawn))
                .small()
                .weak(),
        );
    }
    if routing.unreachable > 0 {
        ui.label(
            egui::RichText::new(t::forms_canvas_unreachable_note(routing.unreachable))
                .small()
                .weak(),
        );
    }
}

/// Whether this panel will draw a live control for `field`.
///
/// **The predicate behind the count line**, and it is deliberately expressed
/// as "will a control be drawn?" rather than as a copy of the row dispatch,
/// because the two would drift. It is exactly the negation of
/// [`rows::block_reason`] plus the rich-text case, which is offered a
/// *conversion* rather than a box and so is not somewhere the operator can
/// type today.
///
/// [`tests::the_fillable_count_agrees_with_what_the_rows_offer`] pins it
/// against `block_reason` itself so the two cannot come apart.
fn offers_a_control(field: &Field) -> bool {
    rows::block_reason(field).is_none() && !field.is_rich_text()
}

/// The Calculated Fields section — decision 009 posture B.
///
/// Above the field list and below the document-wide disclosures, because a
/// recompute acts on the whole form and because its result changes what the
/// rows below it show. An operator who scrolled past this and then read a
/// stale total would have been misled by the layout.
///
/// **Collapsed by default and never auto-run.** Merely opening a form must not
/// change a computed `/V`. The plan is computed on every frame the section is
/// open — cheap on any real form, and always current with the fills the
/// operator just made, which a cached plan would not be.
fn calculated_fields(
    ui: &mut egui::Ui,
    view: &pdfcer_core::view::DocumentView<'_>,
    fill_refusal: Option<&'static str>,
    edits: &mut Vec<FormEdit>,
) {
    egui::CollapsingHeader::new(t::recompute_heading())
        .id_salt("pdfcer-forms-recompute")
        .default_open(false)
        .show(ui, |ui| {
            ui.label(t::recompute_explainer());
            let plan = pdfcer_core::form_script::recompute::plan(
                view,
                pdfcer_core::form_script::calc::CommaPolicy::default(),
            );

            if plan.not_reproducible > 0 {
                ui.label(t::recompute_not_considered(plan.not_reproducible));
            }
            // ★ A rule-4 disclosure: pdfcer INFERRED an evaluation order the
            // document was required to state, the inference decides the
            // numbers below, and another reader may compute different ones.
            if plan.order_source.is_pdfcer_choice() {
                ui.colored_label(
                    ui.visuals().warn_fg_color,
                    t::recompute_order_is_a_guess(plan.unlisted_calculations),
                );
            }
            // Skips are listed BEFORE the changes, not after. A field pdfcer
            // declined to compute is the thing an operator most needs to
            // notice, and a list of successful changes above it reads as
            // completeness.
            //
            // `AlreadyCorrect` is filtered out because it is not a skip in the
            // sense the operator cares about — it is a field pdfcer checked and
            // found right, which the summary line below already covers.
            for skipped in &plan.skipped {
                if skipped.reason == pdfcer_core::form_script::recompute::Skip::AlreadyCorrect {
                    continue;
                }
                ui.colored_label(
                    ui.visuals().warn_fg_color,
                    t::recompute_skip_row(&skipped.field, &skipped.reason.to_string()),
                );
            }

            if plan.is_empty() {
                ui.label(if plan.skipped.is_empty() {
                    t::recompute_nothing_recognised()
                } else {
                    t::recompute_up_to_date()
                });
                return;
            }

            ui.label(t::recompute_pending(
                plan.changes.len(),
                plan.coerced_operands(),
            ));
            // ★ EVERY PROPOSED VALUE IS ON SCREEN BEFORE THE BUTTON THAT
            // COMMITS IT. Rule 4's disclosure obligation, satisfied by the
            // values being visible and the commit being a deliberate click on
            // a control at a fixed position — not by a confirm box anchored to
            // the page, which decision 024 §4.4 forbids by name.
            for change in &plan.changes {
                ui.label(t::recompute_change_row(
                    &change.field,
                    &change.previous,
                    &change.proposed,
                ))
                .on_hover_text(change.disclosure.message());
            }

            let button = ui.add_enabled(
                fill_refusal.is_none(),
                egui::Button::new(t::recompute_apply_button()),
            );
            let button = match fill_refusal {
                Some(note) => button.on_disabled_hover_text(note),
                None => button.on_hover_text(t::recompute_apply_tooltip()),
            };
            if button.clicked() {
                edits.push(FormEdit::Recompute {
                    changes: plan
                        .changes
                        .iter()
                        .map(|c| (c.field.clone(), c.proposed.clone()))
                        .collect(),
                });
            }
        });
}

/// The Reset-to-defaults section (§12.7.5.3).
///
/// Beside the recompute section and collapsed for the same reason, but the
/// disclosure is doing more work here: a recompute writes a number the
/// operator can check, a reset **destroys what they typed**. So the section
/// lists every field it would clear, with its current value, before offering
/// the button — the loss is what has to be on screen, not the outcome.
///
/// # ★ The preview comes from core, and is filtered here
///
/// `EditSession::reset_preview` returns a row for **every** field in scope,
/// including ones that are ineligible and ones that already hold their reset
/// value; core's own doc calls filtering the shell's job. That is the third
/// bite again in miniature — `preview.len()` is not the number to display, and
/// the number that matters is the count of rows with `would_change` set, which
/// core pins as equal to `ResetOutcome::fields_reset`.
///
/// Re-deriving the preview here instead would be a second reset algebra beside
/// the engine's, and the two would disagree the first time `/DV` inheritance
/// changed.
fn reset_section(
    ui: &mut egui::Ui,
    doc: &OpenDoc,
    fill_refusal: Option<&'static str>,
    edits: &mut Vec<FormEdit>,
) {
    egui::CollapsingHeader::new(t::reset_heading())
        .id_salt("pdfcer-forms-reset")
        .default_open(false)
        .show(ui, |ui| {
            ui.colored_label(ui.visuals().warn_fg_color, t::reset_explainer());

            let preview = doc.session.reset_preview(None);
            let mut clearing = 0usize;
            let mut ineligible = 0usize;
            let mut already = 0usize;
            for row in &preview {
                if row.ineligible.is_some() {
                    ineligible += 1;
                    continue;
                }
                if !row.would_change {
                    already += 1;
                    continue;
                }
                clearing += 1;
                // `would_remove` is carried separately from an empty `target`
                // because an absent `/V` and a `/V` set to the empty string are
                // different bytes, and a panel that showed both as `""` would
                // be describing the wrong edit.
                let to = if row.would_remove {
                    t::reset_to_empty().to_owned()
                } else {
                    row.target.clone()
                };
                ui.label(t::reset_row(&row.field, &row.current, &to));
            }
            if already > 0 {
                ui.label(t::reset_already_default(already));
            }
            if clearing == 0 {
                ui.label(t::reset_nothing_to_do());
                return;
            }
            ui.label(t::reset_pending(clearing, ineligible));

            let button =
                ui.add_enabled(fill_refusal.is_none(), egui::Button::new(t::reset_button()));
            let button = match fill_refusal {
                Some(note) => button.on_disabled_hover_text(note),
                None => button.on_hover_text(t::reset_tooltip()),
            };
            if button.clicked() {
                edits.push(FormEdit::Reset);
            }
        });
}

/// The two controls that act on the whole form.
///
/// **Placed above the list, not below it**, because they act on everything
/// beneath them: a control that acts on everything below it belongs above it,
/// and a Flatten button under a forty-row list is a button an operator scrolls
/// past without meeting.
///
/// # ★ Redraw comes first, and the order is load-bearing
///
/// Flatten works by invoking each widget's **existing** `/AP` as a page
/// XObject. A field with no drawn appearance has nothing to invoke, so
/// flattening burns nothing for it and then removes the field — the typed
/// value disappears from the visible page. Core's own guidance is to
/// regenerate first, and this panel both orders the buttons that way and says
/// so, conditionally, when the document actually has fields at risk.
fn whole_form_controls(
    ui: &mut egui::Ui,
    form: &AcroForm,
    fill_refusal: Option<&'static str>,
    structural_refusal: Option<&'static str>,
    edits: &mut Vec<FormEdit>,
) {
    ui.horizontal(|ui| {
        let redraw = ui.add_enabled(
            fill_refusal.is_none(),
            egui::Button::new(t::forms_regenerate_button()),
        );
        let redraw = match fill_refusal {
            Some(note) => redraw.on_disabled_hover_text(note),
            None => redraw.on_hover_text(t::forms_regenerate_tooltip()),
        };
        if redraw.clicked() {
            edits.push(FormEdit::RegenerateAppearances);
        }

        // Delete-shaped weight: a rich, honest tooltip and one undo step — NOT
        // redaction's blocking modal. Argued in `text::forms`'
        // `forms_flatten_tooltip` against what each operation actually does:
        // flatten APPENDS an overlay stream and leaves existing content
        // byte-verbatim, so under the default incremental save the prior
        // revision still holds the values. Its irreversibility is conditional
        // on the save mode, not structural.
        let flatten = ui.add_enabled(
            structural_refusal.is_none(),
            egui::Button::new(t::forms_flatten_button()),
        );
        let flatten = match structural_refusal {
            Some(note) => flatten.on_disabled_hover_text(note),
            None => flatten.on_hover_text(t::forms_flatten_tooltip()),
        };
        if flatten.clicked() {
            edits.push(FormEdit::Flatten);
        }
    });

    // Conditional, so it is a signal and not noise: a form whose every field
    // is drawn says nothing about redrawing.
    let undrawn = form.fields.iter().filter(|f| !f.has_appearance()).count();
    if undrawn > 0 {
        ui.colored_label(
            ui.visuals().warn_fg_color,
            t::forms_flatten_needs_redraw_note(undrawn),
        );
    }
}

/// The scrolling list of field rows.
///
/// The scroll area wraps **only** the rows. Everything above it — the
/// disclosures, the two review sections, the whole-form controls — stays put,
/// because a disclosure that scrolls out of sight while the operator works
/// through a long form has stopped disclosing.
fn field_list(
    ui: &mut egui::Ui,
    doc: &OpenDoc,
    form: &AcroForm,
    fill_refusal: Option<&'static str>,
    edits: &mut Vec<FormEdit>,
    actions: &mut Vec<Action>,
) {
    // A page-object-id -> 1-based page number map, so a row can say WHICH page
    // its field is on. Built once per frame rather than per row: a 400-field
    // form would otherwise do 400 linear scans of the page list.
    let page_numbers: HashMap<ObjId, usize> = doc
        .pages
        .iter()
        .enumerate()
        .map(|(i, p)| (p.id, i + 1))
        .collect();
    let mut ui_state = FormsUi::load(ui, doc);
    ui_state.prune(form);

    // ★ The draft is moved OUT of `ui_state` for the duration of the rows and
    // put back afterwards, because `RowContext` borrows it mutably and
    // `ui_state.drafts` is borrowed mutably at the same time. Two mutable
    // borrows of one struct is the ordinary Rust shape here and the ordinary
    // answer is to split them; taking and replacing keeps the state in one
    // place, which is what makes `FormsUi::prune` able to reason about it.
    let mut button_draft = ui_state.button_draft.take();
    let mut ctx = RowContext {
        page_numbers: &page_numbers,
        fill_refusal,
        doc: Some(doc),
        // ★★★ **What the operator is typing ON THE PAGE, asked once for the
        // whole frame** — the 2026-09 review's row A12c.
        //
        // Beside `page_numbers` and `fill_refusal` because it is the same kind
        // of thing: a fact about the document that is identical for every row,
        // and one a 400-field form must not ask 400 times. `canvas::forms`
        // holds one focus, so the answer is one field or none.
        //
        // ★ Asked from the PANEL rather than pushed by the canvas, which is
        // the direction `canvas::forms::placed` already established between
        // these two modules: the surface that owns the answer publishes it,
        // and the surface that needs it reads it. The reverse — the canvas
        // writing into `FormsUi` — would put a second writer on state whose
        // whole correctness argument is its `(path, epoch)` key.
        live_canvas_draft: crate::canvas::forms::live_draft(ui.ctx(), doc),
        button_draft: &mut button_draft,
        actions,
    };

    egui::ScrollArea::vertical()
        .id_salt("pdfcer-forms-rows")
        .show(ui, |ui| {
            for (index, field) in form.fields.iter().enumerate() {
                rows::row(ui, field, index, &mut ctx, &mut ui_state.drafts, edits);
                ui.separator();
            }
        });
    ui_state.button_draft = button_draft;

    ui_state.store(ui);
}

/// Turn one [`FormEdit`] into the action that carries it across the funnel.
///
/// **This is the single line of wiring this module is waiting on**, and it is
/// isolated into a function of its own so that the change is one edit in one
/// place rather than nine call sites.
///
/// # What `crate::app::actions` needs
///
/// One variant:
///
/// ```text
/// /// One form-filling verb — see `crate::panels::forms::edit`.
/// Form(crate::panels::forms::edit::FormEdit),
/// ```
///
/// and one arm in `PdfcerApp::apply`:
///
/// ```text
/// FieldAction::Edit(edit) => crate::panels::forms::edit::apply(doc, &edit),
/// ```
///
/// That is the whole of it. The mutation protocol, the epoch bump, the texture
/// invalidation and the refusal trace all live in [`edit::apply`], for the
/// reasons that module's header sets out — chiefly that the six form outcome
/// types do not unify into `vector_edit`'s `Result<Vec<String>, EditError>`.
fn raise(actions: &mut Vec<Action>, edit: FormEdit) {
    actions.push(FieldAction::Edit(edit).into());
}

/// The Forms panel's own inter-frame state: one text draft per field.
///
/// # ★ Why this is not on `crate::panels::PanelsState`
///
/// It should be, and the constraint is a boundary rather than a design
/// judgement: `PanelsState` is defined in `crate::panels`' own `mod.rs`, which
/// this work may add exactly one `pub mod forms;` line to. The **preferred**
/// shape, for whoever lifts that constraint, is a `forms: FormsUi` field
/// beside `tree: ObjectTreeUi`, dropped by `PanelsState::forget_document`
/// exactly as everything else there is.
///
/// # Why egui's memory is nonetheless a sound home, and not a smuggled mutation
///
/// The actions-not-mutations invariant is about **the document**. This is not
/// document state and it is not derived from the document: it is what the
/// operator has typed and not yet committed, which is the same category as the
/// caret position `TextEdit` already keeps in exactly this store. Nothing here
/// can change a pixel of the page; only [`FormEdit`] can, and only through the
/// funnel.
///
/// # ★ The key is `(path, edit_epoch)`, which is what makes UNDO correct
///
/// This is [`crate::panels::PanelsState::sync`]'s discipline applied to a
/// different kind of state, and the epoch half is the interesting one.
///
/// Without it: the operator types "Anna", tabs away (committed), presses
/// Ctrl+Z. The document reverts to empty and the draft still says "Anna", so
/// the panel shows a filled box over an empty field — it disagrees with the
/// document it is describing, and the next thing the operator does re-commits
/// the value they just undid.
///
/// With it, every draft is dropped the moment anything about the document
/// changes and re-seeded from the stored value on the next frame. **Nothing is
/// lost by that**, and the argument is worth writing down because it looks
/// lossy: a draft that differs from the stored value belongs to a field that
/// still has focus, and every gesture that can bump the epoch — clicking
/// another field, a check box, a button — takes focus away first, which
/// commits that field in the same frame. So by the time the epoch moves, every
/// other draft already equals what the document holds.
///
/// The path half handles the plainer case: a different document makes every
/// field name here meaningless.
///
/// # Cost
///
/// One clone of the map per frame, in and out of the store. A few hundred
/// short strings, against a panel that is already laying out a few hundred
/// egui widgets. Measure before trading it for an `Arc<Mutex<_>>`.
#[derive(Clone, Default)]
pub struct FormsUi {
    /// The `(document path, edit epoch)` [`Self::drafts`] describes.
    key: Option<(PathBuf, u64)>,
    /// What the operator has typed into each text field, by fully-qualified
    /// name, not yet written to the session.
    ///
    /// Keyed by NAME rather than by row index because several terminal fields
    /// may share a fully-qualified name and a fill applies to all of them — so
    /// they share one draft, which is correct, and a positional key would give
    /// them two that could disagree.
    drafts: BTreeMap<String, String>,
    /// **Which push button's action chooser is open, and what it is set to.**
    ///
    /// ★ Held here rather than in `egui`'s temp data for the reason every other
    /// field of this struct is: it is keyed to a `(document, epoch)` pair and
    /// pruned with the form. A chooser left open over a field that an edit has
    /// removed would be a control editing something that is not there.
    ///
    /// One at a time, by construction. Two open choosers would let an operator
    /// set one and lose the other without a word.
    button_draft: Option<(String, crate::canvas::formfield::action::ButtonDoes)>,
}

impl FormsUi {
    /// The egui id this state is stored under.
    ///
    /// One id for the whole panel rather than one per field: the drafts are a
    /// single coherent unit keyed on one document revision, and splitting them
    /// would mean the key had to be checked per field.
    fn id() -> egui::Id {
        egui::Id::new("pdfcer-forms-ui")
    }

    /// Read this frame's state, dropping it if it describes a different
    /// document or a different revision.
    fn load(ui: &egui::Ui, doc: &OpenDoc) -> Self {
        let key = (doc.path.clone(), doc.edit_epoch);
        let mut state: Self = ui
            .data(|d| d.get_temp::<Self>(Self::id()))
            .unwrap_or_default();
        if state.key.as_ref() != Some(&key) {
            state = Self {
                button_draft: None,
                key: Some(key),
                drafts: BTreeMap::new(),
            };
        }
        state
    }

    /// Write this frame's state back.
    fn store(self, ui: &egui::Ui) {
        ui.data_mut(|d| d.insert_temp(Self::id(), self));
    }

    /// Drop drafts for names this form no longer has.
    ///
    /// The epoch key already catches an edit, so this exists for the case the
    /// key cannot see: a field that was never in this form to begin with,
    /// which is reachable when the same path is reopened after being changed
    /// elsewhere. Cheap, and it stops the map growing without bound across a
    /// long session.
    fn prune(&mut self, form: &AcroForm) {
        let names: BTreeSet<&str> = form
            .fields
            .iter()
            .map(|f| f.fully_qualified_name.as_str())
            .collect();
        self.drafts.retain(|k, _| names.contains(k.as_str()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shell::{commands, manifest};
    use egui_shell::CommandRegistry;
    use std::collections::BTreeSet;

    /// **★ This panel is reachable from the ribbon.**
    ///
    /// The check three panels in the old shell shipped without. Its
    /// `panels_structure.rs` header records what that cost:
    ///
    /// > All three shipped with a `PaneSubject`, a panel body, a rail entry
    /// > and a diagnostic step — and no control an operator could click.
    /// > Their only callers were the harness step handlers, so every
    /// > verification passed while the panels were unreachable in a real
    /// > build.
    ///
    /// Two assertions, and both are needed. A command **the manifest
    /// references** is one the ribbon draws a control for; a command **the
    /// registry holds** is one that has a label, a tooltip and an enable
    /// predicate. Either alone is half a control.
    ///
    /// Written here rather than left to
    /// `crate::panels::tests::every_panel_is_reachable_from_the_ribbon`
    /// because that sweep iterates `Panel::ALL`, and this panel is not on that
    /// enum yet — the enum lives in a file this work may not extend. When the
    /// variant lands, the sweep covers this too and this test becomes the
    /// belt to its braces; it is deliberately the *same* two assertions so
    /// that is a clean duplication rather than a divergent one.
    #[test]
    fn the_forms_command_is_reachable_from_the_ribbon() {
        let shell = manifest::built_in();
        let mut registry = CommandRegistry::new();
        commands::register(&mut registry);
        let referenced: BTreeSet<String> = shell
            .command_references()
            .into_iter()
            .map(|(_, id)| id)
            .collect();

        assert!(
            referenced.contains(COMMAND_ID),
            "the Forms panel names `{COMMAND_ID}`, and no tab, QAT slot or key \
             binding references it. An operator cannot open this panel."
        );
        assert!(
            registry.get(COMMAND_ID).is_some(),
            "the Forms panel names `{COMMAND_ID}`, which is not registered — so \
             the ribbon has an id with no label, no tooltip and no enable \
             predicate, and draws nothing for it."
        );
    }

    /// **★ The "you can fill here" count agrees with what the rows draw.**
    ///
    /// The whole of the third bite, pinned. [`offers_a_control`] and
    /// [`rows::row`]'s dispatch are two statements of one rule, and the
    /// failure when they drift is silent: a count line promising twelve
    /// fillable fields above twelve disabled boxes.
    ///
    /// Asserted against `block_reason` — the function the row actually calls —
    /// rather than against a re-derivation, so the test cannot pass by
    /// agreeing with a third copy of the rule.
    #[test]
    fn the_fillable_count_agrees_with_what_the_rows_offer() {
        // `Quadding` is re-exported through `forms` only as a private `use`;
        // its home is `vartext`, which is where a caller must name it.
        use pdfcer_core::forms::{ButtonKind, FieldFlags, FieldType, FieldValue};
        use pdfcer_core::vartext::Quadding;

        // A minimal terminal field, built by hand: no fixture in the engine's
        // corpus carries all five refusal shapes at once, and the point here
        // is the PREDICATE rather than any one document.
        let base = Field {
            id: pdfcer_core::object::ObjId::new(1, 0),
            fully_qualified_name: "F".to_owned(),
            partial_name: None,
            alternate_name: None,
            mapping_name: None,
            rich_value: None,
            default_style: None,
            field_type: Some(FieldType::Text),
            button_kind: None,
            flags: FieldFlags(0),
            value: FieldValue::Absent,
            default_value: FieldValue::Absent,
            default_appearance: None,
            quadding: Quadding::Left,
            max_len: None,
            options: Vec::new(),
            top_index: 0,
            selected_indices: Vec::new(),
            widgets: Vec::new(),
            merged: false,
            has_additional_actions: false,
            shares_parent_name: false,
            parent: None,
        };

        // An ordinary text field: counted, and offered a box.
        assert!(offers_a_control(&base));

        // Read-only, signature, push button: each blocked, each uncounted.
        for blocked in [
            Field {
                flags: FieldFlags(FieldFlags::READ_ONLY),
                ..base.clone()
            },
            Field {
                field_type: Some(FieldType::Signature),
                ..base.clone()
            },
            Field {
                field_type: Some(FieldType::Button),
                button_kind: Some(ButtonKind::Push),
                ..base.clone()
            },
        ] {
            assert!(
                rows::block_reason(&blocked).is_some(),
                "this field must be blocked for the assertion below to mean \
                 anything"
            );
            assert!(
                !offers_a_control(&blocked),
                "a blocked field was counted as one the operator can fill"
            );
        }

        // ★ Rich text is the case `block_reason` deliberately does NOT cover:
        //   the row offers a CONVERSION, not a box, so it must not be counted
        //   as somewhere the operator can type.
        let rich = Field {
            flags: FieldFlags(FieldFlags::RICH_TEXT),
            ..base.clone()
        };
        assert!(rich.is_rich_text(), "the fixture must be rich text");
        assert!(
            rows::block_reason(&rich).is_none(),
            "rich text must not be a blanket refusal — the row offers a \
             disclosed conversion"
        );
        assert!(
            !offers_a_control(&rich),
            "a rich-text field was counted as one the operator can type into"
        );

        // ★ And the bit-26 overload: a radio group with RadiosInUnison set
        //   carries the SAME bit as RichText. If the count asked the flag
        //   directly it would drop every such group out of the fillable total.
        let unison = Field {
            field_type: Some(FieldType::Button),
            button_kind: Some(ButtonKind::Radio),
            flags: FieldFlags(FieldFlags::RADIOS_IN_UNISON),
            ..base
        };
        assert!(
            !unison.is_rich_text(),
            "bit 26 on a /Btn field is RadiosInUnison, not RichText"
        );
        assert!(
            offers_a_control(&unison),
            "a radio group in unison must still be offered a control"
        );
    }

    /// **★ An edit forgets the drafts, which is what makes undo correct.**
    ///
    /// The defect this prevents, in full: the operator types "Anna", tabs away
    /// so it commits, then presses Ctrl+Z. The document reverts to empty. If
    /// the draft survived, the panel would show "Anna" in a box over a field
    /// holding nothing — disagreeing with the document it is describing, and
    /// arming a re-commit of the value that was just undone.
    ///
    /// Exercised through the real key, without an egui context: [`FormsUi`]'s
    /// key comparison is the whole mechanism, and it is a pure comparison.
    #[test]
    fn a_revision_change_forgets_every_draft() {
        let path = PathBuf::from("form.pdf");
        let mut state = FormsUi {
            button_draft: None,
            key: Some((path.clone(), 3)),
            drafts: BTreeMap::from([("Name".to_owned(), "Anna".to_owned())]),
        };

        // Same document, same revision: the draft survives, or typing would be
        // impossible.
        assert_eq!(state.key, Some((path.clone(), 3)));
        assert!(state.drafts.contains_key("Name"));

        // An edit — a fill, a toggle, an undo, a redo — moves the epoch.
        let stale = state.key.as_ref() != Some(&(path.clone(), 4));
        assert!(stale, "an epoch change must invalidate the drafts");

        // A different document, same epoch: also stale. The path half matters
        // on its own because epochs restart at zero for each open.
        let other = state.key.as_ref() != Some(&(PathBuf::from("other.pdf"), 3));
        assert!(other, "a different document must invalidate the drafts");

        // And the reset really empties it, rather than merely re-keying.
        state = FormsUi {
            button_draft: None,
            key: Some((path, 4)),
            drafts: BTreeMap::new(),
        };
        assert!(state.drafts.is_empty());
    }

    /// **Pruning drops a draft whose field no longer exists.**
    ///
    /// The case the epoch key cannot see — the same path reopened after being
    /// changed elsewhere — and the thing that stops the map growing without
    /// bound across a long session.
    #[test]
    fn a_draft_for_a_departed_field_is_dropped() {
        let form = AcroForm {
            fields: Vec::new(),
            groups: Vec::new(),
            need_appearances: false,
            sig_flags: 0,
            signatures_exist: false,
            append_only: false,
            calc_order_count: 0,
            calc_order: Vec::new(),
            has_default_resources: false,
            default_appearance: None,
            quadding: pdfcer_core::vartext::Quadding::Left,
            xfa: pdfcer_core::forms::XfaPresence::None,
            inline_field_roots: 0,
        };
        let mut state = FormsUi {
            button_draft: None,
            key: None,
            drafts: BTreeMap::from([("Gone".to_owned(), "typed".to_owned())]),
        };
        state.prune(&form);
        assert!(
            state.drafts.is_empty(),
            "a draft outlived the field it belongs to"
        );
    }
}
