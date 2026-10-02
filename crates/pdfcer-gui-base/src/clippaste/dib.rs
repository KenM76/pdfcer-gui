//! Decoding a clipboard device-independent bitmap (`CF_DIB`, `CF_DIBV5`) to
//! straight-alpha RGBA, top row first.
//!
//! Contract: accepts a `BITMAPINFOHEADER`, `BITMAPV4HEADER` or
//! `BITMAPV5HEADER` followed by its masks, palette and pixels, exactly as the
//! clipboard carries it (no `BITMAPFILEHEADER`). Supported: 24- and 32-bit
//! `BI_RGB`, 16- and 32-bit `BI_BITFIELDS`/`BI_ALPHABITFIELDS`, and 1-, 4- and
//! 8-bit palettes. A 32-bit bitmap whose fourth bytes are all zero is opaque:
//! `BI_RGB` defines that byte as unused and most producers leave it zero.

/// A decoded bitmap.
#[derive(Debug, Clone, PartialEq)]
pub struct Rgba {
    /// Pixels across.
    pub width: u32,
    /// Pixels down.
    pub height: u32,
    /// Straight (not premultiplied) RGBA, `width * height * 4` bytes, top row
    /// first.
    pub pixels: Vec<u8>,
    /// Pixels per inch from the header, when it states one.
    pub dpi: Option<f32>,
}

/// Why a bitmap could not be decoded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DibError {
    /// Shorter than its header says.
    Truncated,
    /// A header size, depth or compression this decoder does not read.
    Unsupported,
    /// Zero or absurd dimensions.
    Size,
}

const BI_RGB: u32 = 0;
const BI_BITFIELDS: u32 = 3;
const BI_ALPHABITFIELDS: u32 = 6;
/// The largest side accepted, so a corrupt header cannot ask for gigabytes.
const MAX_SIDE: u32 = 30_000;

fn u16_at(b: &[u8], at: usize) -> Result<u16, DibError> {
    b.get(at..at + 2)
        .map(|s| u16::from_le_bytes([s[0], s[1]]))
        .ok_or(DibError::Truncated)
}

fn u32_at(b: &[u8], at: usize) -> Result<u32, DibError> {
    b.get(at..at + 4)
        .map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
        .ok_or(DibError::Truncated)
}

/// The header fields the decoder needs.
struct Header {
    width: u32,
    height: u32,
    top_down: bool,
    bits: u16,
    compression: u32,
    masks: [u32; 4],
    palette_at: usize,
    palette_len: usize,
    pixels_at: usize,
    dpi: Option<f32>,
}

fn header(b: &[u8]) -> Result<Header, DibError> {
    let size = u32_at(b, 0)? as usize;
    if !matches!(size, 40 | 52 | 56 | 108 | 124) {
        return Err(DibError::Unsupported);
    }
    let width = i32::from_le_bytes(u32_at(b, 4)?.to_le_bytes());
    let height = i32::from_le_bytes(u32_at(b, 8)?.to_le_bytes());
    let bits = u16_at(b, 14)?;
    let compression = u32_at(b, 16)?;
    let ppm = u32_at(b, 24)?;
    let used = u32_at(b, 32)? as usize;
    let (w, h) = (width.unsigned_abs(), height.unsigned_abs());
    if width <= 0 || h == 0 || w > MAX_SIDE || h > MAX_SIDE {
        return Err(DibError::Size);
    }
    let masked = matches!(compression, BI_BITFIELDS | BI_ALPHABITFIELDS);
    let mask_count = if compression == BI_ALPHABITFIELDS {
        4
    } else {
        3
    };
    let (masks, after_masks) = if !masked {
        ([0; 4], size)
    } else if size >= 52 {
        let alpha = if size >= 56 { u32_at(b, 52)? } else { 0 };
        (
            [u32_at(b, 40)?, u32_at(b, 44)?, u32_at(b, 48)?, alpha],
            size,
        )
    } else {
        let alpha = if mask_count == 4 { u32_at(b, 52)? } else { 0 };
        let m = [u32_at(b, 40)?, u32_at(b, 44)?, u32_at(b, 48)?, alpha];
        (m, size + 4 * mask_count)
    };
    let palette_len = match bits {
        1 | 4 | 8 if used == 0 => 1usize << bits,
        1 | 4 | 8 => used.min(1usize << bits),
        _ => 0,
    };
    let masks = if bits == 16 && !masked {
        [0x7C00, 0x03E0, 0x001F, 0]
    } else {
        masks
    };
    // ui-text-exempt: a lint reason
    #[allow(clippy::cast_precision_loss, reason = "a resolution, never near 2^24")]
    // NOT A DOCUMENT LENGTH: a pixel density, pixels per metre to per inch.
    let dpi = (ppm > 0).then_some(ppm as f32 * 0.0254);
    // A 24- or 32-bit bitmap may still carry an optimisation palette.
    let palette_skip = if bits > 8 { used } else { palette_len };
    Ok(Header {
        width: w,
        height: h,
        top_down: height < 0,
        bits,
        compression,
        masks,
        palette_at: after_masks,
        palette_len,
        pixels_at: after_masks + 4 * palette_skip,
        dpi,
    })
}

/// The 8-bit value of the field `mask` selects in `px`, scaled to 0–255.
fn channel(px: u32, mask: u32) -> Option<u8> {
    if mask == 0 {
        return None;
    }
    let shift = mask.trailing_zeros();
    let max = mask >> shift;
    let v = (px & mask) >> shift;
    u8::try_from(v * 255 / max).ok()
}

/// Decode `dib`.
///
/// # Errors
/// [`DibError`] when the bytes are short, the format is one this decoder does
/// not read, or the dimensions are zero or beyond [`MAX_SIDE`].
pub fn decode(dib: &[u8]) -> Result<Rgba, DibError> {
    let h = header(dib)?;
    let masked = matches!(h.compression, BI_BITFIELDS | BI_ALPHABITFIELDS);
    let ok = match h.bits {
        1 | 4 | 8 | 24 => h.compression == BI_RGB,
        16 => masked || h.compression == BI_RGB,
        32 => masked || h.compression == BI_RGB,
        _ => false,
    };
    if !ok {
        return Err(DibError::Unsupported);
    }
    let stride = (h.width as usize * usize::from(h.bits)).div_ceil(32) * 4;
    let need = h.pixels_at + stride * h.height as usize;
    if dib.len() < need {
        return Err(DibError::Truncated);
    }
    let mut out = Vec::with_capacity(h.width as usize * h.height as usize * 4);
    for row in 0..h.height as usize {
        let src = if h.top_down {
            row
        } else {
            h.height as usize - 1 - row
        };
        let line = &dib[h.pixels_at + src * stride..][..stride];
        for x in 0..h.width as usize {
            out.extend_from_slice(&pixel(dib, &h, line, x)?);
        }
    }
    if h.bits == 32 && h.masks[3] == 0 && out.chunks_exact(4).all(|p| p[3] == 0) {
        out.chunks_exact_mut(4).for_each(|p| p[3] = 255);
    }
    Ok(Rgba {
        width: h.width,
        height: h.height,
        pixels: out,
        dpi: h.dpi,
    })
}

/// Pixel `x` of `line` as RGBA. A 32-bit `BI_RGB` pixel carries its fourth
/// byte as alpha here; [`decode`] makes an all-zero alpha opaque afterwards.
fn pixel(dib: &[u8], h: &Header, line: &[u8], x: usize) -> Result<[u8; 4], DibError> {
    match h.bits {
        24 => Ok([line[x * 3 + 2], line[x * 3 + 1], line[x * 3], 255]),
        32 if h.compression == BI_RGB => {
            let p = &line[x * 4..x * 4 + 4];
            Ok([p[2], p[1], p[0], p[3]])
        }
        16 | 32 => {
            let px = if h.bits == 16 {
                u32::from(u16::from_le_bytes([line[x * 2], line[x * 2 + 1]]))
            } else {
                u32_at(line, x * 4)?
            };
            let [r, g, b, a] = h.masks;
            let get = |m| channel(px, m).ok_or(DibError::Unsupported);
            let alpha = if a == 0 { 255 } else { get(a)? };
            Ok([get(r)?, get(g)?, get(b)?, alpha])
        }
        bits => {
            let per_byte = 8 / usize::from(bits);
            let byte = line[x / per_byte];
            let shift = 8 - usize::from(bits) * (x % per_byte + 1);
            let index = usize::from(byte >> shift) & ((1 << bits) - 1);
            if index >= h.palette_len {
                return Err(DibError::Truncated);
            }
            let at = h.palette_at + index * 4;
            let q = dib.get(at..at + 4).ok_or(DibError::Truncated)?;
            Ok([q[2], q[1], q[0], 255])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A `BITMAPINFOHEADER` for `w`×`h` (negative `h` is top-down).
    fn info(w: i32, h: i32, bits: u16, compression: u32, used: u32) -> Vec<u8> {
        let mut b = Vec::new();
        b.extend(40u32.to_le_bytes());
        b.extend(w.to_le_bytes());
        b.extend(h.to_le_bytes());
        b.extend(1u16.to_le_bytes());
        b.extend(bits.to_le_bytes());
        b.extend(compression.to_le_bytes());
        b.extend(0u32.to_le_bytes());
        b.extend(3780u32.to_le_bytes());
        b.extend(3780u32.to_le_bytes());
        b.extend(used.to_le_bytes());
        b.extend(0u32.to_le_bytes());
        b
    }

    #[test]
    fn a_24_bit_bottom_up_bitmap_is_flipped_and_padded_rows_skipped() {
        let mut b = info(1, 2, 24, BI_RGB, 0);
        b.extend([0, 0, 255, 0]); // bottom row: red, padded to 4
        b.extend([255, 0, 0, 0]); // top row: blue
        let d = decode(&b).unwrap();
        assert_eq!(d.pixels, vec![0, 0, 255, 255, 255, 0, 0, 255]);
        assert!((d.dpi.unwrap() - 96.0).abs() < 0.1);
    }

    #[test]
    fn a_32_bit_bitmap_with_zero_alpha_is_opaque_and_real_alpha_is_kept() {
        let mut b = info(2, -1, 32, BI_RGB, 0);
        b.extend([1, 2, 3, 0, 4, 5, 6, 0]);
        assert_eq!(decode(&b).unwrap().pixels, vec![3, 2, 1, 255, 6, 5, 4, 255]);
        let mut b = info(2, -1, 32, BI_RGB, 0);
        b.extend([1, 2, 3, 128, 4, 5, 6, 0]);
        assert_eq!(decode(&b).unwrap().pixels, vec![3, 2, 1, 128, 6, 5, 4, 0]);
    }

    #[test]
    fn bitfields_masks_after_the_header_are_honoured() {
        let mut b = info(1, 1, 32, BI_BITFIELDS, 0);
        for m in [0x00FF_0000u32, 0x0000_FF00, 0x0000_00FF] {
            b.extend(m.to_le_bytes());
        }
        b.extend(0x0011_2233u32.to_le_bytes());
        assert_eq!(decode(&b).unwrap().pixels, vec![0x11, 0x22, 0x33, 255]);
    }

    #[test]
    fn a_v5_header_carries_its_own_masks_including_alpha() {
        let mut b = info(1, 1, 32, BI_BITFIELDS, 0);
        b[0..4].copy_from_slice(&124u32.to_le_bytes());
        for m in [0x00FF_0000u32, 0x0000_FF00, 0x0000_00FF, 0xFF00_0000] {
            b.extend(m.to_le_bytes());
        }
        b.resize(124, 0);
        b.extend(0x8011_2233u32.to_le_bytes());
        assert_eq!(decode(&b).unwrap().pixels, vec![0x11, 0x22, 0x33, 0x80]);
    }

    #[test]
    fn a_16_bit_565_bitmap_scales_each_field_to_a_byte() {
        let mut b = info(1, 1, 16, BI_BITFIELDS, 0);
        for m in [0xF800u32, 0x07E0, 0x001F] {
            b.extend(m.to_le_bytes());
        }
        b.extend([0xFF, 0xFF, 0, 0]);
        assert_eq!(decode(&b).unwrap().pixels, vec![255, 255, 255, 255]);
    }

    #[test]
    fn an_8_bit_palette_bitmap_looks_up_its_colours() {
        let mut b = info(2, 1, 8, BI_RGB, 2);
        b.extend([0, 0, 0, 0, 10, 20, 30, 0]);
        b.extend([1, 0, 0, 0]);
        assert_eq!(
            decode(&b).unwrap().pixels,
            vec![30, 20, 10, 255, 0, 0, 0, 255]
        );
    }

    #[test]
    fn short_or_unsupported_bitmaps_are_refused() {
        let b = info(4, 4, 24, BI_RGB, 0);
        assert_eq!(decode(&b), Err(DibError::Truncated));
        assert_eq!(decode(&info(1, 1, 24, 1, 0)), Err(DibError::Unsupported));
        assert_eq!(decode(&info(0, 1, 24, BI_RGB, 0)), Err(DibError::Size));
        assert_eq!(decode(&[0; 8]), Err(DibError::Unsupported));
        assert_eq!(decode(&[40, 0]), Err(DibError::Truncated));
    }
}
