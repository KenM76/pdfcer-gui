//! # `app::markupband` — the five Format ▸ Markup controls the ribbon cannot
//! draw itself
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/markupband.md`.

use egui::Ui;
use egui_shell::commands::{CommandRegistry, ConditionSet, HandlerToken};
use pdfcer_core::annot_author::{Color, LineEnding};
use pdfcer_core::edit::{MarkupStyle, MarkupStyleSupport, StyleEdit};
use pdfcer_gui_base::entry;

use crate::app::state::OpenDoc;
use crate::canvas::selection::annot::{AnnotKind, AnnotTarget};
use crate::text::panels::properties as t;
use crate::text::ribbon as r;

/// The narrowest border this shell offers, in points.
const MIN_WIDTH_PT: f64 = 0.25;

/// The widest, in points.
const MAX_WIDTH_PT: f64 = 12.0;

/// The width of the two numeric fields, in points.
const FIELD_WIDTH: f32 = 46.0;

/// The width of the arrowhead chooser, in points.
const ENDINGS_WIDTH: f32 = 84.0;

/// The width of the line-style chooser, in points.
const DASH_WIDTH: f32 = 88.0;

/// **One property of one mark, parked for the dispatcher.**
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
    /// A `StyleEdit` since 2026-09-06, and the two arms are two different
    /// files that draw the same line: `Set` writes the array — including
    /// `Set((None, None))`, which is *"no arrowheads"* stated explicitly — and
    /// `Clear` removes `/LE` altogether, so the mark goes back out the way it
    /// came in. See this module's header for why the second is offered as an
    /// action rather than as a fifth position in the chooser.
    Endings(StyleEdit<(LineEnding, LineEnding)>),
    /// `/BS` `/S` + `/D` — the **line style**: dashed, or solid.
    ///
    /// `RIBBON_IA.md` §5.8's eighth control, and the only one of the eight
    /// that had *"no engine verb at all"* until the afternoon of 2026-09-06.
    /// `MarkupStyle::dash` is that verb, and its two arms are the two things
    /// this chooser can mean: `Set(dash)` makes the border dashed with that
    /// pattern, `Clear` makes it solid.
    ///
    /// **The third arm is `None`, and it is what NOT touching the control
    /// does.** A restyle that does not mention `dash` preserves whatever the
    /// annotation has, *including a dash pdfcer never authored* — so an
    /// operator who changes a foreign-dashed mark's colour keeps their dash,
    /// which is exactly the defect this shell filed and the reason
    /// [`crate::canvas::markup::linestyle::DashReading::Foreign`] is a state
    /// the chooser can show and not one it has to repair.
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

    // The read-back is attempted only when the control could be live, and
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
            // The `Ready` values, or the defaults a greyed control shows. The
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
                k if k == crate::shell::manifest::MARKUP_ENDINGS => {
                    endings(ui, *current, target, parked)
                }
                // Unreachable while `command_for` names exactly these six, and
                // named rather than left as the arrowhead chooser's catch-all:
                // a seventh Markup kind added to `command_for` and forgotten
                // here would draw an **arrowhead chooser** under its own
                // tooltip — a control that works, reports a rect, and edits the
                // wrong property.
                _ => false,
            };
        })
        .response;

    crate::diag::ui_rect(&egui_shell::ribbon::report::band_item(id), response.rect);

    // **The sentence depends on WHY the control is not live**, which is the
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
#[derive(Debug, Clone, Copy)]
struct Current {
    /// **Which of these properties this `/Subtype` can take at all — the
    /// ENGINE's answer, not this module's.**
    ///
    ///
    /// Distinct from a value being `None`, and the distinction is the whole
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
    /// Tracked apart from [`Self::interior`] because a CMYK fill is *set* and
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
    /// Read off the **dictionary**, not off the spec, and that is not a
    /// departure from this struct's rule — it is the same exception `/CA` is,
    /// one property along. A dash cuts across `MarkupSpec`'s variants rather
    /// than belonging to any of them, so the engine carries it in
    /// `pdfcer_core::annot_author::AppearanceOptions` beside the spec instead
    /// of inside it, and `spec_from_dict` therefore does not return one. The
    /// engine's own reader, `annot_author::read_border_dash`, is
    /// `pub(crate)`, so
    /// [`crate::canvas::markup::linestyle::read`] is this shell's copy of it —
    /// with the copy declared as a copy in that function's header, and the
    /// bound on what a divergence can cost written down beside it.
    dash: crate::canvas::markup::linestyle::DashReading,
    /// The `/LE` pair the mark currently draws, when [`Self::support`] says it
    /// has one.
    ///
    /// This is a **value**, and it comes from `MarkupSpec::Line`'s own field
    /// because only that arm has one — a fact the compiler checks and the
    /// engine publishes no API for. Whether the control is offered at all is
    /// `support.takes_endings`, which is a different question with a different
    /// owner. The header's table draws the line.
    endings: Option<(LineEnding, LineEnding)>,
    /// **Whether `/LE` is actually IN the dictionary**, as opposed to being
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

/// Even "nothing to show" asks the engine what the properties are.
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
        // `/CA` straight off the dictionary rather than through the spec: it
        // is not part of `MarkupSpec` at all — the engine composites the
        // annotation onto the page rather than letting the appearance draw it,
        // which is why `set_markup_style` applies it to the dictionary
        // directly.
        //
        // `ObjectGraph::resolve` comes from the TRAIT, so it has to be in
        // scope; there is no inherent method, and reaching for one is the error
        // a reader hits first. Following the reference through the session's
        // overlay rather than through the base file is what makes an unsaved
        // edit visible here.
        let alpha = dict
            .get(b"CA")
            .map(|o| graph.resolve(o))
            .and_then(Object::as_number);

        // **The capability question, asked of the engine, off the same key
        // the engine itself reads.** `set_markup_style` derives its
        // `MarkupStyleSupport` from `/Subtype` on the annotation dictionary; so
        // does this. One key, one function, one
        // answer — which is what makes a control shown here and a call refused
        // there impossible to disagree.
        //
        // Read through `graph.resolve` for the same reason `/CA` is: an
        // indirect `/Subtype` is legal, and following it through the session's
        // overlay is what makes an unsaved edit visible.
        let subtype = dict
            .get(b"Subtype")
            .map(|o| graph.resolve(o))
            .and_then(Object::as_name)
            .map_or_else(Vec::new, |n| n.as_bytes().to_vec());
        let support = MarkupStyleSupport::for_subtype(&subtype);

        // Presence, not value. See `Self::endings_key_present`: this is the
        // one fact `spec_from_dict` deliberately erases, because Table 176's
        // default makes an absent `/LE` and a written `[/None /None]` the same
        // picture — and the *Clear the setting* action exists precisely to tell
        // them apart.
        let endings_key_present = dict
            .get(b"LE")
            .map(|o| graph.resolve(o))
            .is_some_and(|o| !matches!(o, Object::Null));

        // Read BEFORE `spec_from_dict`, and carried across its refusal —
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
        // **This `match` reads VALUES; it no longer decides CAPABILITIES.**
        //
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
                interior,
                ..
            } => {
                current.stroke = rgb_of(color);
                current.interior_set = interior.is_some();
                current.interior = interior.and_then(rgb_of);
                current.width = Some(width);
                current.endings = Some(endings);
            }
            MarkupSpec::PolyLine { color, width, .. } | MarkupSpec::Ink { color, width, .. } => {
                current.stroke = rgb_of(color);
                current.width = Some(width);
            }
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
    // WHETHER a control is drawn — one question, one place, testable
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
    const fn offers_fill(self) -> bool {
        self.support.takes_interior
    }

    /// Whether the width field draws.
    const fn offers_width(self) -> bool {
        self.support.takes_border && self.width.is_some()
    }

    /// Whether the Line style chooser draws.
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
    /// Strictly narrower than [`Self::offers_endings`]: there has to be a
    /// chooser to put it in **and** a `/LE` in the file to take out.
    const fn offers_endings_clear(self) -> bool {
        self.offers_endings() && self.endings_key_present
    }
}

/// The outline colour, `/C`.
fn stroke(
    ui: &mut Ui,
    current: Current,
    target: &AnnotTarget,
    parked: &mut Option<(AnnotTarget, MarkupEdit)>,
) -> bool {
    // The default is BLACK rather than an invented colour, and it is the same
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
    // WHITE, not black, and it is the one default in this module that differs
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
    // The draft, not the document — see `drafted`. Re-seeding from `was`
    // every frame is what made this control undraggable for its whole life.
    let draft_id = ui.id().with("markup.width.draft");
    let mut value = drafted::<f64>(ui, draft_id, was);
    let (widget, refusal) = entry::drag_value(
        ui,
        &mut value,
        entry::Kind::Length(entry::LengthUnit::Point),
    );
    let response = refusal.show(
        ui.add(
            widget
                .range(MIN_WIDTH_PT..=MAX_WIDTH_PT)
                .speed(0.1)
                .max_decimals(2),
        ),
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
    // Same draft as `width`, and it had the same defect — the driven check
    // named width and flagged this one as "built the same way and worth
    // checking". It was. Both are fixed by the same two calls.
    let draft_id = ui.id().with("markup.opacity.draft");
    let mut percent = drafted::<u8>(ui, draft_id, was);
    let (widget, refusal) = entry::drag_value(ui, &mut percent, entry::Kind::Number(&["%"]));
    let response = refusal.show(
        ui.add(
            widget
                .range(0..=100)
                .speed(1.0)
                .suffix(t::markup_opacity_suffix()),
        ),
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

/// **The border line style, `/BS` `/S` and `/D` — the eighth control.**
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
    // The one place `BorderDash::new`'s `Option` is answered on this surface,
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

/// Which ends of a `/Line` carry an arrowhead.
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
            // The separator is the whole of the presentation decision: what
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
fn arrow_shape(pair: (LineEnding, LineEnding)) -> LineEnding {
    if pair.0 == LineEnding::ClosedArrow || pair.1 == LineEnding::ClosedArrow {
        LineEnding::ClosedArrow
    } else {
        LineEnding::OpenArrow
    }
}

/// An annotation colour as sRGB bytes, if it is one a swatch can show.
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
fn srgb_to_colour(rgb: [u8; 3]) -> Color {
    Color::Rgb(
        f64::from(rgb[0]) / 255.0,
        f64::from(rgb[1]) / 255.0,
        f64::from(rgb[2]) / 255.0,
    )
}

/// **Holding a spinner's value across frames while it is being dragged.**
///
use crate::app::spinnerdraft::{drafted, keep_draft};

#[cfg(test)]
mod tests;
