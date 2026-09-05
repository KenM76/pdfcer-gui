//! # `ribbon::sizing` — how much room one control asks for, and what it shows
//!
//! `RIBBON_SCALING.md`, and `OPERATOR_REQUESTS.md` O31.
//!
//! ## Why this file exists
//!
//! Until 2026-08-24 every control in the band was the same: icon, gap, label,
//! one row, always. [`crate::ribbon::band::render_command`] passed a hard-coded
//! `shows_label: true` and the comment beside it argued the case —
//! *"icon-only belongs to the QAT … in the band there are forty and the label
//! is the only thing that makes one findable"*.
//!
//! That argument is right about **findability** and was wrong about **every
//! control**, and driving Word settled it. Measured at 884 client points, on
//! the widest tab each application has:
//!
//! | | groups on the band |
//! |---|---|
//! | Word | **10** |
//! | this shell, before | **3** — four in a `⏷ 4 more` menu |
//!
//! Word gets there by mixing three sizes in one group: its Clipboard is one
//! Large button beside a column of three icon-only Small ones, its Font group
//! is two combos and thirteen Smalls, its Editing group is three Mediums. The
//! label is not what makes `B` findable; its position in a cluster of type
//! controls is.
//!
//! ## ★★★ The rule that keeps `Small` honest
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
//! ★ That is a real constraint on the manifest and it is worth stating rather
//! than discovering: a Large item written in the middle of a group is hoisted
//! to the front. It is also how every group in Word is actually built — Paste
//! leads Clipboard, the three Acrobat buttons are the whole group — so the
//! constraint costs nothing an author wanted, and the alternative is a
//! two-dimensional packing problem for a gain nobody asked for.

use egui::{Sense, TextStyle, Vec2, vec2};

use crate::commands::Command;
use crate::manifest::{Item, ItemSize};
use crate::ribbon::ctx::Ctx;
use crate::ribbon::ctx::IconRequest;
use crate::ribbon::measure::{button_padding, text_width};
use crate::ribbon::plan::ItemWidths;

/// The gap between a Large control's icon and its label, in points.
///
/// ★ **2 → 4 on 2026-09-04**, from `mockups/pdfcer-shell.html`'s
/// `.rb.big { gap: 4px }`. The old value's argument — *"vertically the two
/// are already separated by the icon's own bottom edge and the label's
/// ascent"* — is sound and was calibrated against a 16 pt glyph. The glyph is
/// now 24 pt ([`crate::theme::Metrics::ribbon_icon_large_pts`]), and a
/// two-point gap under a half-again-larger picture reads as the label
/// touching it.
pub(super) const LARGE_STACK_GAP: f32 = 4.0;

/// How much wider a Large control is than its widest part, in points.
///
/// A Large button's content is centred rather than left-aligned, so it needs
/// symmetric breathing room; the ordinary button padding is tuned for a row of
/// text and looks tight around a centred icon.
///
/// ★ **10 → 8 on 2026-09-04** — `.rb.big { padding: 5px 8px 2px }`, the
/// horizontal figure. Narrower, and that is the point: the mockup pays for a
/// Large control's presence with **height and glyph size**, not with width,
/// so its columns sit closer together than the shipped band's did.
pub(super) const LARGE_SIDE_PADDING: f32 = 8.0;

/// The narrowest a Large control may be drawn, in points —
/// `.rb.big { min-width: 52px }`.
///
/// ★ Why a floor is needed at all, when the control is already as wide as its
/// widest part plus padding: because its widest part can be *tiny*. A Large
/// control whose label is `Save` and whose glyph is 24 pt measures
/// `24 + 16 = 40` pt, and a run of Large controls that changed width with
/// every word would read as a ragged fence rather than as a row of equal
/// buttons. Word, Acrobat and the mockup all pin a floor; this is the
/// mockup's.
///
/// It is **not** applied to the label's wrap width — see
/// [`LARGE_LABEL_WRAP`]. A floor that also widened the text would make the
/// floor unreachable, because every label would grow to meet it.
pub(super) const LARGE_MIN_WIDTH: f32 = 52.0;

/// The width a Large control's label wraps at, in points —
/// `.rb.big .lb { max-width: 76px }`.
///
/// ★★★ **The label WRAPS, since 2026-09-04, and before that it did not.**
///
/// This is the second half of the operator's *"text label location"*
/// complaint and it is the half that is not a manifest change. A Large
/// control is *icon above label, centred*, and the labels this ribbon
/// carries are sentences by button standards — `Recognise text…`,
/// `Save a compacted copy…`, `New from template…`. Laid out on one line,
/// such a control is 120 pt wide and 56 pt tall: a letterbox, which is not
/// what a Large control looks like anywhere, and which pushes the groups
/// beside it off the band.
///
/// So the label is laid out into a galley of at most this width and may take
/// two lines (or more — nothing here caps the line count, because a cap would
/// mean silently dropping a word, and `RIBBON_SCALING.md`'s ladder is this
/// project's answer to "it does not fit"). The height that galley reports is
/// then part of the control's own content height, which is what stops a
/// two-line label from being clipped by
/// [`crate::theme::Metrics::ribbon_large_pts`].
pub(super) const LARGE_LABEL_WRAP: f32 = 76.0;

/// **The one layout of a Large control's label**, shared by the measuring
/// path and the drawing path.
///
/// ★★ [`width`] and [`render_large`] are one decision written twice and must
/// not diverge — the module's own standing warning, and the reason
/// `crate::ribbon::width_tests`'s `a_band_that_claims_to_fit_really_does_fit`
/// exists. A wrapped label makes that warning sharper, because the wrap point
/// is a *font-dependent* fact rather than an arithmetic one: two call sites
/// that each asked `egui` to wrap "the same" text could disagree over a font
/// change, a size change, or a stray trailing space. They now cannot, because
/// there is one call.
///
/// [`egui::Color32::PLACEHOLDER`] rather than a real colour so the galley is
/// the **same cache entry** the painter will later ask for — `egui` memoizes
/// layout jobs under a placeholder colour and substitutes the real one at
/// paint time, so measuring costs a hash lookup rather than a second layout.
/// It is also what lets one galley serve the enabled, disabled and selected
/// paints, which differ only in ink.
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
///
/// The `visible_when` filter, applied **before measurement**, which is the
/// whole point: a hidden item must not merely be skipped when drawing, or the
/// group reserves space for a control that never appears and the band's plan
/// is wrong by exactly the width of every hidden item.
///
/// ★★★ This is **visibility**, not enablement, and R9 draws the line: *an
/// unavailable capability renders nothing; greying is reserved for
/// **temporarily** unavailable and is always explained on hover.*
/// [`Command::enable`] is the greying — no document open, empty undo stack.
/// This is the disappearing — the command does not apply on this surface, in
/// this mode, in this build.
///
/// An item with no condition is always visible, which is nearly all of them.
#[must_use]
pub(crate) fn visible(item: &Item, conditions: &crate::commands::ConditionSet) -> bool {
    item.visible_condition()
        .is_none_or(|name| conditions.is_set(name))
}

/// **The size this control will actually render at**, which is not always the
/// size the manifest asked for.
///
/// See the module header: `Small` is earned. `can_paint` is whether the
/// application installed an icon painter at all — a manifest asking for
/// icon-only controls in a build with no icons would otherwise draw a band of
/// empty rectangles.
///
/// ★ `Large` is **not** conditional on an icon. A large button with no icon is
/// a large label, which is odd-looking but legible and unambiguous; a large
/// button with an icon and no label would be the mystery, and `Large` always
/// draws its label.
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
///
/// ★★ This and [`render`] are one decision written twice, and they must not
/// diverge: a control measured at one width and drawn at another is how a band
/// that "claims to fit" clips its last group. `band`'s own comment makes the
/// same point about the icon slot. Every branch here has a matching branch
/// there, in the same order, and
/// [`crate::ribbon::width_tests`]'s `a_band_that_claims_to_fit_really_does_fit`
/// is what would catch a drift between them.
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
        // ★ Three things changed here on 2026-09-04 and each is visible in
        // the arithmetic:
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
///
/// # Why this is built by hand rather than from `egui::Button`
///
/// `egui::Atoms` lays out with `push_right`; there is no vertical form, so a
/// `Button` cannot stack an icon over a label. The alternatives were a nested
/// `Ui` inside a frame that *looks* like a button — which does not respond
/// like one, and gets the hover and pressed visuals wrong on the day the theme
/// changes — or this: allocate the rect, take a real `Response`, and paint the
/// button's own `WidgetVisuals` into it.
///
/// ★ Painting from `ui.style().interact(&response)` rather than from theme
/// colours directly is what keeps a Large control identical to every other
/// button under hover, focus, disabled and selected. A hand-drawn control that
/// picked its own colours is the shape of the defect this project's
/// `check-theme-colors` gate exists to refuse.
pub(crate) fn render_large(
    ui: &mut egui::Ui,
    ctx: &mut Ctx<'_>,
    command: &Command,
    selected: bool,
    enabled: bool,
    height: f32,
) -> egui::Response {
    let icon_size = ctx.theme.metrics.ribbon_icon_large_pts;
    // ★ The label is laid out ONCE, here, and the same galley is measured for
    // the control's height and painted into it — see [`large_label`] for why
    // one call rather than two.
    let galley = large_label(ui, ctx, &command.label);
    // ★★★ NEVER SHORTER THAN ITS OWN CONTENT — and this was a shipped defect,
    // caught by driving.
    //
    // `height` is the band's row area, which a Large control spans. In the
    // **overflow menu** there is no row area: a group in the menu is drawn
    // with `GroupBox::NATURAL`, whose `rows` is `0.0` deliberately, so that a
    // one-row group in the popup does not get a hole under it. A Large control
    // handed that zero allocated a rect of zero height — it painted (the icon
    // and label are placed from the rect's centre, which still exists), it
    // reported its rect as required, and it was **not clickable**, because a
    // zero-height rect has no area to hit.
    //
    // `ui-verify` caught it in the honest way: `print_dialog_reaches_the_spooler`
    // opened the overflow menu, found `ribbon.item.file.print` declared at
    // `y 148.0 .. 148.0`, and said so — *"which has no usable area — the
    // control is laid out and not on screen"*. Every unit test passed, because
    // the band path hands a real row height and only the menu path does not.
    //
    // So: span the rows when there are rows, and be as tall as the content
    // otherwise. Both are the same expression.
    let content_height = icon_size + LARGE_STACK_GAP + galley.size().y + LARGE_STACK_GAP * 2.0;
    // ★★ **CAPPED AT [`crate::theme::Metrics::ribbon_large_pts`], 2026-09-04.**
    //
    // `height` is the band's row area, and until this pass a Large control
    // simply *was* that area. The mockup says otherwise:
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
    // ★★★ **ALLOCATED FROM A DISABLED SCOPE WHEN IT IS DISABLED**, since
    // 2026-08-31, and the line it replaces was wrong about the one thing it
    // claimed.
    //
    // It read `ui.allocate_exact_size(want, Sense::click())` followed by
    // `response.on_disabled_hover_text(...)`, with a comment promising *"the
    // response is neutered … and still refuses the click."*
    //
    // **It refused nothing.** `Ui::interact` passes `self.enabled` into the
    // response's `ENABLED` flag (egui 0.35 `ui.rs:928`, `context.rs:1385`),
    // and this allocated from an *enabled* `Ui` — it only painted greyed,
    // choosing `visuals.widgets.inactive` by hand fifteen lines below. So
    // `response.enabled()` was **always true**, with two consequences:
    //
    // 1. **The tooltip was dead.** `on_disabled_hover_text` opens only when
    //    `!response.enabled()`, so it never ran — here, and again at the
    //    caller in `ribbon::control`, which attaches the same explanation the
    //    same way. Every Large band command is greyed with no explanation, and
    //    R9 requires one.
    // 2. ★★★ **And the click still fired.** `ribbon::control` does
    //    `if response.clicked() { ctx.invoke(command.handler) }` with no
    //    second gate, so pressing a greyed Large control **invoked its
    //    command**. The band said no and the shell did it anyway.
    //
    // ⇒ The scope is the fix for both, because both read one flag. It wraps
    // the ALLOCATION only; the painting below still uses the outer `ui`'s
    // painter, so the greyed appearance is unchanged to the pixel and the
    // hand-picked `inactive` visuals keep working. Wrapping the painting too
    // would multiply the disabled alpha a second time and dim every greyed
    // Large control twice over.
    //
    // ★ Found by `OPERATOR_REQUESTS.md` O77's sweep for dead hover
    // explanations. The sweep was looking for silence and found a control that
    // acts.
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
    // ★★★ **FRAMELESS AT REST**, 2026-09-04 — the operator's single biggest
    // complaint about this band, and the one that held at every width:
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
    // ★ **The disabled state is frameless too**, which is a deliberate
    // asymmetry with the shipped behaviour. `.rb[disabled]` in the mockup
    // changes the *ink* (`color: var(--ink-quiet); opacity: .45`) and nothing
    // else. Painting a greyed plate behind a greyed label was the shipped
    // band's way of saying "unavailable", and it said it by drawing MORE ink
    // than an available control — which is backwards, and is most of why a
    // File tab with three greyed groups read as louder than one with none.
    //
    // ★★ **Feedback is not lost, it is relocated** — see the sibling note in
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
    // ★ `painter.galley` rather than `painter.text`, since 2026-09-04, and the
    // difference is the wrap. `Painter::text` lays a string out **unwrapped**
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

    /// ★★★ **A response allocated from an ENABLED `Ui` is enabled, however it
    /// is painted** — the assumption `render_large` made and that was false.
    ///
    /// This is a claim about **egui**, so it is asserted against egui rather
    /// than reasoned about. `render_large` painted itself greyed by choosing
    /// `visuals.widgets.inactive` by hand, and allocated its response from the
    /// ordinary `Ui` — so `response.enabled()` stayed true, its
    /// `on_disabled_hover_text` never opened, and `ribbon::control`'s
    /// `if response.clicked() { ctx.invoke(…) }` **still invoked the command**.
    /// The band said no and the shell did it anyway.
    ///
    /// The second half is the fix: allocating inside `ui.disable()`'s scope
    /// produces a response that reports itself disabled, which is what both
    /// the tooltip and the click gate read.
    ///
    /// ★ Written as a table over the two cases rather than asserting only the
    /// fixed one, because a build in which BOTH were disabled would satisfy a
    /// one-sided assertion and would grey every Large control permanently.
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

    /// ★★★ **`frame_when_inactive(false)` removes the resting frame and
    /// nothing else** — the egui contract the whole frameless change rests on.
    ///
    /// Asserted **against egui**, in the shape of
    /// [`only_a_disabled_scope_produces_a_disabled_response`] above, because
    /// it is a claim about a library rather than about this crate. The
    /// operator's complaint was *"every ribbon item in the real build is drawn
    /// with a visible button FRAME … the mockup draws them frameless"*, and
    /// `crate::ribbon::control::command_button`'s answer is one builder call.
    /// That call is only safe if two things are true of egui, and neither is
    /// obvious from the method's one-line doc:
    ///
    /// 1. **The button does not move.** `button.rs:363` keeps
    ///    `frame.inner_margin` in *both* branches — the framed one and the
    ///    margin-only one — so a control that loses its resting ink keeps its
    ///    rectangle. Everything the band plans is built on
    ///    `measure::button_padding`, and a frameless button that measured
    ///    differently from a framed one would make every planned group width
    ///    a lie at rest and true under the pointer.
    ///
    /// 2. **The ink really goes.** A frameless resting button emits strictly
    ///    fewer paint shapes than a framed one. Without this half the test
    ///    would pass against an implementation where the flag did nothing at
    ///    all — which is precisely the state this change is trying to leave.
    ///
    /// ★ The second assertion counts `Shape::Rect`s recursively, because
    /// `egui` nests shapes (`Shape::Vec`) and a top-level count would miss a
    /// frame painted inside a group. Counting *fewer* rather than an exact
    /// number is deliberate: the button also paints its text and, in the band,
    /// its icon, and pinning a total would make this test fail every time
    /// something unrelated changed how a label is emitted.
    #[test]
    fn frame_when_inactive_removes_the_resting_ink_and_not_the_rectangle() {
        /// How many rectangles this shape tree paints that a person could
        /// SEE — a fill that is not transparent, or a stroke with width.
        ///
        /// ★ Counting bare `Shape::Rect`s does not work, and finding that out
        /// is the useful half of this test. `egui`'s margin-only branch still
        /// emits a `RectShape` — `Frame::paint` always does — just one whose
        /// fill is `TRANSPARENT` and whose stroke is `Stroke::NONE`. A count
        /// of rectangles is therefore 1 in both cases and would have passed
        /// against an implementation where the flag did nothing at all, which
        /// is exactly the vacuity this test exists to avoid.
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

    /// ★★★ `Small` is earned three ways, and failing any one of them falls
    /// back to `Medium` rather than drawing an unlabelled rectangle.
    ///
    /// Asserted as a table over all eight combinations, because the rule is a
    /// conjunction and a test of one clause at a time would pass against an
    /// implementation that had dropped a different one.
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
    ///
    /// ★ `Large` deliberately does not require an icon: a large button with no
    /// icon is a large label, which is legible. The mystery this rule guards
    /// against is an icon with no name, and `Large` always draws its label.
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
