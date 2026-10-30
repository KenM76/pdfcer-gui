//! # `dialogs::xfa_preview` — a dynamic XFA form's own pages
//!
//! The dispatch target for `view.form_preview`. A dynamic form's PDF pages are
//! a stand-in; its real pages exist only as `pdfcer_core::xfa::layout_template`
//! output. This window merges the form's data into its template
//! (`pdfcer_core::xfa::merge_data`), lays it out and rasterises every computed page
//! with `pdfcer_render::xfa::render_xfa_page` on a worker thread, then lists
//! the values filled in and every `MergeWarning`, `LayoutWarning` and
//! `PaintNote` beside the pages: what the preview
//! leaves out is disclosed here, never marked on the pictures. Nothing is
//! written to the file, and the canvas keeps showing the file's own pages.
//!
//! The "Reading choices" are the merge and paint options where the XFA
//! standard leaves the answer open; changing one redraws on a new worker while
//! the previous pages stay shown.

use std::sync::{Arc, Mutex};

use egui_shell::theme::Theme;
use pdfcer_core::edit::EditSession;
use pdfcer_core::xfa::{
    BareRef, CheckCaption, DataAttributes, LayoutOptions, MergeOptions, MergeWarningKind,
    PaintNoteKind, PaintOptions, ScopeSearch, ThreeDStroke,
};

use crate::text::xfapreview::{self as t, ChoiceKey};

/// Named region: the whole dialog body.
const REGION_BODY: &str = "dialog:form-preview"; // ui-text-exempt: trace region name, never displayed

/// Device pixels per point the pages are drawn at.
const SCALE: f32 = 1.5;

/// Pages drawn at most; the rest are counted in the trace and the heading.
const MAX_PAGES: usize = 50;

/// Every open choice, in display order.
const CHOICES: [ChoiceKey; 5] = [
    ChoiceKey::ScopeSearch,
    ChoiceKey::BareRef,
    ChoiceKey::DataAttributes,
    ChoiceKey::ThreeDStroke,
    ChoiceKey::CheckCaption,
];

/// The operator's answers, indexed as [`CHOICES`]: `true` is the second
/// (non-default) answer.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct Choices([bool; 5]);

impl Choices {
    fn get(self, key: ChoiceKey) -> bool {
        CHOICES
            .iter()
            .position(|k| *k == key)
            .is_some_and(|i| self.0[i])
    }

    fn merge(self) -> MergeOptions {
        let scope = if self.get(ChoiceKey::ScopeSearch) {
            ScopeSearch::Children
        } else {
            ScopeSearch::Upward
        };
        let bare = if self.get(ChoiceKey::BareRef) {
            BareRef::DataRef
        } else {
            BareRef::Ignore
        };
        let attributes = if self.get(ChoiceKey::DataAttributes) {
            DataAttributes::Ignore
        } else {
            DataAttributes::Load
        };
        MergeOptions::new()
            .with_scope_search(scope)
            .with_bare_ref(bare)
            .with_data_attributes(attributes)
    }

    fn paint(self) -> PaintOptions {
        let stroke = if self.get(ChoiceKey::ThreeDStroke) {
            ThreeDStroke::Plain
        } else {
            ThreeDStroke::Bevel
        };
        let caption = if self.get(ChoiceKey::CheckCaption) {
            CheckCaption::Remainder
        } else {
            CheckCaption::Fit
        };
        PaintOptions::new()
            .with_three_d_stroke(stroke)
            .with_check_caption(caption)
    }

    /// The trace's spelling: one digit per choice, `0` the default.
    fn code(self) -> String {
        self.0.iter().map(|b| if *b { '1' } else { '0' }).collect()
    }
}

/// One drawn page, ready to upload.
struct PagePixels {
    size: [usize; 2],
    rgba: Vec<u8>,
}

/// The worker's result: the pages and every sentence of disclosure.
struct Drawn {
    pages: Vec<PagePixels>,
    total_pages: usize,
    values: String,
    value_count: usize,
    merge_warning_count: usize,
    approximated: usize,
    notes: Vec<String>,
    warnings: Vec<String>,
}

type Slot = Arc<Mutex<Option<Result<Drawn, String>>>>;

/// What the window shows once the worker has answered.
enum Shown {
    Pages {
        textures: Vec<egui::TextureHandle>,
        total_pages: usize,
        values: String,
        notes: Vec<String>,
        warnings: Vec<String>,
    },
    Failed(String),
}

/// The Form-preview dialog. Its existence is its "open" state.
pub struct XfaPreviewDialog {
    session: Arc<EditSession>,
    options: pdfcer_render::RenderOptions,
    choices: Choices,
    slot: Slot,
    /// A worker is drawing; `shown` keeps the previous result until it lands.
    pending: bool,
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
        let mut dialog = Self {
            session,
            options,
            choices: Choices::default(),
            slot: Arc::default(),
            pending: false,
            shown: None,
            close_requested: false,
        };
        dialog.spawn(ctx);
        dialog
    }

    /// Draw the form with the current choices on a new worker. An earlier
    /// worker's late answer lands in its own slot, which is no longer read.
    fn spawn(&mut self, ctx: &egui::Context) {
        self.slot = Arc::default();
        self.pending = true;
        let out = Arc::clone(&self.slot);
        let repaint = ctx.clone();
        let session = Arc::clone(&self.session);
        let (options, choices) = (self.options.clone(), self.choices);
        let spawned = std::thread::Builder::new()
            .name("form-preview".into())
            .spawn(move || {
                let drawn = draw(&session, &options, choices);
                if let Ok(mut s) = out.lock() {
                    *s = Some(drawn);
                }
                repaint.request_repaint();
            });
        if let Err(e) = spawned {
            self.pending = false;
            self.shown = Some(Shown::Failed(t::failed(&e.to_string())));
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
        if !self.pending {
            return;
        }
        let Some(done) = self.slot.lock().ok().and_then(|mut s| s.take()) else {
            return;
        };
        self.pending = false;
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
                let (values, merged) = (d.value_count, d.merge_warning_count);
                let (approximated, choices) = (d.approximated, self.choices.code());
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    format!(
                        "form-preview pages={pages} notes={notes} warnings={warnings} \
                         values={values} merge_warnings={merged} \
                         approximated={approximated} choices={choices}"
                    )
                });
                Shown::Pages {
                    textures,
                    total_pages: d.total_pages,
                    values: d.values,
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
        if self.choices_ui(ui) {
            self.spawn(ui.ctx());
        }
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
                    values,
                    notes,
                    warnings,
                }) => {
                    if self.pending {
                        ui.horizontal(|ui| {
                            ui.add(egui::Spinner::new().color(theme.palette.text));
                            ui.label(t::working());
                        });
                    }
                    ui.label(values.as_str());
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

impl XfaPreviewDialog {
    /// The open choices as radio pairs. Returns `true` when one changed.
    fn choices_ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;
        egui::CollapsingHeader::new(t::choices_heading())
            .id_salt("form-preview-choices") // ui-text-exempt: widget id, never displayed
            .default_open(true)
            .show(ui, |ui| {
                ui.label(t::choices_caveat());
                for (i, key) in CHOICES.into_iter().enumerate() {
                    let (label, answers) = t::choice(key);
                    ui.horizontal_wrapped(|ui| {
                        ui.label(label);
                        for (n, answer) in answers.into_iter().enumerate() {
                            let on = self.choices.0[i] == (n == 1);
                            let r = ui.radio(on, answer);
                            // ui-text-exempt: trace region name, never displayed
                            let region = format!("form-preview.choice.{i}.{n}");
                            crate::diag::ui_rect(&region, r.rect);
                            if r.clicked() && !on {
                                self.choices.0[i] = n == 1;
                                changed = true;
                            }
                        }
                    });
                }
            });
        changed
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
fn draw(
    session: &EditSession,
    options: &pdfcer_render::RenderOptions,
    choices: Choices,
) -> Result<Drawn, String> {
    let view = session.view();
    let xdp = pdfcer_core::xfa::read_xdp(&view)
        .map_err(|e| t::failed(&e.to_string()))?
        .ok_or_else(|| t::no_template().to_owned())?;
    let template = pdfcer_core::xfa::read_template(&xdp)
        .map_err(|e| t::failed(&e.to_string()))?
        .ok_or_else(|| t::no_template().to_owned())?;
    let merged = pdfcer_core::xfa::merge_data(&template, &xdp, &choices.merge())
        .map_err(|e| t::failed(&e.to_string()))?;
    let template = &merged.template;
    let layout = pdfcer_core::xfa::layout_template(template, &LayoutOptions::new())
        .map_err(|e| t::failed(&e.to_string()))?;
    let paint = choices.paint();
    let images = pdfcer_core::xfa::XfaImages::from_view(&view);
    let mut pages = Vec::new();
    let mut counts: Vec<(PaintNoteKind, usize)> = Vec::new();
    for page in layout.pages.iter().take(MAX_PAGES) {
        let r = pdfcer_render::xfa::render_xfa_page_with_images(
            &view, template, page, &paint, &images, SCALE, options,
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
    let mut by_kind: Vec<(MergeWarningKind, usize)> = Vec::new();
    for w in &merged.warnings {
        match by_kind.iter_mut().find(|(k, _)| *k == w.kind) {
            Some((_, n)) => *n += 1,
            None => by_kind.push((w.kind, 1)),
        }
    }
    let mut warnings: Vec<String> = by_kind
        .into_iter()
        .map(|(k, n)| t::merge_warning(k, n))
        .collect();
    for w in &layout.warnings {
        let line = t::layout_warning(w).to_owned();
        if !warnings.contains(&line) {
            warnings.push(line);
        }
    }
    let approximated = counts
        .iter()
        .find(|(k, _)| *k == PaintNoteKind::StrokeApproximated)
        .map_or(0, |(_, n)| *n);
    Ok(Drawn {
        pages,
        total_pages: layout.pages.len(),
        values: t::values_merged(merged.bound.len(), merged.has_data),
        value_count: merged.bound.len(),
        merge_warning_count: merged.warnings.len(),
        approximated,
        notes: counts
            .into_iter()
            .map(|(k, n)| t::paint_note(k, n))
            .collect(),
        warnings,
    })
}
