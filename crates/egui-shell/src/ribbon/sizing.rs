//! # `ribbon::sizing` — how much room one control asks for, and what it shows
//!
//! `RIBBON_SCALING.md`, and `OPERATOR_REQUESTS.md` O31.
//!
//! ## Why this file exists
//!
//! One shape for every control — icon, gap, label, one row, always — is
//! defensible on **findability** grounds and fits very little on the band.
//! Measured against Word at 884 client points, on the widest tab it has:
//!
//! | | groups on the band |
//! |---|---|
//! | Word | **10** |
//! | one size for every control | **3**, the rest behind a `⏷ N more` menu |
//!
//! Word gets there by mixing three sizes in one group: its Clipboard is one
//! Large button beside a column of three icon-only Small ones, its Font group
//! is two combos and thirteen Smalls, its Editing group is three Mediums. The
//! label is not what makes `B` findable; its position in a cluster of type
//! controls is.
//!
//! ## The rule that keeps `Small` honest
//!
//! A control renders icon-only **only when it has earned it**: it names an
//! icon, it carries a **tooltip**, and a painter is actually installed. That
//! is [`crate::ribbon::qat::shows_label`]'s rule, unchanged, applied to a
//! second surface — and the reason is the same one that module gives at
//! length: the tooltip is the icon's **accessible name**. Without one, an
//! icon-only button is an unlabelled rectangle to a screen reader and a
//! guess to everybody else.
//!
//! A `Small` that has not earned it **falls back to `Medium`**. It does not
//! render a mystery, and it does not refuse to render. This is the same shape
//! as the QAT's fallback and it means a manifest can ask for `Small`
//! everywhere without an author having to audit which commands have tooltips.
//!
//! ## The layout rule for `Large`, and the one thing it changes about a group
//!
//! A Large control is **icon above label**, spanning the band's rows. It
//! therefore cannot live inside the row-wrapping that
//! [`crate::ribbon::plan::wrap_group`] does, because that partitions items
//! *into* rows and a Large item is beside them.
//!
//! So: **within a group, Large items lead.** They are drawn first, in a
//! horizontal run at the group's left, at full height; everything else wraps
//! into the rows to their right.
//!
//! That is a real constraint on the manifest and it is worth stating rather
//! than discovering: a Large item written in the middle of a group is hoisted
//! to the front. It is also how every group in Word is actually built — Paste
//! leads Clipboard, the three Acrobat buttons are the whole group — so the
//! constraint costs nothing an author wanted, and the alternative is a
//! two-dimensional packing problem for a gain nobody asked for.
//!
//! Design and rationale: `docs/modules/egui-shell/ribbon/sizing.md`.

use egui::{Sense, TextStyle, Vec2, vec2};

use crate::commands::Command;
use crate::manifest::{Item, ItemSize};
use crate::ribbon::ctx::Ctx;
use crate::ribbon::ctx::IconRequest;
use crate::ribbon::measure::{button_padding, text_width};
use crate::ribbon::plan::ItemWidths;

/// The gap between a Large control's icon and its label, in points.
pub(super) const LARGE_STACK_GAP: f32 = 4.0;

/// How much wider a Large control is than its widest part, in points.
pub(super) const LARGE_SIDE_PADDING: f32 = 8.0;

/// The narrowest a Large control may be drawn, in points —
/// `.rb.big { min-width: 52px }`.
pub(super) const LARGE_MIN_WIDTH: f32 = 52.0;

/// The width a Large control's label wraps at, in points —
/// `.rb.big .lb { max-width: 76px }`.
pub(super) const LARGE_LABEL_WRAP: f32 = 76.0;

/// **The one layout of a Large control's label**, shared by the measuring
/// path and the drawing path.
fn large_label(ui: &egui::Ui, ctx: &Ctx<'_>, text: &str) -> std::sync::Arc<egui::Galley> {
    let font = egui::FontId::proportional(ctx.theme.metrics.ribbon_caption_pts);
    ui.ctx().fonts_mut(|fonts| {
        fonts.layout(
            text.to_owned(),
            font,
            egui::Color32::PLACEHOLDER,
            LARGE_LABEL_WRAP,
        )
    })
}

/// **Is this item drawn at all?** — `RIBBON_SCALING.md` §5.3.
#[must_use]
pub(crate) fn visible(item: &Item, conditions: &crate::commands::ConditionSet) -> bool {
    item.visible_condition()
        .is_none_or(|name| conditions.is_set(name))
}

/// **The size this control will actually render at**, which is not always the
/// size the manifest asked for.
#[must_use]
pub(crate) fn resolved(command: &Command, asked: ItemSize, can_paint: bool) -> ItemSize {
    match asked {
        ItemSize::Small if !can_paint || command.icon.is_none() || command.tooltip.is_none() => {
            ItemSize::Medium
        }
        other => other,
    }
}

/// The width one command control occupies at `size`.
#[must_use]
pub(crate) fn width(ui: &egui::Ui, ctx: &Ctx<'_>, command: &Command, size: ItemSize) -> f32 {
    let icon = if command.icon.is_some() {
        ctx.theme.metrics.icon_pts
    } else {
        0.0
    };
    match size {
        ItemSize::Medium => ItemWidths {
            icon,
            text: text_width(ui, &command.label, &TextStyle::Button),
            gap: ui.spacing().icon_spacing,
            padding: button_padding(ui),
        }
        .total(),
        // Icon only. The text is measured as zero rather than omitted, so the
        // `gap` term switches itself off through `ItemWidths`' own rule rather
        // than through a second copy of it here.
        ItemSize::Small => ItemWidths {
            icon,
            text: 0.0,
            gap: ui.spacing().icon_spacing,
            padding: button_padding(ui),
        }
        .total(),
        // Stacked: the wider of the two parts decides, and neither is a gap
        // away from the other horizontally.
        //
        // Three decisions, each visible in the arithmetic:
        //
        // 1. The icon term is the **Large** icon (24 pt, not 16), because a
        //    Large control draws a bigger picture rather than the same picture
        //    with more air.
        // 2. The text term is the **wrapped galley's** width, not the
        //    unwrapped string's. A galley wrapped at `LARGE_LABEL_WRAP`
        //    reports the width it actually used, so `Save` stays 26 pt wide
        //    and `Recognise text…` becomes 76 rather than 118.
        // 3. The result has a floor ([`LARGE_MIN_WIDTH`]), applied AFTER the
        //    padding, so a run of Large controls is a row of equal buttons.
        ItemSize::Large => {
            let icon = if command.icon.is_some() {
                ctx.theme.metrics.ribbon_icon_large_pts
            } else {
                0.0
            };
            let text = large_label(ui, ctx, &command.label).size().x;
            (icon.max(text) + LARGE_SIDE_PADDING * 2.0).max(LARGE_MIN_WIDTH)
        }
    }
}

/// Draw one Large control — icon above label, spanning `height`.
pub(crate) fn render_large(
    ui: &mut egui::Ui,
    ctx: &mut Ctx<'_>,
    command: &Command,
    selected: bool,
    enabled: bool,
    height: f32,
) -> egui::Response {
    let icon_size = ctx.theme.metrics.ribbon_icon_large_pts;
    // The label is laid out ONCE, here, and the same galley is measured for
    // the control's height and painted into it — see [`large_label`] for why
    // one call rather than two.
    let galley = large_label(ui, ctx, &command.label);
    // NEVER SHORTER THAN ITS OWN CONTENT.
    //
    // `height` is the band's row area, which a Large control spans. In the
    // **overflow menu** there is no row area: a group in the menu is drawn
    // with `GroupBox::NATURAL`, whose `rows` is `0.0` deliberately, so that a
    // one-row group in the popup does not get a hole under it. A Large control
    // handed that zero would allocate a rect of zero height — which still
    // paints (the icon and label are placed from the rect's centre, which
    // exists) and still reports its rect, and is **not clickable**, because a
    // zero-height rect has no area to hit. Nothing in this crate's unit tests
    // can see that: they exercise the band path, which hands a real row
    // height, and only a driven check reads the declared rect back.
    //
    // So: span the rows when there are rows, and be as tall as the content
    // otherwise. Both are the same expression.
    let content_height = icon_size + LARGE_STACK_GAP + galley.size().y + LARGE_STACK_GAP * 2.0;
    // **CAPPED AT [`crate::theme::Metrics::ribbon_large_pts`].**
    //
    // `height` is the band's row area, and a Large control is not simply that
    // area. The mockup's arithmetic:
    // `.rb.big { height: 56px }` inside a 68 px row area, top-aligned by
    // `.grp .items { align-items: flex-start }`. A Large control spans most
    // of the band and not all of it, which is what stops a group made only of
    // Large controls — Pages ▸ Clipboard, Pages ▸ Transform — from reading as
    // one solid block of colour when its members are pressed.
    //
    // `min` then `max` rather than a `clamp`, and the order is load-bearing:
    // the content floor must survive the cap, because in the overflow menu
    // `height` is 0 (see the paragraph above) and `0.min(56) = 0` must still
    // come back up to `content_height`. A `clamp(content_height, large_pts)`
    // would panic the day a two-line label made the content taller than the
    // cap, which is a reachable state and not an error.
    let want = vec2(
        width(ui, ctx, command, ItemSize::Large),
        height
            .min(ctx.theme.metrics.ribbon_large_pts)
            .max(content_height),
    );
    // **ALLOCATED FROM A DISABLED SCOPE WHEN IT IS DISABLED**, and this
    // is the only thing that makes `enabled` mean anything here.
    //
    // `Ui::interact` passes `self.enabled` into the response's `ENABLED` flag
    // (`egui-0.35.0/src/ui.rs:928`, `egui-0.35.0/src/context.rs:1385`). Allocating from an *enabled*
    // `Ui` and merely painting greyed — choosing `visuals.widgets.inactive` by
    // hand fifteen lines below — therefore leaves `response.enabled()`
    // **true**, with two consequences:
    //
    // 1. **The tooltip is dead.** `on_disabled_hover_text` opens only when
    //    `!response.enabled()`, so it never runs — here, and again at the
    //    caller in `ribbon::control`, which attaches the same explanation the
    //    same way. A Large band command would be greyed with no explanation,
    //    and R9 requires one.
    // 2. **And the click still fires.** `ribbon::control` does
    //    `if response.clicked() { ctx.invoke(command.handler) }` with no
    //    second gate, so pressing a greyed Large control would **invoke its
    //    command**. The band says no and the shell does it anyway.
    //
    // ⇒ One scope answers both, because both read one flag. It wraps the
    // ALLOCATION only; the painting below still uses the outer `ui`'s painter,
    // so the greyed appearance is unchanged to the pixel and the hand-picked
    // `inactive` visuals keep working. Wrapping the painting too would
    // multiply the disabled alpha a second time and dim every greyed Large
    // control twice over.
    let (rect, response) = if enabled {
        ui.allocate_exact_size(want, Sense::click())
    } else {
        ui.scope(|ui| {
            ui.disable();
            ui.allocate_exact_size(want, Sense::click())
        })
        .inner
    };

    let visuals = if enabled {
        ui.style().interact_selectable(&response, selected)
    } else {
        ui.style().visuals.widgets.inactive
    };
    // **FRAMELESS AT REST** — the operator's single biggest complaint
    // about this band, and the one that held at every width:
    //
    // > "Every ribbon item in the real build is drawn with a visible button
    // >  FRAME … the mockup draws them frameless."
    //
    // The mockup's own rule is not "no border". It is
    // `.rb { border: 1px solid transparent }` with `.rb:hover { background }`
    // and `.rb[aria-pressed="true"] { background: var(--plate) }` — the frame
    // is *reserved and invisible*, so the control does not move when it
    // acquires one, and the interactive states paint into it.
    //
    // That is exactly what this condition does, and it is why the rect is
    // still `visuals.*` rather than nothing: at rest nothing is painted; the
    // instant the control is hovered, focused, pressed or selected the full
    // frame appears at the size it always occupied.
    //
    // **The disabled state is frameless too**, deliberately.
    // `.rb[disabled]` in the mockup changes the *ink*
    // (`color: var(--ink-quiet); opacity: .45`) and nothing else. Painting a
    // greyed plate behind a greyed label says "unavailable" by drawing MORE
    // ink than an available control — which is backwards, and is what makes a
    // tab of mostly-greyed groups read as louder than one with none.
    //
    // **Feedback is not lost, it is relocated** — see the sibling note in
    // `super::control::command_button`, which reaches the same behaviour
    // through `egui::Button::frame_when_inactive` rather than by hand.
    let interacting =
        response.hovered() || response.has_focus() || response.is_pointer_button_down_on();
    if selected || (enabled && interacting) {
        ui.painter().rect(
            rect,
            visuals.corner_radius,
            visuals.weak_bg_fill,
            visuals.bg_stroke,
            egui::StrokeKind::Inside,
        );
    }

    // The icon occupies a square at the top, centred; the label sits beneath
    // it, centred. Both are placed from the rect rather than from a cursor, so
    // the two halves cannot drift apart when the height changes.
    let label_height = galley.size().y;
    let stack = icon_size + LARGE_STACK_GAP + label_height;
    let top = rect.top() + ((rect.height() - stack) / 2.0).max(0.0);
    if let Some(key) = command.icon.clone()
        && let Some(painter) = ctx.icons.take()
    {
        let icon_rect = egui::Rect::from_min_size(
            egui::pos2(rect.center().x - icon_size / 2.0, top),
            Vec2::splat(icon_size),
        );
        painter(
            ui.painter(),
            &IconRequest {
                key: &key,
                rect: icon_rect,
                tint: visuals.fg_stroke.color,
                enabled,
                selected,
            },
        );
        ctx.icons = Some(painter);
    }
    // `painter.galley` rather than `painter.text`, and the difference is
    // the wrap. `Painter::text` lays a string out **unwrapped**
    // at the point it is given — there is no width to wrap against — so a
    // Large control's label could only ever be one line, however long. Here
    // the galley was already laid out at [`LARGE_LABEL_WRAP`] by
    // [`large_label`]; painting it is what puts the second line on screen.
    //
    // `Align2::CENTER_TOP` is spelled by hand because a galley is painted from
    // its top-LEFT: half its own width back from the control's centre line.
    // The galley itself is centre-aligned internally (`Align::Center` is
    // `layout`'s default for a wrapped job's horizontal alignment within its
    // wrap width), so a two-line label reads as a centred block rather than as
    // a ragged left edge.
    //
    // The colour is passed as the *fallback*, which is what
    // `Color32::PLACEHOLDER` in the layout job resolves to — so one cached
    // galley serves the enabled, disabled and selected paints.
    ui.painter().galley(
        egui::pos2(
            rect.center().x - galley.size().x / 2.0,
            top + icon_size + LARGE_STACK_GAP,
        ),
        galley,
        visuals.fg_stroke.color,
    );

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::{Command, ConditionSet, HandlerToken};
    use crate::manifest::Item;

    fn command(id: &str) -> Command {
        Command::new(id, "Label", HandlerToken::new(1))
    }

    /// **A response allocated from an ENABLED `Ui` is enabled, however it
    /// is painted** — the assumption [`render_large`] would otherwise be
    /// making.
    #[test]
    fn only_a_disabled_scope_produces_a_disabled_response() {
        let ctx = egui::Context::default();
        let mut plain = None;
        let mut scoped = None;
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            let (_, response) =
                ui.allocate_exact_size(egui::vec2(40.0, 20.0), egui::Sense::click());
            plain = Some(response.enabled());

            let (_, response) = ui
                .scope(|ui| {
                    ui.disable();
                    ui.allocate_exact_size(egui::vec2(40.0, 20.0), egui::Sense::click())
                })
                .inner;
            scoped = Some(response.enabled());
        });
        assert_eq!(
            plain,
            Some(true),
            "painting a control greyed does not disable its response — that was the bug"
        );
        assert_eq!(
            scoped,
            Some(false),
            "…and allocating inside a disabled scope is what does, which is what              `on_disabled_hover_text` and the click gate both read"
        );
    }

    /// **`frame_when_inactive(false)` removes the resting frame and
    /// nothing else** — the egui contract the whole frameless change rests on.
    #[test]
    fn frame_when_inactive_removes_the_resting_ink_and_not_the_rectangle() {
        /// How many rectangles this shape tree paints that a person could
        /// SEE — a fill that is not transparent, or a stroke with width.
        fn inked(shape: &egui::Shape) -> usize {
            match shape {
                egui::Shape::Rect(r) => {
                    usize::from(r.fill.a() > 0 || (r.stroke.width > 0.0 && r.stroke.color.a() > 0))
                }
                egui::Shape::Vec(v) => v.iter().map(inked).sum(),
                _ => 0,
            }
        }

        let ctx = egui::Context::default();
        let mut framed = None;
        let mut frameless = None;
        let output = ctx.run_ui(egui::RawInput::default(), |ui| {
            framed = Some(
                ui.add(egui::Button::new("Open…").frame_when_inactive(true))
                    .rect,
            );
        });
        let framed_shapes: usize = output.shapes.iter().map(|c| inked(&c.shape)).sum();

        let ctx2 = egui::Context::default();
        let output2 = ctx2.run_ui(egui::RawInput::default(), |ui| {
            frameless = Some(
                ui.add(egui::Button::new("Open…").frame_when_inactive(false))
                    .rect,
            );
        });
        let frameless_shapes: usize = output2.shapes.iter().map(|c| inked(&c.shape)).sum();

        let framed = framed.expect("the framed closure never ran");
        let frameless = frameless.expect("the frameless closure never ran");
        assert_eq!(
            framed.size(),
            frameless.size(),
            "a frameless button measured {:?} against the framed one's {:?}. The band \
             plans every group's width from `button_padding`, so a size that depends \
             on whether the frame is painted would make the plan true only while the \
             pointer is over the control",
            frameless.size(),
            framed.size()
        );
        assert!(
            frameless_shapes < framed_shapes,
            "a resting frameless button painted {frameless_shapes} rectangles and a \
             framed one painted {framed_shapes}. The flag changed nothing, so every \
             control in the band is still drawn in its own box"
        );
    }

    /// `Small` is earned three ways, and failing any one of them falls
    /// back to `Medium` rather than drawing an unlabelled rectangle.
    #[test]
    fn small_is_earned_and_falls_back_when_it_is_not() {
        for icon in [false, true] {
            for tooltip in [false, true] {
                for painter in [false, true] {
                    let mut c = command("x");
                    if icon {
                        c = c.with_icon("k");
                    }
                    if tooltip {
                        c = c.with_tooltip("t");
                    }
                    let got = resolved(&c, ItemSize::Small, painter);
                    let earned = icon && tooltip && painter;
                    assert_eq!(
                        got,
                        if earned {
                            ItemSize::Small
                        } else {
                            ItemSize::Medium
                        },
                        "icon={icon} tooltip={tooltip} painter={painter}"
                    );
                }
            }
        }
    }

    /// `Medium` and `Large` are never downgraded — only `Small` is earned.
    #[test]
    fn only_small_is_ever_downgraded() {
        let bare = command("x");
        for painter in [false, true] {
            assert_eq!(resolved(&bare, ItemSize::Medium, painter), ItemSize::Medium);
            assert_eq!(resolved(&bare, ItemSize::Large, painter), ItemSize::Large);
        }
    }

    /// An item with no condition is always visible; one with a condition is
    /// visible exactly while it holds.
    #[test]
    fn visibility_follows_the_condition_and_defaults_to_shown() {
        let plain = Item::command("a");
        let gated = Item::command("b").shown_when("mode.edit");
        let mut set = ConditionSet::default();

        assert!(
            visible(&plain, &set),
            "an unconditioned item is always shown"
        );
        assert!(
            !visible(&gated, &set),
            "a condition that is not set hides it"
        );
        set.set("mode.edit");
        assert!(visible(&gated, &set));
        assert!(
            visible(&plain, &set),
            "setting an unrelated condition changes nothing"
        );
    }

    /// A separator and a custom item carry no condition and are always
    /// visible — the honest answer, since neither can state one yet.
    #[test]
    fn an_item_that_cannot_be_conditioned_is_shown() {
        let set = ConditionSet::default();
        assert!(visible(&Item::Separator, &set));
        assert!(visible(&Item::custom("swatch"), &set));
    }
}
