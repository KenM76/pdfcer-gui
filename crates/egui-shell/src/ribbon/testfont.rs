//! A synthetic TrueType face, assembled in memory, so that this crate's
//! layout tests can measure **real text**.
//!
//! # ★ Why this file exists — the defect it retires
//!
//! `egui-shell` depends on `egui` with `default-features = false`. That
//! is a deliberate dependency-posture decision (`Cargo.toml` says why),
//! and it has a consequence nobody wrote down until it cost two defects:
//!
//! > With `default_fonts` off there is **no font data at all**, so every
//! > galley measures ≈ 0 × 0 and every width comparison in the ribbon is
//! > trivially satisfied.
//!
//! The whole ribbon width layer — group measurement, the overflow
//! reservation, the band budget, the mode selector's track — had therefore
//! only ever been exercised against text of zero width. Its tests passed
//! because there was nothing for them to fail against.
//!
//! Worse, the failure was **conditional on who was building**:
//!
//! | Command | What `egui-shell`'s tests get |
//! |---|---|
//! | `cargo test -p egui-shell --lib` | `egui` alone → no fonts → zero widths |
//! | `cargo test --workspace` | `pdfcer-gui` → `eframe` → `egui/default_fonts` → **real** widths |
//!
//! Cargo unifies features across a workspace build, so the same test
//! source measured different text depending on which sibling crate
//! happened to be in the build graph. Two real defects (an overflow
//! affordance placed off screen, and a `usize` underflow in a failure
//! message) lived in the gap between those two columns, invisible to the
//! narrower command.
//!
//! # What this module guarantees
//!
//! A font that is **present in both builds and identical in both**. It
//! depends on no feature, no system font directory, no network and no
//! file on disk; it is a `Vec<u8>` this file computes. Tests that install
//! it therefore exercise the width arithmetic with real, non-zero,
//! *per-character* advances no matter how the crate is built or which
//! sibling pulled in which feature.
//!
//! [`install`] additionally **asserts that the font took effect** — that a
//! sample string measures wider than nothing, and that a longer string
//! measures wider than a shorter one. That check is the point of the
//! module as much as the font is: a synthetic font that silently failed
//! to load would restore the exact vacuum this file exists to remove, and
//! it would do so while the tests still passed.
//!
//! # Why synthesise rather than embed or borrow
//!
//! - **Embedding a real `.ttf`** means committing a binary blob and its
//!   licence into a crate whose whole dependency argument is that it does
//!   not acquire fonts it did not ask for.
//! - **Loading a system font** (`C:\Windows\Fonts\…`, `/usr/share/fonts`)
//!   makes the test's *numbers* depend on the machine, and its *existence*
//!   depend on the platform. A layout test that is skipped on the CI box
//!   is a layout test that does not exist.
//! - **Reaching for `epaint_default_fonts`** would mean adding a
//!   dev-dependency, i.e. re-introducing the very feature whose presence
//!   or absence is the thing under test.
//!
//! Synthesising is the only option that is byte-identical everywhere,
//! costs nothing at runtime, and cannot be turned off by a feature flag.
//!
//! # The font, precisely
//!
//! A minimal but genuinely valid `sfnt` with seven tables — `cmap`,
//! `glyf`, `head`, `hhea`, `hmtx`, `loca`, `maxp` — in the ascending tag
//! order the format requires. 99 glyphs:
//!
//! | Glyph ids | Characters | Advance (font units, 1000/em) |
//! |---|---|---|
//! | 0 | `.notdef` | 480 |
//! | 1 – 95 | U+0020 – U+007E, one glyph each | 250 – 700, by character |
//! | 96 | `…` U+2026 | 900 |
//! | 97 | `⏷` U+2304 (the overflow chevron) | 700 |
//! | 98 | `�` U+FFFD (epaint's replacement char) | 700 |
//!
//! Advances are **proportional, not monospaced** — `W` is 700 units and
//! `l` is 280 — because a monospaced synthetic font would hide precisely
//! the class of bug real proportional text causes: a string whose width
//! is not a function of its character count. `"⏷ 8 more"` being wider than
//! `"⏷ 9 more"` is exactly that class, and [`super::plan::overflow_width`]
//! is written against it.
//!
//! Every glyph carries a real outline (a rectangle, one closed contour,
//! four on-curve points) rather than being blank, so the rasterisation
//! path is exercised too and a change that starts consulting glyph bounds
//! rather than advances does not silently measure nothing.
//!
//! The em square is 1000 units with an 800/−200 ascent/descent, which is
//! ordinary enough that a control sized from these metrics has believable
//! proportions: at `egui`'s 14 pt body size a capital is 9.8 pt wide.

use std::sync::Arc;

/// Units per em. 1000 is the TrueType convention and makes the advance
/// table readable as "per mille of the em".
const UNITS_PER_EM: u16 = 1000;

/// Ascent in font units — the top of a capital plus a little.
const ASCENT: i16 = 800;

/// Descent in font units, negative as the format requires.
const DESCENT: i16 = -200;

/// Height of every glyph's rectangle, in font units.
const GLYPH_TOP: i16 = 600;

/// First and last character of the contiguous ASCII run that gets one
/// glyph each. Glyph id is `1 + (c - ASCII_FIRST)`.
const ASCII_FIRST: u32 = 0x20;
const ASCII_LAST: u32 = 0x7E;

/// Characters outside the ASCII run that the ribbon actually draws, each
/// with its own glyph appended after the run.
///
/// `…` appears in command labels ("Open…"), `⏷` is the overflow
/// affordance's chevron, and `�` is what `epaint` looks for as its
/// replacement character — supplying it keeps a "failed to find
/// replacement characters" warning out of the test log and gives any
/// unmapped character a visible, measurable width rather than nothing.
const EXTRAS: [(char, u16); 3] = [('…', 900), ('⏷', 700), ('\u{FFFD}', 700)];

/// The name this face is registered under in [`egui::FontDefinitions`].
pub(crate) const FAMILY_NAME: &str = "egui-shell-test-face";

/// The advance width, in font units, of one character's glyph.
///
/// Deliberately irregular. See the module header: a monospaced synthetic
/// font would make every string's width a function of its length, which is
/// the one property real proportional text does not have and the one the
/// ribbon's arithmetic must not assume.
fn advance_for(c: char) -> u16 {
    match c {
        ' ' => 250,
        '.' | ',' | ':' | ';' | '\'' | '`' | '|' | '!' | 'i' | 'l' | 'I' | 'j' | 't' => 280,
        'm' | 'w' => 640,
        'M' | 'W' => 780,
        'A'..='Z' => 700,
        'a'..='z' => 520,
        '0'..='9' => 550,
        _ => 480,
    }
}

/// Every glyph's advance width, indexed by glyph id.
///
/// Glyph 0 is `.notdef`; ids 1..=95 are the ASCII run; the rest are
/// [`EXTRAS`], in order.
fn advances() -> Vec<u16> {
    let mut out = vec![advance_for('\u{0}')];
    for code in ASCII_FIRST..=ASCII_LAST {
        out.push(advance_for(char::from_u32(code).expect("ASCII is a char")));
    }
    out.extend(EXTRAS.iter().map(|(_, w)| *w));
    out
}

// ---------------------------------------------------------------------
// Byte assembly
//
// Everything below writes big-endian, which is the only endianness the
// `sfnt` container has. The helpers exist so the table builders read as a
// transcription of the specification's field lists rather than as
// `to_be_bytes` noise.
// ---------------------------------------------------------------------

fn u16be(out: &mut Vec<u8>, v: u16) {
    out.extend_from_slice(&v.to_be_bytes());
}

fn i16be(out: &mut Vec<u8>, v: i16) {
    out.extend_from_slice(&v.to_be_bytes());
}

fn u32be(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_be_bytes());
}

/// `head` — 54 bytes. Supplies the em square and, critically,
/// `indexToLocFormat = 0`, which declares that `loca` holds *halved*
/// 16-bit offsets. Every glyph record below is therefore an even number of
/// bytes long.
fn head(num_glyphs: u16) -> Vec<u8> {
    let mut t = Vec::new();
    u32be(&mut t, 0x0001_0000); // version 1.0
    u32be(&mut t, 0x0001_0000); // fontRevision
    u32be(&mut t, 0); // checkSumAdjustment — unchecked by skrifa
    u32be(&mut t, 0x5F0F_3CF5); // magicNumber, and it is checked
    u16be(&mut t, 0); // flags
    u16be(&mut t, UNITS_PER_EM);
    u32be(&mut t, 0); // created  (i64, high)
    u32be(&mut t, 0); // created  (i64, low)
    u32be(&mut t, 0); // modified (i64, high)
    u32be(&mut t, 0); // modified (i64, low)
    i16be(&mut t, 0); // xMin
    i16be(&mut t, 0); // yMin
    i16be(&mut t, UNITS_PER_EM as i16); // xMax
    i16be(&mut t, GLYPH_TOP); // yMax
    u16be(&mut t, 0); // macStyle
    u16be(&mut t, 8); // lowestRecPPEM
    i16be(&mut t, 2); // fontDirectionHint
    i16be(&mut t, 0); // indexToLocFormat: short
    i16be(&mut t, 0); // glyphDataFormat
    debug_assert_eq!(t.len(), 54);
    let _ = num_glyphs;
    t
}

/// `hhea` — 36 bytes. `numberOfHMetrics == numGlyphs`, so `hmtx` carries a
/// full advance for every glyph and no glyph inherits the last one.
fn hhea(num_glyphs: u16) -> Vec<u8> {
    let mut t = Vec::new();
    u32be(&mut t, 0x0001_0000); // version 1.0
    i16be(&mut t, ASCENT);
    i16be(&mut t, DESCENT);
    i16be(&mut t, 0); // lineGap
    u16be(&mut t, UNITS_PER_EM); // advanceWidthMax
    i16be(&mut t, 0); // minLeftSideBearing
    i16be(&mut t, 0); // minRightSideBearing
    i16be(&mut t, UNITS_PER_EM as i16); // xMaxExtent
    i16be(&mut t, 1); // caretSlopeRise
    i16be(&mut t, 0); // caretSlopeRun
    i16be(&mut t, 0); // caretOffset
    for _ in 0..4 {
        i16be(&mut t, 0); // reserved
    }
    i16be(&mut t, 0); // metricDataFormat
    u16be(&mut t, num_glyphs); // numberOfHMetrics
    debug_assert_eq!(t.len(), 36);
    t
}

/// `maxp` version 1.0 — 32 bytes. The `max*` fields describe the
/// rectangle glyphs built by [`glyf_and_loca`]: one contour, four points,
/// no instructions, no composites.
fn maxp(num_glyphs: u16) -> Vec<u8> {
    let mut t = Vec::new();
    u32be(&mut t, 0x0001_0000); // version 1.0
    u16be(&mut t, num_glyphs);
    u16be(&mut t, 4); // maxPoints
    u16be(&mut t, 1); // maxContours
    u16be(&mut t, 0); // maxCompositePoints
    u16be(&mut t, 0); // maxCompositeContours
    u16be(&mut t, 1); // maxZones
    u16be(&mut t, 0); // maxTwilightPoints
    u16be(&mut t, 0); // maxStorage
    u16be(&mut t, 0); // maxFunctionDefs
    u16be(&mut t, 0); // maxInstructionDefs
    u16be(&mut t, 0); // maxStackElements
    u16be(&mut t, 0); // maxSizeOfInstructions
    u16be(&mut t, 0); // maxComponentElements
    u16be(&mut t, 0); // maxComponentDepth
    debug_assert_eq!(t.len(), 32);
    t
}

/// `hmtx` — one `(advanceWidth, leftSideBearing)` pair per glyph.
fn hmtx(advances: &[u16]) -> Vec<u8> {
    let mut t = Vec::new();
    for a in advances {
        u16be(&mut t, *a);
        i16be(&mut t, 50); // leftSideBearing, matching the outline below
    }
    t
}

/// `glyf` and `loca` together, because the second is an index into the
/// first and building them apart is how they drift.
///
/// Every glyph is one closed contour: a rectangle from `x = 50` to
/// `x = advance − 50`, `y = 0` to `y = GLYPH_TOP`, four on-curve points,
/// no hinting instructions. All four coordinate flags are bare `0x01`
/// (`ON_CURVE`), which means the deltas that follow are signed 16-bit —
/// the long form, chosen because it is the one with no special cases.
///
/// Each record is 34 bytes, which is even, which is what short `loca`
/// (`indexToLocFormat = 0`, offsets stored halved) requires.
fn glyf_and_loca(advances: &[u16]) -> (Vec<u8>, Vec<u8>) {
    let mut glyf = Vec::new();
    let mut loca = Vec::new();

    for advance in advances {
        u16be(&mut loca, (glyf.len() / 2) as u16);

        let x0: i16 = 50;
        let x1: i16 = (*advance as i16 - 50).max(x0 + 10);

        u16be(&mut glyf, 1); // numberOfContours
        i16be(&mut glyf, x0); // xMin
        i16be(&mut glyf, 0); // yMin
        i16be(&mut glyf, x1); // xMax
        i16be(&mut glyf, GLYPH_TOP); // yMax
        u16be(&mut glyf, 3); // endPtsOfContours[0]: last point index
        u16be(&mut glyf, 0); // instructionLength
        glyf.extend_from_slice(&[0x01, 0x01, 0x01, 0x01]); // flags: on-curve
        // x deltas, then y deltas, each i16 because the flags say so.
        for dx in [x0, x1 - x0, 0, x0 - x1] {
            i16be(&mut glyf, dx);
        }
        for dy in [0, 0, GLYPH_TOP, 0] {
            i16be(&mut glyf, dy);
        }
    }

    // `loca` has numGlyphs + 1 entries: the last one closes the final
    // glyph. Without it the last glyph has no end and skrifa reads it as
    // empty.
    u16be(&mut loca, (glyf.len() / 2) as u16);
    (glyf, loca)
}

/// `cmap` — one format 4 subtable under platform 3, encoding 1.
///
/// Format 4 rather than the simpler format 12 because it is the one every
/// shaper and every parser has supported since 1996; a synthetic font is
/// not the place to discover which of `skrifa`'s or `harfrust`'s subtable
/// preferences applies.
///
/// Four real segments plus the mandatory `0xFFFF` terminator:
///
/// | Segment | Characters | Glyphs |
/// |---|---|---|
/// | 0 | U+0020 – U+007E | 1 – 95 |
/// | 1 | `…` | 96 |
/// | 2 | `⏷` | 97 |
/// | 3 | `�` | 98 |
/// | 4 | U+FFFF | 0 (required end marker) |
///
/// `idDelta` is `(glyph − character) mod 65536` and `idRangeOffset` is
/// zero throughout, which is the direct-mapping form of the table.
fn cmap() -> Vec<u8> {
    let ascii_glyph_base: u32 = 1;
    let extra_base: u32 = 1 + (ASCII_LAST - ASCII_FIRST + 1);

    // (start, end, first glyph)
    let mut segments: Vec<(u32, u32, u32)> = vec![(ASCII_FIRST, ASCII_LAST, ascii_glyph_base)];
    for (i, (c, _)) in EXTRAS.iter().enumerate() {
        let code = *c as u32;
        segments.push((code, code, extra_base + i as u32));
    }
    segments.push((0xFFFF, 0xFFFF, 0)); // terminator: maps to .notdef
    segments.sort_by_key(|(start, _, _)| *start);

    let seg_count = segments.len() as u16;
    let mut sub = Vec::new();
    u16be(&mut sub, 4); // format
    u16be(&mut sub, 16 + 8 * seg_count); // length
    u16be(&mut sub, 0); // language
    u16be(&mut sub, seg_count * 2); // segCountX2
    let entry_selector = 15 - seg_count.leading_zeros() as u16; // floor(log2)
    let search_range = 2 * (1 << entry_selector);
    u16be(&mut sub, search_range);
    u16be(&mut sub, entry_selector);
    u16be(&mut sub, seg_count * 2 - search_range); // rangeShift
    for (_, end, _) in &segments {
        u16be(&mut sub, *end as u16);
    }
    u16be(&mut sub, 0); // reservedPad
    for (start, _, _) in &segments {
        u16be(&mut sub, *start as u16);
    }
    for (start, _, glyph) in &segments {
        let delta = if *glyph == 0 {
            // The 0xFFFF terminator maps to glyph 0. The convention is a
            // delta of 1, which wraps 0xFFFF round to 0.
            1_i32
        } else {
            *glyph as i32 - *start as i32
        };
        i16be(&mut sub, delta as i16);
    }
    for _ in &segments {
        u16be(&mut sub, 0); // idRangeOffset: direct mapping
    }
    debug_assert_eq!(sub.len(), 16 + 8 * seg_count as usize);

    let mut t = Vec::new();
    u16be(&mut t, 0); // version
    u16be(&mut t, 1); // numTables
    u16be(&mut t, 3); // platformID: Windows
    u16be(&mut t, 1); // encodingID: Unicode BMP
    u32be(&mut t, 12); // offset to the subtable
    t.extend_from_slice(&sub);
    t
}

/// Wrap the tables in an `sfnt` container.
///
/// `tables` must already be in ascending tag order — the format requires
/// the directory to be sorted, and parsers binary-search it. Each table's
/// data is padded to a four-byte boundary; the recorded length is the
/// *unpadded* one, as the specification says.
///
/// Checksums are written as zero. `skrifa` does not verify them, and a
/// test font that computed them would be spending code on the one field
/// nothing in this stack reads.
fn assemble(tables: &[(&[u8; 4], Vec<u8>)]) -> Vec<u8> {
    let n = tables.len() as u16;
    let entry_selector = 15 - n.leading_zeros() as u16;
    let search_range = 16 * (1 << entry_selector);

    let mut out = Vec::new();
    u32be(&mut out, 0x0001_0000); // sfntVersion: TrueType outlines
    u16be(&mut out, n);
    u16be(&mut out, search_range);
    u16be(&mut out, entry_selector);
    u16be(&mut out, n * 16 - search_range); // rangeShift

    let mut offset = 12 + 16 * tables.len() as u32;
    for (tag, data) in tables {
        out.extend_from_slice(*tag);
        u32be(&mut out, 0); // checkSum
        u32be(&mut out, offset);
        u32be(&mut out, data.len() as u32);
        offset += data.len().next_multiple_of(4) as u32;
    }
    for (_, data) in tables {
        out.extend_from_slice(data);
        while out.len() % 4 != 0 {
            out.push(0);
        }
    }
    out
}

/// The synthetic face, as TrueType bytes.
pub(crate) fn font_bytes() -> Vec<u8> {
    let advances = advances();
    let num_glyphs = advances.len() as u16;
    let (glyf, loca) = glyf_and_loca(&advances);

    // Ascending tag order, which the directory requires:
    // cmap < glyf < head < hhea < hmtx < loca < maxp.
    assemble(&[
        (b"cmap", cmap()),
        (b"glyf", glyf),
        (b"head", head(num_glyphs)),
        (b"hhea", hhea(num_glyphs)),
        (b"hmtx", hmtx(&advances)),
        (b"loca", loca),
        (b"maxp", maxp(num_glyphs)),
    ])
}

/// The `egui` font set containing only the synthetic face.
///
/// Built from [`egui::FontDefinitions::empty`] rather than `default()`,
/// and that is the load-bearing choice: `default()` is *itself* the thing
/// that varies with the `default_fonts` feature, so a test built on it
/// would go on measuring different text in the two build configurations —
/// which is the defect, not the fix. `empty()` plus one known face is the
/// same font set in every build.
pub(crate) fn definitions() -> egui::FontDefinitions {
    let mut defs = egui::FontDefinitions::empty();
    defs.font_data.insert(
        FAMILY_NAME.to_owned(),
        Arc::new(egui::FontData::from_owned(font_bytes())),
    );
    for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        defs.families.insert(family, vec![FAMILY_NAME.to_owned()]);
    }
    defs
}

/// Install the synthetic face into `ctx`, **and prove it took effect**.
///
/// The proof is not ceremony. A font that failed to load leaves `egui`
/// measuring every string as zero — the precise condition this module
/// exists to eliminate — and every width assertion downstream would then
/// pass for the wrong reason, silently, exactly as they did before this
/// file was written. So three things are checked, and a failure here is a
/// failure of the test suite rather than a warning in a log:
///
/// 1. A sample string has a positive width.
/// 2. A longer string is wider than a shorter one.
/// 3. Two strings of the **same length** but different characters have
///    **different** widths — i.e. the face really is proportional, and a
///    test that depends on real metrics cannot be satisfied by a
///    fixed-pitch stand-in.
///
/// `egui` parses fonts eagerly on the frame after `set_fonts`, and a
/// malformed one panics inside `epaint` with `"Error parsing … TTF/OTF
/// font file"`, so a broken synthetic font can never be mistaken for a
/// missing one.
pub(crate) fn install(ctx: &egui::Context) {
    ctx.set_fonts(definitions());

    // `egui` builds its `Fonts` at the start of a frame, and
    // `Context::fonts_mut` panics with "No fonts available until first
    // call to Context::run()" before then. One empty frame is what turns
    // the definitions above into something measurable — and it is also
    // where a malformed face would panic inside `epaint`, which is why
    // this happens here rather than at the first real assertion.
    let _ = ctx.run_ui(egui::RawInput::default(), |_ui| {});

    let font_id = egui::FontId::proportional(14.0);
    let width = |s: &str| {
        ctx.fonts_mut(|f| {
            f.layout_no_wrap(s.to_owned(), font_id.clone(), egui::Color32::PLACEHOLDER)
                .size()
                .x
        })
    };

    let sample = width("Page display");
    assert!(
        sample > 1.0,
        "the synthetic face did not take effect: \"Page display\" measured {sample} pt. \
         Every width assertion in this crate would now be satisfied by text that \
         occupies no space, which is the exact condition this module exists to remove."
    );
    assert!(
        width("Page display and more") > sample,
        "longer text did not measure wider — the face is loaded but its advances are not \
         being used"
    );
    assert!(
        (width("WWWW") - width("llll")).abs() > 1.0,
        "the face is measuring as monospaced; a proportional face is required, because \
         the arithmetic under test must not assume width is a function of length"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **★ The synthetic face parses, loads, and measures real text.**
    ///
    /// The self-test of the harness. Everything in
    /// [`super::super::width_tests`] is worthless if this is not true, and
    /// "worthless" here means "passing" — which is why it is asserted
    /// rather than assumed.
    #[test]
    fn the_synthetic_face_loads_and_measures_real_text() {
        let ctx = egui::Context::default();
        install(&ctx); // asserts internally

        let width = |s: &str| {
            ctx.fonts_mut(|f| {
                f.layout_no_wrap(
                    s.to_owned(),
                    egui::FontId::proportional(14.0),
                    egui::Color32::PLACEHOLDER,
                )
                .size()
                .x
            })
        };

        // The characters the ribbon actually draws, including the two
        // non-ASCII ones. A missing glyph would fall back to the
        // replacement character rather than to nothing, so this asserts
        // proportionality rather than mere presence.
        assert!(
            width("⏷ 3 more") > width("⏷ 3"),
            "the chevron string scales"
        );
        assert!(width("Open…") > width("Open"), "the ellipsis has a width");
        assert!(
            width("MMMM") > width("iiii"),
            "capitals must be wider than the narrow letters, or the advance table \
             is not being read"
        );
    }

    /// The header of the assembled file is a well-formed `sfnt`
    /// directory, checked directly rather than only through `egui`.
    ///
    /// If `egui` ever changes font backends, this test says whether the
    /// bytes or the consumer moved.
    #[test]
    fn the_assembled_bytes_are_a_well_formed_sfnt_directory() {
        let bytes = font_bytes();
        assert_eq!(&bytes[0..4], &[0x00, 0x01, 0x00, 0x00], "TrueType magic");
        let num_tables = u16::from_be_bytes([bytes[4], bytes[5]]) as usize;
        assert_eq!(num_tables, 7);

        let mut previous = [0_u8; 4];
        for i in 0..num_tables {
            let rec = 12 + 16 * i;
            let tag: [u8; 4] = bytes[rec..rec + 4].try_into().expect("four bytes");
            assert!(
                tag > previous,
                "table tags must ascend; {:?} followed {:?}",
                std::str::from_utf8(&tag),
                std::str::from_utf8(&previous)
            );
            previous = tag;

            let offset =
                u32::from_be_bytes(bytes[rec + 8..rec + 12].try_into().expect("four bytes"))
                    as usize;
            let length =
                u32::from_be_bytes(bytes[rec + 12..rec + 16].try_into().expect("four bytes"))
                    as usize;
            assert!(
                offset + length <= bytes.len(),
                "table {:?} runs past the end of the file",
                std::str::from_utf8(&tag)
            );
            assert_eq!(offset % 4, 0, "tables are four-byte aligned");
        }
    }
}
