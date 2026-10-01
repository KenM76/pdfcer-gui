//! # `canvas::textedit::shaped` — **an existing run's draft, in the run's own
//! font, where the run is**
//!
//! [`refresh`] runs once a frame after actions apply and asks the engine's
//! layout (`EditSession::edit_text_preview`) for the draft. It caches, per draft text, the replacement's glyph outlines
//! (`pdfcer_render::edit_preview::preview_outlines`) and caret stops, all in
//! page space. [`paint`] draws them over the run. [`read`] returning `None`
//! means the caller draws the shell-font editor box instead; that happens when
//! the draft is not on an existing run, the engine would refuse the edit, the
//! font has no outlines (Type 3, unsupported machinery), the run is invisible
//! (render mode 3 or 7), or glyphs and characters do not pair one to one, so no
//! caret could be placed. Each case is held as a `PreviewFallback` reason
//! ([`fallback`]) for the status bar to word.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/textedit/shaped.md`.

use std::sync::Arc;

use egui::{Color32, Pos2, Vec2};
use pdfcer_core::text_edit::{PreviewColour, TextEditPreview};
use pdfcer_gui_base::text::previewfallback::PreviewFallback;
use pdfcer_render::tiny_skia::{self, Path};

use super::{Anchor, Draft, Preview};
use crate::app::settings::SettingsExt;
use crate::app::state::OpenDoc;

const KEY: &str = "textedit-shaped"; // ui-text-exempt: a memory key, never displayed.
const TEXTURE: &str = "textedit-shaped-ink"; // ui-text-exempt: a texture name, never displayed.

/// The largest side, in pixels, the draft's ink is rasterised at. Past it the
/// editor box is drawn instead.
const MAX_SIDE_PX: f32 = 4096.0;

/// What the cached layout is a layout of.
#[derive(Clone, PartialEq, Eq)]
struct Key {
    page: usize,
    run: usize,
    original: String,
    text: String,
}

impl Key {
    fn of(draft: &Draft) -> Option<Self> {
        let Anchor::Run { run, original } = &draft.anchor else {
            return None;
        };
        Some(Self {
            page: draft.page,
            run: *run,
            original: original.clone(),
            text: draft.text.clone(),
        })
    }
}

/// The engine's layout of a draft. Page space throughout.
pub struct Shaped {
    /// One outline per character; `None` for a blank.
    outlines: Vec<Option<Path>>,
    /// The caret positions: before each character, then after the last.
    pub(super) stops: Vec<Pos2>,
    /// One em along the run's own vertical.
    up: Vec2,
    /// The run's ink.
    ink: Color32,
    /// The replacement's extent: advance by ascent and descent.
    bbox: [f64; 4],
    /// The text laid out.
    pub(super) text: String,
    /// The page-space box `[x0, y0, x1, y1]` of the original glyphs the
    /// replacement covers, when the preview is spliced from a narrowed edit
    /// (`splice`); `None` blanks the whole body.
    pub(super) blank: Option<[f32; 4]>,
}

#[derive(Clone)]
struct Cached {
    key: Key,
    shaped: Option<Arc<Shaped>>,
    /// Why `shaped` is `None`; `None` for an empty draft, which has nothing
    /// to draw in any font.
    fallback: Option<PreviewFallback>,
}

/// The latest layout for the run `draft` edits.
///
/// Keystrokes land during the frame and [`refresh`] runs after it, so for one
/// frame the layout can be of the previous text; it is still drawn, rather than
/// dropping to the editor box for that frame and flickering on every key.
#[must_use]
pub fn read(ctx: &egui::Context, draft: &Draft) -> Option<Arc<Shaped>> {
    let key = Key::of(draft)?;
    ctx.data(|d| d.get_temp::<Cached>(egui::Id::new(KEY)))
        .filter(|c| {
            c.key.page == key.page && c.key.run == key.run && c.key.original == key.original
        })
        .and_then(|c| c.shaped)
}

/// Why the run `draft` edits has no layout: `None` before its first layout,
/// `Some(None)` when it has one or the draft is empty. Matched as [`read`] is.
#[must_use]
pub fn fallback(ctx: &egui::Context, draft: &Draft) -> Option<Option<PreviewFallback>> {
    let key = Key::of(draft)?;
    ctx.data(|d| d.get_temp::<Cached>(egui::Id::new(KEY)))
        .filter(|c| {
            c.key.page == key.page && c.key.run == key.run && c.key.original == key.original
        })
        .map(|c| c.fallback)
}

/// Lay the current draft out, when its text changed since the last layout.
///
/// The first layout on a page decodes and walks it (hundreds of
/// milliseconds on a dense drawing); later keystrokes cost about ten.
pub fn refresh(ctx: &egui::Context, doc: &OpenDoc) {
    let id = egui::Id::new(KEY);
    let Some(draft) = super::read(ctx) else {
        ctx.data_mut(|d| d.remove::<Cached>(id));
        return;
    };
    let Some(key) = Key::of(&draft) else {
        ctx.data_mut(|d| d.remove::<Cached>(id));
        return;
    };
    if ctx
        .data(|d| d.get_temp::<Cached>(id))
        .is_some_and(|c| c.key == key)
    {
        return;
    }
    // The commit plans again, so this plan's traces and its record of what the
    // commit will write are both kept out of the record.
    let kept = super::last_commit();
    let plan =
        crate::diag::muted(|| super::plan::plan(doc, key.page, key.run, &key.original, &key.text));
    super::restore_last_commit(kept);
    let (laid, tier) = plan.attempt("preview", |request| {
        doc.session.edit_text_preview(request, &plan.options)
    });
    let refused = laid.as_ref().err().map(ToString::to_string);
    let shaped = match laid {
        Ok(p) => shape_any(doc, &p, &key, tier, &plan),
        Err(_) => Err(PreviewFallback::Refused),
    };
    let fallback = shaped
        .as_ref()
        .err()
        .copied()
        .filter(|_| !key.text.is_empty());
    let shaped = shaped.ok().map(Arc::new);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "text-edit-shaped page={} run={} chars={} shaped={} refused={} tier={tier:?}",
            key.page,
            key.run,
            key.text.chars().count(),
            u8::from(shaped.is_some()),
            u8::from(refused.is_some()),
        )
    });
    ctx.data_mut(|d| {
        d.insert_temp(
            id,
            Cached {
                key,
                shaped,
                fallback,
            },
        );
    });
    ctx.request_repaint();
}

/// The preview of `plan` on `tier`, whole or spliced: a narrowed request lays
/// out its operators, and a whole-line request spanning several operators lays
/// out only the part the engine trims it to.
fn shape_any(
    doc: &OpenDoc,
    preview: &TextEditPreview,
    key: &Key,
    tier: super::tier::Tier,
    plan: &super::plan::Plan,
) -> Result<Shaped, PreviewFallback> {
    use super::splice::Part;
    let splice =
        |part: Part<'_>| super::splice::shape(doc, preview, key.page, key.run, &part, &key.text);
    if let (super::tier::Tier::Narrowed, Some(n)) = (tier, &plan.narrowed) {
        return splice(Part {
            span: n.touched.original.clone(),
            replacement: &n.touched.replacement,
        });
    }
    if preview.glyphs.len() == key.text.chars().count() {
        return shape(doc, preview, &key.text);
    }
    let request = &plan.request;
    match pdfcer_gui_base::editmodel::narrow::engine_trim(&request.find, &request.replace) {
        Some((span, replacement)) if request.find == key.original => splice(Part {
            span,
            replacement: &replacement,
        }),
        _ => Err(PreviewFallback::Unpaired),
    }
}

#[allow(clippy::cast_possible_truncation)]
pub(super) fn shape(
    doc: &OpenDoc,
    preview: &TextEditPreview,
    text: &str,
) -> Result<Shaped, PreviewFallback> {
    let count = text.chars().count();
    if count == 0 || preview.glyphs.len() != count {
        return Err(PreviewFallback::Unpaired);
    }
    let ink = match preview.render_mode {
        0 | 2 | 4 | 6 => &preview.fill,
        1 | 5 => &preview.stroke,
        _ => return Err(PreviewFallback::Invisible),
    };
    let outlines = pdfcer_render::edit_preview::preview_outlines(
        &doc.session.view(),
        preview,
        &doc.settings.render_options().fonts,
    );
    if outlines.skipped.is_some() {
        return Err(PreviewFallback::NoOutlines);
    }
    let origin = |m: &[f64; 6]| Pos2::new(m[4] as f32, m[5] as f32);
    let mut stops: Vec<Pos2> = preview.glyphs.iter().map(|g| origin(&g.matrix)).collect();
    let last = preview
        .glyphs
        .last()
        .ok_or(PreviewFallback::Unpaired)?
        .matrix;
    let along = Vec2::new(last[0] as f32, last[1] as f32);
    if along.length() <= f32::EPSILON {
        return Err(PreviewFallback::Unpaired);
    }
    let unit = along.normalized();
    let from = origin(&last);
    let [x0, y0, x1, y1] = preview.bbox.map(|v| v as f32);
    let reach = [(x0, y0), (x1, y0), (x0, y1), (x1, y1)]
        .into_iter()
        .map(|(x, y)| (Pos2::new(x, y) - from).dot(unit))
        .fold(0.0_f32, f32::max);
    stops.push(from + unit * reach);
    Ok(Shaped {
        outlines: outlines.glyphs,
        stops,
        up: Vec2::new(last[2] as f32, last[3] as f32),
        ink: colour(ink),
        bbox: preview.bbox,
        text: text.to_owned(),
        blank: None,
    })
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn colour(c: &PreviewColour) -> Color32 {
    // DOCUMENT COLOUR: the run's own fill, as the saved page draws it; an
    // unmodelled space falls back to the PDF's initial fill, black.
    let u = |v: f64| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    match c {
        PreviewColour::Gray(g) => Color32::from_gray(u(*g)),
        PreviewColour::Rgb([r, g, b]) => Color32::from_rgb(u(*r), u(*g), u(*b)),
        PreviewColour::Cmyk([c, m, y, k]) => Color32::from_rgb(
            u((1.0 - c) * (1.0 - k)),
            u((1.0 - m) * (1.0 - k)),
            u((1.0 - y) * (1.0 - k)),
        ),
        // DOCUMENT COLOUR: the PDF's initial fill colour.
        _ => Color32::BLACK,
    }
}

/// The page-space to screen affine, as `[a, b, c, d, e, f]` with
/// `x' = a·x + c·y + e` and `y' = b·x + d·y + f`, read off three mapped points.
fn page_to_screen(p: &Preview<'_>, page: &pdfcer_core::page_tree::Page) -> Option<[f32; 6]> {
    let at = |x: f32, y: f32| {
        crate::viewer::pdf_space_to_canvas(Pos2::new(x, y), page).map(|c| p.map.to_screen(c))
    };
    let o = at(0.0, 0.0)?;
    let ex = at(1.0, 0.0)? - o;
    let ey = at(0.0, 1.0)? - o;
    Some([ex.x, ex.y, ey.x, ey.y, o.x, o.y])
}

fn apply(m: &[f32; 6], q: Pos2) -> Pos2 {
    Pos2::new(
        m[0] * q.x + m[2] * q.y + m[4],
        m[1] * q.x + m[3] * q.y + m[5],
    )
}

/// **Draw the draft as it will be written**: the run's own glyphs, in its own
/// ink, over a paper-white cover of what they replace; the selection under
/// them; the caret; an accent edge marking the editor as open. `covered` is the
/// original run's box on screen. Returns `false` when nothing was drawn and
/// the caller must draw the editor box.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn paint(
    ui: &egui::Ui,
    ctx: &egui::Context,
    p: &Preview<'_>,
    draft: &Draft,
    shaped: &Shaped,
    covered: egui::Rect,
) -> bool {
    let Some(page) = p.doc.pages.get(p.page_index) else {
        return false;
    };
    let Some(m) = page_to_screen(p, page) else {
        return false;
    };
    let [x0, y0, x1, y1] = shaped.bbox.map(|v| v as f32);
    let ink_box = egui::Rect::from_points(
        &[(x0, y0), (x1, y0), (x0, y1), (x1, y1)].map(|(x, y)| apply(&m, Pos2::new(x, y))),
    );
    let body = ink_box.union(covered).expand(1.0);
    let ppp = ctx.pixels_per_point();
    if body.width() * ppp > MAX_SIDE_PX || body.height() * ppp > MAX_SIDE_PX {
        return false;
    }
    let Some(ink) = ink_texture(ctx, &m, shaped, body) else {
        return false;
    };
    let theme = egui_shell::theme::Theme::of(ctx);
    let painter = ui.painter();
    let up = |at: Pos2, by: f32| apply(&m, at + shaped.up * by);

    // NOT A THEME COLOUR: paper white, the page's own background, which the
    // cover must match; the image tint below is the identity.
    let white = shaped.blank.map_or(body, |[x0, y0, x1, y1]| {
        let held = egui::Rect::from_points(
            &[(x0, y0), (x1, y0), (x0, y1), (x1, y1)].map(|(x, y)| apply(&m, Pos2::new(x, y))),
        );
        ink_box.union(held).expand(1.0)
    });
    painter.rect_filled(white, 0.0, Color32::WHITE);
    if let Some((from, to)) = super::caret::range(draft.mark, draft.caret) {
        for i in from..to.min(shaped.stops.len() - 1) {
            let (a, b) = (shaped.stops[i], shaped.stops[i + 1]);
            painter.add(egui::Shape::convex_polygon(
                vec![up(a, -0.25), up(b, -0.25), up(b, 0.9), up(a, 0.9)],
                theme.palette.selection_fill,
                egui::Stroke::NONE,
            ));
        }
    }
    painter.image(
        ink.id(),
        body,
        egui::Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
        // NOT A THEME COLOUR: the identity tint.
        Color32::WHITE,
    );
    painter.rect_stroke(
        body,
        0.0,
        egui::Stroke::new(1.0, theme.palette.accent),
        egui::StrokeKind::Outside,
    );
    let on = (ui.input(|i| i.time) * 1.6) as i64 % 2 == 0;
    ctx.request_repaint_after(std::time::Duration::from_millis(400));
    if on {
        let at = shaped.stops[draft.caret.min(shaped.stops.len() - 1)];
        painter.line_segment(
            [up(at, -0.25), up(at, 0.9)],
            egui::Stroke::new(1.5, theme.palette.accent),
        );
    }
    crate::diag::ui_rect(super::paint::REGION_BOX, body);
    super::hit::publish(
        ctx,
        super::hit::Layout {
            body,
            body_canvas: egui::Rect::from_two_pos(p.map.to_page(body.min), p.map.to_page(body.max)),
            caret: super::hit::Caret::Stops(shaped.stops.iter().map(|s| apply(&m, *s)).collect()),
        },
    );
    true
}

/// The draft's glyphs rasterised to cover `body`, reused while neither the
/// text nor where it lands on screen has changed.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn ink_texture(
    ctx: &egui::Context,
    m: &[f32; 6],
    shaped: &Shaped,
    body: egui::Rect,
) -> Option<egui::TextureHandle> {
    let ppp = ctx.pixels_per_point();
    let key = (
        shaped.text.clone(),
        [body.min.x, body.min.y, body.max.x, body.max.y, ppp].map(|v| (v * 8.0).round() as i64),
    );
    let id = egui::Id::new(TEXTURE);
    if let Some((held, texture)) =
        ctx.data(|d| d.get_temp::<((String, [i64; 5]), egui::TextureHandle)>(id))
        && held == key
    {
        return Some(texture);
    }
    let (w, h) = (
        (body.width() * ppp).ceil().max(1.0) as u32,
        (body.height() * ppp).ceil().max(1.0) as u32,
    );
    let mut pixmap = tiny_skia::Pixmap::new(w, h)?;
    let to_pixels = tiny_skia::Transform::from_row(
        m[0] * ppp,
        m[1] * ppp,
        m[2] * ppp,
        m[3] * ppp,
        (m[4] - body.min.x) * ppp,
        (m[5] - body.min.y) * ppp,
    );
    let mut paint = tiny_skia::Paint::default();
    paint.set_color_rgba8(shaped.ink.r(), shaped.ink.g(), shaped.ink.b(), 255);
    paint.anti_alias = true;
    for path in shaped.outlines.iter().flatten() {
        pixmap.fill_path(path, &paint, tiny_skia::FillRule::Winding, to_pixels, None);
    }
    let image = egui::ColorImage::from_rgba_premultiplied([w as usize, h as usize], pixmap.data());
    crate::render::pressure::record_other(ctx, crate::render::pressure::Surface::TextDraft, w, h);
    let texture = ctx.load_texture(TEXTURE, image, egui::TextureOptions::LINEAR);
    ctx.data_mut(|d| d.insert_temp(id, (key, texture.clone())));
    Some(texture)
}
