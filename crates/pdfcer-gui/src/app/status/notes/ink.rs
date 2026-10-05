//! The colour and overprint half of the render notes.
//!
//! [`ROSTER`] holds every counter the engine's render-metrics table (the
//! `pdfcer-cli` crate header) labels DIVERGENCE, keyed by that table's metrics
//! key. The engine offers no enumeration of its divergence counters, so this
//! list is written by hand; `tools/gates/check-ink-roster.py` reads the pinned
//! engine's table and fails when the two disagree.

use crate::text::status as t;

/// One divergence counter: its engine metrics key, how to read it, and the
/// sentence that puts it into words.
pub(super) struct InkNote {
    /// The key on the engine's metrics line. Matched by the roster gate.
    pub key: &'static str,
    /// The count, read from a render's diagnostics.
    pub read: fn(&pdfcer_render::Diagnostics) -> usize,
    /// The catalog phrase for a non-zero count.
    pub say: fn(usize) -> String,
}

/// Overprint first: it is the shortfall that looks like a right picture.
pub(super) const ROSTER: [InkNote; 15] = [
    InkNote {
        key: "overprint_refused",
        read: |d| d.overprint_refused,
        say: t::ink_overprint_refused,
    },
    InkNote {
        key: "overprint_images_unsupported",
        read: |d| d.overprint_images_unsupported,
        say: t::ink_overprint_images,
    },
    InkNote {
        key: "overprint_process_images_unsupported",
        read: |d| d.overprint_process_images_unsupported,
        say: t::ink_overprint_images,
    },
    InkNote {
        key: "overprint_shadings_unsupported",
        read: |d| d.overprint_shadings_unsupported,
        say: t::ink_overprint_shadings,
    },
    InkNote {
        key: "cmyk_buffer_refused",
        read: |d| d.cmyk_buffer_refused,
        say: t::ink_buffer_refused,
    },
    InkNote {
        key: "blends_in_wrong_space",
        read: |d| d.blends_in_wrong_space,
        say: t::ink_blends_wrong_space,
    },
    InkNote {
        key: "cmyk_groups_approximated",
        read: |d| usize::try_from(d.cmyk_groups_approximated).unwrap_or(usize::MAX),
        say: t::ink_groups_approximated,
    },
    InkNote {
        key: "soft_masks_ignored",
        read: |d| d.soft_masks_ignored,
        say: t::ink_soft_masks_ignored,
    },
    InkNote {
        key: "soft_mask_tr_ignored",
        read: |d| d.soft_mask_transfer_ignored,
        say: t::ink_mask_transfer_ignored,
    },
    InkNote {
        key: "blend_modes_ignored",
        read: |d| d.blend_modes_ignored,
        say: t::ink_blend_modes_ignored,
    },
    InkNote {
        key: "groups_flattened",
        read: |d| d.transparency_groups_flattened,
        say: t::ink_groups_flattened,
    },
    InkNote {
        key: "cs_unresolved",
        read: |d| d.color.spaces_unresolved,
        say: t::ink_spaces_unresolved,
    },
    InkNote {
        key: "tint_not_applied",
        read: |d| d.color.tint_transform_not_applied,
        say: t::ink_tint_not_applied,
    },
    InkNote {
        key: "patterns_unpainted",
        read: |d| d.color.patterns_unpainted,
        say: t::ink_patterns_unpainted,
    },
    InkNote {
        key: "shadings_refused",
        read: |d| d.shading.refused,
        say: t::ink_shadings_refused,
    },
];

/// The colour findings that occurred, in [`ROSTER`] order.
pub(super) fn findings(d: &pdfcer_render::Diagnostics) -> impl Iterator<Item = String> + '_ {
    ROSTER.iter().filter_map(|note| {
        let n = (note.read)(d);
        (n > 0).then(|| (note.say)(n))
    })
}

/// The roster as `key=count` pairs, for the trace.
pub(super) fn trace_pairs(d: &pdfcer_render::Diagnostics) -> String {
    ROSTER
        .iter()
        .map(|note| format!("{}={}", note.key, (note.read)(d)))
        .collect::<Vec<_>>()
        .join(" ") // ui-text-exempt: trace field separator, never displayed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_clean_render_has_no_colour_findings() {
        let d = pdfcer_render::Diagnostics::default();
        assert_eq!(findings(&d).count(), 0);
    }

    #[test]
    fn a_refused_overprint_is_reported() {
        let d = pdfcer_render::Diagnostics {
            overprint_refused: 3,
            ..Default::default()
        };
        let found: Vec<String> = findings(&d).collect();
        assert_eq!(found, vec![t::ink_overprint_refused(3)]);
    }

    #[test]
    fn every_key_is_distinct() {
        let mut keys: Vec<&str> = ROSTER.iter().map(|n| n.key).collect();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), ROSTER.len());
    }
}
