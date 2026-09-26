//! The pixel buffer the oracles read, plus getting pixels in and out of it.
//!
//! ## One representation, chosen for the capture path
//!
//! [`Image`] holds **BGRA, 8 bits per channel, top row first** — which is what
//! `GetDIBits` hands back and therefore costs no conversion on the hot path.
//! Everything else converts at its own edge: the PNG writer swizzles to RGB on
//! the way out, and the BMP reader swizzles on the way in.
//!
//! ## Reading a PNG somebody else wrote
//!
//! [`Image::load_png`] shells out to PowerShell, converts the file to a 24-bit
//! BMP in a temporary directory, and parses that. It does **not** carry a PNG
//! decoder, and the reason is proportion: the only PNGs this crate reads are
//! ones a human handed it — a dated evidence screenshot, a captured artefact
//! from an earlier run — and an inflate implementation is several hundred lines
//! of exacting code to serve a path that runs once per offline check.
//!
//! BMP is the right intermediate because it is the format whose parser is
//! genuinely trivial: a header, a pixel array, no compression, no filtering.
//! The whole reader below is forty lines and every one of them is checkable by
//! eye.
//!
//! The cost is a dependency on `powershell.exe`, which is stated rather than
//! hidden: on a machine without it, [`Image::load_png`] fails with a message
//! naming that, and the check reports SKIPPED. Live capture — the path that
//! actually matters — does not use this and does not care.
//!
//! Design and rationale: `docs/modules/ui-verify/image.md`.

use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::geom::PixRect;

/// A red/green/blue triple. Alpha is dropped at the boundary: a screenshot of
/// a composited desktop is opaque, and carrying an alpha channel into the
/// contrast maths would invite somebody to average it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgb {
    /// Red.
    pub r: u8,
    /// Green.
    pub g: u8,
    /// Blue.
    pub b: u8,
}

impl Rgb {
    /// A colour.
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

impl std::fmt::Display for Rgb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

/// A captured image: BGRA, top row first.
#[derive(Clone, Debug)]
pub struct Image {
    width: u32,
    height: u32,
    bgra: Vec<u8>,
}

impl Image {
    /// Wrap a BGRA buffer.
    pub fn from_bgra(width: u32, height: u32, bgra: Vec<u8>) -> Result<Self> {
        let expected = (width as usize) * (height as usize) * 4;
        if bgra.len() != expected {
            return Err(Error::new(format!(
                "BGRA buffer is {} bytes, expected {expected} for {width}x{height}",
                bgra.len()
            )));
        }
        Ok(Self {
            width,
            height,
            bgra,
        })
    }

    /// Width in pixels.
    #[must_use]
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Height in pixels.
    #[must_use]
    pub fn height(&self) -> u32 {
        self.height
    }

    /// The pixel at `(x, y)`, or `None` outside the image.
    #[must_use]
    pub fn pixel(&self, x: u32, y: u32) -> Option<Rgb> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let i = ((y as usize) * (self.width as usize) + (x as usize)) * 4;
        Some(Rgb::new(self.bgra[i + 2], self.bgra[i + 1], self.bgra[i]))
    }

    /// Every pixel in `region` that lies inside the image.
    pub fn pixels_in(&self, region: PixRect) -> impl Iterator<Item = Rgb> + '_ {
        let x1 = region.x.saturating_add(region.w).min(self.width);
        let y1 = region.y.saturating_add(region.h).min(self.height);
        (region.y..y1).flat_map(move |y| (region.x..x1).filter_map(move |x| self.pixel(x, y)))
    }

    /// Crop, clipped to the image.
    #[must_use]
    pub fn crop(&self, region: PixRect) -> Self {
        let x1 = region.x.saturating_add(region.w).min(self.width);
        let y1 = region.y.saturating_add(region.h).min(self.height);
        let (w, h) = (x1.saturating_sub(region.x), y1.saturating_sub(region.y));
        let mut bgra = Vec::with_capacity((w as usize) * (h as usize) * 4);
        for y in region.y..y1 {
            let start = ((y as usize) * (self.width as usize) + (region.x as usize)) * 4;
            bgra.extend_from_slice(&self.bgra[start..start + (w as usize) * 4]);
        }
        Self {
            width: w,
            height: h,
            bgra,
        }
    }

    /// Write the image as a PNG.
    pub fn save_png(&self, path: &Path) -> Result<()> {
        let mut rgb = Vec::with_capacity((self.width as usize) * (self.height as usize) * 3);
        for chunk in self.bgra.chunks_exact(4) {
            rgb.extend_from_slice(&[chunk[2], chunk[1], chunk[0]]);
        }
        let png = crate::png::encode_rgb(self.width, self.height, &rgb)
            .ok_or_else(|| Error::new("the PNG encoder refused the buffer"))?;
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(path, png)
            .map_err(|e| Error::new(format!("cannot write {}: {e}", path.display())))
    }

    /// Read a PNG by converting it to a BMP with PowerShell first.
    ///
    /// See the module docs for why this is not a decoder.
    pub fn load_png(path: &Path) -> Result<Self> {
        if !path.is_file() {
            return Err(Error::new(format!("no image at {}", path.display())));
        }
        let bmp = temp_path("ui-verify-convert", "bmp");
        // `-NoProfile` because a profile that prints a banner would land in
        // stderr and confuse the diagnosis of a genuine failure.
        let script = format!(
            "Add-Type -AssemblyName System.Drawing; \
             $b=[System.Drawing.Bitmap]::FromFile('{}'); \
             $c=New-Object System.Drawing.Bitmap $b.Width,$b.Height,\
             ([System.Drawing.Imaging.PixelFormat]::Format24bppRgb); \
             $g=[System.Drawing.Graphics]::FromImage($c); $g.DrawImage($b,0,0); \
             $c.Save('{}',[System.Drawing.Imaging.ImageFormat]::Bmp); \
             $g.Dispose(); $c.Dispose(); $b.Dispose()",
            path.display(),
            bmp.display()
        );
        let out = std::process::Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &script])
            .output()
            .map_err(|e| {
                Error::new(format!(
                    "cannot run powershell to convert {} for reading: {e}. ui-verify has no \
                     PNG decoder of its own — see src/image.rs.",
                    path.display()
                ))
            })?;
        if !bmp.is_file() {
            return Err(Error::new(format!(
                "the PNG-to-BMP conversion of {} produced nothing. powershell said: {}",
                path.display(),
                String::from_utf8_lossy(&out.stderr).trim()
            )));
        }
        let bytes = std::fs::read(&bmp)?;
        let _ = std::fs::remove_file(&bmp);
        Self::from_bmp(&bytes)
    }

    /// Parse an uncompressed 24- or 32-bit BMP.
    pub fn from_bmp(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 54 || &bytes[..2] != b"BM" {
            return Err(Error::new("not a BMP (bad signature)"));
        }
        let read_u32 = |at: usize| {
            u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
        };
        let read_i32 = |at: usize| read_u32(at) as i32;

        let data_offset = read_u32(10) as usize;
        let width = read_i32(18);
        let raw_height = read_i32(22);
        let bpp = u16::from_le_bytes([bytes[28], bytes[29]]);
        let compression = read_u32(30);

        if compression != 0 {
            return Err(Error::new(format!(
                "BMP compression {compression} is not supported (only BI_RGB)"
            )));
        }
        if bpp != 24 && bpp != 32 {
            return Err(Error::new(format!("BMP bit depth {bpp} is not 24 or 32")));
        }
        if width <= 0 || raw_height == 0 {
            return Err(Error::new(format!("BMP has no area: {width}x{raw_height}")));
        }

        let bottom_up = raw_height > 0;
        let height = raw_height.unsigned_abs();
        let width_u = width as u32;
        let bytes_per_px = (bpp / 8) as usize;
        // Every BMP row is padded to a 4-byte boundary.
        let stride = ((width_u as usize) * bytes_per_px).div_ceil(4) * 4;

        let needed = data_offset + stride * (height as usize);
        if bytes.len() < needed {
            return Err(Error::new(format!(
                "BMP is truncated: {} bytes, needs {needed}",
                bytes.len()
            )));
        }

        let mut bgra = vec![0u8; (width_u as usize) * (height as usize) * 4];
        for row in 0..height as usize {
            let src_row = if bottom_up {
                height as usize - 1 - row
            } else {
                row
            };
            let src = data_offset + src_row * stride;
            for col in 0..width_u as usize {
                let s = src + col * bytes_per_px;
                let d = (row * width_u as usize + col) * 4;
                bgra[d] = bytes[s];
                bgra[d + 1] = bytes[s + 1];
                bgra[d + 2] = bytes[s + 2];
                bgra[d + 3] = 0xFF;
            }
        }
        Self::from_bgra(width_u, height, bgra)
    }
}

/// A unique path in the system temp directory.
fn temp_path(stem: &str, ext: &str) -> PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("{stem}-{}-{n}.{ext}", std::process::id()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid(w: u32, h: u32, c: Rgb) -> Image {
        let mut bgra = Vec::new();
        for _ in 0..w * h {
            bgra.extend_from_slice(&[c.b, c.g, c.r, 0xFF]);
        }
        Image::from_bgra(w, h, bgra).unwrap()
    }

    #[test]
    fn a_mismatched_buffer_is_refused() {
        assert!(Image::from_bgra(2, 2, vec![0; 8]).is_err());
    }

    #[test]
    fn pixels_round_trip_through_bgra() {
        let img = solid(2, 2, Rgb::new(1, 2, 3));
        assert_eq!(img.pixel(1, 1), Some(Rgb::new(1, 2, 3)));
        assert_eq!(img.pixel(2, 0), None, "out of bounds must be None");
    }

    #[test]
    fn cropping_clips_to_the_image() {
        let img = solid(4, 4, Rgb::new(9, 9, 9));
        let c = img.crop(PixRect::new(2, 2, 10, 10));
        assert_eq!((c.width(), c.height()), (2, 2));
    }

    #[test]
    fn pixels_in_counts_only_what_is_inside() {
        let img = solid(3, 3, Rgb::new(0, 0, 0));
        assert_eq!(img.pixels_in(PixRect::new(1, 1, 10, 10)).count(), 4);
    }

    /// A bottom-up BMP (the normal kind) must come back the right way up. The
    /// fixture's two rows differ so a vertical mirror cannot pass.
    #[test]
    fn a_bottom_up_bmp_is_flipped_back() {
        let mut bmp = vec![0u8; 54];
        bmp[0] = b'B';
        bmp[1] = b'M';
        bmp[10] = 54; // data offset
        bmp[18] = 1; // width = 1
        bmp[22] = 2; // height = +2 (bottom-up)
        bmp[28] = 24; // bpp
        // Row 0 in the file is the BOTTOM row of the image. Each row is three
        // bytes of B,G,R padded to the 4-byte boundary every BMP row uses.
        bmp.extend_from_slice(&[0x11, 0x22, 0x33, 0]); // bottom: B,G,R + pad
        bmp.extend_from_slice(&[0x44, 0x55, 0x66, 0]); // top
        let img = Image::from_bmp(&bmp).expect("parses");
        assert_eq!(img.pixel(0, 0), Some(Rgb::new(0x66, 0x55, 0x44)), "top row");
        assert_eq!(
            img.pixel(0, 1),
            Some(Rgb::new(0x33, 0x22, 0x11)),
            "bottom row"
        );
    }

    #[test]
    fn a_compressed_bmp_is_refused_rather_than_guessed_at() {
        let mut bmp = vec![0u8; 54];
        bmp[0] = b'B';
        bmp[1] = b'M';
        bmp[30] = 1; // BI_RLE8
        assert!(Image::from_bmp(&bmp).is_err());
    }
}
