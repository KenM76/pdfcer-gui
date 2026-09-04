//! The tab-strip row — the QAT, the tabs, the tabs' overflow menu and the
//! mode selector, and the arithmetic that stops any one of them eating the
//! others.
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────────────────┐
//! │ [Open][Save] │ File View Pages ⏷ 3 more  ( Read │ Review │ Edit ) [⧉] │
//! └──────────────────────────────────────────────────────────────────────┘
//!   └── QAT ───┘  └─── tabs ───┘└affordance┘ └─── mode selector ───┘  └──┘
//!        1               5           4                  2              3
//! ```
//!
//! The fifth region — [`super::trailing`] — is the only one on this row that
//! may be granted **nothing** when the row is narrow. See
//! [`super::plan::plan_strip_row`]'s ★★ for why an optional extra is a
//! different kind of claimant from a promise the interface has already made.
//!
//! # ★ The defect this module exists to retire
//!
//! `MODES_AND_PANELS.md` Part 2, failure mode #8:
//!
//! > **Tab overflow has no escape** — past ~6 tabs the overflow *button
//! > itself* gets hidden, leaving no route to the hidden tabs. → *The
//! > overflow affordance is reserved space, never the first thing squeezed
//! > out.*
//!
//! [`super::band`] and [`super::plan`] made that unreachable **in the
//! band**. The row above the band had the same defect and no cure, and it
//! was worse, because the band's overflow menu at least reaches every
//! group: the strip's hidden tabs could not be reached at all.
//!
//! What was there before was one right-to-left `Ui` with a left-to-right
//! `Ui` nested in it (`tabs::with_right_island`). The mode selector was
//! emitted first, from the right edge, and the QAT and tabs took the
//! remainder — which is exactly the "reserve first" shape, and it still
//! failed. Two reasons, and both are the same reason:
//!
//! 1. **`egui` does not clip a `Ui`'s children to its `max_rect`.** A
//!    widget that does not fit is still laid out, still allocated, still
//!    given a `Response` and a `Rect`. It is simply painted where nobody
//!    can see or click it. Nothing errors and nothing warns.
//! 2. **Reserving the right island protects the right island.** Nothing
//!    was reserved for the QAT, nothing was reserved for the tabs, and
//!    nothing at all was reserved for a tab overflow affordance because
//!    there was not one.
//!
//! Measured against the synthetic proportional face of
//! [`super::testfont`], with the two-control QAT and two tabs of the test
//! manifest:
//!
//! ```text
//! window  QAT             tabs                selector      verdict
//!  500    0..166          188..265            322..500      correct
//!  320    0..166          188..265            142..320      tabs UNDER the selector
//!  180   -6..160          182..259              2..180      both tabs off screen
//! ```
//!
//! At 180 pt the first QAT control starts at **x = −6** and neither tab is
//! on screen. Every unit test passed, because with
//! `egui = { default-features = false }` there is no font data, every
//! galley measures ≈ 0, the row always fits, and the failure cannot be
//! reproduced. See `D:\dev\rag\egui\` for both findings written up.
//!
//! # The cure: plan the row, then draw into the plan
//!
//! Nothing here nests layouts and hopes. The row is divided into three
//! rectangles **before a single widget is emitted**, and each region is
//! drawn into a `Ui` whose `max_rect` is its own rectangle:
//!
//! | Step | Function | What it decides |
//! |---|---|---|
//! | bounds | [`super::band::entitled_bounds`] | the row's true width — *not* `available_rect_before_wrap()` |
//! | 1 & 2 | [`super::plan::plan_strip_row`] | QAT ← left, selector ← right, tabs ← the rest |
//! | 3 & 4 | [`super::plan::plan_tab_strip`] | which tabs are drawn, which are in the menu, and the affordance's width |
//!
//! The reservation order is **the order of those two calls**, and it is
//! not re-derivable anywhere else: [`render`] computes `row`, calls
//! `plan_strip_row`, calls `plan_tab_strip` with `RowPlan::tabs`, and then
//! only draws. There is no branch in the drawing code that can reach into
//! a region it was not given, because the drawing code is not laying out
//! in that space.
//!
//! # ★ Four rules, and the disclosure each one carries
//!
//! Every degradation below is announced through [`crate::verify`], for the
//! reason that channel exists: *a control that silently rendered at less
//! than the size it asked for is a different fact from one that fitted*,
//! and the difference is invisible in a screenshot until somebody looks
//! hard at it.
//!
//! | Rule | What gives | Disclosure |
//! |---|---|---|
//! | The affordance is reserved before the tabs | tabs move into the menu | `ribbon-tab-strip-overflowed` |
//! | The **active tab is pinned** — never in the menu | its label truncates | `ribbon-active-tab-truncated` |
//! | The QAT may not consume the row | its labels truncate | `ribbon-qat-truncated` |
//! | The mode selector may not consume the row | its track compresses | `ribbon-mode-selector-compressed` |
//! | The trailing region yields before anything load-bearing | it is not drawn at all | `ribbon-trailing-dropped` |
//! | The affordance itself may be crowded | its label truncates | `ribbon-tab-overflow-clamped` |
//!
//! A **contextual** tab needs no rule of its own. It is appended last by
//! [`super::tabs::visible_tabs`], so a contextual tab arriving into a full
//! strip is simply the next thing the fill cannot place: it goes into the
//! menu like any other tab, it cannot displace the active one, and it is
//! announced by the affordance's count going up. See
//! [`super::plan::plan_tab_strip`] for why that falls out rather than
//! being arranged.
//!
//! # The `egui` 0.35 ceiling, restated
//!
//! [`super::a11y`] records that `egui` 0.35 has no `WidgetType::Tab` and
//! no `TabList`, so a ribbon tab announces as a *selectable label that is
//! or is not selected* rather than as "tab 2 of 7". That ceiling extends
//! to this module's affordance and menu: a screen-reader user is told
//! there is a button called "⏷ 3 more ribbon tabs" and, on opening it,
//! hears three more selectable labels. **The set relationship is not
//! announced** — nothing says the menu's contents belong to the same tab
//! list as the strip's. The count in the label is what carries that
//! information, which is why the count is in the label rather than being a
//! bare chevron, and it is the best this toolkit version allows. It is
//! stated here rather than papered over.

use egui::{Align, Layout, Rect, RichText, Sense, UiBuilder, pos2};

use crate::manifest::{Shell, Tab};

use super::band;
use super::ctx::Ctx;
use super::mode_selector;
use super::plan::{self, StripPlan};
use super::qat;
use super::report;
use super::tabs;
use super::trailing;

/// What the tab-strip row did on one frame.
///
/// Returned to [`super::render`] rather than written into
/// [`super::RibbonState`] here, for the reason that module's header gives:
/// a tab click or a mode change lands on the **next** frame, so nothing
/// drawn this frame can already be reacting to it.
#[derive(Debug, Clone, Default)]
pub(crate) struct StripOutcome {
    /// The tab the operator clicked, in the strip or in the menu.
    pub clicked_tab: Option<String>,
    /// The mode the operator chose, if it changed.
    pub chosen_mode: Option<String>,
    /// How many tabs were drawn in the strip itself.
    pub tabs_in_strip: usize,
    /// How many tabs the plan moved into the overflow menu.
    pub tabs_overflowed: usize,
    /// Whether the strip's overflow affordance was drawn.
    pub tab_overflow_visible: bool,
    /// Whether the strip gave up the pin because the row could not hold a
    /// tab and an affordance at once — see
    /// [`super::plan::plan_tab_strip`]'s collapse section.
    pub tab_strip_collapsed: bool,
    /// The `egui::Id` of that affordance, when one was drawn.
    ///
    /// Published for the same reason [`super::FrameReport::overflow_id`]
    /// is: a rectangle proves a thing was allocated, and only `egui`'s own
    /// hit test proves it can be reached.
    pub tab_overflow_id: Option<egui::Id>,
}

/// Draw the whole tab-strip row.
///
/// `entitled` is the rectangle the application handed
/// [`super::Ribbon::render`], read **before** anything was drawn into it.
/// It is a parameter rather than something this function derives for the
/// reason [`super::band::entitled_bounds`] gives at length: by the time a
/// row is being drawn, the `Ui` it is given may already have been widened
/// by a sibling that overflowed, and a `Ui` that reports a width the
/// window does not have is how a reserved control ends up off screen.
///
/// The tab-strip row is the *first* thing the ribbon draws, so in practice
/// nothing has had a chance to inflate anything yet. It is intersected
/// anyway, because "in practice nothing has yet" is a statement about the
/// current call order and not about this function, and the call order is
/// exactly the kind of thing a later edit reorders without noticing.
#[allow(clippy::too_many_arguments)]
pub(crate) fn render(
    ui: &mut egui::Ui,
    ctx: &mut Ctx<'_>,
    shell: &Shell,
    visible: &[&Tab],
    active_id: Option<&str>,
    selected_mode: Option<&str>,
    entitled: Rect,
) -> StripOutcome {
    let mut outcome = StripOutcome::default();

    // The row is exactly one control tall. `entitled_bounds` negotiates
    // only the horizontal extent (see its header), so the height is
    // applied here — allocating the full remaining rectangle would consume
    // the space the band is about to be drawn in.
    let bounds = band::entitled_bounds(ui, entitled);
    let height = tabs::strip_height(ctx);
    let row = Rect::from_min_max(
        bounds.min,
        pos2(bounds.right(), bounds.top() + height.max(0.0)),
    );
    let gap = ui.spacing().item_spacing.x;

    // -----------------------------------------------------------------
    // MEASURE. Every claimant states its natural width before any of them
    // is drawn. All three use the same galley cache and the same padding
    // constants — see `band::text_width` on why that matters.
    // -----------------------------------------------------------------
    let qat_wanted = qat::measure(ui, ctx, shell.qat.as_ref());
    let qat_floor = qat::min_width(ui, ctx, shell.qat.as_ref());
    let (_, selector_wanted) = mode_selector::measure_track(ui, shell.modes());
    let trailing_wanted = trailing::measure(ui, ctx, shell.trailing.as_ref());
    let trailing_floor = trailing::min_width(ui, ctx, shell.trailing.as_ref());
    let tab_widths: Vec<f32> = visible.iter().map(|t| tabs::measure_tab(ui, t)).collect();
    let affordance_wanted = plan::overflow_width(visible.len(), band::button_padding(ui), |s| {
        band::text_width(ui, s, &egui::TextStyle::Button)
    });

    // ★ The measured floor everything on this row turns on: below it,
    // `Button::truncate()` stops shrinking and the control is drawn
    // outside whatever rectangle it was given. See
    // `band::min_button_width`.
    let button_floor = band::min_button_width(ui);

    // -----------------------------------------------------------------
    // PLAN. Steps 1 and 2: the row's three regions, outermost first.
    // -----------------------------------------------------------------
    let row_plan = plan::plan_strip_row(
        row.width(),
        plan::RowDemand {
            qat: qat_wanted,
            qat_floor,
            selector: selector_wanted,
            // One control's width per position. `MODES_AND_PANELS.md`
            // Part 1's "all three labels visible" stops being true below
            // that, whatever the arithmetic says.
            selector_floor: selector_wanted.min(shell.modes().len() as f32 * button_floor),
            // A pinned tab and an affordance beside it.
            tabs_floor: 2.0 * button_floor + gap,
            button_floor,
            trailing: trailing_wanted,
            trailing_floor,
        },
    );
    if row_plan.trailing_dropped {
        // A control the operator asked to have on the ribbon and cannot
        // reach. See `plan_strip_row`'s ★★ on why this region is the one
        // allowed to vanish — and why vanishing still gets said out loud.
        crate::verify::event("ribbon-trailing-dropped")
            .kv("wanted", format!("{trailing_wanted:.1}"))
            .kv("row", format!("{:.1}", row.width()))
            .emit();
    }
    if row_plan.qat_truncated {
        crate::verify::event("ribbon-qat-truncated")
            .kv("wanted", format!("{qat_wanted:.1}"))
            .kv("granted", format!("{:.1}", row_plan.qat))
            .kv("row", format!("{:.1}", row.width()))
            .emit();
    }
    // The selector's own compression is disclosed by `mode_selector::render`
    // as `ribbon-mode-selector-compressed`; `row_plan.selector_truncated`
    // is the same fact one step earlier and would double-report it.

    // Steps 3 and 4: the affordance, then the tabs, inside what is left.
    let active_index = active_id.and_then(|id| visible.iter().position(|t| t.id == id));
    let tab_plan = plan::plan_tab_strip(
        row_plan.tabs,
        &tab_widths,
        active_index,
        gap,
        affordance_wanted,
        button_floor,
    );
    disclose_tab_plan(&tab_plan, visible, row_plan.tabs);
    outcome.tab_strip_collapsed = tab_plan.collapsed;

    // -----------------------------------------------------------------
    // RECTANGLES. Derived from the plan and from the row's own edges;
    // nothing below consults `ui.available_width()` again.
    // -----------------------------------------------------------------
    let (top, bottom) = (row.top(), row.bottom());
    let region =
        |left: f32, right: f32| Rect::from_min_max(pos2(left, top), pos2(left.max(right), bottom));

    let qat_rect = region(row.left(), row.left() + row_plan.qat);
    let trailing_rect = region(row.right() - row_plan.trailing, row.right());
    let selector_rect = region(
        trailing_rect.left() - row_plan.selector,
        trailing_rect.left(),
    );
    // The affordance hugs the right edge of the tab area, immediately left
    // of the selector. `plan_tab_strip` guarantees its reservation is no
    // wider than that area, so no clamp is needed here — the clamping is
    // the plan's, and it happens where it can be unit-tested.
    let affordance_rect = tab_plan.has_overflow().then(|| {
        region(
            selector_rect.left() - tab_plan.overflow_width,
            selector_rect.left(),
        )
    });
    let tabs_rect = region(qat_rect.right(), qat_rect.right() + tab_plan.tab_budget);

    // -----------------------------------------------------------------
    // DRAW. Each region into a `Ui` that is its own rectangle and nothing
    // more. `set_max_width` is the second half of the enforcement — the
    // first is that the rectangles above cannot overlap.
    // -----------------------------------------------------------------
    ui.allocate_rect(row, Sense::hover());

    island(ui, "egui-shell-ribbon-qat", qat_rect, |ui| {
        qat::render(ui, ctx, shell.qat.as_ref());
    });

    let shown: Vec<&Tab> = tab_plan.shown.iter().map(|&i| visible[i]).collect();
    let clicked_in_strip = island(ui, "egui-shell-ribbon-tabs", tabs_rect, |ui| {
        tabs::render_tabs(ui, ctx, &shown, active_id)
    });
    outcome.tabs_in_strip = shown.len();

    let mut clicked_in_menu = None;
    if let Some(rect) = affordance_rect {
        let hidden: Vec<&Tab> = tab_plan.hidden.iter().map(|&i| visible[i]).collect();
        let (picked, id) = render_affordance(ui, ctx, &hidden, active_id, rect);
        clicked_in_menu = picked;
        outcome.tabs_overflowed = hidden.len();
        outcome.tab_overflow_visible = true;
        outcome.tab_overflow_id = Some(id);
    }

    // ★ A zero-width region is not drawn at all. `mode_selector::render`
    // reads `ui.available_width()` as the room to compress its track into,
    // and `fit_track` reads a non-positive room as "no constraint known"
    // — correctly, because an unbounded container reports one — so
    // handing it a zero-width `Ui` would make it lay the track out at its
    // full natural size, from the right-hand edge of the row, straight
    // over the tabs. Observed at a 40 pt viewport: a 178 pt track at
    // x = 40..218.
    //
    // A row too narrow for any selector at all draws none. That is
    // disclosed by `plan_strip_row` marking it truncated, and it is the
    // honest outcome: the alternative is a control drawn where it cannot
    // be seen.
    if selector_rect.width() > 0.0 {
        outcome.chosen_mode = island(ui, "egui-shell-ribbon-modes", selector_rect, |ui| {
            mode_selector::render(ui, ctx, shell.modes(), selected_mode)
        });
    }

    // ★ The same zero-width guard the selector carries, and for a related
    // reason: a `Ui` with no width still lays its children out, and `egui`
    // does not clip them to it. A zero-width island here would draw the
    // control from the row's right edge leftwards, straight over the mode
    // selector — the one place on this row where an overlap is guaranteed to
    // be misread as a click on the wrong control.
    if trailing_rect.width() > 0.0 {
        island(ui, "egui-shell-ribbon-trailing", trailing_rect, |ui| {
            trailing::render(ui, ctx, shell.trailing.as_ref());
        });
    }

    // A click in the menu wins over one in the strip. They cannot both
    // happen in one frame — a click closes the popup — but stating the
    // precedence costs nothing and removes the question.
    outcome.clicked_tab = clicked_in_menu.or(clicked_in_strip);
    outcome
}

/// Lay a region out inside a `Ui` that **is** `rect`, left to right.
///
/// The `id_salt` is fixed per region rather than derived from the content,
/// so `egui`'s per-id state — focus, hover, an open popup — survives a
/// resize that moves a tab into or out of the menu. See
/// [`super::ctx::Ctx::id`] on why that matters: an id that shifts with the
/// layout produces a control that loses keyboard focus when the window is
/// dragged, which reads as a focus bug rather than as an id bug.
fn island<R>(
    ui: &mut egui::Ui,
    id_salt: &'static str,
    rect: Rect,
    body: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    ui.scope_builder(
        UiBuilder::new()
            .id_salt(id_salt)
            .max_rect(rect)
            .layout(Layout::left_to_right(Align::Center)),
        |ui| {
            ui.set_max_width(rect.width());
            body(ui)
        },
    )
    .inner
}

/// The "⏷ N more" affordance and the menu of hidden tabs behind it.
///
/// Modelled on [`super::band`]'s, deliberately, down to the `min_size` and
/// the `truncate()`:
///
/// - `min_size(rect.size())` makes the control exactly as big as the
///   arithmetic promised, so the reservation is not quietly under-spent.
/// - `truncate()` is the other half. Without it a label wider than the
///   rect makes the *button* wider than the rect, and the affordance hangs
///   into the mode selector in precisely the situation — a crowded row —
///   where it most needs to be reachable. Truncating spends the shortfall
///   on characters, which is recoverable: the tooltip states the count in
///   full.
///
/// Returns the tab the operator picked, if any, and the affordance's
/// `egui::Id` so a harness can hit-test it.
fn render_affordance(
    ui: &mut egui::Ui,
    ctx: &mut Ctx<'_>,
    hidden: &[&Tab],
    active_id: Option<&str>,
    rect: Rect,
) -> (Option<String>, egui::Id) {
    let label = plan::overflow_label(hidden.len());
    let id = ctx.id("tab-overflow", "strip");
    let response = ui
        .scope_builder(
            UiBuilder::new()
                .id_salt(id)
                .max_rect(rect)
                .layout(Layout::left_to_right(Align::Center)),
            |ui| {
                ui.set_max_width(rect.width());
                ui.add(
                    egui::Button::new(RichText::new(&label))
                        .min_size(rect.size())
                        .truncate(),
                )
            },
        )
        .inner;

    ctx.reporter
        .report_static(response.rect, report::tab_overflow());

    // The count is the information; "button" would be as useless here as
    // it is on an icon. See this module's header for what `egui` 0.35
    // cannot express beyond this.
    let announced = format!("{label} ribbon tabs");
    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, true, announced.clone())
    });
    let response = response.on_hover_text(format!(
        "{} tab{} do not fit; open them here",
        hidden.len(),
        if hidden.len() == 1 { "" } else { "s" }
    ));

    let mut picked = None;
    egui::Popup::menu(&response).show(|ui| {
        picked = tabs::render_overflow_menu(ui, ctx, hidden, active_id);
    });
    (picked, response.id)
}

/// Announce everything the tab plan gave up, once per frame.
///
/// Kept out of [`render`] so the drawing code reads as drawing. Each event
/// is a separate fact — a strip that overflowed, a pinned tab that
/// truncated, an affordance that was crowded — and a harness that wants
/// only one of them should not have to parse the others out of a combined
/// line.
fn disclose_tab_plan(plan: &StripPlan, visible: &[&Tab], room: f32) {
    if !plan.hidden.is_empty() {
        crate::verify::event("ribbon-tab-strip-overflowed")
            .kv("tabs", visible.len().to_string())
            .kv("shown", plan.shown.len().to_string())
            .kv("hidden", plan.hidden.len().to_string())
            .kv("room", format!("{room:.1}"))
            .emit();
    }
    if plan.active_truncated {
        crate::verify::event("ribbon-active-tab-truncated")
            .kv("budget", format!("{:.1}", plan.tab_budget))
            .emit();
    }
    if plan.overflow_truncated {
        crate::verify::event("ribbon-tab-overflow-clamped")
            .kv("granted", format!("{:.1}", plan.overflow_width))
            .kv("room", format!("{room:.1}"))
            .emit();
    }
    if plan.collapsed {
        // The one state in which the active tab is *not* in the strip.
        // Disclosed separately from `ribbon-tab-strip-overflowed` because
        // it is a different fact: the pin was given up, deliberately, to
        // keep every tab reachable. See `plan::plan_tab_strip`'s collapse
        // section.
        crate::verify::event("ribbon-tab-strip-collapsed")
            .kv("tabs", visible.len().to_string())
            .kv("room", format!("{room:.1}"))
            .kv("affordance", format!("{:.1}", plan.overflow_width))
            .emit();
    }
}
