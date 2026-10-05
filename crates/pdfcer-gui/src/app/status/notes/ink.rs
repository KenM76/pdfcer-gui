//! The colour and overprint half of the render notes.
//!
//! The counters are the engine's: `Diagnostics::divergences` yields every
//! DIVERGENCE counter by its metrics key. This module only words them, and a
//! key it has no phrase for is still reported, under its key.

use crate::text::status as t;

/// The catalog phrase for a non-zero divergence counter, by engine key.
fn phrase(key: &str) -> Option<fn(usize) -> String> {
    let say: fn(usize) -> String = match key {
        "overprint_refused" => t::ink_overprint_refused,
        "overprint_images_unsupported" | "overprint_process_images_unsupported" => {
            t::ink_overprint_images
        }
        "overprint_shadings_unsupported" => t::ink_overprint_shadings,
        "cmyk_buffer_refused" => t::ink_buffer_refused,
        "blends_in_wrong_space" => t::ink_blends_wrong_space,
        "cmyk_groups_approximated" => t::ink_groups_approximated,
        "soft_masks_ignored" => t::ink_soft_masks_ignored,
        "soft_mask_transfer_ignored" => t::ink_mask_transfer_ignored,
        "blend_modes_ignored" => t::ink_blend_modes_ignored,
        "transparency_groups_flattened" => t::ink_groups_flattened,
        "color.spaces_unresolved" => t::ink_spaces_unresolved,
        "color.tint_transform_not_applied" => t::ink_tint_not_applied,
        "color.patterns_unpainted" => t::ink_patterns_unpainted,
        "shading.refused" => t::ink_shadings_refused,
        _ => return None,
    };
    Some(say)
}

/// The colour findings that occurred, in the engine's order.
pub(super) fn findings(d: &pdfcer_render::Diagnostics) -> impl Iterator<Item = String> + '_ {
    d.divergences().filter(|&(_, n)| n > 0).map(|(key, n)| {
        let n = usize::try_from(n).unwrap_or(usize::MAX);
        phrase(key).map_or_else(|| t::ink_unworded(key, n), |say| say(n))
    })
}

/// Every divergence counter as `key=count` pairs, zeros included, for the
/// trace.
pub(super) fn trace_pairs(d: &pdfcer_render::Diagnostics) -> String {
    d.divergences()
        .map(|(key, n)| format!("{key}={n}"))
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

    /// Red at a pin move that adds a counter: the new key is reported under
    /// its own name until it is given a sentence here.
    #[test]
    fn every_engine_counter_has_a_sentence() {
        let unworded: Vec<&str> = pdfcer_render::Diagnostics::DIVERGENCE_KEYS
            .into_iter()
            .filter(|key| phrase(key).is_none())
            .collect();
        assert!(unworded.is_empty(), "no sentence for {unworded:?}");
    }

    #[test]
    fn an_unworded_counter_still_names_itself() {
        assert!(phrase("not_a_counter").is_none());
        assert!(t::ink_unworded("not_a_counter", 2).contains("not_a_counter"));
    }
}
