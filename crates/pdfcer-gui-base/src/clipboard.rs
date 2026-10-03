//! # `clipboard` — the bytes a copy-out places, and the order they go in
//!
//!
//! > *"Also I'd like to be able to copy and paste anything to other software -
//! > like copy and paste vector graphics into word or inkscape for example if
//! > possible."*
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/clipboard.md`.

pub mod place;
pub mod snapshot;

use pdfcer_render::tiny_skia::Pixmap;

/// One entry on the clipboard: a name Windows knows it by, and what the bytes
/// under that name are for.
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
    #[must_use]
    pub const fn is_registered(self) -> bool {
        matches!(self, Self::Svg | Self::Png)
    }
}

/// **The placement order, and it is measured rather than chosen.**
pub const ORDER: [ClipFormat; 4] = [
    ClipFormat::Svg,
    ClipFormat::Emf,
    ClipFormat::Png,
    ClipFormat::DibV5,
];

/// What a copy-out would put on the clipboard.
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
    /// The pixmap rather than the PNG bytes, because `CF_DIBV5` wants
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

    /// **Whether placing this payload would give Word a flat picture.**
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

/// **The SVG payload exactly as Chromium writes it: UTF-8, plus one NUL.**
#[must_use]
pub fn svg_payload(svg: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(svg.len().saturating_add(1));
    out.extend_from_slice(svg.as_bytes());
    out.push(0);
    out
}

/// `CF_DIBV5` bytes for a premultiplied RGBA pixmap.
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
#[must_use]
pub fn pixels_per_metre(dpi: f32) -> u32 {
    if dpi.is_finite() && dpi > 0.0 {
        crate::units::pixels_per_metre(f64::from(dpi))
            .round()
            .max(0.0) as u32
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The placement order is SVG, EMF, PNG, DIB — and nothing else.**
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

    /// **The two vector formats come before the two raster ones.**
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

    /// **The SVG entry carries one trailing NUL and the source string
    /// does not.**
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

    /// **The NUL is not in the payload struct**, so the same string can be
    /// written to a `.svg` file.
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

    /// **A payload with rasters and no vectors is refusable, by name.**
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

    /// **`formats()` reports what would be placed, in `ORDER`** — never in
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

    /// **The two registered names are exactly `image/svg+xml` and `PNG`.**
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

    /// **The DIB header is 124 bytes, top-down, `BI_BITFIELDS`, BGRA.**
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

    /// **The pixels are premultiplied BGRA, not straight alpha.**
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

    /// **Pixels per metre is the exact inch, rounded to nearest.**
    #[test]
    fn the_dib_resolution_is_the_exact_inch_rounded_to_nearest() {
        assert_eq!(pixels_per_metre(300.0), 11811);
        assert_eq!(pixels_per_metre(96.0), 3780);
        assert_eq!(pixels_per_metre(72.0), 2835);
        for nonsense in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            assert_eq!(pixels_per_metre(nonsense), 0, "{nonsense}");
        }
    }

    /// A zero-area pixmap cannot exist, and a one-pixel one produces a
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
