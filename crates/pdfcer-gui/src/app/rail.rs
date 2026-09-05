//! # `app::rail` — drawing the left rail
//!
//! `OPERATOR_REQUESTS.md` **O123** part 7 and **O126**'s addendum. The
//! permanent vertical strip down the left dock's outer edge: the five panel
//! tabs, the navigate selectors, the selection controls, rotate.
//!
//! Three files, three jobs, and the split is the R7 line:
//!
//! | file | owns |
//! |---|---|
//! | `crate::shell::manifest::rail` | **what is in it** — command ids, groups, fold policy, as manifest data |
//! | `egui_shell::dock::rail` | **the geometry and the fold ladder** — a constant width, and which rows exist at which budget |
//! | this file | **what a row looks like**, and what a press does |
//!
//! `tools/gates/check-shell-purity.sh` is what makes that split load-bearing
//! rather than tidy: the shell may not learn that `pages` is a page thumbnail
//! list, so the shell plans ids and this file paints them.
//!
//! ## ★★★ Why every row publishes through `ui_rect_visible`
//!
//! `crate::diag::ui_rect` says *"this region was laid out at these
//! coordinates"*. That is **not** the claim a rail needs to make, and the
//! distance between the two claims is this exact feature's own defect: on
//! 2026-08-10 Bookmarks, Layers and Signatures shipped **unreachable**, each
//! with a rail entry, each publishing a perfectly healthy rectangle, every
//! gate green. `SHELL_LAYOUT_PROPOSAL.md` §5 made converting the dock's rect
//! channel from layout to visibility a **precondition** for scheduling this
//! work, precisely because no driven check could otherwise tell a working rail
//! from that defect. The channel was converted on 2026-09-04; this file is the
//! first consumer that exists because of it.
//!
//! ⇒ So there are **two** regions per press target, deliberately:
//! `dock.left.toolrail` from the shell says *the strip is on screen*, and
//! `rail.<group>.<command>` from here says *this control was drawn inside it
//! and enough of it survived the clip to click*. A build whose handler
//! returned early would keep the first and lose the second.
//!
//! ## The width is not this file's to choose
//!
//! [`egui_shell::dock::rail::WIDTH_PTS`] is a constant and the strip's `Ui` is
//! already clipped to it. Nothing here measures a label. See that module's
//! header on R128 and the fit-zoom feedback loop; the short form is that a
//! rail sized from the word `Signatures` moves the canvas, which re-fits the
//! zoom, which is a loop this project has paid for twice.
//!
//! ## The strip scrolls rather than truncating
//!
//! `RIBBON_SCALING.md`'s third rung. The ladder stops at
//! [`Rung::Cramped`](egui_shell::dock::rail::Rung::Cramped) and does not shed
//! further rows; below that the `ScrollArea` here carries them. A rail that
//! simply cut its last entry off the bottom edge of a short window would be
//! the unreachable-control defect arriving by a different route.

use egui::{Align, Layout, Sense, Vec2};
use egui_shell::commands::{Command, CommandRegistry, ConditionSet, HandlerToken};
use egui_shell::dock::rail::{RailRow, WIDTH_PTS};
use egui_shell::manifest::Rail;

use egui_shell::ribbon::IconRequest;

/// The trace-region prefix every rail control publishes under.
pub const REGION_PREFIX: &str = "rail"; // ui-text-exempt: trace region name, never displayed

/// The region the overflow chevron publishes.
pub const REGION_CHEVRON: &str = "rail.chevron"; // ui-text-exempt: trace region name, never displayed

/// The region name for one control: `rail.<group>.<command id>`.
///
/// The group is in the name because a command may legitimately appear in two
/// groups one day, and because a check that failed on `rail.view.tool_hand`
/// would not say which run of the strip lost it.
#[must_use]
pub fn region(group: &str, id: &str) -> String {
    format!("{REGION_PREFIX}.{group}.{id}")
}

/// Horizontal inset from the strip's edges, per side.
const INSET: f32 = 3.0;

/// Draw the rail and return the handler tokens the operator invoked.
///
/// Called from inside the dock's rail handler, so the `Ui` it is given is
/// already `WIDTH_PTS` wide and already clipped to the strip.
///
/// # Why the tokens come back rather than being dispatched here
///
/// The same borrow rule the tab menu and the tool banner obey: this closure
/// lives across `Dock::show`, which is holding `self.dock` mutably, so it
/// cannot reach the dispatcher. Record and act after `show` returns — which is
/// also what makes the press order well defined.
pub fn show(
    ui: &mut egui::Ui,
    rail: &Rail,
    registry: &CommandRegistry,
    conditions: &ConditionSet,
) -> Vec<HandlerToken> {
    let mut tokens = Vec::new();
    if rail.is_empty() {
        return tokens;
    }

    // ★ The budget is the strip's own available height, read before anything
    // is drawn into it. Reading it *after* a row would make the ladder a
    // function of what the ladder had already decided, which is the shape of
    // every layout feedback loop in this project's RAG.
    let budget = ui.available_height();
    let plan = egui_shell::dock::rail::plan(rail, conditions, budget);

    crate::diag::trace_changed("rail-plan", || {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed.
            "rail-plan rung={:?} rows={} folded={} wants={:.0} budget={:.0}",
            plan.rung,
            plan.rows.len(),
            plan.folded.len(),
            plan.height_pts,
            budget
        )
    });

    egui::ScrollArea::vertical()
        .id_salt("rail-scroll")
        .show(ui, |ui| {
            ui.set_width(WIDTH_PTS - INSET * 2.0);
            ui.spacing_mut().item_spacing = Vec2::new(0.0, 1.0);
            for row in &plan.rows {
                match row {
                    RailRow::Rule => {
                        ui.add_space(4.0);
                        ui.separator();
                        ui.add_space(2.0);
                    }
                    RailRow::Caption(word) => {
                        ui.with_layout(Layout::top_down(Align::Center), |ui| {
                            ui.add(
                                egui::Label::new(egui::RichText::new(word).small().weak())
                                    .truncate(),
                            );
                        });
                    }
                    RailRow::Entry {
                        group,
                        id,
                        with_label,
                        selected,
                        pinned,
                    } => {
                        let Some(command) = registry.get(id) else {
                            // ★ Nothing is drawn for an unregistered id, and
                            // nothing needs to be: `Shell::validate` walks the
                            // rail and refuses the whole manifest at start-up,
                            // so reaching here means the manifest was bypassed.
                            // A placeholder rectangle in permanent chrome would
                            // be worse than the hole.
                            continue;
                        };
                        if let Some(token) = entry(
                            ui,
                            command,
                            group,
                            *with_label,
                            *selected,
                            *pinned,
                            command.is_enabled(conditions),
                        ) {
                            tokens.push(token);
                        }
                    }
                    RailRow::Chevron { folded } => {
                        tokens.extend(chevron(ui, &plan.folded, *folded, registry, conditions));
                    }
                }
            }
        });

    tokens
}

/// One control: a picture, optionally with its word under it.
///
/// ★ Hand-drawn rather than an `egui::Button`, and the reason is the width.
/// A button sizes itself from its content; this row must be exactly the strip
/// wide at every rung, whatever the label says. Allocating the rectangle first
/// and painting into it is the only arrangement in which the label physically
/// cannot influence the geometry — which is the R128 argument made structural
/// rather than promised in a comment.
#[allow(clippy::too_many_arguments, clippy::fn_params_excessive_bools)]
fn entry(
    ui: &mut egui::Ui,
    command: &Command,
    group: &str,
    with_label: bool,
    selected: bool,
    pinned: bool,
    enabled: bool,
) -> Option<HandlerToken> {
    let height = if with_label {
        egui_shell::dock::rail::ROW_LABELLED_PTS
    } else {
        egui_shell::dock::rail::ROW_ICON_ONLY_PTS
    };
    let width = WIDTH_PTS - INSET * 2.0;
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(width, height), Sense::click_and_drag());
    let response = if enabled {
        response
    } else {
        // A disabled row is still allocated and still publishes — R9's second
        // half needs somewhere to hang the explanation, and a control that
        // vanished when it greyed would move every row below it.
        response.on_disabled_hover_text(command.tooltip.clone().unwrap_or_default())
    };

    // ★★ The selected pair comes from the THEME, never from
    // `ui.visuals().selection` — `tools/gates/check-selection-channel.sh`
    // forbids the raw read, and its reason applies exactly here: this row is
    // hand-drawn, so it is one of the sites that would go on painting whatever
    // that channel happened to hold after somebody re-pointed it. It has been
    // re-pointed twice already. `Theme::selected_widget_pair` returns
    // `(selected_plate, accent)` and a shell test pins that it is bit-for-bit
    // what `egui` would have painted.
    let (plate, accent) = egui_shell::theme::Theme::selected_widget_pair(ui.ctx());
    let visuals = ui.style().interact_selectable(&response, selected);
    if selected || response.hovered() {
        ui.painter().rect_filled(
            rect,
            2.0,
            if selected {
                plate
            } else {
                visuals.weak_bg_fill
            },
        );
    }
    if selected {
        // The accent bar down the inner edge — the mockup's
        // `border-left:2px solid var(--accent)`. It is what tells an armed
        // tool from a hovered one when both are tinted.
        ui.painter().rect_filled(
            egui::Rect::from_min_max(rect.min, egui::pos2(rect.left() + 2.0, rect.bottom())),
            0.0,
            accent,
        );
    }

    let tint = if enabled {
        visuals.fg_stroke.color
    } else {
        ui.visuals().weak_text_color()
    };
    let icon_size = 16.0;
    let icon_rect = egui::Rect::from_center_size(
        egui::pos2(
            rect.center().x,
            if with_label {
                rect.top() + icon_size / 2.0 + 4.0
            } else {
                rect.center().y
            },
        ),
        Vec2::splat(icon_size),
    );
    if let Some(key) = &command.icon {
        crate::icons::paint_ribbon_icon(
            ui.painter(),
            &IconRequest {
                key,
                rect: icon_rect,
                tint,
                enabled,
                selected,
            },
        );
    }
    if with_label {
        ui.painter().text(
            egui::pos2(rect.center().x, icon_rect.bottom() + 1.0),
            egui::Align2::CENTER_TOP,
            &command.label,
            egui::TextStyle::Small.resolve(ui.style()),
            tint,
        );
    }

    // ★★ `ui_rect_visible`, never `ui_rect`. See the module header: a rail
    // entry that is laid out but unreachable is precisely the 2026-08-10
    // defect, and this feature is the one that shipped it.
    crate::diag::ui_rect_visible(&region(group, &command.id), rect, ui.clip_rect());

    // ★ The hover sentence is composed in `crate::text::rail`, not here. R1
    // (`tools/gates/check-ui-strings.sh`): every operator-visible string lives
    // in the catalog, and a `format!` that joins a label to a sentence with an
    // em dash IS an operator-visible string — the em dash and the blank line
    // are typography decisions, and they belong beside the words they punctuate.
    let response = response.on_hover_text(crate::text::rail::hover(
        &command.label,
        command.tooltip.as_deref(),
        pinned,
    ));

    (enabled && response.clicked()).then(|| {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("rail-command-invoked id={} group={group}", command.id)
        });
        command.handler
    })
}

/// The overflow chevron, and the menu of everything the strip folded away.
///
/// ★★ It is drawn only when it holds something — a chevron over an empty
/// overflow is the dead control R9 forbids — and it is **never itself
/// folded**: [`egui_shell::dock::rail::build`] appends it after the ladder has
/// run. That is Inkscape failure mode #8 (past about six tabs the overflow
/// button is the thing that gets hidden) refused by construction.
fn chevron(
    ui: &mut egui::Ui,
    folded: &[String],
    count: usize,
    registry: &CommandRegistry,
    conditions: &ConditionSet,
) -> Vec<HandlerToken> {
    let mut tokens = Vec::new();
    let width = WIDTH_PTS - INSET * 2.0;
    let (rect, response) = ui.allocate_exact_size(
        Vec2::new(width, egui_shell::dock::rail::CHEVRON_PTS),
        Sense::click(),
    );
    let visuals = ui.style().interact(&response);
    ui.painter()
        .rect_stroke(rect, 2.0, visuals.bg_stroke, egui::StrokeKind::Inside);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        crate::text::rail::chevron_glyph(count),
        egui::TextStyle::Small.resolve(ui.style()),
        visuals.fg_stroke.color,
    );
    crate::diag::ui_rect_visible(REGION_CHEVRON, rect, ui.clip_rect());

    // The hover names what went, in the order it went, so the operator can see
    // what the strip gave up without opening the menu.
    let names: Vec<&str> = folded
        .iter()
        .filter_map(|id| registry.get(id).map(|c| c.label.as_str()))
        .collect();
    let response = response.on_hover_text(crate::text::rail::chevron_hint(&names));

    egui::Popup::menu(&response).show(|ui| {
        for id in folded {
            let Some(command) = registry.get(id) else {
                continue;
            };
            let enabled = command.is_enabled(conditions);
            let selected = conditions.is_set(&egui_shell::ribbon::band::selected_condition(id));
            let row = ui.add_enabled(
                enabled,
                egui::Button::new(&command.label).selected(selected),
            );
            crate::diag::ui_rect_visible(&region("chevron", id), row.rect, ui.clip_rect());
            if row.clicked() {
                tokens.push(command.handler);
                ui.close();
            }
        }
    });

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Region names are namespaced by group, so two runs of the strip cannot
    /// publish the same name.
    #[test]
    fn a_region_names_its_group_and_its_command() {
        assert_eq!(
            region("navigate", "view.tool_hand"),
            "rail.navigate.view.tool_hand"
        );
        assert!(region("tabs", "view.panel_pages").starts_with(REGION_PREFIX));
    }
}
