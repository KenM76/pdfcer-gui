//! The Sign window's *Picture in the box*: a PNG or JPEG the engine draws in
//! a visible signature beside its text.
//!
//! Regions: `sign-picture` (the chooser), `sign-picture-name` and
//! `sign-picture-remove` once one is chosen. Each pick traces
//! `sign-picture chosen=… width=… height=…`.

use egui_shell::theme::Theme;

use crate::app::files::Picked;
use crate::dialogs::filename::file_name_of;
use crate::sign::picture::AppearancePicture;
use crate::text::sign_picture as t;

const REGION_CHOOSE: &str = "sign-picture"; // ui-text-exempt: trace region name, never displayed
const REGION_NAME: &str = "sign-picture-name"; // ui-text-exempt: trace region name, never displayed
const REGION_REMOVE: &str = "sign-picture-remove"; // ui-text-exempt: trace region name, never displayed

/// The chosen picture and the requests the draw raised.
#[derive(Default)]
pub(super) struct Picture {
    chosen: Option<(String, AppearancePicture)>,
    error: Option<String>,
    pick_requested: bool,
    remove_requested: bool,
}

impl Picture {
    /// Draw the choice; nothing while the signature draws no box.
    pub(super) fn show(&mut self, ui: &mut egui::Ui, theme: &Theme, visible: bool) {
        if !visible {
            return;
        }
        ui.add_space(8.0);
        ui.label(t::heading());
        ui.horizontal_wrapped(|ui| {
            let choose = ui.button(t::choose());
            crate::diag::ui_rect(REGION_CHOOSE, choose.rect);
            self.pick_requested |= choose.clicked();
            if let Some((name, picture)) = &self.chosen {
                let (w, h) = picture.size();
                let named = ui.label(t::chosen(name, w, h));
                crate::diag::ui_rect(REGION_NAME, named.rect);
                let remove = ui.button(t::remove());
                crate::diag::ui_rect(REGION_REMOVE, remove.rect);
                self.remove_requested |= remove.clicked();
            } else {
                ui.label(egui::RichText::new(t::none()).color(theme.palette.text_muted));
            }
        });
        if let Some(why) = &self.error {
            ui.label(egui::RichText::new(t::unreadable(why)).color(theme.palette.danger));
        }
    }

    /// The side effects the draw asked for, run after the window's closure.
    pub(super) fn after(&mut self) {
        if std::mem::take(&mut self.remove_requested) {
            self.chosen = None;
            self.error = None;
            self.trace();
        }
        if !std::mem::take(&mut self.pick_requested) {
            return;
        }
        let Picked::Path(path) = crate::app::files::pick_image_source(t::picker_title()) else {
            return;
        };
        match AppearancePicture::read(&path) {
            Ok(picture) => {
                self.chosen = Some((file_name_of(&path), picture));
                self.error = None;
            }
            Err(why) => self.error = Some(why),
        }
        self.trace();
    }

    /// The picture a signature drawing a box carries.
    pub(super) fn for_request(&self, visible: bool) -> Option<AppearancePicture> {
        self.chosen
            .as_ref()
            .filter(|_| visible)
            .map(|(_, picture)| picture.clone())
    }

    fn trace(&self) {
        let (w, h) = self.chosen.as_ref().map_or((0, 0), |(_, p)| p.size());
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "sign-picture chosen={} width={w} height={h} error={}",
                u8::from(self.chosen.is_some()),
                u8::from(self.error.is_some())
            )
        });
    }
}
