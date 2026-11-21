//! # `dialogs::cardswitches` — every graphics-card switch, beside what it did
//!
//! One window listing each path that can draw on the graphics card, its
//! switch where it has one, and its latest outcome on this machine from
//! [`pdfcer_gui_base::cardstatus`]. It is a second home for switches that
//! keep their first ones, and it edits the same values:
//!
//! - *Pages in the page view* is `Prefs::card::draw_pages`, saved at once.
//!   An open Settings window adopts the change into its draft
//!   (`app::settings_window`), so its Save cannot put the old value back.
//! - The 3D picture and its parts are the viewer's `CardParts`: a change is
//!   merged into the saved `model3d_card_*` defaults and applied to the open
//!   viewer, if any. Without one, the switches show the saved defaults.
//!
//! Bounce light and Measure light have no switch; they follow the picture's.
//!
//! Regions: `dialog:card-switches`, `cards.switch.<path>` and `cards.status.<path>`, `<path>` being
//! `cardstatus::Path::token`. Traces: `card-switches-shown rows=` once per
//! opening, and `card-switch path= on=` per change.

use egui::Ui;
use pdfcer_gui_base::cardstatus::{self, Outcome, Path};

use crate::app::actions::Action;
use crate::app::prefs::Prefs;
#[cfg(feature = "3d")]
use crate::dialogs::model3d::{self, CardParts};
use crate::text::cardstatus as t;
#[cfg(feature = "3d")]
use crate::text::model3dlook as k;

/// The open 3D viewer the 3D rows also apply to.
#[cfg(feature = "3d")]
pub(crate) type Viewer = crate::dialogs::model3d::ModelView;
/// No 3D viewer in a build without it.
#[cfg(not(feature = "3d"))]
pub(crate) enum Viewer {}

/// The window's body region.
pub const REGION_BODY: &str = "dialog:card-switches"; // ui-text-exempt: trace region name, never displayed
/// A switch's region, after this prefix its path's token.
pub const REGION_SWITCH: &str = "cards.switch."; // ui-text-exempt: trace region name, never displayed
/// A status cell's region, after this prefix its path's token.
pub const REGION_STATUS: &str = "cards.status."; // ui-text-exempt: trace region name, never displayed

/// The open window.
#[derive(Debug, Default)]
pub struct CardSwitchesDialog {
    told: bool,
}

/// What the rows read and write this frame.
struct Rows<'a> {
    prefs: &'a mut Prefs,
    #[cfg(feature = "3d")]
    card: CardParts,
    #[cfg(feature = "3d")]
    forced: bool,
    status: [Outcome; cardstatus::ALL.len()],
}

impl CardSwitchesDialog {
    /// Draw a frame; `false` once closed.
    pub(crate) fn show(
        &mut self,
        ctx: &egui::Context,
        prefs: &mut Prefs,
        viewer: Option<&mut Viewer>,
        actions: &mut Vec<Action>,
    ) -> bool {
        #[cfg(feature = "3d")]
        let card = viewer
            .as_ref()
            .map_or_else(|| model3d::saved_card(&prefs.model_view), |v| v.card());
        let mut rows = Rows {
            prefs,
            #[cfg(feature = "3d")]
            card,
            #[cfg(feature = "3d")]
            forced: model3d::card_forced_by_environment(),
            status: cardstatus::snapshot(),
        };
        let (frame, ()) = crate::dialogs::host::Host::new(
            "card-switches", // ui-text-exempt: a viewport key, never displayed.
            t::title(),
            egui::vec2(620.0, 480.0),
            egui::vec2(440.0, 260.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| body(ui, &mut rows));
        });
        if !self.told {
            self.told = true;
            crate::diag::trace(|| format!("card-switches-shown rows={}", cardstatus::ALL.len())); // ui-text-exempt: diagnostic trace, never displayed
        }
        #[cfg(feature = "3d")]
        if rows.card != card {
            let keys = model3d::card_keys(&rows.card);
            actions.push(Action::Pref(
                crate::app::actions::prefs::PrefAction::ModelViewMerge(keys),
            ));
            if let Some(v) = viewer {
                v.set_card(rows.card);
            }
        }
        #[cfg(not(feature = "3d"))]
        let _ = (viewer, actions);
        !frame.closed
    }
}

fn body(ui: &mut Ui, rows: &mut Rows<'_>) {
    let side = ui.ctx().input(|i| i.max_texture_side);
    ui.label(t::renderer(side));
    ui.label(t::memory(cardstatus::out_of_memory_seen()));
    ui.separator();
    egui::Grid::new("card-switches")
        .num_columns(2)
        .striped(true)
        .spacing([18.0, 6.0])
        .show(ui, |ui| {
            let [a, b] = t::columns();
            ui.strong(a);
            ui.strong(b);
            ui.end_row();
            pages_row(ui, rows);
            #[cfg(feature = "3d")]
            model_rows(ui, rows);
            #[cfg(feature = "3d")]
            for (path, label) in [
                (Path::Bounce, t::bounce()),
                (Path::Meter, t::meter()),
                (Path::Map, t::light_map()),
            ] {
                let r = ui.label(label).on_hover_text(t::follows_picture());
                region(REGION_SWITCH, path, &r, ui);
                status(ui, rows, path);
            }
        });
    ui.separator();
    ui.small(t::processor_only());
}

fn pages_row(ui: &mut Ui, rows: &mut Rows<'_>) {
    let r = ui
        .checkbox(&mut rows.prefs.card.draw_pages, t::pages())
        .on_hover_text(t::pages_tooltip());
    region(REGION_SWITCH, Path::Pages, &r, ui);
    if r.changed() {
        let on = rows.prefs.card.draw_pages;
        if let Err(error) = rows.prefs.save() {
            crate::diag::trace(|| format!("prefs-save-failed reason={error}")); // ui-text-exempt: diagnostic trace
        }
        changed(Path::Pages, on);
    }
    status(ui, rows, Path::Pages);
}

#[cfg(feature = "3d")]
fn model_rows(ui: &mut Ui, rows: &mut Rows<'_>) {
    let r = ui
        .add_enabled(
            !rows.forced,
            egui::Checkbox::new(&mut rows.card.picture, t::picture()),
        )
        .on_hover_text(t::model_tooltip())
        .on_disabled_hover_text(k::CARD_FORCED_OFF_TOOLTIP);
    region(REGION_SWITCH, Path::Picture, &r, ui);
    if r.changed() {
        changed(Path::Picture, rows.card.picture);
    }
    status(ui, rows, Path::Picture);
    let live = rows.card.picture && !rows.forced;
    for (i, part) in k::CardPart::ALL.into_iter().enumerate() {
        let path = Path::Part(i);
        let r = ui
            .horizontal(|ui| {
                ui.add_space(18.0);
                ui.add_enabled(
                    live,
                    egui::Checkbox::new(&mut rows.card.parts[i], k::card_part(part)),
                )
                .on_hover_text(t::model_tooltip())
                .on_disabled_hover_text(t::part_greyed())
            })
            .inner;
        region(REGION_SWITCH, path, &r, ui);
        if r.changed() {
            changed(path, rows.card.parts[i]);
        }
        status(ui, rows, path);
    }
}

fn status(ui: &mut Ui, rows: &Rows<'_>, path: Path) {
    let r = ui.label(t::outcome(&rows.status[path.index()]));
    region(REGION_STATUS, path, &r, ui);
    ui.end_row();
}

fn region(prefix: &str, path: Path, r: &egui::Response, ui: &Ui) {
    let name = format!("{prefix}{}", path.token());
    crate::diag::ui_rect_visible(&name, r.rect, ui.clip_rect());
}

fn changed(path: Path, on: bool) {
    let p = path.token();
    crate::diag::trace(|| format!("card-switch path={p} on={on}")); // ui-text-exempt: diagnostic trace, never displayed
}
