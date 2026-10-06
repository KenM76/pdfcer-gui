//! # `dialogs::handsign::picture` — the *Sign here* window's Picture tab
//!
//! Contract: [`PictureTab::show`] offers a file choice and, once a picture
//! reads, the make-white-see-through option when the picture can have it;
//! [`PictureTab::chosen`] is what Place would place. The preview is the
//! engine's own drawing of the picture (`handsign::picture::preview`), so a
//! cleared white shows cleared.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/handsign.md`.

use std::sync::Arc;

use egui::{Painter, TextureHandle, TextureOptions, Ui, Vec2};
use egui_shell::theme::Theme;
use pdfcer_gui_base::handsign::picture::{self, SigPicture};

use super::placing::Ink;
use crate::text::handsign as t;

/// The Choose picture button.
// ui-text-exempt: trace region name, never displayed
pub const REGION_CHOOSE: &str = "handsign.choose-picture";
/// The make-white-see-through option.
// ui-text-exempt: trace region name, never displayed
pub const REGION_CLEAR_WHITE: &str = "handsign.clear-white";

/// The egui texture name the preview is uploaded under.
const TEXTURE: &str = "hand-sign-picture"; // ui-text-exempt: texture id, never displayed

/// A picture that read.
pub(super) struct Chosen {
    /// The picture and the clear-white choice.
    pub sig: SigPicture,
    /// Its displayed proportions.
    pub size: Vec2,
    can_clear: bool,
    /// The engine's drawing of it, and the clear-white choice it was drawn
    /// with.
    texture: Option<(bool, TextureHandle)>,
}

/// The tab's state.
#[derive(Default)]
pub(super) struct PictureTab {
    /// The operator's render settings, which the preview is drawn under.
    render: pdfcer_render::RenderOptions,
    /// The picture chosen, once one reads.
    pub chosen: Option<Chosen>,
    /// Why the last file chosen did not read.
    error: Option<String>,
}

impl PictureTab {
    /// The tab, starting from `sig` when there is a remembered picture.
    pub fn with(sig: Option<SigPicture>, render: pdfcer_render::RenderOptions) -> Self {
        let mut tab = Self {
            render,
            ..Self::default()
        };
        if let Some(sig) = sig {
            tab.take(sig);
        }
        tab
    }

    /// The picture Place would place.
    pub fn signature(&self) -> Option<SigPicture> {
        self.chosen.as_ref().map(|c| c.sig.clone())
    }

    /// Use `sig`, white cleared only where it can be.
    fn take(&mut self, sig: SigPicture) {
        // Asked of the picture as read: one already keyed has no key left to add.
        let raw = SigPicture {
            clear_white: false,
            ..sig.clone()
        };
        match raw.image() {
            Ok(image) => {
                let can_clear = picture::can_clear_white(&image);
                self.chosen = Some(Chosen {
                    size: picture::ink_size(&image),
                    can_clear,
                    sig: SigPicture {
                        clear_white: sig.clear_white && can_clear,
                        ..sig
                    },
                    texture: None,
                });
                self.error = None;
            }
            Err(why) => {
                self.chosen = None;
                self.error = Some(why);
            }
        }
    }

    /// Ask for a file and read it. A new picture has white cleared when it
    /// can be: a scanned signature's paper is what the operator least wants
    /// over the form.
    fn pick(&mut self) {
        let crate::app::files::Picked::Path(path) = crate::app::files::pick_image_source() else {
            return;
        };
        match std::fs::read(&path) {
            Ok(bytes) => self.take(SigPicture {
                bytes: Arc::new(bytes),
                clear_white: true,
            }),
            Err(why) => {
                self.chosen = None;
                self.error = Some(why.to_string());
            }
        }
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            match &self.chosen {
                Some(c) => format!(
                    "hand-sign-picture-chosen read=1 width={:.0} height={:.0} clear_white={} offered={}",
                    c.size.x,
                    c.size.y,
                    u8::from(c.sig.clear_white),
                    u8::from(c.can_clear)
                ),
                None => "hand-sign-picture-chosen read=0".to_owned(),
            }
        });
    }

    /// The choice row, and why a file did not read.
    pub fn show(&mut self, ui: &mut Ui) {
        ui.label(t::picture_intro());
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            let choose = ui.button(t::choose_picture());
            crate::diag::ui_rect_visible(REGION_CHOOSE, choose.rect, ui.clip_rect());
            if choose.clicked() {
                self.pick();
            }
            if let Some(c) = &mut self.chosen {
                let clear = ui
                    .add_enabled(
                        c.can_clear,
                        egui::Checkbox::new(&mut c.sig.clear_white, t::clear_white()),
                    )
                    .on_hover_text(t::clear_white_hover())
                    .on_disabled_hover_text(t::clear_white_unavailable());
                crate::diag::ui_rect_visible(REGION_CLEAR_WHITE, clear.rect, ui.clip_rect());
            }
        });
        if let Some(why) = &self.error {
            let danger = Theme::of(ui.ctx()).palette.danger;
            ui.colored_label(danger, t::picture_unreadable(why));
        }
    }

    /// Draw the picture into `at`, redrawing it through the engine when the
    /// clear-white choice changed.
    pub fn paint(&mut self, ctx: &egui::Context, painter: &Painter, at: Ink) {
        let render = &self.render;
        let Some(c) = &mut self.chosen else {
            return;
        };
        if c.texture
            .as_ref()
            .is_none_or(|(clear, _)| *clear != c.sig.clear_white)
        {
            c.texture = drawn(ctx, &c.sig, render).map(|tex| (c.sig.clear_white, tex));
        }
        if let Some((_, texture)) = &c.texture {
            let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
            painter.image(
                texture.id(),
                at.screen,
                uv,
                // NOT A THEME COLOUR: the identity tint.
                egui::Color32::WHITE,
            );
        }
    }
}

/// The engine's drawing of `sig`, uploaded.
fn drawn(
    ctx: &egui::Context,
    sig: &SigPicture,
    render: &pdfcer_render::RenderOptions,
) -> Option<TextureHandle> {
    let image = sig.image().ok()?;
    let (size, rgba) = picture::preview(&image, render).ok()?;
    #[allow(
        clippy::cast_possible_truncation,
        reason = "the preview is at most a few hundred pixels a side" // ui-text-exempt: a lint justification
    )]
    let (w, h) = (size[0] as u32, size[1] as u32);
    crate::render::pressure::record_other(
        ctx,
        crate::render::pressure::Surface::SignaturePicture,
        w,
        h,
    );
    let colour = egui::ColorImage::from_rgba_premultiplied(size, &rgba);
    Some(ctx.load_texture(TEXTURE, colour, TextureOptions::LINEAR))
}
