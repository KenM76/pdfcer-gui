//! # `canvas::forms` — filling a form **where it is drawn**
//!
//! The operator's complaint that started this module is one sentence: *"today
//! fields can only be filled through the side panel, and every PDF reader lets
//! you click the field on the page and type."* This is the click, and the
//! typing.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/forms.md`.

/// Where a form's widgets are, what a click on one would mean, and the five
/// reasons one is filled in the panel instead. The pure half — see its header
/// on why the split is a seam rather than a cut.
pub mod boxes;

/// The Edit-mode half: which field is selected, what its outline looks like,
/// and what a click means when a click is not a request to type. Split out
/// under R2; see its header for why the seam is real and not a cut.
mod selecting;

/// The order Tab visits a page's fields in, built from the engine's own
/// `page_tab_sequence`. Split out under R2; see its header.
mod ring;

/// Tab's own half: advancing the ring, and the focus a button holds while
/// it waits for a Space. Split out under R2; see its header.
mod tabbing;

/// A choice field's option list: the popup's side, its keyboard, and what a
/// pick sends. Split out under R2; see its header, and in particular its on
/// why the side is chosen before the constraint rather than after it.
mod choosing;

/// The live `egui::TextEdit` laid over a widget rectangle, and the three
/// document properties that dress it. Shared by the `/Tx` editor below and by
/// [`choosing`]'s editable combo box, so a form cannot be typed into with the
/// wrong quadding on one surface and the right one on the other.
mod textbox;

/// Re-exported so the path `canvas::forms::right_click_hits_a_field` — which
/// `canvas::rightclick` and `panels::properties::formfield` both cite by name
/// — survived the R2 split unchanged.
pub use selecting::right_click_hits_a_field;

use crate::app::actions::forms::FieldAction;
use std::path::PathBuf;
use std::sync::Arc;

use egui::{Id, Key, Ui};

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::canvas::forms::boxes::{BoxKind, WidgetBox, editor_rect, hit, offered_in, truncate};
use crate::canvas::strip::{DrawnPage, PageView};
use crate::canvas::tool::CanvasTool;
use crate::panels::forms::edit::FormEdit;

/// `egui::Memory` key for the frame's widget boxes, in canvas space.
const BOXES_KEY: &str = "pdfcer-canvas-form-boxes"; // ui-text-exempt: internal memory id, never displayed

/// `egui::Memory` key for which field is being typed into, and what into it.
const FOCUS_KEY: &str = "pdfcer-canvas-form-focus"; // ui-text-exempt: internal memory id, never displayed

/// `egui::Memory` key for "this frame's Escape was spent abandoning a draft".
const ESCAPE_KEY: &str = "pdfcer-canvas-form-escape"; // ui-text-exempt: internal memory id, never displayed

/// Id prefix for the focused field's editor. Salted with page, name and widget
/// index so two widgets of one field cannot collide.
const EDITOR_KEY: &str = "pdfcer-canvas-form-editor"; // ui-text-exempt: internal widget id, never displayed

/// Trace slot for the box census — deduplicated, because it is a fact about
/// the document rather than about a gesture and would otherwise print once per
/// frame forever.
const BOXES_SLOT: &str = "canvas-form-boxes"; // ui-text-exempt: trace slot name, never displayed
// ===========================================================================
// The state that outlives a frame
// ===========================================================================

/// Which field is being typed into, and what into it.
#[derive(Clone, Debug, PartialEq, Default)]
pub(super) struct Focus {
    /// The document this belongs to. A different file makes every field name
    /// here meaningless.
    pub(super) path: PathBuf,
    /// The revision the draft was seeded from.
    pub(super) epoch: u64,
    /// 0-based page index.
    pub(super) page: usize,
    /// The field's fully-qualified name.
    pub(super) field: String,
    /// The widget's index within the field.
    pub(super) widget: usize,
    /// What the operator has typed and not yet committed.
    pub(super) draft: String,
    /// `false` until the frame after the click, so the editor knows to ask for
    /// keyboard focus and to put the caret at the end exactly once.
    pub(super) seated: bool,
    /// How many more frames this focus survives not being drawable.
    ///
    /// Zero for a focus that arrived by a click: the box was under the
    /// pointer, so it is on screen, and a frame that cannot draw it is a
    /// frame that has genuinely lost it.
    ///
    /// Non-zero only for a focus a Tab press moved to — see [`advance`].
    /// That focus names a box which may be below the fold or on another
    /// page, and the scroll that brings it into view lands one or more
    /// frames later. Without this, [`editor`]'s two undrawable branches
    /// would settle the focus on the very frame the reveal was asked for,
    /// and Tab would appear to do nothing at all whenever the next field was
    /// off screen — which is most of the time on a form worth tabbing
    /// through.
    pub(super) waiting: u8,
}

impl Focus {
    /// The `egui` id of this focus's editor.
    pub(super) fn editor_id(&self) -> Id {
        Id::new((EDITOR_KEY, self.page, self.field.as_str(), self.widget))
    }

    /// Bring a stored focus up to date with the document, or discard it.
    fn sync(mut self, doc: &OpenDoc, stored: &str) -> Option<Self> {
        if self.path != doc.path {
            return None;
        }
        if self.epoch != doc.edit_epoch {
            self.epoch = doc.edit_epoch;
            self.draft = stored.to_owned();
        }
        Some(self)
    }
}

/// Read the stored focus.
pub(super) fn load_focus(ctx: &egui::Context) -> Option<Focus> {
    ctx.data(|d| d.get_temp::<Focus>(Id::new(FOCUS_KEY)))
}

/// Store, or forget, the focus.
pub(super) fn store_focus(ctx: &egui::Context, focus: Option<Focus>) {
    let id = Id::new(FOCUS_KEY);
    ctx.data_mut(|d| match focus {
        Some(f) => {
            d.insert_temp(id, f);
        }
        None => {
            d.remove::<Focus>(id);
        }
    });
}

/// **What the operator is typing on the page right now**, as
/// `(field name, draft)` — or `None` when they are not typing on the page.
pub(crate) fn live_draft(ctx: &egui::Context, doc: &OpenDoc) -> Option<(String, String)> {
    let focus = load_focus(ctx)?;
    if focus.path != doc.path || focus.epoch != doc.edit_epoch {
        return None;
    }
    if ctx.memory(egui::Memory::focused) != Some(focus.editor_id()) {
        return None;
    }
    Some((focus.field, focus.draft))
}

/// **The one walk of the form**, built at most once per `(document, revision)`
/// and shared by both surfaces.
pub(crate) fn placed(ctx: &egui::Context, doc: &OpenDoc) -> Arc<boxes::Placed> {
    let id = Id::new(BOXES_KEY);
    let key = (doc.path.clone(), doc.edit_epoch);
    if let Some((cached_key, list)) = ctx.data(|d| d.get_temp::<(BoxKey, Arc<boxes::Placed>)>(id))
        && cached_key == key
    {
        return list;
    }

    let view = doc.session.view();
    let list: Arc<boxes::Placed> = Arc::new(
        pdfcer_core::forms::parse_acroform(&view)
            .map(|form| {
                let annots: Vec<Vec<(pdfcer_core::object::ObjId, [f64; 4])>> = (0..doc.pages.len())
                    .map(|page| doc.session.widget_rects(page))
                    .collect();
                boxes::place(&form, &doc.pages, &annots)
            })
            .unwrap_or_default(),
    );
    ctx.data_mut(|d| d.insert_temp(id, (key, Arc::clone(&list))));

    crate::diag::trace_changed(BOXES_SLOT, || {
        let pages = list
            .boxes
            .iter()
            .map(|b| b.page)
            .collect::<std::collections::BTreeSet<_>>()
            .len();
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "form-boxes n={} pages={pages} undrawn={} unreachable={}",
            list.boxes.len(),
            list.routing.undrawn,
            list.routing.unreachable,
        )
    });
    // Then the census itself, one line per box, in CANVAS space.
    //
    // The summary above proves boxes exist; this proves *where*, which is the
    // only thing that makes a hit test checkable from outside the process. A
    // click that focused the field next to the one it aimed at is the same
    // screenshot as a click that worked — a defect no picture can carry —
    // and a harness can only tell the two apart by knowing the target
    // before it aims. Canvas space, because that is the frame the `canvas
    // rect=… zoom=…` line already publishes the map for: screen = rect.min +
    // canvas × zoom, which is arithmetic a harness can do.
    //
    // Written once per `(document, revision)` because it sits after the cache
    // miss, and capped because `MAX_FORM_FIELDS` is 500,000 — an uncapped
    // census on a pathological form would bury every other line in the
    // capture, which is the same "fifty identical lines in nine seconds"
    // failure `trace::pointer` was fixed for.
    // The SELECTABLE census, beside the fillable one and deliberately
    // separate. The two sets differ — a push button and an undrawn widget are
    // selectable and not fillable — and that difference is the whole
    // of what form authoring added to this surface. One census reporting the
    // union would make a harness unable to tell "this widget cannot be typed
    // into" from "this widget cannot be reached at all", which are the two
    // failures it most needs to distinguish.
    if crate::diag::enabled() {
        for t in list.targets.iter().take(MAX_TRACED_BOXES) {
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "form-target page={} field={} widget={} rect=({:.1},{:.1})+({:.1},{:.1})",
                    t.page,
                    t.field,
                    t.widget,
                    t.rect.min.x,
                    t.rect.min.y,
                    t.rect.width(),
                    t.rect.height(),
                )
            });
        }
    }
    if crate::diag::enabled() {
        for b in list.boxes.iter().take(MAX_TRACED_BOXES) {
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "form-box page={} field={} widget={} kind={} rect=({:.1},{:.1})+({:.1},{:.1})",
                    b.page,
                    b.field,
                    b.widget,
                    kind_label(&b.kind),
                    b.rect.min.x,
                    b.rect.min.y,
                    b.rect.width(),
                    b.rect.height(),
                )
            });
        }
        if list.boxes.len() > MAX_TRACED_BOXES {
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "form-box-census-truncated shown={MAX_TRACED_BOXES} total={}",
                    list.boxes.len()
                )
            });
        }
    }
    list
}

/// How many boxes the census names before it says how many it left out.
const MAX_TRACED_BOXES: usize = 64;

/// The cache key: which document, at which revision.
type BoxKey = (PathBuf, u64);

// ===========================================================================
// Escape's rung
// ===========================================================================

/// Record that this frame's Escape settled a draft — it wrote what was in the
/// editor and closed it. The rung exists to stop one press also ascending the
/// selection ladder, not to report a discard: nothing here discards.
pub(super) fn note_escape(ctx: &egui::Context) {
    ctx.data_mut(|d| d.insert_temp(Id::new(ESCAPE_KEY), true));
}

/// **Escape's claimant 0** — whether a focused field took this frame's key.
#[must_use]
pub fn escape_spent(ctx: &egui::Context) -> bool {
    let id = Id::new(ESCAPE_KEY);
    ctx.data_mut(|d| d.remove_temp::<bool>(id)).unwrap_or(false)
}

// ===========================================================================
// The frame
// ===========================================================================

/// **The one entry point.** Read the click, draw the focused field's editor,
/// and raise whatever the operator asked for.
pub(super) fn overlay(
    ui: &mut Ui,
    doc: &OpenDoc,
    pages: &[PageView],
    drawn: &[DrawnPage],
    tool: CanvasTool,
    authoring: bool,
    actions: &mut Vec<Action>,
) {
    // Cleared FIRST, before any early return, so a flag set on a frame where
    // the canvas then stopped drawing cannot be read by a later frame's
    // Escape. The alternative — clearing it where it is read — leaves it set
    // on exactly the frames nobody reads it.
    let ctx = ui.ctx().clone();
    ctx.data_mut(|d| d.remove::<bool>(Id::new(ESCAPE_KEY)));

    // **IN EDIT MODE A CLICK SELECTS THE FIELD; ELSEWHERE IT FILLS IT.**
    //
    // The operator: *"when I click on an existing form field on the page its
    // properties should come up in our side pane for editing its
    // properties."*
    //
    // The split is by mode rather than by a modifier, and that is the
    // conventional model rather than an invention: every program that both
    // fills and authors forms — Acrobat above all — separates the two into
    // distinct activities, because the same click cannot both type a value and
    // select the box to rename it. pdfcer already has the vocabulary for that
    // separation and it is the mode selector, whose whole job is *what will
    // this program let me do*. Read and Review fill; Edit authors.
    //
    // What it costs, stated rather than hidden: **filling on the page is
    // not available in Edit mode.** That is the correct trade and it is
    // reversible in one line if it proves wrong, but it is a real change — an
    // operator who was filling a form in Edit mode drops to Review to go on
    // doing it, and every field remains fillable in the Forms panel in every
    // mode. The alternative — a modifier key — would make the commonest
    // gesture on this surface depend on a key nobody discovers.
    //
    // Note it is asked BEFORE `offer`. Selection is not filling and must not
    // inherit filling's gates: `fill_refusal()` is `Some` for a certified
    // document, where the operator may still legitimately want to look at what
    // a field IS. What it does share is `annotations_visible`, because a
    // hidden widget is one nobody can see to click.
    if authoring {
        settle(&ctx, doc, actions);
        if doc.annotations_visible() {
            let placed = placed(&ctx, doc);
            selecting::seeded_select(doc, &placed.targets, actions);
            selecting::select_click(&ctx, doc, pages, drawn, &placed.targets, actions);
            selecting::select_cursor(&ctx, pages, &placed.targets);
            // **DRAW THE BOXES THEMSELVES** — O209, *"when I am in edit
            // mode I can't see these boxes."*
            //
            // He is right and the cause was here: the wash that makes a form
            // field findable was drawn below this branch's `return`, so it
            // reached every mode *except* the one whose whole job is finding a
            // box in order to move or resize it. An authoring surface that
            // hides its own subject is not usable however well the selection
            // works underneath.
            //
            // `targets`, not `boxes`, and unconditional rather than gated on
            // the display option — `form_marks::authoring_boxes` carries both
            // arguments and why each is not the filling surface's answer.
            crate::canvas::form_marks::authoring_boxes(ui, pages, &placed.targets);
            // DRAW THE SELECTION. `OPERATOR_REQUESTS.md` **O53**.
            //
            // Nothing painted a selected form field. The click landed, the
            // action was raised, `doc.selected_field` was set, the Properties
            // panel filled in -- and **the canvas showed no change at all**.
            //
            // That is the largest part of his *"I can't select it on the
            // canvas to move or resize"*: a selection with no visible outline
            // is not a selection an operator can believe in, whatever the state
            // underneath says. They click, see nothing, and conclude the click
            // did not work -- which is the correct conclusion from the evidence
            // available to them.
            //
            // => A selection is a claim the program makes to the operator. If
            // it is not drawn, the claim was never made, and every capability
            // that depends on it is unreachable however well it works.
            //
            // The grips come with it, which is H7: `pressing::grabbable`
            // hands `GripSet::scale_only()` for this selection, so the eight
            // squares are hit-tested whether or not they are painted -- and an
            // invisible target that steals a press is worse than a visible
            // control that does nothing.
            selecting::selection_overlay(&ctx, ui.visuals(), doc, pages, &placed.targets);
        }
        return;
    }

    if !offer(doc, tool) {
        // Whatever was focused, it is not focusable now: a certification
        // signature, a hidden annotation layer or another tool. Commit rather
        // than discard — see `settle`.
        settle(&ctx, doc, actions);
        return;
    }

    let placed = placed(&ctx, doc);
    let list = &placed.boxes;
    // FIRST, so the editor drawn below is the one Tab has just arrived at.
    // Asked before the empty-list return, so a press claimed on the frame an
    // undo removed the last field is consumed rather than left parked for a
    // later frame to act on.
    tabbing::advance(&ctx, doc, list, actions);
    if list.is_empty() {
        return;
    }

    // **THE FIELD WASH** — `OPERATOR_REQUESTS.md` O96, *"an option to shade
    // the form fields like acrobat does."*
    //
    // FIRST in this function, so every other overlay this module draws — the
    // focused editor, the selection outline, the grips — lands on top of it. A
    // wash painted last would sit over the caret and the text the operator is
    // typing, which is the one thing that must stay legible.
    //
    // Drawn here rather than by the page rasterizer, and that is what keeps
    // it inside rule 4: it is over the finished texture, so it reaches no
    // print, no export, no Save and no `render-page`.
    //
    // Gated on the preference only. `offer` above has already established
    // that this mode fills forms, that annotations are visible and that the
    // document is not certified — so a field that is not fillable here is a
    // field this function has already returned before reaching.
    crate::canvas::form_marks::shade(ui, doc, pages, list);

    // **THE SPOTLIGHT** — `OPERATOR_REQUESTS.md` O98. The field the Forms
    // panel is pointing at, outlined so the operator can see which box on the
    // page they are filling.
    //
    // After the wash and before everything else: it must sit *on* the shade
    // rather than under it, and *under* the focused editor's own caret and
    // text, which are what must stay legible.
    //
    // `crate::panels::forms::spotlight` carries why this is a cursor rather
    // than a mark on the content, and quotes the panel header that named this
    // gap — and named it permitted — long before it was built.
    crate::canvas::form_marks::spotlight(ui, pages, list);

    // The focused field's editor FIRST, so that a click on another field is
    // seen by the editor it is leaving (as a focus loss, hence a commit)
    // before it is read as a request to focus something else.
    let claimed = editor(ui, doc, pages, list, actions);
    // …then the click, which may be the one that just closed that editor.
    if !claimed {
        click(&ctx, doc, pages, drawn, list, actions);
    }
    cursor(&ctx, pages, list);
}

/// Whether this frame offers form filling at all — the two document-wide
/// gates, asked once. See the module header §5.
fn offer(doc: &OpenDoc, tool: CanvasTool) -> bool {
    offered_in(tool) && doc.annotations_visible() && doc.session.fill_refusal().is_none()
}

/// Commit and forget whatever was focused, because it cannot be focused any
/// more.
pub(super) fn settle(ctx: &egui::Context, doc: &OpenDoc, actions: &mut Vec<Action>) {
    let Some(focus) = load_focus(ctx) else {
        return;
    };
    store_focus(ctx, None);
    if focus.path != doc.path || focus.epoch != doc.edit_epoch {
        // The draft describes a document or a revision that is no longer on
        // screen. Writing it would be writing a value against a document the
        // operator has not seen since they typed it.
        return;
    }
    commit(&focus, doc, actions);
}

/// Raise a fill if the draft differs from what the document holds.
pub(super) fn commit(focus: &Focus, doc: &OpenDoc, actions: &mut Vec<Action>) {
    let stored = stored_value(doc, &focus.field).unwrap_or_default();
    let Some(value) =
        crate::panels::forms::rows::commit(true, focus.draft.as_str(), stored.as_str())
    else {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("form-commit field={} outcome=unchanged", focus.field)
        });
        return;
    };
    crate::diag::trace(|| {
        // The character COUNT, never the characters. See the module header §8.
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "form-commit field={} chars={}",
            focus.field,
            value.chars().count()
        )
    });
    actions.push(
        FieldAction::Edit(FormEdit::FillText {
            field: focus.field.clone(),
            value,
        })
        .into(),
    );
}

/// What the document currently holds for `field`, as display text.
pub(super) fn stored_value(doc: &OpenDoc, field: &str) -> Option<String> {
    let view = doc.session.view();
    let form = pdfcer_core::forms::parse_acroform(&view)?;
    form.fields
        .iter()
        .find(|f| f.fully_qualified_name == field)
        .map(|f| f.value.display_text())
}

/// **Interact with a focused field's rectangle and keep the keys it reads.**
pub(super) fn keyboard_box(ui: &mut Ui, id: Id, rect: egui::Rect) -> egui::Response {
    let response = ui.interact(rect, id, egui::Sense::focusable_noninteractive());
    ui.memory_mut(|m| {
        m.set_focus_lock_filter(
            id,
            egui::EventFilter {
                tab: false,
                horizontal_arrows: false,
                vertical_arrows: true,
                escape: true,
            },
        );
    });
    response
}

/// Draw the focused field's editor, and settle it when it is finished.
fn editor(
    ui: &mut Ui,
    doc: &OpenDoc,
    pages: &[PageView],
    list: &[WidgetBox],
    actions: &mut Vec<Action>,
) -> bool {
    let ctx = ui.ctx().clone();
    let Some(focus) = load_focus(&ctx) else {
        return false;
    };

    // The box the focus names, on a page this frame actually drew. A focus
    // whose field has gone (an undo removed it) or whose page has scrolled out
    // of the strip cannot be drawn, and a focus nobody can see is one the
    // operator cannot leave.
    let stored = stored_value(doc, &focus.field);
    let Some(focus) = focus.sync(doc, stored.as_deref().unwrap_or_default()) else {
        store_focus(&ctx, None);
        return false;
    };
    let placed = list
        .iter()
        .find(|b| b.page == focus.page && b.field == focus.field && b.widget == focus.widget)
        .and_then(|b| pages.iter().find(|v| v.page == b.page).map(|v| (b, v.map)));
    let Some((widget_box, map)) = placed else {
        return tabbing::hold_or_settle(&ctx, doc, focus, actions);
    };

    let rect = editor_rect(&map, widget_box.rect);
    if !ui.clip_rect().intersects(rect) {
        // Scrolled out of the viewport. Same answer as a page that left the
        // strip: commit what is there -- unless a Tab put the focus here and
        // the scroll that reveals it is still in flight.
        return tabbing::hold_or_settle(&ctx, doc, focus, actions);
    }

    if matches!(widget_box.kind, BoxKind::Choice { .. }) {
        // Routed before the destructure below, which binds its fields by copy
        // — a `Choice` carries two `Vec`s and cannot. `choosing::choose` draws
        // the ring itself, for the same reason `tabbing::button_focus` does:
        // the box takes the keyboard and the popup is the only thing on screen.
        return choosing::choose(ui, focus, widget_box, rect, actions);
    }

    let BoxKind::Text {
        multiline,
        password,
        max_len,
        align,
    } = widget_box.kind
    else {
        // A button cannot hold a caret, so it holds a focus RING instead and
        // reads Space, Enter and the arrow keys. Reached both by a Tab onto a
        // check box and by a click on one.
        return tabbing::button_focus(ui, doc, list, focus, widget_box, rect, actions);
    };

    let id = focus.editor_id();
    let mut draft = truncate(&focus.draft, max_len);
    // Which font, which end of the box, and what colour — all three are
    // properties of the document rather than of this interaction, so they live
    // in [`textbox`] and the editable combo box reads the same rules. The
    // three arguments that admit `/Q` and `/MK` `/BG` and refuse `/DA` are
    // on [`textbox::lay`].
    let response = textbox::lay(
        ui,
        &mut draft,
        &textbox::Spec {
            id,
            rect,
            align,
            fill: widget_box.fill,
            multiline,
            password,
            // One line per opened editor, not one per frame: the seating
            // branch below fires on the frame the caret is placed.
            trace: (!focus.seated).then_some(focus.field.as_str()),
        },
    );
    // The canvas now holds the keyboard, and says so -- this is what lets
    // `raw_input_hook` take the next Tab away from egui's focus walk before
    // `Focus::begin_pass` can latch it. Published every frame the editor is
    // drawn, because the identity test that reads it is also its freshness
    // test: see `canvas::tabnav`'s header.
    crate::canvas::tabnav::publish(&ctx, crate::canvas::tabnav::Scope::Field, id);

    // Seat the caret exactly once, at the END rather than over a selection.
    // The click that asked for this editor was consumed by the PAGE (see the
    // module header §4), so there is no click position to place a caret from,
    // and selecting all would turn the operator's next keystroke into a
    // deletion of the field's contents. [`textbox::seat`] carries why an
    // editable combo box answers this differently.
    if !focus.seated {
        response.request_focus();
        textbox::seat(&ctx, id, &draft, false);
    }

    // Escape closes the editor and **writes what is in it**, and says so.
    // Read BEFORE the `lost_focus` branch for the same reason `gesture` reads
    // it before its release branch: `egui`'s own `TextEdit` surrenders focus on
    // Escape, so without this arm the two are the same event and which one runs
    // is an accident of ordering.
    //
    // `OPERATOR_REQUESTS.md` **O223** — *"escape should also save changes
    // to the text … it is easy to accidentally press escape and lose a lot of
    // text that has been entered."* It is [`settle`]'s rule one surface over,
    // and [`settle`] already carried the argument: *a half-typed field value is
    // something they typed on purpose.* Escape was the one exit on this surface
    // that did not obey it.
    //
    // [`commit`] raises nothing when the draft matches what the document holds,
    // so Escape out of a field the operator only looked at still writes nothing
    // and puts nothing on the undo stack.
    if ctx.input(|i| i.key_pressed(Key::Escape)) {
        let leaving = Focus { draft, ..focus };
        store_focus(&ctx, None);
        note_escape(&ctx);
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("form-escape field={}", leaving.field)
        });
        commit(&leaving, doc, actions);
        return true;
    }

    if response.lost_focus() {
        let leaving = Focus { draft, ..focus };
        store_focus(&ctx, None);
        commit(&leaving, doc, actions);
        // The press that took focus away is still this frame's press, and it
        // may have landed on another field. Not claimed, so `overlay` reads it.
        return false;
    }

    store_focus(
        &ctx,
        Some(Focus {
            draft,
            seated: true,
            // The editor drew, so whatever reveal it was waiting for has
            // landed. See `Focus::waiting`.
            waiting: 0,
            ..focus
        }),
    );
    // A press inside the editor belongs to the editor — that is what
    // registering it in this layer bought — so it is claimed whether or not
    // `egui` calls it a click this frame.
    response.contains_pointer() && ctx.input(|i| i.pointer.any_pressed())
}

/// Read a primary click on a page, and act on the widget it landed in.
fn click(
    ctx: &egui::Context,
    doc: &OpenDoc,
    pages: &[PageView],
    drawn: &[DrawnPage],
    list: &[WidgetBox],
    actions: &mut Vec<Action>,
) {
    let Some(pos) = ctx.pointer_interact_pos() else {
        return;
    };
    let Some(page) = drawn
        .iter()
        .find(|d| d.response.clicked_by(egui::PointerButton::Primary))
        .map(|d| d.page)
    else {
        return;
    };
    let Some(map) = pages.iter().find(|v| v.page == page).map(|v| v.map) else {
        return;
    };

    let point = map.to_page(pos);
    let Some(widget_box) = hit(list, page, point) else {
        return;
    };
    crate::diag::trace(|| {
        let r = widget_box.rect;
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "form-hit page={page} field={} widget={} kind={} at=({:.1},{:.1}) \
             rect=({:.1},{:.1})+({:.1},{:.1})",
            widget_box.field,
            widget_box.widget,
            kind_label(&widget_box.kind),
            point.x,
            point.y,
            r.min.x,
            r.min.y,
            r.width(),
            r.height(),
        )
    });

    match &widget_box.kind {
        // Re-focusing the field that already has focus would re-seed the
        // draft from the document — which is to say, it would silently throw
        // away everything the operator has typed and not yet committed.
        //
        // Unreachable through the ordinary path (a press inside the editor is
        // claimed by the editor, which is the whole point of registering it in
        // the topmost layer), and guarded anyway: the cost is one comparison
        // and the failure it prevents is losing typing, which is the worst
        // thing this module could do.
        BoxKind::Text { .. }
            if load_focus(ctx).is_some_and(|f| {
                f.page == page && f.field == widget_box.field && f.widget == widget_box.widget
            }) => {}
        BoxKind::Text { .. } => {
            store_focus(
                ctx,
                Some(Focus {
                    path: doc.path.clone(),
                    epoch: doc.edit_epoch,
                    page,
                    field: widget_box.field.clone(),
                    widget: widget_box.widget,
                    draft: stored_value(doc, &widget_box.field).unwrap_or_default(),
                    seated: false,
                    waiting: 0,
                }),
            );
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!(
                    "form-focus page={page} field={} widget={}",
                    widget_box.field, widget_box.widget
                )
            });
        }
        // A button commits immediately and keeps no draft: one atomic change,
        // no intermediate state to protect, and therefore none of the
        // sixty-undo-entries argument that governs a text field.
        BoxKind::Check { on_state, on } => {
            let state = if *on {
                "Off".to_owned()
            } else {
                on_state.clone()
            };
            raise_button(&widget_box.field, state, actions);
            focus_button(ctx, doc, page, widget_box);
        }
        // Clicking the selected radio does nothing — see `BoxKind::Radio`.
        BoxKind::Radio { on_state, on } => {
            if !*on {
                raise_button(&widget_box.field, on_state.clone(), actions);
            }
            focus_button(ctx, doc, page, widget_box);
        }
        // A click writes nothing here: it opens the list, and the pick is the
        // command. Unlike a button, whose click IS the answer, a choice field
        // has to be asked which option — so the gesture is two-step and the
        // first step is not an edit.
        BoxKind::Choice { .. } => choosing::focus_choice(ctx, doc, page, widget_box, point),
    }
}

/// Put the keyboard on the button that was just clicked.
fn focus_button(ctx: &egui::Context, doc: &OpenDoc, page: usize, widget_box: &WidgetBox) {
    store_focus(
        ctx,
        Some(Focus {
            path: doc.path.clone(),
            epoch: doc.edit_epoch,
            page,
            field: widget_box.field.clone(),
            widget: widget_box.widget,
            // Equal to the stored value by construction, which is what makes
            // a button's focus harmless to `commit`: it never differs, so
            // nothing is ever written on the way out.
            draft: stored_value(doc, &widget_box.field).unwrap_or_default(),
            seated: false,
            waiting: 0,
        }),
    );
}

/// **Whether a canvas form ring is holding the keyboard**, and so must be
/// left the space bar.
#[must_use]
pub fn ring_takes_space(ctx: &egui::Context) -> bool {
    crate::canvas::tabnav::owns_focus(ctx, crate::canvas::tabnav::Scope::Field)
}

/// Push one button-state change.
pub(super) fn raise_button(field: &str, state: String, actions: &mut Vec<Action>) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("form-button field={field} state={state}")
    });
    actions.push(
        FieldAction::Edit(FormEdit::SetButtonState {
            field: field.to_owned(),
            state,
        })
        .into(),
    );
}

/// The trace's one-word name for a kind.
fn kind_label(kind: &BoxKind) -> &'static str {
    match kind {
        BoxKind::Text { .. } => "text",     // ui-text-exempt: trace token
        BoxKind::Check { .. } => "check",   // ui-text-exempt: trace token
        BoxKind::Radio { .. } => "radio",   // ui-text-exempt: trace token
        BoxKind::Choice { .. } => "choice", // ui-text-exempt: trace token
    }
}

/// Set the pointer's shape over a fillable widget.
fn cursor(ctx: &egui::Context, pages: &[PageView], list: &[WidgetBox]) {
    let Some(pos) = ctx.pointer_latest_pos() else {
        return;
    };
    for view in pages {
        if !view.map.image_rect().contains(pos) {
            continue;
        }
        if let Some(widget_box) = hit(list, view.page, view.map.to_page(pos)) {
            ctx.set_cursor_icon(match widget_box.kind {
                BoxKind::Text { .. } => egui::CursorIcon::Text,
                _ => egui::CursorIcon::PointingHand,
            });
            return;
        }
    }
}

/// See `canvas/forms/tests.rs` — a separate file under **R2**.
#[cfg(test)]
mod tests;
