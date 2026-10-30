//! # `dialogs::xfa_preview` — a dynamic XFA form's own pages
//!
//! The dispatch target for `view.form_preview`. A dynamic form's PDF pages are
//! a stand-in; its real pages exist only as `pdfcer_core::xfa::layout_template`
//! output. This window lays the form out and rasterises every computed page
//! with `pdfcer_render::xfa::render_xfa_page` on a worker thread, then lists
//! every `LayoutWarning` and `PaintNote` beside the pages: what the preview
//! leaves out is disclosed here, never marked on the pictures. Nothing is
//! written to the file, and the canvas keeps showing the file's own pages.

use std::sync::{Arc, Mutex};

use egui_shell::theme::Theme;
use pdfcer_core::edit::EditSession;
use pdfcer_core::xfa::{LayoutOptions, PaintNoteKind, PaintOptions};

use crate::text::xfapreview as t;

/// Named region: the whole dialog body.
const REGION_BODY: &str = "dialog:form-preview"; // ui-text-exempt: trace region name, never displayed

/// Device pixels per point the pages are drawn at.
const SCALE: f32 = 1.5;

/// Pages drawn at most; the rest are counted in the trace and the heading.
const MAX_PAGES: usize = 50;

/// One drawn page, ready to upload.
struct PagePixels {
    size: [usize; 2],
    rgba: Vec<u8>,
}

/// The worker's result: the pages and every sentence of disclosure.
struct Drawn {
    pages: Vec<PagePixels>,
    total_pages: usize,
    notes: Vec<String>,
    warnings: Vec<String>,
}

type Slot = Arc<Mutex<Option<Result<Drawn, String>>>>;

/// What the window shows once the worker has answered.
enum Shown {
    Pages {
        textures: Vec<egui::TextureHandle>,
        total_pages: usize,
        notes: Vec<String>,
        warnings: Vec<String>,
    },
    Failed(String),
}

/// The Form-preview dialog. Its existence is its "open" state.
pub struct XfaPreviewDialog {
    slot: Slot,
    shown: Option<Shown>,
    close_requested: bool,
}

impl std::fmt::Debug for XfaPreviewDialog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("XfaPreviewDialog")
            .field("ready", &self.shown.is_some())
            .finish_non_exhaustive()
    }
}

impl XfaPreviewDialog {
    /// Start drawing `session`'s form on a worker thread.
    pub(super) fn open(
        ctx: &egui::Context,
        session: Arc<EditSession>,
        options: pdfcer_render::RenderOptions,
    ) -> Self {
        let slot: Slot = Arc::default();
        let out = Arc::clone(&slot);
        let repaint = ctx.clone();
        let spawned = std::thread::Builder::new()
            .name("form-preview".into())
            .spawn(move || {
                let drawn = draw(&session, &options);
                if let Ok(mut s) = out.lock() {
                    *s = Some(drawn);
                }
                repaint.request_repaint();
            });
        let shown = spawned
            .err()
            .map(|e| Shown::Failed(t::failed(&e.to_string())));
        Self {
            slot,
            shown,
            close_requested: false,
        }
    }

    /// Draw one frame. Returns `false` when it should close.
    pub(super) fn show(&mut self, ctx: &egui::Context) -> bool {
        self.collect(ctx);
        let (frame, ()) = crate::dialogs::host::Host::new(
            "form-preview", // ui-text-exempt: a viewport key, never displayed.
            t::title(),
            egui::vec2(720.0, 820.0),
            egui::vec2(420.0, 320.0),
        )
        .show(ctx, |ui| self.body(ui));
        !frame.closed && !std::mem::take(&mut self.close_requested)
    }

    /// Upload the worker's pages once they arrive.
    fn collect(&mut self, ctx: &egui::Context) {
        if self.shown.is_some() {
            return;
        }
        let Some(done) = self.slot.lock().ok().and_then(|mut s| s.take()) else {
            return;
        };
        self.shown = Some(match done {
            Ok(d) => {
                let textures: Vec<_> = d
                    .pages
                    .iter()
                    .enumerate()
                    .map(|(i, p)| {
                        let image = egui::ColorImage::from_rgba_premultiplied(p.size, &p.rgba);
                        crate::render::pressure::record_other(
                            ctx,
                            crate::render::pressure::Surface::FormPreview,
                            p.size[0] as u32,
                            p.size[1] as u32,
                        );
                        ctx.load_texture(
                            format!("form-preview-{i}"), // ui-text-exempt: texture name, never displayed
                            image,
                            egui::TextureOptions::LINEAR,
                        )
                    })
                    .collect();
                let (pages, notes, warnings) = (d.total_pages, d.notes.len(), d.warnings.len());
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    format!("form-preview pages={pages} notes={notes} warnings={warnings}")
                });
                Shown::Pages {
                    textures,
                    total_pages: d.total_pages,
                    notes: d.notes,
                    warnings: d.warnings,
                }
            }
            Err(why) => {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    format!("form-preview failed why={why:?}")
                });
                Shown::Failed(why)
            }
        });
    }

    /// Everything inside the window.
    fn body(&mut self, ui: &mut egui::Ui) {
        let theme = Theme::of(ui.ctx());
        crate::diag::ui_rect(REGION_BODY, ui.max_rect());
        ui.label(egui::RichText::new(t::caveat()).color(theme.palette.text_muted));
        ui.add_space(6.0);
        const FOOTER_RESERVE: f32 = 40.0;
        const LIST_FLOOR: f32 = 48.0;
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .max_height((ui.available_height() - FOOTER_RESERVE).max(LIST_FLOOR))
            .show(ui, |ui| match &self.shown {
                None => {
                    ui.horizontal(|ui| {
                        ui.add(egui::Spinner::new().color(theme.palette.text));
                        ui.label(t::working());
                    });
                }
                Some(Shown::Failed(why)) => {
                    ui.label(why.as_str());
                }
                Some(Shown::Pages {
                    textures,
                    total_pages,
                    notes,
                    warnings,
                }) => {
                    disclosures(ui, notes, warnings);
                    for (i, texture) in textures.iter().enumerate() {
                        ui.add_space(8.0);
                        ui.label(t::page_heading(i + 1, *total_pages));
                        let size = texture.size_vec2() / SCALE;
                        let width = ui.available_width().min(size.x);
                        let fitted = egui::vec2(width, size.y * width / size.x);
                        ui.image((texture.id(), fitted));
                    }
                }
            });
        ui.separator();
        if ui.button(t::close()).clicked() {
            self.close_requested = true;
        }
    }
}

/// The list of what the preview leaves out, above the pages.
fn disclosures(ui: &mut egui::Ui, notes: &[String], warnings: &[String]) {
    ui.label(egui::RichText::new(t::notes_heading()).heading());
    if notes.is_empty() && warnings.is_empty() {
        ui.label(t::notes_clean());
    }
    for line in warnings.iter().chain(notes) {
        ui.label(line.as_str());
    }
}

/// Lay out and draw every computed page with the operator's render settings.
/// Runs on the worker.
fn draw(session: &EditSession, options: &pdfcer_render::RenderOptions) -> Result<Drawn, String> {
    let view = session.view();
    let xdp = pdfcer_core::xfa::read_xdp(&view)
        .map_err(|e| t::failed(&e.to_string()))?
        .ok_or_else(|| t::no_template().to_owned())?;
    let template = pdfcer_core::xfa::read_template(&xdp)
        .map_err(|e| t::failed(&e.to_string()))?
        .ok_or_else(|| t::no_template().to_owned())?;
    let layout = pdfcer_core::xfa::layout_template(&template, &LayoutOptions::new())
        .map_err(|e| t::failed(&e.to_string()))?;
    let paint = PaintOptions::new();
    let images = pdfcer_core::xfa::XfaImages::from_view(&view);
    let mut pages = Vec::new();
    let mut counts: Vec<(PaintNoteKind, usize)> = Vec::new();
    for page in layout.pages.iter().take(MAX_PAGES) {
        let r = pdfcer_render::xfa::render_xfa_page_with_images(
            &view, &template, page, &paint, &images, SCALE, options,
        )
        .map_err(|e| t::failed(&e.to_string()))?;
        for note in &r.notes {
            match counts.iter_mut().find(|(k, _)| *k == note.kind) {
                Some((_, n)) => *n += 1,
                None => counts.push((note.kind, 1)),
            }
        }
        let pixmap = &r.page.pixmap;
        pages.push(PagePixels {
            size: [pixmap.width() as usize, pixmap.height() as usize],
            rgba: pixmap.data().to_vec(),
        });
    }
    let mut warnings: Vec<String> = Vec::new();
    for w in &layout.warnings {
        let line = t::layout_warning(w).to_owned();
        if !warnings.contains(&line) {
            warnings.push(line);
        }
    }
    Ok(Drawn {
        pages,
        total_pages: layout.pages.len(),
        notes: counts
            .into_iter()
            .map(|(k, n)| t::paint_note(k, n))
            .collect(),
        warnings,
    })
}
