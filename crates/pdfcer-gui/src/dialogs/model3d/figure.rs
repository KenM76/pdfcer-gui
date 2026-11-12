//! The figure seen from behind: its parts as made, read from the `.figure`
//! files beside this module, and fitted to the operator's measures.
//!
//! A `.figure` file is text, metres, Z up, facing +Y, soles at z = 0:
//!
//! ```text
//! figure <name> eye <eye height>
//! part <name> <parent|-> <pivot x y z>     pivot absolute; a parent comes first
//! v x y z                                  relative to the part's pivot
//! f i j k r g b                            0-based in the part, CCW from outside
//! ```
//!
//! `tools/figures/make_figures.py` writes them from Blender. A part is posed
//! by turning it about its pivot (see [`super::animate`]); its role comes from
//! its name, `_l` being the figure's left (−X).

use std::sync::OnceLock;

use egui::Color32;

/// Which side of the figure a limb is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Side {
    Left,
    Right,
}

/// What a part is, for posing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Role {
    Hips,
    Torso,
    Head,
    UpperArm(Side),
    ForeArm(Side),
    Hand(Side),
    Thigh(Side),
    Shin(Side),
    Foot(Side),
    Tail,
    /// A part posing leaves as made.
    Other,
}

impl Role {
    fn of(name: &str) -> Self {
        let (stem, side) = match name.rsplit_once('_') {
            Some((stem, "l")) => (stem, Some(Side::Left)),
            Some((stem, "r")) => (stem, Some(Side::Right)),
            _ => (name, None),
        };
        match (stem, side) {
            ("hips", None) => Self::Hips,
            ("torso", None) => Self::Torso,
            ("head", None) => Self::Head,
            ("tail", None) => Self::Tail,
            ("upper_arm", Some(s)) => Self::UpperArm(s),
            ("fore_arm", Some(s)) => Self::ForeArm(s),
            ("hand", Some(s)) => Self::Hand(s),
            ("thigh", Some(s)) => Self::Thigh(s),
            ("shin", Some(s)) => Self::Shin(s),
            ("foot", Some(s)) => Self::Foot(s),
            _ => Self::Other,
        }
    }
}

/// One rigid part.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Part {
    pub role: Role,
    /// Index of the parent part, which comes earlier.
    pub parent: Option<usize>,
    /// The joint it turns about, absolute.
    pub pivot: [f64; 3],
    /// Corners relative to the pivot.
    pub verts: Vec<[f64; 3]>,
    pub faces: Vec<([usize; 3], Color32)>,
}

/// A figure, metres.
#[derive(Debug, Clone, PartialEq, Default)]
pub(super) struct Figure {
    pub parts: Vec<Part>,
}

/// Which of the built-in figures is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum Kind {
    #[default]
    Human,
    Werewolf,
}

fn numbers<const N: usize>(words: &[&str]) -> Result<[f64; N], String> {
    let mut out = [0.0; N];
    for (slot, word) in out.iter_mut().zip(words) {
        *slot = word.parse().map_err(|_| format!("not a number: {word}"))?; // ui-text-exempt: a built-in figure file's parse error, refused by a test
    }
    if words.len() < N {
        return Err(format!("{} numbers, {N} wanted", words.len())); // ui-text-exempt: a built-in figure file's parse error, refused by a test
    }
    Ok(out)
}

fn face(words: &[&str], corners: usize) -> Result<([usize; 3], Color32), String> {
    let [i, j, k, r, g, b] = numbers::<6>(words)?;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let index = |v: f64| -> Result<usize, String> {
        let n = v as usize;
        if v < 0.0 || n >= corners {
            return Err(format!("corner {v} of {corners}")); // ui-text-exempt: a built-in figure file's parse error, refused by a test
        }
        Ok(n)
    };
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let channel = |v: f64| v.clamp(0.0, 255.0) as u8;
    Ok((
        [index(i)?, index(j)?, index(k)?],
        Color32::from_rgb(channel(r), channel(g), channel(b)), // NOT A THEME COLOUR: the figure model's own paint
    ))
}

/// Read a `.figure` file.
pub(super) fn parse(text: &str) -> Result<Figure, String> {
    let mut names: Vec<&str> = Vec::new();
    let mut figure = Figure::default();
    for (n, line) in text.lines().enumerate() {
        let words: Vec<&str> = line.split_whitespace().collect();
        let fail = |why: String| format!("line {}: {why}", n + 1); // ui-text-exempt: a built-in figure file's parse error, refused by a test
        let has_part = !figure.parts.is_empty();
        match words.first().copied() {
            None | Some("figure") => {}
            Some(w) if w.starts_with('#') => {}
            Some("part") if words.len() >= 6 => {
                let parent = match words[2] {
                    "-" => None,
                    p => Some(
                        names
                            .iter()
                            .position(|n| *n == p)
                            .ok_or_else(|| fail(format!("no part {p} before")))?, // ui-text-exempt: a built-in figure file's parse error, refused by a test
                    ),
                };
                figure.parts.push(Part {
                    role: Role::of(words[1]),
                    parent,
                    pivot: numbers::<3>(&words[3..]).map_err(fail)?,
                    verts: Vec::new(),
                    faces: Vec::new(),
                });
                names.push(words[1]);
            }
            Some(kind @ ("v" | "f")) if has_part => {
                let Some(part) = figure.parts.last_mut() else {
                    continue;
                };
                if kind == "v" {
                    part.verts.push(numbers::<3>(&words[1..]).map_err(fail)?);
                } else {
                    let f = face(&words[1..], part.verts.len()).map_err(fail)?;
                    part.faces.push(f);
                }
            }
            _ => return Err(fail(format!("not understood: {line}"))), // ui-text-exempt: a built-in figure file's parse error, refused by a test
        }
    }
    Ok(figure)
}

/// The built-in figure `kind`, as made; empty if its file does not read,
/// which a test forbids.
pub(super) fn builtin(kind: Kind) -> &'static Figure {
    static HUMAN: OnceLock<Figure> = OnceLock::new();
    static WEREWOLF: OnceLock<Figure> = OnceLock::new();
    let (cell, text) = match kind {
        Kind::Human => (&HUMAN, include_str!("figures/human.figure")),
        Kind::Werewolf => (&WEREWOLF, include_str!("figures/werewolf.figure")),
    };
    cell.get_or_init(|| parse(text).unwrap_or_default())
}

/// The range of each proportion.
pub(super) const MEASURE: std::ops::RangeInclusive<f64> = 0.5..=2.0;

/// The operator's proportions and shadows for the figure. Its size is the
/// eye height's: its eyes are at the walker's eye.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Measures {
    pub head: f64,
    pub shoulders: f64,
    pub arms: f64,
    pub legs: f64,
    pub build: f64,
    /// A soft patch under the feet.
    pub blob: bool,
    /// The figure's shape cast on the floor.
    pub cast: bool,
}

impl Default for Measures {
    fn default() -> Self {
        Self {
            head: 1.0,
            shoulders: 1.0,
            arms: 1.0,
            legs: 1.0,
            build: 1.0,
            blob: true,
            cast: false,
        }
    }
}

impl Measures {
    /// The proportions only, as made.
    pub fn as_made(self) -> Self {
        Self {
            blob: self.blob,
            cast: self.cast,
            ..Self::default()
        }
    }

    /// How `role`'s part is stretched along the figure's x, y and z.
    fn stretch(&self, role: Role) -> [f64; 3] {
        let b = self.build;
        match role {
            Role::Head => [self.head; 3],
            Role::Torso => [b * self.shoulders, b, 1.0],
            Role::UpperArm(_) | Role::ForeArm(_) | Role::Hand(_) => [b, b, self.arms],
            Role::Thigh(_) | Role::Shin(_) => [b, b, self.legs],
            Role::Foot(_) => [b, 1.0, 1.0],
            Role::Hips | Role::Tail | Role::Other => [b, b, 1.0],
        }
    }
}

fn mul(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] * b[0], a[1] * b[1], a[2] * b[2]]
}

/// `figure` stretched by `m`: each part's corners by its own stretch and its
/// joint's offset from its parent's by the parent's, then stood on z = 0 and
/// scaled to a height of 1.
pub(super) fn fitted(figure: &Figure, m: &Measures) -> Figure {
    let mut out = figure.clone();
    let stretch: Vec<[f64; 3]> = figure.parts.iter().map(|p| m.stretch(p.role)).collect();
    for i in 0..out.parts.len() {
        let part = &figure.parts[i];
        if let Some(up) = part.parent {
            let offset = std::array::from_fn(|a| part.pivot[a] - figure.parts[up].pivot[a]);
            let moved = mul(stretch[up], offset);
            out.parts[i].pivot = std::array::from_fn(|a| out.parts[up].pivot[a] + moved[a]);
        }
        for v in &mut out.parts[i].verts {
            *v = mul(stretch[i], *v);
        }
    }
    let (low, high) = span(&out);
    let tall = high - low;
    let k = if tall > 1e-6 { 1.0 / tall } else { 1.0 };
    for part in &mut out.parts {
        part.pivot = [
            part.pivot[0] * k,
            part.pivot[1] * k,
            (part.pivot[2] - low) * k,
        ];
        for v in &mut part.verts {
            *v = v.map(|c| c * k);
        }
    }
    out
}

/// The lowest and highest z of `figure`'s corners as made.
pub(super) fn span(figure: &Figure) -> (f64, f64) {
    figure
        .parts
        .iter()
        .flat_map(|p| p.verts.iter().map(move |v| p.pivot[2] + v[2]))
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), z| {
            (lo.min(z), hi.max(z))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_figures_read_with_every_part_posed() {
        for kind in [Kind::Human, Kind::Werewolf] {
            let text = match kind {
                Kind::Human => include_str!("figures/human.figure"),
                Kind::Werewolf => include_str!("figures/werewolf.figure"),
            };
            let figure = parse(text).unwrap();
            assert!(figure.parts.len() >= 15, "{kind:?}");
            assert!(
                figure.parts.iter().all(|p| p.role != Role::Other),
                "{kind:?}"
            );
            let triangles: usize = figure.parts.iter().map(|p| p.faces.len()).sum();
            assert!((300..=2000).contains(&triangles), "{kind:?} {triangles}");
            let (low, _) = span(&figure);
            assert!(low.abs() < 0.02, "{kind:?} soles at {low}");
        }
    }

    #[test]
    fn a_fitted_figure_stands_on_zero_one_tall() {
        let m = Measures {
            legs: 1.5,
            head: 0.7,
            ..Measures::default()
        };
        let fit = fitted(builtin(Kind::Human), &m);
        let (low, high) = span(&fit);
        assert!(
            low.abs() < 1e-9 && (high - 1.0).abs() < 1e-9,
            "{low} {high}"
        );
    }

    #[test]
    fn longer_legs_raise_the_hips_share_of_the_height() {
        let hips = |legs: f64| {
            let fit = fitted(
                builtin(Kind::Human),
                &Measures {
                    legs,
                    ..Measures::default()
                },
            );
            fit.parts
                .iter()
                .find(|p| p.role == Role::Hips)
                .unwrap()
                .pivot[2]
        };
        assert!(hips(1.5) > hips(1.0) + 0.05);
    }

    #[test]
    fn a_file_out_of_order_is_refused() {
        assert!(parse("part a b 0 0 0\n").is_err());
        assert!(parse("part a - 0 0 0\nv 0 0 0\nf 0 0 1 9 9 9\n").is_err());
    }
}
