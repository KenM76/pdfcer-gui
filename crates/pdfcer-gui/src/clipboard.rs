//! # `clipboard` — the bytes a copy-out places, and the order they go in
//!
//! This module builds and orders the payload for `OPERATOR_REQUESTS.md`
//! **O120**'s second half — the operator's own words, 2026-09-03:
//!
//! > *"Also I'd like to be able to copy and paste anything to other software -
//! > like copy and paste vector graphics into word or inkscape for example if
//! > possible."*
//!
//! ## Where the two halves live
//!
//! | half | where | what it decides |
//! |---|---|---|
//! | **the bytes and the order** | this file | [`ORDER`], [`svg_payload`]'s trailing NUL, [`dib_v5`]'s premultiplied top-down `BI_BITFIELDS` framing, [`pixels_per_metre`], and [`CopyPayload::degrades_word_to_a_picture`] |
//! | **the placement** | [`place`], over `crates/native-clipboard` | producing the payload from a real page or selection, refusing a set that would degrade, and the one ordered transaction |
//!
//! ★★★ The seam is not tidiness. **What decides whether a paste into Word
//! arrives as an editable graphic or as a flat picture is entirely in this
//! file**, and none of it needs a syscall:
//!
//! * **the order** the formats are placed in ([`ORDER`]) — measured by the
//!   engine against a real Word paste, not chosen;
//! * **the trailing NUL** on the SVG ([`svg_payload`]) — Chromium's exact
//!   byte shape, which is what Microsoft validated Office against;
//! * **the DIB's premultiplied, top-down, `BI_BITFIELDS` framing**
//!   ([`dib_v5`]) — a straight-alpha DIB looks wrong in exactly the readers
//!   that fall back to it.
//!
//! Every one of those is pure and every one is asserted below, against bytes
//! rather than against a clipboard.
//!
//! ## ★★★ The property the whole feature rests on: half is worse than none
//!
//! A copy-out that places only the raster formats is *worse than no copy-out at
//! all*, because Word silently degrades such a paste to a flat picture and the
//! operator has no way to tell that from a feature that works. The engine's
//! note is unambiguous: *"place only the raster formats and it degrades to a
//! plain picture."*
//!
//! ⇒ [`CopyPayload::degrades_word_to_a_picture`] is the predicate that answers
//! it, and [`place`] asks it **before framing a single byte** — see that
//! module's `staged`. `native-clipboard` completes the property from the other
//! end by creating every OS handle *before* the clipboard is opened, so a
//! refusal anywhere leaves the operator's clipboard exactly as it was.
//!
//! ## ★★ Why `native-clipboard` is a crate and not a module here
//!
//! `CF_ENHMETAFILE` is a **GDI handle**, not an `HGLOBAL`, so the metafile must
//! go on through `SetEnhMetaFileBits` followed by `SetClipboardData` — two
//! `unsafe` calls with a real ownership contract between them (the handle
//! becomes the clipboard's on success and must be deleted by the caller on
//! failure). `crates/pdfcer-gui/src/lib.rs` and `main.rs` both open with
//! `#![forbid(unsafe_code)]`, and `forbid` cannot be relaxed from the inside —
//! that is the whole point of choosing it over `deny`.
//!
//! ★ **This project had already answered the same question once.**
//! `crates/native-window` exists for four `user32` calls, and its manifest says
//! why in one sentence: *"Its own crate rather than a module here, because this
//! crate's `#![forbid(unsafe_code)]` is a claim worth keeping and `forbid`
//! cannot be relaxed from the inside."* `crates/native-clipboard` is that
//! answer applied one question further on, built to the same shape: no
//! dependencies, hand-written `extern` declarations, a `SAFETY` comment per
//! call, and RAII types rather than careful sequencing.
//!
//! ⇒ Neither `clipboard-win` nor `arboard` was adopted, and the second could
//! not have been: `arboard` has **no registered-format API at all**, so it
//! cannot place entries 1 and 3 of [`ORDER`] — the two that make a Word paste
//! land as an editable graphic. The engine's note says so in one line:
//! *"`arboard` cannot do 1 or 4 (no registered-format API); use
//! `clipboard-win` directly as the CLI does."*
//!
//! ## ★★ Why this file stays pure
//!
//! Because the part that is easy to get wrong is the part that needs no
//! dependency, and keeping the two apart is what makes the hard half testable
//! at all. Everything above is decided here, in code that crosses no syscall
//! and can therefore be asserted byte for byte. [`place`] is the plumbing:
//! produce a payload, ask the predicate, hand the ordered set over.
//!
//! ⚠ **No test in this file — or in [`place`] — touches the real clipboard,
//! and none ever should.** The clipboard is global state on the operator's
//! machine: a test that placed bytes would silently destroy whatever he had
//! copied, from a `cargo test` run he did not connect to his clipboard. Every
//! assertion here is on the bytes that *would* be placed, and the `unsafe`
//! placement inside `native-clipboard` is verified **by construction and by
//! review** — stated plainly rather than dressed up as coverage.
//!
//! ## Sources
//!
//! * `D:\Dev\FeatureRequests\pdfce_FeatureRequests\open\note_export_to_png_jpeg_svg_and_copy_out_ship_here_is_what_to_wire.md`
//!   and its addendum — the measured order, the Word paste, the EMF placement.
//! * `crates/pdfcer-cli/src/clipboard.rs` in the engine tree — the worked
//!   ~60-line placement [`place`] mirrors, with one deliberate departure
//!   documented on `native_clipboard::place`: the metafile handle is created
//!   *before* the clipboard is opened, not inside the guard.
//! * The engine's `docs/clipboard-interop-survey.md` §7 — application source
//!   at pinned revisions, which is where the reader preferences come from.

pub mod place;

use pdfcer_render::tiny_skia::Pixmap;

/// One entry on the clipboard: a name Windows knows it by, and what the bytes
/// under that name are for.
///
/// A shell enum rather than a `u32` format id, because a format id cannot be
/// matched on, cannot be printed in a disclosure, and — for the three
/// registered names — does not exist until `RegisterClipboardFormat` has been
/// called at run time. What is *stable* about a clipboard format is its name
/// and its position in [`ORDER`], and those are what this type carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ClipFormat {
    /// Registered `"image/svg+xml"` — UTF-8 SVG plus one trailing NUL.
    ///
    /// The one that makes a Word paste an **editable graphic**: Word stores it
    /// as `svgBlip` in the OOXML and the shape lands at the page's physical
    /// size. Inkscape's second preference, above EMF and above PDF.
    Svg,
    /// `CF_ENHMETAFILE` — an [MS-EMF] metafile, placed as a GDI handle.
    ///
    /// LibreOffice 24.x's **only** vector route on Windows: it cannot read a
    /// foreign SVG clipboard entry before 25.2. Also what Office's *Paste
    /// Special ▸ Picture (Enhanced Metafile)* takes, and what Visio,
    /// CorelDRAW and most CAD importers read.
    Emf,
    /// Registered `"PNG"` — PNG file bytes, straight alpha, DPI in `pHYs`.
    ///
    /// Office's preferred raster, and what Paint.NET, GIMP, Firefox, Chromium
    /// and Snip & Sketch reach for.
    Png,
    /// `CF_DIBV5` — `BITMAPV5HEADER` plus premultiplied top-down BGRA.
    ///
    /// For readers older than the `"PNG"` convention. Windows synthesises
    /// `CF_DIB` and `CF_BITMAP` from it, so placing it is what makes a paste
    /// work in programs that have never heard of any of the above.
    DibV5,
}

impl ClipFormat {
    /// The name Windows knows this format by.
    ///
    /// For [`Self::Svg`] and [`Self::Png`] this is the string handed to
    /// `RegisterClipboardFormat` **verbatim**, and it is case- and
    /// byte-sensitive: `"PNG"` is the registered name every browser and Office
    /// itself uses, and `"png"` would register a different, private format
    /// that nothing reads.
    ///
    /// For the two predefined formats it is the Win32 constant's name, which
    /// is not passed to any API and exists so a disclosure can say which
    /// formats went on in words an operator can search for.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            // ui-text-exempt: registered clipboard format names, passed to
            // `RegisterClipboardFormat` and matched by other applications.
            // These are wire identifiers, never prose.
            Self::Svg => "image/svg+xml",
            Self::Emf => "CF_ENHMETAFILE",
            Self::Png => "PNG",
            Self::DibV5 => "CF_DIBV5",
        }
    }

    /// Whether this format's name must be registered at run time
    /// (`RegisterClipboardFormat`) rather than being a predefined `CF_*`
    /// constant.
    ///
    /// ★ This is the whole of why `arboard` cannot do this job: it offers no
    /// API for a registered format, and the two formats that return `true`
    /// here are the two that make a Word paste editable and an Inkscape paste
    /// vector.
    #[must_use]
    pub const fn is_registered(self) -> bool {
        matches!(self, Self::Svg | Self::Png)
    }
}

/// ★★★ **The placement order, and it is measured rather than chosen.**
///
/// # Why an order exists at all
///
/// A pasting application "typically retrieves … the first format it
/// recognizes". So the order is not a preference — it *is* the design. Every
/// application that can read two of these will take whichever pdfcer placed
/// first, and there is no second chance to influence that at paste time.
///
/// # What each position buys, and who it buys it from
///
/// | # | format | the reader it is there for |
/// |---|---|---|
/// | 1 | [`ClipFormat::Svg`] | Word / PowerPoint / Excel, which store it as `svgBlip` and place the shape at the page's physical size; Inkscape, whose own `clipboard.cpp` ranks SVG above EMF above PDF; LibreOffice ≥ 25.2 |
/// | 2 | [`ClipFormat::Emf`] | LibreOffice 24.x, which has no other vector route on Windows; Office *Paste Special ▸ Picture (Enhanced Metafile)*; Visio, CorelDRAW, CAD importers |
/// | 3 | [`ClipFormat::Png`] | Paint.NET, GIMP, browsers, Snip & Sketch — and Office, when the operator deliberately pastes as a picture |
/// | 4 | [`ClipFormat::DibV5`] | everything older than the `"PNG"` convention; Windows synthesises `CF_DIB` and `CF_BITMAP` from it |
///
/// # ★★★ The property that makes a partial implementation harmful
///
/// **The two vector entries come first, and if they are absent Word's paste
/// silently becomes a flat picture.** Not an error, not a warning — a picture
/// that looks correct at 100% and cannot be scaled, recoloured or ungrouped.
/// An operator would report that as *"pdfcer's copy doesn't paste as
/// vectors"*, which is indistinguishable from the feature not existing, except
/// that it costs them the time to discover it.
///
/// ⇒ Which is why this module builds bytes and places nothing. Measured by the
/// engine on 2026-09-03, `hello.pdf` through `pdfcer copy-page`, pasted into a
/// throw-away Word document driven by combridge: **one inline shape at
/// 200.2 × 120.0 pt — the page's physical size — with `svgBlip` in the
/// OOXML.** The same paste with only the raster formats placed: a plain
/// picture.
///
/// # Why `application/pdf` is not here
///
/// The engine's note offers it as an optional fifth entry and says only
/// Inkscape reads it — and Inkscape already takes the SVG from position 1, so
/// it would be a payload for nobody. It is also the most expensive one to
/// build (a one-page PDF through `ObjectClip::to_pdf`), which is a real cost
/// on a copy the operator expects to be instant.
pub const ORDER: [ClipFormat; 4] = [
    ClipFormat::Svg,
    ClipFormat::Emf,
    ClipFormat::Png,
    ClipFormat::DibV5,
];

/// What a copy-out would put on the clipboard.
///
/// Every field is optional so that a caller which could not produce one
/// payload still places the rest — but see [`ORDER`] on why "the rest" is a
/// dangerous thing to place when the missing one is [`ClipFormat::Svg`] or
/// [`ClipFormat::Emf`]. [`CopyPayload::degrades_word_to_a_picture`] is the
/// predicate that answers it, and a caller is expected to refuse rather than
/// place a payload for which it returns `true`.
#[derive(Debug, Default, Clone)]
pub struct CopyPayload {
    /// The SVG document, as `pdfcer_render::svg::export_svg_view` produced it.
    /// The trailing NUL is **not** in here — it is added by [`svg_payload`] at
    /// placement, so that the same string can also be written to a file.
    pub svg: Option<String>,
    /// [MS-EMF] bytes from `pdfcer_render::emf::export_emf_view`.
    pub emf: Option<Vec<u8>>,
    /// PNG file bytes from `pdfcer_render::export::encode_png`, straight
    /// alpha, with the resolution in `pHYs`.
    pub png: Option<Vec<u8>>,
    /// The raster the PNG was made from, for [`dib_v5`].
    ///
    /// ★ The pixmap rather than the PNG bytes, because `CF_DIBV5` wants
    /// **premultiplied** BGRA and the PNG carries straight alpha. Re-deriving
    /// one from the other would mean decoding the PNG we just encoded and
    /// premultiplying it back — two conversions to arrive at the buffer we
    /// already had.
    pub pixmap: Option<Pixmap>,
    /// Pixels per metre for the DIB header — `dpi / 0.0254` — or 0 for
    /// "unspecified".
    pub pixels_per_metre: u32,
}

impl CopyPayload {
    /// The formats this payload can supply, **in [`ORDER`]**.
    ///
    /// ★ Derived from `ORDER` by filtering rather than by a hand-written list,
    /// so the order cannot be stated correctly in one place and wrongly in
    /// another. A second list is a second answer, and the one that would go
    /// stale is whichever is not the one being read at the time.
    #[must_use]
    pub fn formats(&self) -> Vec<ClipFormat> {
        ORDER
            .into_iter()
            .filter(|format| match format {
                ClipFormat::Svg => self.svg.is_some(),
                ClipFormat::Emf => self.emf.is_some(),
                ClipFormat::Png => self.png.is_some(),
                ClipFormat::DibV5 => self.pixmap.is_some(),
            })
            .collect()
    }

    /// ★★★ **Whether placing this payload would give Word a flat picture.**
    ///
    /// True when there is a raster to place and no vector to place before it.
    /// A caller must refuse rather than place such a payload: the paste
    /// succeeds, looks right, and is not what was asked for, and nothing in
    /// the receiving application says so.
    ///
    /// ⇒ Stated as a predicate on the payload rather than as a comment on the
    /// placement function, because it is a property of *what was produced* and
    /// the producer is where it can still be fixed — by rendering the SVG
    /// again, or by declining the copy with a sentence.
    #[must_use]
    pub fn degrades_word_to_a_picture(&self) -> bool {
        let has_vector = self.svg.is_some() || self.emf.is_some();
        let has_raster = self.png.is_some() || self.pixmap.is_some();
        has_raster && !has_vector
    }

    /// Whether there is anything at all to place.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.formats().is_empty()
    }
}

/// ★★★ **The SVG payload exactly as Chromium writes it: UTF-8, plus one NUL.**
///
/// # Why a trailing NUL on a format whose length is already known
///
/// Because that is the byte shape Microsoft validated Office against. Chromium
/// (≥ M127) writes the SVG to `"image/svg+xml"` NUL-terminated, and Office's
/// importer was tested against Chromium's clipboard rather than against a
/// specification — so the NUL is a compatibility fact, not a framing
/// requirement. `HGLOBAL` clipboard entries carry their own size; nothing
/// *needs* the terminator.
///
/// ⚠ The failure mode if it is omitted is the worst kind: it very likely works
/// in most readers, and the one that reads past the end or refuses the entry
/// does so on somebody else's machine, in a version of Office nobody here has.
/// It costs one byte. It goes on.
///
/// # Why the NUL is added here and is not part of `CopyPayload::svg`
///
/// So the same `String` can be written to a `.svg` file, which must **not**
/// have one. A NUL inside an XML document is not permitted by XML 1.0 §2.2 at
/// all, and a file carrying one is refused by strict parsers. Keeping the
/// terminator at the placement boundary means the file path and the clipboard
/// path cannot accidentally share it in either direction.
#[must_use]
pub fn svg_payload(svg: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(svg.len().saturating_add(1));
    out.extend_from_slice(svg.as_bytes());
    out.push(0);
    out
}

/// `CF_DIBV5` bytes for a premultiplied RGBA pixmap.
///
/// A `BITMAPV5HEADER` (124 bytes) followed by 32-bit-per-pixel BGRA rows,
/// **top-down** (a negative height), `BI_BITFIELDS` with explicit channel
/// masks, and the sRGB colour space.
///
/// # ★★ Premultiplied, and why the format below it is not
///
/// `CF_DIBV5`'s alpha convention is not written down anywhere normative — it
/// is whatever the ecosystem settled on. Chromium writes premultiplied
/// (`CreateDIBV5ImageDataFromN32SkBitmap`) and Mozilla settled on reading
/// premultiplied, so a straight-alpha DIB looks wrong — dark haloes around
/// anything soft-edged — in precisely the readers that fall back to this
/// format at all.
///
/// ⇒ Which is why `"PNG"` is placed **before** it. A PNG's alpha is straight
/// and unambiguous (ISO 15948 §6.1), so every reader that understands the
/// registered `"PNG"` name gets the unambiguous answer, and only readers old
/// enough to need `CF_DIBV5` are exposed to the convention.
///
/// `tiny_skia` stores premultiplied RGBA natively, so the per-pixel work here
/// is a channel reorder and nothing else — no multiply, no divide, no
/// rounding, and therefore no place for the conversion to lose a value.
///
/// # The header fields that are not obvious
///
/// * `bV5Height` is **negative**. A positive height means bottom-up, which is
///   the DIB default and would paste every copy upside down.
/// * `bV5Compression` is `BI_BITFIELDS` (3) rather than `BI_RGB` (0), because
///   `BI_RGB` at 32 bpp leaves the fourth byte formally undefined and readers
///   disagree about whether it is alpha or padding. The explicit masks remove
///   the question.
/// * `bV5CSType` is `LCS_sRGB` — the four bytes `'sRGB'` as a little-endian
///   `u32`, which is `0x7352_4742`. The endpoint and gamma fields that follow
///   are unused for a named colour space and are written as zero.
#[must_use]
pub fn dib_v5(pixmap: &Pixmap, pixels_per_metre: u32) -> Vec<u8> {
    let (width, height) = (pixmap.width(), pixmap.height());
    let row_bytes = width as usize * 4;
    let image_bytes = row_bytes * height as usize;
    let mut out = Vec::with_capacity(DIB_V5_HEADER_LEN + image_bytes);
    let u32le = |v: u32| v.to_le_bytes();
    let i32le = |v: i32| v.to_le_bytes();

    out.extend_from_slice(&u32le(DIB_V5_HEADER_LEN as u32)); // bV5Size
    out.extend_from_slice(&i32le(width as i32)); // bV5Width
    out.extend_from_slice(&i32le(-(height as i32))); // bV5Height — negative: top-down
    out.extend_from_slice(&1u16.to_le_bytes()); // bV5Planes
    out.extend_from_slice(&32u16.to_le_bytes()); // bV5BitCount
    out.extend_from_slice(&u32le(BI_BITFIELDS)); // bV5Compression
    out.extend_from_slice(&u32le(image_bytes as u32)); // bV5SizeImage
    out.extend_from_slice(&i32le(pixels_per_metre as i32)); // bV5XPelsPerMeter
    out.extend_from_slice(&i32le(pixels_per_metre as i32)); // bV5YPelsPerMeter
    out.extend_from_slice(&u32le(0)); // bV5ClrUsed
    out.extend_from_slice(&u32le(0)); // bV5ClrImportant
    out.extend_from_slice(&u32le(0x00FF_0000)); // bV5RedMask
    out.extend_from_slice(&u32le(0x0000_FF00)); // bV5GreenMask
    out.extend_from_slice(&u32le(0x0000_00FF)); // bV5BlueMask
    out.extend_from_slice(&u32le(0xFF00_0000)); // bV5AlphaMask
    out.extend_from_slice(&u32le(LCS_SRGB)); // bV5CSType
    out.extend_from_slice(&[0u8; 36]); // bV5Endpoints — unused for sRGB
    out.extend_from_slice(&u32le(0)); // bV5GammaRed
    out.extend_from_slice(&u32le(0)); // bV5GammaGreen
    out.extend_from_slice(&u32le(0)); // bV5GammaBlue
    out.extend_from_slice(&u32le(LCS_GM_IMAGES)); // bV5Intent
    out.extend_from_slice(&u32le(0)); // bV5ProfileData
    out.extend_from_slice(&u32le(0)); // bV5ProfileSize
    out.extend_from_slice(&u32le(0)); // bV5Reserved
    debug_assert_eq!(out.len(), DIB_V5_HEADER_LEN);

    for px in pixmap.pixels() {
        out.extend_from_slice(&[px.blue(), px.green(), px.red(), px.alpha()]);
    }
    out
}

/// `sizeof(BITMAPV5HEADER)`. Fixed by the Win32 structure; a reader takes the
/// first `u32` as the header length and skips exactly that far to the pixels,
/// so a wrong value here does not fail — it reads pixels from the wrong offset
/// and pastes noise.
const DIB_V5_HEADER_LEN: usize = 124;

/// `BI_BITFIELDS` — channel masks are explicit rather than implied.
const BI_BITFIELDS: u32 = 3;

/// `LCS_sRGB` — the four characters `'sRGB'` read as a little-endian `u32`.
const LCS_SRGB: u32 = 0x7352_4742;

/// `LCS_GM_IMAGES` — the rendering intent for pictorial content.
const LCS_GM_IMAGES: u32 = 4;

/// Pixels per metre for a resolution in dots per inch.
///
/// One inch is exactly 0.0254 m (the international inch, fixed by definition
/// since 1959), so this is a conversion rather than an approximation. Rounded
/// to nearest because the DIB field is an integer and a truncation would put
/// a 300 DPI copy at 11,810 rather than 11,811 pixels per metre — which is
/// how a paste ends up a hair's breadth off the page size it should have had.
#[must_use]
pub fn pixels_per_metre(dpi: f32) -> u32 {
    if dpi.is_finite() && dpi > 0.0 {
        (f64::from(dpi) / 0.0254).round().max(0.0) as u32
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★★★ **The placement order is SVG, EMF, PNG, DIB — and nothing else.**
    ///
    /// The single most important assertion in this module. The order was
    /// *measured* by the engine against a real Word paste through combridge,
    /// not chosen for tidiness, and a reader takes the first format it
    /// recognises — so reordering these four silently changes what Word,
    /// LibreOffice and Inkscape each receive, with no error anywhere.
    ///
    /// Asserted as the whole array in one comparison rather than as four
    /// index checks, so a *swap* (the likeliest edit) fails as loudly as a
    /// replacement.
    #[test]
    fn the_placement_order_is_the_measured_one() {
        assert_eq!(
            ORDER,
            [
                ClipFormat::Svg,
                ClipFormat::Emf,
                ClipFormat::Png,
                ClipFormat::DibV5
            ],
            "the order is MEASURED — SVG first is what makes Word store an \
             svgBlip and place the shape at the page's physical size; EMF \
             second is LibreOffice 24.x's only vector route. Reordering these \
             changes what every pasting application receives, silently."
        );
    }

    /// ★★ **The two vector formats come before the two raster ones.**
    ///
    /// Stated as a property rather than as the literal array above, because
    /// it is the property the whole feature rests on and it should survive a
    /// deliberate, considered change to the order of two entries within a
    /// half. If a raster format ever precedes a vector one, Word's paste is a
    /// flat picture and the feature has quietly stopped working.
    #[test]
    fn every_vector_format_precedes_every_raster_one() {
        let first_raster = ORDER
            .iter()
            .position(|f| matches!(f, ClipFormat::Png | ClipFormat::DibV5))
            .expect("a raster format must be placed");
        let last_vector = ORDER
            .iter()
            .rposition(|f| matches!(f, ClipFormat::Svg | ClipFormat::Emf))
            .expect("a vector format must be placed");
        assert!(
            last_vector < first_raster,
            "a raster format placed before a vector one degrades Word's paste \
             to a plain picture: {ORDER:?}"
        );
    }

    /// ★★★ **The SVG entry carries one trailing NUL and the source string
    /// does not.**
    ///
    /// Chromium's exact byte shape, which is what Office was validated
    /// against. Both halves are asserted: the terminator is present, and it is
    /// *one* byte rather than being doubled by a caller that had already added
    /// one.
    #[test]
    fn the_svg_payload_is_utf8_with_exactly_one_trailing_nul() {
        let payload = svg_payload("<svg/>");
        assert_eq!(payload, b"<svg/>\0");
        assert_eq!(
            payload.last(),
            Some(&0u8),
            "Office was validated against Chromium's NUL-terminated shape"
        );
        assert_eq!(
            payload.iter().filter(|b| **b == 0).count(),
            1,
            "exactly one terminator — a doubled one is a different byte string"
        );
        // Non-ASCII survives as UTF-8 rather than being re-encoded.
        let unicode = svg_payload("<svg>é</svg>");
        assert_eq!(&unicode[..unicode.len() - 1], "<svg>é</svg>".as_bytes());
    }

    /// ★ **The NUL is not in the payload struct**, so the same string can be
    /// written to a `.svg` file.
    ///
    /// XML 1.0 §2.2 does not permit a NUL anywhere in a document, so a file
    /// carrying one is refused by strict parsers. This asserts the seam: the
    /// terminator belongs to the clipboard boundary and nowhere else.
    #[test]
    fn the_stored_svg_has_no_terminator_of_its_own() {
        let payload = CopyPayload {
            svg: Some("<svg/>".to_owned()),
            ..CopyPayload::default()
        };
        let stored = payload.svg.as_deref().expect("just set");
        assert!(
            !stored.contains('\0'),
            "a NUL inside the stored SVG would travel into any file written \
             from it, where XML forbids it"
        );
        assert_eq!(svg_payload(stored).len(), stored.len() + 1);
    }

    /// ★★★ **A payload with rasters and no vectors is refusable, by name.**
    ///
    /// The engine's note: *"place only the raster formats and it degrades to
    /// a plain picture."* This is the predicate a caller asks before placing,
    /// and the reason a half-built copy-out is worse than none.
    #[test]
    fn a_raster_only_payload_reports_that_it_would_degrade_words_paste() {
        let raster_only = CopyPayload {
            png: Some(vec![1, 2, 3]),
            ..CopyPayload::default()
        };
        assert!(raster_only.degrades_word_to_a_picture());

        let with_svg = CopyPayload {
            svg: Some("<svg/>".to_owned()),
            png: Some(vec![1, 2, 3]),
            ..CopyPayload::default()
        };
        assert!(!with_svg.degrades_word_to_a_picture());

        // EMF alone is enough: LibreOffice 24.x reads it, and Word's Paste
        // Special reaches it. It is a worse answer than the SVG and it is not
        // a degradation to a picture.
        let with_emf = CopyPayload {
            emf: Some(vec![1, 2, 3]),
            png: Some(vec![1, 2, 3]),
            ..CopyPayload::default()
        };
        assert!(!with_emf.degrades_word_to_a_picture());

        // An empty payload degrades nothing; it places nothing.
        assert!(!CopyPayload::default().degrades_word_to_a_picture());
        assert!(CopyPayload::default().is_empty());
    }

    /// ★★ **`formats()` reports what would be placed, in `ORDER`** — never in
    /// the order the fields were filled.
    #[test]
    fn the_reported_formats_follow_the_order_and_not_the_struct() {
        let payload = CopyPayload {
            png: Some(vec![0]),
            svg: Some("<svg/>".to_owned()),
            emf: Some(vec![0]),
            ..CopyPayload::default()
        };
        assert_eq!(
            payload.formats(),
            vec![ClipFormat::Svg, ClipFormat::Emf, ClipFormat::Png],
            "the SVG was assigned second and must still be reported first"
        );
        // The DIB is keyed off the PIXMAP, not off the PNG: a payload with
        // PNG bytes and no pixmap cannot build a DIB.
        assert!(!payload.formats().contains(&ClipFormat::DibV5));
    }

    /// ★★ **The two registered names are exactly `image/svg+xml` and `PNG`.**
    ///
    /// Byte-for-byte, because `RegisterClipboardFormat` is case-sensitive:
    /// `"png"` registers a different, private format that nothing on the
    /// machine reads, and the copy would appear to succeed.
    #[test]
    fn the_registered_names_are_byte_exact() {
        assert_eq!(ClipFormat::Svg.name(), "image/svg+xml");
        assert_eq!(ClipFormat::Png.name(), "PNG");
        assert!(ClipFormat::Svg.is_registered());
        assert!(ClipFormat::Png.is_registered());
        assert!(
            !ClipFormat::Emf.is_registered() && !ClipFormat::DibV5.is_registered(),
            "CF_ENHMETAFILE and CF_DIBV5 are predefined constants; registering \
             their names would create two private formats nothing reads"
        );
    }

    /// ★★★ **The DIB header is 124 bytes, top-down, `BI_BITFIELDS`, BGRA.**
    ///
    /// Every one of those four is a silent-corruption failure if it is wrong:
    /// a wrong header length reads pixels from the wrong offset, a positive
    /// height pastes the picture upside down, `BI_RGB` leaves the alpha byte
    /// formally undefined, and a channel-order slip turns red into blue.
    #[test]
    fn the_dib_header_is_top_down_bitfields_bgra() {
        let mut pixmap = Pixmap::new(2, 1).expect("2x1 is a valid pixmap");
        // Opaque red. Premultiplied and opaque are the same bytes, so this
        // isolates the CHANNEL ORDER from the premultiply question.
        pixmap.fill(pdfcer_render::tiny_skia::Color::from_rgba8(255, 0, 0, 255));
        let dib = dib_v5(&pixmap, 11811);

        assert_eq!(dib.len(), 124 + 2 * 4, "header plus two BGRA pixels");
        assert_eq!(u32::from_le_bytes([dib[0], dib[1], dib[2], dib[3]]), 124);
        assert_eq!(i32::from_le_bytes([dib[4], dib[5], dib[6], dib[7]]), 2);
        assert_eq!(
            i32::from_le_bytes([dib[8], dib[9], dib[10], dib[11]]),
            -1,
            "a NEGATIVE height is what makes the rows top-down; a positive one \
             pastes every copy upside down"
        );
        assert_eq!(u16::from_le_bytes([dib[14], dib[15]]), 32, "bits per pixel");
        assert_eq!(
            u32::from_le_bytes([dib[16], dib[17], dib[18], dib[19]]),
            3,
            "BI_BITFIELDS — BI_RGB leaves the fourth byte formally undefined"
        );
        assert_eq!(
            i32::from_le_bytes([dib[24], dib[25], dib[26], dib[27]]),
            11811,
            "the resolution travels in bV5XPelsPerMeter"
        );
        // Opaque red as BGRA.
        assert_eq!(&dib[124..128], &[0, 0, 255, 255]);
    }

    /// ★★ **The pixels are premultiplied BGRA, not straight alpha.**
    ///
    /// The convention Chromium writes and Mozilla reads. A straight-alpha DIB
    /// produces dark haloes around soft edges in exactly the readers that fall
    /// back to `CF_DIBV5`, and it looks correct in every reader that does not.
    ///
    /// `tiny_skia` stores premultiplied natively, so what this really asserts
    /// is that nothing on the way out **un**-premultiplies — which is the
    /// tempting "fix" for a channel that looks too dark.
    #[test]
    fn the_dib_pixels_are_premultiplied_and_not_unpremultiplied_on_the_way_out() {
        let mut pixmap = Pixmap::new(1, 1).expect("1x1 is a valid pixmap");
        // Half-opaque red. Premultiplied: R = 255 * 128/255 = 128.
        pixmap.fill(pdfcer_render::tiny_skia::Color::from_rgba8(255, 0, 0, 128));
        let dib = dib_v5(&pixmap, 0);
        let (blue, green, red, alpha) = (dib[124], dib[125], dib[126], dib[127]);
        assert_eq!((blue, green), (0, 0));
        assert_eq!(alpha, 128, "alpha travels unchanged");
        assert!(
            red < 255,
            "premultiplied: a half-transparent full red stores a HALVED red \
             channel. A straight-alpha 255 here means somebody un-premultiplied \
             on the way out, which haloes every soft edge in the readers that \
             use this format. Got {red}"
        );
        assert_eq!(
            red,
            pixmap.pixels()[0].red(),
            "the channel is copied, never recomputed — tiny_skia already \
             stores premultiplied, so any arithmetic here is a second rounding"
        );
    }

    /// ★ **Pixels per metre is the exact inch, rounded to nearest.**
    ///
    /// 300 DPI / 0.0254 is 11811.02…, so truncation gives 11810 and a paste
    /// lands very slightly wrong. A nonsense resolution yields 0, which is
    /// `CF_DIBV5`'s own "unspecified" and is better than a garbage number a
    /// reader would honour.
    #[test]
    fn the_dib_resolution_is_the_exact_inch_rounded_to_nearest() {
        assert_eq!(pixels_per_metre(300.0), 11811);
        assert_eq!(pixels_per_metre(96.0), 3780);
        assert_eq!(pixels_per_metre(72.0), 2835);
        for nonsense in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            assert_eq!(pixels_per_metre(nonsense), 0, "{nonsense}");
        }
    }

    /// ★ A zero-area pixmap cannot exist, and a one-pixel one produces a
    /// header plus four bytes — the smallest well-formed DIB.
    #[test]
    fn the_smallest_dib_is_a_header_and_one_pixel() {
        let pixmap = Pixmap::new(1, 1).expect("1x1 is a valid pixmap");
        assert_eq!(dib_v5(&pixmap, 0).len(), 124 + 4);
        assert!(
            Pixmap::new(0, 0).is_none(),
            "tiny_skia refuses a zero-area pixmap, so `dib_v5` can never be \
             handed one and needs no guard for it"
        );
    }
}
