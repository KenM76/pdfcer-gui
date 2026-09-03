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
//! between themselves — *"that module is drivable by a headless
//! `egui::Context`, this one needs a window"* — applied one level down, and it
//! was forced by R2's 1,500-line ceiling exactly as the `keys` split was. It
//! turns out to be a real seam and not merely a cut: [`super`] contains no
//! decision at all, only the wiring that spends the decisions below.
//!
//! ## ★ One correction this file made after being driven
//!
//! [`super`]'s §5.1 says an undrawn field's remedy is "Redraw appearances".
//! That is **half true**, and the half was found by pressing the button on
//! `demo-form.pdf` rather than by any test:
//! `EditSession::regenerate_appearances` skips a text field whose `/V` is
//! absent, so it does nothing for an *empty* undrawn field. The remedy that
//! always works is a fill — `fill_text_field` writes the value and regenerates
//! every widget's `/AP`, so filling once in the panel makes the field clickable
//! on the page from then on. See
//! [`crate::text::forms::forms_canvas_undrawn_note`], which carries the
//! measurement.
//!
//! Read [`super`]'s header first. It carries the whole argument — why this is
//! not a [`CanvasTool`] variant, why the panel is not replaced, what the
//! editor cannot promise, how input layers, why the hit test takes no
//! tolerance, and the five reasons a field is routed to the panel instead.
//! This file is where those five reasons are actually decided
//! ([`classify`]), where the geometry is done
//! ([`crate::canvas::mapping::annot_canvas_rect`], which serves annotation
//! selection too) and
//! where the hit test lives ([`hit`]).

use egui::{Pos2, Rect, Vec2};
use pdfcer_core::forms::{AcroForm, ButtonKind, Field, FieldFlags, FieldType, FieldValue, Widget};
use pdfcer_core::object::ObjId;
use pdfcer_core::page_tree::Page;

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
/// Three variants, not five: choice fields and everything in
/// [`NotOnCanvas::NotOffered`] have no canvas gesture at all, so they are
/// absent rather than present-and-inert. The "no placeholders" invariant
/// applies to enums as much as to labels — a variant nothing can raise is dead
/// code wearing a design pattern.
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
    /// One variant for what used to be two — "no `/Rect`" and "no `/P`" — and
    /// the merge is the point rather than a tidy-up. See [`place`]'s ★ section:
    /// the question *"which page is this widget on?"* is now answered by
    /// walking each page's `/Annots`, so there is no `/P` to be absent, and a
    /// widget that no page lists is a widget with no place whatever its own
    /// dictionary says.
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
    /// What a click means.
    pub kind: BoxKind,
    /// Where it is, in canvas space.
    pub rect: Rect,
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
/// # ★ It asks nothing about geometry, and that is the change `widget_rects`
/// forced
///
/// This used to reject `Widget::rect` being absent or degenerate. It does not
/// any more, because the geometry a click is tested against no longer comes
/// from `Widget::rect` at all — see [`place`]. Asking here as well would be a
/// second source of truth for where a widget is, and the one that is *not* the
/// one being hit-tested.
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
/// This is the correction that matters most in this file, and it is a
/// correction: the first version of this walk read `pdfcer_core::forms::Widget::page`
/// — the widget's `/P` entry — and looked the page object up by id. It is the
/// obvious implementation and it is **silently wrong on a large class of real
/// files**.
///
/// `/P` is *Optional* (§12.5.2 Table 164). A widget that omits it is perfectly
/// conformant and is common in the wild, and `pdfcer-core` additionally reads
/// the key **without resolving through the graph**, so a direct rather than
/// indirect `/P` also reads as absent. Either way a `/P`-keyed placement
/// returns *nothing at all* for such a form: no error, no refusal, no trace —
/// a form on which clicking a field simply does not work, with the panel
/// cheerfully reporting every field as fillable.
///
/// **No test written against the fixture corpus can catch this.** All ten form
/// fixtures in `D:\Dev\pdfcer\fixtures\synthetic\forms\` write `/P` on every
/// widget, so the failing case is unreachable from them; the engine team hit
/// exactly this when a deliberate sabotage of their own implementation passed,
/// and had to build an in-memory form that omits the key. That is `HANDOFF.md`
/// §2's lesson in a new place — *a test that cannot reach the case is satisfied
/// by any implementation* — and it is why
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
            // ★★★ `classify` moved BELOW the rectangle when selection arrived,
            // and `reachable` DID NOT MOVE WITH IT. The distinction is the one
            // this whole change turns on and it is easy to get wrong — I got it
            // wrong first, and `an_undrawn_widget_is_still_selectable` is what
            // said so.
            //
            // `reachable` means *"some widget of this field can be FILLED on
            // the page"*, and it is what suppresses the panel's
            // `routing.undrawn` disclosure — the sentence that tells an operator
            // a field exists but has to be filled in the side panel. Setting it
            // beside the rectangle made every drawn-nothing field look
            // reachable, silently deleting that disclosure for the exact
            // documents it was written for.
            //
            // So: a rectangle makes a widget SELECTABLE; a successful
            // `classify` makes it FILLABLE; and only the second answers
            // `reachable`.
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
                    kind,
                    rect: canvas,
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
mod tests {
    use super::*;
    use pdfcer_core::object::Dict;
    use pdfcer_core::page_tree::Rect as PageRect;
    use pdfcer_core::vartext::Quadding;

    /// A terminal text field with one drawn widget, built by hand.
    ///
    /// By hand rather than from a fixture because the predicate under test is
    /// about combinations no single real document carries — a rich-text
    /// read-only field on a rotated page is not a document anybody shipped.
    fn text_field() -> Field {
        Field {
            id: ObjId::new(1, 0),
            fully_qualified_name: "Name".to_owned(),
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
            widgets: vec![drawn_widget()],
            merged: false,
            has_additional_actions: false,
            shares_parent_name: false,
            parent: None,
        }
    }

    /// A widget with a drawn appearance.
    ///
    /// # ★ `page: None` and `rect: None` are the point of this fixture
    ///
    /// Both keys are Optional in the spec and **both are frequently absent in
    /// real files** — and `pdfcer-core` additionally reads `/P` without
    /// resolving through the graph, so a direct rather than indirect `/P` also
    /// reads as absent. Every one of the ten form fixtures in
    /// `D:\Dev\pdfcer\fixtures\synthetic\forms\` writes `/P` on every widget, so
    /// a test built from a fixture cannot reach the case; the engine team hit
    /// exactly that when a deliberate sabotage of their own `/P` handling
    /// passed against their whole corpus.
    ///
    /// So the fixture omits both, and every assertion in this module is
    /// therefore also an assertion that neither is consulted. If someone
    /// reintroduces a `/P` lookup or a `Widget::rect` read, these tests stop
    /// passing here rather than stopping working in the field.
    fn drawn_widget() -> Widget {
        Widget {
            id: ObjId::new(2, 0),
            rect: None,
            appearance_state: None,
            on_states: Vec::new(),
            // ★ `rotation` arrived with the engine's `rotate_widget` Pass on
            // 2026-08-30. `None` here means the file states none, which is what
            // every fixture in this shell wants: a widget with no `/MK /R`.
            rotation: None,
            has_off_appearance: false,
            page: None,
            caption: None,
            // `Pass 146.0`'s three, in a test fixture: the file states no
            // border and no unusual flags. `None` is the honest value for a
            // synthetic widget — it means "this file says nothing", which is
            // exactly true of one built in a test.
            border: None,
            visibility: None,
            annot_flags: pdfcer_core::annot::AnnotFlags(0),
            has_normal_appearance: true,
            merged: false,
        }
    }

    /// The `/Rect` `EditSession::widget_rects` reports for [`drawn_widget`] —
    /// near the TOP of a 612×792 page in PDF terms, i.e. a large Y.
    const WIDGET_RECT: [f64; 4] = [100.0, 700.0, 300.0, 720.0];

    /// One page's `/Annots` listing, as the engine's verb hands it over.
    fn annots_listing(widget: &Widget) -> Vec<Vec<(ObjId, [f64; 4])>> {
        vec![vec![(widget.id, WIDGET_RECT)]]
    }

    /// A one-field form around `field`.
    fn form_of(field: Field) -> AcroForm {
        AcroForm {
            fields: vec![field],
            groups: Vec::new(),
            need_appearances: false,
            sig_flags: 0,
            signatures_exist: false,
            append_only: false,
            calc_order_count: 0,
            calc_order: Vec::new(),
            has_default_resources: false,
            default_appearance: None,
            quadding: Quadding::Left,
            xfa: pdfcer_core::forms::XfaPresence::None,
            inline_field_roots: 0,
        }
    }

    fn page(rotate: u16) -> Page {
        Page {
            id: ObjId::new(9, 0),
            resources: Dict::new(),
            media_box: PageRect::from_corners(0.0, 0.0, 612.0, 792.0),
            crop_box: PageRect::from_corners(0.0, 0.0, 612.0, 792.0),
            rotate,
            contents: Vec::new(),
            contents_unresolved: 0,
            contents_flattened: 0,
        }
    }

    /// ★ **A field with no `/AP` is not offered on the page.**
    ///
    /// The decision the module header §5.1 argues for, pinned. `demo-form.pdf`
    /// carries this case, and the failure if it regressed is the worst kind:
    /// an invisible click target over blank paper, which an operator can only
    /// find by accident and cannot find again.
    #[test]
    fn an_undrawn_widget_is_not_offered_on_the_canvas() {
        let field = text_field();
        let mut widget = drawn_widget();
        widget.has_normal_appearance = false;
        assert_eq!(
            classify(&field, &widget, 0),
            Err(NotOnCanvas::NoAppearance),
            "a widget the page draws nothing for must not be clickable"
        );
        // …and the drawn twin IS offered, or the assertion above passes on a
        // build where nothing is ever offered.
        assert!(classify(&field, &drawn_widget(), 0).is_ok());
    }

    /// ★ **A rotated page withholds the EDITOR, not the click.**
    ///
    /// Both halves, because the interesting content of the decision is the
    /// asymmetry: a text field cannot be edited in place on a `/Rotate 90`
    /// page (egui cannot rotate a `TextEdit`), while a check box has no text
    /// direction and is offered exactly as it is anywhere else. A build that
    /// refused both would be over-cautious in a way no operator could
    /// understand.
    #[test]
    fn a_rotated_page_withholds_a_text_editor_but_not_a_button() {
        let field = text_field();
        for rotate in [90u16, 180, 270] {
            assert_eq!(
                classify(&field, &drawn_widget(), rotate),
                Err(NotOnCanvas::RotatedPage),
                "rotate={rotate}"
            );
        }
        assert!(classify(&field, &drawn_widget(), 0).is_ok());

        let mut check = text_field();
        check.field_type = Some(FieldType::Button);
        check.button_kind = Some(ButtonKind::Check);
        let mut widget = drawn_widget();
        widget.on_states = vec![b"Yes".to_vec()];
        for rotate in [0u16, 90, 180, 270] {
            assert!(
                classify(&check, &widget, rotate).is_ok(),
                "a check box has no text direction: rotate={rotate}"
            );
        }
    }

    /// ★ **The canvas refuses exactly what the panel's `block_reason` refuses.**
    ///
    /// Asserted against the panel's own function rather than a re-derivation,
    /// so the test cannot pass by agreeing with a third copy of the rule. The
    /// silent failure it guards is two surfaces disagreeing about which fields
    /// are fillable — an operator clicking a field on the page that the panel
    /// says is read-only, or the reverse.
    #[test]
    fn the_canvas_declines_every_field_the_panel_blocks() {
        use pdfcer_core::forms::ButtonKind;

        let blocked = [
            Field {
                flags: FieldFlags(FieldFlags::READ_ONLY),
                ..text_field()
            },
            Field {
                field_type: Some(FieldType::Signature),
                ..text_field()
            },
            Field {
                field_type: Some(FieldType::Button),
                button_kind: Some(ButtonKind::Push),
                ..text_field()
            },
        ];
        for field in blocked {
            assert!(
                crate::panels::forms::rows::block_reason(&field).is_some(),
                "the fixture must be blocked for the assertion below to mean \
                 anything"
            );
            assert_eq!(
                classify(&field, &drawn_widget(), 0),
                Err(NotOnCanvas::NotOffered),
            );
        }

        // Rich text is the case `block_reason` deliberately does NOT cover —
        // the panel offers a conversion. There is no conversion gesture on a
        // page, so the canvas declines it on its own account.
        let rich = Field {
            flags: FieldFlags(FieldFlags::RICH_TEXT),
            ..text_field()
        };
        assert!(rich.is_rich_text());
        assert!(crate::panels::forms::rows::block_reason(&rich).is_none());
        assert_eq!(
            classify(&rich, &drawn_widget(), 0),
            Err(NotOnCanvas::NotOffered),
        );
    }

    /// ★ **A radio widget carries its OWN on-state, not the field's first.**
    ///
    /// The defect this prevents is a radio group in which every button selects
    /// the first option: the field's `/V` would be set to the same name
    /// whichever kid was clicked, and the group would look broken in a way
    /// that reads as an engine fault rather than a shell one.
    #[test]
    fn each_radio_widget_selects_its_own_state() {
        let mut field = text_field();
        field.field_type = Some(FieldType::Button);
        field.button_kind = Some(ButtonKind::Radio);
        field.value = FieldValue::Name(b"Blue".to_vec());

        let mut red = drawn_widget();
        red.on_states = vec![b"Red".to_vec()];
        let mut blue = drawn_widget();
        blue.on_states = vec![b"Blue".to_vec()];

        assert_eq!(
            classify(&field, &red, 0),
            Ok(BoxKind::Radio {
                on_state: "Red".to_owned(),
                on: false
            })
        );
        assert_eq!(
            classify(&field, &blue, 0),
            Ok(BoxKind::Radio {
                on_state: "Blue".to_owned(),
                on: true
            })
        );
    }

    /// A button with no on-state anywhere is not offered, because
    /// `set_button_state` would refuse every name but `Off`.
    #[test]
    fn a_button_with_no_on_state_is_not_offered() {
        let mut field = text_field();
        field.field_type = Some(FieldType::Button);
        field.button_kind = Some(ButtonKind::Check);
        assert_eq!(
            classify(&field, &drawn_widget(), 0),
            Err(NotOnCanvas::NotOffered)
        );
    }

    /// ★ **A widget with no `/P` entry is still placed** — the defect no
    /// fixture in the corpus can catch.
    ///
    /// `/P` is Optional (§12.5.2 Table 164) and frequently absent, and
    /// `pdfcer-core` reads it without resolving through the graph, so a direct
    /// `/P` reads as absent too. The obvious implementation of *"which page is
    /// this widget on?"* — look up `Widget::page` — therefore returns **nothing
    /// at all** on such a form: no error, no refusal, no trace, just a form on
    /// which clicking a field silently does not work.
    ///
    /// Every one of the ten form fixtures in
    /// `D:\Dev\pdfcer\fixtures\synthetic\forms\` writes `/P` on every widget, so
    /// a test opening a fixture cannot reach this. The engine team hit exactly
    /// that: a deliberate sabotage of their own `/P` handling passed against
    /// their whole corpus, and they had to build a form in memory that omits the
    /// key. This is that form, in this shell — [`drawn_widget`] omits `/P`, and
    /// the assertion is that the box is produced anyway.
    ///
    /// It is `HANDOFF.md` §2's lesson in a new place: **a test that cannot
    /// reach the case is satisfied by any implementation.**
    #[test]
    fn a_widget_with_no_p_entry_is_still_placed() {
        let field = text_field();
        assert!(
            field.widgets[0].page.is_none(),
            "the fixture must omit /P, or this test proves nothing"
        );

        let placed = place(
            &form_of(field),
            &[page(0)],
            &annots_listing(&drawn_widget()),
        );
        assert_eq!(
            placed.boxes.len(),
            1,
            "a widget with no /P must still be placed from its page's /Annots"
        );
        assert_eq!(placed.boxes[0].page, 0);
        assert_eq!(placed.routing, Routing::default());
    }

    /// A widget no page's `/Annots` lists has no place, and is counted as
    /// unreachable rather than silently dropped.
    #[test]
    fn a_widget_no_page_lists_is_unreachable_and_counted() {
        let placed = place(&form_of(text_field()), &[page(0)], &[Vec::new()]);
        assert!(placed.boxes.is_empty());
        assert_eq!(
            placed.routing,
            Routing {
                undrawn: 0,
                unreachable: 1
            }
        );
    }

    /// ★ **The hit test is containment, and it is exclusive between
    /// neighbours.**
    ///
    /// The property the no-tolerance decision buys, asserted as the thing an
    /// operator would notice: two fields one point apart — the ordinary shape
    /// of a form table — resolve to exactly one answer each, and the gutter
    /// between them resolves to neither. A six-point catch radius would make
    /// all three of these ambiguous.
    #[test]
    fn two_adjacent_fields_never_claim_each_others_clicks() {
        let boxes = vec![
            WidgetBox {
                page: 0,
                field: "A".to_owned(),
                widget: 0,
                kind: BoxKind::Text {
                    multiline: false,
                    password: false,
                    max_len: None,
                },
                rect: Rect::from_min_max(Pos2::new(10.0, 10.0), Pos2::new(60.0, 30.0)),
            },
            WidgetBox {
                page: 0,
                field: "B".to_owned(),
                widget: 0,
                kind: BoxKind::Text {
                    multiline: false,
                    password: false,
                    max_len: None,
                },
                rect: Rect::from_min_max(Pos2::new(61.0, 10.0), Pos2::new(110.0, 30.0)),
            },
        ];

        assert_eq!(hit(&boxes, 0, Pos2::new(59.5, 20.0)).unwrap().field, "A");
        assert_eq!(hit(&boxes, 0, Pos2::new(61.5, 20.0)).unwrap().field, "B");
        assert!(
            hit(&boxes, 0, Pos2::new(60.5, 20.0)).is_none(),
            "the gutter between two fields belongs to neither"
        );
        // A different page never answers, however well the point fits.
        assert!(hit(&boxes, 1, Pos2::new(30.0, 20.0)).is_none());
    }

    /// A widget drawn over another wins, because it is the one the operator
    /// can see.
    #[test]
    fn a_widget_drawn_over_another_claims_the_click() {
        let under = WidgetBox {
            page: 0,
            field: "Under".to_owned(),
            widget: 0,
            kind: BoxKind::Check {
                on_state: "Yes".to_owned(),
                on: false,
            },
            rect: Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(100.0, 100.0)),
        };
        let over = WidgetBox {
            field: "Over".to_owned(),
            rect: Rect::from_min_max(Pos2::new(40.0, 40.0), Pos2::new(60.0, 60.0)),
            ..under.clone()
        };
        let boxes = vec![under, over];
        assert_eq!(hit(&boxes, 0, Pos2::new(50.0, 50.0)).unwrap().field, "Over");
        assert_eq!(
            hit(&boxes, 0, Pos2::new(10.0, 10.0)).unwrap().field,
            "Under"
        );
    }

    /// ★ **A tiny field still gets a legible editor, and the box stays
    /// centred on it.**
    ///
    /// A 12 pt field at 25 % zoom is three screen points tall. Without the
    /// minimum the operator cannot read what they typed; without the centring,
    /// growing it would slide the box off the field it belongs to and the
    /// editor would appear to jump as the zoom changed.
    #[test]
    fn a_field_too_small_to_read_is_grown_about_its_own_centre() {
        let extent = (612.0_f32, 792.0);
        let map = PageMapping::new(
            Rect::from_min_size(Pos2::new(20.0, 20.0), egui::vec2(153.0, 198.0)),
            extent,
            0.25,
        );
        let canvas = Rect::from_min_max(Pos2::new(100.0, 100.0), Pos2::new(140.0, 112.0));
        let natural = map.rect_to_screen(canvas);
        assert!(
            natural.height() < MIN_EDITOR.y,
            "the fixture must be too small, or the test is vacuous: {natural:?}"
        );

        let grown = editor_rect(&map, canvas);
        assert!(grown.width() >= MIN_EDITOR.x && grown.height() >= MIN_EDITOR.y);
        assert!(
            (grown.center() - natural.center()).length() < 0.01,
            "growing must not move the box: {natural:?} -> {grown:?}"
        );

        // …and a field that is already big enough is not touched at all.
        let big = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(600.0, 300.0));
        assert_eq!(editor_rect(&map, big), map.rect_to_screen(big));
    }

    /// The editor's text size stays inside the legible band at every zoom,
    /// and grows with the box in between.
    #[test]
    fn the_editor_text_size_is_clamped_at_both_ends() {
        assert_eq!(editor_font_size(1.0), EDITOR_TEXT_RANGE.0);
        assert_eq!(editor_font_size(10_000.0), EDITOR_TEXT_RANGE.1);
        let small = editor_font_size(20.0);
        let large = editor_font_size(30.0);
        assert!(small < large, "{small} !< {large}");
    }

    /// `/MaxLen` truncates by character, not by byte.
    ///
    /// The byte version compiles, runs, and refuses an accented name three
    /// letters early — while also being able to split a character in half.
    #[test]
    fn max_len_counts_characters_not_bytes() {
        assert_eq!(truncate("Ångström", Some(4)), "Ångs");
        assert_eq!(truncate("Ångström", None), "Ångström");
        assert_eq!(truncate("abc", Some(10)), "abc");
        assert_eq!(truncate("", Some(0)), "");
    }

    /// ★ **Filling is offered in the select tool and in no other.**
    ///
    /// The whole of the "no `CanvasTool` variant" decision, expressed as the
    /// one line it is. The markup rows matter most: a pen that also filled a
    /// field would make one press mean two things, and the operator would
    /// discover it by finding text in a box they were drawing a rectangle
    /// over.
    #[test]
    fn only_the_select_tool_fills_a_form() {
        use crate::canvas::markup::MarkupKind;

        assert!(offered_in(CanvasTool::Select));
        assert!(!offered_in(CanvasTool::Hand));
        for &kind in MarkupKind::ALL {
            assert!(!offered_in(CanvasTool::Markup(kind)), "{kind:?}");
        }
    }

    /// ★ **A whole document's boxes, from a real form fixture.**
    ///
    /// The end-to-end shape of the read path — parse, place, project — on the
    /// document the panel's own disclosures were written against. It asserts
    /// what a screenshot cannot: that boxes exist at all, that each one names a
    /// field the form really has, and that each lands inside the page it
    /// claims.
    ///
    /// Note what it deliberately does **not** prove:
    /// [`a_widget_with_no_p_entry_is_still_placed`] exists because this test
    /// cannot reach the `/P`-absent case — every widget in this fixture carries
    /// `/P`, so a `/P`-keyed implementation would pass here.
    #[test]
    fn a_real_form_produces_boxes_inside_its_own_pages() {
        let doc = crate::app::state::open_fixture("forms/demo-form.pdf");
        let view = doc.session.view();
        let form = pdfcer_core::forms::parse_acroform(&view).expect("the fixture has a form");
        let pages = &doc.pages;
        let annots: Vec<Vec<(ObjId, [f64; 4])>> = (0..pages.len())
            .map(|page| doc.session.widget_rects(page))
            .collect();

        let placed = place(&form, pages, &annots);
        let boxes = &placed.boxes;
        assert!(
            !boxes.is_empty(),
            "a fillable form produced no clickable boxes at all"
        );

        let names: std::collections::BTreeSet<&str> = form
            .fields
            .iter()
            .map(|f| f.fully_qualified_name.as_str())
            .collect();
        for b in boxes {
            assert!(
                names.contains(b.field.as_str()),
                "{} is not a field of this form",
                b.field
            );
            assert!(b.page < pages.len(), "{} is on page {}", b.field, b.page);
            let (w, h) = crate::viewer::page_extent_pts(&pages[b.page]);
            let page_rect = Rect::from_min_size(Pos2::ZERO, egui::vec2(w, h));
            assert!(
                page_rect.expand(1.0).contains_rect(b.rect),
                "{}'s box {:?} is outside its own {w}x{h} page",
                b.field,
                b.rect
            );
        }

        // ★ And the fixture's undrawn field really is absent from the list —
        // the panel already discloses that this document has one, so this is
        // the same fact asserted from the other end.
        let undrawn: Vec<&str> = form
            .fields
            .iter()
            .filter(|f| !f.has_appearance())
            .map(|f| f.fully_qualified_name.as_str())
            .collect();
        for name in undrawn {
            assert!(
                !boxes.iter().any(|b| b.field == name),
                "{name} has no drawn appearance and must not be clickable"
            );
        }
    }

    /// ★ **A generator, not a check: build a form with a DRAWN text field.**
    ///
    /// ```text
    /// cargo test -p pdfcer-gui a_drawn_text_field_fixture -- --ignored --nocapture
    /// ```
    ///
    /// # Why this has to exist
    ///
    /// **Not one of the eleven form fixtures in
    /// `D:\Dev\pdfcer\fixtures\synthetic\forms\` carries a text field with a
    /// drawn appearance.** Measured by driving the binary over every one of
    /// them and reading the `form-box` census: `demo-form` yields one check
    /// box, `radio-choice-form` five radios, `radio-group-form` three radios,
    /// and the other eight yield **nothing at all** — every text field in the
    /// corpus is `/AP`-less, which is exactly the case §5.1 routes to the
    /// panel.
    ///
    /// So the corpus cannot exercise the in-place editor, which is the largest
    /// thing this feature adds. That is a fact about the fixtures rather than
    /// about the feature, and the honest response is to make the missing
    /// document rather than to declare the path verified because the tests are
    /// green — `HANDOFF.md` §2's whole subject.
    ///
    /// It is `#[ignore]` and writes outside the source tree, following
    /// `crate::shell::ron`'s generator precedent: a test that writes a file is
    /// run deliberately, never as part of a sweep.
    ///
    /// What it produces: `demo-form.pdf` with `Full name` filled. Filling is
    /// what draws it — `fill_text_field` writes `/V` **and** regenerates every
    /// widget's `/AP` — which is also the remedy
    /// [`crate::text::forms::forms_canvas_undrawn_note`] names.
    #[test]
    #[ignore = "generator: writes a PDF for driving the binary; run deliberately"]
    fn a_drawn_text_field_fixture() {
        use pdfcer_core::edit::EditSession;
        use pdfcer_core::writer::SaveOptions;

        let path = crate::panels::objects::test_support::engine_fixture("forms/demo-form.pdf");
        let doc = pdfcer_core::document::Document::load(&path).expect("the fixture loads");
        let mut session = EditSession::new(doc);
        // The first UNDRAWN text field, found rather than named: the fixture's
        // fully-qualified names are not the `/TU` labels the panel shows, and a
        // generator that hard-coded one would break the day the fixture is
        // regenerated.
        let target = {
            let view = session.view();
            let form = pdfcer_core::forms::parse_acroform(&view).expect("the fixture has a form");
            form.fields
                .iter()
                .find(|f| {
                    f.field_type == Some(FieldType::Text)
                        && !f.has_appearance()
                        && !f.flags.read_only()
                })
                .map(|f| f.fully_qualified_name.clone())
                .expect("demo-form has an undrawn text field")
        };
        println!("filling {target}");
        session
            .fill_text_field(&target, "Ken Mantle")
            .expect("an undrawn text field can be filled");

        let (bytes, _) = session
            .to_incremental_bytes(&SaveOptions::identity())
            .expect("an incremental save of one fill");
        let out = std::env::temp_dir().join("pdfcer-drawn-text-form.pdf");
        std::fs::write(&out, bytes).expect("the temp directory is writable");
        println!("wrote {}", out.display());

        // ★ …and the same document turned a quarter-turn, because the ROTATED
        // decision has no fixture either. §5.2 says a rotated page withholds
        // the text EDITOR and keeps the button click, and that asymmetry is
        // only checkable by opening a rotated form and reading the `form-box`
        // census: the check box must still appear and the text field must not.
        session
            .set_page_rotation(0, 90)
            .expect("a page can be turned");
        let (turned, _) = session
            .to_incremental_bytes(&SaveOptions::identity())
            .expect("an incremental save of a rotation");
        let rotated = std::env::temp_dir().join("pdfcer-drawn-text-form-rotated.pdf");
        std::fs::write(&rotated, turned).expect("the temp directory is writable");
        println!("wrote {}", rotated.display());

        // Prove the point of the exercise: the field is now DRAWN, so the
        // canvas will offer it. If this ever stops being true the generator is
        // producing a document that does not test what it exists to test.
        let view = session.view();
        let form = pdfcer_core::forms::parse_acroform(&view).expect("the form survived");
        let field = form
            .fields
            .iter()
            .find(|f| f.fully_qualified_name == target)
            .expect("the field survived");
        assert!(
            field.has_appearance(),
            "filling must draw the field, or the generator produces nothing new"
        );
    }

    /// ★★★ **A kind that cannot be FILLED on the canvas can still be
    /// SELECTED there**, which is the whole reason [`FieldTarget`] exists.
    ///
    /// A drop-down (`/Ch`) is `NotOffered` — this shell has no canvas gesture
    /// for one, and `classify` refuses it. Before selection existed, that
    /// refusal removed it from the only list the canvas hit-tested, so a field
    /// the operator could plainly see was not clickable at all.
    ///
    /// The two assertions are deliberately opposite, because a test that only
    /// checked the target would pass against a change that made every widget
    /// fillable — which is a different bug with the same symptom on this test.
    #[test]
    fn a_choice_field_is_not_fillable_on_the_canvas_but_is_selectable() {
        let mut field = text_field();
        field.field_type = Some(FieldType::Choice);
        let widget = field.widgets[0].clone();
        let placed = place(&form_of(field), &[page(0)], &annots_listing(&widget));

        assert!(
            placed.boxes.is_empty(),
            "a drop-down has no canvas fill gesture and must not offer one"
        );
        assert_eq!(
            placed.targets.len(),
            1,
            "…and must still be selectable, or its properties are unreachable \
             from the page it is drawn on"
        );
    }

    /// **A widget with no appearance is selectable too.**
    ///
    /// `NoAppearance` routes a field to the panel for FILLING because the page
    /// draws nothing there — but pdfcer authors widgets, and a widget it made
    /// and then failed to draw is exactly the one an operator needs to reach in
    /// order to delete it. The rectangle is real even when the appearance is
    /// not.
    #[test]
    fn an_undrawn_widget_is_still_selectable() {
        let mut field = text_field();
        field.widgets[0].has_normal_appearance = false;
        let widget = field.widgets[0].clone();
        let placed = place(&form_of(field), &[page(0)], &annots_listing(&widget));

        assert!(
            placed.boxes.is_empty(),
            "nothing is drawn there to type into"
        );
        assert_eq!(placed.routing.undrawn, 1, "and the panel is told why");
        assert_eq!(
            placed.targets.len(),
            1,
            "but it occupies a rectangle, so it can be selected and deleted"
        );
    }

    /// **A widget no page lists is selectable from nowhere**, because it has no
    /// rectangle to click. The one exclusion that is not a policy.
    #[test]
    fn an_unplaced_widget_is_not_selectable() {
        let field = text_field();
        let placed = place(&form_of(field), &[page(0)], &[vec![]]);
        assert!(placed.targets.is_empty());
        assert_eq!(placed.routing.unreachable, 1);
    }

    /// **The hit test takes the widget drawn on top**, the same rule the fill
    /// hit test follows, because it is the same question: which one can the
    /// operator see?
    #[test]
    fn the_selection_hit_test_prefers_the_widget_drawn_last() {
        let under = FieldTarget {
            page: 0,
            field: "Under".to_owned(),
            widget: 0,
            rect: Rect::from_min_size(Pos2::new(0.0, 0.0), egui::vec2(100.0, 100.0)),
        };
        let over = FieldTarget {
            field: "Over".to_owned(),
            ..under.clone()
        };
        let targets = vec![under, over];
        assert_eq!(
            hit_target(&targets, 0, Pos2::new(50.0, 50.0)).map(|t| t.field.as_str()),
            Some("Over")
        );
        assert!(
            hit_target(&targets, 1, Pos2::new(50.0, 50.0)).is_none(),
            "wrong page"
        );
        assert!(
            hit_target(&targets, 0, Pos2::new(150.0, 50.0)).is_none(),
            "outside"
        );
    }
}
