//! # `canvas::forms` — filling a form **where it is drawn**
//!
//! The operator's complaint that started this module is one sentence: *"today
//! fields can only be filled through the side panel, and every PDF reader lets
//! you click the field on the page and type."* This is the click, and the
//! typing.
//!
//! It is a **second surface onto one implementation**, never a second
//! implementation. Everything below decides *which field the operator pointed
//! at* and *what they typed into it*; the moment either is settled it becomes
//! the same [`FormEdit`] the Forms panel raises, travels the same
//! `Action::Form` funnel, and reaches the same `EditSession` verb. There is no
//! fill path here. `grep FormEdit::` still answers "what can change a form?"
//! completely.
//!
//! ---
//!
//! ## 1. Why this is not a [`CanvasTool`] variant
//!
//! [`crate::canvas::tool`]'s header sets the bar for admission — a candidate
//! must *"arrive with its own state"* and must be an answer to **what does the
//! primary button mean right now?** Form filling fails both, and the second
//! failure is the decisive one.
//!
//! A form widget's `/Rect` is a small bounded region **the file itself marks as
//! interactive**, and its `/AP` already draws something that looks like an input
//! control, because that is what the document's author drew. There is no
//! ambiguity for a mode to resolve: a press inside that rectangle can only
//! sensibly mean *"fill this"*, and a press one point outside it can only mean
//! what a press on the page has always meant. Markup needed a variant because a
//! drag on blank paper is genuinely ambiguous between panning, marqueeing and
//! drawing. This is the opposite case.
//!
//! `D:\Dev\pdfcer\docs\ui_specs\pass-7-form-fill.md` §1.1 reached the same
//! conclusion in the old shell and stated the stronger half of the argument:
//! gating filling behind a mode *"would hide the primary reason most form
//! documents exist"*. `forms-panel.md` §1.2 records that the conclusion
//! survived the shell rewrite. And it is what the operator's standing
//! tie-breaker — *"make it work the way other programs do"* — asks for: no
//! reader in the world has a form-fill mode.
//!
//! So this lives in the ordinary [`CanvasTool::Select`] tool and is offered in
//! no other, which is one line ([`offered_in`]) rather than a variant.
//!
//! ---
//!
//! ## 2. The panel is not replaced, and could not be
//!
//! [`crate::panels::forms`] stays exactly as it is, in exactly its role. Two
//! reasons, and the first is not a preference:
//!
//! 1. **Accessibility.** The panel's rows are real `egui` widgets in a real
//!    layout: they get tab order, they get AccessKit exposure, and their labels
//!    are `/TU` — the string a screen reader actually announces for an
//!    interactive field. What this module projects onto the canvas gets none of
//!    that, and cannot: the thing underneath it is a **page raster**, which is
//!    a picture with no text alternative. An operator who cannot see the page
//!    cannot discover that a field exists here, let alone which one it is.
//! 2. **Everything this surface declines is still fillable there.** §5 lists
//!    five reasons a field is not offered on the page. Every one of them has
//!    the panel as its answer, and the panel says so
//!    ([`crate::text::forms::forms_canvas_undrawn_note`] and
//!    [`crate::text::forms::forms_canvas_unreachable_note`]).
//!
//! The canvas is an **additional** way in. If it ever becomes the only one,
//! this build has lost a capability it currently has.
//!
//! ---
//!
//! ## 3. What the operator sees, and what it does not promise
//!
//! **An unfocused widget is not painted at all.** Rule 4's one-line test is
//! *would a screenshot of the editing canvas differ from a screenshot of the
//! same document saved and reopened?* — so this module paints no highlight, no
//! outline and no tint over a field that is merely available. What it does
//! instead is change the **cursor** to an I-beam over a text field and a
//! pointing hand over a button, which rule 4 permits by name (*"a snap
//! indicator, a hover highlight, a rubber-band, a selection handle — these are
//! the cursor"*) and which costs the page nothing.
//!
//! That makes the cursor the whole discovery mechanism, and it is worth saying
//! plainly that it is a weaker one than Acrobat's blue field tint. The honest
//! remedy is a **"highlight fillable fields" toggle**, which is a ribbon
//! command; this module deliberately adds none and the entry point is reported
//! rather than wired.
//!
//! **A focused text field is replaced by an editor**, and the editor is
//! deliberately *not* a facsimile:
//!
//! - It draws in the theme's own text-edit colours at a size derived from the
//!   widget's height on screen ([`editor_font_size`]), not from the field's
//!   `/DA`.
//! - **It does honour `/Q`** ([`boxes::editor_align`]) — see the ★ below.
//! - It is never smaller than [`MIN_EDITOR`], so a 4 pt field at 25 % zoom is
//!   still something an operator can read what they typed in. It may therefore
//!   overhang the field it is editing. An editor that is honest about its
//!   position and unreadable is worse than one that is legible and a few points
//!   too big.
//!
//! **The reason it cannot be a facsimile is not effort, it is arithmetic.**
//! The overlay is a font substitution by construction: the glyphs come from
//! egui's bundled UI font, the document's come from the field's `/DA` font, and
//! `pdfcer-core`'s variable-text generator refuses font substitution in variable
//! text *precisely because it changes glyph advances*. A box that pretended to
//! be the appearance stream would be making a fidelity claim it cannot keep —
//! the caret would drift from where the glyph will actually land, and it would
//! drift further the longer the string. So the editor says **"you are typing
//! here"** and the `/AP` regenerated on commit says **"and this is what it
//! looks like"**, one gesture later. That is the same bargain a spreadsheet
//! makes between the formula bar and the rendered cell.
//!
//! ### ★★★ …and the exact reach of that argument, which was over-read
//!
//! The paragraph above is about **glyph advances**, and for eleven days it was
//! cited as though it settled every appearance property. It does not. An
//! outside review read it back to this project in 2026-09 and the reading is
//! correct: *"the argument is arithmetic — it does not reach text alignment or
//! the widget's background colour, so those are an unexamined gap rather than
//! a decision."*
//!
//! ⇒ **The test a property must pass to be honoured here is whether honouring
//! it makes a claim about where a particular glyph will land.** A font and a
//! size do — that is the whole substitution problem. `/Q` does not: it names
//! which end of the box the run is anchored to, and it names the same end
//! whatever font draws the run. So `/Q` is now read, through
//! [`boxes::editor_align`], and the operator's text no longer jumps from the
//! left of a centred field to its middle at the moment they tab away.
//!
//! The **background colour** half of the same finding was recorded here as a
//! boundary, and ⚠ **that boundary is gone — corrected 2026-09-07.**
//!
//! > *"`/MK` `/BG` is not in `pdfcer_core::forms::Widget` at all
//! > (`forms.rs:429` says why), so this shell cannot ask the question, let
//! > alone answer it. Stated, not guessed at."*
//!
//! `Widget::background` shipped 2026-09-04 and `Widget::border_color` on
//! 2026-09-07, and the line this cited now reads *"carries `/BC`, `/BG` (now
//! modelled)"*. **The question can be asked, and as of 2026-09-11 it is
//! answered**: `boxes::editor_fill` resolves `/MK` `/BG` onto the cached
//! [`boxes::WidgetBox`], and the editor below pairs it through
//! `Theme::foreign_fill_pair` before handing the `TextEdit` a
//! `.background_color()` and a `.text_color()`. A tinted form box keeps its
//! tint while it is being edited, and the ink on it is chosen against that
//! tint rather than against the theme's.
//!
//! ★ Kept rather than deleted because the sentence was **correct when
//! written and correctly filed** — it is what the engine request was argued
//! from. What expired is the tense, and a claim whose history is erased
//! cannot be audited.
//!
//! ### What this surface cannot promise, stated rather than discovered
//!
//! - **Glyph-accurate preview.** See above. The committed appearance is the
//!   truth; the editor is a place to type.
//! - **A caret where you clicked.** The first click is consumed by the page
//!   (§4), so the editor is created on the *next* frame and has no click to
//!   place a caret from. The caret goes to the end of the text, which is the
//!   least destructive place for it to be. A second click, once the editor
//!   exists, does place a caret — and a double-click selects a word — because
//!   by then the operator is interacting with a real `egui::TextEdit`.
//! - **Live agreement between two boxes of one field.** A field may have
//!   several widgets (§7). While one is being typed into, the others still
//!   show the `/AP` the document currently holds, and catch up on commit —
//!   because `/AP` is regenerated by the engine and there is no half-committed
//!   appearance to draw.
//!
//!   ★★★ **The Forms PANEL is deliberately not in this list, and used to be in
//!   it without saying so.** The 2026-09 review's row A12c found the panel row
//!   showing the old value for as long as the operator kept typing on the
//!   page, which reads as exactly the same lag — and is not. The sibling
//!   widget above shows an appearance nobody has generated yet; the panel row
//!   shows a `String` this module is holding. So the panel reads it
//!   ([`live_draft`]) rather than being told to expect a delay. The rule the
//!   pair states: **disclose a lag only when the value the surface would need
//!   does not exist.** Where it exists, a disclosure is an apology for a
//!   second draft store nobody had to keep.
//!
//! ---
//!
//! ## 4. Input layering — what claims a press, and what deliberately does not
//!
//! [`crate::canvas::guides`]' header §3 is the precedent: a widget registered
//! **after** the page widgets is the topmost one under the pointer, so a press
//! on it never reaches the page's `Response` and therefore never reaches the
//! gesture machine. That is exactly what a focused field needs, and exactly
//! what an *unfocused* one must not have — an interactive rectangle sitting
//! over every fillable field would swallow marquees, would swallow the hover
//! that gates Ctrl+wheel zoom, and would do both silently.
//!
//! So the two halves are asymmetric on purpose:
//!
//! | | registers a widget? | consequence |
//! |---|---|---|
//! | an **unfocused** widget | **no** — the click is read from the page's own `Response` | pans, marquees, Ctrl+wheel and hover over the page are untouched |
//! | the **focused** text field | **yes** — a real [`egui::TextEdit`] | its drags select text, its double-clicks select words, and none of it reaches the gesture machine |
//!
//! Reading the click off the page's `Response` rather than off a widget of our
//! own has one visible consequence and it is the right one: the click **also**
//! reaches the selection layer, so clicking a form field clears whatever vector
//! object was selected. That is what a click on "somewhere else" means
//! everywhere else in this canvas, and inventing an exception would be a rule
//! nobody could predict.
//!
//! ### ★ The hit test takes no tolerance, and that is a decision
//!
//! Every other hit test in `canvas/` passes [`PageMapping::tolerance`], because
//! [`crate::canvas::mapping`]'s header is about exactly one defect: *a screen
//! number used where a page number was meant*. This one passes nothing, and the
//! reason is that a tolerance answers a question a widget rect does not ask.
//!
//! A tolerance exists so a **hairline** — a stroked path a fraction of a point
//! wide — can be hit at all. A form widget is an **area** the document
//! declares, typically 10–30 points tall, and form fields are routinely
//! *adjacent*: a table of boxes with a one-point gutter between them is the
//! ordinary shape of a real form. A 6-pixel catch radius there would make the
//! boundary between two fields ambiguous and would silently focus the neighbour
//! of the one the operator aimed at — a wrong answer, delivered confidently,
//! which is the failure mode `mapping`'s own header calls the worse one.
//!
//! So [`hit`] is a plain containment test, and this paragraph exists so that
//! nobody "fixes" it by adding the tolerance every neighbouring call site uses.
//!
//! ---
//!
//! ## 5. Five reasons a field is not offered here — and all five keep the panel
//!
//! [`NotOnCanvas`] is the complete list, and each entry is a fact the *file*
//! states rather than a limit this module chose:
//!
//! 1. **[`NotOnCanvas::NoAppearance`] — the widget has no `/AP` `/N`.** It is
//!    therefore drawn as *nothing*, and a click target over blank paper is an
//!    invisible affordance. The rule this module holds to is **the canvas
//!    offers exactly what the page draws**; anything else means either an
//!    unhittable target or a painted placeholder, and the "no placeholders"
//!    invariant forbids the second. The remedy already exists and is one
//!    button away: `FormEdit::RegenerateAppearances` draws every field's
//!    current value, after which the field is visible **and** clickable. The
//!    panel says so ([`crate::text::forms::forms_canvas_undrawn_note`]).
//!    `D:\Dev\pdfcer\fixtures\synthetic\forms\demo-form.pdf` carries exactly
//!    this case.
//! 2. **[`NotOnCanvas::RotatedPage`] — the page's `/Rotate` is not 0.** The
//!    geometry is fine ([`widget_canvas_rect`] goes through the renderer's own
//!    transform, so the box lands in the right place at every rotation) — what
//!    breaks is the *editor*. `egui` cannot rotate a `TextEdit`, so on a
//!    `/Rotate 90` page the operator would type horizontally across text the
//!    `/AP` draws vertically. **Text fields only**: a check box has no text
//!    direction, so a button on a rotated page is offered exactly as it is
//!    anywhere else, and only the editor is withheld.
//! 3. **[`NotOnCanvas::NoRect`] — no usable `/Rect`.** A missing or zero-area
//!    rectangle is a widget with no place on the page. (§12.7.4.5 makes a
//!    zero-area `/Rect` *deliberate* invisibility for a signature field, which
//!    is not offered here anyway.)
//! 4. **[`NotOnCanvas::UnknownPage`] — the file does not say which page.**
//!    `pdfcer_core::forms::Widget::page` is `/P`, and core reads it without
//!    resolving through the graph, so a direct (non-reference) `/P` reads as
//!    absent. Either way there is no page to place the box on. Reported as a
//!    boundary observation rather than worked around: the shell cannot repair
//!    a fact the model does not carry.
//! 5. **[`NotOnCanvas::NotOffered`] — this kind has no canvas gesture.**
//!    Read-only, signature and push-button fields (the panel's
//!    [`crate::panels::forms::rows::block_reason`], asked here rather than
//!    re-derived), rich text (which the panel offers a *conversion* rather than
//!    a box), choice fields, and a button with no on-state. A choice field
//!    would need a dropdown anchored to the page, which is a second popup
//!    surface with its own placement rules and no gesture the panel does not
//!    already have.
//!
//! Two document-wide gates sit in front of all five, in [`offer`]:
//!
//! - **`EditSession::fill_refusal`** — a certification signature forbids
//!   filling the whole document, so nothing is offered anywhere. Asked once per
//!   frame, exactly as the panel asks it once per frame (R83, know before you
//!   offer).
//! - **`OpenDoc::annotations_visible`** — with `view.show_annotations` off,
//!   `/Widget` appearances are not painted at all. Offering a click on a field
//!   the operator has asked not to see would be offering a click on nothing,
//!   and it is the same rule as reason 1 wearing the operator's hat instead of
//!   the document's.
//!
//! ---
//!
//! ## 6. Escape, and where it sits in the ladder
//!
//! [`crate::canvas::keys`]' header ranks Escape's claimants. A focused field is
//! rung **0** — the most transient thing on the canvas, because it is the thing
//! the operator's hands are on — and it abandons the draft without writing
//! anything.
//!
//! The exclusion is **mechanical rather than ordered**, which is worth being
//! precise about because it is the only rung that works that way. While this
//! module's `TextEdit` holds focus, `egui::Context::text_edit_focused` is true,
//! and that predicate already turns off *every* other claimant: `canvas_keys`
//! returns on it (`DEFECTS.md` D1's guard), and `interact` builds the gesture
//! machine's `cancel` flag from it. So no other claimant can even see the key.
//!
//! What that leaves is the **other** direction: `egui`'s own `TextEdit`
//! surrenders focus on Escape, and it does so *before* `canvas_keys` runs. One
//! press would then abandon the draft **and** ascend a selection rung — the
//! exact double effect decision 025's L1 forbids. [`escape_spent`] is how this
//! module says it took the key, read by `canvas_keys` as claimant 0, in the
//! same report-rather-than-re-derive shape every other claimant uses.
//!
//! ---
//!
//! ## 7. The commit boundary, and one field with several widgets
//!
//! **Focus-loss-and-changed, plus Enter** — and it is
//! [`crate::panels::forms::rows::commit`], the panel's own pure function,
//! called rather than restated. Both halves of its condition matter here for
//! the same reasons they matter there: `changed()` would make one typed word a
//! dozen undo entries and a dozen appearance regenerations, and an unchanged
//! field left without typing must write nothing at all. Enter needs no code —
//! `egui` surrenders a singleline `TextEdit`'s focus on Enter, so Enter *is*
//! focus loss; a multiline field keeps Enter for the newline `/Ff` `Multiline`
//! asks for.
//!
//! `EditSession::fill_text_field` writes `/V` once and regenerates the `/AP` of
//! **every** widget of the field, as one undo entry, so the write is already
//! correct for a field presented in several places. What needs discipline is
//! the *draft*, and the discipline is the panel's: the draft is keyed by
//! `(path, edit_epoch)`, and an epoch change re-seeds it from the document
//! rather than surviving it. See [`Focus::sync`] for the one place this differs
//! from the panel and why.
//!
//! ---
//!
//! ## 8. Proving it from outside
//!
//! `HANDOFF.md` §2: eight defects have been found by running the program and
//! none of them by the suite. A hit test is invisible in a screenshot — a click
//! that focused the right field and a click that focused the field next to it
//! are the same picture at 100 % zoom. So every decision this module makes is
//! on the `PDFCER_DIAG` channel:
//!
//! ```text
//! form-boxes n=3 pages=1            # deduped: how many boxes exist at all
//! form-hit page=0 field=Name kind=text at=(126.4,203.1) rect=(120.0,198.0)+(200.0,14.0)
//! form-focus page=0 field=Name widget=0
//! form-commit field=Name chars=4
//! form-abandon field=Name
//! form-button field=Agree state=Yes
//! ```
//!
//! The field **name** is on the line and the typed **value** never is. That is
//! the same split [`crate::panels::forms::edit::FormEdit::label`] makes and for
//! the same reason: a name is metadata a log needs in order to be useful, and a
//! value may be what an operator typed into a `/Ff` `Password` field, which
//! pdfcer has just warned them is stored in the clear.

/// Where a form's widgets are, what a click on one would mean, and the five
/// reasons one is filled in the panel instead. The pure half — see its header
/// on why the split is a seam rather than a cut.
pub mod boxes;

/// The Edit-mode half: which field is selected, what its outline looks like,
/// and what a click means when a click is not a request to type. Split out
/// under R2; see its header for why the seam is real and not a cut.
mod selecting;

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
use crate::canvas::forms::boxes::{
    BoxKind, WidgetBox, editor_align, editor_font_size, editor_rect, hit, offered_in, truncate,
};
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
///
/// One draft, not a map: exactly one field on the canvas can hold focus, which
/// is the structural difference from [`crate::panels::forms::FormsUi`] — that
/// one draws every row every frame and so must keep a draft per row.
#[derive(Clone, Debug, PartialEq, Default)]
struct Focus {
    /// The document this belongs to. A different file makes every field name
    /// here meaningless.
    path: PathBuf,
    /// The revision the draft was seeded from.
    epoch: u64,
    /// 0-based page index.
    page: usize,
    /// The field's fully-qualified name.
    field: String,
    /// The widget's index within the field.
    widget: usize,
    /// What the operator has typed and not yet committed.
    draft: String,
    /// `false` until the frame after the click, so the editor knows to ask for
    /// keyboard focus and to put the caret at the end exactly once.
    seated: bool,
}

impl Focus {
    /// The `egui` id of this focus's editor.
    fn editor_id(&self) -> Id {
        Id::new((EDITOR_KEY, self.page, self.field.as_str(), self.widget))
    }

    /// Bring a stored focus up to date with the document, or discard it.
    ///
    /// # ★ Where this differs from the panel, and why
    ///
    /// [`crate::panels::forms::FormsUi`] keys its drafts on `(path, epoch)` and
    /// **drops them all** when either moves. Doing that here would be wrong in
    /// a case the panel does not have: clicking field B while field A is
    /// focused commits A *and* focuses B in one frame, so the very next frame
    /// carries a new epoch — and dropping the focus on it would put the caret
    /// out of a field the operator had just clicked into.
    ///
    /// So a path change discards, and an epoch change **re-seeds**: the focus
    /// survives, the draft is replaced by what the document now holds. That is
    /// the same correctness the panel's rule buys — after an undo the box shows
    /// the reverted value rather than the typed one, so it cannot re-commit
    /// what was just undone — with the one behaviour the panel does not need.
    ///
    /// Nothing is lost by re-seeding for the same reason nothing is lost in the
    /// panel: an epoch moves only when a document edit lands, every gesture
    /// that can land one takes focus away first, and taking focus away commits.
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
fn load_focus(ctx: &egui::Context) -> Option<Focus> {
    ctx.data(|d| d.get_temp::<Focus>(Id::new(FOCUS_KEY)))
}

/// Store, or forget, the focus.
fn store_focus(ctx: &egui::Context, focus: Option<Focus>) {
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
///
/// # ★★★ Why this exists: the Forms panel was a frame behind by a whole
/// gesture
///
/// The 2026-09 outside review, row **A12c**: *"the Fill-form panel does not
/// update while you type on the page."* It was exactly right, and the
/// mechanism is two draft stores that had never been introduced to each other.
/// [`Focus::draft`] holds what is being typed on the canvas;
/// [`crate::panels::forms::FormsUi::drafts`] holds what is being typed in the
/// panel; the only thing that ever reconciled them was a **commit**, which
/// happens on focus loss. So an operator filling a field on the page watched
/// the panel row beside it go on showing the old value for as long as they
/// kept typing — two boxes, side by side, disagreeing about one field.
///
/// ⇒ **The honest fix was to delete one of the two answers, not to disclose
/// the gap.** §3's list of what this surface cannot promise carries the
/// *sibling-widget* version of this lag, and that one is genuinely
/// undisclosable-away: a second widget of the same field draws the `/AP` the
/// document currently holds, and there is no half-committed appearance for it
/// to draw instead, because `/AP` is regenerated by the engine on commit. The
/// panel is not in that position at all. A panel row is an `egui::TextEdit`
/// over a `String`, and the `String` it should be showing is right here.
///
/// So the panel now **reads this** rather than keeping a second opinion, which
/// is the same shape as [`placed`] one function below: one answer, two
/// surfaces, and no drift possible between them because there is nothing to
/// drift from.
///
/// # ★★ The `egui` focus check is the whole of the safety argument
///
/// A stored [`Focus`] is not by itself evidence that the operator is typing on
/// the page — it survives until the editor loses focus, and this function is
/// called from a panel that is drawn on every frame, including frames where
/// the canvas is not the thing being interacted with. Returning a draft in
/// that state would let the canvas's stale value overwrite the panel's live
/// one, which is the defect this fixes wearing the other hat.
///
/// `ctx.memory(focused) == Some(editor_id)` is exact: `egui` has one focused
/// widget, so the predicate is true on precisely the frames the page's editor
/// owns the keyboard, and false on every frame the panel's own row does. It is
/// also what makes the commit path safe — the frame the editor loses focus is
/// a frame this returns `None`, so the panel is never mirroring a draft that
/// is on its way to becoming a document value.
///
/// # ★ Why the `(path, epoch)` guard as well
///
/// The same reason [`Focus::sync`] carries it. A draft belonging to another
/// document is meaningless, and a draft taken at another revision describes a
/// value the operator has not seen since they typed it. The panel drops its
/// own drafts on both changes ([`crate::panels::forms::FormsUi::load`]), so
/// mirroring across either boundary would be handing the panel back the one
/// thing its key exists to throw away.
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
///
/// Held as an `Arc` so the per-frame read is a refcount bump rather than a
/// clone of every field name in the document.
///
/// The cache is keyed on `(path, edit_epoch)` and **not** on the zoom, the
/// scroll offset or the page: canvas space is invariant under all three, which
/// is the property that makes this cheap enough to consult on every frame in
/// order to set a cursor.
///
/// # ★ `pub(crate)`, because the panel reads the same answer
///
/// [`crate::panels::forms::canvas_routing`] needs to know how many fields the
/// page cannot be clicked for, and the only correct source of that number is
/// the walk that decided it. Handing the panel this cache rather than letting
/// it repeat the walk buys three things: the count cannot disagree with the
/// behaviour, the form is parsed once per revision instead of twice per frame,
/// and `EditSession::widget_rects` is asked once per page per revision instead
/// of once per page per frame.
///
/// # Why the geometry comes from `widget_rects` and not from the form
///
/// `EditSession::widget_rects(page)` reports every `/Widget` **that page's
/// `/Annots` lists**, with corners already normalised and the session overlay
/// applied. See [`boxes::place`]'s ★ section for why asking the pages is the
/// only correct direction and why asking the widgets' `/P` is a defect no
/// fixture in the corpus can catch.
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
    // ★ Then the census itself, one line per box, in CANVAS space.
    //
    // The summary above proves boxes exist; this proves *where*, which is the
    // only thing that makes a hit test checkable from outside the process. A
    // click that focused the field next to the one it aimed at is the same
    // screenshot as a click that worked — `HANDOFF.md` §2's defect 8 in a new
    // place — and a harness can only tell the two apart by knowing the target
    // before it aims. Canvas space, because that is the frame the `canvas
    // rect=… zoom=…` line already publishes the map for: screen = rect.min +
    // canvas × zoom, which is arithmetic a harness can do.
    //
    // Written once per `(document, revision)` because it sits after the cache
    // miss, and capped because `MAX_FORM_FIELDS` is 500,000 — an uncapped
    // census on a pathological form would bury every other line in the
    // capture, which is the same "fifty identical lines in nine seconds"
    // failure `trace::pointer` was fixed for.
    // ★★ The SELECTABLE census, beside the fillable one and deliberately
    // separate. The two sets differ — a drop-down, a push button and an undrawn
    // widget are selectable and not fillable — and that difference is the whole
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
///
/// Not a round number for its own sake: it is comfortably more than any real
/// form's *visible* field count and small enough that a truncated census is
/// still a few screens of trace rather than a wall. A form that exceeds it
/// says so on the next line rather than silently stopping, because a census
/// that ends without saying it ended is a census a harness would read as
/// complete.
const MAX_TRACED_BOXES: usize = 64;

/// The cache key: which document, at which revision.
type BoxKey = (PathBuf, u64);

// ===========================================================================
// Escape's rung
// ===========================================================================

/// Record that this frame's Escape abandoned a draft.
fn note_escape(ctx: &egui::Context) {
    ctx.data_mut(|d| d.insert_temp(Id::new(ESCAPE_KEY), true));
}

/// **Escape's claimant 0** — whether a focused field took this frame's key.
///
/// Read *and cleared*, so one press cannot be claimed twice. See the module
/// header §6 for why this rung is needed at all when `text_edit_focused`
/// already silences every other one.
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
///
/// Called from [`crate::canvas::show_in`] immediately after
/// [`crate::canvas::guides::canvas_drag`] — the same place, the same layer and
/// for the same reason: everything registered here is later than every page
/// widget, so the focused editor is the topmost thing under the pointer and
/// the gesture machine never sees a press meant for it.
///
/// Nothing is mutated. Every outcome leaves as a [`FormEdit`] on `actions`,
/// which is the invariant the whole shell is built on and which this surface
/// honours by construction: `doc` arrives as a shared reference.
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

    // ★★★ **IN EDIT MODE A CLICK SELECTS THE FIELD; ELSEWHERE IT FILLS IT.**
    //
    // The operator, 2026-08-26: *"when I click on an existing form field on the
    // page its properties should come up in our side pane for editing its
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
    // ★★ What it costs, stated rather than hidden: **filling on the page is
    // not available in Edit mode.** That is the correct trade and it is
    // reversible in one line if it proves wrong, but it is a real change — an
    // operator who was filling a form in Edit mode drops to Review to go on
    // doing it, and every field remains fillable in the Forms panel in every
    // mode. The alternative — a modifier key — would make the commonest
    // gesture on this surface depend on a key nobody discovers.
    //
    // ★ Note it is asked BEFORE `offer`. Selection is not filling and must not
    // inherit filling's gates: `fill_refusal()` is `Some` for a certified
    // document, where the operator may still legitimately want to look at what
    // a field IS. What it does share is `annotations_visible`, because a
    // hidden widget is one nobody can see to click.
    if authoring {
        settle(&ctx, doc, actions);
        if doc.annotations_visible() {
            let placed = placed(&ctx, doc);
            selecting::select_click(&ctx, doc, pages, drawn, &placed.targets, actions);
            selecting::select_cursor(&ctx, pages, &placed.targets);
            // ★★★ DRAW THE SELECTION. `OPERATOR_REQUESTS.md` **O53**.
            //
            // Nothing painted a selected form field. The click landed, the
            // action was raised, `doc.selected_field` was set, the Properties
            // panel filled in -- and **the canvas showed no change at all**.
            //
            // ★★★ That is the largest part of his *"I can't select it on the
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
            // ★★ The grips come with it, which is H7: `pressing::grabbable`
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
    if list.is_empty() {
        return;
    }

    // ★★★ **THE FIELD WASH** — `OPERATOR_REQUESTS.md` O96, *"an option to shade
    // the form fields like acrobat does."*
    //
    // FIRST in this function, so every other overlay this module draws — the
    // focused editor, the selection outline, the grips — lands on top of it. A
    // wash painted last would sit over the caret and the text the operator is
    // typing, which is the one thing that must stay legible.
    //
    // ★★ Drawn here rather than by the page rasterizer, and that is what keeps
    // it inside rule 4: it is over the finished texture, so it reaches no
    // print, no export, no Save and no `render-page`.
    //
    // ★ Gated on the preference only. `offer` above has already established
    // that this mode fills forms, that annotations are visible and that the
    // document is not certified — so a field that is not fillable here is a
    // field this function has already returned before reaching.
    crate::canvas::form_marks::shade(ui, doc, pages, list);

    // ★★★ **THE SPOTLIGHT** — `OPERATOR_REQUESTS.md` O98. The field the Forms
    // panel is pointing at, outlined so the operator can see which box on the
    // page they are filling.
    //
    // After the wash and before everything else: it must sit *on* the shade
    // rather than under it, and *under* the focused editor's own caret and
    // text, which are what must stay legible.
    //
    // ★ `crate::panels::forms::spotlight` carries why this is a cursor rather
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
///
/// Reached when the tool changes, the annotations are hidden, the document is
/// certified between frames, or the focused field's page scrolls out of the
/// strip. **Committing rather than discarding** is the old spec's rule and it
/// is right: a half-drawn markup shape has nothing an operator would miss, and
/// a half-typed field value is something they typed on purpose.
fn settle(ctx: &egui::Context, doc: &OpenDoc, actions: &mut Vec<Action>) {
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
///
/// [`crate::panels::forms::rows::commit`] is the rule — **the panel's own pure
/// function**, called rather than restated, so "tabbing through a field writes
/// nothing" is one statement with one test and not two.
fn commit(focus: &Focus, doc: &OpenDoc, actions: &mut Vec<Action>) {
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
///
/// Re-read from the session on every frame that needs it rather than carried
/// on [`Focus`], for the reason [`crate::panels::forms::rows`] re-reads it: the
/// stored value is the thing a draft is compared against, and a cached copy is
/// one more thing that can be stale at exactly the moment the comparison
/// decides whether to write.
fn stored_value(doc: &OpenDoc, field: &str) -> Option<String> {
    let view = doc.session.view();
    let form = pdfcer_core::forms::parse_acroform(&view)?;
    form.fields
        .iter()
        .find(|f| f.fully_qualified_name == field)
        .map(|f| f.value.display_text())
}

/// Draw the focused field's editor, and settle it when it is finished.
///
/// Returns whether the editor claimed this frame's primary click, so
/// [`overlay`] does not also read the same press as a request to focus
/// something.
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
        settle(&ctx, doc, actions);
        return false;
    };
    let BoxKind::Text {
        multiline,
        password,
        max_len,
        align,
    } = widget_box.kind
    else {
        // A button cannot hold a caret. Reachable only if a document changed
        // a field's type under a live focus, which is not a case to panic on.
        store_focus(&ctx, None);
        return false;
    };

    let rect = editor_rect(&map, widget_box.rect);
    if !ui.clip_rect().intersects(rect) {
        // Scrolled out of the viewport. Same answer as a page that left the
        // strip: commit what is there rather than keep an invisible caret.
        settle(&ctx, doc, actions);
        return false;
    }

    let id = focus.editor_id();
    let mut draft = truncate(&focus.draft, max_len);
    let font = egui::FontId::proportional(editor_font_size(rect.height()));
    // ★★★ **`/Q`, and it is the one appearance property this editor reads.**
    //
    // §3 above refuses to make this box a facsimile, and the refusal is
    // arithmetic — glyph advances the substituted font cannot promise. It does
    // not reach quadding, which states which END of the box the text is
    // anchored to and says the same thing in any font. Until 2026-09-04 the
    // editor read `/Q` nowhere, so a centred or right-aligned field was typed
    // into left-aligned and its text jumped across the box the instant the
    // operator tabbed away — the appearance regenerated on commit had always
    // honoured `/Q`, and only the place the operator was looking did not.
    //
    // The whole of the rule is [`boxes::editor_align`]; see its doc and
    // [`BoxKind::Text::align`] for why this one property is admissible and the
    // `/DA` font and size are not.
    let halign = editor_align(align);

    // ★★★ **`/MK` `/BG`, the second appearance property this editor reads, and
    // it is here for the same reason `/Q` is.**
    //
    // Until this existed the live box was `extreme_bg_color` — near-white under
    // every light preset — so a pale-yellow or shaded field **turned grey the
    // moment the operator touched it** and turned back a gesture later. Nothing
    // in the file changed; the only thing that changed was the colour of the
    // thing being looked at, which is exactly the flicker pdfcer's rule 4
    // exists to forbid. The engine's own `Widget::background` doc names this
    // editor as its intended consumer, in these words: *"an on-page field
    // EDITOR that lays a live text box over the raster … so a pale-yellow field
    // does not flash white while the operator types."*
    //
    // ★★ **The fill and the ink arrive together, and that is not tidiness.**
    // A document-derived fill under a theme-chosen foreground is `DEFECTS.md`
    // D2's second shape — *a foreground assigned for a fill the text is not
    // on* — and it is how the old GUI shipped near-white headings on light
    // grey. [`Theme::foreign_fill_pair`] measures the pair and answers `None`
    // when no theme ink reads on that fill; `None` means **paint neither**, so
    // an unreadable field keeps the theme's own readable box rather than
    // becoming a tinted one the operator cannot read their own typing in.
    //
    // What is deliberately NOT tinted: the focus ring. A ring is the *cursor*,
    // which rule 4 admits in full — it says where the keystrokes are going, not
    // what the document contains.
    //
    // The whole of which colour this is, including why DeviceCMYK is converted
    // here and refused in the markup band, is [`boxes::editor_fill`].
    let tint = widget_box
        .fill
        .and_then(|srgb| egui_shell::theme::Theme::foreign_fill_pair(&ctx, srgb));

    let mut edit = if multiline {
        egui::TextEdit::multiline(&mut draft)
    } else {
        egui::TextEdit::singleline(&mut draft).password(password)
    }
    .id(id)
    .horizontal_align(halign)
    .font(egui::FontSelection::from(font));
    if let Some((fill, ink)) = tint {
        edit = edit.background_color(fill).text_color(ink);
    }

    // ★★ **The refusal is traced, because an operator cannot see one.**
    //
    // Three outcomes reach this point and only two of them are visible. A
    // field with no `/BG` keeps the theme box, which is right and expected. A
    // field WITH a `/BG` that the theme has no readable ink for also keeps the
    // theme box — identical on screen, a different fact about the file — and
    // pdfcer's rule 4 is explicit that an inference the operator cannot see
    // still owes an off-canvas report. This is that report, in the place this
    // shell puts machine-readable ones.
    //
    // Emitted once per opened editor rather than per frame: it is keyed on the
    // seating branch below, which fires on the frame the caret is placed.
    // Components are printed as decimals rather than `Debug`-formatted,
    // because a driven check reads this line and a `{:?}` tuple is a shape
    // that changes when the type does.
    if !focus.seated {
        crate::diag::trace(|| match (widget_box.fill, tint) {
            (None, _) => {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!("form-editor-tint field={} bg=absent", focus.field)
            }
            (Some(srgb), Some((_, ink))) => format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "form-editor-tint field={} bg={:.3},{:.3},{:.3} ink={},{},{}",
                focus.field,
                srgb[0],
                srgb[1],
                srgb[2],
                ink.r(),
                ink.g(),
                ink.b()
            ),
            (Some(srgb), None) => format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "form-editor-tint field={} bg={:.3},{:.3},{:.3} declined=unreadable",
                focus.field, srgb[0], srgb[1], srgb[2]
            ),
        });
    }
    let response = ui.put(rect, edit);

    // ★ Seat the caret exactly once. The click that asked for this editor was
    // consumed by the PAGE (see the module header §4), so there is no click
    // position to place a caret from; the end of the text is the least
    // destructive place for it, because the alternative — selecting all — turns
    // the operator's next keystroke into a deletion of the field's contents.
    if !focus.seated {
        response.request_focus();
        let end = egui::text::CCursor::new(draft.chars().count());
        if let Some(mut state) = egui::TextEdit::load_state(&ctx, id) {
            state
                .cursor
                .set_char_range(Some(egui::text::CCursorRange::one(end)));
            egui::TextEdit::store_state(&ctx, id, state);
        }
    }

    // ★ Escape abandons, and says so. Read BEFORE the commit branch for the
    // same reason `gesture` reads it before its release branch: `egui`'s own
    // `TextEdit` surrenders focus on Escape, so without this the abandon and
    // the commit are the same event.
    if ctx.input(|i| i.key_pressed(Key::Escape)) {
        store_focus(&ctx, None);
        note_escape(&ctx);
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("form-abandon field={}", focus.field)
        });
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
            ..focus
        }),
    );
    // A press inside the editor belongs to the editor — that is what
    // registering it in this layer bought — so it is claimed whether or not
    // `egui` calls it a click this frame.
    response.contains_pointer() && ctx.input(|i| i.pointer.any_pressed())
}

/// Read a primary click on a page, and act on the widget it landed in.
///
/// The click is read from the **page's own `Response`** rather than from a
/// widget of this module's, which is the whole of the input-layering decision —
/// see the module header §4.
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
        // ★ Re-focusing the field that already has focus would re-seed the
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
        }
        // Clicking the selected radio does nothing — see `BoxKind::Radio`.
        BoxKind::Radio { on_state, on } => {
            if !*on {
                raise_button(&widget_box.field, on_state.clone(), actions);
            }
        }
    }
}

/// Push one button-state change.
fn raise_button(field: &str, state: String, actions: &mut Vec<Action>) {
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
        BoxKind::Text { .. } => "text",   // ui-text-exempt: trace token
        BoxKind::Check { .. } => "check", // ui-text-exempt: trace token
        BoxKind::Radio { .. } => "radio", // ui-text-exempt: trace token
    }
}

/// Set the pointer's shape over a fillable widget.
///
/// **The whole of the discovery affordance**, and the only thing this module
/// puts on screen for a field that is not being edited — see the module header
/// §3 on why rule 4 permits a cursor and forbids a tint.
///
/// Set here, before
/// [`crate::canvas::interact`](crate::canvas::interact::interact) runs, so
/// [`crate::canvas::tool::cursor_for`] still has the last word: with the select
/// tool it has no opinion and this survives, and where it does have one — an
/// in-flight drag, a hovered resize grip — that opinion is about a gesture
/// already under way and outranks a hover.
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

/// See `canvas/forms/tests.rs` — moved out under R2 on 2026-09-04.
#[cfg(test)]
mod tests;
