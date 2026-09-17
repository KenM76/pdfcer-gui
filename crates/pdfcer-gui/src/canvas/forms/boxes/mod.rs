//! # `canvas::forms::boxes` — where a form's widgets are, and what a click on
//! one would mean
//!
//! The **pure half** of filling a form on the page. Everything here is a
//! function of the document and a rectangle; nothing here needs an
//! `egui::Ui`, a laid-out scroll area or a live pointer, and every rule this
//! surface is judged on is therefore something a unit test can hold rather
//! than something a running window has to be trusted to demonstrate.
//!
//! That split is the seam `canvas/mod.rs` and `canvas/keys.rs` already draw
//! between themselves — one side is drivable by a headless `egui::Context`,
//! the other needs a window — applied one level down. It is a subject and not
//! a line count: [`super`] contains no decision at all, only the wiring that
//! spends the decisions below.
//!
//! ## ★ "Redraw appearances" is only half the remedy for an undrawn field
//!
//! [`super`]'s §5 reason 1 offers `RegenerateAppearances` to an operator whose
//! field draws nothing. `EditSession::regenerate_appearances` writes an `/AP`
//! for a text field only when that field holds a `/V`, so it does nothing at
//! all for an **empty** undrawn one. The remedy that always works is a fill:
//! `fill_text_field` writes the value and regenerates every widget's `/AP`, so
//! filling once in the panel makes the field clickable on the page from then
//! on. [`crate::text::forms::forms_canvas_undrawn_note`] is where the panel
//! says that to the operator.
//!
//! Read [`super`]'s header first. It carries the whole argument — why this is
//! not a [`CanvasTool`] variant, why the panel is not replaced, what the
//! editor cannot promise, how input layers, why the hit test takes no
//! tolerance, and the four reasons a field is routed to the panel instead.
//! This file is where those four reasons are actually decided
//! ([`classify`]), where the geometry is done
//! ([`crate::canvas::mapping::annot_canvas_rect`], which serves annotation
//! selection too) and
//! where the hit test lives ([`hit`]).

use egui::{Align, Pos2, Rect, Vec2};
use pdfcer_core::forms::{
    AcroForm, ButtonKind, Field, FieldFlags, FieldType, FieldValue, MkColor, Widget,
};
use pdfcer_core::object::ObjId;
use pdfcer_core::page_tree::Page;
use pdfcer_core::vartext::Quadding;

use crate::canvas::mapping::PageMapping;
use crate::canvas::tool::CanvasTool;

/// The smallest an editor may be drawn, in **screen** points.
///
/// A form field is whatever size its author made it, and at 25 % zoom a
/// perfectly ordinary 12 pt field is three pixels tall. An editor that small
/// is an editor nobody can read what they typed in, so the box is grown about
/// its own centre until it reaches this — which means it can overhang the
/// field it is editing.
///
/// That overhang is the deliberate half. The alternative is an editor that
/// sits exactly on a field the operator cannot see into, which trades a
/// visible, self-explaining imprecision for an invisible, silent one. It also
/// has an obvious operator-side remedy that needs no code: zoom in.
const MIN_EDITOR: Vec2 = Vec2::new(60.0, 18.0);

/// The proportion of an editor's height the text is set at.
///
/// A glyph box is taller than its letters, and a font size equal to the box
/// height clips descenders. 0.62 is the ratio at which an ascender-plus-
/// descender line fits inside the box with the padding `egui` adds, measured
/// against the theme's own text style rather than derived.
const EDITOR_TEXT_RATIO: f32 = 0.62;

/// The smallest and largest point size the editor will set text at.
///
/// The lower bound is legibility; the upper bound stops a full-page field —
/// a signature block, a comment box — from being typed into at 40 pt, which
/// reads as a bug rather than as fidelity.
const EDITOR_TEXT_RANGE: (f32, f32) = (9.0, 22.0);
// ===========================================================================
// What a widget is, on screen
// ===========================================================================

/// What clicking a widget means.
///
/// One variant per gesture, and no others: everything in
/// [`NotOnCanvas::NotOffered`] has no canvas gesture at all, so it is absent
/// here rather than present-and-inert. The "no placeholders" invariant applies
/// to enums as much as to labels — a variant nothing can raise is dead code
/// wearing a design pattern.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BoxKind {
    /// A `/Tx` field. A click focuses an editor; the value is committed on
    /// focus loss.
    Text {
        /// `/Ff` `Multiline` — the editor keeps Enter for a newline.
        multiline: bool,
        /// `/Ff` `Password` — the editor masks what is typed. It does **not**
        /// make the stored value secret, which is what
        /// [`crate::text::forms::form_field_password_tooltip`] exists to say.
        password: bool,
        /// `/MaxLen` in **characters**, truncating live for the reason
        /// [`crate::panels::forms::rows`] gives: a limit discovered at commit
        /// is a limit the operator finds out about by losing text.
        max_len: Option<usize>,
        /// `/Q` — **which end of the box the field's text is set against**
        /// (§12.7.4.3, Table 222: `0` left, `1` centre, `2` right), resolved by
        /// `pdfcer-core` through the field tree and the AcroForm default.
        ///
        /// ## ★★★ Why the editor honours this when it honours no other
        /// appearance property
        ///
        /// [`super`]'s §3 refuses to make the editor a facsimile of the
        /// rendered widget, and that refusal is **arithmetic**: the overlay is
        /// a font substitution by construction, so a box pretending to be the
        /// appearance stream would put the caret where the glyph is *not*
        /// going to land, and would be wrong by more the longer the string.
        ///
        /// That argument is about **glyph advances**, and it does not reach
        /// this. Quadding is not a measurement — it is a statement about which
        /// end of the box the run of text is anchored to, and it is the same
        /// statement whatever font draws it. Honouring it costs no accuracy
        /// claim that could later be falsified: a centred field's editor is
        /// centred, the committed `/AP` is centred, and the operator's text
        /// does not jump from one end of the box to the other at the moment
        /// they tab away. An editor that read `/Q` nowhere would type every
        /// field left-aligned, and a centred or right-aligned form would
        /// re-lay itself out the moment the value committed.
        ///
        /// ⇒ The rule the two paragraphs together state, for whoever extends
        /// this: an appearance property may be honoured here when honouring it
        /// makes no promise about *where a particular glyph will be*. `/Q`
        /// passes that test. `/DA`'s font and size do not, which is why
        /// [`editor_font_size`] still derives its number from the box.
        ///
        /// ## ★ The background colour is honoured too, and by the same test
        ///
        /// `/MK` `/BG` (Table 189) is a fill and not a placement, so honouring
        /// it makes no claim about where a glyph lands and it passes the rule
        /// above exactly as `/Q` does. [`editor_fill`] resolves it to an sRGB
        /// triple, [`WidgetBox`] carries it, and `canvas::forms` pairs it
        /// through `Theme::foreign_fill_pair` so the ink stays legible on it —
        /// which is why a shaded field does not turn grey when the operator
        /// clicks into it.
        align: Quadding,
    },
    /// A `/Btn` check box. A click toggles between `on_state` and `Off`.
    Check {
        /// The name a tick selects (§12.7.4.2.3).
        on_state: String,
        /// Whether the field currently holds that name.
        on: bool,
    },
    /// One widget of a `/Btn` radio group. A click selects **this widget's**
    /// on-state.
    ///
    /// Clicking the already-selected button does nothing, even on a group
    /// whose `/Ff` permits toggling off. Every reader behaves that way, and
    /// clearing a radio group is a deliberate act with a deliberate control —
    /// the panel's, which is labelled.
    Radio {
        /// This widget's own on-state name.
        on_state: String,
        /// Whether the field currently holds it.
        on: bool,
    },
    /// A `/Ch` field — a combo box or a list box. A click focuses the widget
    /// and opens its option list beside it; every pick is a complete command,
    /// with no draft in between, which is why nothing here has a `Text`-like
    /// commit boundary.
    Choice {
        /// `/Opt` as `(export, display)` pairs, in the file's own order.
        ///
        /// §12.7.4.4 makes displaying them in `/Opt` order a conformance
        /// requirement rather than a presentation choice; the `Sort` flag is
        /// an instruction to whoever *writes* the list.
        ///
        /// Either half may be empty. A single-string `/Opt` entry sets both to
        /// the same text, and a two-element `[export display]` entry may carry
        /// an explicitly empty display string — which is why
        /// `super::choosing`'s rows fall back to the export for a label.
        options: Vec<(String, String)>,
        /// `/V`, decoded, in whichever shape it legally takes: an array for a
        /// `MultiSelect` field, a bare string otherwise.
        ///
        /// Stored strings rather than indices into `options`, because a `/V`
        /// naming no option is a real state — another program wrote it, or the
        /// option list changed under it — and an index has nowhere to put it.
        /// What that costs is that every match asks both halves of an option;
        /// what it buys is that `super::choosing` can *see* the unlisted value
        /// and drop it, instead of carrying it into an engine refusal that
        /// would name a value the operator never touched.
        selected: Vec<String>,
        /// `/Ff` `MultiSelect` — several rows may be ticked at once, and the
        /// list stays open between ticks.
        multi: bool,
        /// `/Ff` `Combo` (§12.7.4.4, Table 230, bit 18) — a **combo box**
        /// rather than a list box.
        ///
        /// ★★★ It decides where the options are drawn, and the two are not
        /// variations of one another. A combo's options drop *below the
        /// widget* as a separate surface; a list box's options render **inside
        /// its own rectangle**, replacing what the appearance stream shows,
        /// with a scroll bar when they do not fit.
        ///
        /// That is not a styling choice — it is the behaviour of the product
        /// class, measured rather than assumed: *"the list option is somehow
        /// hidden from view in Acrobat until I click on it, then I can select
        /// options and if the box is too small for all of the options it gives
        /// a scroll bar."* `super::super::choosing` carries the photographs it
        /// was written against.
        combo: bool,
        /// `/Ff` `Edit` (§12.7.4.4, Table 230, bit 19) **and** `Combo` — the
        /// operator may type a value the list does not contain.
        ///
        /// ★★★ Both bits, never bit 19 alone. The spec's own words for bit 19
        /// are *"used only with Combo"*, so `Edit` on a list box is a
        /// meaningless bit rather than a meaningful one, and a decoder that
        /// honoured it would put a caret into a surface whose whole behaviour
        /// is choosing from rows. `pdfcer-core`'s `set_choice_value` gates its
        /// free-text branch on the same conjunction
        /// (`edit.rs`, `editable_combo`), so a shell that disagreed here would
        /// offer typing into a field the engine then refuses with
        /// `ChoiceValueNotInOptions`.
        ///
        /// ★★ What it changes on the page is the **whole control**, not a
        /// property of one: an editable combo box is a live text box with a
        /// drop button beside it, where a plain one is a focus ring with a
        /// popup. `super::super::choosing::typing` draws the first;
        /// [`super::super::choosing::choose`] draws the second.
        editable: bool,
        /// `/Q`, carried for the same reason [`BoxKind::Text::align`] is and
        /// read by the same code — but only on an `editable` combo box, which
        /// is the one choice surface with a live text box in it. A plain combo
        /// or a list box draws no text of its own, so nothing here has an end
        /// of the box to be set against.
        align: Quadding,
    },
}

/// Why a field is not offered on the page. See the module header §5.
///
/// Every variant leaves the field fillable **in the panel**, which is what
/// makes this a routing decision rather than a refusal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotOnCanvas {
    /// The widget carries no `/AP` `/N`, so the page draws nothing there.
    NoAppearance,
    /// The page's `/Rotate` is not 0 and this is a text field, so an
    /// unrotatable `egui::TextEdit` would run across the appearance's text.
    RotatedPage,
    /// **No page's `/Annots` lists this widget with a usable rectangle.**
    ///
    /// One variant rather than a separate "no `/Rect`" and "no `/P`", and the
    /// merge is the point. See [`place`]'s ★ section: the question *"which
    /// page is this widget on?"* is answered by walking each page's `/Annots`,
    /// so there is no `/P` to be absent, and a widget that no page lists is a
    /// widget with no place whatever its own dictionary says.
    NotPlaced,
    /// This field kind has no canvas gesture.
    NotOffered,
}

/// How many of a form's fillable fields have to be filled in the **panel**,
/// and why.
///
/// Produced by the same walk that produces the boxes ([`place`]) rather than
/// by a second pass, so the count and the behaviour cannot disagree — a panel
/// promising "3 fields can only be filled here" over a canvas that declined
/// four is worse than no count at all.
///
/// Counted **per field, not per widget**: one clickable widget is enough for
/// the field to be reachable on the page, and a per-widget count would report a
/// two-page field as unreachable because one of its two boxes sits on a
/// rotated sheet.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Routing {
    /// Fields with no drawn appearance anywhere. The remedy is "Redraw
    /// appearances", which is why this is counted separately from the rest.
    pub undrawn: usize,
    /// Fields that are drawn but still cannot be typed into on the page — a
    /// rotated sheet, or a widget no page lists.
    pub unreachable: usize,
}

/// One widget of one field, placed — **whatever kind it is and whether or not
/// it can be filled.**
///
/// ## ★★★ Why this is not [`WidgetBox`], and why it comes from the same walk
///
/// A `WidgetBox` is a widget a click can **fill**, and five conditions narrow
/// the set: no appearance, a rotated page, an unlisted widget, a kind with no
/// canvas gesture (a drop-down, a button), or a field type this shell does not
/// type into. Every one of those is right for filling and **wrong for
/// selecting**. An operator who has just placed a drop-down and wants to look
/// at its properties must be able to click the thing they can plainly see.
///
/// So the authoring surface needs a wider set. It is produced by the **same
/// walk** ([`place`]) rather than by a second one, which is this module's
/// standing rule stated in [`Placed`]'s own doc: two walks are two statements
/// of the placement rule, and the drift between them is a click that selects a
/// field the canvas is not drawing.
///
/// The only condition that still excludes a widget here is the one that is not
/// a policy: **no canvas rectangle**, which means a non-invertible page
/// transform or a degenerate `/Rect` — a widget with no area to click.
#[derive(Clone, Debug, PartialEq)]
pub struct FieldTarget {
    /// 0-based page index.
    pub page: usize,
    /// The field's fully-qualified name — the vocabulary every field verb
    /// takes, and the only handle `rename_field` and `delete_field` accept.
    pub field: String,
    /// This widget's index within `Field::widgets`.
    ///
    /// Carried because `delete_widget` takes one, and because a field with two
    /// widgets on two pages is one field the operator can select from either
    /// place — the properties surface has to be able to say *which* box they
    /// clicked without pretending it is a different field.
    pub widget: usize,
    /// Where it is, in canvas space.
    pub rect: Rect,
}

/// Everything one walk of the form produces: the boxes the canvas hit-tests,
/// and the counts the panel discloses.
///
/// One type, because they are one walk. The alternative — a `boxes_for` and a
/// separate `routing_for` — is two statements of the five-reason rule in §5,
/// which is exactly the drift this module exists to prevent.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Placed {
    /// Every fillable widget, in canvas space.
    pub boxes: Vec<WidgetBox>,
    /// What could not be placed, and why.
    pub routing: Routing,
    /// Every widget with a rectangle, fillable or not — what the **authoring**
    /// surface hit-tests. See [`FieldTarget`].
    pub targets: Vec<FieldTarget>,
}

/// One fillable widget, placed.
///
/// **`rect` is CANVAS space**, which is what makes this cacheable across zooms
/// and scrolls: canvas space is the frame `PageMapping` converts *to*, and a
/// canvas coordinate does not move when the view does (`mapping`'s
/// `a_canvas_point_survives_every_zoom_and_scroll_position`). A screen-space
/// cache would have to be rebuilt on every wheel notch, and a stale one would
/// focus the wrong field.
#[derive(Clone, Debug, PartialEq)]
pub struct WidgetBox {
    /// 0-based page index.
    pub page: usize,
    /// The field's fully-qualified name — the vocabulary **every** fill verb
    /// takes. No fill verb takes an `ObjId`, so a click has to resolve widget
    /// → field → name, and this is where it lands.
    pub field: String,
    /// The widget's index within `Field::widgets`, used only to salt the
    /// editor's `egui` id so two boxes of one field cannot share one caret.
    pub widget: usize,
    /// The widget annotation's own object id.
    ///
    /// Carried so a tab ring can be built from the ENGINE's answer rather
    /// than from this list's order. `EditSession::page_tab_sequence` returns
    /// `Vec<ObjId>`, and an object id is the only vocabulary in which this
    /// list and that answer can be joined — `/Annots` order, which is what
    /// this list is sorted into, is the tab order only on a page with no
    /// `/Tabs` entry. See `OPERATOR_REQUESTS.md` O204.
    pub id: ObjId,
    /// What a click means.
    pub kind: BoxKind,
    /// Where it is, in canvas space.
    pub rect: Rect,
    /// The widget's own `/MK` `/BG` as sRGB components in `0.0..=1.0`, or
    /// `None` when the file states no background — see [`editor_fill`], which
    /// is the whole of the rule and carries the reasoning.
    ///
    /// Carried on the box rather than looked up at draw time because this list
    /// is the cache: it is rebuilt per `(document, edit epoch)`, and an editor
    /// that re-read the `AcroForm` every frame to find one colour would be
    /// re-deriving a fact this structure exists to hold.
    pub fill: Option<[f32; 3]>,
}
// ===========================================================================
// The pure rules
// ===========================================================================

/// Whether this tool offers form filling at all.
///
/// [`CanvasTool::Select`] and nothing else — see the module header §1. The
/// hand tool's click means "nothing" everywhere else in this canvas and must
/// go on meaning it here; a markup tool's press is claimed unconditionally by
/// `gesture::press_kind`, and a pen that filled a field as well as drawing
/// would be two gestures on one press.
///
/// A held space bar borrows the hand out of Select before this is asked
/// (`tool::resolve`), so filling is suspended for exactly as long as the bar
/// is down and returns with nothing stored — the same property the space
/// override buys everywhere else.
#[must_use]
pub fn offered_in(tool: CanvasTool) -> bool {
    matches!(tool, CanvasTool::Select)
}

/// The widget's own background colour as sRGB components, or `None` for
/// "leave the theme's box alone".
///
/// # ★★★ What this is for, and why it is not a facsimile
///
/// The in-canvas field editor lays a live `egui` text box over the raster for
/// the duration of a keystroke. With no fill that box is `extreme_bg_color` —
/// near-white under the light presets — so a pale-yellow or shaded form field
/// **turns grey the moment the operator touches it** and turns back a gesture
/// later: a visible change to content nobody asked for, which is the thing
/// pdfcer's rule 4 forbids.
///
/// It is not a fidelity claim, and the module header's §3 is the reason it
/// does not have to be one. §3 refuses to make this box a facsimile because a
/// substituted font cannot promise the document font's glyph advances — an
/// **arithmetic** argument, and one that reaches exactly as far as the
/// arithmetic does. The test a property must pass to be honoured here is
/// therefore *does honouring it make a claim about where a particular glyph
/// will land*. A fill does not, exactly as `/Q` does not — and the engine's
/// own `Widget::background` doc names this editor as the intended consumer.
///
/// # The three-state `/BG`, which is why this takes the widget and not a colour
///
/// Table 189 distinguishes **absent** (`background == None`) from an **empty
/// array** ([`MkColor::None`] — *explicitly no colour, transparent*). Both
/// answer `None` here, because both mean *keep the theme's box*, but they are
/// different facts about the file and a caller that collapsed them upstream
/// would have lost one. This function is where they are allowed to merge, and
/// it merges them at the point of use rather than at the point of reading.
///
/// # ★★ DeviceCMYK IS converted here, and elsewhere in this shell it is not
///
/// `app::markupband::rgb_of` returns `None` for a CMYK mark and
/// `app::fontband` greys the swatch, on a rule this project holds firmly: *a
/// swatch showing a converted colour is a control whose readback is a
/// conversion the operator never asked for — pick it up, put it down
/// unchanged, and the file now says something different.*
///
/// That argument is about a **round trip**, and this is not one. Nothing here
/// is written back; the value is a tint on a transient overlay that is gone
/// the moment the edit commits. And the conversion is
/// [`pdfcer_core::color::cmyk_to_srgb`] — the engine's own calibrated one, the
/// same conversion that produced **the raster pixels immediately around the
/// box**. Refusing it would not avoid a conversion; it would make the editor
/// disagree with the page it is sitting on.
///
/// ★ The match is exhaustive with no wildcard, so a new [`MkColor`] variant
/// stops the build here rather than silently taking the `None` arm.
#[must_use]
pub fn editor_fill(widget: &Widget) -> Option<[f32; 3]> {
    match widget.background? {
        MkColor::None => None,
        MkColor::Gray(g) => Some([g, g, g]),
        MkColor::Rgb(r, g, b) => Some([r, g, b]),
        MkColor::Cmyk(c, m, y, k) => Some(pdfcer_core::color::cmyk_to_srgb(c, m, y, k)),
    }
}

/// What a click on `widget` would mean, or why nothing.
///
/// Pure, and deliberately takes the page's `/Rotate` as a number rather than a
/// `&Page`: the rotation is the only thing about the page this decision
/// depends on, and passing the page would make the rule untestable without
/// building one.
///
/// # The order of the questions is the rule
///
/// Appearance first, because a field nothing draws cannot be pointed at
/// whatever else is true of it. Then the panel's own
/// [`block_reason`](crate::panels::forms::rows::block_reason), **asked rather
/// than re-derived** — a read-only field must be refused here for the same
/// reason and in the same words it is refused there, and two statements of one
/// rule is how the two surfaces come to disagree about which fields are
/// fillable.
///
/// Rotation is asked **last, and only for text**, which is the whole of the
/// rotated-page decision: the box is placed correctly at every rotation, and
/// it is only the editor that cannot be.
///
/// # ★ It asks nothing about geometry
///
/// The geometry a click is tested against does not come from `Widget::rect`
/// — [`place`] takes it from each page's `/Annots` instead. Asking about it
/// here as well would be a second source of truth for where a widget is, and
/// the one that is *not* the one being hit-tested.
pub fn classify(field: &Field, widget: &Widget, rotate: u16) -> Result<BoxKind, NotOnCanvas> {
    if !widget.has_normal_appearance {
        return Err(NotOnCanvas::NoAppearance);
    }
    if crate::panels::forms::rows::block_reason(field).is_some() {
        return Err(NotOnCanvas::NotOffered);
    }

    match (field.field_type, field.button_kind) {
        // Rich text is not refused by `block_reason` on purpose — the panel
        // offers it a disclosed *conversion*. There is no conversion gesture
        // on a page, so here it is simply not offered.
        (Some(FieldType::Text), _) if field.is_rich_text() => Err(NotOnCanvas::NotOffered),
        (Some(FieldType::Text), _) => {
            if !rotate.is_multiple_of(360) {
                return Err(NotOnCanvas::RotatedPage);
            }
            Ok(BoxKind::Text {
                multiline: field.flags.has(FieldFlags::MULTILINE),
                password: field.flags.has(FieldFlags::PASSWORD),
                // Negative and absurd `/MaxLen` values are real in the wild;
                // `try_from` rejecting them is the same guard the panel's row
                // applies, expressed as an `Option` rather than as an `if`.
                max_len: field
                    .max_len
                    .filter(|m| *m > 0)
                    .and_then(|m| usize::try_from(m).ok()),
                // Read off the FIELD rather than off the widget, because that
                // is where `/Q` lives: §12.7.3.3 makes it an entry of the
                // variable-text field dictionary, and core has already
                // resolved it up the field tree and through the AcroForm
                // default (`Field::quadding`, default `Left` per Table 222).
                // A widget of a centred field is centred; there is no
                // per-widget override to look for.
                align: field.quadding,
            })
        }
        (Some(FieldType::Button), Some(kind @ (ButtonKind::Check | ButtonKind::Radio))) => {
            // The clicked widget's OWN on-state, not the field's first. For a
            // check box the two are the same; for a radio group they are the
            // whole point — each kid carries the name selecting *it*.
            let Some(on_state) = widget.on_states.first() else {
                // No on-state on this widget means `set_button_state` would
                // refuse every name but `Off`, which is not a click. The panel
                // draws the same case disabled and explained.
                return Err(NotOnCanvas::NotOffered);
            };
            let on_state = String::from_utf8_lossy(on_state).into_owned();
            let on = match &field.value {
                FieldValue::Name(n) => String::from_utf8_lossy(n) == on_state,
                _ => false,
            };
            Ok(match kind {
                ButtonKind::Check => BoxKind::Check { on_state, on },
                _ => BoxKind::Radio { on_state, on },
            })
        }
        (Some(FieldType::Choice), _) => {
            // An empty `/Opt` is not offered, and that is the same rule as a
            // button with no on-state: there is no pick to make. The panel
            // says so in words (`form_field_choice_no_options`); an empty
            // popup over the page would be a placeholder.
            if field.options.is_empty() {
                return Err(NotOnCanvas::NotOffered);
            }
            // Rotation is NOT asked here, unlike the text case above, and the
            // asymmetry is the rotated-page decision stated exactly. What
            // `RotatedPage` refuses is an `egui::TextEdit` laid OVER the
            // appearance it is editing, running horizontally across text the
            // `/AP` draws vertically. A choice field has no editor over the
            // box: the ring traces the box, and the list is drawn beside it in
            // screen space. Neither overlays the widget's own text, so a
            // `/Rotate 90` sheet's choice fields are offered exactly as any
            // other sheet's are.
            //
            // Both halves come from the panel's own readers rather than from a
            // second decode here, for the reason the `block_reason` call above
            // exists: two statements of one rule is how the page and the panel
            // come to offer different options for one field.
            Ok(BoxKind::Choice {
                options: crate::panels::forms::rows::choice_options(field),
                selected: crate::panels::forms::rows::choice_selections(field),
                multi: field.flags.has(FieldFlags::MULTI_SELECT),
                combo: field.flags.has(FieldFlags::COMBO),
                editable: field.flags.has(FieldFlags::COMBO) && field.flags.has(FieldFlags::EDIT),
                align: field.quadding,
            })
        }
        _ => Err(NotOnCanvas::NotOffered),
    }
}

/// Every fillable box in a document, in canvas space, plus the counts for the
/// fields that got none.
///
/// Built once per `(document, edit epoch)` and cached — see
/// [`super::placed`] — rather than per frame, which is what makes an I-beam
/// cursor over a form affordable. The whole document rather than the visible
/// pages, because the cache key has no room for a scroll position and a form is
/// small: `pdfcer-core`'s corpus has nothing over a thousand fields.
///
/// `annots[i]` is page `i`'s `EditSession::widget_rects(i)` — every `/Widget`
/// annotation that page's `/Annots` lists, with its `/Rect` already normalised.
///
/// # ★ Which page a widget is on is answered by `/Annots`, never by `/P`
///
/// The obvious implementation reads `pdfcer_core::forms::Widget::page` — the
/// widget's `/P` entry — and looks the page object up by id. It is
/// **silently wrong on a large class of real files**.
///
/// `/P` is *Optional* (§12.5.2 Table 164). A widget that omits it is perfectly
/// conformant and is common in the wild, and `pdfcer-core` additionally reads
/// the key **without resolving through the graph**, so a direct rather than
/// indirect `/P` also reads as absent. Either way a `/P`-keyed placement
/// returns *nothing at all* for such a form: no error, no refusal, no trace —
/// a form on which clicking a field simply does not work, with the panel
/// cheerfully reporting every field as fillable.
///
/// **No test written against the fixture corpus can catch this.** The form
/// fixtures in `D:\Dev\pdfcer\fixtures\synthetic\forms\` write `/P` on every
/// widget, so the failing case is unreachable from them, and *a test that
/// cannot reach the case is satisfied by any implementation* — sabotage the
/// implementation and the suite stays green. That is why
/// [`tests::a_widget_with_no_p_entry_is_still_placed`] builds its input by hand
/// rather than opening a fixture.
///
/// So the direction is inverted: rather than asking each widget which page it
/// claims, each **page** is asked which widgets it lists, and `/P` is not
/// consulted anywhere in this module. A widget no page lists is
/// [`NotOnCanvas::NotPlaced`], which is the honest statement of the same fact
/// and is *true* rather than merely defaulted.
///
/// # Ordering
///
/// Within a page, `/Annots` order — **paint order**, and absent `/Tabs` also
/// tab order. Deliberately not the panel's order, which is `/AcroForm`
/// `/Fields` order: the two commonly differ, and they answer different
/// questions. [`hit`] depends on this one (a widget painted over another wins
/// the click); the panel's list depends on its own. Making either match the
/// other would break the surface that needed it.
#[must_use]
pub fn place(form: &AcroForm, pages: &[Page], annots: &[Vec<(ObjId, [f64; 4])>]) -> Placed {
    // Widget object id -> (page index, normalised `/Rect`). First listing wins:
    // a widget appearing in two pages' `/Annots` is malformed, and the earlier
    // page is the one a reader draws it on.
    let mut placement: std::collections::HashMap<ObjId, (usize, [f64; 4])> =
        std::collections::HashMap::new();
    // Kept alongside, because a hit test needs `/Annots` order and a `HashMap`
    // has none. `rank` is "how late in its page's `/Annots` this widget is",
    // which is what "drawn over" means.
    let mut rank: std::collections::HashMap<ObjId, usize> = std::collections::HashMap::new();
    for (page_index, page_annots) in annots.iter().enumerate() {
        for (order, (id, rect)) in page_annots.iter().enumerate() {
            placement.entry(*id).or_insert((page_index, *rect));
            rank.entry(*id).or_insert(order);
        }
    }

    let mut out: Vec<(usize, WidgetBox)> = Vec::new();
    let mut targets: Vec<(usize, FieldTarget)> = Vec::new();
    let mut routing = Routing::default();
    for field in &form.fields {
        // A field is routed to the panel only when NO widget of it can be
        // clicked — see [`Routing`].
        let mut reachable = false;
        let mut reasons: Vec<NotOnCanvas> = Vec::new();
        for (widget_index, widget) in field.widgets.iter().enumerate() {
            let Some((page_index, rect)) = placement.get(&widget.id).copied() else {
                reasons.push(NotOnCanvas::NotPlaced);
                continue;
            };
            let Some(page) = pages.get(page_index) else {
                reasons.push(NotOnCanvas::NotPlaced);
                continue;
            };
            // A projection failure here is a non-invertible page transform or a
            // degenerate rectangle — the one case both coordinate bridges
            // decline together, and a widget with no area on screen.
            let Some(canvas) = crate::canvas::mapping::annot_canvas_rect(rect, page) else {
                reasons.push(NotOnCanvas::NotPlaced);
                continue;
            };
            // ★★ SELECTABLE FROM HERE, and the position of this push is the
            // whole point: it is **above** the `classify` call and below the
            // rectangle, so a widget is selectable exactly when it has a place
            // on the canvas and regardless of whether it can be filled. A
            // drop-down and a push button reach this line and are refused by
            // `classify` one line down; they are still things the operator can
            // see and must be able to click.
            targets.push((
                rank.get(&widget.id).copied().unwrap_or(0),
                FieldTarget {
                    page: page_index,
                    field: field.fully_qualified_name.clone(),
                    widget: widget_index,
                    rect: canvas,
                },
            ));
            // ★★★ `reachable` is answered by `classify` and NOT by the
            // rectangle pushed above, and the two are easy to conflate.
            //
            // `reachable` means *"some widget of this field can be FILLED on
            // the page"*, and it is what suppresses the panel's
            // `routing.undrawn` disclosure — the sentence that tells an
            // operator a field exists but has to be filled in the side panel.
            // Setting it beside the rectangle makes every drawn-nothing field
            // look reachable, silently deleting that disclosure for the exact
            // documents it was written for.
            //
            // So: a rectangle makes a widget SELECTABLE; a successful
            // `classify` makes it FILLABLE; and only the second answers
            // `reachable`. `an_undrawn_widget_is_still_selectable` holds the
            // first half of that.
            let kind = match classify(field, widget, page.rotate) {
                Ok(kind) => kind,
                Err(reason) => {
                    reasons.push(reason);
                    continue;
                }
            };
            reachable = true;
            out.push((
                rank.get(&widget.id).copied().unwrap_or(0),
                WidgetBox {
                    page: page_index,
                    field: field.fully_qualified_name.clone(),
                    widget: widget_index,
                    id: widget.id,
                    kind,
                    rect: canvas,
                    fill: editor_fill(widget),
                },
            ));
        }
        if reachable || reasons.is_empty() {
            continue;
        }
        // The most SPECIFIC reason wins when a field's widgets disagree, and
        // "not drawn" is the specific one because it is the one with a remedy
        // the operator can act on.
        if reasons.contains(&NotOnCanvas::NoAppearance) {
            routing.undrawn += 1;
        } else if reasons
            .iter()
            .any(|r| matches!(r, NotOnCanvas::RotatedPage | NotOnCanvas::NotPlaced))
        {
            routing.unreachable += 1;
        }
    }

    // Back into `/Annots` order within each page. The field walk above visits
    // in `/Fields` order, and [`hit`] resolves overlaps by taking the LAST
    // match — which is only "the one drawn on top" if this list is in paint
    // order. A stable sort, so two widgets that somehow share a rank keep the
    // order the pages listed them in.
    out.sort_by_key(|(order, b)| (b.page, *order));
    // The same paint-order sort, for the same reason: [`hit_target`] takes the
    // LAST match, which is only "the one drawn on top" if this list is in
    // `/Annots` order within each page.
    targets.sort_by_key(|(order, t)| (t.page, *order));
    Placed {
        boxes: out.into_iter().map(|(_, b)| b).collect(),
        routing,
        targets: targets.into_iter().map(|(_, t)| t).collect(),
    }
}

/// Which box a canvas-space point is inside, on `page`.
///
/// **Containment, no tolerance** — see the module header §4 for why this is
/// the one hit test in `canvas/` that takes none.
///
/// Later boxes win. `/Annots` order is paint order, so a widget drawn over
/// another is the one the operator can see, and the one they can see is the
/// one they meant.
#[must_use]
pub fn hit(boxes: &[WidgetBox], page: usize, point: Pos2) -> Option<&WidgetBox> {
    boxes
        .iter()
        .rfind(|b| b.page == page && b.rect.contains(point))
}

/// Which **selectable** widget a canvas-space point is inside, on `page`.
///
/// [`hit`]'s twin over the wider set. Containment with no tolerance and later
/// boxes winning, for the identical reasons — a widget drawn over another is
/// the one the operator can see, and the one they can see is the one they
/// meant.
#[must_use]
pub fn hit_target(targets: &[FieldTarget], page: usize, point: Pos2) -> Option<&FieldTarget> {
    targets
        .iter()
        .rfind(|t| t.page == page && t.rect.contains(point))
}

/// The editor's rect on screen: the widget's own, grown to [`MIN_EDITOR`].
///
/// Grown about the **centre** rather than from the top-left, so a field that
/// is already wide enough and only too short does not slide sideways under the
/// operator's pointer between one zoom and the next.
#[must_use]
pub fn editor_rect(map: &PageMapping, canvas: Rect) -> Rect {
    let screen = map.rect_to_screen(canvas);
    let grow = Vec2::new(
        (MIN_EDITOR.x - screen.width()).max(0.0) / 2.0,
        (MIN_EDITOR.y - screen.height()).max(0.0) / 2.0,
    );
    screen.expand2(grow)
}

/// The point size the editor sets text at, for a box `height` points tall on
/// screen.
///
/// Derived from the box rather than from the field's `/DA`, and that is the
/// honest choice rather than the lazy one: the `/DA` size is stated in *page*
/// units for a *document* font, and this editor draws a *substituted* font at
/// *screen* scale. Honouring the `/DA` number would produce a box whose text
/// is the right nominal size and the wrong physical one, which looks like a
/// fidelity claim and is not one. See the module header §3.
#[must_use]
pub fn editor_font_size(height: f32) -> f32 {
    (height * EDITOR_TEXT_RATIO).clamp(EDITOR_TEXT_RANGE.0, EDITOR_TEXT_RANGE.1)
}

/// Which end of the editor the operator's text is set against, for a field's
/// `/Q`.
///
/// One line of arithmetic-free translation, given a function of its own for
/// two reasons that are both about evidence rather than about tidiness:
///
/// 1. **It is the whole of the `/Q` decision**, and this file is the half of
///    the surface a unit test can hold ([the module header](self)). Applying
///    the mapping inline in [`super::editor`] would put the only statement of
///    §12.7.4.3's three codes inside a function that needs a laid-out
///    `egui::Ui`, a live pointer and a page raster to run at all — which is to
///    say, inside the half that can only be checked by looking.
/// 2. **The failure it guards is a silent transposition.** `1` is centre and
///    `2` is right; swapping them compiles, draws, passes every other test in
///    this file and is visible only as a right-aligned form typed into
///    centred. [`Quadding`] has already turned the integers into names, and
///    this keeps the names paired with `egui`'s in one place.
///
/// [`Quadding::from_code`] has already applied Table 222's own tolerance — any
/// `/Q` that is not `1` or `2` is left — so there is no malformed case left
/// for this to decide.
#[must_use]
pub fn editor_align(quadding: Quadding) -> Align {
    match quadding {
        Quadding::Left => Align::LEFT,
        Quadding::Center => Align::Center,
        Quadding::Right => Align::RIGHT,
    }
}

/// Truncate a draft to `/MaxLen`, in **characters**.
///
/// Live rather than at commit, and by character rather than by byte — the
/// panel's rule, restated as a function so the two surfaces cannot enforce
/// different limits. A byte index would both split a multi-byte character and
/// refuse an accented name three letters early.
#[must_use]
pub fn truncate(draft: &str, max_len: Option<usize>) -> String {
    match max_len {
        Some(max) if draft.chars().count() > max => draft.chars().take(max).collect(),
        _ => draft.to_owned(),
    }
}

#[cfg(test)]
mod tests;
