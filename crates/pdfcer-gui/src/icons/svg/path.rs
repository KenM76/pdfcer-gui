//! # icons::svg::path — the SVG path-data grammar
//!
//! One `d` attribute in, a stroked/fillable `tiny_skia::Path` out. Split out
//! of [`super`] because the two halves are genuinely different subjects: the
//! parent is an *element scanner* that walks tags and attributes, and this is
//! a *grammar* with its own lexer, its own state machine and its own
//! numerical geometry. They are also the two halves that grow independently
//! — a new element type touches only the parent, a new path command touches
//! only this file — and keeping them in one 1,400-line module was already
//! within a hundred lines of the project's 1,500-line limit.
//!
//! ## The grammar implemented here
//!
//! The complete SVG path grammar: `M m L l H h V v C c S s Q q T t A a Z z`.
//! Implementing all of it rather than only the commands today's assets happen
//! to use costs a few dozen lines and removes a whole class of future failure
//! — an icon redrawn with a smooth-quadratic `T` two years from now must not
//! become a build break. Anything that is not one of those letters is
//! [`IconError::UnsupportedPathCommand`], never a skip.
//!
//! ## ★ Two lexing rules that look like details and are not
//!
//! * **Number extent is computed, not delegated.** `1.5.5` is two numbers,
//!   `1-2` is two numbers, and `M6 14h12l4 4` has no separators at all.
//!   Handing a slice to `f32::from_str` requires already knowing where the
//!   number ends, which is the hard part — and getting it wrong is exactly
//!   the "silently draws the wrong glyph" failure this whole module exists to
//!   avoid. See [`PathLexer::take_number`].
//! * **An arc flag is ONE character, never a number.** `link.svg` is written
//!   `a6 6 0 008 8`, where `008` is large-arc=0, sweep=0, x=8. A number lexer
//!   would swallow `008` as the single value 8 and draw a wildly wrong chain
//!   link. See [`PathLexer::take_flag`], and `parses_packed_arc_flags`.
//!
//! ## Arcs
//!
//! `tiny-skia` has no arc primitive, so elliptical-arc segments are converted
//! to cubic Béziers by [`arc_to_cubics`], following the endpoint→centre
//! parameterization in the SVG 1.1 implementation notes (F.6.5) and the
//! ≤90°-per-segment subdivision of F.6.6.

use pdfcer_render::tiny_skia::PathBuilder;

use super::IconError;

/// A cursor over an SVG path `d` string.
///
/// Byte-oriented because the grammar is pure ASCII; any non-ASCII byte can
/// only be a stray character and will fail the command/number lex rather
/// than being mis-sliced.
struct PathLexer<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> PathLexer<'a> {
    fn new(s: &'a str) -> Self {
        Self {
            bytes: s.as_bytes(),
            pos: 0,
        }
    }

    /// Skip whitespace and the optional comma separator. SVG treats commas
    /// and whitespace interchangeably, and permits neither at all when the
    /// tokens are unambiguous (`M6 14h12l4 4`), so this is called before
    /// every token and is allowed to consume nothing.
    ///
    /// `\r` is in the set alongside `\n`, and that is not decoration: with
    /// `core.autocrlf=true` a fresh clone gets CRLF assets, and a scanner
    /// that treated `\r` as a stray byte would ship blank icons on exactly
    /// the machines that had never seen the repository before.
    fn skip_separators(&mut self) {
        while self.pos < self.bytes.len() {
            match self.bytes[self.pos] {
                b' ' | b'\t' | b'\r' | b'\n' | b',' => self.pos += 1,
                _ => break,
            }
        }
    }

    fn at_end(&mut self) -> bool {
        self.skip_separators();
        self.pos >= self.bytes.len()
    }

    /// Peek the next byte without consuming, after separators.
    fn peek(&mut self) -> Option<u8> {
        self.skip_separators();
        self.bytes.get(self.pos).copied()
    }

    /// Consume a command letter.
    fn take_command(&mut self) -> Option<char> {
        let b = self.peek()?;
        if b.is_ascii_alphabetic() {
            self.pos += 1;
            Some(b as char)
        } else {
            None
        }
    }

    /// Lex one number:
    /// `[+-]? ( digits [. digits?] | . digits ) ( [eE] [+-]? digits )?`.
    ///
    /// Written by hand rather than by handing a slice to `f32::from_str`
    /// because the *extent* of the number is the hard part: `1.5.5` is two
    /// numbers, `1-2` is two numbers, and `M6 14h12l4 4` has no separators at
    /// all. Getting the extent wrong is precisely the "silently draws the
    /// wrong glyph" failure this module exists to avoid, so the extent is
    /// computed explicitly and only then handed to `from_str`.
    fn take_number(&mut self) -> Result<f32, IconError> {
        self.skip_separators();
        let start = self.pos;
        if self.pos < self.bytes.len()
            && (self.bytes[self.pos] == b'+' || self.bytes[self.pos] == b'-')
        {
            self.pos += 1;
        }
        let mut digits = false;
        while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_digit() {
            self.pos += 1;
            digits = true;
        }
        if self.pos < self.bytes.len() && self.bytes[self.pos] == b'.' {
            self.pos += 1;
            while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_digit() {
                self.pos += 1;
                digits = true;
            }
        }
        if !digits {
            return Err(IconError::MalformedNumber { offset: start });
        }
        // Exponent, only if it is actually well formed — a trailing `e` that
        // is not followed by digits belongs to the next token, not this
        // number.
        if self.pos < self.bytes.len() && (self.bytes[self.pos] | 0x20) == b'e' {
            let save = self.pos;
            self.pos += 1;
            if self.pos < self.bytes.len()
                && (self.bytes[self.pos] == b'+' || self.bytes[self.pos] == b'-')
            {
                self.pos += 1;
            }
            let exp_start = self.pos;
            while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_digit() {
                self.pos += 1;
            }
            if self.pos == exp_start {
                self.pos = save;
            }
        }
        let text = std::str::from_utf8(&self.bytes[start..self.pos])
            .map_err(|_| IconError::MalformedNumber { offset: start })?;
        text.parse::<f32>()
            .map_err(|_| IconError::MalformedNumber { offset: start })
    }

    /// Lex an arc flag: exactly ONE `0` or `1` character.
    ///
    /// This is not a number lex, and the difference is load-bearing.
    /// `link.svg` is written `a6 6 0 008 8`, where `008` is large-arc=0,
    /// sweep=0, x=8. A number lexer would swallow `008` as the single value
    /// 8 and draw a wildly wrong chain link.
    fn take_flag(&mut self) -> Result<bool, IconError> {
        self.skip_separators();
        match self.bytes.get(self.pos) {
            Some(b'0') => {
                self.pos += 1;
                Ok(false)
            }
            Some(b'1') => {
                self.pos += 1;
                Ok(true)
            }
            Some(_) => Err(IconError::MalformedFlag { offset: self.pos }),
            None => Err(IconError::UnexpectedEnd),
        }
    }
}

/// Parse an SVG path `d` string into `builder`.
///
/// Implements the full path grammar (module header). Three pieces of state
/// make the whole thing work and are worth naming explicitly:
///
/// * `cur` — the current point. Relative commands are offsets from it, and a
///   command that needs it before any `M` is [`IconError::NoCurrentPoint`]
///   rather than an implicit origin, because an implicit origin silently
///   draws a glyph anchored at the viewBox corner.
/// * `start` — the current subpath's first point, which `Z` returns to and
///   which a command *after* a `Z` continues from (SVG's rule, and the one
///   most often got wrong).
/// * `cubic_reflect` / `quad_reflect` — the previous cubic/quadratic control
///   point, mirrored on demand by the smooth forms `S`/`T`. Reset to `None`
///   after any non-curve command, per the spec: `S` after an `L` is a plain
///   curve, not a reflection of something three commands ago.
pub(super) fn parse_path_data(d: &str, builder: &mut PathBuilder) -> Result<(), IconError> {
    let mut lex = PathLexer::new(d);
    let mut cur = (0.0f32, 0.0f32);
    let mut start = (0.0f32, 0.0f32);
    let mut cubic_reflect: Option<(f32, f32)> = None;
    let mut quad_reflect: Option<(f32, f32)> = None;
    let mut have_current = false;
    let mut command: Option<char> = None;

    loop {
        if lex.at_end() {
            break;
        }
        // A command letter may be omitted for repeated parameter sets
        // ("M10 10 20 20" is a moveto then an implicit lineto; "h12 4" is two
        // horizontal linetos). When the next token is not a letter we reuse
        // the previous command — with SVG's one special case that a repeated
        // `M`/`m` becomes `L`/`l`.
        if let Some(c) = lex.peek() {
            if c.is_ascii_alphabetic() {
                command = lex.take_command();
            } else {
                command = match command {
                    Some('M') => Some('L'),
                    Some('m') => Some('l'),
                    other => other,
                };
            }
        }
        let Some(c) = command else {
            return Err(IconError::UnexpectedEnd);
        };

        let relative = c.is_ascii_lowercase();
        let base = if relative { cur } else { (0.0, 0.0) };

        // Every command except M/m needs a current point.
        if !have_current && !matches!(c, 'M' | 'm') {
            return Err(IconError::NoCurrentPoint(c));
        }

        match c {
            'M' | 'm' => {
                let x = lex.take_number()? + base.0;
                let y = lex.take_number()? + base.1;
                builder.move_to(x, y);
                cur = (x, y);
                start = (x, y);
                have_current = true;
                cubic_reflect = None;
                quad_reflect = None;
            }
            'L' | 'l' => {
                let x = lex.take_number()? + base.0;
                let y = lex.take_number()? + base.1;
                builder.line_to(x, y);
                cur = (x, y);
                cubic_reflect = None;
                quad_reflect = None;
            }
            'H' | 'h' => {
                let x = lex.take_number()? + base.0;
                builder.line_to(x, cur.1);
                cur = (x, cur.1);
                cubic_reflect = None;
                quad_reflect = None;
            }
            'V' | 'v' => {
                let y = lex.take_number()? + base.1;
                builder.line_to(cur.0, y);
                cur = (cur.0, y);
                cubic_reflect = None;
                quad_reflect = None;
            }
            'C' | 'c' => {
                let x1 = lex.take_number()? + base.0;
                let y1 = lex.take_number()? + base.1;
                let x2 = lex.take_number()? + base.0;
                let y2 = lex.take_number()? + base.1;
                let x = lex.take_number()? + base.0;
                let y = lex.take_number()? + base.1;
                builder.cubic_to(x1, y1, x2, y2, x, y);
                cur = (x, y);
                cubic_reflect = Some((x2, y2));
                quad_reflect = None;
            }
            'S' | 's' => {
                let (x1, y1) = match cubic_reflect {
                    Some((px, py)) => (2.0 * cur.0 - px, 2.0 * cur.1 - py),
                    None => cur,
                };
                let x2 = lex.take_number()? + base.0;
                let y2 = lex.take_number()? + base.1;
                let x = lex.take_number()? + base.0;
                let y = lex.take_number()? + base.1;
                builder.cubic_to(x1, y1, x2, y2, x, y);
                cur = (x, y);
                cubic_reflect = Some((x2, y2));
                quad_reflect = None;
            }
            'Q' | 'q' => {
                let x1 = lex.take_number()? + base.0;
                let y1 = lex.take_number()? + base.1;
                let x = lex.take_number()? + base.0;
                let y = lex.take_number()? + base.1;
                builder.quad_to(x1, y1, x, y);
                cur = (x, y);
                quad_reflect = Some((x1, y1));
                cubic_reflect = None;
            }
            'T' | 't' => {
                let (x1, y1) = match quad_reflect {
                    Some((px, py)) => (2.0 * cur.0 - px, 2.0 * cur.1 - py),
                    None => cur,
                };
                let x = lex.take_number()? + base.0;
                let y = lex.take_number()? + base.1;
                builder.quad_to(x1, y1, x, y);
                cur = (x, y);
                quad_reflect = Some((x1, y1));
                cubic_reflect = None;
            }
            'A' | 'a' => {
                let rx = lex.take_number()?;
                let ry = lex.take_number()?;
                let rot = lex.take_number()?;
                let large_arc = lex.take_flag()?;
                let sweep = lex.take_flag()?;
                let x = lex.take_number()? + base.0;
                let y = lex.take_number()? + base.1;
                match arc_to_cubics(cur, rx, ry, rot, large_arc, sweep, (x, y)) {
                    Some(segments) => {
                        for [x1, y1, x2, y2, ex, ey] in segments {
                            builder.cubic_to(x1, y1, x2, y2, ex, ey);
                        }
                    }
                    // SVG: a zero radius (or a zero-length arc) degenerates
                    // to a straight line rather than being an error.
                    None => builder.line_to(x, y),
                }
                cur = (x, y);
                cubic_reflect = None;
                quad_reflect = None;
            }
            'Z' | 'z' => {
                builder.close();
                cur = start;
                cubic_reflect = None;
                quad_reflect = None;
            }
            other => return Err(IconError::UnsupportedPathCommand(other)),
        }
    }

    Ok(())
}

/// Convert one SVG elliptical-arc segment to a list of cubic Béziers.
///
/// Implements the endpoint→centre parameterization of the SVG 1.1
/// implementation notes F.6.5, then subdivides the swept angle into segments
/// of at most 90° (F.6.6) because a single cubic cannot approximate a larger
/// arc acceptably — the 270° arcs in `undo.svg` and `rotate-ccw.svg` become
/// three cubics each.
///
/// Returns `None` for the degenerate cases the spec says to treat as a
/// straight line: either radius zero, or coincident endpoints.
///
/// Parameters mirror the SVG grammar exactly: `from`/`to` are endpoints,
/// `rx`/`ry` radii, `rot_deg` the x-axis rotation, and the two booleans the
/// large-arc and sweep flags. Radii are enlarged (never shrunk) when they are
/// too small to span the endpoints, per F.6.6 step 3 — otherwise `sqrt` of a
/// negative number would silently produce NaN geometry.
///
/// The arithmetic is `f64` throughout even though the geometry is `f32`: the
/// centre parameterization takes a difference of squared radii, which is
/// where `f32` loses the digits that matter, and the result is rounded back
/// to `f32` only at the end.
#[allow(clippy::too_many_arguments)]
fn arc_to_cubics(
    from: (f32, f32),
    rx: f32,
    ry: f32,
    rot_deg: f32,
    large_arc: bool,
    sweep: bool,
    to: (f32, f32),
) -> Option<Vec<[f32; 6]>> {
    let (x1, y1) = (from.0 as f64, from.1 as f64);
    let (x2, y2) = (to.0 as f64, to.1 as f64);
    let mut rx = (rx as f64).abs();
    let mut ry = (ry as f64).abs();
    if rx == 0.0 || ry == 0.0 || ((x1 - x2).abs() < f64::EPSILON && (y1 - y2).abs() < f64::EPSILON)
    {
        return None;
    }
    let phi = (rot_deg as f64).to_radians();
    let (sin_phi, cos_phi) = phi.sin_cos();

    // F.6.5.1 — endpoints into the rotated, midpoint-centred frame.
    let dx = (x1 - x2) / 2.0;
    let dy = (y1 - y2) / 2.0;
    let x1p = cos_phi * dx + sin_phi * dy;
    let y1p = -sin_phi * dx + cos_phi * dy;

    // F.6.6.2 — grow radii if they cannot span the chord.
    let lambda = (x1p * x1p) / (rx * rx) + (y1p * y1p) / (ry * ry);
    if lambda > 1.0 {
        let s = lambda.sqrt();
        rx *= s;
        ry *= s;
    }

    // F.6.5.2 — the centre in the rotated frame.
    let num = (rx * rx * ry * ry) - (rx * rx * y1p * y1p) - (ry * ry * x1p * x1p);
    let den = (rx * rx * y1p * y1p) + (ry * ry * x1p * x1p);
    let sign = if large_arc == sweep { -1.0 } else { 1.0 };
    let coef = sign * (num / den).max(0.0).sqrt();
    let cxp = coef * rx * y1p / ry;
    let cyp = -coef * ry * x1p / rx;

    // F.6.5.3 — back to user space.
    let cx = cos_phi * cxp - sin_phi * cyp + (x1 + x2) / 2.0;
    let cy = sin_phi * cxp + cos_phi * cyp + (y1 + y2) / 2.0;

    // F.6.5.5/6 — start angle and swept angle.
    let ux = (x1p - cxp) / rx;
    let uy = (y1p - cyp) / ry;
    let vx = (-x1p - cxp) / rx;
    let vy = (-y1p - cyp) / ry;
    let theta1 = angle_between(1.0, 0.0, ux, uy);
    let mut delta = angle_between(ux, uy, vx, vy);
    if !sweep && delta > 0.0 {
        delta -= std::f64::consts::TAU;
    } else if sweep && delta < 0.0 {
        delta += std::f64::consts::TAU;
    }

    // F.6.6 — subdivide into <=90 degree pieces.
    let count = (delta.abs() / std::f64::consts::FRAC_PI_2).ceil().max(1.0) as usize;
    let step = delta / count as f64;
    // The tangent-scaling factor for a cubic approximating a `step`-wide arc.
    // At step = 90 degrees this reduces to KAPPA.
    let alpha = 4.0 / 3.0 * (step / 4.0).tan();

    let point_at = |t: f64| -> (f64, f64) {
        let (s, c) = t.sin_cos();
        (
            cx + rx * c * cos_phi - ry * s * sin_phi,
            cy + rx * c * sin_phi + ry * s * cos_phi,
        )
    };
    let deriv_at = |t: f64| -> (f64, f64) {
        let (s, c) = t.sin_cos();
        (
            -rx * s * cos_phi - ry * c * sin_phi,
            -rx * s * sin_phi + ry * c * cos_phi,
        )
    };

    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        let t1 = theta1 + step * i as f64;
        let t2 = t1 + step;
        let (p1x, p1y) = point_at(t1);
        let (p2x, p2y) = point_at(t2);
        let (d1x, d1y) = deriv_at(t1);
        let (d2x, d2y) = deriv_at(t2);
        out.push([
            (p1x + alpha * d1x) as f32,
            (p1y + alpha * d1y) as f32,
            (p2x - alpha * d2x) as f32,
            (p2y - alpha * d2y) as f32,
            (p2x) as f32,
            (p2y) as f32,
        ]);
    }
    // Snap the final endpoint back onto the requested one. The trigonometric
    // round trip lands within ~1e-6 of it, which is invisible, but an exact
    // match keeps the following command's relative offsets exact and makes
    // `Z` close cleanly.
    if let Some(last) = out.last_mut() {
        last[4] = to.0;
        last[5] = to.1;
    }
    Some(out)
}

/// Signed angle from vector *(ux, uy)* to *(vx, vy)*, in radians, in
/// (-pi, pi]. SVG F.6.5.4.
fn angle_between(ux: f64, uy: f64, vx: f64, vy: f64) -> f64 {
    let dot = ux * vx + uy * vy;
    let len = ((ux * ux + uy * uy) * (vx * vx + vy * vy)).sqrt();
    if len == 0.0 {
        return 0.0;
    }
    let mut a = (dot / len).clamp(-1.0, 1.0).acos();
    if ux * vy - uy * vx < 0.0 {
        a = -a;
    }
    a
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a path and report how many verbs it produced — the cheapest
    /// proxy for "the parser understood the command" that does not depend on
    /// tiny-skia's internal point layout.
    fn verbs(d: &str) -> Result<usize, IconError> {
        let mut pb = PathBuilder::new();
        parse_path_data(d, &mut pb)?;
        Ok(pb.len())
    }

    // -- the path grammar -------------------------------------------------

    #[test]
    fn parses_absolute_and_relative_move_and_line() {
        // M + L, M + l, and the implicit-lineto-after-moveto rule.
        assert_eq!(verbs("M10 10L20 20").unwrap(), 2);
        assert_eq!(verbs("M10 10l10 10").unwrap(), 2);
        assert_eq!(verbs("M10 10 20 20 30 30").unwrap(), 3);
    }

    #[test]
    fn parses_horizontal_and_vertical() {
        assert_eq!(verbs("M6 14h12v22H6V14").unwrap(), 5);
    }

    #[test]
    fn parses_cubic_and_smooth_cubic() {
        assert_eq!(verbs("M10 34C10 18 24 34 38 14").unwrap(), 2);
        assert_eq!(verbs("M0 0C1 1 2 2 3 3S4 4 5 5").unwrap(), 3);
    }

    #[test]
    fn parses_quadratic_and_smooth_quadratic() {
        assert_eq!(verbs("M0 0Q1 1 2 2").unwrap(), 2);
        assert_eq!(verbs("M0 0Q1 1 2 2T4 4").unwrap(), 3);
    }

    #[test]
    fn parses_close() {
        // M, L, L, Z => 4 verbs.
        assert_eq!(verbs("M10 10L20 10L20 20Z").unwrap(), 4);
    }

    #[test]
    fn parses_arc_as_cubics() {
        // A 270 degree arc subdivides into three <=90 degree cubics, so
        // move + 3 cubics = 4 verbs. This is the exact construction undo.svg
        // uses.
        assert_eq!(verbs("M24 10A14 14 0 1 0 38 24").unwrap(), 4);
    }

    /// The packed-flag form that appears verbatim in `link.svg`. A number
    /// lexer would read `008` as `8` and silently draw a wrong glyph; this
    /// asserts the flags are lexed one character at a time.
    #[test]
    fn parses_packed_arc_flags() {
        let mut pb = PathBuilder::new();
        parse_path_data("M16 24l-4 4a6 6 0 008 8l4-4", &mut pb).expect("packed flags parse");
        // move, line, (arc -> >=1 cubic), line.
        assert!(pb.len() >= 4);

        // And the same arc written with separated flags must agree.
        let mut spaced = PathBuilder::new();
        parse_path_data("M16 24l-4 4a6 6 0 0 0 8 8l4-4", &mut spaced).expect("spaced flags parse");
        assert_eq!(pb.len(), spaced.len());
    }

    #[test]
    fn parses_negative_and_fractional_numbers_without_separators() {
        // "-2" terminates the previous number; ".5" needs no leading zero.
        assert_eq!(verbs("M8 10l2-2l.5.5").unwrap(), 3);
    }

    #[test]
    fn parses_exponent_numbers() {
        assert_eq!(verbs("M1e1 1E1L2e1 2e1").unwrap(), 2);
    }

    // -- refusal (never silent mis-drawing) --------------------------------

    #[test]
    fn refuses_unknown_path_command() {
        assert_eq!(
            verbs("M0 0 X10 10").unwrap_err(),
            IconError::UnsupportedPathCommand('X')
        );
    }

    #[test]
    fn refuses_command_before_any_moveto() {
        assert_eq!(verbs("L10 10").unwrap_err(), IconError::NoCurrentPoint('L'));
    }

    #[test]
    fn refuses_malformed_number() {
        // `M` needs two numbers; the second is not one.
        assert!(matches!(
            verbs("M10 abc").unwrap_err(),
            IconError::MalformedNumber { .. }
        ));
        // A lone sign is not a number.
        assert!(matches!(
            verbs("M10 -").unwrap_err(),
            IconError::MalformedNumber { .. }
        ));
    }

    #[test]
    fn refuses_truncated_command() {
        assert!(matches!(
            verbs("M10 10L20").unwrap_err(),
            IconError::MalformedNumber { .. } | IconError::UnexpectedEnd
        ));
    }

    #[test]
    fn refuses_bad_arc_flag() {
        assert!(matches!(
            verbs("M0 0A6 6 0 2 0 8 8").unwrap_err(),
            IconError::MalformedFlag { .. }
        ));
    }
}
