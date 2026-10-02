//! # `ribbon::control` — drawing ONE control
//!
//! The seam [`super::band`]'s header describes from the other side:
//! everything here answers *"how is a single item drawn?"*, and everything in
//! `band` answers *"how are the groups arranged?"*.
//!
//! The public entry points, in the order they call each other:
//!
//! | function | question |
//! |---|---|
//! | [`render_item_at`] | what KIND of item is this — a command, a separator, an application's own custom widget? |
//! | [`render_command`] | at what SIZE, and does the registry even have it? |
//! | [`command_button`] | the button itself: icon, optional label, selection, enablement, tooltip |
//!
//! Nothing here knows about rows, groups, captions or the band's width. That
//! is what makes the seam hold: a change to how the band scales cannot reach
//! into this file, and a change to how a button looks cannot reach out of it.
//!
//! Design and rationale: `docs/modules/egui-shell/ribbon/control.md`.

use egui::{Atoms, RichText, Vec2, vec2};

use super::a11y;
use super::band::selected_condition;
use super::ctx::{Ctx, CustomItem, IconRequest};
use super::plan::CUSTOM_ITEM_WIDTH;
use super::report;
use super::sizing;
use crate::manifest::{Item, ItemSize};

/// Draw one item of a group, at the size it resolved to.
///
/// `rows_height` is the height of the band's row area, which only a `Large`
/// control uses — it spans the rows rather than sitting in one.
pub(crate) fn render_item_at(
    ui: &mut egui::Ui,
    ctx: &mut Ctx<'_>,
    tab_id: &str,
    group_id: &str,
    item: &Item,
    size: ItemSize,
    rows_height: f32,
) {
    match item {
        Item::Separator => {
            ui.separator();
        }
        Item::Command { id, .. } => {
            render_command(ui, ctx, id, size, rows_height);
        }
        Item::Custom { kind, payload, .. } => {
            let request = CustomItem {
                kind,
                payload: payload.as_deref(),
                tab: tab_id,
                group: group_id,
            };
            // `take` so the borrow of `ctx.custom` does not conflict with
            // `ctx.invoke`; put back immediately, because a renderer that
            // vanished after the first custom item would be a very
            // confusing bug.
            if let Some(renderer) = ctx.custom.take() {
                let drawn = ui.scope(|ui| renderer(ui, &request));
                let token = drawn.inner;
                remember_custom_width(ui.ctx(), kind, drawn.response.rect.width());
                ctx.custom = Some(renderer);
                if let Some(token) = token {
                    ctx.invoke(token);
                }
            } else {
                // No renderer: reserve the space the plan budgeted for
                // it, so the band's arithmetic stays true and the gap is
                // visible rather than silently closing up. An application
                // that put a custom item in its manifest and supplied no
                // renderer has a defect, and a hole is how it finds out.
                crate::verify::event("ribbon-custom-item-unrendered")
                    .kv("kind", kind)
                    .kv("group", group_id)
                    .emit();
                ui.allocate_space(vec2(CUSTOM_ITEM_WIDTH, ctx.theme.metrics.control_height));
            }
        }
    }
}

fn custom_width_key(kind: &str) -> egui::Id {
    egui::Id::new(("egui-shell/ribbon/custom-width", kind))
}

/// Record how wide a custom item of `kind` was drawn, for the next frame's
/// band plan. Its width is intrinsic to the application's controls, not to
/// the space offered, so feeding it back cannot oscillate.
fn remember_custom_width(ctx: &egui::Context, kind: &str, width: f32) {
    if width.is_finite() && width > 0.0 {
        ctx.data_mut(|d| d.insert_temp(custom_width_key(kind), width));
    }
}

/// What the band plan should budget for a custom item of `kind`: its width
/// when last drawn, or [`CUSTOM_ITEM_WIDTH`] before it has been drawn once.
pub(crate) fn custom_width(ctx: &egui::Context, kind: &str) -> f32 {
    ctx.data(|d| d.get_temp(custom_width_key(kind)))
        .unwrap_or(CUSTOM_ITEM_WIDTH)
}

/// Report that `id` is enabled or not, but only when the answer has changed.
fn report_enablement(ctx: &egui::Context, id: &str, enabled: bool) {
    if !crate::verify::enabled() {
        return;
    }
    let key = egui::Id::new(("egui-shell/ribbon/enablement", id));
    let previous: Option<bool> = ctx.data(|d| d.get_temp(key));
    if previous == Some(enabled) {
        return;
    }
    ctx.data_mut(|d| d.insert_temp(key, enabled));
    crate::verify::event(report::ENABLEMENT_EVENT)
        .kv("id", id)
        .kv("enabled", u8::from(enabled))
        .emit();
}

/// Report that `id` is drawn pressed or not, but only when that has changed.
fn report_selected(ctx: &egui::Context, id: &str, selected: bool) {
    if !crate::verify::enabled() {
        return;
    }
    let key = egui::Id::new(("egui-shell/ribbon/selected", id));
    let previous: Option<bool> = ctx.data(|d| d.get_temp(key));
    if previous.unwrap_or(false) == selected {
        return;
    }
    ctx.data_mut(|d| d.insert_temp(key, selected));
    crate::verify::event(report::SELECTED_EVENT)
        .kv("id", id)
        .kv("selected", u8::from(selected))
        .emit();
}

/// Draw one command control, honouring its enable predicate and its
/// selected condition.
pub(crate) fn render_command(
    ui: &mut egui::Ui,
    ctx: &mut Ctx<'_>,
    id: &str,
    size: ItemSize,
    rows_height: f32,
) {
    let Some(command) = ctx.command(id).cloned() else {
        return;
    };
    let enabled = command.is_enabled(ctx.conditions);
    report_enablement(ui.ctx(), &command.id, enabled);
    let selected = ctx.conditions.is_set(&selected_condition(&command.id));
    report_selected(ui.ctx(), &command.id, selected);

    // The three sizes — `RIBBON_SCALING.md`, and `sizing`'s header for the
    // measured case.
    //
    // A band that drew every control at one size would be a band that fits
    // far fewer groups than the product class does: driving Word at 884
    // client points puts ten groups on the band, and the reason is that Word
    // mixes sizes within a group. Findability does not require a label on
    // every control — what makes `B` findable is its position in a cluster of
    // type controls.
    //
    // The findability argument still governs where it applies: `Medium` is
    // the default, and a `Small` that has not earned its icon-only rendering
    // falls back to it rather than drawing a mystery.
    let response = match size {
        ItemSize::Large => sizing::render_large(ui, ctx, &command, selected, enabled, rows_height),
        ItemSize::Small => command_button(ui, ctx, &command, false, selected, enabled, false),
        ItemSize::Medium => command_button(ui, ctx, &command, true, selected, enabled, false),
    };

    // **Where this control was drawn** — published on the frame it was
    // drawn, under the stable name [`report::band_item`] builds.
    //
    // Group rects alone would make every *command* in the ribbon
    // unlocatable from outside the process. A caption's rect answers "is
    // this label legible"; it cannot answer "did clicking Rectangle arm
    // anything", because nothing outside the window could find the
    // Rectangle button in order to click it. Without this name the only
    // evidence available for a ribbon click's whole chain — click →
    // dispatch → tool armed → control renders pressed — is a set of unit
    // tests, one per link, none of which observes the links being
    // connected: every part tested, the join untested.
    //
    // Reported for **every** command, enabled or disabled, selected or
    // not, in the band and in a collapsed group's popup alike — because
    // the question a consumer asks is *where is this control*, and a
    // control that is greyed is still a control that was drawn somewhere.
    // A report conditioned on state would go quiet in exactly the cases a
    // harness most wants to look at.
    //
    // The shell learns nothing about what the id *means*. It publishes
    // that a control registered under some id occupied some rectangle;
    // what an id like `tool.rectangle` stands for is the application's
    // business, and naming it here would make this crate an application.
    ctx.reporter
        .report(response.rect, || report::band_item(&command.id));

    a11y::describe_command(&response, &command, true, enabled);
    let response = match (&command.tooltip, enabled) {
        (Some(tip), true) => response.on_hover_text(tip),
        (Some(tip), false) => response.on_disabled_hover_text(tip),
        (None, _) => response,
    };

    if response.clicked() {
        ctx.invoke(command.handler);
        crate::verify::event("ribbon-command-invoked")
            .kv("id", &command.id)
            .kv("handler", command.handler.get())
            .emit();
    }
}

/// The button itself: an optional icon slot, an optional label, the
/// selected state, and the icon painting seam.
pub(crate) fn command_button(
    ui: &mut egui::Ui,
    ctx: &mut Ctx<'_>,
    command: &crate::commands::Command,
    shows_label: bool,
    selected: bool,
    enabled: bool,
    truncate: bool,
) -> egui::Response {
    let icon_size = ctx.theme.metrics.icon_pts;
    let icon_slot = command
        .icon
        .as_ref()
        .map(|key| (key.clone(), ctx.id("icon", &command.id)));

    let mut atoms = Atoms::default();
    if let Some((_, slot_id)) = &icon_slot {
        atoms.push_right(egui::Atom::custom(*slot_id, Vec2::splat(icon_size)));
    }
    if shows_label || icon_slot.is_none() {
        // The `||` is the accessibility floor: a command with no icon key
        // draws its label even in an icon-only context, because a control
        // with neither an icon nor a label is an empty rectangle.
        atoms.push_right(RichText::new(&command.label));
    }

    let laid_out = ui
        .scope(|ui| {
            if !enabled {
                ui.disable();
            }
            // **FRAMELESS AT REST** — this one line answers the operator's
            // biggest single complaint about the band:
            //
            // > "Every ribbon item in the real build is drawn with a visible
            // >  button FRAME. Every one — New, Open…, Recent, Close, Save,
            // >  Save as…, all of them — sits in its own outlined box. The
            // >  mockup draws them frameless."
            //
            // ## What the mockup actually specifies, which is NOT "no frame"
            //
            // ```css
            // .rb                       { border: 1px solid transparent }
            // .rb:hover                 { background: var(--chrome-3) }
            // .rb[aria-pressed="true"]  { background: var(--plate);
            //                             color: var(--accent);
            //                             border-color: var(--plate) }
            // ```
            //
            // The frame is **reserved and invisible**. That is a different
            // thing from absent, and the difference is the one property that
            // makes the whole change safe: the control occupies the same
            // rectangle at rest as it does under the pointer, so acquiring a
            // frame cannot move it, cannot reflow its row, and cannot make
            // the band's planned width a lie.
            //
            // ## Why `frame_when_inactive(false)` and not `frame(false)`
            //
            // `Button::frame(false)` is the obvious spelling and it is the
            // wrong one — it would remove the hover and pressed feedback
            // along with the resting frame, which is precisely the failure
            // this project would rather ship an ugly band than ship. From
            // `egui-0.35.0/src/widgets/button.rs:363`:
            //
            // ```text
            // layout = if has_frame_margin
            //     && (state != WidgetState::Inactive || frame_when_inactive) {
            //         layout.frame(frame)              // fill + stroke
            //     } else {
            //         layout.frame(Frame::new()
            //             .inner_margin(frame.inner_margin))   // margin only
            //     };
            // ```
            //
            // Two facts fall out of those four lines and both are what is
            // wanted:
            //
            //  1. Only the **Inactive** state loses its ink.
            //     `WidgetState::Hovered` and `WidgetState::Active`
            //     (`egui-0.35.0/src/widget_style.rs:105-113`: pointer down, focused, or
            //     clicked) still paint `frame` in full — fill *and* stroke. So
            //     hover feedback and pressed feedback survive intact; what
            //     goes away is the box around forty resting controls.
            //  2. The **inner margin is identical in both branches**, so the
            //     button measures the same in every state. This is why the
            //     band's width planner needed no change: `sizing::width` was
            //     always measuring `button_padding`, and `button_padding` is
            //     what both branches keep.
            //
            // ## And `selected` is passed through as the exception
            //
            // `.frame_when_inactive(selected)`, not `(false)`. A *selected*
            // control — an armed tool, the current page-display mode — is
            // pressed-looking while nobody is touching it, i.e. it is in the
            // Inactive state and must still draw its plate. This is exactly
            // the composition `egui::Button::selectable` performs
            // (`egui-0.35.0/src/widgets/button.rs:78-83`), and the plate it draws is
            // `visuals.selection.bg_fill` = `Palette::selected_plate` with
            // `selection.stroke` = `Palette::accent` as ink — bit for bit the
            // mockup's `background: var(--plate); color: var(--accent)`.
            //
            // ## What is deliberately NOT matched
            //
            // The mockup's hover paints a background and no border; egui's
            // hovered state paints both. Zeroing `widgets.hovered.bg_stroke`
            // to match would change that state's `inner_margin` — it is
            // `button_padding + expansion − bg_stroke.width`
            // (`egui-0.35.0/src/widget_style.rs:163`) — so the button would grow by a point
            // in each direction the moment the pointer touched it. A control
            // that twitches under the cursor is a worse defect than a
            // hairline the operator only ever sees on the one control they
            // are already pointing at.
            let mut button = egui::Button::new(atoms)
                .selected(selected)
                .frame_when_inactive(selected);
            if truncate {
                button = button.truncate();
            }
            button.atom_ui(ui)
        })
        .inner;

    if let Some((key, slot_id)) = icon_slot
        && let Some(rect) = laid_out.rect(slot_id)
        && let Some(painter) = ctx.icons.take()
    {
        let visuals = ui.style().interact(&laid_out.response);
        painter(
            ui.painter(),
            &IconRequest {
                key: &key,
                rect,
                tint: visuals.fg_stroke.color,
                enabled,
                selected,
            },
        );
        ctx.icons = Some(painter);
    }

    laid_out.response
}
