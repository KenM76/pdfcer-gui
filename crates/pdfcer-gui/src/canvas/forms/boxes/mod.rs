//! # `canvas::forms::boxes` — where a form's widgets are, and what a click on
//! one would mean
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/forms/boxes/mod.md`.

use egui::{Pos2, Rect, Vec2};
use pdfcer_core::forms::{
    AcroForm, ButtonKind, Field, FieldFlags, FieldType, FieldValue, MkColor, Widget,
};
use pdfcer_core::object::ObjId;
use pdfcer_core::page_tree::Page;
use pdfcer_core::vartext::Quadding;

use crate::canvas::mapping::PageMapping;
use crate::canvas::tool::CanvasTool;

pub use pdfcer_gui_base::formeditortext::{editor_align, editor_font_size};
#[cfg(test)]
use {egui::Align, pdfcer_gui_base::formeditortext::EDITOR_TEXT_RANGE};

/// The smallest an editor may be drawn, in **screen** points.
const MIN_EDITOR: Vec2 = Vec2::new(60.0, 18.0);

// ===========================================================================
// What a widget is, on screen
// ===========================================================================

/// What clicking a widget means.
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
        /// ## Why the editor honours this when it honours no other
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
        /// ## The background colour is honoured too, and by the same test
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
        /// It decides where the options are drawn, and the two are not
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
        /// Both bits, never bit 19 alone. The spec's own words for bit 19
        /// are *"used only with Combo"*, so `Edit` on a list box is a
        /// meaningless bit rather than a meaningful one, and a decoder that
        /// honoured it would put a caret into a surface whose whole behaviour
        /// is choosing from rows. `pdfcer-core`'s `set_choice_value` gates its
        /// free-text branch on the same conjunction
        /// (`edit.rs`, `editable_combo`), so a shell that disagreed here would
        /// offer typing into a field the engine then refuses with
        /// `ChoiceValueNotInOptions`.
        ///
        /// What it changes on the page is the **whole control**, not a
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
    /// merge is the point. See [`place`]'s section: the question *"which
    /// page is this widget on?"* is answered by walking each page's `/Annots`,
    /// so there is no `/P` to be absent, and a widget that no page lists is a
    /// widget with no place whatever its own dictionary says.
    NotPlaced,
    /// This field kind has no canvas gesture.
    NotOffered,
}

/// How many of a form's fillable fields have to be filled in the **panel**,
/// and why.
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
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Placed {
    /// Every fillable widget, in canvas space.
    pub boxes: Vec<WidgetBox>,
    /// What could not be placed, and why.
    pub routing: Routing,
    /// Every widget with a rectangle, fillable or not — what the **authoring**
    /// surface hit-tests. See [`FieldTarget`].
    pub targets: Vec<FieldTarget>,
    /// Every unsigned signature widget — what gets the red tag and the
    /// click-to-sign (O266).
    pub unsigned: Vec<FieldTarget>,
}

/// One fillable widget, placed.
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
#[must_use]
pub fn offered_in(tool: CanvasTool) -> bool {
    matches!(tool, CanvasTool::Select) || types_into_fields(tool)
}

/// Whether this tool is a text tool, whose click on a field fills it even
/// where the Select tool would select the field for authoring.
#[must_use]
pub fn types_into_fields(tool: CanvasTool) -> bool {
    matches!(
        tool,
        CanvasTool::Text | CanvasTool::TextEdit(crate::canvas::textedit::TextEditKind::Edit)
    )
}

/// The widget's own background colour as sRGB components, or `None` for
/// "leave the theme's box alone".
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
    let mut unsigned: Vec<(usize, FieldTarget)> = Vec::new();
    let mut routing = Routing::default();
    for field in &form.fields {
        let signable =
            field.field_type == Some(FieldType::Signature) && field.value == FieldValue::Absent;
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
            // SELECTABLE FROM HERE, and the position of this push is the
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
            if signable && let Some((order, target)) = targets.last() {
                unsigned.push((*order, target.clone()));
            }
            // `reachable` is answered by `classify` and NOT by the
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
    unsigned.sort_by_key(|(order, t)| (t.page, *order));
    Placed {
        boxes: out.into_iter().map(|(_, b)| b).collect(),
        routing,
        targets: targets.into_iter().map(|(_, t)| t).collect(),
        unsigned: unsigned.into_iter().map(|(_, t)| t).collect(),
    }
}

/// Which box a canvas-space point is inside, on `page`.
#[must_use]
pub fn hit(boxes: &[WidgetBox], page: usize, point: Pos2) -> Option<&WidgetBox> {
    boxes
        .iter()
        .rfind(|b| b.page == page && b.rect.contains(point))
}

/// Which **selectable** widget a canvas-space point is inside, on `page`.
#[must_use]
pub fn hit_target(targets: &[FieldTarget], page: usize, point: Pos2) -> Option<&FieldTarget> {
    targets
        .iter()
        .rfind(|t| t.page == page && t.rect.contains(point))
}

/// The editor's rect on screen: the widget's own, grown to [`MIN_EDITOR`].
#[must_use]
pub fn editor_rect(map: &PageMapping, canvas: Rect) -> Rect {
    let screen = map.rect_to_screen(canvas);
    let grow = Vec2::new(
        (MIN_EDITOR.x - screen.width()).max(0.0) / 2.0,
        (MIN_EDITOR.y - screen.height()).max(0.0) / 2.0,
    );
    screen.expand2(grow)
}

/// Truncate a draft to `/MaxLen`, in **characters**.
#[must_use]
pub fn truncate(draft: &str, max_len: Option<usize>) -> String {
    match max_len {
        Some(max) if draft.chars().count() > max => draft.chars().take(max).collect(),
        _ => draft.to_owned(),
    }
}

#[cfg(test)]
mod tests;
