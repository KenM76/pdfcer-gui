//! # `canvas::textedit::freetext` — a text box typed in place
//!
//! A dragged text box opens a draft anchored at [`Anchor::FreeText`]. Each
//! frame the draft's words are built into the annotation the commit will
//! author (`pdfcer_core::annot_author::free_text_preview`) and painted by the
//! interpreter that paints a saved one
//! (`pdfcer_render::edit_preview::paint_text_annotation_preview`), so the
//! pixels while typing are the commit's. The caret and the selection are read
//! off the same layout.
//!
//! The engine clips words that overflow the box and never grows it; the box
//! is grown downward here to the height its lines need, and the grown
//! rectangle is the one committed ([`commit_rect`]).
//!
//! Coordinates: the layout is in the box's own space (origin at its
//! lower-left corner, y up, points) with BYTE offsets into the trimmed text;
//! the draft's caret and mark are CHARACTER indices into the untrimmed text.

use egui::{Color32, Pos2, Rect};
use pdfcer_core::annot_author::{Color, FreeTextPreview, StampName, free_text_preview};
use pdfcer_core::page_tree::Rect as PageRect;

use super::{Anchor, Draft, Preview, TextEditKind, hit, store};
use crate::app::settings::SettingsExt;
use crate::canvas::textannot::{self, TextAnnotKind};
use crate::canvas::tool::CanvasTool;

/// The trace region of the box being typed in.
pub const REGION_BOX: &str = "text-box.draft"; // ui-text-exempt: trace region name, never displayed
const TEXTURE: &str = "text-box-draft-ink"; // ui-text-exempt: a texture name, never displayed.
const TRACE: &str = "text-box-preview"; // ui-text-exempt: diagnostic trace event name.
/// A drag smaller than this on either side opens nothing.
const MIN_PT: f64 = 12.0;
/// What an empty box is measured with, for the caret's height and baseline.
const METRICS_PROBE: &str = "M"; // ui-text-exempt: a measuring string, never displayed.

/// Whether `draft` is a text box being typed in place.
#[must_use]
pub const fn is(draft: &Draft) -> bool {
    matches!(draft.anchor, Anchor::FreeText { .. })
}

/// Whether `tool` is the one a text-box draft belongs to.
#[must_use]
pub fn tool_matches(tool: CanvasTool) -> bool {
    tool == CanvasTool::TextAnnot(TextAnnotKind::TextBox)
}

/// Whether a text-box draft is open under its own tool, so the canvas types
/// into it and paints it.
#[must_use]
pub fn live(ctx: &egui::Context, tool: CanvasTool) -> bool {
    tool_matches(tool) && super::read(ctx).is_some_and(|d| is(&d))
}

/// Whether the dragged text box on `page` is typed in place. A rotated page's
/// box is turned upright by the commit, which the preview does not model, so
/// it keeps the dialog.
#[must_use]
pub fn typed_in_place(kind: TextAnnotKind, page: &pdfcer_core::page_tree::Page) -> bool {
    kind == TextAnnotKind::TextBox && page.rotate == 0
}

/// Open an empty draft over `rect` (PDF user space) on `page`. `false` when
/// the rectangle is too small to be a box.
pub fn begin(ctx: &egui::Context, page: usize, rect: PageRect) -> bool {
    let (w, h) = (rect.urx - rect.llx, rect.ury - rect.lly);
    if w < MIN_PT || h < MIN_PT {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("text-box-draft-declined w={w:.1} h={h:.1} floor={MIN_PT:.1}")
        });
        return false;
    }
    store(
        ctx,
        Draft {
            page,
            kind: TextEditKind::Add,
            anchor: Anchor::FreeText {
                llx: rect.llx,
                lly: rect.lly,
                urx: rect.urx,
                ury: rect.ury,
            },
            text: String::new(),
            caret: 0,
            mark: None,
            seeded: false,
        },
    );
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "text-box-draft-begun page={page} rect={:.1},{:.1},{:.1},{:.1}",
            rect.llx, rect.lly, rect.urx, rect.ury
        )
    });
    true
}

/// The annotation `text` would author in `rect`, grown downward until every
/// line shows. `None` for blank text or a refused build.
fn bake(rect: PageRect, text: &str, colour: Color) -> Option<FreeTextPreview> {
    let at = |rect: PageRect| {
        let spec = textannot::spec(
            TextAnnotKind::TextBox,
            rect,
            text,
            StampName::Approved,
            &textannot::DEFAULT_STICKY_ICON,
            textannot::DEFAULT_STAMP_SIZE,
            colour,
        )?;
        free_text_preview(&spec).ok().flatten()
    };
    let first = at(rect)?;
    let grown = first.rect_grown_downward();
    if grown.lly < first.authored.rect.lly {
        at(grown)
    } else {
        Some(first)
    }
}

/// The rectangle a commit of `text` dragged as `rect` authors: `rect` grown
/// downward to the height its lines need.
#[must_use]
pub fn commit_rect(rect: PageRect, text: &str) -> PageRect {
    bake(rect, text, Color::Gray(0.0)).map_or(rect, |p| p.authored.rect)
}

/// Whether `text` overflows the box the commit authors, so some of it is
/// clipped: a word wider than the box.
#[must_use]
pub fn clips(spec: &pdfcer_core::annot_author::TextAnnotSpec) -> bool {
    free_text_preview(spec)
        .ok()
        .flatten()
        .is_some_and(|p| p.clips())
}

/// One caret stop: the line it is on and its x, both in the box's space.
#[derive(Clone, Copy)]
struct Stop {
    line: usize,
    x: f64,
    /// Lines below the last laid-out one: a caret after trailing line breaks,
    /// which the commit trims.
    below: usize,
}

/// The caret stop for byte offset `at` of the trimmed text.
fn stop_at(layout: &pdfcer_core::vartext::TextLayout, at: usize) -> Stop {
    for (line, l) in layout.lines.iter().enumerate() {
        if let Some(c) = l.chars.iter().find(|c| c.offset == at) {
            return Stop {
                line,
                x: c.x0,
                below: 0,
            };
        }
    }
    let line = layout
        .lines
        .iter()
        .rposition(|l| l.start <= at)
        .unwrap_or(0);
    let x = layout.lines.get(line).map_or(0.0, |l| {
        l.chars
            .iter()
            .rev()
            .find(|c| c.end <= at)
            .map_or(l.x0, |c| c.x1)
    });
    Stop { line, x, below: 0 }
}

/// A stop per character index of `text` (one more than its characters), read
/// off the layout of its trimmed form.
fn stops(layout: &pdfcer_core::vartext::TextLayout, text: &str, blank: bool) -> Vec<Stop> {
    let lead = text.len() - text.trim_start().len();
    let body = text.trim().len();
    let first_x = layout.lines.first().map_or(0.0, |l| l.x0);
    let mut out = Vec::with_capacity(text.chars().count() + 1);
    let offsets = text.char_indices().map(|(b, _)| b).chain([text.len()]);
    for b in offsets {
        let breaks_before = |from: usize| text[from..b].matches('\n').count();
        out.push(if blank {
            Stop {
                line: 0,
                x: first_x,
                below: breaks_before(0),
            }
        } else if b > lead + body {
            let end = stop_at(layout, body);
            match breaks_before(lead + body) {
                0 => end,
                below => Stop {
                    line: layout.lines.len().saturating_sub(1),
                    x: first_x,
                    below,
                },
            }
        } else {
            stop_at(layout, b.saturating_sub(lead))
        });
    }
    out
}

/// Box space to screen, for the box whose lower-left corner is `(llx, lly)`.
struct ToScreen {
    m: [f32; 6],
    llx: f64,
    lly: f64,
}

impl ToScreen {
    #[allow(clippy::cast_possible_truncation)]
    fn at(&self, x: f64, y: f64) -> Pos2 {
        let (x, y) = ((self.llx + x) as f32, (self.lly + y) as f32);
        let m = self.m;
        Pos2::new(m[0] * x + m[2] * y + m[4], m[1] * x + m[3] * y + m[5])
    }
}

/// The baseline of `stop`, in the box's space.
fn baseline(layout: &pdfcer_core::vartext::TextLayout, stop: Stop) -> f64 {
    let pitch = match layout.lines.as_slice() {
        [a, b, ..] => a.baseline - b.baseline,
        _ => layout.ascent + layout.descent,
    };
    #[allow(clippy::cast_precision_loss)]
    let below = stop.below as f64;
    layout.lines.get(stop.line).map_or(0.0, |l| l.baseline) - below * pitch
}

/// Paint the committed appearance of `preview`, cached on what it depends on.
#[allow(clippy::cast_possible_truncation)]
fn ink(
    ui: &egui::Ui,
    p: &Preview<'_>,
    m: [f32; 6],
    preview: &FreeTextPreview,
    opacity: f32,
) -> Option<(u32, u32, usize)> {
    use std::hash::{Hash, Hasher};
    let ctx = ui.ctx();
    let crate::canvas::dimpreview::Patch {
        body,
        w,
        h,
        to_pixels,
        mut pixmap,
    } = crate::canvas::dimpreview::patch(ctx, m, preview.authored.rect, ui.clip_rect())?;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    preview.authored.ap_content.hash(&mut hasher);
    let r = preview.authored.rect;
    for v in [r.llx, r.lly, r.urx, r.ury] {
        v.to_bits().hash(&mut hasher);
    }
    for v in m {
        v.to_bits().hash(&mut hasher);
    }
    [body.min.x, body.min.y, body.max.x, body.max.y]
        .map(f32::to_bits)
        .hash(&mut hasher);
    ctx.pixels_per_point().to_bits().hash(&mut hasher);
    let key = hasher.finish();
    let memory = egui::Id::new(TEXTURE);
    let cached = ctx.data(|d| d.get_temp::<(u64, egui::TextureHandle, usize)>(memory));
    let (texture, faults) = if let Some((_, texture, faults)) = cached.filter(|(k, ..)| *k == key) {
        (texture, faults)
    } else {
        let diagnostics = pdfcer_render::edit_preview::paint_text_annotation_preview(
            &p.doc.session.view(),
            &preview.authored,
            &p.doc.settings.render_options(),
            to_pixels,
            &mut pixmap,
        );
        let image =
            egui::ColorImage::from_rgba_premultiplied([w as usize, h as usize], pixmap.data());
        crate::render::pressure::record_other(
            ctx,
            crate::render::pressure::Surface::TextDraft,
            w,
            h,
        );
        let texture = ctx.load_texture(TEXTURE, image, egui::TextureOptions::NEAREST);
        let faults = diagnostics.sample_ops.len();
        ctx.data_mut(|d| d.insert_temp(memory, (key, texture.clone(), faults)));
        (texture, faults)
    };
    ui.painter().image(
        texture.id(),
        body,
        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
        Color32::WHITE.gamma_multiply(opacity), // NOT A THEME COLOUR: the tint that draws the baked page pixels at the pen's opacity
    );
    Some((w, h, faults))
}

/// Paint the selected characters `from..to`.
fn selection(
    painter: &egui::Painter,
    layout: &pdfcer_core::vartext::TextLayout,
    stops: &[Stop],
    to_screen: &ToScreen,
    (from, to): (usize, usize),
    fill: Color32,
) {
    for i in from..to.min(stops.len().saturating_sub(1)) {
        let (here, next) = (stops[i], stops[i + 1]);
        if here.below > 0 {
            continue;
        }
        let same_line = next.line == here.line && next.below == 0;
        let right = if same_line {
            next.x
        } else {
            layout
                .lines
                .get(here.line)
                .map_or(here.x, |l| l.x1.max(here.x))
        };
        let y = baseline(layout, here);
        let cell = Rect::from_two_pos(
            to_screen.at(here.x, y - layout.descent),
            to_screen.at(right, y + layout.ascent),
        );
        painter.rect_filled(cell, 0.0, fill);
    }
}

/// The action that writes `draft` as a text box, or `None` when it holds no
/// words or is not a text-box draft.
#[must_use]
pub fn commit(draft: &Draft) -> Option<crate::app::actions::Action> {
    let Anchor::FreeText { llx, lly, urx, ury } = draft.anchor else {
        return None;
    };
    if draft.text.trim().is_empty() {
        return None;
    }
    let dragged = PageRect { llx, lly, urx, ury };
    Some(crate::app::actions::Action::CommitTextAnnot {
        page: draft.page,
        kind: TextAnnotKind::TextBox,
        rect: commit_rect(dragged, &draft.text),
        text: draft.text.clone(),
        stamp: pdfcer_core::annot_author::StampName::Approved,
        stamp_size: crate::canvas::textannot::DEFAULT_STAMP_SIZE,
        icon: crate::canvas::textannot::DEFAULT_STICKY_ICON,
        custom: None,
    })
}

/// Draw the blinking caret at `stop`.
fn caret_line(
    ui: &egui::Ui,
    layout: &pdfcer_core::vartext::TextLayout,
    to_screen: &ToScreen,
    stop: Stop,
    colour: egui::Color32,
) {
    #[allow(clippy::cast_possible_truncation)]
    let blink = (ui.input(|i| i.time) * 1.6) as i64 % 2 == 0;
    if blink {
        let y = baseline(layout, stop);
        ui.painter().line_segment(
            [
                to_screen.at(stop.x, y - layout.descent),
                to_screen.at(stop.x, y + layout.ascent),
            ],
            egui::Stroke::new(1.5, colour),
        );
    }
}

/// Paint the open text-box draft: its committed appearance, its selection,
/// its outline and its caret, and publish the caret layout for the pointer.
pub(super) fn paint(ui: &egui::Ui, ctx: &egui::Context, p: &Preview<'_>, draft: &Draft) {
    let Anchor::FreeText { llx, lly, urx, ury } = draft.anchor else {
        return;
    };
    let Some(page) = p.doc.pages.get(p.page_index) else {
        return;
    };
    let Some(m) = crate::canvas::dimpreview::page_to_screen(page, p.map) else {
        return;
    };
    let dragged = PageRect { llx, lly, urx, ury };
    let colour = p.pen.text_annot_colour(TextAnnotKind::TextBox);
    let blank = draft.text.trim().is_empty();
    let words = if blank { METRICS_PROBE } else { &draft.text };
    let Some(preview) = bake(dragged, words, colour) else {
        crate::diag::trace_on_change(TRACE, || "baked=0".to_owned()); // ui-text-exempt: trace
        return;
    };
    let theme = egui_shell::theme::Theme::of(ctx);
    let painter = ui.painter();
    let rect = preview.authored.rect;
    let to_screen = ToScreen {
        m,
        llx: rect.llx,
        lly: rect.lly,
    };
    let body = Rect::from_two_pos(
        to_screen.at(0.0, 0.0),
        to_screen.at(rect.urx - rect.llx, rect.ury - rect.lly),
    );
    #[allow(clippy::cast_possible_truncation)]
    let opacity = p.pen.opacity_option().map_or(1.0, |o| o as f32);
    let drawn = if blank {
        None
    } else {
        ink(ui, p, m, &preview, opacity)
    };
    let layout = &preview.layout;
    let stops = stops(layout, &draft.text, blank);
    if let Some(range) = super::caret::range(draft.mark, draft.caret) {
        let fill = theme.palette.selection_fill;
        selection(painter, layout, &stops, &to_screen, range, fill);
    }
    painter.rect_stroke(
        body,
        0.0,
        egui::Stroke::new(1.0, theme.palette.accent),
        egui::StrokeKind::Outside,
    );
    let on_screen: Vec<Pos2> = stops
        .iter()
        .map(|s| to_screen.at(s.x, baseline(layout, *s)))
        .collect();
    super::ime::show(ui, ctx, body);
    crate::diag::ui_rect(REGION_BOX, body);
    let caret = stops.get(draft.caret).or(stops.last()).copied();
    hit::publish(
        ctx,
        hit::Layout {
            body,
            body_canvas: Rect::from_two_pos(p.map.to_page(body.min), p.map.to_page(body.max)),
            caret: hit::Caret::Stops(on_screen),
        },
    );
    ctx.request_repaint_after(std::time::Duration::from_millis(400));
    if let Some(stop) = caret {
        caret_line(ui, layout, &to_screen, stop, theme.palette.accent);
    }
    traced(&preview, drawn, blank, draft.text.chars().count(), lly);
}

/// Trace what was baked and painted; `dragged_lly` is the dragged box's
/// bottom, from which the growth is read.
fn traced(
    preview: &FreeTextPreview,
    drawn: Option<(u32, u32, usize)>,
    blank: bool,
    chars: usize,
    dragged_lly: f64,
) {
    crate::diag::trace_on_change(TRACE, || {
        let (w, h, faults) = drawn.unwrap_or((0, 0, 0));
        let rect = preview.authored.rect;
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "baked=1 blank={blank} px={w}x{h} faults={faults} lines={} chars={chars} \
             rect={:.1},{:.1},{:.1},{:.1} grown={:.1} clips={} unencodable={}",
            preview.layout.lines.len(),
            rect.llx,
            rect.lly,
            rect.urx,
            rect.ury,
            dragged_lly - rect.lly,
            !blank && preview.clips(),
            if blank {
                0
            } else {
                preview.authored.unencodable_chars
            },
        )
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    const BOX: PageRect = PageRect {
        llx: 100.0,
        lly: 200.0,
        urx: 220.0,
        ury: 230.0,
    };

    /// A box too short for its lines is committed taller, downward, and no
    /// wider; one that fits is committed as dragged.
    #[test]
    fn the_committed_box_grows_downward_to_show_every_line() {
        assert_eq!(commit_rect(BOX, "one line"), BOX);
        let tall = commit_rect(BOX, "one\ntwo\nthree\nfour\nfive");
        assert!(tall.lly < BOX.lly, "grew downward: {tall:?}");
        assert_eq!((tall.llx, tall.urx, tall.ury), (BOX.llx, BOX.urx, BOX.ury));
        let grown = bake(BOX, "one\ntwo\nthree\nfour\nfive", Color::Gray(0.0)).expect("bakes");
        assert!(!grown.clips(), "the grown box shows all of it");
    }

    /// Caret stops are one per character plus the end, advance along a line
    /// and drop to the next line at a break.
    #[test]
    fn caret_stops_follow_the_layout() {
        let text = "ab\ncd";
        let p = bake(BOX, text, Color::Gray(0.0)).expect("bakes");
        let s = stops(&p.layout, text, false);
        assert_eq!(s.len(), 6);
        assert!(s[1].x > s[0].x && s[2].x > s[1].x);
        assert_eq!((s[0].line, s[2].line), (0, 0));
        assert_eq!((s[3].line, s[5].line), (1, 1));
        assert!(s[5].x > s[3].x);
    }

    /// Leading blanks are trimmed by the commit, so their stops sit at the
    /// first drawn character; a trailing break puts the caret a line down.
    #[test]
    fn trimmed_blanks_still_have_caret_stops() {
        let text = "  ab\n";
        let p = bake(BOX, text, Color::Gray(0.0)).expect("bakes");
        let s = stops(&p.layout, text, false);
        assert_eq!(s.len(), 6);
        assert!((s[0].x - s[2].x).abs() < 1e-9);
        assert_eq!(s[5].below, 1);
        assert_eq!(s[4].below, 0);
    }

    /// An empty box still has a caret, at the first line's start.
    #[test]
    fn an_empty_box_has_one_stop() {
        let p = bake(BOX, METRICS_PROBE, Color::Gray(0.0)).expect("bakes");
        let s = stops(&p.layout, "", true);
        assert_eq!(s.len(), 1);
        assert_eq!((s[0].line, s[0].below), (0, 0));
    }
}
