//! One stack's tab bar, and the overflow menu that makes it safe.
//!
//! # ★ The reservation, and the exact ordering that enforces it
//!
//! `MODES_AND_PANELS.md` Part 2, failure mode #8:
//!
//! > **Tab overflow has no escape** — past ~6 tabs the overflow *button
//! > itself* gets hidden, leaving no route to the hidden tabs. → *The
//! > overflow affordance is reserved space, never the first thing
//! > squeezed out.*
//!
//! And the same document's assessment of the engine originally chosen for
//! this dock, which is why the requirement is written down at all:
//!
//! > And one it walks straight into: **#8, tab overflow.** `egui_tiles`
//! > 0.16 answers an overflowing tab bar by hiding tabs behind scroll
//! > arrows with `ScrollBarVisibility::AlwaysHidden` — the same class of
//! > failure. The existing `dock.rs` already caps default tab groups at
//! > two panes specifically to dodge it, with a test enforcing the cap.
//! > **The ~1-day overflow menu is what retires that cap safely**, and it
//! > must reserve its own space rather than compete for it.
//!
//! This file is that overflow menu, and **the cap is retired**: nothing
//! in this crate limits how many panels a stack may hold, and
//! `a_stack_of_nine_panels_keeps_every_one_reachable` is the test that
//! replaces the old two-pane cap test. The cap was a mitigation for a
//! missing affordance; with the affordance present it would only be a
//! restriction.
//!
//! ## The order of operations, which is the whole mechanism
//!
//! ```text
//! 1. measure every label                 (cheap: memoized galleys)
//! 2. overflow_width  = max over EVERY reachable "⏷ N more" label
//! 3. tab_budget      = bar_width − overflow_width − gap        ← FIRST
//! 4. choose the visible window within tab_budget
//! 5. lay the tabs into a rect that IS tab_budget wide
//! 6. lay the affordance into the reserved rect
//! ```
//!
//! Step 3 happens before step 4, so no outcome of step 4 can reach the
//! reservation. Step 5 then enforces it a second time, in a different
//! currency: the tabs are given a rectangle whose width *is* the budget,
//! so `egui`'s own clipping backs up the arithmetic. Two independent
//! mechanisms, because the field report describes a control that was
//! *drawn* — the arithmetic alone is what everybody writes and it is what
//! failed.
//!
//! The arithmetic itself lives in [`super::plan`] with no `egui` in its
//! signatures, so it can be swept across hundreds of widths in a unit
//! test. This file is the rendering that obeys it.
//!
//! # Two more failure modes this file answers
//!
//! **#3 — the widest hidden tab dictates the minimum width.** Nothing
//! here reports a minimum size. The stack's width comes from the
//! column's share and [`super::plan::MIN_COLUMN_WIDTH`], both of which
//! are ignorant of the tab list. An inactive tab therefore cannot hold
//! the dock open — the observed defect where you must close a panel you
//! cannot see in order to narrow a dock you can.
//!
//! **#11 — focus-existing shows stale content.** *"Reopening a stacked
//! dialog selected its tab but kept rendering the previous one."* This
//! cannot arise here, because the tab bar does not *select* anything: it
//! records a `super::ctx::Intent`, and the body is drawn from
//! `stack.active` on the next frame. There is exactly one source of
//! truth for what is painted, and the tab bar reads it rather than
//! shadowing it.
//!
//! # Who owns a tab's secondary click
//!
//! The dock does, until an application asks for it — see
//! [`super::tab_menu`], which carries the whole argument. In one
//! paragraph:
//!
//! A tab's right-click is the dock's by default, and offers **Close**,
//! because that is the only verb the dock can name without knowing what
//! the application is. It is not the dock's by right. An application that
//! calls [`super::Dock::with_tab_menu`] is handed the tab's
//! [`egui::Response`] and the [`super::PanelId`] it belongs to, and the
//! dock draws no menu of its own for that tab — not out of politeness but
//! because `egui` gives a `Response` exactly one popup id, so two menus on
//! one tab are two writers of one open/closed flag. The dock's Close is
//! then reachable as [`super::TabMenu::request_close`], which produces the
//! same [`Intent::Close`] the built-in button does.
//!
//! # Accessibility
//!
//! Every tab carries a `WidgetInfo` with the panel's **purpose** as its
//! accessible name — not its label, which a screen-reader user would
//! learn nothing from that a sighted user does not already see — and its
//! selected state. This is carried across wholesale from the previous
//! implementation, which had to supply the same information manually
//! because its engine *"ships its tab bars unnamed to AccessKit"* while
//! still making them focusable, which it correctly called the worst case.
//!
//! The same honest limitation applies and is restated rather than
//! quietly dropped: **`egui` 0.35 has no `Tab` or `TabList`
//! [`egui::WidgetType`]** (see `D:/dev/rag/egui/egui_035_no_tab_tablist_widgettype.md`),
//! so these controls announce the right *name* and the right *selected
//! state* with the wrong *role*. Only the role is missing, and it cannot
//! be supplied short of an upstream change.
//!
//! Selected state is never colour alone: the active tab's label is drawn
//! **bold** as well as filled, because a weight cue survives greyscale
//! and colour-vision deficiency and a fill does not.

use egui::{Align, Layout, Rect, RichText, TextStyle, UiBuilder, Vec2};

use super::ctx::{Ctx, Intent};
use super::model::{DockSide, Stack};
use super::plan::{self, TabPlan};
use super::report;
use super::tab_menu::TabMenu;

/// What one tab bar did this frame.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct TabBarOutcome {
    /// How many tabs were moved into the overflow menu.
    pub hidden: usize,
    /// Whether the overflow affordance was drawn.
    pub overflow_drawn: bool,
}

/// Draw one stack's tab bar in `rect`, recording intents into `ctx`.
///
/// `rect` is the whole bar. The affordance's reservation is taken from
/// its right edge; see the module header for why that subtraction comes
/// first.
pub(crate) fn tab_bar(
    ui: &mut egui::Ui,
    ctx: &mut Ctx<'_>,
    side: DockSide,
    column: usize,
    stack_index: usize,
    stack: &Stack,
    rect: Rect,
) -> TabBarOutcome {
    let mut outcome = TabBarOutcome::default();
    if stack.tabs.is_empty() || rect.width() <= 0.0 {
        return outcome;
    }

    ui.painter().rect_filled(rect, 0.0, ctx.theme.palette.panel);
    ctx.reporter
        .report(ui, rect, || report::tab_bar(side, column, stack_index));

    // 1. Measure. `egui` memoizes layout jobs, so asking for the width of
    //    a label that is about to be drawn costs a hash lookup rather
    //    than a second text layout.
    let labels: Vec<String> = stack.tabs.iter().map(|p| ctx.describe(p).0).collect();
    let widths: Vec<f32> = labels
        .iter()
        .map(|l| plan::tab_width(text_width(ui, l)))
        .collect();

    // 2 & 3. The reservation, and the budget it is subtracted from.
    let overflow_w =
        plan::overflow_width(stack.tabs.len(), plan::TAB_PADDING, |s| text_width(ui, s));
    let bar = plan::plan_tabs(
        &widths,
        stack.active,
        rect.width(),
        plan::TAB_GAP,
        overflow_w,
    );

    outcome.hidden = bar.hidden;

    // 4 & 5. The visible window, laid into a rect that IS the budget.
    let mut x = rect.left();
    for i in bar.start..bar.start + bar.shown {
        let width = widths[i].min((rect.left() + bar.tab_budget - x).max(0.0));
        if width <= 0.0 {
            break;
        }
        let tab_rect =
            Rect::from_min_size(egui::pos2(x, rect.top()), Vec2::new(width, rect.height()));
        draw_tab(
            ui,
            ctx,
            side,
            column,
            stack_index,
            stack,
            i,
            &labels[i],
            tab_rect,
        );
        x += width + plan::TAB_GAP;
    }

    // 6. The affordance, in the space nothing else was allowed to touch.
    if bar.has_overflow() {
        outcome.overflow_drawn = true;
        draw_overflow(
            ui,
            ctx,
            side,
            column,
            stack_index,
            stack,
            &labels,
            &bar,
            rect,
        );
    }

    outcome
}

/// Draw one tab button.
#[allow(clippy::too_many_arguments)]
fn draw_tab(
    ui: &mut egui::Ui,
    ctx: &mut Ctx<'_>,
    side: DockSide,
    column: usize,
    stack_index: usize,
    stack: &Stack,
    index: usize,
    label: &str,
    rect: Rect,
) {
    let panel = &stack.tabs[index];
    let selected = index == stack.active;
    let (_, announced) = ctx.describe(panel);

    // R84 — selected state is never colour alone. The fill is the
    // familiar cue; the weight is the one that survives greyscale and
    // colour-vision deficiency, and the previous implementation added it
    // for exactly that reason after an audit found colour-fill-only
    // selection to be a recurring blind spot on this project.
    let text = if selected {
        RichText::new(label)
            .strong()
            .color(ctx.theme.palette.on_accent)
    } else {
        RichText::new(label).color(ctx.theme.palette.text_muted)
    };

    let id = ctx
        .id("tab", side, column, stack_index)
        .with(panel.as_str());
    let response = ui
        .scope_builder(
            UiBuilder::new()
                .id_salt(id)
                .max_rect(rect)
                .layout(Layout::left_to_right(Align::Center)),
            |ui| {
                ui.set_max_width(rect.width());
                // `min_size` fills the allocation so the control is
                // exactly as big as the arithmetic promised, and
                // `truncate` is the other half: without it a label wider
                // than its rect makes the *button* wider than the rect,
                // and the tab would overhang into the reservation — the
                // defect this file exists to prevent, arriving from the
                // tab's side rather than the affordance's.
                // ★★★ THE PLATE IS STATED, and not stating it was a defect
                // that shipped in all three presets — 2026-09-03.
                //
                // `Button::selected(true)` alone does NOT leave the fill
                // alone. `egui::Style::button_style` overwrites it:
                //
                //     visuals.weak_bg_fill = self.visuals.selection.bg_fill;
                //     visuals.bg_fill      = self.visuals.selection.bg_fill;
                //     visuals.fg_stroke    = self.visuals.selection.stroke;
                //     ws.text.color        = self.visuals.selection.stroke.color;
                //
                // (`egui-0.35.0/src/widget_style.rs:150-155`, verbatim.)
                //
                // This theme points `selection.bg_fill` at
                // `Palette::selection_fill`, a **27 %-alpha wash** whose real
                // job is tinting selected objects on the CANVAS. Composited
                // over the tab bar's `palette.panel` it leaves the
                // `on_accent` label above with a luminance gap of
                //
                //     Quiet 44.8 · Airy 28.2 · Dark 52.6
                //
                // against this project's own readable floor of **90**. Airy is
                // the worst because its panel is pure white, so the wash barely
                // darkens it.
                //
                // ★★ `ribbon::tabs` had the identical shape and already states
                // its fill — `Button::selectable(...).fill(accent)`. The two
                // are the same control in two docks and they now agree.
                //
                // ★★★ AND `tools/gates/check-strong-text.sh` WAS BLESSING THIS
                // SITE ON A FALSE PREMISE. Its header said of both tab files:
                // *"Both are drawn ON the accent fill, so `on_accent` is the
                // right colour anyway."* True of `ribbon/tabs.rs`, which fills.
                // This file contained **no `.fill(` at all**. The gate's
                // sentence is corrected in the same change; the sentence is
                // now true rather than merely written down.
                //
                // `.fill()` wins over the class-based styling because
                // `Button`'s own fill is applied after `button_style` has run.
                let mut button = egui::Button::new(text)
                    .min_size(rect.size())
                    .truncate()
                    .selected(selected);
                if selected {
                    button = button.fill(ctx.theme.palette.accent);
                }
                ui.add(button)
            },
        )
        .inner;

    if response.clicked() {
        ctx.intents.push(Intent::Activate(panel.clone()));
    }

    // ★ The accessible name is published BEFORE anything else is allowed
    // to touch this response.
    //
    // Both of the next two things — the built-in menu and, more to the
    // point, an application's handler — receive this response, and the
    // handler is arbitrary code the dock knows nothing about. Announcing
    // first means a tab announces itself correctly whatever the handler
    // does or fails to do: the seam can only *add* to the tab, never
    // silently subtract its name from the accessibility tree. See
    // `super::tab_menu`'s accessibility section.
    let response = response.on_hover_text(announced.clone());
    response.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::WidgetType::SelectableLabel,
            true,
            selected,
            announced.clone(),
        )
    });

    // ★ ONE owner for the secondary click, chosen once.
    //
    // Right-click to close, rather than an ✕ on every tab: a per-tab close
    // glyph would have to be inside the tab's width, so the tab-width
    // arithmetic — and therefore the reservation the whole module turns on
    // — would depend on whether a tab is closable. That is a coupling with
    // no benefit at this stage: closing a panel is an occasional act, and
    // a context menu is where a desktop operator already looks for
    // occasional acts on an object.
    //
    // But *what* a right-click offers is the application's business, not
    // the dock's, so an application may take the click instead. It cannot
    // take it *as well*: a context menu's popup id is
    // `response.id.with("popup")`, and two menus on one response are two
    // writers of one flag in `egui`'s memory. Whichever party owns it, the
    // close it produces is recorded as an `Intent` — never applied here —
    // so a menu-driven close and the built-in one are the same event
    // downstream, and `DockFrameReport::closed` is true for both.
    let mut close_requested = false;
    if let Some(handler) = ctx.tab_menu.as_deref_mut() {
        let mut tab = TabMenu::new(panel, &response);
        handler(&mut tab);
        close_requested = tab.close_requested();
    } else {
        response.context_menu(|ui| {
            if ui.button("Close").clicked() {
                close_requested = true;
                ui.close();
            }
        });
    }
    if close_requested {
        ctx.intents.push(Intent::Close(panel.clone()));
    }

    ctx.reporter
        .report(ui, response.rect, || report::tab(panel));
}

/// Draw the "⏷ N more" affordance and the menu behind it.
#[allow(clippy::too_many_arguments)]
fn draw_overflow(
    ui: &mut egui::Ui,
    ctx: &mut Ctx<'_>,
    side: DockSide,
    column: usize,
    stack_index: usize,
    stack: &Stack,
    labels: &[String],
    bar: &TabPlan,
    rect: Rect,
) {
    // ★ Clamped, not subtracted.
    //
    // `right − width` goes negative the moment the bar is narrower than
    // its own reservation, and a control at a negative x is drawn,
    // reported with a perfectly plausible `Rect`, and unclickable. That
    // is the exact failure the RAG entry
    // `a_sibling_row_that_overflows_grows_the_parent_max_rect_...`
    // records costing this project a real defect in the ribbon a few
    // hours before this file was written. Clamping keeps the affordance
    // on screen and spends the shortfall on characters, which the
    // tooltip recovers; spending it on position recovers nothing.
    let left = rect.left().max(rect.right() - bar.overflow_width);
    let control = Rect::from_min_max(egui::pos2(left, rect.top()), rect.right_bottom());

    let label = plan::overflow_label(bar.hidden);
    let id = ctx.id("overflow", side, column, stack_index);
    let response = ui
        .scope_builder(
            UiBuilder::new()
                .id_salt(id)
                .max_rect(control)
                .layout(Layout::left_to_right(Align::Center)),
            |ui| {
                ui.set_max_width(control.width());
                ui.add(
                    egui::Button::new(RichText::new(&label).color(ctx.theme.palette.text))
                        .min_size(control.size())
                        .truncate(),
                )
            },
        )
        .inner;

    ctx.reporter.report(ui, response.rect, || {
        report::overflow(side, column, stack_index)
    });

    // The count is the information. "Button" would be as useless here as
    // it is on an icon.
    let announced = format!("{label} panels");
    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, true, announced.clone())
    });
    let response = response.on_hover_text(format!(
        "{} panel{} do not fit; open them here",
        bar.hidden,
        if bar.hidden == 1 { "" } else { "s" }
    ));

    egui::Popup::menu(&response).show(|ui| {
        // EVERY hidden tab, from both ends of the window — those before
        // `start` as well as those after it. A menu that listed only the
        // trailing ones would leave the leading tabs reachable by
        // nothing at all once the operator scrolled the strip, which is
        // failure mode #8 with the affordance present and lying.
        for (i, label) in labels.iter().enumerate() {
            if bar.is_visible(i) {
                continue;
            }
            let panel = &stack.tabs[i];
            let (_, announced) = ctx.describe(panel);
            if ui.button(label).on_hover_text(announced).clicked() {
                ctx.intents.push(Intent::Activate(panel.clone()));
                ui.close();
            }
        }
    });
}

/// Measure a string in the font `egui` will draw it in.
///
/// Uses [`egui::Color32::PLACEHOLDER`] so the galley produced here is the
/// **same cache entry** the widget will later ask for with its real
/// colour — `egui` memoizes layout jobs, and a placeholder-coloured
/// galley is the form it stores. Measuring therefore costs a hash lookup
/// rather than a second text layout.
///
/// Deliberately a local six-line function rather than a call into the
/// ribbon's identical helper. The two surfaces are independently
/// refactorable, and a shared private helper between them would make the
/// dock's width arithmetic break when the ribbon's file layout changed —
/// a coupling with no upside, since the body of the function is the
/// documentation.
fn text_width(ui: &egui::Ui, text: &str) -> f32 {
    if text.is_empty() {
        return 0.0;
    }
    let font_id = TextStyle::Button.resolve(ui.style());
    ui.ctx().fonts_mut(|fonts| {
        fonts
            .layout_no_wrap(text.to_owned(), font_id, egui::Color32::PLACEHOLDER)
            .size()
            .x
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dock::model::{Column, DockLayout, PanelId, PanelInfo, PanelRegistry, SideLayout};
    use crate::dock::{Dock, DockFrameReport, DockState, RectReport};

    /// A registry of `n` panels with short, distinct labels.
    fn registry(n: usize) -> PanelRegistry {
        let mut r = PanelRegistry::new();
        for i in 0..n {
            r.register(
                PanelInfo::new(format!("p{i}"), format!("Panel {i}")).with_tooltip(format!(
                    "Panel {i} — the {i}th thing to look at while working"
                )),
            );
        }
        r
    }

    /// One stack holding `n` tabs on the left side.
    fn layout(n: usize, width: f32) -> DockLayout {
        DockLayout::new(
            SideLayout::new([Column::new([Stack::tabbed(
                (0..n).map(|i| format!("p{i}")).collect::<Vec<_>>(),
            )])])
            .with_width(width),
            SideLayout::none(),
        )
    }

    /// Render one frame at a given window size and collect every
    /// published rect.
    fn render(n: usize, dock_width: f32, window: Vec2) -> Vec<(String, Rect)> {
        let registry = registry(n);
        let mut state = DockState::new(layout(n, dock_width));
        let mut rects: Vec<(String, Rect)> = Vec::new();
        let ctx = egui::Context::default();
        {
            let mut sink = |r: &RectReport<'_>| rects.push((r.name.to_owned(), r.rect));
            let input = egui::RawInput {
                screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, window)),
                ..Default::default()
            };
            let _ = ctx.run_ui(input, |ui| {
                Dock::new()
                    .with_registry(&registry)
                    .reporting_rects_to(&mut sink)
                    .show(ui, &mut state, |_panel, _ui| {});
            });
        }
        rects
    }

    /// ★ **The two-pane cap is retired: nine panels in one stack, and
    /// every one of them is reachable.**
    ///
    /// The previous implementation capped a default tab group at two
    /// panes and enforced the cap with a test, *"specifically to dodge"*
    /// the engine hiding overflowing tabs behind scroll arrows. This is
    /// the test that replaces it, and it asserts the property the cap was
    /// a proxy for: at a width that cannot show nine tabs, the affordance
    /// exists — so nothing is stranded.
    #[test]
    fn a_stack_of_nine_panels_keeps_every_one_reachable() {
        let rects = render(9, 240.0, Vec2::new(1280.0, 800.0));
        let overflow = rects
            .iter()
            .find(|(n, _)| n.ends_with(".overflow"))
            .map(|(_, r)| *r);
        let overflow = overflow.expect("nine tabs in a 240 pt dock must overflow");
        assert!(
            overflow.width() > 1.0 && overflow.height() > 1.0,
            "the affordance has no area: {overflow:?}"
        );
    }

    /// ★ **Failure mode #8 against a rendered frame: the affordance is
    /// inside the bar it belongs to.**
    ///
    /// The unit tests in [`super::super::plan`] prove the arithmetic; this
    /// proves the *drawing* obeys it. Both are needed, because the field
    /// report describes a control that was computed correctly and placed
    /// where nobody could click it.
    #[test]
    fn the_overflow_affordance_is_drawn_inside_its_tab_bar() {
        let rects = render(9, 240.0, Vec2::new(1280.0, 800.0));
        let bar = rects
            .iter()
            .find(|(n, _)| n.ends_with(".tabbar"))
            .map(|(_, r)| *r)
            .expect("a tab bar was published");
        let overflow = rects
            .iter()
            .find(|(n, _)| n.ends_with(".overflow"))
            .map(|(_, r)| *r)
            .expect("an overflow affordance was published");
        assert!(
            overflow.right() <= bar.right() + 1.0,
            "the affordance {overflow:?} hangs off the bar {bar:?}"
        );
        assert!(
            overflow.left() >= bar.left() - 1.0,
            "the affordance {overflow:?} starts left of the bar {bar:?}"
        );
    }

    /// No visible tab overlaps the affordance's reserved space.
    #[test]
    fn no_visible_tab_encroaches_on_the_reserved_space() {
        let rects = render(9, 240.0, Vec2::new(1280.0, 800.0));
        let overflow = rects
            .iter()
            .find(|(n, _)| n.ends_with(".overflow"))
            .map(|(_, r)| *r)
            .expect("an overflow affordance was published");
        for (name, rect) in &rects {
            if !name.starts_with("dock.tab.") {
                continue;
            }
            assert!(
                rect.right() <= overflow.left() + 1.0,
                "tab {name} at {rect:?} overlaps the affordance at {overflow:?}"
            );
        }
    }

    /// Two panels in a wide dock fit, so nothing is reserved — an
    /// affordance that took space with nothing to show would be a
    /// permanent tax.
    #[test]
    fn a_stack_that_fits_draws_no_affordance() {
        let rects = render(2, 400.0, Vec2::new(1600.0, 800.0));
        assert!(
            !rects.iter().any(|(n, _)| n.ends_with(".overflow")),
            "nothing overflowed, yet an affordance was drawn"
        );
        assert_eq!(
            rects
                .iter()
                .filter(|(n, _)| n.starts_with("dock.tab."))
                .count(),
            2,
            "both tabs should be visible"
        );
    }

    /// The active tab is drawn even when it is late in a long list — the
    /// visible set is a window, not a prefix.
    #[test]
    fn the_active_tab_is_drawn_even_when_it_is_the_last_of_nine() {
        let registry = registry(9);
        let mut layout = layout(9, 240.0);
        layout.activate(&PanelId::new("p8"));
        let mut state = DockState::new(layout);
        let mut rects: Vec<(String, Rect)> = Vec::new();
        let ctx = egui::Context::default();
        {
            let mut sink = |r: &RectReport<'_>| rects.push((r.name.to_owned(), r.rect));
            let input = egui::RawInput {
                screen_rect: Some(Rect::from_min_size(
                    egui::Pos2::ZERO,
                    Vec2::new(1280.0, 800.0),
                )),
                ..Default::default()
            };
            let _ = ctx.run_ui(input, |ui| {
                Dock::new()
                    .with_registry(&registry)
                    .reporting_rects_to(&mut sink)
                    .show(ui, &mut state, |_panel, _ui| {});
            });
        }
        assert!(
            rects.iter().any(|(n, _)| n == "dock.tab.p8"),
            "the active tab was not drawn; published: {:?}",
            rects.iter().map(|(n, _)| n).collect::<Vec<_>>()
        );
    }

    // =======================================================================
    // The tab-menu seam
    //
    // These drive the dock across several frames with synthetic pointer
    // input, because the two claims that matter cannot be observed from one
    // static render: that an application handed the tab's `Response` can do
    // something with it, and that an application that was handed nothing
    // still has the Close it has always had. See `super::super::tab_menu`.
    // =======================================================================

    /// What one rendered frame produced.
    struct Frame {
        /// The dock's own report — `closed`, `activated`, `layout_changed`.
        report: DockFrameReport,
        /// Every rect the dock published, by stable name.
        rects: Vec<(String, Rect)>,
        /// `egui`'s accessibility/interaction events for the frame. A
        /// clicked widget appears here **with the `WidgetInfo` the widget
        /// published**, which is how a test can see an accessible name
        /// without the `accesskit` feature being on.
        events: Vec<egui::output::OutputEvent>,
    }

    impl Frame {
        /// The rect published under an exact name.
        fn rect(&self, name: &str) -> Option<Rect> {
            self.rects.iter().find(|(n, _)| n == name).map(|(_, r)| *r)
        }

        /// Every panel whose tab was drawn, in bar order.
        fn drawn_tabs(&self) -> Vec<String> {
            self.rects
                .iter()
                .filter_map(|(n, _)| n.strip_prefix("dock.tab.").map(str::to_owned))
                .collect()
        }
    }

    /// A dock driven frame by frame, with or without a tab-menu handler.
    ///
    /// The `egui::Context` is kept across frames deliberately: `egui`
    /// interacts against the **previous** frame's widget rects and keeps a
    /// popup's open state in memory, so a click is a two-frame event and a
    /// menu is a three-frame one. A per-frame context would make every
    /// interaction test silently do nothing.
    struct Harness {
        ctx: egui::Context,
        registry: PanelRegistry,
        state: DockState,
        window: Vec2,
    }

    impl Harness {
        fn new(n: usize, dock_width: f32, window: Vec2) -> Self {
            let ctx = egui::Context::default();
            // ★ Real font metrics, or every rect below is satisfied by text
            // that occupies no space — including the menu's own rows, which
            // would collapse the popup to a few points and make a click
            // "inside the menu" land nowhere. See `super::super::width_tests`
            // and the `#[path]` note in `super::super`'s header.
            crate::dock::testfont::install(&ctx);
            Self {
                ctx,
                registry: registry(n),
                state: DockState::new(layout(n, dock_width)),
                window,
            }
        }

        /// One frame with the dock owning the tabs' secondary click.
        fn frame(&mut self, events: Vec<egui::Event>) -> Frame {
            self.run(events, None::<&mut fn(&mut TabMenu<'_>)>)
        }

        /// One frame with `handler` owning it instead.
        fn frame_with(
            &mut self,
            events: Vec<egui::Event>,
            handler: &mut impl FnMut(&mut TabMenu<'_>),
        ) -> Frame {
            self.run(events, Some(handler))
        }

        fn run<H: FnMut(&mut TabMenu<'_>)>(
            &mut self,
            events: Vec<egui::Event>,
            handler: Option<&mut H>,
        ) -> Frame {
            let Self {
                ctx,
                registry,
                state,
                window,
            } = self;
            let mut rects: Vec<(String, Rect)> = Vec::new();
            let mut report = DockFrameReport::default();
            let input = egui::RawInput {
                screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, *window)),
                events,
                ..Default::default()
            };
            let out = {
                let mut sink = |r: &RectReport<'_>| rects.push((r.name.to_owned(), r.rect));
                let mut handler = handler;
                ctx.run_ui(input, |ui| {
                    let mut dock = Dock::new()
                        .with_registry(registry)
                        .reporting_rects_to(&mut sink);
                    if let Some(h) = handler.as_deref_mut() {
                        dock = dock.with_tab_menu(h);
                    }
                    report = dock.show(ui, state, |_panel, _ui| {});
                })
            };
            Frame {
                report,
                rects,
                events: out.platform_output.events,
            }
        }

        /// Click `button` at `pos`, and return the frame the click landed
        /// on.
        ///
        /// ★ **Two frames, and the first one is not optional.** `egui`
        /// resolves a press against the hit test it computed *before* the
        /// frame ran, from the pointer position it had then — so a
        /// pointer that arrives and presses in the same pass presses on
        /// whatever was under its *previous* position, and a widget that
        /// has just appeared (a menu row, say) is never hovered and never
        /// clicked. Measured here: with move-and-press in one frame the
        /// menu row reported `contains_pointer = true` and
        /// `hovered = false`, the click went to the layer underneath, and
        /// the menu closed as "clicked outside" — a test that would have
        /// concluded the built-in Close was broken.
        ///
        /// So: one frame to move, one to click. This is a property of
        /// driving `egui` from synthetic input, not of the dock.
        fn click(&mut self, pos: egui::Pos2, button: egui::PointerButton) -> Frame {
            let _ = self.frame(vec![egui::Event::PointerMoved(pos)]);
            self.frame(vec![
                egui::Event::PointerButton {
                    pos,
                    button,
                    pressed: true,
                    modifiers: egui::Modifiers::default(),
                },
                egui::Event::PointerButton {
                    pos,
                    button,
                    pressed: false,
                    modifiers: egui::Modifiers::default(),
                },
            ])
        }

        /// [`Self::click`], with `handler` owning the tabs' secondary
        /// click for both frames.
        fn click_with(
            &mut self,
            pos: egui::Pos2,
            button: egui::PointerButton,
            handler: &mut impl FnMut(&mut TabMenu<'_>),
        ) -> Frame {
            let _ = self.frame_with(vec![egui::Event::PointerMoved(pos)], handler);
            self.frame_with(
                vec![
                    egui::Event::PointerButton {
                        pos,
                        button,
                        pressed: true,
                        modifiers: egui::Modifiers::default(),
                    },
                    egui::Event::PointerButton {
                        pos,
                        button,
                        pressed: false,
                        modifiers: egui::Modifiers::default(),
                    },
                ],
                handler,
            )
        }

        /// The rect of the topmost open popup, if one is open.
        ///
        /// A context menu is an `Area` at [`egui::Order::Foreground`], so
        /// the top foreground layer *is* the menu. Read from memory rather
        /// than from the dock's rect sink because the popup is `egui`'s
        /// surface, not the dock's — the dock never learns where it went.
        fn popup_rect(&self) -> Option<Rect> {
            self.ctx.memory(|m| {
                let layer = m.areas().top_layer_id(egui::Order::Foreground)?;
                m.area_rect(layer.id)
            })
        }

        fn any_popup_open(&self) -> bool {
            egui::Popup::is_any_open(&self.ctx)
        }
    }

    /// ★ **A supplied handler is offered every drawn tab, and each one
    /// knows which panel it is.**
    ///
    /// The seam's central claim. If the handler were offered the wrong
    /// `PanelId` — the stack's active one, say, or the last one drawn —
    /// every application menu would act on the wrong panel while looking
    /// completely correct, because the popup would still appear under the
    /// pointer. That is the defect this test exists to make impossible.
    #[test]
    fn a_supplied_handler_is_offered_every_drawn_tab_with_its_own_panel_id() {
        let mut h = Harness::new(3, 400.0, Vec2::new(1600.0, 800.0));
        let mut seen: Vec<(String, Rect)> = Vec::new();
        let frame = h.frame_with(vec![], &mut |tab: &mut TabMenu<'_>| {
            seen.push((tab.panel().as_str().to_owned(), tab.response().rect));
        });

        let offered: Vec<String> = seen.iter().map(|(p, _)| p.clone()).collect();
        assert_eq!(
            offered,
            vec!["p0".to_owned(), "p1".to_owned(), "p2".to_owned()],
            "the handler must see every drawn tab, in bar order, once"
        );
        assert_eq!(offered, frame.drawn_tabs(), "offered ≠ drawn");

        // And the `Response` handed out is the tab that was drawn — not
        // some ancestor `Ui`'s response, which would put an application's
        // menu at the wrong place and give it the wrong hit area.
        for (panel, rect) in &seen {
            let published = frame
                .rect(&format!("dock.tab.{panel}"))
                .unwrap_or_else(|| panic!("{panel} published no rect"));
            assert_eq!(*rect, published, "{panel}'s response is not its tab");
        }
    }

    /// A tab behind the overflow affordance is not offered, because it was
    /// never drawn and therefore has no `Response` to hand out.
    ///
    /// Stated as a test rather than left implicit: an application that
    /// counted handler calls to enumerate its panels would be wrong, and
    /// the honest answer — ask the layout, not the seam — is the same one
    /// [`DockFrameReport::panels_drawn`]'s documentation gives.
    #[test]
    fn a_tab_hidden_behind_the_overflow_affordance_is_not_offered() {
        let mut h = Harness::new(9, 240.0, Vec2::new(1280.0, 800.0));
        let mut offered: Vec<String> = Vec::new();
        let frame = h.frame_with(vec![], &mut |tab: &mut TabMenu<'_>| {
            offered.push(tab.panel().as_str().to_owned());
        });
        assert_eq!(offered, frame.drawn_tabs());
        assert!(
            offered.len() < 9,
            "nine tabs in a 240 pt dock must not all fit; got {offered:?}"
        );
        assert!(
            !offered.is_empty(),
            "something must still be drawn and offered"
        );
    }

    /// ★ **A handler that asks for a close gets the dock's own close
    /// path** — not a mutation of its own, and not a second mechanism.
    ///
    /// `request_close` is what makes the seam compatible with the intent
    /// model instead of a hole in it: the panel is gone, the frame report
    /// names it, and `layout_changed` is set, so an application that
    /// persists on that flag persists this close exactly as it persists a
    /// splitter drag.
    #[test]
    fn a_handler_that_requests_a_close_goes_through_the_intent_queue() {
        let mut h = Harness::new(3, 400.0, Vec2::new(1600.0, 800.0));
        let frame = h.frame_with(vec![], &mut |tab: &mut TabMenu<'_>| {
            if tab.panel().as_str() == "p1" {
                tab.request_close();
            }
        });

        assert_eq!(
            frame.report.closed,
            Some(PanelId::new("p1")),
            "the close was not reported as the dock's own"
        );
        assert!(
            frame.report.layout_changed,
            "a close must mark the layout worth saving"
        );
        // The panel was still drawn on the frame it was closed on — the
        // layout is immutable while it is drawn — and is gone on the next.
        assert!(frame.drawn_tabs().contains(&"p1".to_owned()));
        let after = h.frame_with(vec![], &mut |_: &mut TabMenu<'_>| {});
        assert_eq!(after.drawn_tabs(), vec!["p0".to_owned(), "p2".to_owned()]);
        assert!(!h.state.layout().contains(&PanelId::new("p1")));
    }

    /// ★ **With no handler, right-click ▸ Close still closes the panel.**
    ///
    /// The compatibility guarantee, driven end to end through real pointer
    /// input rather than asserted about the code: secondary-click the tab,
    /// the dock's own menu opens, click its one row, the panel is gone.
    /// Any consumer that has not adopted the seam must see exactly this,
    /// which is why the seam landing must not be able to break it silently.
    #[test]
    fn the_built_in_close_still_closes_a_panel_with_no_handler() {
        let mut h = Harness::new(2, 400.0, Vec2::new(1600.0, 800.0));
        let first = h.frame(vec![]);
        let tab = first.rect("dock.tab.p1").expect("p1's tab was drawn");

        let opened = h.click(tab.center(), egui::PointerButton::Secondary);
        assert_eq!(opened.report.closed, None, "nothing closes on the click");
        assert!(
            h.any_popup_open(),
            "the dock's own context menu did not open"
        );

        let menu = h.popup_rect().expect("the menu has a rect");
        let closed = h.click(menu.center(), egui::PointerButton::Primary);
        assert_eq!(
            closed.report.closed,
            Some(PanelId::new("p1")),
            "the built-in Close did not close the panel"
        );
        assert!(closed.report.layout_changed);
    }

    /// ★ **One `Response`, one popup, one owner.**
    ///
    /// A handler that attaches nothing means a right-click that does
    /// nothing — the dock must not "helpfully" fall back to its own menu,
    /// because a fallback is exactly the second writer of
    /// `response.id.with("popup")` that the seam exists to avoid, and it
    /// would arrive on whichever frame the application's own menu declined
    /// to open. The contrast with the previous test is the whole point:
    /// same input, same tab, opposite outcome, decided by one builder call.
    #[test]
    fn a_supplied_handler_takes_the_secondary_click_away_from_the_dock() {
        let mut h = Harness::new(2, 400.0, Vec2::new(1600.0, 800.0));
        let mut nothing = |_: &mut TabMenu<'_>| {};
        let first = h.frame_with(vec![], &mut nothing);
        let tab = first.rect("dock.tab.p1").expect("p1's tab was drawn");

        let after = h.click_with(tab.center(), egui::PointerButton::Secondary, &mut nothing);
        assert!(
            !h.any_popup_open(),
            "the dock drew a menu on a tab whose click the application owns"
        );
        assert_eq!(after.report.closed, None);
    }

    /// ★ **The accessible name is published before the response is handed
    /// out**, so a handler cannot cost the tab its announcement.
    ///
    /// Observed through `egui`'s own output events: a clicked widget emits
    /// `OutputEvent::Clicked` carrying the `WidgetInfo` the widget
    /// published, which is the same value that fills an accesskit node.
    /// The name is the panel's **purpose** — its tooltip — per
    /// `crate::ribbon::a11y`'s convention and this module's header.
    #[test]
    fn a_tab_announces_its_purpose_even_when_a_handler_owns_its_menu() {
        let mut h = Harness::new(2, 400.0, Vec2::new(1600.0, 800.0));
        let mut nothing = |_: &mut TabMenu<'_>| {};
        let first = h.frame_with(vec![], &mut nothing);
        let tab = first.rect("dock.tab.p1").expect("p1's tab was drawn");

        let clicked = h.click_with(tab.center(), egui::PointerButton::Primary, &mut nothing);
        let announced: Vec<String> = clicked
            .events
            .iter()
            .filter_map(|e| match e {
                egui::output::OutputEvent::Clicked(info) => info.label.clone(),
                _ => None,
            })
            .collect();
        assert!(
            announced.iter().any(|l| l.starts_with("Panel 1 —")),
            "the tab did not announce its purpose; announced: {announced:?}"
        );
    }
}
