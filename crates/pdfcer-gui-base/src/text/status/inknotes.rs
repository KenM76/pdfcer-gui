//! Render-notes phrases for the colour and overprint findings: the counters the
//! engine's metrics table labels DIVERGENCE. Lower-case fragments, joined by
//! `diagnostics_join` like the drawing findings beside them.

/// `"1 <one>"` or `"<n> <many>"`.
fn counted(n: usize, one: &str, many: &str) -> String {
    if n == 1 {
        format!("1 {one}")
    } else {
        format!("{n} {many}")
    }
}

/// Paints that should have overprinted and were drawn knocking out instead.
#[must_use]
pub fn ink_overprint_refused(n: usize) -> String {
    counted(
        n,
        "overprinted mark shown knocked out (a press would keep the ink beneath)",
        "overprinted marks shown knocked out (a press would keep the ink beneath)",
    )
}

/// Images owed an overprint composite that did not get one.
#[must_use]
pub fn ink_overprint_images(n: usize) -> String {
    counted(
        n,
        "overprinting image shown knocked out",
        "overprinting images shown knocked out",
    )
}

/// Gradients painted under overprint that could not honour it.
#[must_use]
pub fn ink_overprint_shadings(n: usize) -> String {
    counted(
        n,
        "overprinting gradient shown knocked out",
        "overprinting gradients shown knocked out",
    )
}

/// Blends computed on the wrong side of the ink/screen complement.
#[must_use]
pub fn ink_blends_wrong_space(n: usize) -> String {
    counted(
        n,
        "blend mixed in screen colour instead of ink (overlap colour may be off)",
        "blends mixed in screen colour instead of ink (overlap colours may be off)",
    )
}

/// The page asked for ink compositing and the colorant buffer was refused.
#[must_use]
pub fn ink_buffer_refused(_n: usize) -> String {
    "page mixed in screen colour, not ink: too large for the ink memory limit (Settings, Colour)"
        .to_owned()
}

/// Groups whose result went through ink and whose interior did not.
#[must_use]
pub fn ink_groups_approximated(n: usize) -> String {
    counted(
        n,
        "see-through group mixed partly in screen colour",
        "see-through groups mixed partly in screen colour",
    )
}

/// Blend modes that fell back to a plain overlay.
#[must_use]
pub fn ink_blend_modes_ignored(n: usize) -> String {
    counted(
        n,
        "blend mode ignored, drawn as a plain overlay",
        "blend modes ignored, drawn as plain overlays",
    )
}

/// Soft masks not applied: the failure paints more than the file asked for.
#[must_use]
pub fn ink_soft_masks_ignored(n: usize) -> String {
    counted(
        n,
        "fade or mask ignored (something meant to be hidden may show)",
        "fades or masks ignored (something meant to be hidden may show)",
    )
}

/// Soft-mask transfer functions read and never applied.
#[must_use]
pub fn ink_mask_transfer_ignored(n: usize) -> String {
    counted(
        n,
        "mask adjustment ignored (a masked area may show what should be hidden)",
        "mask adjustments ignored (masked areas may show what should be hidden)",
    )
}

/// Transparency groups painted element by element instead of as a unit.
#[must_use]
pub fn ink_groups_flattened(n: usize) -> String {
    counted(
        n,
        "see-through group drawn piece by piece (overlaps may look darker)",
        "see-through groups drawn piece by piece (overlaps may look darker)",
    )
}

/// Colour spaces named by the content that could not be resolved.
#[must_use]
pub fn ink_spaces_unresolved(n: usize) -> String {
    counted(
        n,
        "colour definition unreadable (marks using it may be the wrong colour)",
        "colour definitions unreadable (marks using them may be the wrong colour)",
    )
}

/// Spot-colour conversions that painted pdfcer's stand-in.
#[must_use]
pub fn ink_tint_not_applied(n: usize) -> String {
    counted(
        n,
        "spot colour shown as a stand-in (the file does not say how to show it)",
        "spot colours shown as stand-ins (the file does not say how to show them)",
    )
}

/// Pattern selections that put nothing on the page.
#[must_use]
pub fn ink_patterns_unpainted(n: usize) -> String {
    counted(n, "pattern fill drew nothing", "pattern fills drew nothing")
}

/// Shading dictionaries rejected outright.
#[must_use]
pub fn ink_shadings_refused(n: usize) -> String {
    counted(
        n,
        "gradient could not be drawn",
        "gradients could not be drawn",
    )
}

/// A divergence counter this build has no sentence for, named by its engine
/// key so a counter added upstream is still disclosed.
#[must_use]
pub fn ink_unworded(key: &str, n: usize) -> String {
    format!("{n} drawn differently from what the file asks ({key})")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_and_many_read_differently() {
        assert_eq!(ink_patterns_unpainted(1), "1 pattern fill drew nothing");
        assert_eq!(ink_patterns_unpainted(3), "3 pattern fills drew nothing");
    }
}
