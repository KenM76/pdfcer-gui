//! # `text::compact_compare` — the compact window's per-picture comparison
//!
//! Every word of the before-and-after table under a finished recompression.

/// The fold's header, naming how many pictures it lists.
#[must_use]
pub fn heading(pictures: usize) -> String {
    format!("Each picture, before and after ({pictures})")
}

/// Hover on the fold's header.
#[must_use]
pub const fn heading_tooltip() -> &'static str {
    "What the compacted copy does to each picture it changes, worked out before anything is \
     saved. A picture drawn in more than one place is listed once."
}

/// The table's column headings, in order.
#[must_use]
pub const fn columns() -> [&'static str; 8] {
    [
        "Picture",
        "Pixels",
        "Resolution",
        "Colours",
        "Stored as",
        "JPEG quality",
        "Match",
        "Size",
    ]
}

/// A picture's name: its object number.
#[must_use]
pub fn picture(number: u32) -> String {
    format!("#{number}")
}

/// `before to after`, or the one value when they are equal. Words, not an
/// arrow: the font stack has no arrow glyph.
#[must_use]
pub fn change(before: &str, after: &str) -> String {
    if before == after {
        before.to_owned()
    } else {
        format!("{before} to {after}")
    }
}

/// A pixel size.
#[must_use]
pub fn pixels((width, height): (u32, u32)) -> String {
    format!("{width}×{height}")
}

/// The resolution of the placement drawn largest, after the change; a dash
/// when the picture was not resized.
#[must_use]
pub fn resolution(ppi: Option<f64>) -> String {
    ppi.map_or_else(|| "—".to_owned(), |p| format!("{p:.0} ppi"))
}

/// A stored filter, by its PDF name, in the words the window uses; an empty
/// name is a picture stored uncompressed.
#[must_use]
pub fn filter(name: &str) -> String {
    match name {
        "" => "uncompressed".to_owned(),
        "DCTDecode" => "JPEG".to_owned(),
        "FlateDecode" => "Flate".to_owned(),
        "LZWDecode" => "LZW".to_owned(),
        "RunLengthDecode" => "run length".to_owned(),
        "CCITTFaxDecode" => "CCITT".to_owned(),
        "JBIG2Decode" => "JBIG2".to_owned(),
        "JPXDecode" => "JPEG 2000".to_owned(),
        "BrotliDecode" => "Brotli".to_owned(),
        other => other.strip_suffix("Decode").unwrap_or(other).to_owned(),
    }
}

/// The JPEG quality cell: the quality written, and the one asked when the
/// engine wrote lower; a dash for a picture not stored as JPEG.
#[must_use]
pub fn quality(asked_used: Option<(u8, u8)>) -> String {
    match asked_used {
        None => "—".to_owned(),
        Some((asked, used)) if used < asked => format!("{used} (of {asked})"),
        Some((_, used)) => used.to_string(),
    }
}

/// The closeness cell: the SSIM reached, in ten-thousandths, and whether it
/// met the target; a dash without a closeness target.
#[must_use]
pub fn closeness(reached: Option<(u16, bool)>) -> String {
    match reached {
        None => "—".to_owned(),
        Some((n, met)) => {
            let score = f64::from(n) / 10_000.0;
            if met {
                format!("{score:.4}")
            } else {
                format!("{score:.4}, short of the target")
            }
        }
    }
}

/// The size cell.
#[must_use]
pub fn size(before: u64, after: u64) -> String {
    change(
        &super::compact::bytes(before),
        &super::compact::bytes(after),
    )
}

/// Said under a table that stops before the last picture.
#[must_use]
pub fn more(count: usize) -> String {
    format!("{count} more picture(s) are changed and not listed.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unchanged_value_is_said_once() {
        assert_eq!(change("Flate", "Flate"), "Flate");
        assert_eq!(change("Flate", "JPEG"), "Flate to JPEG");
    }

    #[test]
    fn a_quality_capped_below_the_ask_names_both() {
        assert_eq!(quality(Some((80, 62))), "62 (of 80)");
        assert_eq!(quality(Some((80, 80))), "80");
        assert_eq!(quality(None), "—");
    }

    #[test]
    fn a_missed_closeness_says_so() {
        assert_eq!(closeness(Some((9512, true))), "0.9512");
        assert_eq!(
            closeness(Some((9312, false))),
            "0.9312, short of the target"
        );
    }
}
