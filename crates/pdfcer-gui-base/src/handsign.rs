//! # `handsign` — a hand signature, drawn or typed: its strokes, their fit into a signature box, the copy kept on this computer, and which boxes this session has signed
//!
//! Contract: [`Mark`] holds strokes in a y-down space of any unit; [`fit`]
//! places a mark inside a y-down target rectangle by the rule below and
//! returns points in that same space; [`Ledger`] answers *"is this field
//! hand-signed in this session?"* from undo-stack depths alone.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/handsign.md`.

use std::path::PathBuf;

use egui::{Pos2, Rect, pos2};

/// The file a remembered signature is kept in, beside `settings.txt`.
pub const SAVED_FILE: &str = "hand-signature.txt"; // ui-text-exempt: a file name, never displayed as copy

pub mod typed;

/// A signature the operator made: drawn with the mouse, or typed in a
/// handwriting face.
#[derive(Clone, Debug, PartialEq)]
pub enum Signature {
    /// Strokes, normalised ([`Mark::normalised`]).
    Drawn(Mark),
    /// A name and the face it is written in.
    Typed(typed::Typed),
}

/// The tolerance [`Mark::simplified`] removes detail below, in pad points.
pub const SIMPLIFY_TOLERANCE: f32 = 0.6;

/// A drawn signature: one point list per pen-down stroke, y-down.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Mark {
    /// Each stroke's points in drawing order. A stroke of one point is a dot.
    pub strokes: Vec<Vec<Pos2>>,
}

impl Mark {
    /// Whether nothing has been drawn.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.strokes.iter().all(Vec::is_empty)
    }

    /// Every point, across strokes.
    #[must_use]
    pub fn point_count(&self) -> usize {
        self.strokes.iter().map(Vec::len).sum()
    }

    /// The smallest rectangle holding every point, or `None` when empty.
    #[must_use]
    pub fn bounds(&self) -> Option<Rect> {
        let mut points = self.strokes.iter().flatten();
        let first = *points.next()?;
        Some(points.fold(Rect::from_min_max(first, first), |r, p| {
            r.union(Rect::from_min_max(*p, *p))
        }))
    }

    /// Whether the mark spans some distance — a lone tap is not a signature.
    #[must_use]
    pub fn has_extent(&self) -> bool {
        self.bounds()
            .is_some_and(|b| b.width() > 0.5 || b.height() > 0.5)
    }

    /// The mark moved to the origin and scaled so its longer side is 1,
    /// keeping its proportions — the form it is saved and carried in.
    #[must_use]
    pub fn normalised(&self) -> Self {
        let Some(b) = self.bounds() else {
            return Self::default();
        };
        let side = b.width().max(b.height()).max(f32::EPSILON);
        Self {
            strokes: self
                .strokes
                .iter()
                .map(|s| {
                    s.iter()
                        .map(|p| pos2((p.x - b.min.x) / side, (p.y - b.min.y) / side))
                        .collect()
                })
                .collect(),
        }
    }

    /// Each stroke reduced by Ramer–Douglas–Peucker to the points that bend
    /// it by more than `tolerance`; empty strokes dropped.
    #[must_use]
    pub fn simplified(&self, tolerance: f32) -> Self {
        Self {
            strokes: self
                .strokes
                .iter()
                .filter(|s| !s.is_empty())
                .map(|s| simplify(s, tolerance))
                .collect(),
        }
    }

    /// The on-disk form: one stroke per line, `x,y` pairs separated by spaces.
    #[must_use]
    pub fn to_text(&self) -> String {
        let mut out = String::new();
        for stroke in self.strokes.iter().filter(|s| !s.is_empty()) {
            let line: Vec<String> = stroke
                .iter()
                .map(|p| format!("{:.4},{:.4}", p.x, p.y))
                .collect();
            out.push_str(&line.join(" ")); // ui-text-exempt: the on-disk point separator, never displayed
            out.push('\n');
        }
        out
    }

    /// Read [`Self::to_text`]'s form. `None` when any pair is malformed or
    /// non-finite, so a damaged file is ignored rather than half-drawn.
    #[must_use]
    pub fn from_text(text: &str) -> Option<Self> {
        let mut strokes = Vec::new();
        for line in text.lines().map(str::trim).filter(|l| !l.is_empty()) {
            let mut stroke = Vec::new();
            for pair in line.split_whitespace() {
                let (x, y) = pair.split_once(',')?;
                let (x, y) = (x.parse::<f32>().ok()?, y.parse::<f32>().ok()?);
                if !x.is_finite() || !y.is_finite() {
                    return None;
                }
                stroke.push(pos2(x, y));
            }
            strokes.push(stroke);
        }
        let mark = Self { strokes };
        (!mark.is_empty()).then_some(mark)
    }
}

/// Ramer–Douglas–Peucker on one polyline. Keeps both ends; a stroke of one
/// or two points comes back unchanged.
fn simplify(points: &[Pos2], tolerance: f32) -> Vec<Pos2> {
    if points.len() < 3 {
        return points.to_vec();
    }
    let mut keep = vec![false; points.len()];
    keep[0] = true;
    keep[points.len() - 1] = true;
    let mut stack = vec![(0, points.len() - 1)];
    while let Some((a, b)) = stack.pop() {
        let (mut far, mut far_d) = (a, 0.0_f32);
        for i in a + 1..b {
            let d = distance_to_segment(points[i], points[a], points[b]);
            if d > far_d {
                far = i;
                far_d = d;
            }
        }
        if far_d > tolerance {
            keep[far] = true;
            stack.push((a, far));
            stack.push((far, b));
        }
    }
    points
        .iter()
        .zip(keep)
        .filter_map(|(p, k)| k.then_some(*p))
        .collect()
}

fn distance_to_segment(p: Pos2, a: Pos2, b: Pos2) -> f32 {
    let ab = b - a;
    let len2 = ab.length_sq();
    if len2 <= f32::EPSILON {
        return (p - a).length();
    }
    let t = ((p - a).dot(ab) / len2).clamp(0.0, 1.0);
    (p - (a + ab * t)).length()
}

/// A mark placed into a box: points in the box's space, and the pen width.
#[derive(Clone, Debug, PartialEq)]
pub struct Fitted {
    /// Each stroke's points, in the target's space. A dot is two equal points
    /// so a round-capped stroke draws it.
    pub strokes: Vec<Vec<Pos2>>,
    /// The pen width, in the target's units.
    pub width: f32,
}

/// Fill fraction of the box width, and of the permitted height.
const FILL: f32 = 0.95;
/// A signature may rise to this many box heights — Acrobat Fill & Sign's
/// behaviour on a short box, approved by the operator.
const RISE: f32 = 2.0;
/// Below this fraction of the box height a mark is centred vertically.
const CENTRE_BELOW: f32 = 0.9;
/// Left inset, as a fraction of the box width.
const INSET: f32 = 0.03;
/// Pen width as a fraction of the placed mark's height, and its limits in
/// target units (points, for a canvas-space target).
const PEN_FRACTION: f32 = 0.04;
const PEN_MIN: f32 = 0.6;
const PEN_MAX: f32 = 2.5;

/// Place `mark` in `target` (y-down): scaled uniformly to 95 % of the box
/// width or of twice its height, whichever binds; left-aligned with a 3 %
/// inset; centred vertically when it fits in 90 % of the height, otherwise
/// standing on the box's lower edge and rising above it. `None` for a mark
/// with no extent or a degenerate target.
#[must_use]
pub fn fit(mark: &Mark, target: Rect) -> Option<Fitted> {
    if !mark.has_extent() || !(target.width() > 0.0 && target.height() > 0.0) {
        return None;
    }
    let ink = mark.bounds()?;
    let (w, h) = (target.width(), target.height());
    let by_width = (ink.width() > 0.0).then(|| FILL * w / ink.width());
    let by_height = (ink.height() > 0.0).then(|| FILL * RISE * h / ink.height());
    let scale = match (by_width, by_height) {
        (Some(a), Some(b)) => a.min(b),
        (Some(a), None) | (None, Some(a)) => a,
        (None, None) => return None,
    };
    let placed_h = ink.height() * scale;
    let top = if placed_h <= CENTRE_BELOW * h {
        target.min.y + (h - placed_h) / 2.0
    } else {
        target.max.y - (1.0 - FILL) * h - placed_h
    };
    let left = target.min.x + INSET * w;
    let strokes = mark
        .strokes
        .iter()
        .filter(|s| !s.is_empty())
        .map(|s| {
            let mut out: Vec<Pos2> = s
                .iter()
                .map(|p| {
                    pos2(
                        left + (p.x - ink.min.x) * scale,
                        top + (p.y - ink.min.y) * scale,
                    )
                })
                .collect();
            if out.len() == 1 {
                out.push(out[0]);
            }
            out
        })
        .collect();
    let width = (placed_h * PEN_FRACTION).clamp(PEN_MIN, PEN_MAX);
    Some(Fitted { strokes, width })
}

/// Where a remembered signature lives, or `None` when this install has
/// nowhere to keep one.
#[must_use]
pub fn saved_path() -> Option<PathBuf> {
    pdfcer_core::settings::resolve_store()
        .directory()
        .map(|dir| dir.join(SAVED_FILE))
}

/// The remembered signature, if one is kept and readable.
#[must_use]
pub fn load_saved() -> Option<Mark> {
    let text = std::fs::read_to_string(saved_path()?).ok()?;
    Mark::from_text(&text)
}

/// Keep `mark` (normalised) on this computer. Returns whether it was written.
pub fn save(mark: &Mark) -> bool {
    let Some(path) = saved_path() else {
        return false;
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(&path, mark.normalised().to_text()).is_ok()
}

/// Delete the remembered signature. Absent already is success.
pub fn forget() -> bool {
    match saved_path() {
        Some(path) => match std::fs::remove_file(path) {
            Ok(()) => true,
            Err(e) => e.kind() == std::io::ErrorKind::NotFound,
        },
        None => true,
    }
}

/// Whether a placement is applied, or sits on the redo stack.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Applied,
    Undone,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Entry {
    field: String,
    /// The undo depth right after the placement committed: the placement is
    /// applied exactly while the undo depth is at least this.
    depth: usize,
    state: State,
}

/// **Which signature fields this session has hand-signed.**
///
/// A hand signature is page content and leaves the `/Sig` field empty, so
/// nothing in the document says the box is signed. This ledger follows each
/// placement through the command log by undo depth:
///
/// - [`Self::placed`] after the placement commits;
/// - [`Self::undone`] / [`Self::redone`] after each history step, with the
///   new undo depth;
/// - [`Self::reconcile`] every frame and before each history step: with the
///   redo stack empty nothing can be on it, so every undone placement is gone.
///
/// The redo case cannot be read from depths alone (a new edit and a redo of
/// the last undone command both leave `undo + 1, redo 0` when one command was
/// undone), which is why history steps report themselves.
///
/// Known limit: once the engine's undo stack reaches its cap and evicts its
/// oldest command, recorded depths are one too high per eviction. The ledger
/// ends with the session; the engine request G073 replaces it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Ledger {
    entries: Vec<Entry>,
}

impl Ledger {
    /// Record a placement on `field` that left the undo stack at `depth`.
    pub fn placed(&mut self, field: &str, depth: usize) {
        self.reconcile(0);
        self.entries.push(Entry {
            field: field.to_owned(),
            depth,
            state: State::Applied,
        });
    }

    /// After an undo left the stack at `undo_depth`.
    pub fn undone(&mut self, undo_depth: usize) {
        for e in &mut self.entries {
            if e.state == State::Applied && e.depth > undo_depth {
                e.state = State::Undone;
            }
        }
    }

    /// After a redo left the stack at `undo_depth`.
    pub fn redone(&mut self, undo_depth: usize) {
        for e in &mut self.entries {
            if e.state == State::Undone && e.depth <= undo_depth {
                e.state = State::Applied;
            }
        }
    }

    /// Drop undone placements when the redo stack is empty.
    pub fn reconcile(&mut self, redo_depth: usize) {
        if redo_depth == 0 {
            self.entries.retain(|e| e.state == State::Applied);
        }
    }

    /// Whether `field` carries an applied hand signature.
    #[must_use]
    pub fn is_signed(&self, field: &str) -> bool {
        self.entries
            .iter()
            .any(|e| e.state == State::Applied && e.field == field)
    }

    /// How many distinct fields are hand-signed.
    #[must_use]
    pub fn signed_count(&self) -> usize {
        let mut fields: Vec<&str> = self
            .entries
            .iter()
            .filter(|e| e.state == State::Applied)
            .map(|e| e.field.as_str())
            .collect();
        fields.sort_unstable();
        fields.dedup();
        fields.len()
    }
}

#[cfg(test)]
mod tests;
