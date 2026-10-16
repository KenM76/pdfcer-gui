//! # `dock::collapse` — **the little tabs that minimise a side, and the rail
//! that brings it back**
//!
//! ## Why this is its own file
//!
//! Everything here answers one question — *how does an operator make a side go
//! away, and how do they get it back?* — and nothing here draws a panel, lays
//! out a column or resolves a width.
//!
//! ## The rule the pair exists to hold
//!
//! The operator's ask: *"add the little tabs that allow the left and right
//! panels to be minimized."*
//!
//! A model flag and a ribbon command make minimising *possible*; they do not
//! make it an affordance, and on their own they leave a collapsed side drawing
//! **nothing at all**, so the only route back is a command the operator has to
//! know exists.
//!
//! **A panel with no visible handle has not been minimised. It has been
//! *lost*** — and somebody who collapsed one by accident has no way to discover
//! what happened. Every program in the class leaves a rail: VS Code's activity
//! bar, Visual Studio's auto-hide tabs, Photoshop's collapsed dock strip. The
//! shape varies; the presence does not.
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
//!
//! Design and rationale: `docs/modules/egui-shell/dock/collapse.md`.

use egui::Rect;

use super::DockFrameReport;
use super::ctx::{Ctx, Intent};
use super::model::DockSide;
use super::plan;
use super::report;

/// How wide the rail a collapsed side leaves behind is, in points.
pub(super) const RAIL_WIDTH_PTS: f32 = 16.0;

/// How tall the clickable part of a collapse control or a rail is, in points.
const RAIL_HIT_PTS: f32 = 22.0;

/// How far down the rail's chevron sits.
const RAIL_TOP_PAD_PTS: f32 = 6.0;

/// **The rail a collapsed side leaves behind** — the way back.
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
        // No inner margin: the stock panel frame's would widen the content
        // past `RAIL_WIDTH_PTS`, and a right rail would then hang off the
        // window's edge with its arrow half clipped.
        .frame(egui::Frame::NONE.fill(ctx.theme.palette.panel))
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

    // The whole RAIL is published, not the button inside it, and the
    // difference is deliberate: the rail is what an operator aims at — a
    // thin strip at the edge of the window — and a driven check that aimed
    // at the glyph's own rect would be measuring the glyph's centring
    // rather than whether the strip is reachable.
    ctx.reporter
        .report(ui, inner.response.rect, || report::rail(side));
    report.sides_drawn.push(side);
}

/// Where an open side's collapse chevron sits: its INNER edge (the one
/// facing the document), just inside the side splitter, as most programs in
/// the class place it. The tab bar it sits on starts or ends clear of it
/// ([`super::stack`]), so no tab or "more" button is ever drawn under it; at
/// the trailing end the right dock's "more" button covered it.
pub(super) fn chevron_rect(side: DockSide, area: Rect) -> Rect {
    let t = plan::SPLITTER_THICKNESS;
    let left = match side {
        DockSide::Left => area.right() - t - RAIL_WIDTH_PTS,
        DockSide::Right => area.left() + t,
    };
    Rect::from_min_size(
        egui::pos2(left, area.top()),
        egui::vec2(RAIL_WIDTH_PTS, RAIL_HIT_PTS),
    )
}

/// **The collapse control on an open side** — the little tab that minimises
/// it.
pub(super) fn draw_collapse(ctx: &mut Ctx<'_>, ui: &mut egui::Ui, side: DockSide, area: Rect) {
    // The chevron points OUT — the direction the panel is about to go. The
    // rail's points in. Getting this pair backwards is a small thing that
    // makes a control feel wrong without an operator being able to say why.
    let glyph = match side {
        DockSide::Left => "\u{25c2}",
        DockSide::Right => "\u{25b8}",
    };
    let rect = chevron_rect(side, area);
    let button = ui.put(
        rect,
        egui::Button::new(egui::RichText::new(glyph).monospace()).frame(false),
    );
    if button.clicked() {
        ctx.intents.push(Intent::ToggleSide(side));
    }
    ctx.reporter.report(ui, rect, || report::collapse(side));
}
