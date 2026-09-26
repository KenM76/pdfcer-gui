//! # `panels::forms` — filling this document's interactive form
//!
//! ## What this panel is for
//!
//! The operator's goal for this build is *"replace Acrobat Reader first"*, and
//! **a reader fills forms; it does not create fields.** So this panel offers
//! text fields, check boxes, radio groups, choice lists, a reviewed reset, a
//! reviewed native recompute of script-driven fields, an appearance redraw and
//! a flatten. Nothing here adds, renames or moves a field.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/forms/mod.md`.

/// The verbs this panel can ask for, and the one place they are applied.
pub mod edit;
/// The names this form files its fields under, and the shell's only route to
/// deleting one. See that module's header for why a grouping node is reachable
/// from nowhere else in the shell, and for the two-press protocol its
/// invisibility forces.
mod groups;
/// One field, one row — the per-field controls.
pub mod rows;
/// **The panel→canvas channel** — which field the panel is pointing at, so the
/// canvas can spotlight it (`OPERATOR_REQUESTS.md` O98). A permitted affordance
/// under rule 4's fourth clause; the module is that channel and nothing more.
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
/// Named here as well as on `crate::panels::Panel` so this module's own
/// reachability test can assert it without going through the enum. See
/// [`tests::the_forms_command_is_reachable_from_the_ribbon`] for what that test
/// defends against: a panel with a body, a rail entry and no control an operator
/// can click is a panel that passes every harness step and ships unreachable.
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
    // **Put the spotlight out before the rows draw, so that a focused row can
    // light it again this frame** — `OPERATOR_REQUESTS.md` O98.
    //
    // Clear-then-set rather than tracking a transition: with no focused row the
    // clear stands and the canvas draws nothing, and the whole "which row lost
    // focus, and was it this one" question never has to be asked. It also means
    // a panel that is not drawn at all — hidden, or a different tab — writes
    // nothing, so hiding the panel puts the spotlight out by construction.
    crate::panels::forms::spotlight::clear(ui.ctx());

    let view = doc.session.view();
    // **NEITHER of these may return early**, however empty the form looks.
    //
    // A document with no `/AcroForm` can still carry `/Widget` annotations, and
    // **pdfcer makes exactly that**: `insert_pages` copies everything reachable
    // from a page, `/Annots` reaches the widgets, `/AcroForm` is a catalog
    // entry that is never in the copied set. Insert a form's pages into a CAD
    // drawing and you get boxes that draw like fields, swallow every keystroke,
    // and belong to nothing.
    //
    // The Tab-order section below is the one surface that lists those widgets
    // and offers to register them. A return here puts it **behind a guard that
    // the very state it exists for cannot pass** — the panel says "this document
    // has no form" and offers nothing, on the one document that most needs the
    // remedy.
    //
    // Both sentences are still shown, because both are true and an operator
    // opening the panel on an ordinary drawing deserves to be told why it is
    // empty. They are simply not the last thing the panel does.
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
        // WRAPPED IN A SCROLL AREA, which the filling path does not need and
        // this path does.
        //
        // The dock gives a panel body a fixed rectangle and no scrolling of its
        // own; the body is expected to create its own `ScrollArea`. On the
        // filling path the field list's scroll area is that mechanism and it
        // takes the rest of the pane. This path has no field list, so without
        // one here the Tab-order section's content is laid out past the bottom
        // of the pane — drawn, published, and unreachable at any pane size,
        // because nothing scrolls.
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
    // FLATTEN ASKS A DIFFERENT GATE, and the difference is not academic.
    //
    // Filling takes core's `/P`-aware gate; flattening removes the form, which
    // is a STRUCTURAL change and takes the strict one. On the ordinary
    // real-world shape — a certified fillable form at `/P 2` — filling is
    // permitted and flattening is refused, so reusing `fill_refusal` here
    // would render an enabled Flatten button whose every press errors.
    //
    // It must ask `EditSession::flatten_refusal` specifically, and not borrow
    // `deletion_refusal`. The two are close enough to look interchangeable —
    // both take the strict certification gate — but flatten additionally
    // CREATES page content, so it carries a suppression guard deletion does
    // not. Two checks of three, which works until it does not, on documents
    // that are not exotic. The reverse substitution is worse rather than safer:
    // `deletion_refusal` matches `deletion_preflight` exactly, so using it here
    // would over-report and disable a Delete control that would have worked.
    let structural_refusal: Option<&'static str> = doc
        .session
        .flatten_refusal()
        .map(|_| t::forms_structural_certification_disabled_tooltip());

    header(ui, form, fill_refusal);
    // Directly under the header, above every control: what the LAST edit
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
    // THE SECOND LIST, and it answers a different question from the one
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
    // THE THIRD LIST, and it is placed here for the two constraints [`groups`]'
    // header sets out.
    //
    // It must be **above** the fill list, because that list's own `ScrollArea`
    // takes the rest of the pane and the panel's top level does not scroll — so
    // anything after it is laid out past the bottom of a container with no way
    // to reach it.
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
fn offers_a_control(field: &Field) -> bool {
    rows::block_reason(field).is_none() && !field.is_rich_text()
}

/// The Calculated Fields section — decision 009 posture B.
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
            // A rule-4 disclosure: pdfcer INFERRED an evaluation order the
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
            // EVERY PROPOSED VALUE IS ON SCREEN BEFORE THE BUTTON THAT
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

    // The draft is moved OUT of `ui_state` for the duration of the rows and
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
        // **What the operator is typing ON THE PAGE, asked once for the whole
        // frame.**
        //
        // Beside `page_numbers` and `fill_refusal` because it is the same kind
        // of thing: a fact about the document that is identical for every row,
        // and one a 400-field form must not ask 400 times. `canvas::forms`
        // holds one focus, so the answer is one field or none.
        //
        // Asked from the PANEL rather than pushed by the canvas, which is the
        // direction `canvas::forms::placed` sets between these two modules: the
        // surface that owns the answer publishes it, and the surface that needs
        // it reads it. The reverse — the canvas writing into `FormsUi` — would
        // put a second writer on state whose whole correctness argument is its
        // `(path, epoch)` key.
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
fn raise(actions: &mut Vec<Action>, edit: FormEdit) {
    actions.push(FieldAction::Edit(edit).into());
}

/// The Forms panel's own inter-frame state: one text draft per field.
///
/// # This is not on `crate::panels::PanelsState`, and the preferred shape is
///
/// A `forms: FormsUi` field beside `tree: ObjectTreeUi`, dropped by
/// `PanelsState::forget_document` exactly as everything else there is. Written
/// down so the current home is read as a position rather than a preference.
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
/// # The key is `(path, edit_epoch)`, which is what makes UNDO correct
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
    /// Held here rather than in `egui`'s temp data for the reason every other
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

    /// **This panel is reachable from the ribbon.**
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

    /// **The "you can fill here" count agrees with what the rows draw.**
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

        // Rich text is the case `block_reason` deliberately does NOT cover:
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

        // And the bit-26 overload: a radio group with RadiosInUnison set
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

    /// **An edit forgets the drafts, which is what makes undo correct.**
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
