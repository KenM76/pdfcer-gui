//! # `panels::properties::widgetedit` — the **box** a form field is drawn in,
//! as opposed to the field itself
//!
//! `Pass 134.0`'s `EditSession::edit_widget`, consumed 2026-08-27.
//!
//! ## ★★★ Why this is a second file and not four more rows in [`super::fieldedit`]
//!
//! Because the engine has two verbs, and it has two verbs because Acrobat's own
//! scripting model has two scopes. Taken verbatim from the design brief: some
//! properties *"apply to all widgets that are children of that field"*, others
//! *"are specific to individual widgets"*.
//!
//! | scope | verb | properties |
//! |---|---|---|
//! | **field** — one write, every placement | `edit_field` | required, read-only, tooltip, and the type flags |
//! | **widget** — per placement | `edit_widget` | rect, border, visibility, caption |
//!
//! > **Getting this backwards is invisible on the ordinary one-widget field and
//! > wrong on every radio group** — where "the border" can only sensibly mean
//! > one button and "required" can only sensibly mean the group.
//!
//! A single file holding both would make that distinction a comment. Two files
//! make it the module boundary, and the pane draws them as two headed sections
//! so an operator meets it as well.
//!
//! ## ★★★ Moving is free and resizing is not. Both, and the difference shows
//!
//! §12.5.5 derives a widget's appearance matrix from the appearance box's
//! corners and the `/Rect` corners. A **pure translation** makes that matrix a
//! pure translation, so the baked artwork moves with the box, exactly and for
//! nothing — which is why `move_widget` regenerates no appearance and is right
//! not to.
//!
//! A changed **extent** puts the same algorithm to work as a *scale*. A text
//! field dragged twice as wide would render its text twice as wide rather than
//! gaining room for more text. So `edit_widget` compares the **extent, not the
//! corners**, and rebuilds only when it changed. `WidgetEditOutcome::resized`
//! reports which happened, and the pane says so, because *"the box moved"* and
//! *"the box was resized and its contents were redrawn"* are different things
//! to have done to a file.
//!
//! ★★ **`appearance_stale` is the one an operator will see and misread.** A
//! resize that could not rebuild the artwork — a push button's baked caption, a
//! signature — leaves the widget rendering **distorted**. The engine names it;
//! this pane prefixes the engine's own string with what it means on screen.
//!
//! ## ★★★ All four properties are here, and the last two took an hour
//!
//! This section read *"`WidgetEdit` carries four properties and this pane
//! offers **two**"* for about an hour on 2026-08-27, and the reason is worth
//! keeping because the outcome is what decision 058 promises and rarely gets to
//! demonstrate.
//!
//! **border** (`/BS`) and **visibility** (`/F`) were writable and **not
//! readable**: `annot_author::read_border_width` was private, `border_style` is
//! a *writer*, and `forms::Widget` modelled no border at all. So the controls
//! were **absent rather than offered**, and the reason was not effort:
//!
//! > A properties control has to show the current value. One seeded from a
//! > default would display *Solid 1 pt* over a widget whose file says *Dashed
//! > 3 pt* and write the invention back on the first press.
//!
//! That was filed rather than worked around, and `Pass 146.0` shipped
//! `Widget::border`, `Widget::visibility` and `Widget::annot_flags` within the
//! hour. The engine checked every claim in the request against their tree
//! before scoping it, and quoted the sentence above into the field's own doc
//! comment, into `docs/core-api/`, and into their test file header — *"because
//! the next person to touch this will be tempted to simplify it."*
//!
//! ## ★★★ `None` is a FACT, and this pane must never substitute a default
//!
//! Both new fields are `Option`, and both `None`s are load-bearing:
//!
//! * **`border: None` means the file states no border.** Not
//!   `BorderSpec::default()` — that default is solid/1 pt because it reproduces
//!   the bytes pdfcer *authors*, which is correct for a writer and a lie from a
//!   reader. Their load-bearing test is named
//!   `a_widget_whose_file_states_no_border_reads_a_dash_not_a_default`, and
//!   sabotaging the reader to return the default turns it red.
//! * ★ **A border of width 0 is a VALUE**, not an absence — Table 166 states it
//!   as *no border*. It reads `0 pt`. Collapsing it to `None` would tell an
//!   operator the file is silent when it has said something definite.
//! * **`visibility: None` means the file's flags are ones pdfcer cannot set.**
//!   The mapping is exact-or-nearest-is-refused: `/F` admits dozens of
//!   combinations and `Visibility` is the four pdfcer can write, so a file
//!   carrying `Print | NoZoom` has no nearest of the four that is not a lie.
//!   `annot_flags` carries the raw word so the pane can say so.
//! * ★★ `None` there can never mean *absent*: Table 164 makes an absent `/F`
//!   equal `0`, which **is** one of the four. So the sentence is always about a
//!   file that said something inexpressible.
//!
//! ## Rule 4
//!
//! Nothing here marks the canvas. A moved box renders exactly where the saved
//! file will render it, and every disclosure — resized, stale artwork, siblings
//! untouched — lands in the status bar.

use egui::Ui;
use pdfcer_core::edit::WidgetEdit;
use pdfcer_core::forms::{Field, Widget};
use pdfcer_core::page_tree::Rect;

use crate::app::actions::Action;
use crate::app::actions::forms::FieldAction;
use crate::panels::PanelsState;
use crate::text::panels::formfield as t;

/// The section's rect, for `ui-verify`.
///
/// ★ Plain [`crate::diag::ui_rect`], not the visibility-gated form, for the
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
/// The border-width spinner's region.
// ui-text-exempt: trace region name, never displayed
pub const BORDER_WIDTH_REGION: &str = "properties.widget_edit.border_width";

/// The rotation row, for layout checks.
pub const ROTATION_REGION: &str = "properties.widget_edit.rotation";

/// ★★ The two rotation buttons, EACH named.
///
/// One region per button rather than one for the row, because a driven check
/// that aimed at a fraction of a shared row is doing coordinate arithmetic the
/// harness has a `declared_center` for — and the first version did exactly
/// that, computed 78 % across, and landed outside the window. A named control
/// is aimed at by name.
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
///
/// ★ A quarter of a point, matching `super::geometry`'s `SPEED`, and the
/// reason is the same: these are **drafting** numbers on a drawing sheet, where
/// a whole point of drift is visible. An operator who wants a big move types
/// the number.
const SPEED: f64 = 0.25;

/// Draw the selected widget's own properties, or nothing.
///
/// Returns whether it drew. `false` when the selection names a widget the
/// field no longer has — reachable through undo, which does not clear a
/// selection, and the right answer is silence rather than a pane describing a
/// box that is not there.
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
        // ★ A widget with no readable `/Rect` renders nothing this pane could
        // describe, and a zero-area rect is *intentional* invisibility for a
        // signature field (§12.7.4.5) rather than a defect — so `None` here is
        // the malformed case only. Silence: four spinners seeded from nothing
        // would invite a press that writes an invented box.
        return false;
    };

    let draft = state.widget_props_mut();
    draft.read(widget, rect, fqn, widget_index, epoch);

    ui.label(t::widget_heading());
    // ★ Said only when there is more than one placement, because that is the
    // only state in which the scope distinction is visible — and it is
    // precisely the state in which an operator would otherwise expect this
    // section to behave like the one above it.
    if field.widgets.len() > 1 {
        ui.small(t::widget_scope_note(field.widgets.len()));
    }
    ui.add_space(2.0);

    geometry_rows(ui, draft, actions, fqn, widget_index);
    ui.add_space(4.0);
    // ★★ Rotation sits WITH the geometry, not after the caption, and it moved
    // here on 2026-08-30 for two reasons that agree.
    //
    // It IS geometry — an operator adjusting where a box is and how big it is is
    // in the same thought as which way round it faces, and the caption is a
    // different subject entirely.
    //
    // ★ And it was measured unreachable at the bottom: with every section drawn
    // the control landed at `y=1379` in a window 768 points tall. The panel
    // scrolls, so it was not lost the way the bookmarks controls were — but a
    // control an operator has to scroll past four unrelated sections to reach is
    // one they will not find, and a driven check could not reach it either.
    rotation_row(ui, widget, fqn, widget_index, actions);
    ui.add_space(4.0);
    border_rows(ui, widget, fqn, widget_index, actions);
    ui.add_space(4.0);
    visibility_row(ui, widget, fqn, widget_index, actions);
    ui.add_space(4.0);
    caption_row(ui, draft, actions, fqn, widget_index);

    crate::diag::ui_rect(REGION, ui.min_rect());
    true
}

/// **Turn the box**, in ninety-degree steps.
///
/// `EditSession::rotate_widget`, shipped 2026-08-30. `/MK /R`, Table 189.
///
/// # ★★★ THE DIRECTION IS THE WHOLE DANGER, AND IT IS NEGATED HERE
///
/// `/MK /R` is **counterclockwise**. The page's `/Rotate` is **clockwise**. The
/// engine flagged this as *"the single most likely thing for a shell to get
/// backwards"*, and the standard makes it easy: the two entries are word for
/// word parallel —
///
/// | | |
/// |---|---|
/// | `/MK /R` (Table 189) | *"…rotated **counterclockwise** relative to the page…"* |
/// | page `/Rotate` (Table 30) | *"…rotated **clockwise** when displayed or printed…"* |
///
/// **The direction word is the only difference between those two sentences.**
/// Worse, the *movie* dictionary's `/Rotate` uses the identical phrase
/// *"relative to the page"* with the **opposite** sense, so that phrase carries
/// no convention at all — only the direction word does.
///
/// ⇒ So the two controls here are labelled **left** and **right**, which is
/// what an operator means, and the negation happens **here, at the UI layer**,
/// exactly as the engine instructed: *"if your rotate control has a clockwise
/// affordance, negate at the UI layer and pass counterclockwise degrees to us.
/// Do not negate inside anything that touches `/MK`."*
///
/// A **right** turn is what the operator sees the box do. That is `-90`
/// counterclockwise, and this is the only place in the program where those two
/// facts meet.
///
/// # Why ±90 buttons and not a typed angle
///
/// The engine refuses anything that is not a multiple of 90 — Table 189 says
/// *"shall be a multiple of 90"* — so a free number is a control most of whose
/// values are refusals. Two buttons offer only what can succeed, which is R9's
/// posture rather than a simplification.
///
/// # Why the current angle is shown even at zero
///
/// Because `Widget::rotation` is `Option<i64>` and `None` means **the file
/// states none**, which is not the same fact as `Some(0)` — the distinction
/// `Widget::border`'s own docs call *"a fact to display, not a value to
/// substitute"*. An operator debugging why a box looks wrong in another viewer
/// wants to know which of the two their file says.
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
        // ★★★ The GATED form, and using the plain one shipped an unreachable
        // control on 2026-08-30.
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
            // ★ Normalised HERE as well as by the engine, so the number in the
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
            // ★★ LEFT is +90 counterclockwise and RIGHT is -90, and that is the
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
///
/// # ★★ Why an Apply button and not commit-on-release
///
/// [`super::fieldedit`]'s max-length spinner commits on release, and this one
/// deliberately does not — the difference is that **these four are one edit**.
/// A box is moved by changing X *and* Y; committing each on release would
/// author two `edit_widget` calls, two undo entries, and an intermediate state
/// in which the box has moved sideways and not down. `super::geometry` reached
/// the same conclusion for the same reason and this follows it, including the
/// button's placement.
///
/// ★ The button is **greyed when nothing was typed**, which is R9's temporarily
/// unavailable case: there is a capability and no operand, and the hover says
/// so.
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
            let response = ui.add(egui::DragValue::new(value).speed(SPEED).fixed_decimals(2));
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
        // ★ The hover names which of the two acts is about to happen, because
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
                // ★ `from_corners`, not a literal `Rect { .. }`: §7.9.5 lets a
                // `/Rect`'s corners arrive in any order and normalises them, and
                // an operator who types a width of -20 has expressed something
                // the standard has an answer for. Building the rect any other
                // way would either refuse a legal input or author a
                // denormalised box.
                // ★★★ **AND THE SAME SCALE ANSWERS THE DRAG CARRIES** —
                // `OPERATOR_REQUESTS.md` O76, wired 2026-08-31 when
                // `pdfcer-core` Pass 187.0 taught `WidgetEdit` to carry them.
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
                // ui-text-exempt: a control name carried for a refusal message.
                touched: "the box",
            }
            .into(),
        );
    }
}

/// The border's style and width — `/BS`, `Pass 146.0`.
///
/// # ★★★ It reads from the DOCUMENT and shows a dash when the file is silent
///
/// There is no draft, deliberately, and the style combo reads
/// `widget.border` fresh every frame — the same argument
/// [`super::fieldedit::flag_row`] makes for its checkboxes: a press the engine
/// refuses leaves the control where it was, because the document did not
/// change. A draft-backed control would show the operator's intent while the
/// document silently disagreed.
///
/// ★★ **`None` renders [`t::border_unstated`] and offers no width at all.** The
/// alternative — a combo pre-set to Solid and a spinner at 1 — is exactly the
/// invention this whole exchange with the engine was about, and the first press
/// would write it into the operator's file. Choosing a style from the combo is
/// how a widget with no stated border gets one, which is an act rather than a
/// default.
///
/// ★ A width of **0** is a value, not an absence, and shows as `0 pt`.
fn border_rows(
    ui: &mut Ui,
    widget: &Widget,
    fqn: &str,
    widget_index: usize,
    actions: &mut Vec<Action>,
) {
    use pdfcer_core::edit::{BorderSpec, BorderStyle};
    // ★ The five pdfcer can write. Not `BorderStyle`'s variants enumerated by
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
    ui.horizontal(|ui| {
        ui.label(t::label_border());
        let shown = current.map_or_else(t::border_unstated, |b| t::border_style_label(b.style));
        let combo = egui::ComboBox::from_id_salt("widget-border-style")
            .selected_text(shown)
            .show_ui(ui, |ui| {
                for style in STYLES {
                    let selected = current.is_some_and(|b| b.style == style);
                    if ui
                        .selectable_label(selected, t::border_style_label(style))
                        .clicked()
                        && !selected
                    {
                        // ★ The width travels with the style, because `/BS` is
                        // one dictionary and `BorderSpec` is one value — there
                        // is no "change the style and leave the width" to
                        // express. A widget with no stated border gets the
                        // standard's own Table 166 default of 1, which is
                        // reading rather than inventing: choosing a style is
                        // the operator committing to having a border.
                        let width = current.map_or(1.0, |b| b.width);
                        actions.push(
                            FieldAction::EditWidget {
                                field: fqn.to_owned(),
                                widget: widget_index,
                                edit: WidgetEdit::new().with_border(BorderSpec { style, width }),
                                // ui-text-exempt: a control name carried for a refusal message.
                                touched: "the border",
                            }
                            .into(),
                        );
                    }
                }
            });
        crate::diag::ui_rect_visible(BORDER_REGION, combo.response.rect, ui.clip_rect());
    });

    // ★ The width is offered only once the file has a border to widen. A
    // spinner over `border: None` would have to show *something*, and any
    // number it showed would be the invention.
    let Some(border) = current else {
        return;
    };
    ui.horizontal(|ui| {
        ui.label(t::label_border_width());
        let mut width = border.width;
        let response = ui.add(
            egui::DragValue::new(&mut width)
                .speed(0.25)
                .range(0.0..=72.0)
                .fixed_decimals(2),
        );
        crate::diag::ui_rect_visible(BORDER_WIDTH_REGION, response.rect, ui.clip_rect());
        let response = response.on_hover_text(t::label_border_width_hover());
        if (response.drag_stopped() || response.lost_focus()) && !near(width, border.width) {
            actions.push(
                FieldAction::EditWidget {
                    field: fqn.to_owned(),
                    widget: widget_index,
                    edit: WidgetEdit::new().with_border(BorderSpec {
                        style: border.style,
                        width,
                    }),
                    // ui-text-exempt: a control name carried for a refusal message.
                    touched: "the border width",
                }
                .into(),
            );
        }
    });
}

/// Where the widget is visible — `/F`, `Pass 146.0`.
///
/// ★★★ **`None` is a sentence, not an empty combo.** The engine's mapping is
/// exact-or-refused, so `None` means the file carries flags pdfcer cannot set —
/// `Print | NoZoom`, say — and it can never mean *absent*, because Table 164
/// makes an absent `/F` equal `0` which is one of the four.
///
/// So the pane says which flags, in hex, and says pdfcer is leaving them alone.
/// The alternative — showing the nearest of the four — is the border defect
/// wearing a different hat, and the operator's first press would collapse a
/// combination the file meant.
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
                                // ui-text-exempt: a control name carried for a refusal message.
                                touched: "where the box is shown",
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
///
/// ★★ **Not cosmetic on a push button**, which is why the engine models this
/// one key out of `/MK` and none of the other ten. A push button has no `/V` at
/// all (§12.7.4.2.2), so the caption is the only thing distinguishing *Submit*
/// from *Reset* to anyone reading the field list.
///
/// ★ Empty commits `Some("")`, which **removes** it. That is the engine's
/// spelling and it is unambiguous, unlike the tooltip's three-state choice —
/// there is no "leave it alone" to express here, because not touching the
/// control is how you leave it alone.
fn caption_row(
    ui: &mut Ui,
    draft: &mut WidgetPropsDraft,
    actions: &mut Vec<Action>,
    fqn: &str,
    widget_index: usize,
) {
    ui.label(t::label_caption());
    let response = ui.add(
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
                // ui-text-exempt: a control name carried for a refusal message.
                touched: "the caption",
            }
            .into(),
        );
    }
}

/// The typed box and caption, and the widget they were read for.
#[derive(Default)]
pub struct WidgetPropsDraft {
    /// `(field name, widget index, edit epoch)` the values below were read at.
    ///
    /// ★ The **widget index** is in the stamp where [`super::fieldedit`]'s
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
    ///
    /// Takes the values rather than a `&Widget`, for the reason
    /// [`super::fieldedit::FieldPropsDraft::sync`] does: `forms::Widget` has no
    /// `Default`, so a unit test cannot build one without a document, and this
    /// function reads exactly five things off it.
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
    ///
    /// ★ An epsilon rather than `!=`, because the spinners round to two
    /// decimals for display and a `/Rect` read out of a file routinely carries
    /// more. Without it the Apply button would be live the moment the pane
    /// opened, on every widget whose box is not exactly hundredths — which is
    /// most of them, and which reads as the program thinking the operator has
    /// unsaved changes they never made.
    fn differs(&self) -> bool {
        let (x, y, w, h) = self.stored;
        !near(self.x, x) || !near(self.y, y) || !near(self.w, w) || !near(self.h, h)
    }

    /// Whether committing would change the **extent**, which is what decides
    /// between a free translation and an appearance rebuild.
    ///
    /// The engine makes the same comparison and its answer is authoritative;
    /// this one exists only so the Apply button's hover can say which act the
    /// operator is about to perform, **before** they perform it.
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

    /// ★★★ **A draft is re-seeded when the WIDGET changes, not only when the
    /// field does.**
    ///
    /// The failure this stamp's middle term exists for, and it is invisible on
    /// every one-widget field: a radio group is one field with several boxes,
    /// so a draft keyed on the name alone would carry the first button's
    /// geometry onto the second, and pressing Apply would move a box the
    /// operator was not looking at.
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
    ///
    /// ★ The second half is the one worth testing. Without the epsilon the
    /// button would be live the moment the pane opened on any widget whose box
    /// is not exactly hundredths — which reads as unsaved changes the operator
    /// never made, on most real documents.
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
