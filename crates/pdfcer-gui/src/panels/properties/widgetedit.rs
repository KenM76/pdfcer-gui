//! # `panels::properties::widgetedit` — the **box** a form field is drawn in,
//! as opposed to the field itself
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/properties/widgetedit.md`.

use egui::Ui;
use pdfcer_core::edit::WidgetEdit;
use pdfcer_core::forms::{Field, Widget};
use pdfcer_core::page_tree::Rect;
use pdfcer_gui_base::entry;

use super::mkcolour;
use crate::app::actions::Action;
use crate::app::actions::forms::FieldAction;
use crate::panels::PanelsState;
use crate::text::panels::formfield as t;

/// Every number in this section is a length in points: shows `pt`, reads any unit.
const POINTS: entry::Kind = entry::Kind::Length(entry::LengthUnit::Point);

/// The section's rect, for `ui-verify`.
///
/// Plain [`crate::diag::ui_rect`], not the visibility-gated form, for the
/// reason [`super::fieldedit`]'s own note records at length: a **section** rect
/// answers *"did this draw?"* and *"where do I scroll?"*, and gating it on
/// 60 % visibility deletes it exactly when the section is taller than its dock
/// slot. The per-control regions below take the gated form, because a check
/// clicks those.
// ui-text-exempt: trace region name, never displayed
pub const REGION: &str = "properties.widget_edit";
/// The four geometry spinners' shared region prefix.
// ui-text-exempt: trace region name, never displayed
pub const GEOMETRY_REGION: &str = "properties.widget_edit.geometry";
/// The border-style combo's region.
// ui-text-exempt: trace region name, never displayed
pub const BORDER_REGION: &str = "properties.widget_edit.border";
/// The border combo's *No border* entry, drawn while the combo is open.
// ui-text-exempt: trace region name, never displayed
pub const BORDER_NONE_REGION: &str = "properties.widget_edit.border.none";
/// The border-width spinner's region.
// ui-text-exempt: trace region name, never displayed
pub const BORDER_WIDTH_REGION: &str = "properties.widget_edit.border_width";
/// The background swatch's region. `O202`.
// ui-text-exempt: trace region name, never displayed
pub const BACKGROUND_REGION: &str = "properties.widget_edit.background";
/// The border-and-mark swatch's region. `O202`.
// ui-text-exempt: trace region name, never displayed
pub const BORDER_COLOR_REGION: &str = "properties.widget_edit.border_color";

/// The rotation row, for layout checks.
pub const ROTATION_REGION: &str = "properties.widget_edit.rotation";

/// The two rotation buttons, EACH named.
pub const ROTATE_LEFT_REGION: &str = "properties.widget_edit.rotate_left";
/// See [`ROTATE_LEFT_REGION`].
pub const ROTATE_RIGHT_REGION: &str = "properties.widget_edit.rotate_right";
/// The visibility combo's region.
// ui-text-exempt: trace region name, never displayed
pub const VISIBILITY_REGION: &str = "properties.widget_edit.visibility";
/// The Apply button — the one control a driven check presses.
// ui-text-exempt: trace region name, never displayed
pub const APPLY_REGION: &str = "properties.widget_edit.apply";

/// How fast a drag on one of the four spinners moves it, in points per pixel.
const SPEED: f64 = 0.25;

/// Draw the selected widget's own properties, or nothing.
pub fn section(
    ui: &mut Ui,
    field: &Field,
    fqn: &str,
    widget_index: usize,
    state: &mut PanelsState,
    epoch: u64,
    actions: &mut Vec<Action>,
) -> bool {
    let Some(widget) = field.widgets.get(widget_index) else {
        return false;
    };
    let Some(rect) = widget.rect else {
        // A widget with no readable `/Rect` renders nothing this pane could
        // describe, and a zero-area rect is *intentional* invisibility for a
        // signature field (§12.7.4.5) rather than a defect — so `None` here is
        // the malformed case only. Silence: four spinners seeded from nothing
        // would invite a press that writes an invented box.
        return false;
    };

    let draft = state.widget_props_mut();
    draft.read(widget, rect, fqn, widget_index, epoch);

    ui.label(t::widget_heading());
    // Said only when there is more than one placement, because that is the
    // only state in which the scope distinction is visible — and it is
    // precisely the state in which an operator would otherwise expect this
    // section to behave like the one above it.
    if field.widgets.len() > 1 {
        ui.small(t::widget_scope_note(field.widgets.len()));
    }
    ui.add_space(2.0);

    geometry_rows(ui, draft, actions, fqn, widget_index);
    ui.add_space(4.0);
    //
    // It IS geometry — an operator adjusting where a box is and how big it is is
    // in the same thought as which way round it faces, and the caption is a
    // different subject entirely.
    //
    // And it was measured unreachable at the bottom: with every section drawn
    // the control landed at `y=1379` in a window 768 points tall. The panel
    // scrolls, so it was not lost the way the bookmarks controls were — but a
    // control an operator has to scroll past four unrelated sections to reach is
    // one they will not find, and a driven check could not reach it either.
    rotation_row(ui, widget, fqn, widget_index, actions);
    ui.add_space(4.0);
    border_rows(ui, field, widget, fqn, widget_index, actions);
    ui.add_space(4.0);
    // Directly under the border STYLE and WIDTH, because the three are one
    // thought and `/BC` is the ink the style is stroked in. Above
    // visibility, which is a different subject.
    chrome_rows(ui, field, widget, fqn, widget_index, actions);
    ui.add_space(4.0);
    visibility_row(ui, widget, fqn, widget_index, actions);
    ui.add_space(4.0);
    caption_row(ui, draft, actions, fqn, widget_index);
    super::buttonicon::rows(ui, field, widget, fqn, widget_index, actions);

    crate::diag::ui_rect(REGION, ui.min_rect());
    true
}

/// **Turn the box**, in ninety-degree steps.
fn rotation_row(
    ui: &mut Ui,
    widget: &pdfcer_core::forms::Widget,
    fqn: &str,
    index: usize,
    actions: &mut Vec<Action>,
) {
    let current = widget.rotation.unwrap_or(0);
    ui.label(t::widget_rotation_label(widget.rotation));

    let mut turn = |ui: &mut Ui, label: &str, region: &str, delta: i64| {
        let response = ui.button(label);
        //
        // This module's own header states the rule two hundred lines above:
        // *"the per-control regions below take the gated form, because a check
        // clicks those."* `rotation_row` used the plain one, so the trace
        // published a rect at the button's **content** position — y = 1,253 in
        // a 758-point window — and the driven check aimed the real pointer at a
        // coordinate outside the window, pressed nothing, and reported the
        // feature as inert.
        //
        // ⇒ A harness limitation reporting as an application defect, which is
        // the failure mode `tools/ui-verify` exists to remove rather than
        // produce. Gated, an off-screen button is **absent** from the trace,
        // which is a fact a check can act on: scroll to it, or say it cannot be
        // reached.
        crate::diag::ui_rect_visible(region, response.rect, ui.clip_rect());
        if response.clicked() {
            // Normalised HERE as well as by the engine, so the number in the
            // trace is the one the file will carry. `rotate_widget` accepts any
            // multiple of 90 and normalises into 0..360 itself — this is not
            // guarding against it, it is making the two agree so a driven check
            // reading either sees the same value.
            let next = (current + delta).rem_euclid(360);
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!(
                    "widget-rotate-requested field={fqn:?} widget={index} was={current} now={next}"
                )
            });
            actions.push(Action::Field(FieldAction::RotateWidget {
                field: fqn.to_owned(),
                widget: index,
                degrees: next,
            }));
        }
    };

    let response = ui
        .horizontal(|ui| {
            // LEFT is +90 counterclockwise and RIGHT is -90, and that is the
            // negation the engine asked for. It happens on this line and
            // nowhere else.
            turn(ui, t::widget_rotate_left(), ROTATE_LEFT_REGION, 90);
            turn(ui, t::widget_rotate_right(), ROTATE_RIGHT_REGION, -90);
        })
        .response;
    // Gated for the reason the two buttons above are: a check reads this row to
    // learn the current angle, and a readout published where nobody can see it
    // is a readout that answers a question about a different screen.
    crate::diag::ui_rect_visible(ROTATION_REGION, response.rect, ui.clip_rect());
    ui.small(t::widget_rotation_hint());
}

/// The four typed numbers and the button that commits them.
fn geometry_rows(
    ui: &mut Ui,
    draft: &mut WidgetPropsDraft,
    actions: &mut Vec<Action>,
    fqn: &str,
    widget_index: usize,
) {
    let spinner = |ui: &mut Ui, label: &str, key: &str, value: &mut f64| {
        ui.horizontal(|ui| {
            ui.label(label);
            let (widget, refusal) = entry::drag_value(ui, value, POINTS);
            let response = refusal.show(ui.add(widget.speed(SPEED).fixed_decimals(2)));
            crate::diag::ui_rect_visible(
                &format!("{GEOMETRY_REGION}.{key}"),
                response.rect,
                ui.clip_rect(),
            );
        });
    };
    // ui-text-exempt: trace region keys, never displayed.
    spinner(ui, t::label_widget_x(), "x", &mut draft.x);
    spinner(ui, t::label_widget_y(), "y", &mut draft.y);
    spinner(ui, t::label_widget_w(), "w", &mut draft.w);
    spinner(ui, t::label_widget_h(), "h", &mut draft.h);

    let changed = draft.differs();
    let apply = ui.add_enabled(changed, egui::Button::new(t::widget_apply()));
    crate::diag::ui_rect_visible(APPLY_REGION, apply.rect, ui.clip_rect());
    let apply = if changed {
        // The hover names which of the two acts is about to happen, because
        // the consequences differ and the operator has already decided: a move
        // keeps the baked artwork exact, a resize rebuilds it and may fail to.
        apply.on_hover_text(t::widget_apply_hover(draft.resizes()))
    } else {
        apply.on_disabled_hover_text(t::widget_apply_disabled())
    };
    if apply.clicked() {
        actions.push(
            FieldAction::EditWidget {
                field: fqn.to_owned(),
                widget: widget_index,
                //
                // Read from the SAME store the Tool row writes and the grip
                // drag reads (`canvas::scaling::read`), not from a second
                // setting of this panel's own. The operator answered the
                // question once; a typed resize and a dragged one are two
                // routes to one act, and a route that quietly used a different
                // answer would be the *"adding a second route is an audit of
                // the capability"* finding arriving as a defect instead.
                edit: WidgetEdit::new()
                    .with_rect(Rect::from_corners(
                        draft.x,
                        draft.y,
                        draft.x + draft.w,
                        draft.y + draft.h,
                    ))
                    .with_resize(crate::canvas::scaling::read(ui.ctx()).to_options()),
                touched: t::touched_box(),
            }
            .into(),
        );
    }
}

/// The border's style and width — `/BS`, `Pass 146.0`.
fn border_rows(
    ui: &mut Ui,
    field: &Field,
    widget: &Widget,
    fqn: &str,
    widget_index: usize,
    actions: &mut Vec<Action>,
) {
    use pdfcer_core::forms::ButtonKind;
    // A check box's or radio's mark is drawn in `/BC` (the engine's builders
    // read it as the control's ink), so *No border* keeps that colour there.
    let keeps_mark = matches!(
        field.button_kind,
        Some(ButtonKind::Check | ButtonKind::Radio)
    );
    trace_shown(ui.ctx(), fqn, widget_index, widget);
    border_style_row(ui, widget, keeps_mark, fqn, widget_index, actions);
    // The width is offered only once the file has a border to widen. A
    // spinner over `border: None` would have to show *something*, and any
    // number it showed would be the invention.
    if let Some(border) = widget.border {
        border_width_row(ui, border, fqn, widget_index, actions);
    }
}

/// Whether `border` draws no frame: no stated border reads as the standard's
/// width of 1, so only a stated width of 0 is none.
fn borderless(border: Option<pdfcer_core::edit::BorderSpec>) -> bool {
    border.is_some_and(|b| b.width <= 0.0)
}

/// Trace the border the panel shows, once per change: what a re-read of the
/// document says after an edit, which is what a driven check asserts on.
fn trace_shown(ctx: &egui::Context, fqn: &str, widget_index: usize, widget: &Widget) {
    let shown = format!(
        // ui-text-exempt: diagnostic trace, never displayed
        "widget-border-shown field={fqn} widget={widget_index} border={} width={} \
         border_colour={}",
        if borderless(widget.border) {
            "none"
        } else if widget.border.is_some() {
            "stated"
        } else {
            "unstated"
        },
        widget
            .border
            .map_or_else(|| "-".to_owned(), |b| format!("{:.2}", b.width)),
        if widget.border_color.is_some() {
            "present"
        } else {
            "absent"
        },
    );
    let id = egui::Id::new("widget-border-shown"); // ui-text-exempt: memory key, never displayed
    let last: Option<String> = ctx.data(|d| d.get_temp(id));
    if last.as_deref() != Some(shown.as_str()) {
        crate::diag::trace(|| shown.clone());
        ctx.data_mut(|d| d.insert_temp(id, shown));
    }
}

/// The style combo, with *No border* first.
fn border_style_row(
    ui: &mut Ui,
    widget: &Widget,
    keeps_mark: bool,
    fqn: &str,
    widget_index: usize,
    actions: &mut Vec<Action>,
) {
    use pdfcer_core::edit::{BorderSpec, BorderStyle};
    // The five pdfcer can write. Not `BorderStyle`'s variants enumerated by
    // hand somewhere else: this is the list the engine's own `edit_widget`
    // accepts, and offering a sixth would be a control whose press is refused.
    const STYLES: [BorderStyle; 5] = [
        BorderStyle::Solid,
        BorderStyle::Dashed,
        BorderStyle::Beveled,
        BorderStyle::Inset,
        BorderStyle::Underline,
    ];

    let current = widget.border;
    let none = borderless(current);
    let mut push = |edit: WidgetEdit| {
        actions.push(
            FieldAction::EditWidget {
                field: fqn.to_owned(),
                widget: widget_index,
                edit,
                touched: t::touched_border(),
            }
            .into(),
        );
    };
    ui.horizontal(|ui| {
        ui.label(t::label_border());
        let shown = if none {
            t::border_none()
        } else {
            current.map_or_else(t::border_unstated, |b| t::border_style_label(b.style))
        };
        let combo = egui::ComboBox::from_id_salt("widget-border-style")
            .selected_text(shown)
            .show_ui(ui, |ui| {
                let entry = ui
                    .selectable_label(none, t::border_none())
                    .on_hover_text(t::border_none_hover(keeps_mark));
                crate::diag::ui_rect_visible(BORDER_NONE_REGION, entry.rect, ui.clip_rect());
                if entry.clicked() && !none {
                    // Width 0 is the standard's own "no border shall be
                    // drawn"; the style is kept for a later style choice.
                    let style = current.map_or(BorderStyle::Solid, |b| b.style);
                    let edit = border_edit(BorderSpec { style, width: 0.0 });
                    push(if keeps_mark {
                        edit
                    } else {
                        edit.without_border_color()
                    });
                }
                for style in STYLES {
                    let selected = !none && current.is_some_and(|b| b.style == style);
                    if ui
                        .selectable_label(selected, t::border_style_label(style))
                        .clicked()
                        && !selected
                    {
                        // The width travels with the style, because `/BS` is
                        // one dictionary and `BorderSpec` is one value. No
                        // stated border, or none, gets the standard's
                        // default of 1: choosing a style is the operator
                        // committing to a border that shows.
                        let width = current.map_or(1.0, |b| b.width);
                        let width = if width > 0.0 { width } else { 1.0 };
                        push(border_edit(BorderSpec { style, width }));
                    }
                }
            });
        crate::diag::ui_rect_visible(BORDER_REGION, combo.response.rect, ui.clip_rect());
    });
}

/// A border edit. It lets the engine redraw a check box or radio button
/// another program drew, because otherwise that artwork keeps its old frame
/// and the change shows nothing; the status line says when it happened.
fn border_edit(border: pdfcer_core::edit::BorderSpec) -> WidgetEdit {
    WidgetEdit::new()
        .with_border(border)
        .with_foreign_appearance(pdfcer_core::edit::ForeignAppearance::Replace)
}

/// The width spinner, for a widget with a stated border.
fn border_width_row(
    ui: &mut Ui,
    border: pdfcer_core::edit::BorderSpec,
    fqn: &str,
    widget_index: usize,
    actions: &mut Vec<Action>,
) {
    use pdfcer_core::edit::BorderSpec;
    ui.horizontal(|ui| {
        ui.label(t::label_border_width());
        let mut width = border.width;
        let (widget, refusal) = entry::drag_value(ui, &mut width, POINTS);
        let response = refusal.show(ui.add(widget.speed(0.25).range(0.0..=72.0).fixed_decimals(2)));
        crate::diag::ui_rect_visible(BORDER_WIDTH_REGION, response.rect, ui.clip_rect());
        let response = response.on_hover_text(t::label_border_width_hover());
        if (response.drag_stopped() || response.lost_focus()) && !near(width, border.width) {
            actions.push(
                FieldAction::EditWidget {
                    field: fqn.to_owned(),
                    widget: widget_index,
                    edit: border_edit(BorderSpec {
                        style: border.style,
                        width,
                    }),
                    touched: t::touched_border_width(),
                }
                .into(),
            );
        }
    });
}

/// Where the widget is visible — `/F`, `Pass 146.0`.
fn visibility_row(
    ui: &mut Ui,
    widget: &Widget,
    fqn: &str,
    widget_index: usize,
    actions: &mut Vec<Action>,
) {
    use pdfcer_core::edit::Visibility;
    const SHOWN: [Visibility; 4] = [
        Visibility::VisibleAndPrints,
        Visibility::ScreenOnly,
        Visibility::PrintOnly,
        Visibility::Hidden,
    ];

    let Some(current) = widget.visibility else {
        ui.label(t::label_visibility());
        ui.small(t::visibility_unmappable(widget.annot_flags.0));
        return;
    };
    ui.horizontal(|ui| {
        ui.label(t::label_visibility());
        let combo = egui::ComboBox::from_id_salt("widget-visibility")
            .selected_text(t::visibility_label(current))
            .show_ui(ui, |ui| {
                for choice in SHOWN {
                    if ui
                        .selectable_label(choice == current, t::visibility_label(choice))
                        .clicked()
                        && choice != current
                    {
                        actions.push(
                            FieldAction::EditWidget {
                                field: fqn.to_owned(),
                                widget: widget_index,
                                edit: WidgetEdit::new().with_visibility(choice),
                                touched: t::touched_visibility(),
                            }
                            .into(),
                        );
                    }
                }
            });
        crate::diag::ui_rect_visible(VISIBILITY_REGION, combo.response.rect, ui.clip_rect());
    });
}

/// `/MK` `/CA` — the widget's caption.
fn caption_row(
    ui: &mut Ui,
    draft: &mut WidgetPropsDraft,
    actions: &mut Vec<Action>,
    fqn: &str,
    widget_index: usize,
) {
    ui.label(t::label_caption());
    let response = ui.add(
        // escape-disposition: commits — `lost_focus` and a changed caption.
        // Empty commits `Some("")`, which removes it.
        egui::TextEdit::singleline(&mut draft.caption)
            .desired_width(f32::INFINITY)
            .hint_text(t::label_caption_hint()),
    );
    let typed = draft.caption.trim().to_owned();
    if response.lost_focus() && typed != draft.caption_stored {
        actions.push(
            FieldAction::EditWidget {
                field: fqn.to_owned(),
                widget: widget_index,
                edit: WidgetEdit::new().with_caption(typed),
                touched: t::touched_caption(),
            }
            .into(),
        );
    }
}

/// The two `/MK` colours — `/BG` (background) and `/BC` (border and mark).
/// `OPERATOR_REQUESTS.md` **O202**, the after-placement half.
fn chrome_rows(
    ui: &mut Ui,
    field: &Field,
    widget: &Widget,
    fqn: &str,
    widget_index: usize,
    actions: &mut Vec<Action>,
) {
    use pdfcer_core::forms::ButtonKind;

    // The disc is a fact about the ENGINE's radio builder, which fills a circle
    // and says so, not a decoration chosen here. A rectangular preview over a
    // control that comes out round is this panel mis-stating the result of the
    // operator's own press.
    let disc = field.button_kind == Some(ButtonKind::Radio);

    chrome_row(
        ui,
        &mkcolour::Row {
            label: t::label_background(),
            hover: t::label_background_hover(),
            // ui-text-exempt: egui id salt and trace region key, never displayed.
            id_salt: "widget-background",
            region: BACKGROUND_REGION,
            colour: widget.background,
            unstated_note: t::background_unstated_note(),
            no_colour_note: t::background_no_colour_note(),
            no_colour_entry: Some(t::background_no_colour_entry()),
            no_colour_unavailable: t::background_no_colour_unavailable(),
            remove_entry: Some(t::background_remove_entry()),
            remove_unavailable: t::background_remove_unavailable(),
            disc,
        },
        t::touched_background(),
        fqn,
        widget_index,
        actions,
        Setters {
            build: WidgetEdit::with_background,
            strip: WidgetEdit::without_background,
        },
    );

    chrome_row(
        ui,
        &mkcolour::Row {
            label: t::label_border_colour(),
            hover: t::label_border_colour_hover(),
            // ui-text-exempt: egui id salt and trace region key, never displayed.
            id_salt: "widget-border-colour",
            region: BORDER_COLOR_REGION,
            colour: widget.border_color,
            unstated_note: t::border_colour_unstated_note(),
            no_colour_note: t::border_colour_no_colour_note(),
            no_colour_entry: None,
            no_colour_unavailable: "",
            remove_entry: Some(t::border_colour_remove_entry()),
            remove_unavailable: t::border_colour_remove_unavailable(),
            disc: false,
        },
        t::touched_border_colour(),
        fqn,
        widget_index,
        actions,
        Setters {
            build: WidgetEdit::with_border_color,
            strip: WidgetEdit::without_border_color,
        },
    );
}

/// The two `WidgetEdit` verbs that write one `/MK` colour key.
struct Setters {
    /// Write a colour, the empty array included.
    build: fn(WidgetEdit, pdfcer_core::forms::MkColor) -> WidgetEdit,
    /// Take the key out of the file.
    strip: fn(WidgetEdit) -> WidgetEdit,
}

/// Draw one `/MK` colour row and queue the edit it produces.
fn chrome_row(
    ui: &mut Ui,
    row: &mkcolour::Row<'_>,
    touched: &'static str,
    fqn: &str,
    widget_index: usize,
    actions: &mut Vec<Action>,
    setters: Setters,
) {
    let Some(pick) = mkcolour::row(ui, row) else {
        return;
    };
    let edit = match pick {
        mkcolour::Pick::Set(colour) => (setters.build)(WidgetEdit::new(), colour),
        mkcolour::Pick::Remove => (setters.strip)(WidgetEdit::new()),
    };
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "widget-colour-requested field={fqn:?} widget={widget_index} key={} was={:?} now={pick:?}",
            row.region, row.colour
        )
    });
    actions.push(
        FieldAction::EditWidget {
            field: fqn.to_owned(),
            widget: widget_index,
            edit,
            touched,
        }
        .into(),
    );
}

/// The typed box and caption, and the widget they were read for.
#[derive(Default)]
pub struct WidgetPropsDraft {
    /// `(field name, widget index, edit epoch)` the values below were read at.
    ///
    /// The **widget index** is in the stamp where [`super::fieldedit`]'s
    /// carries only a name, and it has to be: one field can be drawn in three
    /// places with three different boxes, and a draft keyed on the name alone
    /// would carry the first box's numbers onto the second placement. On a
    /// radio group that is the ordinary case rather than the exotic one.
    stamp: Option<(String, usize, u64)>,
    /// Lower-left x, in PDF user space.
    x: f64,
    /// Lower-left y.
    y: f64,
    /// Width.
    w: f64,
    /// Height.
    h: f64,
    /// The four as the document holds them, so Apply can tell whether the
    /// operator changed anything and `resizes()` can tell which act it is.
    stored: (f64, f64, f64, f64),
    /// The caption being typed.
    caption: String,
    /// The caption as the document holds it.
    caption_stored: String,
}

impl WidgetPropsDraft {
    /// Pull the values off a real widget, and sync.
    fn read(&mut self, widget: &Widget, rect: Rect, fqn: &str, widget_index: usize, epoch: u64) {
        let caption = widget
            .caption
            .as_deref()
            .map(|raw| String::from_utf8_lossy(raw).into_owned())
            .unwrap_or_default();
        self.sync(
            (rect.llx, rect.lly, rect.urx - rect.llx, rect.ury - rect.lly),
            caption,
            fqn,
            widget_index,
            epoch,
        );
    }

    /// Re-read when the stamp has moved; otherwise keep what is on screen.
    fn sync(
        &mut self,
        rect: (f64, f64, f64, f64),
        caption: String,
        fqn: &str,
        widget_index: usize,
        epoch: u64,
    ) {
        let stamp = (fqn.to_owned(), widget_index, epoch);
        if self.stamp.as_ref() == Some(&stamp) {
            return;
        }
        self.stamp = Some(stamp);
        self.stored = rect;
        (self.x, self.y, self.w, self.h) = rect;
        self.caption_stored = caption;
        self.caption.clone_from(&self.caption_stored);
    }

    /// Whether any of the four numbers has been typed away from the document's.
    fn differs(&self) -> bool {
        let (x, y, w, h) = self.stored;
        !near(self.x, x) || !near(self.y, y) || !near(self.w, w) || !near(self.h, h)
    }

    /// Whether committing would change the **extent**, which is what decides
    /// between a free translation and an appearance rebuild.
    fn resizes(&self) -> bool {
        let (_, _, w, h) = self.stored;
        !near(self.w, w) || !near(self.h, h)
    }
}

/// Two values within display precision of each other.
///
/// Half a hundredth: the spinners show two decimals, so anything closer than
/// that is a difference the operator cannot see and did not type.
fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 0.005
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A draft is re-seeded when the WIDGET changes, not only when the
    /// field does.**
    #[test]
    fn a_draft_follows_the_widget_and_not_just_the_field() {
        let mut draft = WidgetPropsDraft::default();
        draft.sync((10.0, 20.0, 100.0, 30.0), String::new(), "Group", 0, 0);
        assert!((draft.x - 10.0).abs() < 1e-9);

        // The same field, a different placement.
        draft.sync((200.0, 400.0, 60.0, 12.0), String::new(), "Group", 1, 0);
        assert!(
            (draft.x - 200.0).abs() < 1e-9,
            "the second button's box must replace the first's"
        );
    }

    /// **Apply is dead until something is typed**, and a `/Rect` carrying more
    /// than two decimals does not count as typed.
    #[test]
    fn apply_is_dead_until_a_number_actually_moves() {
        let mut draft = WidgetPropsDraft::default();
        draft.sync((10.0016, 20.0, 100.0, 30.0), String::new(), "F", 0, 0);
        assert!(
            !draft.differs(),
            "a sub-display-precision difference is not a change"
        );

        draft.x = 12.0;
        assert!(draft.differs());
        assert!(!draft.resizes(), "moving is not resizing");

        draft.w = 140.0;
        assert!(draft.resizes(), "and changing the extent is");
    }

    /// **A move and a resize are told apart**, which is what the Apply hover
    /// promises before the press and the status line reports after it.
    #[test]
    fn a_pure_translation_is_never_reported_as_a_resize() {
        let mut draft = WidgetPropsDraft::default();
        draft.sync((10.0, 20.0, 100.0, 30.0), String::new(), "F", 0, 0);
        draft.x += 50.0;
        draft.y -= 12.5;
        assert!(draft.differs());
        assert!(
            !draft.resizes(),
            "both corners moved by the same amount, so the extent is unchanged"
        );
    }
}
