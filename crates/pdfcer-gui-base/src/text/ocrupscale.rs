//! # `text::ocrupscale` — the enlarge-before-reading copy in File ▸ Recognise text…

/// The opt-in checkbox.
#[must_use]
pub const fn enlarge() -> &'static str {
    "Enlarge each page before reading"
}

/// Its tooltip.
#[must_use]
pub const fn enlarge_tooltip() -> &'static str {
    "For scans whose text is too small for the recogniser: each page is enlarged \
     and smoothed before it is read, and the words are put back where they are on \
     the page. It adds no detail, so a page that already reads well can read worse. \
     Not offered for PaddleOCR-VL, which resizes everything it reads itself."
}

/// The factor slider's label.
#[must_use]
pub const fn factor() -> &'static str {
    "Enlarge by"
}

/// The factor slider's suffix: the multiplication sign.
#[must_use]
pub const fn times() -> &'static str {
    "×"
}

/// The bicubic choice.
#[must_use]
pub const fn bicubic() -> &'static str {
    "Bicubic (recommended)"
}

/// The Lanczos choice.
#[must_use]
pub const fn lanczos() -> &'static str {
    "Lanczos (sharper edges, more ringing)"
}

/// The sizes the first chosen page is read at.
#[must_use]
pub fn sizes(from: (u32, u32), to: (u32, u32)) -> String {
    format!(
        "The first page is drawn at {} × {} pixels and read at {} × {}.",
        from.0, from.1, to.0, to.1
    )
}

/// Under [`sizes`] when the enlarged page is more than the recogniser takes.
#[must_use]
pub fn too_large(limit: usize) -> String {
    format!(
        "That is more than the {} million pixels a page may be enlarged to, so the \
         run will stop at it. Choose a smaller factor.",
        limit / 1_000_000
    )
}

/// The result's disclosure; `filter` is the engine's name for the kernel.
#[must_use]
pub fn enlarged(factor: f32, filter: &str, from: (u32, u32), to: (u32, u32)) -> String {
    format!(
        "Each page was enlarged {}× ({filter}) before reading; the last went from {} × {} \
         to {} × {} pixels.",
        factor_text(factor),
        from.0,
        from.1,
        to.0,
        to.1
    )
}

/// A factor as typed: `2` or `2.5`.
#[must_use]
pub fn factor_text(factor: f32) -> String {
    if factor.fract() == 0.0 {
        format!("{factor:.0}")
    } else {
        format!("{factor:.1}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whole_factors_print_without_a_decimal() {
        assert_eq!(factor_text(2.0), "2");
        assert_eq!(factor_text(2.5), "2.5");
    }

    #[test]
    fn the_limit_is_stated_in_millions() {
        assert!(too_large(1 << 27).contains("134 million"));
    }

    #[test]
    fn sentences_carry_no_line_break() {
        for s in [
            enlarge_tooltip().to_owned(),
            too_large(1 << 27),
            enlarged(3.0, "bicubic", (1, 2), (3, 6)),
        ] {
            assert!(!s.contains('\n') && !s.contains("  "), "{s}");
        }
    }
}
