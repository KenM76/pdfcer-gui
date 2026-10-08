//! # `canvas::ocrink` — the OCR layer, drawn in its own fonts
//!
//! Each invisible run of the page the layer shows is laid out by the same two
//! calls that draw a text edit while it is typed (`textedit::shaped`):
//! `EditSession::edit_text_preview` of each of its show operators, unchanged,
//! then `pdfcer_render::edit_preview::preview_outlines`. The layer,
//! an edit in progress and the text once committed are therefore one drawing,
//! in the run's font, size, horizontal scaling and position.
//!
//! Contract:
//! - [`step`] lays out runs for at most [`BUDGET`] of each frame and caches
//!   the outlines per (session, edit epoch, page); while runs remain it asks
//!   for another frame. Nothing waits on it.
//! - A run with no outlines here — not reached yet, refused by the engine for
//!   any of its show operators, in a font with no outlines, or laid out with a
//!   different glyph count from its text — is drawn by
//!   `ocrlayer`'s fitted stand-in; [`Ink::drawn`] says which.
//! - [`paint`] rasterises the outlines to one texture covering the visible
//!   part of the page, in the layer's colour, and draws it at the layer's
//!   opacity. The run being edited is left out; `textedit::shaped` draws it.
//!
//! The engine's renderer cannot paint invisible text; once it can, this module
//! is replaced by a render of the layer. Design: `docs/modules/pdfcer-gui/canvas/ocrink.md`.

use std::sync::Arc;
use std::time::{Duration, Instant};

use egui::{Color32, Painter, Pos2, Rect};
use pdfcer_core::text_edit::{
    BlockRecognitionOptions, EditOptions, EditRequest, EditableTextModel,
};
use pdfcer_core::text_extract::PageText;
use pdfcer_render::tiny_skia::Path;

use crate::app::settings::SettingsExt;
use crate::app::state::OpenDoc;
use crate::canvas::strip::PageView;
use crate::canvas::textedit::{pin, shaped};

const KEY: &str = "ocr-ink"; // ui-text-exempt: a memory key, never displayed.
const TEXTURE: &str = "ocr-ink-texture"; // ui-text-exempt: a texture name, never displayed.

/// The wall time one frame may spend laying runs out.
pub(super) const BUDGET: Duration = Duration::from_millis(6);

/// What a layout is of.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Key {
    /// The session's address: a document replaced in its tab is a new one.
    session: usize,
    epoch: u64,
    page: usize,
}

/// The page's invisible runs, laid out so far. Indexed as the page's
/// provenance-bearing extraction (`OpenDoc::provenance_page_text`) indexes
/// its runs.
#[derive(Clone)]
pub(super) struct Ink {
    key: Key,
    /// One entry per run reached: the run's glyph outlines in page space, or
    /// `None` when it is drawn by the stand-in.
    runs: Arc<Vec<Option<Vec<Path>>>>,
    /// How many runs the page has.
    total: usize,
    /// Wall time spent laying out, summed over frames.
    spent: Duration,
}

impl Ink {
    /// Whether run `i` is drawn here rather than by the stand-in.
    #[must_use]
    pub fn drawn(&self, i: usize) -> bool {
        self.runs.get(i).is_some_and(Option::is_some)
    }

    fn done(&self) -> bool {
        self.runs.len() >= self.total
    }
}

/// Lay out the next runs of `page` and answer everything laid out so far.
pub(super) fn step(ctx: &egui::Context, doc: &OpenDoc, text: &PageText) -> Ink {
    let id = egui::Id::new(KEY);
    let key = Key {
        session: Arc::as_ptr(&doc.session) as usize,
        epoch: doc.edit_epoch,
        page: text.page_index,
    };
    let held = ctx.data_mut(|d| {
        let held = d.get_temp::<Ink>(id);
        d.remove::<Ink>(id);
        held
    });
    let mut ink = held.filter(|i| i.key == key).unwrap_or_else(|| Ink {
        key,
        runs: Arc::new(Vec::new()),
        total: text.runs.len(),
        spent: Duration::ZERO,
    });
    if !ink.done() {
        extend(doc, text, &mut ink);
        if ink.done() {
            trace_built(&ink);
        } else {
            ctx.request_repaint();
        }
    }
    ctx.data_mut(|d| d.insert_temp(id, ink.clone()));
    ink
}

/// Lay out runs from where the last frame stopped until the budget is spent.
fn extend(doc: &OpenDoc, text: &PageText, ink: &mut Ink) {
    let started = Instant::now();
    let model = EditableTextModel::recognize(text, &BlockRecognitionOptions::default());
    let runs = Arc::make_mut(&mut ink.runs);
    while runs.len() < ink.total && started.elapsed() < BUDGET {
        let i = runs.len();
        let run = &text.runs[i];
        runs.push(
            super::ocrlayer::is_ocr_run(run)
                .then(|| lay(doc, &model, text, i))
                .flatten(),
        );
    }
    ink.spent += started.elapsed();
}

/// Run `i`'s outlines: each of its show operators laid out as an unchanged
/// edit of that operator lays it out. `None` when any one cannot be.
fn lay(
    doc: &OpenDoc,
    model: &EditableTextModel<'_>,
    text: &PageText,
    i: usize,
) -> Option<Vec<Path>> {
    let run = &text.runs[i].text;
    let mut paths = Vec::new();
    for op in pin::operators_in_run(model, text, i) {
        paths.extend(lay_operator(
            doc,
            text.page_index,
            op.pin,
            run.get(op.text)?,
        )?);
    }
    (!paths.is_empty()).then_some(paths)
}

/// One show operator's outlines, through the text editor's preview.
fn lay_operator(doc: &OpenDoc, page: usize, pinned: pin::Pinned, text: &str) -> Option<Vec<Path>> {
    let mut request = EditRequest::find_replace(page, "", text);
    request.pinned_span = Some(pinned.span);
    request.target = pinned.target;
    let preview = doc
        .session
        .edit_text_preview(&request, &EditOptions::default())
        .ok()?;
    if preview.rewritten.is_some() || preview.glyphs.len() != text.chars().count() {
        return None;
    }
    let outlines = pdfcer_render::edit_preview::preview_outlines(
        &doc.session.view(),
        &preview,
        &doc.settings.render_options().fonts,
    );
    if outlines.skipped.is_some() {
        return None;
    }
    Some(outlines.glyphs.into_iter().flatten().collect())
}

fn trace_built(ink: &Ink) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        let laid = ink.runs.iter().filter(|r| r.is_some()).count();
        format!(
            "ocr-ink-built page={} runs={} laid={laid} ms={}",
            ink.key.page,
            ink.total,
            ink.spent.as_millis()
        )
    });
}

/// **Draw the laid-out runs** of `ink` on `view`, all but `held`, in `colour`
/// at opacity `alpha`, within the painter's clip.
#[allow(clippy::cast_possible_truncation)]
pub(super) fn paint(
    painter: &Painter,
    doc: &OpenDoc,
    view: &PageView,
    ink: &Ink,
    held: Option<usize>,
    colour: Color32,
    alpha: u8,
) {
    let Some(page) = doc.pages.get(ink.key.page) else {
        return;
    };
    let Some(m) = shaped::page_to_screen(&view.map, page) else {
        return;
    };
    let paths = || {
        ink.runs
            .iter()
            .enumerate()
            .filter(move |(i, _)| Some(*i) != held)
            .filter_map(|(_, r)| r.as_ref())
            .flatten()
    };
    let Some(extent) = paths().map(Path::bounds).reduce(|a, b| {
        let (l, t) = (a.left().min(b.left()), a.top().min(b.top()));
        let (r, bt) = (a.right().max(b.right()), a.bottom().max(b.bottom()));
        pdfcer_render::tiny_skia::Rect::from_ltrb(l, t, r, bt).unwrap_or(a)
    }) else {
        return;
    };
    let corners = [
        (extent.left(), extent.top()),
        (extent.right(), extent.top()),
        (extent.left(), extent.bottom()),
        (extent.right(), extent.bottom()),
    ]
    .map(|(x, y)| shaped::apply(&m, Pos2::new(x, y)));
    let body = Rect::from_points(&corners)
        .expand(1.0)
        .intersect(painter.clip_rect());
    if !body.is_positive() {
        return;
    }
    let ctx = painter.ctx();
    let ppp = ctx.pixels_per_point();
    let tag = (
        ink.key,
        ink.runs.len(),
        held,
        colour.to_array(),
        [body.min.x, body.min.y, body.max.x, body.max.y, ppp].map(|v| (v * 8.0).round() as i64),
    );
    let id = egui::Id::new(TEXTURE);
    let cached = ctx
        .data(|d| d.get_temp::<(Tag, egui::TextureHandle)>(id))
        .filter(|(t, _)| *t == tag)
        .map(|(_, t)| t);
    let texture = match cached {
        Some(t) => t,
        None => {
            let Some(image) = shaped::fill_paths(paths(), &m, body, ppp, colour) else {
                return;
            };
            let [w, h] = image.size;
            crate::render::pressure::record_other(
                ctx,
                crate::render::pressure::Surface::OcrLayer,
                w as u32,
                h as u32,
            );
            let t = ctx.load_texture(TEXTURE, image, egui::TextureOptions::LINEAR);
            ctx.data_mut(|d| d.insert_temp(id, (tag, t.clone())));
            t
        }
    };
    painter.image(
        texture.id(),
        body,
        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
        // NOT A THEME COLOUR: the identity tint, scaled to the layer's opacity.
        Color32::WHITE.gamma_multiply(f32::from(alpha) / 255.0),
    );
}

type Tag = (Key, usize, Option<usize>, [u8; 4], [i64; 5]);
