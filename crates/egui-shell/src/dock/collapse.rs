//! # `dock::collapse` — **the little tabs that minimise a side, and the rail
//! that brings it back**
//!
//! ## Why this is its own file
//!
//! R2, on 2026-08-20, when the pair pushed `dock` past 1,500 lines. It is a
//! real seam rather than a convenient cut: everything here answers one
//! question — *how does an operator make a side go away, and how do they get it
//! back?* — and nothing here draws a panel, lays out a column or resolves a
//! width.
//!
//! ## The operator's ask, and the half that was missing
//!
//! > *"add the little tabs that allow the left and right panels to be
//! > minimized."* — 2026-08-20
//!
//! The model already had `visible`, and a ribbon command already flipped it. So
//! minimising was *possible* and had no affordance, and — worse — a collapsed
//! side drew **nothing at all**, which meant the only route back was a command
//! the operator had to know existed.
//!
//! A panel with no visible handle has not been minimised. It has been **lost**,
//! and somebody who collapsed one by accident has no way to discover what
//! happened. Every program in the class leaves a rail: VS Code's activity bar,
//! Visual Studio's auto-hide tabs, Photoshop's collapsed dock strip. The shape
//! varies; the presence does not.
//!
//! ## The two controls are mirror images, and the chevrons must agree
//!
//! | state | control | chevron points |
//! |---|---|---|
//! | open | the collapse tab, on the side's INNER edge | **out** — where the panel is going |
//! | collapsed | the rail, at the window edge | **in** — where the panel comes back from |
//!
//! Getting that pair backwards is a small thing that makes a control feel wrong
//! without an operator being able to say why.
//!
//! ## Both raise an intent; neither writes the layout
//!
//! Load-bearing rather than ceremonial. Flipping `visible` mid-frame would
//! change the width of a panel that has **already laid out inside it**, so one
//! frame would draw a body at one width inside a container at another. The
//! apply phase runs once, after every side has drawn.

use egui::Rect;

use super::DockFrameReport;
use super::ctx::{Ctx, Intent};
use super::model::DockSide;
use super::plan;
use super::report;

/// How wide the rail a collapsed side leaves behind is, in points.
///
/// Narrow enough that it costs the document almost nothing, wide enough that it
/// is unmistakably a control rather than a border. Every program in the class
/// lands in the same range.
const RAIL_WIDTH_PTS: f32 = 16.0;

/// How tall the clickable part of a collapse control or a rail is, in points.
///
/// ★ Larger than the glyph it contains, deliberately. `handles` H4: the live
/// target may exceed the drawn affordance and must never be smaller. A chevron
/// is a few points across and would be a miserable thing to hit.
const RAIL_HIT_PTS: f32 = 22.0;

/// How far down the rail's chevron sits.
///
/// Aligned with the top of where the panel's own tab bar would be, so
/// collapsing and expanding do not make the control jump. An operator who
/// clicks to collapse should find the way back under the pointer they still
/// have there.
const RAIL_TOP_PAD_PTS: f32 = 6.0;

/// **The rail a collapsed side leaves behind** — the way back.
///
/// The operator's ask of 2026-08-20: *"add the little tabs that allow the
/// left and right panels to be minimized."* The minimising half is
/// [`draw_collapse`]; this is the half that makes it reversible.
///
/// # ★★ Why a collapsed side must leave something on screen
///
/// Before this, a hidden side drew **nothing**. The only route back was a
/// ribbon command the operator had to know existed and go looking for. A
/// panel with no visible handle has not been minimised, it has been *lost*
/// — and an operator who collapsed it by accident has no way to discover
/// what happened.
///
/// Every program in the class leaves a rail: VS Code's activity bar,
/// Visual Studio's auto-hide tabs, Photoshop's collapsed dock strip. The
/// shape varies; the presence does not.
///
/// # It is not drawn for an EMPTY side
///
/// A side with no panels in it has nothing to bring back, and a control
/// that opened an empty compartment would be an affordance for something
/// that cannot happen — the no-placeholders rule, which this crate holds to
/// as strictly as its host does. The caller checks `is_empty` first.
///
/// # The chevron points where the panel will go
///
/// Inward on a collapsed side, because that is the direction the panel
/// arrives from. The mirror of the collapse control, which points outward.
/// Getting this backwards is a small thing that makes a control feel wrong
/// without the operator being able to say why.
pub(super) fn draw_collapsed_rail(
    ui: &mut egui::Ui,
    ctx: &mut Ctx<'_>,
    side: DockSide,
    report: &mut DockFrameReport,
) {
    let panel = match side {
        DockSide::Left => egui::Panel::left(egui::Id::new(("egui-shell-dock-rail", side.key()))),
        DockSide::Right => egui::Panel::right(egui::Id::new(("egui-shell-dock-rail", side.key()))),
    };
    let inner = panel
        // `exact_size` for the same reason the side itself uses it: a
        // content-dependent width next to a fit-to-viewport zoom is the
        // R128 oscillation, and a rail is the last place to reintroduce it.
        .exact_size(RAIL_WIDTH_PTS)
        .resizable(false)
        .show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(RAIL_TOP_PAD_PTS);
                let glyph = match side {
                    DockSide::Left => "\u{25b8}",
                    DockSide::Right => "\u{25c2}",
                };
                let button = ui.add(
                    egui::Button::new(egui::RichText::new(glyph).monospace())
                        .frame(false)
                        .min_size(egui::vec2(RAIL_WIDTH_PTS - 2.0, RAIL_HIT_PTS)),
                );
                if button.clicked() {
                    ctx.intents.push(Intent::ToggleSide(side));
                }
                button
            })
            .inner
        });

    // ★ The whole RAIL is published, not the button inside it, and the
    // difference is deliberate: the rail is what an operator aims at — a
    // thin strip at the edge of the window — and a driven check that aimed
    // at the glyph's own rect would be measuring the glyph's centring
    // rather than whether the strip is reachable.
    ctx.reporter
        .report(ui, inner.response.rect, || report::rail(side));
    report.sides_drawn.push(side);
}

/// **The collapse control on an open side** — the little tab that minimises
/// it.
///
/// Drawn at the top of the side, on its INNER edge, so it sits against the
/// canvas rather than against the window frame. That is where every
/// program in the class puts it, and the reason is that the operator's
/// hand is already over the document.
///
/// # It raises an intent rather than writing the layout
///
/// Everything in this crate that changes the layout does, and here it is
/// load-bearing rather than ceremonial: flipping `visible` mid-frame would
/// change the width of a panel that has **already laid out inside it**, so
/// the frame would draw a body at one width inside a container at another.
/// The apply phase runs once, after every side has drawn.
pub(super) fn draw_collapse(ctx: &mut Ctx<'_>, ui: &mut egui::Ui, side: DockSide, area: Rect) {
    // The chevron points OUT — the direction the panel is about to go. The
    // rail's points in. Getting this pair backwards is a small thing that
    // makes a control feel wrong without an operator being able to say why.
    let glyph = match side {
        DockSide::Left => "\u{25c2}",
        DockSide::Right => "\u{25b8}",
    };
    // ★★ ALWAYS THE TRAILING END OF THE TAB ROW — the right-hand end of the
    // dock, on both sides.
    //
    // The first attempt put it on each side's INNER edge, on the argument that
    // the operator's hand is already over the document. That is true and it is
    // not the constraint that binds: **tabs start at the dock's left edge in
    // both docks**, so on the right-hand dock the inner edge is exactly where
    // the first tab is — and the chevron landed on top of it. Caught by looking
    // at a screenshot, which is the only oracle a layout collision has.
    //
    // So the rule is the one every program in the class uses: the collapse
    // control sits at the END of the tab row, where the tabs are not. For the
    // left dock that happens to be the inner edge; for the right dock it is the
    // outer one.
    //
    // Inset by the splitter's thickness only on the side the splitter is on —
    // a click that could mean either "collapse" or "resize" is a click whose
    // meaning depends on a pixel.
    let t = plan::SPLITTER_THICKNESS;
    let right = match side {
        DockSide::Left => area.right() - t - RAIL_WIDTH_PTS,
        DockSide::Right => area.right() - RAIL_WIDTH_PTS,
    };
    let rect = Rect::from_min_size(
        egui::pos2(right, area.top()),
        egui::vec2(RAIL_WIDTH_PTS, RAIL_HIT_PTS),
    );
    let button = ui.put(
        rect,
        egui::Button::new(egui::RichText::new(glyph).monospace()).frame(false),
    );
    if button.clicked() {
        ctx.intents.push(Intent::ToggleSide(side));
    }
    ctx.reporter.report(ui, rect, || report::collapse(side));
}
