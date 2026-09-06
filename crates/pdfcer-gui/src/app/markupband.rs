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
//! twenty lines. The two are allowed to differ, and one of them does — this one
//! also reads `/LE` and the *interior*, which the panel offers no control for.
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
use pdfcer_core::edit::{MarkupStyle, StyleEdit};

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
#[derive(Debug, Clone, Copy, PartialEq)]
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
    /// `/LE` — the pair of line endings. `/Line` only.
    Endings((LineEnding, LineEnding)),
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
            Self::Endings(pair) => MarkupStyle {
                endings: Some(pair),
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
#[derive(Debug, Clone, Copy, Default)]
struct Current {
    /// `/C` as sRGB, if it is a colour a swatch can show without converting.
    stroke: Option<[u8; 3]>,
    /// Whether `/IC` means anything for this subtype at all.
    ///
    /// ★ Distinct from `interior` being `None`, and the distinction is the
    /// whole of why the fill control is **absent** on an arrow rather than
    /// greyed: `MarkupStyle::interior` says the subtypes with no interior
    /// *ignore* it, which is *"a property of the shape and not an error"* — so
    /// a control offered there would be an inert control, which this project
    /// forbids, and a greyed one would imply that an arrow could be filled if
    /// only something were different. Nothing is.
    fillable: bool,
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
    /// `/LE`. `Some` for a `/Line` and nothing else.
    endings: Option<(LineEnding, LineEnding)>,
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

        let Ok(spec) = spec_from_dict(&graph, dict) else {
            return Self {
                alpha,
                ..Self::default()
            };
        };
        let mut current = Self {
            alpha,
            ..Self::default()
        };
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
                current.fillable = true;
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
                current.fillable = true;
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
            // ★ A text markup has a colour and no border at all — its shape is
            // `/QuadPoints` and there is nothing to stroke. The width control
            // is therefore ABSENT for one, which is R9: an unavailable
            // capability renders nothing, and a greyed spinner here would be
            // pdfcer implying that a highlight could have a line width if only
            // something were different.
            MarkupSpec::TextMarkup { color, .. } => current.stroke = rgb_of(color),
            // `MarkupSpec` is `#[non_exhaustive]`. A kind this build does not
            // know the shape of gets no readback, which is the same answer a
            // refused parse gets and for the same reason: nothing is destroyed
            // by touching nothing.
            _ => {}
        }
        current
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
/// ★ The whole control is absent for a subtype with no interior. See
/// [`Current::fillable`].
fn fill(
    ui: &mut Ui,
    current: Current,
    target: &AnnotTarget,
    parked: &mut Option<(AnnotTarget, MarkupEdit)>,
) -> bool {
    if !current.fillable {
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
fn width(
    ui: &mut Ui,
    current: Current,
    target: &AnnotTarget,
    parked: &mut Option<(AnnotTarget, MarkupEdit)>,
) -> bool {
    let Some(was) = current.width else {
        return false;
    };
    let mut value = was;
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
    if (response.drag_stopped() || response.lost_focus()) && (value - was).abs() > f64::EPSILON {
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
    let mut percent = was;
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
    if (response.drag_stopped() || response.lost_focus()) && percent != was {
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
/// on. The control decides that itself, from [`Current::endings`], rather than
/// from a published condition: the same shape
/// `panels::properties::markup::width_row` uses for a highlight, and the reason
/// is that a condition would put one question in two places and let them
/// disagree. It costs no space — `egui_shell::ribbon::control` reserves the
/// budgeted width only when the application supplies **no renderer at all**.
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
fn endings(
    ui: &mut Ui,
    current: Current,
    target: &AnnotTarget,
    parked: &mut Option<(AnnotTarget, MarkupEdit)>,
) -> bool {
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
                    *parked = Some((target.clone(), MarkupEdit::Endings(option.applied(shape))));
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

#[cfg(test)]
mod tests {
    use super::*;

    /// ★★★ **Every custom kind the manifest declares for this group is drawn
    /// here, and every kind drawn here backs a registered command.**
    ///
    /// The assertion that closes the gap `manifest::COLOUR_SWATCH`'s own doc
    /// comment records: the manifest wrote a custom kind, **no renderer ever
    /// matched it**, and the Markup ▸ Style group drew a caption over an empty
    /// band for the whole of v0.1.0 with nothing anywhere reporting the
    /// mismatch. The shell reserves the item's space, the application declines
    /// to draw, and the only symptom is a gap.
    ///
    /// It is asserted through `manifest::CUSTOM_BACKED`, which already pairs a
    /// command id with the kind that draws it and is already tested against the
    /// manifest in both directions. Reading it here makes the chain complete:
    /// manifest → register → renderer → registry.
    #[test]
    fn every_markup_kind_in_the_register_is_drawn_by_this_module() {
        let mut registry = egui_shell::commands::CommandRegistry::new();
        crate::shell::commands::register(&mut registry);
        for (id, kind, _) in crate::shell::manifest::CUSTOM_BACKED {
            let Some(mapped) = command_for(kind) else {
                // Not this module's kind — `file.recent` and the three Font
                // controls are the other entries.
                continue;
            };
            assert_eq!(
                mapped, *id,
                "`{kind}` is registered as backing `{id}` and this module draws it for `{mapped}`"
            );
            assert!(
                registry.get(id).is_some(),
                "`{id}` is drawn by this module and is not in the registry, so the control would \
                 silently vanish"
            );
        }
    }

    /// The five kinds this module claims are exactly the five the manifest
    /// declares — asserted as an **exact set**, not as five `contains`.
    ///
    /// ★ A sixth kind added here and not to the manifest is a renderer arm
    /// nothing can ever reach; a sixth added to the manifest and not here is the
    /// empty-band defect above. Only an equality catches both.
    ///
    /// ★★ It also asserts that this module does **not** claim the Font group's
    /// three or the Markup tab's pen swatch. `COLOUR_SWATCH` is the one that
    /// matters: it is a colour control called `colour_swatch` that sits two tabs
    /// away and means the opposite thing about *when* — it chooses the colour of
    /// the mark you are about to draw, where `MARKUP_STROKE` restyles the mark
    /// you have selected. Claiming it here would put a document-editing verb
    /// behind a control that edits `PdfcerApp::pen`.
    #[test]
    fn this_module_draws_exactly_the_five_markup_kinds() {
        use crate::shell::manifest::{
            COLOUR_SWATCH, FONT_COLOUR, FONT_FACE, FONT_SIZE, MARKUP_ENDINGS, MARKUP_FILL,
            MARKUP_OPACITY, MARKUP_STROKE, MARKUP_WIDTH,
        };
        let mine: Vec<&str> = [
            MARKUP_STROKE,
            MARKUP_FILL,
            MARKUP_WIDTH,
            MARKUP_OPACITY,
            MARKUP_ENDINGS,
        ]
        .into_iter()
        .filter(|k| command_for(k).is_some())
        .collect();
        assert_eq!(
            mine,
            [
                MARKUP_STROKE,
                MARKUP_FILL,
                MARKUP_WIDTH,
                MARKUP_OPACITY,
                MARKUP_ENDINGS
            ]
        );
        for foreign in [COLOUR_SWATCH, FONT_FACE, FONT_SIZE, FONT_COLOUR] {
            assert!(
                command_for(foreign).is_none(),
                "`{foreign}` is not a Format ▸ Markup control and must not be claimed here"
            );
        }
        assert!(command_for("nonsense").is_none());
    }

    /// ★★★ **Every parked edit sets exactly ONE field of `MarkupStyle`.**
    ///
    /// The rule `MarkupStyle`'s own doc states — *"a Format tab whose colour
    /// picker also had to restate the current width would overwrite whatever
    /// the operator had set from the other control"* — asserted rather than
    /// trusted, because the failure it prevents has no symptom the operator
    /// could report: a width silently reverting one frame after it was set
    /// reads as *the drag did not take*.
    ///
    /// ★ Counted by comparing against `MarkupStyle::default()` field by field,
    /// which is the only way to state "exactly one" over a struct of `Option`s.
    #[test]
    fn only_one_field_is_ever_set() {
        let cases = [
            MarkupEdit::Stroke(StyleEdit::Set(Color::Rgb(1.0, 0.0, 0.0))),
            MarkupEdit::Stroke(StyleEdit::Clear),
            MarkupEdit::Interior(StyleEdit::Set(Color::Gray(0.5))),
            MarkupEdit::Interior(StyleEdit::Clear),
            MarkupEdit::Width(2.5),
            MarkupEdit::Opacity(StyleEdit::Set(0.5)),
            MarkupEdit::Opacity(StyleEdit::Clear),
            MarkupEdit::Endings((LineEnding::None, LineEnding::OpenArrow)),
        ];
        for case in cases {
            let style = case.into_style();
            let set = usize::from(style.stroke.is_some())
                + usize::from(style.interior.is_some())
                + usize::from(style.width.is_some())
                + usize::from(style.opacity.is_some())
                + usize::from(style.endings.is_some());
            assert_eq!(
                set, 1,
                "`{case:?}` sets {set} fields of MarkupStyle; it must set exactly one"
            );
            assert!(!style.is_empty(), "`{case:?}` produced a no-op override");
        }
    }

    /// ★★ **The arrowhead chooser changes WHICH ends, never WHAT SHAPE.**
    ///
    /// The property that makes a four-entry list honest over a nine-value
    /// field: a mark drawn with closed arrowheads keeps them when the operator
    /// moves the head to the other end. Without it the chooser would answer a
    /// question nobody asked, and the rewrite would be invisible here and
    /// visible in another viewer.
    #[test]
    fn changing_which_ends_preserves_the_arrowhead_shape() {
        use LineEnding::{ClosedArrow, None as NoEnd, OpenArrow};
        // A closed head at the end, moved to both ends: still closed.
        let pair = (NoEnd, ClosedArrow);
        assert_eq!(Ends::of(pair), Ends::End);
        assert_eq!(
            Ends::Both.applied(arrow_shape(pair)),
            (ClosedArrow, ClosedArrow)
        );
        // An open head, moved to the start: still open.
        let pair = (NoEnd, OpenArrow);
        assert_eq!(Ends::of(pair), Ends::End);
        assert_eq!(Ends::Start.applied(arrow_shape(pair)), (OpenArrow, NoEnd));
        // A plain line given its first head gets the one pdfcer's pen authors.
        let pair = (NoEnd, NoEnd);
        assert_eq!(Ends::of(pair), Ends::None);
        assert_eq!(Ends::End.applied(arrow_shape(pair)), (NoEnd, OpenArrow));
        // Every position round-trips through `of`, whichever shape it is in.
        for shape in [OpenArrow, ClosedArrow] {
            for &ends in Ends::ALL {
                assert_eq!(Ends::of(ends.applied(shape)), ends);
            }
        }
    }

    /// The four positions have four distinct, non-empty labels, in the
    /// documented order.
    ///
    /// ★ The **order** is the assertion worth making: the chooser lists them
    /// fewest-endings-first, so an operator scanning for "both" finds it last
    /// every time, and a reordering that put the common case first would be a
    /// change to the control's shape rather than to its wording.
    #[test]
    fn the_four_arrowhead_positions_are_named_and_ordered() {
        assert_eq!(
            Ends::ALL,
            [Ends::None, Ends::Start, Ends::End, Ends::Both].as_slice()
        );
        let mut labels: Vec<&str> = Ends::ALL.iter().map(|e| e.label()).collect();
        for label in &labels {
            assert!(!label.trim().is_empty());
        }
        let total = labels.len();
        labels.sort_unstable();
        labels.dedup();
        assert_eq!(labels.len(), total, "two positions share a label");
    }

    /// A swatch shows only colours it can show without converting, and an sRGB
    /// pick round-trips back out as `DeviceRGB`.
    ///
    /// ★ Grey is accepted **in** and never produced **out**, which is the
    /// asymmetry `srgb_to_colour` argues: reading `Gray(v)` as an equal-channel
    /// swatch is lossless, and writing an equal-channel pick back as `Gray`
    /// would be pdfcer choosing a colour space the operator did not ask for.
    #[test]
    fn a_swatch_shows_only_colours_it_can_show_without_converting() {
        assert_eq!(rgb_of(Color::Rgb(1.0, 0.0, 0.0)), Some([255, 0, 0]));
        assert_eq!(rgb_of(Color::Gray(0.0)), Some([0, 0, 0]));
        assert_eq!(rgb_of(Color::Gray(1.0)), Some([255, 255, 255]));
        assert_eq!(rgb_of(Color::Cmyk(0.0, 0.0, 0.0, 1.0)), None);
        assert_eq!(
            srgb_to_colour([128, 128, 128]),
            Color::Rgb(128.0 / 255.0, 128.0 / 255.0, 128.0 / 255.0)
        );
        assert_eq!(rgb_of(srgb_to_colour([1, 2, 3])), Some([1, 2, 3]));
    }

    /// ★ The width range is the same one the markup pen offers, and the same
    /// one the Properties panel offers.
    ///
    /// Two ranges for one quantity would let an operator author a 2 pt mark and
    /// then be unable to set 2 pt on it — or, worse, set a width here the pen
    /// could not have produced, so a document would carry marks the shell
    /// cannot make.
    #[test]
    fn the_width_range_matches_the_pen_that_authors() {
        assert!((MIN_WIDTH_PT - crate::canvas::markup::pen::MIN_WIDTH_PTS).abs() < f64::EPSILON);
        assert!((MAX_WIDTH_PT - crate::canvas::markup::pen::MAX_WIDTH_PTS).abs() < f64::EPSILON);
    }
}
