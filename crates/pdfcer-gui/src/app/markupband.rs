//! # `app::markupband` — the five Format ▸ Markup controls the ribbon cannot
//! draw itself
//!
//! `RIBBON_IA.md` §5.8's *Markup annotation* row, and the operator's ask of
//! 2026-09-06: *"getting full editing working for the Markup tools."*
//!
//! ## What this module is
//!
//! The Markup group has five controls and **not one of them is a button**: two
//! colours that must also *show* the mark's current one, a width the operator
//! drags, a percentage, and a four-way choice of arrowheads.
//! `egui_shell::manifest::Item::Custom` is the extension point for exactly that
//! — it hands the application a `Ui` and gets out of the way — and this module
//! is what goes in that `Ui`.
//!
//! [`crate::app::fontband`] is the precedent and this file is deliberately its
//! twin: same shape, same four obligations, same park-and-report contract. Read
//! that module's header first; what is below is what differs, and every
//! difference is argued rather than inherited.
//!
//! ## ★★★ The shell reserves the slot; everything else is ours, including the
//! greying
//!
//! `egui_shell::ribbon::control::render_command` does four things for a command
//! item: it evaluates `enable`, it draws the control greyed when the predicate
//! is false, it shows the tooltip through `on_hover_text` **or**
//! `on_disabled_hover_text`, and it publishes the control's rect under
//! `ribbon.item.<id>`. For a custom item it does **none** of them — it cannot,
//! because it does not know what is being drawn.
//!
//! So all four are done here, in the same shapes and under the same names. The
//! rect matters as much as the greying: it is published under
//! `egui_shell::ribbon::report::band_item(id)`, the same name the shell builds
//! for a command control, so a driven check finds the fill swatch the way it
//! finds a Delete button. A second naming scheme for *the same kind of thing,
//! drawn by the other half of the program* is how a harness comes to have two
//! lookup paths, which is the defect `driving::declared_or_in_overflow` was
//! written to end.
//!
//! ## ★★ It reports; it does not dispatch
//!
//! Every control here parks a [`MarkupEdit`] and returns the command's
//! `HandlerToken`. It raises no `Action` and touches no document.
//!
//! That is `egui-shell`'s contract — *"the shell reports, the application
//! dispatches"* — and R8's dispatch choke point besides: a capability that
//! edits the document is a registered command, and a registered command is
//! invoked through `PdfcerApp::dispatch_command`, which is the same point a
//! chord and a context-menu row reach.
//!
//! ⚠ **`panels::properties::markup` pushes an `Action` straight into the queue
//! and this must not copy it.** That is the panel's established path and
//! `app::surfaces` states the rule at the ribbon's own custom-item closure: the
//! three Font controls *"DO return a token, where the pen's swatch above does
//! not, and the difference is what the control acts on."* A ribbon control that
//! built its own `Action` would put the operand derivation — *which annotation,
//! on which page* — in the renderer, where a chord never reaches it, and the
//! copy in `app::dispatch::format` would be the one that went stale.
//!
//! ## ★★★ One field per raised action, never a struct reassembled from widgets
//!
//! [`MarkupEdit`] carries exactly one property and [`MarkupEdit::into_style`]
//! sets exactly one field of `MarkupStyle`. `MarkupStyle`'s own doc comment is
//! the rule and the reason:
//!
//! > Every field is `None` by default … That shape is deliberate: a Format tab
//! > whose colour picker also had to restate the current width would overwrite
//! > whatever the operator had set from the other control.
//!
//! The failure that prevents is specific and this surface is where it would
//! have happened: five controls drawn from one annotation, one of them stale by
//! a frame, and a colour change that silently reverts a width set a moment
//! earlier. `panels::properties::markup` §"Every control is `None` unless the
//! operator touched it" argues it at length; this module obeys the same rule
//! through a type rather than through care.
//!
//! ## ★★ Absence and greying, and why this group's answers differ from Font's
//!
//! | state | Font group | Markup group |
//! |---|---|---|
//! | the mode cannot author | **absent** (`mode.edit_content`) | **absent** (`selection.markup_restylable`) |
//! | no operand | **greyed**, and the tooltip says how to get one | **absent** — same condition |
//! | the operand refuses | the run is CMYK: greyed swatch, own sentence | the mark is **locked**: greyed, own sentence |
//!
//! The middle row is the whole difference. A greyed Font control is the one
//! surface in this application that can say *sweep with the Text tool first*
//! (O37); a greyed Markup control could only say *select a mark*, which the
//! operator has already done or the contextual tab would not be on screen. R9
//! then requires absence, and `manifest::format`'s `MARKUP_VISIBLE_WHEN`
//! carries the argument in full.
//!
//! ⇒ The bottom row is what greying is left for, and it is R9's textbook case:
//! §12.5.3 Table 165 bit 8 is a fact about **this annotation** rather than about
//! the build or the mode — click a different mark and the controls work. The
//! sentence is `text::panels::properties::markup_locked`, which is the string
//! the Properties panel shows, so the two surfaces cannot refuse for different
//! reasons.
//!
//! ## ★★★ WHICH subtype takes WHICH property is the ENGINE's question, and this
//! module no longer holds an answer to it
//!
//! Until 2026-09-06 this file decided, **in this crate**, that a text markup has
//! no border to widen, that a line has no interior to fill, and that a `/Line`
//! alone has ends to put a head on. It derived all three by matching on
//! `MarkupSpec`'s arms in [`Current::read`]. That list was correct on the day it
//! was written, and this project filed it as a boundary defect anyway, in these
//! words:
//!
//! > *"That list is the engine's to know. The first subtype that gains or loses
//! > a border is the day our copy is wrong and nothing tells us."*
//!
//! ⇒ The engine shipped **`pdfcer_core::edit::MarkupStyleSupport::for_subtype`**
//! (`edit.rs:4493`, the type at `edit.rs:4460`) with `takes_border`,
//! `takes_interior` and `takes_endings` — and quoted that sentence into the
//! type's own doc comment as its justification. [`Current::support`] is the
//! answer, asked once per frame off the annotation's `/Subtype`; the three
//! controls read it and **nothing here re-derives it**.
//!
//! ### ⚠ The distinction that must not be collapsed
//!
//! | question | whose | where it is answered |
//! |---|---|---|
//! | *"does this subtype take a border width?"* | the **engine's** | [`Current::support`] |
//! | *"what IS this mark's border width?"* | this module's | the `MarkupSpec` arm |
//! | *"which anchors do I paint for this shape?"* | this module's | `canvas::annotnodes` |
//!
//! The middle row is why the `match` on `MarkupSpec` survives: only
//! `MarkupSpec::Square` has a `border_width` field, so reading the **value**
//! out of an arm is something the compiler checks and something the engine has
//! no API for. What is gone is the arm deciding whether the control exists.
//! `canvas::annotnodes`' header draws the third distinction and is right; this
//! module does not touch it.
//!
//! ### ★★ Belt and braces — a predicate to ask, and a refusal if you ask anyway
//!
//! The same Pass shipped `EditError::StylePropertyNotApplicable { id, subtype,
//! property }` (`edit.rs:7353`), raised at `edit.rs:26460`–`26483` **before**
//! anything is regenerated, so a width sent to a highlight leaves the file
//! untouched. Both mechanisms are in use here and neither replaces the other:
//! the predicate shapes the UI so the refusal is unreachable, and the refusal
//! is what makes a drifted shell loud instead of silent.
//!
//! ★ Surfacing it costs this module nothing, and that is by design rather than
//! by omission: `app::actions::funnel::vector_edit`'s `Err` arm
//! (`funnel.rs:276`) already routes every `EditError` to the decline channel —
//! `crate::text::status::edit_declined_by_engine` on screen, the engine's own
//! sentence into `PDFCER_DIAG` — and `check-ui-strings.sh`'s exclusion 3 says
//! in as many words that an error's `Display` is *"not permission to route UI
//! text through an error type"*. So the refusal arrives; it arrives through the
//! one channel every refusal uses, and this module does not build a second.
//!
//! ## ★★★ The genuine fifth state of the arrowhead chooser
//!
//! `MarkupStyle::endings` became `Option<StyleEdit<(LineEnding, LineEnding)>>`
//! on the same day (`edit.rs:4390`): `Set` writes `/LE`, **`Clear` removes it**
//! (`edit.rs:26557`). The four positions this chooser offers all *write* the
//! key, so *"no arrowheads"* — `/LE [/None /None]` — and *"no line-ending entry
//! at all"* are two different files that draw the same line, and only the first
//! was reachable.
//!
//! ### ★★ It is an ACTION below a separator, not a fifth peer — and here is why
//!
//! A fifth position in the list would be a fifth answer to the question the
//! list asks (*which ends carry a head?*) that **draws identically to the
//! first**. Two entries a drafter cannot tell apart by looking is the one thing
//! this control cannot afford: the mistake is undiscoverable on screen and
//! shows up in a byte comparison of a drawing that has already gone out. It
//! would also make the combo's `selected_text` ambiguous, because a mark with
//! no `/LE` and a mark with `/LE [/None /None]` would both have a claim on it.
//!
//! ⇒ So the four positions stay four, and the removal sits under a separator as
//! what it is — an act, not a state. Three consequences fall out of that shape
//! and each of them is the reason:
//!
//! - **It costs no band width.** `ribbon::plan::CUSTOM_ITEM_WIDTH` is 96 and
//!   this group is already sized to fit inside it; a button beside the combo
//!   would have spent the budget on the rarest control in the group.
//! - **It is ABSENT when there is no `/LE` to remove**, which is the rule
//!   [`fill`]'s Clear and `panels::properties::markup`'s three Clears already
//!   obey: *a Clear beside a mark that has nothing to clear is a control whose
//!   only possible effect is an undo entry the operator did not earn.* That is
//!   what [`Current::endings_key_present`] is for, and it is read off the
//!   dictionary rather than off the spec, because `spec_from_dict` supplies
//!   Table 176's default and so cannot tell an absent key from a written one.
//! - **It is where the operator already is.** They opened the chooser to change
//!   their mind about arrowheads; the way back to the file they were given is
//!   in the same popup, not on a second control they must go looking for.
//!
//! ★ The wording is a drafter's and lives in the catalog, once, read by **both**
//! surfaces: `text::panels::properties::markup_endings_clear` and its hover.
//! One string, so the tab and the panel cannot come to describe one act two
//! ways.
//!
//! ## ★ Rule 15 — a ce dimension is a different verb and must not appear here
//!
//! **ce dimensions** are the ones pdfcer authors; **pdf dimensions** are CAD
//! page content. A ce dimension is `panels::properties::dimension`'s and uses
//! `set_dimension_style`; handing one to `set_markup_style` regenerates it as a
//! bare line with its label and witness lines gone, and the engine refuses it
//! by name.
//!
//! The guard is a `match` on `AnnotKind` that the **compiler** checks, in two
//! places that cannot disagree: [`resolved`] here and `app::conditions`'
//! publication of the condition. It is deliberately not a comparison of
//! `/Subtype` strings, because a ce dimension's `/Subtype` **is** `/Line`,
//! exactly like an arrow's — a string test would restyle the operator's
//! dimensions into bare lines and would look correct while doing it.
//!
//! ## ★ Why this module reads the session itself rather than sharing the
//! panel's reader
//!
//! [`Current::read`] is a twenty-line dictionary read that
//! `panels::properties::markup::Current::read` also performs. Duplicating it is
//! the deliberate choice: that function is private to its module, five tracks
//! were writing in this tree on the day this landed, and making it
//! `pub(crate)` would have been an edit to another author's file to save
//! twenty lines.
//!
//! ⚠ **Corrected 2026-09-06.** This paragraph used to end: *"The two are
//! allowed to differ, and one of them does — this one also reads `/LE` and the
//! interior, which the panel offers no control for."* That was true when it was
//! written and stopped being true the same day: `panels::properties::markup`
//! gained `fill_row` and `endings_row` in the session that added this band, so
//! **both surfaces now read all five terms**. The permission still stands — the
//! two readers are allowed to differ — but the example of a difference is gone,
//! and a header that keeps a stale example teaches the next reader a fact about
//! the panel that is no longer so.
//!
//! ⚠ What is **not** duplicated is the derivation that matters: both read
//! through `annot_author::spec_from_dict`, the author's view, which is what
//! `set_markup_style` reads when it plans. One derivation of *what this mark
//! currently is*, two readers of it.
//!
//! ## Rule 4 / R8b — fuzzy, never sneaky
//!
//! Nothing here marks the canvas and nothing here can. The restyled mark
//! renders exactly as the saved file will render it; what the engine could not
//! reproduce is raised by `app::actions::annots` into the status bar, and what
//! a wider pen does to the annotation's `/Rect` is disclosed in this group's
//! tooltips and in the Properties panel's standing note.

use egui::Ui;
use egui_shell::commands::{CommandRegistry, ConditionSet, HandlerToken};
use pdfcer_core::annot_author::{Color, LineEnding};
use pdfcer_core::edit::{MarkupStyle, MarkupStyleSupport, StyleEdit};

use crate::app::state::OpenDoc;
use crate::canvas::selection::annot::{AnnotKind, AnnotTarget};
use crate::text::panels::properties as t;
use crate::text::ribbon as r;

/// The narrowest border this shell offers, in points.
///
/// The same floor `panels::properties::markup` and `canvas::markup::pen` use,
/// and the reason is theirs: §8.4.3.2 gives `0` a defined meaning — *the
/// thinnest line the device can render* — which on a 600 dpi plot is a hairline
/// and on screen at 25 % is invisible. An operator who wants a mark they cannot
/// see has the visibility toggle; what they must not get is a mark whose weight
/// depends on the output device without being told.
const MIN_WIDTH_PT: f64 = 0.25;

/// The widest, in points.
///
/// Beyond about twelve points a border stops reading as a border and starts
/// reading as a filled shape. Same ceiling as the pen that authors, so an
/// operator cannot set a width here that no gesture could have produced.
const MAX_WIDTH_PT: f64 = 12.0;

/// The width of the two numeric fields, in points.
///
/// ★ Chosen against `egui_shell::ribbon::plan::CUSTOM_ITEM_WIDTH`, which is
/// **96** and is what the band budgets for a custom item it cannot measure.
/// That module is explicit about the asymmetry — *"an estimate that is too
/// small costs a clipped group; it cannot cost the overflow control"* — so
/// every control here is sized to fit **inside** the budget rather than to look
/// comfortable. `app::fontband`'s size field is 46 for the same reason and
/// these match it, because two drag fields on one tab that are different widths
/// read as a layout accident.
const FIELD_WIDTH: f32 = 46.0;

/// The width of the arrowhead chooser, in points.
///
/// Wider than the fields because its longest entry is a phrase rather than a
/// number, and still inside the 96-point budget with room for the combo's own
/// frame and arrow.
const ENDINGS_WIDTH: f32 = 84.0;

/// The width of the line-style chooser, in points.
///
/// ★ Narrower than [`ENDINGS_WIDTH`] even though its list has a longer entry,
/// and that is a deliberate acceptance of one clipped reading rather than a
/// measurement mistake. The four *entries* are short — the longest is
/// *"Dash-dot"* — and the only long string this combo can show is
/// `linestyle::DashReading::Foreign`'s *"Dashed (the file's own pattern)"*,
/// which appears on a producer's mark and not on anything this shell drew. A
/// group of six custom items has to fit inside
/// `egui_shell::ribbon::plan::CUSTOM_ITEM_WIDTH` apiece or it costs a clipped
/// band, and spending the extra points on the rarer string would be the wrong
/// trade.
///
/// ⇒ The full reading is still reachable: the Properties panel's Line style row
/// draws the same chooser with the panel's width, which is where a reader who
/// wants the whole sentence goes. That is §5.8's division of labour working as
/// intended rather than a gap.
const DASH_WIDTH: f32 = 88.0;

/// **One property of one mark, parked for the dispatcher.**
///
/// ★★★ The type is the enforcement of this module's central rule. Each variant
/// carries exactly one field of `MarkupStyle`, so there is no expressible value
/// that restates a property the operator did not touch — which is what
/// `MarkupStyle`'s own doc requires and what a `MarkupStyle` parked directly
/// would have made merely a matter of care.
///
/// ★ It is this module's own type rather than a reuse of `MarkupStyle`, and
/// that is worth one sentence: `MarkupStyle` is an **input struct** the engine
/// deliberately left non-`#[non_exhaustive]` so callers can build it, and it is
/// perfectly buildable here. What it cannot express is *"exactly one field, and
/// the caller chose which"*, which is the invariant the dispatcher relies on.
/// ⚠ **Not `Copy` since 2026-09-06, and the engine's own note predicted it.**
/// [`Self::Dash`] carries a `BorderDash`, which owns a `Vec<f64>`
/// (`pdfcer-core` `annot_author.rs:157`), so the derive lost `Copy` the day the
/// dash arrived. Every consequence was mechanical — this type is *constructed*
/// at each control and *consumed* by `into_style`, and nothing ever read it
/// twice. It is called out because the reply that shipped the field warned that
/// move errors would appear and warned against reaching for a `clone` in a hot
/// path to silence them; there is none, and there should not be one.
#[derive(Debug, Clone, PartialEq)]
pub enum MarkupEdit {
    /// `/C` — the outline colour.
    Stroke(StyleEdit<Color>),
    /// `/IC` — the interior colour, or its removal (*no fill*).
    Interior(StyleEdit<Color>),
    /// `/BS` `/W` — the stroke width, in points.
    ///
    /// A bare `f64` and not a `StyleEdit`, because the engine's field is a bare
    /// `Option<f64>`: a border width has no *absent* state that means anything
    /// different from the standard's 1.0, so there is nothing for `Clear` to do.
    Width(f64),
    /// `/CA` — the constant opacity, `0.0`–`1.0`.
    Opacity(StyleEdit<f64>),
    /// `/LE` — the pair of line endings, or the **removal of the key**.
    /// `/Line` only.
    ///
    /// ★★ A `StyleEdit` since 2026-09-06, and the two arms are two different
    /// files that draw the same line: `Set` writes the array — including
    /// `Set((None, None))`, which is *"no arrowheads"* stated explicitly — and
    /// `Clear` removes `/LE` altogether, so the mark goes back out the way it
    /// came in. See this module's header for why the second is offered as an
    /// action rather than as a fifth position in the chooser.
    Endings(StyleEdit<(LineEnding, LineEnding)>),
    /// ★★★ `/BS` `/S` + `/D` — the **line style**: dashed, or solid.
    ///
    /// `RIBBON_IA.md` §5.8's eighth control, and the only one of the eight that
    /// had *"no engine verb at all"* until the afternoon of 2026-09-06.
    /// `MarkupStyle::dash` (`pdfcer-core` `edit.rs:4422`) is that verb, and its
    /// two arms are the two things this chooser can mean: `Set(dash)` makes the
    /// border dashed with that pattern, `Clear` makes it solid.
    ///
    /// ★★ **The third arm is `None`, and it is what NOT touching the control
    /// does.** A restyle that does not mention `dash` preserves whatever the
    /// annotation has, *including a dash pdfcer never authored*
    /// (`edit.rs:4396-4425`) — so an operator who changes a foreign-dashed
    /// mark's colour keeps their dash, which is exactly the defect this shell
    /// filed and the reason [`crate::canvas::markup::linestyle::DashReading::Foreign`]
    /// is a state the chooser can show and not one it has to repair.
    ///
    /// The pattern is built by
    /// [`crate::canvas::markup::linestyle::LineStyle::style_edit`], which is
    /// also where `BorderDash::new`'s `Option` is handled — [`dash`] parks
    /// nothing when it answers `None`, so nothing unbuildable ever reaches this
    /// variant.
    Dash(StyleEdit<pdfcer_core::annot_author::BorderDash>),
}

impl MarkupEdit {
    /// Turn the parked property into the partial override the engine takes.
    ///
    /// Every arm sets **one** field and leaves the rest at `Default` — which is
    /// `None`, which is *"do not touch this property"*. That is the whole
    /// contract, and it is asserted from the outside by
    /// `only_one_field_is_ever_set` below rather than trusted.
    pub(super) fn into_style(self) -> MarkupStyle {
        match self {
            Self::Stroke(edit) => MarkupStyle {
                stroke: Some(edit),
                ..MarkupStyle::default()
            },
            Self::Interior(edit) => MarkupStyle {
                interior: Some(edit),
                ..MarkupStyle::default()
            },
            Self::Width(width) => MarkupStyle {
                width: Some(width),
                ..MarkupStyle::default()
            },
            Self::Opacity(edit) => MarkupStyle {
                opacity: Some(edit),
                ..MarkupStyle::default()
            },
            // ★ Passed through whole rather than wrapped in `Set` here, which
            // is what the variant's own `StyleEdit` buys: the decision between
            // *write the array* and *remove the key* was taken at the control,
            // by the operator, and this function's job is to place it in one
            // field of an otherwise-default struct. Wrapping here would have
            // put the fifth state out of reach again for exactly the reason the
            // engine's field was a bare `Option` before 2026-09-06.
            // ★ Passed through whole for [`Self::Endings`]' reason exactly: the
            // decision between *dashed* and *solid* was taken at the chooser,
            // by the operator, and `Clear` here is the engine's own spelling of
            // *make it solid* rather than the removal of a control's value.
            Self::Dash(edit) => MarkupStyle {
                dash: Some(edit),
                ..MarkupStyle::default()
            },
            Self::Endings(edit) => MarkupStyle {
                endings: Some(edit),
                ..MarkupStyle::default()
            },
        }
    }
}

/// Draw one Format ▸ Markup custom item, or nothing.
///
/// Returns the command's handler token when the operator changed something, in
/// which case `parked` holds the change and `target` holds the annotation it is
/// about. `None` means *nothing was invoked*, which is what the shell expects
/// for a frame in which the operator merely looked at the control.
///
/// # ★ `kind` is matched, not asserted
///
/// An unrecognised kind returns `None` and draws nothing, exactly as
/// `PdfcerApp::ribbon_band`'s renderer does for one it does not know. A
/// manifest is data; the honest response to a kind nobody implements is a gap,
/// not a panic in the paint loop.
pub(super) fn draw(
    ui: &mut Ui,
    kind: &str,
    registry: &CommandRegistry,
    conditions: &ConditionSet,
    doc: Option<&OpenDoc>,
    parked: &mut Option<(AnnotTarget, MarkupEdit)>,
) -> Option<HandlerToken> {
    let id = command_for(kind)?;
    // R8: a build that does not register the command draws no control for it.
    let command = registry.get(id)?;
    let enabled = command.is_enabled(conditions);

    // ★ The read-back is attempted only when the control could be live, and
    // unlike `fontband`'s equivalent that is **correctness** rather than
    // performance: `Current::read` walks one annotation dictionary and is
    // cheap, but with no markup selected there is nothing to read and a control
    // that asked anyway would be building a value it must not show. A greyed
    // control has no value; see `t::text_value_absent` for why showing one
    // anyway is a claim about the operator's document.
    let operand = if enabled {
        resolved(doc)
    } else {
        Operand::Absent
    };
    let live = matches!(operand, Operand::Ready(..));

    let mut invoked = false;
    let response = ui
        .add_enabled_ui(live, |ui| {
            // ★ The `Ready` values, or the defaults a greyed control shows. The
            // `let else` rather than an `expect`: a paint-loop panic on a state
            // that is merely unexpected is a worse failure than a control that
            // draws inert, and `add_enabled_ui(false, ..)` has already made it
            // take no clicks.
            let Operand::Ready(target, current) = &operand else {
                placeholder(ui, kind);
                return;
            };
            invoked = match kind {
                k if k == crate::shell::manifest::MARKUP_STROKE => {
                    stroke(ui, *current, target, parked)
                }
                k if k == crate::shell::manifest::MARKUP_FILL => fill(ui, *current, target, parked),
                k if k == crate::shell::manifest::MARKUP_WIDTH => {
                    width(ui, *current, target, parked)
                }
                k if k == crate::shell::manifest::MARKUP_OPACITY => {
                    opacity(ui, *current, target, parked)
                }
                k if k == crate::shell::manifest::MARKUP_DASH => dash(ui, *current, target, parked),
                _ => endings(ui, *current, target, parked),
            };
        })
        .response;

    crate::diag::ui_rect(&egui_shell::ribbon::report::band_item(id), response.rect);

    // ★★★ **The sentence depends on WHY the control is not live**, which is the
    // defect `fontband::colour` records paying for over eight days: one hover
    // string answering two unrelated states, confidently and wrongly for one of
    // them. There are exactly two here and they are told apart by
    // [`Operand`]'s own variants rather than by a heuristic:
    //
    // | state | hover |
    // |---|---|
    // | the mark is **locked** | `t::markup_locked` — names the standard, and says deleting is still possible |
    // | anything else | the registry's own tooltip for the command |
    //
    // The second case is barely reachable — the group is absent unless a markup
    // is selected — but it is not unreachable: a condition is evaluated a
    // frame's worth of state earlier than this draw, so a selection cleared
    // within the frame lands here. Answering it with the locked sentence would
    // be a confident claim about a document that said no such thing.
    let locked = matches!(operand, Operand::Locked);
    if locked {
        response.on_disabled_hover_text(t::markup_locked());
    } else if let Some(tip) = command.tooltip.as_ref() {
        if live {
            response.on_hover_text(tip);
        } else {
            response.on_disabled_hover_text(tip);
        }
    }

    invoked.then_some(command.handler)
}

/// The command each custom kind draws the control for.
///
/// One place, so that the kind → id mapping cannot be spelled one way in the
/// renderer and another in `manifest::CUSTOM_BACKED` — which is the register
/// that keeps these five from looking like orphaned commands to the
/// reachability check, and which is asserted against the manifest rather than
/// against this function.
fn command_for(kind: &str) -> Option<&'static str> {
    match kind {
        // ui-text-exempt: command ids, never displayed.
        k if k == crate::shell::manifest::MARKUP_STROKE => Some("format.colour"),
        k if k == crate::shell::manifest::MARKUP_FILL => Some("format.fill"),
        k if k == crate::shell::manifest::MARKUP_WIDTH => Some("format.line_width"),
        k if k == crate::shell::manifest::MARKUP_OPACITY => Some("format.opacity"),
        k if k == crate::shell::manifest::MARKUP_DASH => Some("format.line_style"),
        k if k == crate::shell::manifest::MARKUP_ENDINGS => Some("format.arrowheads"),
        _ => None,
    }
}

/// What these controls have to act on, and — when they have nothing — *why*.
///
/// ★ Three variants and not an `Option`, because the two failure states get
/// different hovers and an `Option` would force the caller to re-derive which
/// one it was. That re-derivation is exactly the shape of `fontband::colour`'s
/// recorded defect, where one arm answered two reasons with one sentence for
/// eight days.
enum Operand {
    /// A markup annotation this mode may restyle, and what it currently says.
    Ready(AnnotTarget, Current),
    /// §12.5.3 Table 165 bit 8 — the document says the user interface may not
    /// change this annotation's properties, and the engine refuses
    /// `set_markup_style` for one by name.
    Locked,
    /// No markup selected. The group is absent in this state, so it is reached
    /// only through a frame's lag or a chord bound to one of the five ids.
    Absent,
}

/// The annotation these controls would act on, read fresh from the session.
///
/// # ★★ Read from the SESSION every frame, never from a cache
///
/// The verb these controls raise rewrites the very values they display, and an
/// action is applied *after* the frame that raised it — so a cached copy would
/// be stale for exactly the frame the operator is looking at, which is the
/// frame they judge the result on. `panels::properties::markup` states the same
/// rule for the same reason.
///
/// # ★ The `AnnotKind` guard is here AND in `app::conditions`, deliberately
///
/// The condition already refuses a ce dimension, so this looks redundant. It is
/// not: a condition is a hint published for the ribbon's benefit and is
/// evaluated a frame's worth of state earlier than this draw, and — the case
/// that actually bites — a **chord** consults no condition at all. Rule 15's
/// guard has to be where the operand is built, and the `match` on `AnnotKind`
/// is what makes routing one to the wrong verb a compile error rather than a
/// wrong `/Subtype` string comparison.
fn resolved(doc: Option<&OpenDoc>) -> Operand {
    let Some(doc) = doc else {
        return Operand::Absent;
    };
    let Some(selection) = doc.selection.annot() else {
        return Operand::Absent;
    };
    match selection.target.kind {
        AnnotKind::Markup => {}
        // A ce dimension. `set_dimension_style` is its verb; see this module's
        // header, Rule 15.
        AnnotKind::CeDimension => return Operand::Absent,
    }
    if selection.target.locked {
        return Operand::Locked;
    }
    let current = Current::read(doc, selection.target.id);
    // Cloned rather than borrowed: `AnnotTarget` carries the `/Subtype` as an
    // owned `String`, so it is not `Copy`, and the parked edit has to outlive
    // the borrow of `doc` that produced it.
    Operand::Ready(selection.target.clone(), current)
}

/// The placeholder a greyed control shows in place of a value it does not have.
///
/// ★★★ **A greyed field shows the PLACEHOLDER, not a number.**
/// `fontband::size` records what the alternative costs: with nothing swept the
/// draft held its `Default` — zero — and `DragValue`'s own range clamped it up,
/// so the greyed control read `1.0 pt`, which is a claim about the operator's
/// document and a false one. A driven check was right to pass; it asserted that
/// the control was drawn, and it was.
///
/// A `Button` rather than an inert `DragValue`, for that module's reason: the
/// shape of the thing an operator is looking at should say *there is no value
/// here*, not *here is a number you may scrub*. It is disabled, so it takes no
/// clicks and reports nothing.
fn placeholder(ui: &mut Ui, kind: &str) {
    let wide = kind == crate::shell::manifest::MARKUP_ENDINGS;
    let want = if wide { ENDINGS_WIDTH } else { FIELD_WIDTH };
    let response = ui.add_enabled(false, egui::Button::new(t::text_value_absent()));
    let _ = ui.allocate_space(egui::Vec2::new(
        (want - response.rect.width()).max(0.0),
        0.0,
    ));
}

/// What the selected mark's dictionary currently says, in the five terms this
/// band can change.
///
/// # ★★ Why it is read through `spec_from_dict` and not from `annot::Annotation`
///
/// `pdfcer_core::annot::Annotation` is the **reader's** view — id, subtype,
/// rect, flags, `/CA`, appearance — and it deliberately carries no `/C`, no
/// `/IC`, no `/BS /W` and no `/LE`, because nothing that renders a page needs
/// them: the picture comes from the baked `/AP`.
///
/// `annot_author::spec_from_dict` is the **author's** view, and it exists for
/// exactly this — *"so an existing annotation can be restyled by regenerating
/// its appearance from its own declared geometry"*. Reading through it means
/// the values these controls show are the values `set_markup_style` will read
/// when it plans: one derivation, not two.
///
/// ★ Its refusals are absent values here rather than an error, and that is
/// honest rather than lax. `SpecReadError`'s own doc says every variant is *"a
/// refusal to guess"* — geometry that is missing, or is not something pdfcer
/// models. A mark like that can still be **given** a colour; what cannot be
/// done is show the one it has.
#[derive(Debug, Clone, Copy)]
struct Current {
    /// ★★★ **Which of these properties this `/Subtype` can take at all — the
    /// ENGINE's answer, not this module's.**
    ///
    /// `MarkupStyleSupport::for_subtype` (`pdfcer-core` `edit.rs:4493`) is
    /// asked once, off the annotation's own `/Subtype`, and its three booleans
    /// are what [`fill`], [`width`] and [`endings`] consult before drawing
    /// anything. Until 2026-09-06 the same three answers were derived here from
    /// `MarkupSpec`'s arms, which is the copy of the engine's list this shell
    /// filed a request to be rid of. See the module header.
    ///
    /// ★ Distinct from a value being `None`, and the distinction is the whole
    /// of why a control is **absent** rather than greyed: `false` means the
    /// property is *"a property of the shape and not an error"*, so a control
    /// offered there would be inert — which this project forbids — and a greyed
    /// one would imply that an arrow could be filled if only something were
    /// different. Nothing is.
    support: MarkupStyleSupport,
    /// `/C` as sRGB, if it is a colour a swatch can show without converting.
    stroke: Option<[u8; 3]>,
    /// `/IC` as sRGB, if it is set and is showable.
    interior: Option<[u8; 3]>,
    /// Whether `/IC` is set at all — showable or not.
    ///
    /// ★ Tracked apart from [`Self::interior`] because a CMYK fill is *set* and
    /// *unshowable*, and the two questions have different answers: the swatch
    /// falls back to its default, and *No fill* must still be offered, because
    /// there is genuinely something to remove.
    interior_set: bool,
    /// `/BS` `/W`, the border width in points.
    width: Option<f64>,
    /// `/CA`, the constant opacity, `0.0`–`1.0`.
    alpha: Option<f64>,
    /// **`/BS` `/S` and `/D`, as the Line style chooser can show them.**
    ///
    /// ★ Read off the **dictionary**, not off the spec, and that is not a
    /// departure from this struct's rule — it is the same exception `/CA` is,
    /// one property along. A dash cuts across `MarkupSpec`'s variants rather
    /// than belonging to any of them, so the engine carries it in
    /// `AppearanceOptions` beside the spec instead of inside it
    /// (`pdfcer-core` `annot_author.rs:1633-1673`), and `spec_from_dict`
    /// therefore does not return one. The engine's own reader is `pub(crate)`
    /// (`annot_author.rs:840`), so
    /// [`crate::canvas::markup::linestyle::read`] is this shell's copy of it —
    /// with the copy declared as a copy in that function's header, and the
    /// bound on what a divergence can cost written down beside it.
    dash: crate::canvas::markup::linestyle::DashReading,
    /// The `/LE` pair the mark currently draws, when [`Self::support`] says it
    /// has one.
    ///
    /// ★ This is a **value**, and it comes from `MarkupSpec::Line`'s own field
    /// because only that arm has one — a fact the compiler checks and the
    /// engine publishes no API for. Whether the control is offered at all is
    /// `support.takes_endings`, which is a different question with a different
    /// owner. The header's table draws the line.
    endings: Option<(LineEnding, LineEnding)>,
    /// ★★ **Whether `/LE` is actually IN the dictionary**, as opposed to being
    /// supplied by Table 176's default on the way through `spec_from_dict`.
    ///
    /// The one thing [`Self::endings`] cannot tell anybody: the spec reader
    /// hands back `(None, None)` for a `/Line` with no `/LE` and for a `/Line`
    /// carrying `/LE [/None /None]`, because those two draw the same picture —
    /// which is correct for a reader whose job is the picture, and useless to a
    /// control whose whole subject is the difference. So the key is looked for
    /// on the dictionary itself.
    ///
    /// It is what gates the *Clear the setting* action: absent when there is
    /// nothing to remove, which is [`fill`]'s rule and the panel's.
    endings_key_present: bool,
}

/// ★ Even "nothing to show" asks the engine what the properties are.
///
/// `MarkupStyleSupport` is `#[non_exhaustive]` and has no `Default`, so this
/// impl is written out — and that is a small piece of luck worth keeping,
/// because the honest default for *"the dictionary could not be read"* is
/// **not** a hand-written all-`false` literal. It is what the engine answers
/// for a subtype it does not recognise, which `for_subtype`'s own doc calls
/// *"the conservative direction"*. Asking for it costs one call and removes the
/// last place a `false` could have been written here by hand.
impl Default for Current {
    fn default() -> Self {
        Self {
            support: MarkupStyleSupport::for_subtype(b""),
            stroke: None,
            interior: None,
            interior_set: false,
            width: None,
            alpha: None,
            // Solid, which is what `linestyle::read` answers for an annotation
            // with no `/BS` at all — so an unreadable dictionary and a plainly
            // solid one produce the same chooser, and neither invents a dash.
            dash: crate::canvas::markup::linestyle::DashReading::Solid,
            endings: None,
            endings_key_present: false,
        }
    }
}

impl Current {
    /// Read it out of the session, this frame.
    fn read(doc: &OpenDoc, id: pdfcer_core::object::ObjId) -> Self {
        use pdfcer_core::annot_author::{MarkupSpec, spec_from_dict};
        use pdfcer_core::graph::ObjectGraph;
        use pdfcer_core::object::Object;

        let graph = doc.session.graph();
        let Some(Object::Dict(dict)) = doc.session.value(id) else {
            return Self::default();
        };
        // ★ `/CA` straight off the dictionary rather than through the spec: it
        // is not part of `MarkupSpec` at all — the engine composites the
        // annotation onto the page rather than letting the appearance draw it,
        // which is why `set_markup_style` applies it to the dictionary
        // directly.
        //
        // ★ `ObjectGraph::resolve` comes from the TRAIT, so it has to be in
        // scope; there is no inherent method, and reaching for one is the error
        // a reader hits first. Following the reference through the session's
        // overlay rather than through the base file is what makes an unsaved
        // edit visible here.
        let alpha = dict
            .get(b"CA")
            .map(|o| graph.resolve(o))
            .and_then(Object::as_number);

        // ★★★ **The capability question, asked of the engine, off the same key
        // the engine itself reads.** `set_markup_style` derives its
        // `MarkupStyleSupport` from `/Subtype` on the annotation dictionary
        // (`edit.rs:26453`–`26460`); so does this. One key, one function, one
        // answer — which is what makes a control shown here and a call refused
        // there impossible to disagree.
        //
        // ★ Read through `graph.resolve` for the same reason `/CA` is: an
        // indirect `/Subtype` is legal, and following it through the session's
        // overlay is what makes an unsaved edit visible.
        let subtype = dict
            .get(b"Subtype")
            .map(|o| graph.resolve(o))
            .and_then(Object::as_name)
            .map_or_else(Vec::new, |n| n.as_bytes().to_vec());
        let support = MarkupStyleSupport::for_subtype(&subtype);

        // ★★ Presence, not value. See `Self::endings_key_present`: this is the
        // one fact `spec_from_dict` deliberately erases, because Table 176's
        // default makes an absent `/LE` and a written `[/None /None]` the same
        // picture — and the *Clear the setting* action exists precisely to tell
        // them apart.
        let endings_key_present = dict
            .get(b"LE")
            .map(|o| graph.resolve(o))
            .is_some_and(|o| !matches!(o, Object::Null));

        // ★★ Read BEFORE `spec_from_dict`, and carried across its refusal —
        // deliberately, and unlike `endings`. `/BS` is a dictionary key that
        // reads fine off an annotation whose *geometry* pdfcer cannot model, and
        // the Line style chooser can commit on such a mark for the same reason
        // the colour swatch can: `set_markup_style` refuses on the spec read, so
        // if the spec is unreadable nothing here commits and the value shown is
        // the only thing at stake. Showing the true one costs nothing.
        let dash = crate::canvas::markup::linestyle::read(&graph, dict);

        let Ok(spec) = spec_from_dict(&graph, dict) else {
            return Self {
                support,
                alpha,
                dash,
                endings_key_present,
                ..Self::default()
            };
        };
        let mut current = Self {
            support,
            alpha,
            dash,
            endings_key_present,
            ..Self::default()
        };
        // ★★★ **This `match` reads VALUES; it no longer decides CAPABILITIES.**
        //
        // Until 2026-09-06 the arms below also set `fillable`, and the absence
        // of a `width` assignment on the `TextMarkup` arm was how the width
        // control came to be hidden for a highlight. Both were this crate
        // holding a copy of the engine's list, which is the boundary defect
        // filed and now answered — `current.support` above carries all three
        // answers and the three control functions consult it.
        //
        // ⚠ What stays is what only an arm can say: `border_width` lives in
        // `MarkupSpec::Square`, `width` in `MarkupSpec::Line`, `endings` in
        // that one arm alone. Those are *this mark's* values, the compiler
        // checks which arm has which, and the engine publishes no API that
        // would answer them. A value read and a capability question are
        // different questions with different owners — see the header's table.
        match spec {
            MarkupSpec::Square {
                border,
                interior,
                border_width,
                ..
            }
            | MarkupSpec::Circle {
                border,
                interior,
                border_width,
                ..
            } => {
                current.stroke = border.and_then(rgb_of);
                current.interior_set = interior.is_some();
                current.interior = interior.and_then(rgb_of);
                current.width = Some(border_width);
            }
            MarkupSpec::Polygon {
                border,
                interior,
                width,
                ..
            }
            | MarkupSpec::Cloud {
                border,
                interior,
                width,
                ..
            } => {
                current.stroke = border.and_then(rgb_of);
                current.interior_set = interior.is_some();
                current.interior = interior.and_then(rgb_of);
                current.width = Some(width);
            }
            MarkupSpec::Line {
                color,
                width,
                endings,
                ..
            } => {
                current.stroke = rgb_of(color);
                current.width = Some(width);
                current.endings = Some(endings);
            }
            MarkupSpec::PolyLine { color, width, .. } | MarkupSpec::Ink { color, width, .. } => {
                current.stroke = rgb_of(color);
                current.width = Some(width);
            }
            // A text markup's shape is `/QuadPoints`, so there is no border
            // width in the arm to read. That it has no border to widen is
            // `support.takes_border`'s to say, not this arm's absence of an
            // assignment — which is the correction of 2026-09-06.
            MarkupSpec::TextMarkup { color, .. } => current.stroke = rgb_of(color),
            // `MarkupSpec` is `#[non_exhaustive]`. A kind this build does not
            // know the shape of gets no readback, which is the same answer a
            // refused parse gets and for the same reason: nothing is destroyed
            // by touching nothing.
            _ => {}
        }
        current
    }

    // -----------------------------------------------------------------------
    // ★★★ WHETHER a control is drawn — one question, one place, testable
    //
    // These four are the whole of what replaced this module's copy of the
    // engine's subtype list, and they are functions rather than expressions
    // inlined at the three call sites for two reasons.
    //
    // 1. **A control's visibility rule is a decision and decisions get
    //    asserted.** The three control functions take a `Ui` and can only be
    //    exercised by driving the binary; these take nothing and are reachable
    //    from a unit test, which is what lets `the_engines_answer_is_what_hides
    //    _a_control_not_the_spec_arm` falsify the claim in both directions.
    // 2. **One place per question.** Two call sites spelling `takes_border &&
    //    width.is_some()` slightly differently is exactly the drift the whole
    //    request was about, one level down.
    // -----------------------------------------------------------------------

    /// Whether the Fill swatch draws at all.
    ///
    /// Purely the engine's answer: `/IC` needs no readback to be *offerable* —
    /// *no fill* is a legitimate current state and [`fill`] shows it as the
    /// swatch's white default with no Clear beside it.
    const fn offers_fill(self) -> bool {
        self.support.takes_interior
    }

    /// Whether the width field draws.
    ///
    /// Both terms, and they mean different things — see [`width`]'s doc. The
    /// first is the engine's (*this subtype has no border*), the second is this
    /// build's (*this arm's width is not one I can read*).
    const fn offers_width(self) -> bool {
        self.support.takes_border && self.width.is_some()
    }

    /// Whether the Line style chooser draws.
    ///
    /// ★★ **Purely the engine's answer, with no second term** — unlike
    /// [`Self::offers_width`], which also asks whether a width was read. The
    /// asymmetry is real rather than an oversight: a width has to be *shown* in
    /// its field, so a mark whose width this build could not read has nothing to
    /// put in one; a line style always has a value, because *solid* is a state
    /// and not an absence, and [`crate::canvas::markup::linestyle::read`] is
    /// total — every dictionary answers it, including one with no `/BS` at all.
    ///
    /// ⇒ So the only question left is the engine's *does this subtype have a
    /// border?*, and `takes_border` is it. That is the same predicate
    /// `set_markup_style` guards `style.dash` with
    /// (`pdfcer-core` `edit.rs:26463-26476`), so a chooser drawn here cannot
    /// produce the `StylePropertyNotApplicable` refusal — which is the belt this
    /// module's header describes, with the engine's braces behind it.
    const fn offers_dash(self) -> bool {
        self.support.takes_border
    }

    /// Whether the arrowhead chooser draws.
    const fn offers_endings(self) -> bool {
        self.support.takes_endings && self.endings.is_some()
    }

    /// Whether the chooser's *Clear the setting* action draws under its
    /// separator.
    ///
    /// ★ Strictly narrower than [`Self::offers_endings`]: there has to be a
    /// chooser to put it in **and** a `/LE` in the file to take out.
    const fn offers_endings_clear(self) -> bool {
        self.offers_endings() && self.endings_key_present
    }
}

/// The outline colour, `/C`.
///
/// # ★ A swatch alone, where the Properties panel has a swatch and a Clear
///
/// `StyleEdit` has two arms and they mean different things in the file: `Set`
/// writes `/C` and `Clear` removes it, restoring the standard's default. The
/// panel offers both, and §5.8's rule is that *"the tab's contents are a
/// **subset**"* of the panel's — so dropping one is permitted where adding one
/// would not be.
///
/// It is dropped rather than kept because a `/C` an operator wants gone is a
/// deliberate, rare act, and the ribbon is the surface for the frequent one:
/// two controls per colour would take two of the group's five slots for the
/// stroke alone. **Fill is the exception**, and [`fill`] argues why — *no fill*
/// is not a rare act there, it is the state every mark starts in.
fn stroke(
    ui: &mut Ui,
    current: Current,
    target: &AnnotTarget,
    parked: &mut Option<(AnnotTarget, MarkupEdit)>,
) -> bool {
    // ★ The default is BLACK rather than an invented colour, and it is the same
    // default `panels::properties::markup` shows. A mark whose `/C` is CMYK, a
    // separation, or absent gets it — see `rgb_of` for why a converted
    // near-match would be worse than a default: pick the swatch up, put it down
    // unchanged, and the file now says something different.
    let existing = current.stroke;
    let mut rgb = existing.unwrap_or([0, 0, 0]);
    if ui.color_edit_button_srgb(&mut rgb).changed() && Some(rgb) != existing {
        *parked = Some((
            target.clone(),
            MarkupEdit::Stroke(StyleEdit::Set(srgb_to_colour(rgb))),
        ));
        return true;
    }
    false
}

/// The interior colour, `/IC`, and its removal.
///
/// # ★★★ *No fill* is a first-class state, and it is why this control is two
/// widgets where [`stroke`] is one
///
/// `canvas::markup::spec` authors every shape with `interior: None`, and its
/// reason is quoted in `panels::properties::markup`'s header: *"a filled
/// comment shape hides the drawing it is a comment about, which on a CAD sheet
/// is the whole content under it."* Acrobat's default is the same for every
/// shape.
///
/// ⇒ So **no fill is where every mark starts**, and a control that could only
/// ever set one would be a one-way door: try a fill on a drawing, decide against
/// it, and there is no way back to the mark you had. `StyleEdit::Clear` is the
/// way back and [`crate::text::ribbon::markup_no_fill`] is what it is called.
///
/// ⚠ **This does not change what NEW markup is authored with.** The pen is
/// `canvas::markup::pen`'s and is untouched; `NO_SURFACE.md` records that
/// reversing the authoring default is the operator's call, and offering a
/// restyle control does not make it.
///
/// ★ The clear button is **absent** when there is nothing to clear rather than
/// greyed — the panel's rule, and its reason: a Clear beside a mark that has no
/// `/IC` is a control whose only possible effect is an undo entry the operator
/// did not earn.
///
/// ★ The whole control is absent for a subtype with no interior, **and the
/// engine says which those are** — `MarkupStyleSupport::takes_interior`, via
/// [`Current::support`]. It was a `fillable` flag this module set from
/// `MarkupSpec`'s arms until 2026-09-06; see the header.
fn fill(
    ui: &mut Ui,
    current: Current,
    target: &AnnotTarget,
    parked: &mut Option<(AnnotTarget, MarkupEdit)>,
) -> bool {
    if !current.offers_fill() {
        return false;
    }
    let existing = current.interior;
    // ★ WHITE, not black, and it is the one default in this module that differs
    // from the Properties panel's. A fill swatch that opens on black offers, as
    // its most likely single click, the colour that hides the most of the
    // drawing underneath — which is the exact outcome pdfcer authors
    // `interior: None` to avoid. White is the least destructive first guess and
    // is what a shape tool defaults to in every drawing program.
    let mut rgb = existing.unwrap_or([255, 255, 255]);
    let mut invoked = false;
    ui.horizontal(|ui| {
        if ui.color_edit_button_srgb(&mut rgb).changed() && Some(rgb) != existing {
            *parked = Some((
                target.clone(),
                MarkupEdit::Interior(StyleEdit::Set(srgb_to_colour(rgb))),
            ));
            invoked = true;
        }
        if current.interior_set && ui.button(r::markup_no_fill()).clicked() {
            *parked = Some((target.clone(), MarkupEdit::Interior(StyleEdit::Clear)));
            invoked = true;
        }
    });
    invoked
}

/// The border width, `/BS` `/W`.
///
/// ⚠ **This moves `/Rect` for every subtype except `Square` and `Circle`** —
/// the rectangle is derived from the geometry plus a margin that contains the
/// stroke and any arrowheads, so a wider pen needs a bigger box. That is the
/// engine's own ⚠ and it is disclosed in the command's tooltip
/// (`text::commands::markupstyle::format_line_width`) rather than here, because
/// a ribbon band has no room for a sentence and the hover is the surface that
/// does.
///
/// ★ Committed on `drag_stopped` or `lost_focus`, **never** on `.changed()`. A
/// `DragValue` reports a change on every pixel of a drag, and each one here
/// regenerates the appearance and is one undo entry — so a single drag across
/// the control would leave forty entries on a `Ctrl+Z` stack the operator could
/// not get back through, and would re-plan the annotation forty times.
///
/// ★ **Absent** rather than greyed when the mark has no border to widen: a
/// highlight is `/QuadPoints` and has nothing to stroke. R9, and the same
/// answer `panels::properties::markup::width_row` gives.
///
/// ★★ **Which subtypes those are is `MarkupStyleSupport::takes_border`'s to
/// say.** This function used to answer it by whether the `MarkupSpec` arm had
/// handed over a width — correct today, wrong the first day the engine gives a
/// text markup a border, and wrong in the silent direction. The value read is
/// still the arm's; the capability question is the engine's.
///
/// ⇒ [`Current::offers_width`] therefore carries two terms and both are needed.
/// `takes_border` false means *this subtype has no border* — nothing renders,
/// forever. A `None` width under a `takes_border` that is true means *this
/// build cannot read this arm's width*, which `MarkupSpec` being
/// `#[non_exhaustive]` makes possible: a control drawn there would have no
/// value to show, which `placeholder`'s own argument forbids. The `let else`
/// below is the extraction, not a third guard.
fn width(
    ui: &mut Ui,
    current: Current,
    target: &AnnotTarget,
    parked: &mut Option<(AnnotTarget, MarkupEdit)>,
) -> bool {
    if !current.offers_width() {
        return false;
    }
    let Some(was) = current.width else {
        return false;
    };
    // ★ The draft, not the document — see `drafted`. Re-seeding from `was`
    // every frame is what made this control undraggable for its whole life.
    let draft_id = ui.id().with("markup.width.draft");
    let mut value = drafted::<f64>(ui, draft_id, was);
    let response = ui.add(
        egui::DragValue::new(&mut value)
            .range(MIN_WIDTH_PT..=MAX_WIDTH_PT)
            .speed(0.1)
            .suffix(t::markup_width_suffix())
            .max_decimals(2),
    );
    let _ = ui.allocate_space(egui::Vec2::new(
        (FIELD_WIDTH - response.rect.width()).max(0.0),
        0.0,
    ));
    let ended = keep_draft(ui, draft_id, &response, value);
    if ended && (value - was).abs() > f64::EPSILON {
        *parked = Some((target.clone(), MarkupEdit::Width(value)));
        return true;
    }
    false
}

/// The constant opacity, `/CA`.
///
/// ★ Shown as a **percentage**, because that is the unit every other
/// application an operator has used states opacity in, and `/CA`'s own
/// `0.0..=1.0` is a file-format detail they should never meet. The Properties
/// panel's twin makes the same choice and this reads the same suffix from the
/// same catalog entry, so the two surfaces cannot come to call it different
/// things.
///
/// ★ Release-not-change, for [`width`]'s reason exactly.
///
/// ★ No Clear. `/CA` absent and `/CA` at 100 % render identically, so removing
/// the key is a change with no visible consequence — and the panel, which has
/// room for a control whose effect is invisible, is where that belongs. Here it
/// would spend a slot on a button an operator could not tell had worked.
fn opacity(
    ui: &mut Ui,
    current: Current,
    target: &AnnotTarget,
    parked: &mut Option<(AnnotTarget, MarkupEdit)>,
) -> bool {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let was = (current.alpha.unwrap_or(1.0) * 100.0)
        .round()
        .clamp(0.0, 100.0) as u8;
    // ★ Same draft as `width`, and it had the same defect — the driven check
    // named width and flagged this one as "built the same way and worth
    // checking". It was. Both are fixed by the same two calls.
    let draft_id = ui.id().with("markup.opacity.draft");
    let mut percent = drafted::<u8>(ui, draft_id, was);
    let response = ui.add(
        egui::DragValue::new(&mut percent)
            .range(0..=100)
            .speed(1.0)
            .suffix(t::markup_opacity_suffix()),
    );
    let _ = ui.allocate_space(egui::Vec2::new(
        (FIELD_WIDTH - response.rect.width()).max(0.0),
        0.0,
    ));
    let ended = keep_draft(ui, draft_id, &response, percent);
    if ended && percent != was {
        *parked = Some((
            target.clone(),
            MarkupEdit::Opacity(StyleEdit::Set(f64::from(percent) / 100.0)),
        ));
        return true;
    }
    false
}

/// Which ends of a `/Line` carry an arrowhead.
///
/// **Absent for every other subtype**, because nothing else has ends to put one
/// on — and **the engine is what says which those are**:
/// `MarkupStyleSupport::takes_endings`, via [`Current::support`]. It was
/// `Current::endings` being `Some` that decided it until 2026-09-06, which was
/// this crate answering a question the engine owns; see the header. It costs no
/// space — `egui_shell::ribbon::control` reserves the budgeted width only when
/// the application supplies **no renderer at all**.
///
/// # ★★★ Four positions, and the SHAPE is preserved rather than chosen
///
/// `/LE` is two independent endings over three shapes each (§12.5.6.7, Table
/// 176) — nine combinations, which is not a list anybody reads on a ribbon
/// band. This offers the four *positions* an operator means and carries the
/// mark's existing arrowhead shape through unchanged, so a closed arrowhead
/// stays closed and an open one stays open.
///
/// ⇒ That is the difference between a control that answers the question asked
/// (*which ends?*) and one that quietly answers a second question nobody asked
/// (*and what shape?*). A chooser that normalised every arrow to `/OpenArrow`
/// would silently rewrite a `/ClosedArrow` the operator's producer had set, and
/// the change would be visible in another viewer.
///
/// # ★★★ …and a fifth state that is not a fifth position
///
/// The four positions all **write** `/LE`. `StyleEdit::Clear` **removes** it,
/// which is a different file drawing the same line, and it is offered here as
/// an **action below a separator** rather than as a fifth entry in the list.
/// The header carries the argument in full; the two sentences that matter are
/// that a fifth entry drawing identically to the first is a distinction a
/// drafter cannot check by looking, and that it would leave the combo's
/// `selected_text` with two equal claimants when `/LE` is absent.
///
/// ★ It is **absent unless `/LE` is in the dictionary**
/// ([`Current::endings_key_present`]), which is [`fill`]'s Clear rule: a
/// removal offered where there is nothing to remove has no possible effect but
/// an undo entry the operator did not earn.
/// **The border line style, `/BS` `/S` and `/D` — the eighth control.**
///
/// # ★★★ What this closes
///
/// `RIBBON_IA.md` §5.8's *Line style* row read **⛔ no engine verb exists** and
/// was the only entry in that row's eight for which that was true. It stopped
/// being true on the afternoon of 2026-09-06, when `pdfcer-core` answered this
/// shell's request with all three halves of the dash rather than the one that
/// was asked for — preserve, author, and restyle. This is the restyle half's
/// ribbon control; [`crate::canvas::markup::swatch`] is the author half's.
///
/// # ★★ The preserve half is why this control is SAFE, and it is why it
/// # shipped at all
///
/// Before that Pass, a dashed mark in the operator's file was **silently
/// converted to a solid one the first time anything about it changed** — and
/// the engine's reply records that the defect was wider than this shell
/// reported it: the recolour path was named, and `resize_annotation`,
/// `reshape_annotation` and authoring solidified a dash too. So dragging a
/// resize handle or a vertex destroyed it, not only pressing the colour swatch.
/// All four carry it now.
///
/// ⇒ That is the precondition a *Line style* control needs. Offering one over
/// an engine that dropped every dash it did not author would have been a control
/// whose neighbours undid it.
///
/// # ★ There is no Clear beside it, and the reason is Table 166
///
/// The other two `StyleEdit` controls in this group put `Clear` on its own
/// button — `fill`'s *No fill*, `endings`' *Clear the setting* — because in both
/// cases the cleared state is the **absence of a key** and has no name in the
/// list. Here it does: `Clear` makes the border solid, and *Solid* is Table
/// 166's own `/S` and the chooser's first entry. A separate button would be a
/// second spelling of one act, and one of the two would eventually be pressed
/// expecting something different from the other.
///
/// # Absent for a subtype with no border
///
/// [`Current::offers_dash`], which is `MarkupStyleSupport::takes_border` and
/// nothing else. R9, and the same answer [`width`] gives for a highlight.
fn dash(
    ui: &mut Ui,
    current: Current,
    target: &AnnotTarget,
    parked: &mut Option<(AnnotTarget, MarkupEdit)>,
) -> bool {
    if !current.offers_dash() {
        return false;
    }
    let Some(picked) = crate::canvas::markup::linestyle::chooser(
        ui,
        // ui-text-exempt: internal widget id, never displayed
        "ribbon-format-markup-dash",
        current.dash,
        DASH_WIDTH,
    ) else {
        return false;
    };
    // ★ The one place `BorderDash::new`'s `Option` is answered on this surface,
    // and the answer is to **do nothing**: no park, no token, no undo entry.
    // Substituting Table 166's default would be this shell writing a pattern the
    // operator did not choose. It is unreachable for the four offered styles —
    // `linestyle::tests::every_offered_pattern_is_one_the_engine_accepts` is
    // what makes that a fact rather than a hope — and it is expressed anyway,
    // because the alternative shape is an `expect` in a paint loop.
    let Some(edit) = picked.style_edit() else {
        return false;
    };
    *parked = Some((target.clone(), MarkupEdit::Dash(edit)));
    true
}

fn endings(
    ui: &mut Ui,
    current: Current,
    target: &AnnotTarget,
    parked: &mut Option<(AnnotTarget, MarkupEdit)>,
) -> bool {
    if !current.offers_endings() {
        return false;
    }
    let Some(pair) = current.endings else {
        return false;
    };
    let was = Ends::of(pair);
    let shape = arrow_shape(pair);
    let mut invoked = false;
    egui::ComboBox::from_id_salt("ribbon-format-markup-endings")
        .width(ENDINGS_WIDTH)
        .selected_text(was.label())
        .show_ui(ui, |ui| {
            for &option in Ends::ALL {
                if ui.selectable_label(option == was, option.label()).clicked() && option != was {
                    *parked = Some((
                        target.clone(),
                        MarkupEdit::Endings(StyleEdit::Set(option.applied(shape))),
                    ));
                    invoked = true;
                }
            }
            // ★ The separator is the whole of the presentation decision: what
            // is above it are four answers to *which ends?*, and what is below
            // it is an act on the file. A `Button` rather than a
            // `selectable_label`, so it cannot render as a selected position.
            if current.offers_endings_clear() {
                ui.separator();
                if ui
                    .button(t::markup_endings_clear())
                    .on_hover_text(t::markup_endings_clear_hint())
                    .clicked()
                {
                    *parked = Some((target.clone(), MarkupEdit::Endings(StyleEdit::Clear)));
                    invoked = true;
                }
            }
        });
    invoked
}

/// Which ends of a line carry an arrowhead.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ends {
    /// A plain line.
    None,
    /// A head where the drag began.
    Start,
    /// A head where the drag finished — what `canvas::markup` authors.
    End,
    /// A head at each end.
    Both,
}

impl Ends {
    /// In the order the chooser draws them: fewest endings first, which is also
    /// increasing commitment and is the reading order §5.8's menu rule gives a
    /// group.
    const ALL: &'static [Self] = &[Self::None, Self::Start, Self::End, Self::Both];

    /// Which positions a `/LE` pair occupies, ignoring the shape.
    fn of(pair: (LineEnding, LineEnding)) -> Self {
        match (pair.0 != LineEnding::None, pair.1 != LineEnding::None) {
            (false, false) => Self::None,
            (true, false) => Self::Start,
            (false, true) => Self::End,
            (true, true) => Self::Both,
        }
    }

    /// The `/LE` pair for these positions, drawn in `shape`.
    fn applied(self, shape: LineEnding) -> (LineEnding, LineEnding) {
        let n = LineEnding::None;
        match self {
            Self::None => (n, n),
            Self::Start => (shape, n),
            Self::End => (n, shape),
            Self::Both => (shape, shape),
        }
    }

    /// What the operator reads.
    fn label(self) -> &'static str {
        match self {
            Self::None => r::markup_endings_none(),
            Self::Start => r::markup_endings_start(),
            Self::End => r::markup_endings_end(),
            Self::Both => r::markup_endings_both(),
        }
    }
}

/// The arrowhead **shape** a `/LE` pair is drawn in, to be carried through a
/// change of position.
///
/// ★ `ClosedArrow` wins when the two ends disagree, and `OpenArrow` is the
/// answer for a line that has no head at all. Both choices are about not
/// destroying information: a mark with one closed head is a mark whose author
/// chose closed, so adding a second head should match it rather than convert
/// it; and `OpenArrow` is what pdfcer's own pen authors and what
/// `LineEnding::OpenArrow`'s doc records as *"Acrobat's default at both ends"*,
/// so a plain line given its first head gets the head this program draws.
fn arrow_shape(pair: (LineEnding, LineEnding)) -> LineEnding {
    if pair.0 == LineEnding::ClosedArrow || pair.1 == LineEnding::ClosedArrow {
        LineEnding::ClosedArrow
    } else {
        LineEnding::OpenArrow
    }
}

/// An annotation colour as sRGB bytes, if it is one a swatch can show.
///
/// ★ `None` for anything that is not RGB or grey, and that is honest rather
/// than lossy: §12.5.2 lets `/C` be a 0-, 1-, 3- or 4-component array, and a
/// swatch showing a CMYK mark's *converted* colour would be a control whose
/// readback is a conversion the operator never asked for — pick it up, put it
/// down unchanged, and the file now says something different, on a drawing
/// heading for a printer that cares.
///
/// Grey is included rather than refused because it is **lossless** in both
/// directions: `Gray(v)` and `Rgb(v, v, v)` are the same ink.
fn rgb_of(color: Color) -> Option<[u8; 3]> {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    match color {
        Color::Rgb(r, g, b) => Some([
            (r.clamp(0.0, 1.0) * 255.0).round() as u8,
            (g.clamp(0.0, 1.0) * 255.0).round() as u8,
            (b.clamp(0.0, 1.0) * 255.0).round() as u8,
        ]),
        Color::Gray(v) => {
            let g = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
            Some([g, g, g])
        }
        Color::Cmyk(..) => None,
    }
}

/// sRGB bytes as the engine's device colour.
///
/// ★ Always `DeviceRGB`, never a grey collapsed from three equal channels. The
/// swatch is an sRGB picker, so what the operator chose *is* an RGB triple; a
/// mark silently written as `DeviceGray` because its channels happened to match
/// would be pdfcer inferring a colour space the operator did not ask for, which
/// is Rule 4's whole subject.
fn srgb_to_colour(rgb: [u8; 3]) -> Color {
    Color::Rgb(
        f64::from(rgb[0]) / 255.0,
        f64::from(rgb[1]) / 255.0,
        f64::from(rgb[2]) / 255.0,
    )
}

/// **Holding a spinner's value across frames while it is being dragged.**
///
/// Split out of this file on 2026-09-06 under R2, in the same commit that fixed
/// the defect it exists for: two `DragValue`s here were re-seeded from the
/// document every frame, so a drag could never accumulate and neither control
/// could be dragged at all. Its header carries the whole finding.
mod draft;
use draft::{drafted, keep_draft};

#[cfg(test)]
mod tests;
