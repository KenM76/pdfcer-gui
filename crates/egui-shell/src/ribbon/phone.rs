//! # The phone layout — `MOBILE_DESIGN.md` §3
//!
//! The same manifest and registry the ribbon draws, laid out for a window
//! narrower than [`PHONE_MAX_WIDTH`] points. Nothing here is a second command
//! surface: every control is drawn by the ribbon's own `render_command`, so a
//! command's enablement, selected plate, glyph, tooltip and click report are
//! the desktop's, and an unregistered command draws nothing (R8, R9).
//!
//! Three regions, added in this order because `egui` panels carve their room
//! from what is left:
//!
//! | Region | Contents |
//! |---|---|
//! | app bar (top) | the title and its subtitle, the QAT glyph-only, an overflow button holding the mode's first tab, then the mode selector on its own row |
//! | bottom bar | the mode's [`Mode::bottom_bar`](crate::manifest::Mode::bottom_bar), else the first group of its second tab, then *All tools* |
//! | tool sheet | over the bottom bar while open: the mode's tabs as chips, each group a heading, each command a cell in a four-column grid |
//!
//! The application adds its own status line and canvas after this returns,
//! exactly as it does after [`Ribbon::render`](super::Ribbon::render).
//!
//! Like the ribbon, a mode or tab chosen here lands on the NEXT frame.

use egui::{RichText, vec2};

use crate::commands::{Command, CommandRegistry, HandlerToken};
use crate::manifest::{Item, ItemSize, Shell, Tab};

use super::ctx::Ctx;
use super::{Ribbon, RibbonState, control, mode_selector, sizing, tabs};

/// Below this content width, in points, the application uses the phone
/// layout. Material 3's compact/medium window-size class boundary; `egui`
/// points equal Android density-independent pixels.
pub const PHONE_MAX_WIDTH: f32 = 600.0;

/// The commands a bottom bar holds before *All tools*.
const BOTTOM_BAR_MAX: usize = 4;

/// Height of a bottom-bar or tool-sheet cell, in points. Above Android's
/// 48 dp minimum touch target with room for a one-line label.
const CELL_HEIGHT: f32 = 56.0;

/// Columns in the tool sheet's grid.
const SHEET_COLUMNS: usize = 4;

/// The share of the window the open tool sheet may take at most.
const SHEET_MAX_SHARE: f32 = 0.55;

/// Whether `ctx`'s window is narrow enough for the phone layout.
#[must_use]
pub fn is_phone(ctx: &egui::Context) -> bool {
    ctx.content_rect().width() < PHONE_MAX_WIDTH
}

/// What the phone layout remembers between frames.
#[derive(Debug, Clone, Default)]
pub struct PhoneState {
    /// Whether the tool sheet is open.
    pub sheet_open: bool,
    /// The tab the tool sheet shows; the mode's first tab when unset or gone.
    sheet_tab: Option<String>,
}

/// The words the phone layout draws, supplied by the application so the
/// shell carries no language of its own.
#[derive(Debug, Clone, Copy, Default)]
pub struct PhoneText<'t> {
    /// The document's name, or the application's when none is open.
    pub title: &'t str,
    /// The smaller line under it, e.g. the page; empty draws nothing.
    pub subtitle: &'t str,
    /// The overflow button's accessible name and tooltip. The button itself
    /// draws three dots, so it needs no glyph from the font.
    pub more: &'t str,
    /// The bottom bar's last cell, which opens the tool sheet.
    pub all_tools: &'t str,
    /// The icon key for that cell, painted through the application's icon
    /// painter like a command's; `None` draws the label alone.
    pub all_tools_icon: Option<&'t str>,
}

impl<'a> Ribbon<'a> {
    /// Draw the phone layout's app bar, bottom bar and tool sheet, and report
    /// the commands the operator invoked.
    pub fn render_phone(
        self,
        ui: &mut egui::Ui,
        shell: &Shell,
        registry: &'a CommandRegistry,
        state: &mut RibbonState,
        phone: &mut PhoneState,
        text: PhoneText<'_>,
    ) -> Vec<HandlerToken> {
        let mut ctx = self.into_ctx(ui.ctx(), registry, state.base_id);
        let mode = state
            .mode
            .clone()
            .or_else(|| shell.modes().first().map(|m| m.id.clone()));
        let visible = tabs::visible_tabs(shell, mode.as_deref(), ctx.conditions);

        let chosen_mode = app_bar(ui, &mut ctx, shell, &visible, mode.as_deref(), text);
        bottom_bar(ui, &mut ctx, shell, &visible, mode.as_deref(), phone, text);
        if phone.sheet_open {
            tool_sheet(ui, &mut ctx, &visible, phone);
        }

        if let Some(id) = chosen_mode {
            state.mode = Some(id);
            phone.sheet_tab = None;
        }
        crate::verify::event("phone-frame")
            .kv("mode", mode.as_deref().unwrap_or(""))
            .kv("sheet", u8::from(phone.sheet_open))
            .kv("invoked", ctx.invoked.len())
            .emit();
        ctx.invoked
    }
}

/// The top bar. Returns the mode the operator chose, if any.
fn app_bar(
    ui: &mut egui::Ui,
    ctx: &mut Ctx<'_>,
    shell: &Shell,
    visible: &[&Tab],
    mode: Option<&str>,
    text: PhoneText<'_>,
) -> Option<String> {
    let mut chosen = None;
    let title_pts = ctx.theme.metrics.icon_pts;
    egui::Panel::top("egui-shell/phone/app-bar").show(ui, |ui| {
        egui::Sides::new().shrink_left().show(
            ui,
            |ui| {
                ui.vertical(|ui| {
                    // `text_color`, not `strong`: the theme's strong colour is the pressed
                    // control's ink, which is near-white on the light chrome.
                    let ink = ui.visuals().text_color();
                    ui.add(
                        egui::Label::new(RichText::new(text.title).color(ink).size(title_pts))
                            .truncate(),
                    );
                    if !text.subtitle.is_empty() {
                        ui.add(egui::Label::new(RichText::new(text.subtitle).small()).truncate());
                    }
                });
            },
            |ui| {
                // Right to left: the overflow button outermost, then the QAT in reverse.
                if let Some(first) = visible.first() {
                    overflow_menu(ui, ctx, first, text.more);
                }
                let qat: Vec<String> = shell
                    .qat
                    .as_ref()
                    .map(|q| q.ids().to_vec())
                    .unwrap_or_default();
                for id in qat.iter().rev() {
                    control::render_command(ui, ctx, id, ItemSize::Small, 0.0);
                }
            },
        );
        ui.vertical_centered(|ui| {
            chosen = mode_selector::render(ui, ctx, shell.modes(), mode);
        });
        ctx.reporter
            .report(ui.min_rect(), || "phone.app-bar".to_owned()); // ui-text-exempt: region id
    });
    chosen
}

/// The overflow button: every item of `tab`, one per row with its label.
/// It paints three vertical dots, Android's overflow mark, rather than a
/// font glyph that a phone's fonts may lack.
fn overflow_menu(ui: &mut egui::Ui, ctx: &mut Ctx<'_>, tab: &Tab, more: &str) {
    let size = ctx.theme.metrics.icon_pts;
    let slot = ctx.id("phone-overflow", "");
    let response = ui.menu_button(egui::Atom::custom(slot, vec2(size, size)), |ui| {
        for group in tab.groups() {
            for item in group.items() {
                control::render_item_at(ui, ctx, &tab.id, &group.id, item, ItemSize::Medium, 0.0);
            }
        }
    });
    let button = &response.response;
    let ink = ui.style().interact(button).fg_stroke.color;
    let centre = button.rect.center();
    let step = size / 3.5;
    for k in [-1.0, 0.0, 1.0] {
        ui.painter()
            .circle_filled(centre + vec2(0.0, k * step), size / 12.0, ink);
    }
    let name = more.to_owned();
    button.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, name.clone()));
    let button = button.clone().on_hover_text(more);
    ctx.reporter
        .report(button.rect, || "phone.overflow".to_owned()); // ui-text-exempt: region id
}

/// The id of the synthetic command the All tools cell is drawn from.
const ALL_TOOLS_ID: &str = "egui-shell.phone.all_tools"; // ui-text-exempt: internal id

/// The commands the bottom bar holds for `mode`: the manifest's list, or the
/// first group of the mode's second tab (its first is behind the overflow button).
fn bottom_bar_ids(shell: &Shell, visible: &[&Tab], mode: Option<&str>) -> Vec<String> {
    let listed = mode
        .and_then(|id| shell.modes().iter().find(|m| m.id == id))
        .map(|m| m.bottom_bar().to_vec())
        .unwrap_or_default();
    if !listed.is_empty() {
        return listed;
    }
    visible
        .get(1)
        .or_else(|| visible.first())
        .and_then(|tab| tab.groups().first())
        .map(|group| {
            group
                .items()
                .iter()
                .filter_map(|item| match item {
                    Item::Command { id, .. } => Some(id.clone()),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default()
}

/// The bottom bar: up to [`BOTTOM_BAR_MAX`] registered commands, then *All tools*.
fn bottom_bar(
    ui: &mut egui::Ui,
    ctx: &mut Ctx<'_>,
    shell: &Shell,
    visible: &[&Tab],
    mode: Option<&str>,
    phone: &mut PhoneState,
    text: PhoneText<'_>,
) {
    let ids: Vec<String> = bottom_bar_ids(shell, visible, mode)
        .into_iter()
        .filter(|id| ctx.registry.get(id).is_some())
        .take(BOTTOM_BAR_MAX)
        .collect();
    egui::Panel::bottom("egui-shell/phone/bottom-bar")
        .exact_size(CELL_HEIGHT + ui.spacing().item_spacing.y * 2.0)
        .show(ui, |ui| {
            let columns = ids.len() + 1;
            let width = ui.available_width() / columns as f32;
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                for id in &ids {
                    ui.allocate_ui(vec2(width, CELL_HEIGHT), |ui| {
                        ui.set_width(width);
                        ui.vertical_centered(|ui| {
                            control::render_command(ui, ctx, id, ItemSize::Large, CELL_HEIGHT);
                        });
                    });
                }
                ui.allocate_ui(vec2(width, CELL_HEIGHT), |ui| {
                    ui.set_width(width);
                    ui.vertical_centered(|ui| {
                        // Drawn as a Large command cell, so it matches its
                        // neighbours; the command is never registered or invoked.
                        let mut cell =
                            Command::new(ALL_TOOLS_ID, text.all_tools, HandlerToken::new(0));
                        cell.icon = text.all_tools_icon.map(str::to_owned);
                        let response = sizing::render_large(
                            ui,
                            ctx,
                            &cell,
                            phone.sheet_open,
                            true,
                            CELL_HEIGHT,
                        );
                        ctx.reporter
                            .report(response.rect, || "phone.all-tools".to_owned()); // ui-text-exempt: region id
                        if response.clicked() {
                            phone.sheet_open = !phone.sheet_open;
                        }
                    });
                });
            });
        });
}

/// The tool sheet: the mode's tabs as chips over a grid of the chosen tab.
fn tool_sheet(ui: &mut egui::Ui, ctx: &mut Ctx<'_>, visible: &[&Tab], phone: &mut PhoneState) {
    let Some(tab) = tabs::resolve_active(visible, phone.sheet_tab.as_deref()) else {
        return;
    };
    let max_height = ui.ctx().content_rect().height() * SHEET_MAX_SHARE;
    egui::Panel::bottom("egui-shell/phone/tool-sheet")
        .max_size(max_height)
        .default_size(max_height)
        .resizable(false)
        .show(ui, |ui| {
            egui::ScrollArea::horizontal()
                .id_salt("egui-shell/phone/sheet-tabs")
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        for candidate in visible {
                            let on = candidate.id == tab.id;
                            if ui
                                .selectable_label(on, tabs::tab_label(candidate))
                                .clicked()
                            {
                                phone.sheet_tab = Some(candidate.id.clone());
                            }
                        }
                    });
                });
            ui.separator();
            egui::ScrollArea::vertical()
                .id_salt("egui-shell/phone/sheet-body")
                .show(ui, |ui| sheet_groups(ui, ctx, tab));
            ctx.reporter
                .report(ui.min_rect(), || "phone.tool-sheet".to_owned()); // ui-text-exempt: region id
        });
}

/// Each group of `tab`: its caption, then its commands four to a row.
fn sheet_groups(ui: &mut egui::Ui, ctx: &mut Ctx<'_>, tab: &Tab) {
    let width = ui.available_width() / SHEET_COLUMNS as f32;
    for group in tab.groups() {
        let items: Vec<&Item> = group
            .items()
            .iter()
            .filter(|item| !matches!(item, Item::Separator))
            .collect();
        if items.is_empty() {
            continue;
        }
        if let Some(caption) = group.caption.as_deref() {
            ui.label(
                RichText::new(caption)
                    .small()
                    .strong()
                    .color(ui.visuals().text_color()),
            );
        }
        for row in items.chunks(SHEET_COLUMNS) {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                for item in row {
                    ui.allocate_ui(vec2(width, CELL_HEIGHT), |ui| {
                        ui.set_width(width);
                        ui.vertical_centered(|ui| {
                            control::render_item_at(
                                ui,
                                ctx,
                                &tab.id,
                                &group.id,
                                item,
                                ItemSize::Large,
                                CELL_HEIGHT,
                            );
                        });
                    });
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{Group, Mode};

    fn shell() -> Shell {
        Shell::new()
            .with_mode(Mode::new("read", "Read", ["file", "view"]).with_bottom_bar(["view.find"]))
            .with_mode(Mode::new("edit", "Edit", ["file", "view"]))
            .with_tab(
                Tab::new("file", "File").with_groups([
                    Group::new("open", "Open").with_items([Item::command("file.open")])
                ]),
            )
            .with_tab(Tab::new("view", "View").with_groups([
                Group::new("zoom", "Zoom").with_items([
                    Item::command("view.zoom_in"),
                    Item::command("view.zoom_out"),
                ]),
            ]))
    }

    #[test]
    fn a_mode_listing_a_bottom_bar_gets_exactly_that_list() {
        let shell = shell();
        let visible: Vec<&Tab> = shell.tabs().iter().collect();
        assert_eq!(
            bottom_bar_ids(&shell, &visible, Some("read")),
            ["view.find"]
        );
    }

    #[test]
    fn a_mode_without_one_falls_back_to_its_second_tabs_first_group() {
        let shell = shell();
        let visible: Vec<&Tab> = shell.tabs().iter().collect();
        assert_eq!(
            bottom_bar_ids(&shell, &visible, Some("edit")),
            ["view.zoom_in", "view.zoom_out"]
        );
    }

    #[test]
    fn a_single_tab_mode_falls_back_to_its_only_tab() {
        let shell = shell();
        let visible: Vec<&Tab> = shell.tabs().iter().take(1).collect();
        assert_eq!(
            bottom_bar_ids(&shell, &visible, Some("edit")),
            ["file.open"]
        );
    }

    #[test]
    fn the_breakpoint_is_materials_compact_width() {
        let ctx = egui::Context::default();
        let mut narrow = false;
        let mut wide = false;
        for (width, out) in [(400.0, &mut narrow), (900.0, &mut wide)] {
            let input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    vec2(width, 800.0),
                )),
                ..Default::default()
            };
            let _ = ctx.run_ui(input, |ui| *out = is_phone(ui.ctx()));
        }
        assert!(narrow && !wide);
    }
}
