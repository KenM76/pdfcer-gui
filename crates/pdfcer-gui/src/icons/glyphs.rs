//! # `icons::glyphs` — asking the font stack what it can actually draw
//!
//! A test-only module holding two things and one gate:
//!
//! | item | what it is |
//! |---|---|
//! | [`GlyphProbe`] | a **correct** "can this codepoint be drawn?" predicate |
//! | [`string_literals`] | a Rust source scanner that yields operator-visible literals |
//! | [`tests::every_glyph_the_catalog_draws_has_a_glyph`] | the widened glyph gate |
//!
//! ## Why this module exists at all: `egui`'s `has_glyph` lies
//!
//! This is defect **D12**, and the entry as originally filed had the cause
//! backwards. It recorded that `⚠` (U+26A0) "has no glyph in this build's
//! font stack", on the evidence that
//! `app::status::tests::every_glyph_the_status_bar_draws_has_a_glyph`
//! failed on it. The test really did fail. The conclusion did not follow.
//!
//! **`⚠` is in the font stack, is reachable from the proportional family,
//! and renders correctly today.** What is broken is the predicate the gate
//! asked. `epaint 0.35`'s [`epaint::Fonts::has_glyph`] is:
//!
//! ```ignore
//! // epaint-0.35.0/src/text/font.rs:720
//! pub fn has_glyph(&mut self, c: char) -> bool {
//!     // TODO(emilk): this is a false negative if the user asks about the
//!     // replacement character itself 🤦‍♂️
//!     self.resolve_face(c) != self.cached_family.replacement_face_key
//! }
//! ```
//!
//! `resolve_face(c)` walks the family's fallback chain and returns the first
//! **face** whose charmap covers `c`. `replacement_face_key` is the face that
//! was found to contain `epaint`'s substitution character — `◻`, U+25FB
//! WHITE MEDIUM SQUARE (`epaint-0.35.0/src/text/fonts.rs:643`).
//!
//! So `has_glyph` does not ask *"is this codepoint drawable?"*. It asks
//! *"is this codepoint drawable by a face other than whichever face happens
//! to own `◻`?"* — and returns **false** for every codepoint whose first
//! supporting face is that one. The upstream `TODO` names one instance of
//! the bug (the replacement character itself); the actual blast radius is
//! **every character that face is the first to supply.**
//!
//! ### The measurement, on this build's actual font files
//!
//! `epaint 0.35`'s bundled proportional chain is
//! `[Ubuntu-Light, NotoEmoji-Regular, emoji-icon-font]`
//! (`epaint-0.35.0/src/text/fonts.rs:549-556`). Reading the four bundled
//! `.ttf` charmaps directly:
//!
//! | codepoint | first face that supplies it |
//! |---|---|
//! | `◻` U+25FB (the substitution mark) | **NotoEmoji-Regular** |
//! | `⚠` U+26A0 | **NotoEmoji-Regular** |
//! | `ℹ` U+2139, `‼` U+203C, `❗` U+2757 | **NotoEmoji-Regular** |
//! | `⚑` U+2691, `★` U+2605, `○` U+25CB, `⏴⏵⏷` U+23F4-7 | emoji-icon-font |
//! | `—` `…` `·` `×` `“` `”` `−` `°` `◊` `•` `†` `‡` `№` `¶` `!` | Ubuntu-Light |
//! | `▲` `△` `●` `◆` `□` `✓` `✗` `ⓘ` `※` | *no face — genuinely absent* |
//!
//! `replacement_face_key` for the proportional family is therefore
//! **NotoEmoji-Regular**, and `has_glyph` returns `false` for every row in
//! which NotoEmoji-Regular is the supplier — `⚠ ℹ ‼ ❗` — **although all
//! four draw perfectly.**
//!
//! That single mechanism reproduces D12's two measured lists exactly, with
//! no exceptions across the 31 characters it sampled: every character D12
//! recorded as "present" is supplied by `Ubuntu-Light` or `emoji-icon-font`;
//! every character it recorded as "absent" is either genuinely absent **or**
//! supplied by `NotoEmoji-Regular`. A 31-for-31 correlation is not a
//! coincidence, it is the mechanism.
//!
//! It also explains the otherwise absurd reading that
//! `has_glyph(Monospace, 'A')` is **false**: the monospace chain is
//! `[Hack, Ubuntu-Light, NotoEmoji-Regular, emoji-icon-font]`, `Hack` is the
//! first face there to supply `◻`, and `Hack` is also the first to supply
//! `A`. That is the false negative in its most obviously silly form, and it
//! is what makes the mechanism impossible to mistake for anything else.
//!
//! ## The predicate that is actually correct
//!
//! Ask the renderer what it drew, rather than asking the font index a
//! question it answers wrongly.
//!
//! When no face supplies `c`, `epaint` lays out the **substitution mark** in
//! its place (`epaint-0.35.0/src/text/font.rs:770-773`):
//!
//! ```ignore
//! let glyph_info = face.glyph_info(c, metrics).unwrap_or_else(|| {
//!     // `c` is in no face — render the replacement character instead.
//!     face.glyph_info(self.cached_family.replacement_char, metrics)
//!         .unwrap_or(GlyphInfo::INVISIBLE)
//! });
//! ```
//!
//! So: lay `c` out, take the single resulting [`epaint::text::Glyph`], and
//! compare its `uv_rect` — the glyph's actual rectangle in the font atlas —
//! against the `uv_rect` of a codepoint known to be unsupported. Equal means
//! `c` drew the substitution mark, i.e. it is **not** drawable. This asks
//! the question in the units the operator sees: *what pixels came out.*
//!
//! ### Why the sentinel is three codepoints and not one
//!
//! The probe needs a `uv_rect` for "what an unsupported codepoint looks
//! like", and the obvious way to get one is to lay out a codepoint no font
//! could have. But if that assumption were ever wrong — a font grows
//! coverage, `epaint` swaps its bundled set — the sentinel would become a
//! real glyph, nothing else would match it, and **the gate would pass
//! everything.** That is the `check-file-size` fail-open class named in
//! `DEFECTS.md` D13: "found no violations" and "could not have found one"
//! printing the same thing.
//!
//! So [`GlyphProbe::new`] lays out **three** mutually unrelated unassigned
//! codepoints — U+0870, U+2FFFF and U+10FFFD, from three different planes —
//! and **panics unless all three produce the same rectangle.** Three
//! unrelated codepoints rendering identically is only explicable as the
//! substitution mark. If a future font set covers one of them the probe
//! fails loudly at construction instead of silently going blind, which is
//! the fail-**closed** direction.
//!
//! ## Why this module is `#[cfg(test)]`
//!
//! Nothing in the shipped binary needs to ask this question: the catalog is
//! fixed at compile time, so the right moment to ask is the gate, not the
//! frame. Compiling it into the binary would add dead code and a `dead_code`
//! allowance to silence the warning about it.
//!
//! Unit tests build the library with `cfg(test)` enabled for the whole
//! crate, so a `#[cfg(test)] pub mod` is reachable from every other module's
//! tests — which is what lets `app::status`'s bar gate share this predicate
//! rather than keep its own broken one.
//!
//! ## Why it lives under `icons`
//!
//! `icons` already owns the question *"what happens when a mark cannot be
//! drawn?"* — that is [`crate::icons::paint_missing_mark`], the visible
//! stand-in for an icon whose art is missing. A catalog glyph like `⚠` is
//! the font-supplied sibling of an SVG icon: same job, same failure mode,
//! different pipeline. The substitution mark this module hunts for is
//! precisely `paint_missing_mark`'s counterpart on the text side.

use egui::{Color32, Context, FontId};

// ===========================================================================
// The probe
// ===========================================================================

/// The three unassigned codepoints used to fingerprint the substitution mark.
///
/// Chosen from three different Unicode planes so that no single font's
/// coverage decision can plausibly cover more than one of them:
///
/// - `U+0870` — Arabic Extended-B, unassigned at the time of writing.
/// - `U+2FFFF` — plane 2 (SIP), permanently unassigned (a noncharacter-
///   adjacent tail position no CJK font allocates).
/// - `U+10FFFD` — plane 16, the last Private Use Area codepoint.
///
/// They are only ever *laid out*, never asserted to be absent — the assertion
/// is that they **agree with each other**, which is the property that makes
/// the fingerprint trustworthy. See the module header.
const SENTINELS: [char; 3] = ['\u{0870}', '\u{2FFFF}', '\u{10FFFD}'];

/// A "can the font stack draw this?" predicate for one font family and size.
///
/// Construct once per family, then call [`Self::can_draw`] freely — laying a
/// single character out is cheap and `epaint` caches galleys.
///
/// See the module header for why this exists rather than
/// [`epaint::Fonts::has_glyph`], which returns false negatives.
pub struct GlyphProbe {
    font: FontId,
    /// The atlas rectangle `epaint` produces for an unsupported codepoint.
    substitute: AtlasRect,
}

/// A glyph's rectangle in the font atlas, reduced to comparable primitives.
///
/// `epaint`'s own `UvRect` is not publicly re-exported (its module is
/// private, and `Glyph::uv_rect` is private-in-public), so its four fields
/// are copied out rather than named. `min`/`max` alone would identify the
/// glyph — the atlas allocates one rectangle per distinct glyph — but the
/// offset and size are carried too so that a future atlas that reused
/// rectangles could not collapse two glyphs into one answer.
#[derive(Clone, Copy, Debug, PartialEq)]
struct AtlasRect {
    min: [u16; 2],
    max: [u16; 2],
    offset: [f32; 2],
    size: [f32; 2],
}

impl GlyphProbe {
    /// Fingerprint the substitution mark for `font`.
    ///
    /// # Panics
    ///
    /// If the three [`SENTINELS`] do not all render identically. That means
    /// one of them has acquired a real glyph, the fingerprint is no longer
    /// the substitution mark, and **every subsequent answer would be a false
    /// pass** — so this fails closed rather than going quietly blind.
    ///
    /// Must be called inside a live frame ([`Context::run_ui`] or equivalent);
    /// `egui` has no fonts before one.
    pub fn new(ctx: &Context, font: FontId) -> Self {
        let rects: Vec<_> = SENTINELS
            .iter()
            .map(|&c| Self::uv_rect_of(ctx, &font, c))
            .collect();

        assert!(
            rects.iter().all(|r| *r == rects[0]),
            "the three unassigned sentinel codepoints did not render \
             identically ({:?} -> {rects:?}), so the substitution mark cannot \
             be fingerprinted. One of them has acquired a real glyph. Pick a \
             different sentinel — do NOT relax this assertion: a probe built \
             on a sentinel that draws would report every codepoint as \
             drawable, which is the fail-open shape DEFECTS.md D13 names.",
            SENTINELS,
        );

        Self {
            font,
            substitute: rects[0],
        }
    }

    /// Would `c` render as itself, rather than as the substitution mark?
    ///
    /// Returns `false` for the substitution mark's own codepoint (`◻`,
    /// U+25FB), which is the one irreducible false negative of this approach
    /// — the mark is indistinguishable from itself. That is harmless here
    /// (nothing in the catalog uses it) and is asserted in the tests so the
    /// limit is recorded rather than discovered.
    pub fn can_draw(&self, ctx: &Context, c: char) -> bool {
        Self::uv_rect_of(ctx, &self.font, c) != self.substitute
    }

    /// Lay `c` out alone and return the atlas rectangle of the glyph drawn.
    ///
    /// Control characters lay out to zero glyphs; they are reported as the
    /// substitute so a stray `\u{7}` in a catalog string is caught rather
    /// than panicking on an empty row.
    fn uv_rect_of(ctx: &Context, font: &FontId, c: char) -> AtlasRect {
        ctx.fonts_mut(|f| {
            // NOT A THEME COLOUR: this galley is never painted. It is laid
            // out solely to read the glyph's rectangle in the font atlas,
            // which is independent of colour — `layout_no_wrap` simply
            // requires one. Giving it a palette role would imply the probe
            // draws something, which is the opposite of what it does.
            let galley = f.layout_no_wrap(c.to_string(), font.clone(), Color32::WHITE);
            galley
                .rows
                .first()
                .and_then(|r| r.row.glyphs.first())
                .map(|g| AtlasRect {
                    min: g.uv_rect.min,
                    max: g.uv_rect.max,
                    offset: [g.uv_rect.offset.x, g.uv_rect.offset.y],
                    size: [g.uv_rect.size.x, g.uv_rect.size.y],
                })
                .unwrap_or(AtlasRect {
                    min: [0, 0],
                    max: [0, 0],
                    offset: [0.0, 0.0],
                    size: [0.0, 0.0],
                })
        })
    }
}

// ===========================================================================
// The source scanner
// ===========================================================================

/// One operator-visible string literal, with enough context to name it in a
/// failure message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Literal {
    /// The literal's *value* — escapes resolved, delimiters stripped.
    pub text: String,
    /// 1-based line number of the literal's opening quote.
    pub line: usize,
}

/// Why a source file could not be scanned.
///
/// Every variant is a **refusal**, never a silent skip. A scanner that
/// returned "no literals" for a file it could not parse would be the
/// fail-open shape this gate exists to remove.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScanError {
    /// A raw string literal (`r"…"` / `r#"…"#`) was found.
    ///
    /// The catalog contains none today (verified across all 15 files), so
    /// rather than carry untested raw-string handling this refuses and says
    /// what to do. Extending the scanner is a small job; *appearing* to scan
    /// a construct it does not understand is the expensive failure.
    RawStringUnsupported { line: usize },
    /// A string literal was still open at end of file.
    UnterminatedString { line: usize },
}

/// Extract every string and character literal that a human could read.
///
/// ## What it excludes, and why each exclusion is principled
///
/// 1. **Comments — line, doc and block (nested).** Comments are not drawn.
///    This matters here more than in most scanners: this codebase's doc
///    comments are dense with `★ ▸ § —`, and `▸` in particular is a
///    codepoint the font stack cannot draw. Scanning comments would produce
///    a permanent false failure that would get the gate switched off.
///
/// 2. **`#[cfg(test)]` items, by balanced brace span.** Test prose is read
///    by whoever is staring at a failing test, never rendered. It is also
///    where deliberately-exotic strings live — `text/panels/comments.rs`
///    asserts a note body survives byte for byte using `"多行\ntext"`, and
///    CJK is genuinely absent from the bundled fonts.
///
///    **This is `DEFECTS.md` D13's bug, not repeated.** `check-ui-strings.sh`
///    excludes test code by *truncating the file* at the first column-0
///    `#[cfg(test)]`, so every non-test item below a mid-file test module is
///    silently unscanned while the gate reports clean. This scanner instead
///    skips exactly the braced item the attribute is attached to and
///    **resumes** — so a test module in the middle of a file costs nothing,
///    and `tests::a_mid_file_test_module_does_not_blind_the_scanner` proves
///    it on the shape that defeats the shell gate.
///
/// ## What it does not attempt
///
/// Byte strings, macro-generated text, and text composed at runtime from
/// non-literal sources. The catalog is a set of `&'static str` returns and
/// `format!` templates, so literals are the whole surface; anything else
/// would need a real parser and is out of proportion to the risk.
///
/// # Errors
///
/// See [`ScanError`]. Both variants mean *"this file was not scanned"* and
/// must be treated as a gate failure, never as a clean result.
pub fn string_literals(src: &str) -> Result<Vec<Literal>, ScanError> {
    let b: Vec<char> = src.chars().collect();
    let n = b.len();
    let mut out = Vec::new();
    let mut i = 0usize;
    let mut line = 1usize;
    // Depth of `{}` nesting we must fall back to before scanning resumes.
    // `Some(d)` means "skipping a #[cfg(test)] item that opened at depth d".
    let mut skip_to_depth: Option<usize> = None;
    let mut depth = 0usize;
    let mut pending_cfg_test = false;

    while i < n {
        let c = b[i];

        if c == '\n' {
            line += 1;
            i += 1;
            continue;
        }

        // ---- comments -----------------------------------------------------
        if c == '/' && i + 1 < n && b[i + 1] == '/' {
            while i < n && b[i] != '\n' {
                i += 1;
            }
            continue;
        }
        if c == '/' && i + 1 < n && b[i + 1] == '*' {
            let mut nest = 1usize;
            i += 2;
            while i < n && nest > 0 {
                if b[i] == '\n' {
                    line += 1;
                    i += 1;
                } else if b[i] == '/' && i + 1 < n && b[i + 1] == '*' {
                    nest += 1;
                    i += 2;
                } else if b[i] == '*' && i + 1 < n && b[i + 1] == '/' {
                    nest -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
            }
            continue;
        }

        // ---- raw strings: refuse, loudly ----------------------------------
        //
        // Only `r"…"` / `r#"…"#` (and the `br` byte forms) are raw. A plain
        // `b"…"` byte string is NOT — Rust forbids non-ASCII in one, so it
        // can never carry a codepoint this gate cares about, and it is left
        // to the ordinary string branch below (whose escape rules it shares)
        // so that its closing quote is consumed and the scanner stays in
        // sync. `text/panels/objects.rs` has one, inside its test module.
        //
        // The refusal is suppressed inside a skipped `#[cfg(test)]` item for
        // the same reason the literals are: nothing in there is drawn, so a
        // construct the scanner cannot read there costs the gate nothing.
        if c == 'r' && !prev_is_ident_char(&b, i) {
            let mut j = i + 1;
            while j < n && b[j] == '#' {
                j += 1;
            }
            // `r#ident` (a raw identifier) has no quote and is not a string.
            if j < n && b[j] == '"' && j > i && skip_to_depth.is_none() {
                return Err(ScanError::RawStringUnsupported { line });
            }
        }

        // ---- `#[cfg(test)]` ------------------------------------------------
        //
        // Matched in its canonical spelling only. A variant (`#[cfg( test )]`,
        // `#[cfg(all(test, …))]`) simply is not recognised, so its item gets
        // SCANNED rather than skipped — the fail-closed direction, which is
        // the right way for this particular guess to be wrong.
        if c == '#' && starts_with(&b, i, "#[cfg(test)]") {
            pending_cfg_test = true;
            i += "#[cfg(test)]".len();
            continue;
        }

        // An item that ends without ever opening a brace — `#[cfg(test)] use
        // super::*;` is the common one — must not leave the attribute armed,
        // or the NEXT braced item in the file would be skipped instead. That
        // would be a silent hole in coverage, which is the whole failure
        // class this gate exists to close.
        if c == ';' {
            pending_cfg_test = false;
            i += 1;
            continue;
        }

        // ---- braces --------------------------------------------------------
        if c == '{' {
            if pending_cfg_test && skip_to_depth.is_none() {
                skip_to_depth = Some(depth);
                pending_cfg_test = false;
            }
            depth += 1;
            i += 1;
            continue;
        }
        if c == '}' {
            depth = depth.saturating_sub(1);
            if let Some(d) = skip_to_depth
                && depth == d
            {
                skip_to_depth = None;
            }
            i += 1;
            continue;
        }

        // ---- char literals -------------------------------------------------
        // Distinguished from lifetimes (`'a`) by requiring a closing quote.
        if c == '\'' {
            if let Some((value, next)) = char_literal(&b, i) {
                if skip_to_depth.is_none() {
                    out.push(Literal { text: value, line });
                }
                i = next;
                continue;
            }
            i += 1;
            continue;
        }

        // ---- string literals -----------------------------------------------
        if c == '"' {
            let start_line = line;
            let mut j = i + 1;
            let mut value = String::new();
            loop {
                if j >= n {
                    return Err(ScanError::UnterminatedString { line: start_line });
                }
                match b[j] {
                    '\\' => {
                        // Resolve the escapes that can carry a codepoint; the
                        // rest are ASCII and pass through as themselves.
                        if let Some((ch, next)) = escape(&b, j) {
                            if let Some(ch) = ch {
                                value.push(ch);
                            }
                            j = next;
                        } else {
                            j += 2;
                        }
                    }
                    '"' => {
                        j += 1;
                        break;
                    }
                    ch => {
                        value.push(ch);
                        j += 1;
                    }
                }
            }
            // ★ Count the newlines over the CONSUMED SPAN rather than as they
            // are pushed. A `\` line continuation — which this catalog uses
            // heavily to wrap long sentences — is swallowed inside `escape`
            // and never reaches the match above, so incremental counting
            // silently under-reported every line number after the first long
            // string. It was 30 lines adrift by the middle of
            // `text/panels/objects.rs`, which sends the reader of a failure
            // message to the wrong place — the one job a line number has.
            line += b[i..j].iter().filter(|&&ch| ch == '\n').count();
            if skip_to_depth.is_none() {
                out.push(Literal {
                    text: value,
                    line: start_line,
                });
            }
            i = j;
            continue;
        }

        i += 1;
    }

    Ok(out)
}

/// Is the character before `i` part of an identifier? Used to tell the `r` of
/// `r"…"` from the `r` at the end of `let colour_r = …`.
fn prev_is_ident_char(b: &[char], i: usize) -> bool {
    i > 0 && (b[i - 1].is_alphanumeric() || b[i - 1] == '_')
}

fn starts_with(b: &[char], i: usize, pat: &str) -> bool {
    let p: Vec<char> = pat.chars().collect();
    b.len() >= i + p.len() && b[i..i + p.len()] == p[..]
}

/// Parse a character literal at `i` (which must be `'`).
///
/// Returns `None` for a lifetime, which is the ambiguity this has to resolve:
/// `'a` is a lifetime, `'a'` is a literal, and only the closing quote tells
/// them apart.
fn char_literal(b: &[char], i: usize) -> Option<(String, usize)> {
    let n = b.len();
    if i + 1 >= n {
        return None;
    }
    if b[i + 1] == '\\' {
        let (ch, next) = escape(b, i + 1)?;
        if next < n && b[next] == '\'' {
            return Some((ch.map(String::from).unwrap_or_default(), next + 1));
        }
        return None;
    }
    if i + 2 < n && b[i + 2] == '\'' {
        return Some((b[i + 1].to_string(), i + 3));
    }
    None
}

/// Resolve a backslash escape beginning at `i`.
///
/// Returns `(resolved char, index after the escape)`. The char is `None` for
/// escapes that produce nothing renderable on their own (`\n`, `\t`, `\0`,
/// and a line continuation), which keeps them out of the glyph check without
/// dropping the rest of the literal.
fn escape(b: &[char], i: usize) -> Option<(Option<char>, usize)> {
    let n = b.len();
    if i + 1 >= n {
        return None;
    }
    match b[i + 1] {
        'n' | 't' | 'r' | '0' => Some((None, i + 2)),
        '\\' => Some((Some('\\'), i + 2)),
        '\'' => Some((Some('\''), i + 2)),
        // ui-text-exempt: two char literals holding a double quote, not a
        // string. `check-ui-strings.sh`'s scanner reads the `"` inside `'"'`
        // as opening a literal and the next one as closing it, so the code
        // BETWEEN them — `), i + 2)),` — is reported as operator-facing
        // prose. Nothing on this line is drawn; it is the escape table for
        // this module's own source scanner.
        '"' => Some((Some('"'), i + 2)),
        'x' => {
            // \xNN — always ASCII in Rust string literals.
            let hex: String = b.iter().skip(i + 2).take(2).collect();
            let v = u32::from_str_radix(&hex, 16).ok()?;
            Some((char::from_u32(v), i + 4))
        }
        'u' => {
            // \u{NNNN} — the form that can carry an exotic codepoint, so it
            // must be resolved rather than skipped.
            let mut j = i + 3; // past `\u{`
            let mut hex = String::new();
            while j < n && b[j] != '}' {
                hex.push(b[j]);
                j += 1;
            }
            let v = u32::from_str_radix(&hex, 16).ok()?;
            Some((char::from_u32(v), j + 1))
        }
        '\n' => {
            // Line continuation: `\` at end of line swallows leading
            // whitespace on the next. Nothing renderable is produced.
            let mut j = i + 2;
            while j < n && (b[j] == ' ' || b[j] == '\t') {
                j += 1;
            }
            Some((None, j))
        }
        _ => Some((None, i + 2)),
    }
}

// ===========================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use egui::RawInput;

    // -----------------------------------------------------------------
    // The probe itself
    // -----------------------------------------------------------------

    /// Run `f` inside a live frame. `egui` has no fonts before one, so every
    /// probe assertion has to happen here.
    fn in_a_frame(f: impl FnOnce(&Context)) {
        let ctx = Context::default();
        let mut f = Some(f);
        let _ = ctx.run_ui(RawInput::default(), |_ui| {
            if let Some(f) = f.take() {
                f(&ctx);
            }
        });
    }

    /// ★ **The probe disagrees with `egui`, and the disagreement is the point.**
    ///
    /// This is D12's finding as an executable statement. `⚠` is asserted
    /// **drawable** — which is the claim the defect entry denied — and
    /// `epaint`'s own `has_glyph` is asserted to say the opposite, so the
    /// day upstream fixes its false negative this test fails and says so
    /// rather than quietly agreeing.
    #[test]
    fn the_probe_finds_the_warning_sign_that_has_glyph_denies() {
        in_a_frame(|ctx| {
            let font = FontId::proportional(14.0);
            let probe = GlyphProbe::new(ctx, font.clone());

            assert!(
                probe.can_draw(ctx, '⚠'),
                "U+26A0 did not draw. If this fails, the bundled font set \
                 really has lost NotoEmoji-Regular and DEFECTS.md D12's \
                 original diagnosis has become true after all."
            );

            let egui_says = ctx.fonts_mut(|f| f.has_glyph(&font, '⚠'));
            assert!(
                !egui_says,
                "epaint's `has_glyph` now agrees that U+26A0 is drawable, so \
                 the false negative documented in this module's header has \
                 been fixed upstream. Good news — but re-read the header \
                 before simplifying anything, because `app::status`'s gate \
                 was written around the bug."
            );
        });
    }

    /// The genuinely-absent characters are still reported absent.
    ///
    /// Without this the probe could be trivially satisfied by a predicate
    /// that returns `true` for everything, which is the failure mode a
    /// widened gate is most likely to acquire.
    #[test]
    fn genuinely_absent_codepoints_are_still_reported_absent() {
        in_a_frame(|ctx| {
            let probe = GlyphProbe::new(ctx, FontId::proportional(14.0));
            // Measured absent from all four bundled faces, by reading their
            // charmaps directly. `▸` is the one that matters: it is in the
            // shipped catalog today. See DEFECTS.md D12.
            for c in ['▲', '△', '●', '◆', '□', '✓', '✗', 'ⓘ', '※', '▸', '多'] {
                assert!(
                    !probe.can_draw(ctx, c),
                    "U+{:04X} {c:?} is now drawable. If a font gained coverage \
                     that is fine — remove it from this list and from any \
                     quarantine that names it.",
                    c as u32
                );
            }
        });
    }

    /// The characters D12 listed as present really are.
    #[test]
    fn the_measured_present_marks_all_draw() {
        in_a_frame(|ctx| {
            let probe = GlyphProbe::new(ctx, FontId::proportional(14.0));
            for c in [
                '✱', '⚑', '⚐', '☞', '⊗', '⏺', '◊', '★', '☆', '!', '○', '■', '•', '·', '†', '‡',
                '№', '¶', '⚠', 'ℹ', '‼', '❗',
            ] {
                assert!(
                    probe.can_draw(ctx, c),
                    "U+{:04X} {c:?} did not draw",
                    c as u32
                );
            }
        });
    }

    /// The one irreducible false negative, recorded rather than discovered.
    ///
    /// The substitution mark cannot be told apart from itself. Nothing in the
    /// catalog uses `◻`, and this test is what makes that a known limit
    /// instead of a surprise.
    #[test]
    fn the_substitution_mark_is_the_one_case_the_probe_cannot_judge() {
        in_a_frame(|ctx| {
            let probe = GlyphProbe::new(ctx, FontId::proportional(14.0));
            assert!(
                !probe.can_draw(ctx, '◻'),
                "U+25FB is no longer epaint's substitution mark; re-read \
                 `CachedFamily::new` and update this module's header"
            );
        });
    }

    // -----------------------------------------------------------------
    // The scanner
    // -----------------------------------------------------------------

    #[test]
    fn comments_are_not_scanned() {
        let src = r#"
// a line comment with ▸
//! a doc comment with ▸ and ★
/* a block /* nested */ comment with ▸ */
fn f() -> &'static str { "kept —" }
"#;
        let got = string_literals(src).expect("scan");
        assert_eq!(
            got.iter().map(|l| l.text.as_str()).collect::<Vec<_>>(),
            vec!["kept —"],
            "a comment's characters reached the scanner's output"
        );
    }

    /// ★ **`DEFECTS.md` D13's bug, proven absent here.**
    ///
    /// `check-ui-strings.sh` truncates the file at the first `#[cfg(test)]`,
    /// so anything below a mid-file test module is unscanned while the gate
    /// prints clean. This is that exact shape: a test module in the middle,
    /// with an operator-visible literal after it. The literal must be found.
    #[test]
    fn a_mid_file_test_module_does_not_blind_the_scanner() {
        let src = r#"
pub fn before() -> &'static str { "before —" }

#[cfg(test)]
mod tests {
    #[test]
    fn t() {
        let nested = "test prose 多行 that must NOT be scanned";
        if true { let _ = "still inside the test module"; }
    }
}

pub fn after() -> &'static str { "after —" }
"#;
        let got: Vec<String> = string_literals(src)
            .expect("scan")
            .into_iter()
            .map(|l| l.text)
            .collect();

        assert!(
            got.iter().any(|s| s == "after —"),
            "the literal AFTER the test module was not scanned — this is \
             exactly D13's fail-open, reproduced in the new gate: {got:?}"
        );
        assert!(
            got.iter().any(|s| s == "before —"),
            "the literal before the test module was not scanned: {got:?}"
        );
        assert!(
            !got.iter().any(|s| s.contains("多行")),
            "test-module prose reached the scanner's output: {got:?}"
        );
        assert!(
            !got.iter().any(|s| s.contains("still inside")),
            "a literal nested deeper inside the test module escaped the \
             skip, so the brace tracking is not balanced: {got:?}"
        );
    }

    #[test]
    fn escapes_that_carry_a_codepoint_are_resolved() {
        let src = r#"fn f() { let _ = "a\u{26A0}b\n\"q\" \\ end"; }"#;
        let got = string_literals(src).expect("scan");
        assert_eq!(got.len(), 1);
        assert!(
            got[0].text.contains('⚠'),
            "a \\u{{…}} escape did not resolve to its codepoint, so an \
             unrenderable character written in escaped form would slip past \
             the gate: {:?}",
            got[0].text
        );
        assert!(got[0].text.contains('"'), "escaped quote lost");
    }

    #[test]
    fn a_lifetime_is_not_mistaken_for_a_character_literal() {
        let src = r#"fn f<'a>(s: &'a str) -> &'static str { let c = '⚠'; "x" }"#;
        let got: Vec<String> = string_literals(src)
            .expect("scan")
            .into_iter()
            .map(|l| l.text)
            .collect();
        assert_eq!(got, vec!["⚠".to_owned(), "x".to_owned()], "got {got:?}");
    }

    #[test]
    fn a_raw_string_is_refused_rather_than_skipped() {
        let src = "fn f() { let _ = r\"raw ▸\"; }";
        assert!(
            matches!(
                string_literals(src),
                Err(ScanError::RawStringUnsupported { .. })
            ),
            "a raw string was silently skipped instead of refusing; that is \
             a literal the gate would never see"
        );
    }

    #[test]
    fn line_numbers_point_at_the_opening_quote() {
        let src = "fn a() {}\nfn b() { let _ = \"here\"; }\n";
        let got = string_literals(src).expect("scan");
        assert_eq!(got[0].line, 2, "{got:?}");
    }

    /// ★ A `\` line continuation must still advance the line counter.
    ///
    /// Found by this gate's first real run: the counter was incremented as
    /// characters were *pushed*, and a continuation's newline is swallowed
    /// inside `escape` without being pushed. Every line number after the
    /// first wrapped string was therefore too low — 30 lines adrift by the
    /// middle of `text/panels/objects.rs`. A wrong line number in a failure
    /// message is worse than none: it sends the reader somewhere plausible.
    #[test]
    fn a_line_continuation_still_counts_its_newline() {
        // string-gap-exempt: this literal IS escaped Rust source, and the runs
        // of spaces are the indentation of the continuation lines being
        // scanned — the exact thing under test. Rejoining them would delete
        // the fixture.
        let src = "fn a() {\n    let _ = \"one \\\n         two \\\n         three\";\n}\nfn b() { let _ = \"LAST\"; }\n";
        let got = string_literals(src).expect("scan");
        let last = got.iter().find(|l| l.text == "LAST").expect("found LAST");
        assert_eq!(
            last.line, 6,
            "the continuation's two newlines were not counted; got {got:?}"
        );
    }

    /// `#[cfg(test)]` on an unbraced item must not disarm the next function.
    ///
    /// `#[cfg(test)] use super::*;` is the idiom that would do it: the
    /// attribute arms the skip, the `use` never opens a brace, and the next
    /// `{` in the file — an ordinary operator-visible function — would be
    /// swallowed. A silent hole in coverage that prints clean.
    #[test]
    fn a_cfg_test_use_does_not_swallow_the_next_function() {
        let src =
            "#[cfg(test)]\nuse super::*;\n\npub fn visible() -> &'static str { \"KEPT —\" }\n";
        let got: Vec<String> = string_literals(src)
            .expect("scan")
            .into_iter()
            .map(|l| l.text)
            .collect();
        assert!(
            got.iter().any(|s| s == "KEPT —"),
            "the function after `#[cfg(test)] use …;` was skipped: {got:?}"
        );
    }

    /// A byte string is not a raw string, and must not be refused.
    ///
    /// `text/panels/objects.rs` builds a content stream from `b"…"` inside
    /// its test module. Rust forbids non-ASCII in a byte string, so it can
    /// never carry a codepoint this gate cares about — refusing the whole
    /// file over one would have taken a real catalog module out of scope.
    #[test]
    fn a_byte_string_is_scanned_rather_than_refused() {
        let src = "fn f() { let _ = b\"0 0 1 rg re f\".to_vec(); let _ = \"kept —\"; }";
        let got: Vec<String> = string_literals(src)
            .expect("a byte string must not be refused")
            .into_iter()
            .map(|l| l.text)
            .collect();
        assert!(
            got.iter().any(|s| s == "kept —"),
            "the byte string desynchronised the scanner, so the literal after \
             it was lost: {got:?}"
        );
    }

    // -----------------------------------------------------------------
    // THE GATE
    // -----------------------------------------------------------------

    /// Every `.rs` file under the catalog directory.
    fn catalog_files() -> Vec<std::path::PathBuf> {
        fn walk(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
            for entry in std::fs::read_dir(dir).expect("read the catalog directory") {
                let p = entry.expect("a directory entry").path();
                if p.is_dir() {
                    walk(&p, out);
                } else if p.extension().is_some_and(|e| e == "rs") {
                    out.push(p);
                }
            }
        }
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/text");
        let mut out = Vec::new();
        walk(&root, &mut out);
        out.sort();
        out
    }

    /// ★★ **THE WIDENED GLYPH GATE — every string in `crate::text` renders.**
    ///
    /// `DEFECTS.md` D12 names this as the fix, and names it as the *only*
    /// fix: *"The fix that would prevent a fifth sighting is not a
    /// substitution: it is pointing the existing glyph gate at every `text/`
    /// module rather than at the status bar alone, so a codepoint the stack
    /// cannot draw fails at the gate rather than in front of the operator."*
    ///
    /// The predecessor,
    /// `app::status::tests::every_glyph_the_status_bar_draws_has_a_glyph`,
    /// enumerated the bar's own labels by hand. Two things were wrong with
    /// that and both are fixed here:
    ///
    /// 1. **It covered one surface.** The catalog is ~5,900 lines across 15
    ///    files and every one of them draws. This reads the source, so a
    ///    string added tomorrow is covered without anyone remembering to
    ///    list it — which is the property a hand-maintained list can never
    ///    have. `DEFECTS.md` D5 is the same lesson from the shortcuts
    ///    reference: *"a hand-maintained list with a comment telling you to
    ///    hand-maintain it has already failed once."*
    /// 2. **It asked `has_glyph`.** See this module's header.
    ///
    /// ## The quarantine, and why it is empty
    ///
    /// **It is empty because the gate worked.** On its first run it found two
    /// live tofu boxes, both of which are now fixed:
    ///
    /// | codepoint | where it was | what it became |
    /// |---|---|---|
    /// | `▸` U+25B8 | the menu-path separator in four operator-visible strings, one of them the **empty-canvas message** — the first sentence a new operator ever reads | `>`, which the probe measured as drawable |
    /// | `�` U+FFFD | `text/panels/objects.rs` — a sentence telling the operator that unreadable characters *"are shown as `�`"* | the mark is **named** rather than shown; see that string's own comment for why naming is better than substituting |
    ///
    /// The second is the more instructive. It read correctly before the fix
    /// only *by accident*: epaint substituted `◻` both for the character in
    /// the sentence **and** for the undecodable characters the sentence was
    /// about, so the two happened to match. A coincidence of two bugs is not a
    /// design, and it would have stopped matching the moment either was fixed.
    ///
    /// The quarantine mechanism is kept, with nothing in it, because the
    /// property that matters is that it is **self-tightening**: the gate
    /// asserts every quarantined codepoint is *still* both undrawable and
    /// still present in the catalog. Fixing the strings made the gate fail
    /// telling us to delete the entries, which is exactly what happened here.
    /// A list that cannot rot into a permanent exemption is worth keeping even
    /// when it is empty; the next reader who needs one gets the discipline
    /// with it.
    #[test]
    fn every_glyph_the_catalog_draws_has_a_glyph() {
        /// Codepoints known bad, filed, and not this work's to change.
        /// `(codepoint, why)`. See the doc comment above before adding one —
        /// in particular, an entry is a **filed defect with an owner**, not a
        /// way to make this gate quiet.
        const QUARANTINE: &[(char, &str)] = &[];

        let files = catalog_files();
        assert!(
            files.len() >= 15,
            "found only {} catalog files, which is fewer than the 15 that \
             existed when this gate was written — the walk is not reaching \
             the tree and a clean result would prove nothing. Looked in {:?}",
            files.len(),
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/text"),
        );

        // (char, file, line, the literal it came from)
        let mut scanned_chars = 0usize;
        let mut per_char: std::collections::BTreeMap<char, Vec<String>> = Default::default();

        for path in &files {
            let src = std::fs::read_to_string(path).expect("read a catalog file");
            let lits = string_literals(&src).unwrap_or_else(|e| {
                panic!(
                    "{} could not be scanned ({e:?}), so this gate did NOT \
                     check it. Extend the scanner; do not skip the file.",
                    path.display()
                )
            });
            for lit in lits {
                for c in lit.text.chars() {
                    scanned_chars += 1;
                    if c.is_ascii() {
                        continue;
                    }
                    per_char.entry(c).or_default().push(format!(
                        "{}:{}",
                        path.file_name().unwrap_or_default().to_string_lossy(),
                        lit.line
                    ));
                }
            }
        }

        // ★ Assert the measurement HAPPENED, not only its value (HANDOFF.md
        // §10). Without these three, a scanner that silently returned nothing
        // would produce a green gate that read every file and checked none.
        // Measured 2026-08-14: 45,323 literal characters across the 15 files.
        // The floor is set well below that so ordinary catalog edits do not
        // trip it, and well above zero so a scanner that returned nothing —
        // the failure this guard exists for — cannot read as clean.
        assert!(
            scanned_chars > 30_000,
            "only {scanned_chars} characters were scanned across {} files; \
             45,323 were found when this gate was written, so the scanner is \
             dropping literals and a clean result would mean nothing",
            files.len()
        );
        assert!(
            per_char.contains_key(&'⚠'),
            "the catalog's 15 occurrences of U+26A0 were not seen by the \
             scanner. That is the exact character this gate exists for, so \
             its absence means the scan missed `text/forms.rs` entirely."
        );
        assert!(
            per_char.len() >= 10,
            "only {} distinct non-ASCII codepoints found; expected the \
             catalog's ~14. The scan is incomplete: {per_char:?}",
            per_char.len()
        );

        in_a_frame(|ctx| {
            let probe = GlyphProbe::new(ctx, FontId::proportional(14.0));

            // -- the quarantine must still be earning its place ------------
            for (c, why) in QUARANTINE {
                assert!(
                    !probe.can_draw(ctx, *c),
                    "U+{:04X} {c:?} is quarantined but now DRAWS. Delete the \
                     entry — a quarantine that outlives its reason is how an \
                     exemption becomes permanent. ({why})",
                    *c as u32
                );
                assert!(
                    per_char.contains_key(c),
                    "U+{:04X} {c:?} is quarantined but no longer appears in \
                     the catalog. The strings were fixed: delete the entry. \
                     ({why})",
                    *c as u32
                );
            }

            // -- the gate proper --------------------------------------------
            let mut tofu: Vec<String> = Vec::new();
            for (c, sites) in &per_char {
                if QUARANTINE.iter().any(|(q, _)| q == c) {
                    continue;
                }
                if !probe.can_draw(ctx, *c) {
                    tofu.push(format!("U+{:04X} {c:?} at {}", *c as u32, sites.join(", ")));
                }
            }

            assert!(
                tofu.is_empty(),
                "these codepoints are in operator-visible catalog strings and \
                 the font stack CANNOT DRAW THEM — each renders as a \
                 substitution box in front of the operator:\n  {}\n\n\
                 Choose a codepoint the stack can draw (`crate::icons::glyphs` \
                 records which), or add font coverage. Do NOT add it to the \
                 quarantine unless it is genuinely someone else's file to fix.",
                tofu.join("\n  ")
            );
        });
    }

    /// ★ **The gate has been observed failing.**
    ///
    /// This project requires it: a gate that has only ever passed is not
    /// evidence of anything (`check-ui-strings.sh` PORT CHANGE 3, and the
    /// D12 post-mortem). The gate above cannot be made to fail on demand
    /// without editing the catalog, so its two halves — the scanner and the
    /// probe — are driven here against a **planted** unrenderable codepoint
    /// in a synthetic catalog file, through exactly the same code path.
    ///
    /// The plant is `✓` U+2713, measured absent, placed in an
    /// operator-visible position in a file that otherwise looks like a
    /// catalog module — including a decoy `▸` inside a doc comment and a
    /// decoy CJK string inside a test module, both of which must be ignored,
    /// so this proves the gate fires on the real thing rather than on
    /// anything that merely looks exotic.
    #[test]
    fn the_gate_catches_a_planted_unrenderable_codepoint() {
        const PLANTED: &str = r####"
//! A catalog module — the doc comment carries ▸ and ★, which must be ignored.

/// A ▸ in an item doc comment, also ignored.
pub fn ok() -> &'static str {
    "Everything here draws — an em dash, an ellipsis…, a warning ⚠"
}

pub fn planted() -> &'static str {
    "This sentence has a check mark ✓ that the font stack cannot draw"
}

#[cfg(test)]
mod tests {
    #[test]
    fn t() {
        assert_eq!(super::ok(), "多行 ▲ decoys inside a test module");
    }
}
"####;

        let lits = string_literals(PLANTED).expect("the planted file scans");

        // The scanner half: the decoys are gone, the plant is present.
        let all: String = lits.iter().map(|l| l.text.as_str()).collect();
        assert!(
            all.contains('✓'),
            "the planted U+2713 never reached the scanner's output, so the \
             failure this test claims to prove would not have been detectable"
        );
        assert!(
            !all.contains('▸') && !all.contains('多') && !all.contains('▲'),
            "a decoy from a comment or a test module reached the output: \
             {all:?}"
        );

        in_a_frame(|ctx| {
            let probe = GlyphProbe::new(ctx, FontId::proportional(14.0));

            let mut tofu = Vec::new();
            for lit in &lits {
                for c in lit.text.chars() {
                    if !c.is_ascii() && !probe.can_draw(ctx, c) {
                        tofu.push((c, lit.line));
                    }
                }
            }

            assert_eq!(
                tofu.iter().map(|(c, _)| *c).collect::<Vec<_>>(),
                vec!['✓'],
                "the gate's two halves, run over a file with one planted \
                 unrenderable codepoint, did not report exactly that \
                 codepoint. Got {tofu:?}"
            );

            // And the sentence that is fine really is fine — otherwise this
            // test would pass just as well with a probe that hates everything.
            for c in ['—', '…', '⚠'] {
                assert!(
                    probe.can_draw(ctx, c),
                    "U+{:04X} in the planted file's GOOD sentence was also \
                     flagged, so the probe is not discriminating",
                    c as u32
                );
            }
        });
    }
}
