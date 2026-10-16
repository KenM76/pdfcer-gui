//! # `canvas::textedit::shapedblock` — an open paragraph, in its own fonts,
//! where it will be written
//!
//! [`shape`] lays an `Anchor::Block` draft out through
//! `EditSession::edit_block_text_preview` and joins the preview's runs, one
//! per look, into one [`Shaped`]: every glyph's outline in its own run's ink,
//! and a caret stop before every character of the draft. A character the
//! layout draws no glyph for (the space where a line wraps, a typed break, a
//! second space the layout collapses) takes the start of the next glyph on
//! its line, else the end of its line. A layout whose drawn characters do not
//! pair with the draft's, in order, is `Unpaired`, and the editor box is drawn.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/textedit/shapedblock.md`.

use egui::{Color32, Pos2, Vec2};
use pdfcer_gui_base::text::previewfallback::PreviewFallback;
use pdfcer_render::tiny_skia::Path;

use super::shaped::{Shaped, colour, ink_of};
use crate::app::settings::SettingsExt;
use crate::app::state::OpenDoc;

/// One drawn glyph of the paragraph, page space.
struct Glyph {
    ch: Option<char>,
    matrix: [f64; 6],
    path: Option<Path>,
    ink: Color32,
    /// The index into `BlockEditPreview::lines` whose baseline it sits on.
    line: usize,
}

/// A laid-out line: its first glyph's origin and natural width, page space.
struct Line {
    x: f64,
    y: f64,
    width: f64,
}

impl Glyph {
    #[allow(clippy::cast_possible_truncation)]
    fn origin(&self) -> Pos2 {
        Pos2::new(self.matrix[4] as f32, self.matrix[5] as f32)
    }
}

/// The paragraph `block` on `page` rewritten to `text`, laid out as it will
/// be committed.
///
/// # Errors
///
/// The [`PreviewFallback`] the status line words: the engine refused the
/// rewrite, a font has no outlines, or glyphs and characters do not pair.
#[allow(clippy::cast_possible_truncation)]
pub(super) fn shape(
    doc: &OpenDoc,
    page: usize,
    block: usize,
    text: &str,
) -> Result<Shaped, PreviewFallback> {
    if text.trim().is_empty() {
        return Err(PreviewFallback::Unpaired);
    }
    let preview = doc
        .session
        .edit_block_text_preview(page, block, text, &super::promote::options(doc))
        .map_err(|_| PreviewFallback::Refused)?;
    let lines: Vec<Line> = preview
        .lines
        .iter()
        .map(|l| Line {
            x: l.origin_x,
            y: l.baseline_y,
            width: l.width,
        })
        .collect();
    let first = preview.runs.first().ok_or(PreviewFallback::Unpaired)?;
    let (first_ink, invisible) = ink_of(first);
    let view = doc.session.view();
    let fonts = &doc.settings.render_options().fonts;
    let mut glyphs = Vec::new();
    for run in &preview.runs {
        let outlines = pdfcer_render::edit_preview::preview_outlines(&view, run, fonts);
        if outlines.skipped.is_some() {
            return Err(PreviewFallback::NoOutlines);
        }
        if outlines.glyphs.len() != run.glyphs.len() {
            return Err(PreviewFallback::Unpaired);
        }
        let ink = colour(ink_of(run).0);
        for (g, path) in run.glyphs.iter().zip(outlines.glyphs) {
            glyphs.push(Glyph {
                ch: g.ch,
                matrix: g.matrix,
                path,
                ink,
                line: line_of(&lines, g.matrix[5]),
            });
        }
    }
    glyphs.sort_by(|a, b| {
        a.line
            .cmp(&b.line)
            .then(a.matrix[4].total_cmp(&b.matrix[4]))
    });
    let stops = stops(&glyphs, &lines, text)?;
    let em = glyphs
        .iter()
        .map(|g| Vec2::new(g.matrix[2] as f32, g.matrix[3] as f32))
        .max_by(|a, b| a.length().total_cmp(&b.length()))
        .ok_or(PreviewFallback::Unpaired)?;
    let b = preview.new_bbox;
    let o = preview.old_bbox;
    let (inks, outlines) = glyphs.into_iter().map(|g| (g.ink, g.path)).unzip();
    Ok(Shaped {
        outlines,
        stops,
        up: em,
        ink: colour(first_ink),
        inks,
        invisible,
        bbox: [
            b.llx.min(o.llx),
            b.lly.min(o.lly),
            b.urx.max(o.urx),
            b.ury.max(o.ury),
        ],
        text: text.to_owned(),
        blank: None,
    })
}

/// The line whose baseline is nearest `y`.
fn line_of(lines: &[Line], y: f64) -> usize {
    lines
        .iter()
        .enumerate()
        .min_by(|a, b| (a.1.y - y).abs().total_cmp(&(b.1.y - y).abs()))
        .map_or(0, |(i, _)| i)
}

/// One caret stop before each character of `text`, then one after the last.
/// `glyphs` are in reading order.
#[allow(clippy::cast_possible_truncation)]
fn stops(glyphs: &[Glyph], lines: &[Line], text: &str) -> Result<Vec<Pos2>, PreviewFallback> {
    let start_of = |l: usize| lines.get(l).map(|li| Pos2::new(li.x as f32, li.y as f32));
    let end_of = |l: usize| {
        let li = lines.get(l)?;
        let drawn = glyphs
            .iter()
            .filter(|g| g.line == l)
            .map(|g| g.matrix[4])
            .fold(f64::NEG_INFINITY, f64::max);
        Some(Pos2::new((li.x + li.width).max(drawn) as f32, li.y as f32))
    };
    let mut out = Vec::with_capacity(text.chars().count() + 1);
    let mut next = 0;
    let mut line = 0;
    for c in text.chars() {
        let glyph = glyphs.get(next);
        if let Some(g) = glyph.filter(|g| g.ch == Some(c)) {
            out.push(g.origin());
            line = g.line;
            next += 1;
            continue;
        }
        if !c.is_whitespace() {
            return Err(PreviewFallback::Unpaired);
        }
        let stop = match glyph {
            Some(g) if g.line == line && c != '\n' => Some(g.origin()),
            _ => end_of(line),
        };
        out.push(stop.ok_or(PreviewFallback::Unpaired)?);
        if c == '\n' {
            line += 1;
        }
    }
    if next != glyphs.len() {
        return Err(PreviewFallback::Unpaired);
    }
    let after = if text.ends_with('\n') {
        start_of(line).or_else(|| end_of(line.saturating_sub(1)))
    } else {
        end_of(line)
    };
    out.push(after.ok_or(PreviewFallback::Unpaired)?);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(x: f64, y: f64, width: f64) -> Line {
        Line { x, y, width }
    }

    fn glyph(ch: char, x: f64, line: usize, y: f64) -> Glyph {
        Glyph {
            ch: Some(ch),
            matrix: [10.0, 0.0, 0.0, 10.0, x, y],
            path: None,
            ink: Color32::BLACK, // NOT A THEME COLOUR: a test glyph; stops ignore ink
            line,
        }
    }

    /// A wrap space sits at its line's end; the next word starts the next line.
    #[test]
    fn a_wrapped_space_takes_its_lines_end() {
        let lines = [line(0.0, 100.0, 12.0), line(0.0, 88.0, 12.0)];
        let glyphs = [
            glyph('a', 0.0, 0, 100.0),
            glyph('b', 6.0, 0, 100.0),
            glyph('c', 0.0, 1, 88.0),
            glyph('d', 6.0, 1, 88.0),
        ];
        let s = stops(&glyphs, &lines, "ab cd").expect("pairs");
        assert_eq!(s.len(), 6);
        assert_eq!(s[2], Pos2::new(12.0, 100.0));
        assert_eq!(s[3], Pos2::new(0.0, 88.0));
        assert_eq!(s[5], Pos2::new(12.0, 88.0));
    }

    /// A glyph whose character differs from the draft's is not paired.
    #[test]
    fn a_substituted_character_is_unpaired() {
        let lines = [line(0.0, 100.0, 12.0)];
        let glyphs = [glyph('a', 0.0, 0, 100.0), glyph('x', 6.0, 0, 100.0)];
        assert!(stops(&glyphs, &lines, "ab").is_err());
    }

    /// A trailing typed break puts the last stop at the start of the new line.
    #[test]
    fn a_trailing_break_ends_on_the_next_line() {
        let lines = [line(5.0, 100.0, 12.0), line(5.0, 88.0, 0.0)];
        let glyphs = [glyph('a', 5.0, 0, 100.0), glyph('b', 11.0, 0, 100.0)];
        let s = stops(&glyphs, &lines, "ab\n").expect("pairs");
        assert_eq!(s[2], Pos2::new(17.0, 100.0));
        assert_eq!(s[3], Pos2::new(5.0, 88.0));
    }
}
