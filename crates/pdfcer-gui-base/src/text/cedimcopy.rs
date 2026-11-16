//! # `text::cedimcopy` — what a page copy did with the ce dimensions on its pages
//!
//! `pageops::extract_with_labels` and `pageops::split_with_labels` carry the
//! ce-dimension sidecar for every ce dimension whose page was copied, so those
//! arrive editable, in their groups, with each group's scale, format and
//! standard. Nothing is said for that, nor for
//! `AssembleReport::ce_dimensions_left` on its own: a ce dimension whose page
//! was not taken is the copy doing what was asked.
//!
//! What is said is the one case where the operator loses something he cannot
//! see: `AssembleReport::ce_dimension_sidecar_newer`, where nothing was
//! carried and every copied ce dimension is a plain line annotation that
//! draws the same and no longer edits as a ce dimension.

/// The sentence for a copy whose source was written by a newer pdfcer.
#[must_use]
pub const fn sidecar_newer() -> &'static str {
    "The ce dimensions on these pages were saved by a newer version of pdfcer, so in the \
     new file they are plain lines: they look the same, but they cannot be edited as ce \
     dimensions and belong to no group."
}

/// The receipt's sentence for one copy, or `None` when nothing needs saying.
#[must_use]
pub const fn copied(sidecar_newer: bool) -> Option<&'static str> {
    if sidecar_newer {
        Some(self::sidecar_newer())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_newer_sidecar_is_said() {
        assert_eq!(copied(false), None);
        assert_eq!(copied(true), Some(sidecar_newer()));
        assert!(sidecar_newer().contains("ce dimensions"));
    }
}
