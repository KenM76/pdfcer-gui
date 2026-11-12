//! # `clipboard` — the bytes a copy-out places, and the order they go in
//!
//!
//! > *"Also I'd like to be able to copy and paste anything to other software -
//! > like copy and paste vector graphics into word or inkscape for example if
//! > possible."*
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/clipboard.md`.

pub mod place;
pub mod richtext;
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
    /// Registered `"application/pdf"` — a complete one-page PDF whose page is
    /// what was copied.
    ///
    /// Only a snapshot copy carries it, and it is placed last, so no program
    /// that took an earlier format changes its pick. pdfcer's own paste reads
    /// it back as a drawing.
    Pdf,
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
            Self::Pdf => "application/pdf",
        }
    }

    /// Whether this format's name must be registered at run time
    /// (`RegisterClipboardFormat`) rather than being a predefined `CF_*`
    /// constant.
    #[must_use]
    pub const fn is_registered(self) -> bool {
        matches!(self, Self::Svg | Self::Png | Self::Pdf)
    }
}

/// **The placement order, and it is measured rather than chosen.**
pub const ORDER: [ClipFormat; 5] = [
    ClipFormat::Svg,
    ClipFormat::Emf,
    ClipFormat::Png,
    ClipFormat::DibV5,
    ClipFormat::Pdf,
];

/// What a copy-out would put on the clipboard.
#[derive(Debug, Default, Clone)]
pub struct CopyPayload {
    /// The SVG document, as `pdfcer_render::svg::export_svg_view` produced it.
    /// The trailing NUL is **not** in here — it is added by [`svg_payload`] at
    /// placement, so that the same string can also be written to a file.
    pub svg: Option<String>,
    /// [MS-EMF] bytes from `pdfcer_emf::export_emf_view`.
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
    /// A one-page PDF of what was copied, for [`ClipFormat::Pdf`].
    pub pdf: Option<Vec<u8>>,
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
                ClipFormat::Pdf => self.pdf.is_some(),
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

/// A picture whose sizes do not fit `BITMAPV5HEADER`'s fields: a side past
/// `i32::MAX`, more than `u32::MAX` bytes of pixels, or a resolution past
/// `i32::MAX` pixels per metre. Refused, never truncated: a wrapped field is a
/// header that tells the reader the wrong size and pastes noise.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DibTooLarge {
    /// The picture's width, pixels.
    pub width: u32,
    /// The picture's height, pixels.
    pub height: u32,
}

/// `CF_DIBV5` bytes for a premultiplied RGBA pixmap, or why its sizes do not
/// fit the header.
pub fn dib_v5(pixmap: &Pixmap, pixels_per_metre: u32) -> Result<Vec<u8>, DibTooLarge> {
    let header = dib_v5_header(pixmap.width(), pixmap.height(), pixels_per_metre)?;
    let refused = DibTooLarge {
        width: pixmap.width(),
        height: pixmap.height(),
    };
    let pixels = pixmap.pixels();
    let len = pixels
        .len()
        .checked_mul(4)
        .and_then(|bytes| bytes.checked_add(DIB_V5_HEADER_LEN))
        .ok_or(refused)?;
    let mut out = Vec::new();
    // An allocation that cannot be had is the same refusal, not an abort.
    out.try_reserve_exact(len).map_err(|_| refused)?;
    out.extend_from_slice(&header);
    for px in pixels {
        out.extend_from_slice(&[px.blue(), px.green(), px.red(), px.alpha()]);
    }
    Ok(out)
}

/// The `BITMAPV5HEADER` for a top-down 32-bit BGRA picture of `width` by
/// `height` pixels. An array, so its length is the structure's by type.
pub fn dib_v5_header(
    width: u32,
    height: u32,
    pixels_per_metre: u32,
) -> Result<[u8; DIB_V5_HEADER_LEN], DibTooLarge> {
    let refused = DibTooLarge { width, height };
    let image_bytes = width
        .checked_mul(4)
        .and_then(|row| row.checked_mul(height))
        .ok_or(refused)?;
    let wide = i32::try_from(width).map_err(|_| refused)?;
    // Negative: top-down. `i32::try_from` bounds it, so the negation cannot wrap.
    let tall = -i32::try_from(height).map_err(|_| refused)?;
    let ppm = i32::try_from(pixels_per_metre).map_err(|_| refused)?;

    let mut out = [0u8; DIB_V5_HEADER_LEN];
    let mut at = 0;
    let mut put = |bytes: &[u8]| {
        out[at..at + bytes.len()].copy_from_slice(bytes);
        at += bytes.len();
    };
    put(&124u32.to_le_bytes()); // bV5Size
    put(&wide.to_le_bytes()); // bV5Width
    put(&tall.to_le_bytes()); // bV5Height
    put(&1u16.to_le_bytes()); // bV5Planes
    put(&32u16.to_le_bytes()); // bV5BitCount
    put(&BI_BITFIELDS.to_le_bytes()); // bV5Compression
    put(&image_bytes.to_le_bytes()); // bV5SizeImage
    put(&ppm.to_le_bytes()); // bV5XPelsPerMeter
    put(&ppm.to_le_bytes()); // bV5YPelsPerMeter
    put(&0u32.to_le_bytes()); // bV5ClrUsed
    put(&0u32.to_le_bytes()); // bV5ClrImportant
    put(&0x00FF_0000u32.to_le_bytes()); // bV5RedMask
    put(&0x0000_FF00u32.to_le_bytes()); // bV5GreenMask
    put(&0x0000_00FFu32.to_le_bytes()); // bV5BlueMask
    put(&0xFF00_0000u32.to_le_bytes()); // bV5AlphaMask
    put(&LCS_SRGB.to_le_bytes()); // bV5CSType
    put(&[0u8; 36]); // bV5Endpoints — unused for sRGB
    put(&[0u8; 12]); // bV5GammaRed, Green, Blue
    put(&LCS_GM_IMAGES.to_le_bytes()); // bV5Intent
    put(&[0u8; 12]); // bV5ProfileData, ProfileSize, Reserved
    // Every field is written: a short write would leave zeros where the
    // reader expects a field, which the type alone cannot see.
    if at != DIB_V5_HEADER_LEN {
        return Err(refused);
    }
    Ok(out)
}

/// `sizeof(BITMAPV5HEADER)`. Fixed by the Win32 structure; a reader takes the
/// first `u32` as the header length and skips exactly that far to the pixels,
/// so a wrong value here does not fail — it reads pixels from the wrong offset
/// and pastes noise.
pub const DIB_V5_HEADER_LEN: usize = 124;

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

    /// **The placement order is SVG, EMF, PNG, DIB, then PDF — and nothing
    /// else.**
    #[test]
    fn the_placement_order_is_the_measured_one() {
        assert_eq!(
            ORDER,
            [
                ClipFormat::Svg,
                ClipFormat::Emf,
                ClipFormat::Png,
                ClipFormat::DibV5,
                ClipFormat::Pdf
            ],
            "the order is MEASURED — SVG first is what makes Word store an \
             svgBlip and place the shape at the page's physical size; EMF \
             second is LibreOffice 24.x's only vector route. PDF goes last so \
             it changes no earlier pick. Reordering these changes what every \
             pasting application receives, silently."
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

    /// **The registered names are exactly `image/svg+xml`, `PNG` and
    /// `application/pdf`.**
    #[test]
    fn the_registered_names_are_byte_exact() {
        assert_eq!(ClipFormat::Svg.name(), "image/svg+xml");
        assert_eq!(ClipFormat::Png.name(), "PNG");
        assert_eq!(ClipFormat::Pdf.name(), "application/pdf");
        assert!(ClipFormat::Svg.is_registered());
        assert!(ClipFormat::Png.is_registered());
        assert!(ClipFormat::Pdf.is_registered());
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
        let dib = dib_v5(&pixmap, 11811).expect("a 2x1 picture fits");

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
        let dib = dib_v5(&pixmap, 0).expect("a 1x1 picture fits");
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
        assert_eq!(dib_v5(&pixmap, 0).map(|d| d.len()), Ok(124 + 4));
        assert!(
            Pixmap::new(0, 0).is_none(),
            "tiny_skia refuses a zero-area pixmap, so `dib_v5` can never be \
             handed one and needs no guard for it"
        );
    }

    /// **Sizes past the header's fields are refused, never wrapped.** Built
    /// from the header alone: no pixmap of these sizes can be allocated.
    #[test]
    fn a_picture_past_four_gibibytes_is_refused() {
        // 32,768 x 32,768 x 4 = 2^32, one past `u32::MAX`; both sides fit `i32`.
        assert_eq!(
            dib_v5_header(32_768, 32_768, 0),
            Err(DibTooLarge {
                width: 32_768,
                height: 32_768
            })
        );
        // One row fewer fits, and says so in bV5SizeImage.
        let header = dib_v5_header(32_768, 32_767, 0).expect("under 4 GiB fits");
        assert_eq!(
            u32::from_le_bytes([header[20], header[21], header[22], header[23]]),
            32_768 * 4 * 32_767
        );
    }

    #[test]
    fn a_side_past_i32_max_is_refused() {
        let past = i32::MAX as u32 + 1;
        assert!(dib_v5_header(past, 0, 0).is_err(), "width");
        assert!(dib_v5_header(0, past, 0).is_err(), "height");
        let header = dib_v5_header(0, i32::MAX as u32, 0).expect("i32::MAX tall, no pixels, fits");
        assert_eq!(
            i32::from_le_bytes([header[8], header[9], header[10], header[11]]),
            -i32::MAX
        );
    }

    #[test]
    fn a_resolution_past_i32_max_is_refused() {
        assert!(dib_v5_header(1, 1, i32::MAX as u32 + 1).is_err());
        let header = dib_v5_header(1, 1, i32::MAX as u32).expect("i32::MAX ppm fits");
        assert_eq!(
            i32::from_le_bytes([header[24], header[25], header[26], header[27]]),
            i32::MAX
        );
    }
}
