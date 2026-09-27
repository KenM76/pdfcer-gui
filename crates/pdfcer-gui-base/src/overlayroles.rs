//! # `overlayroles` — the application's own colour roles, published per
//! frame
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/overlayroles.md`.

use egui_shell::theme::{Overlays, Theme};

/// Build the application's overlay roles from a resolved theme.
#[must_use]
pub fn overlays_for(theme: &Theme) -> Overlays {
    Overlays::new()
        .with(crate::snapmark::SNAP_INDICATOR_ROLE, theme.palette.notice)
        .with(crate::snapmark::SNAP_COMMITTED_ROLE, theme.palette.accent)
}

/// Publish the roles for this frame.
pub fn install(ctx: &egui::Context, theme: &Theme) {
    Overlays::install(ctx, overlays_for(theme));
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_shell::theme::Preset;

    /// **The preview role and the committed role are different colours, on
    /// every preset.**
    #[test]
    fn the_preview_and_committed_roles_are_distinct_on_every_preset() {
        for preset in Preset::ALL {
            let overlays = overlays_for(&Theme::new(*preset));
            assert!(
                overlays
                    .assert_distinct(&[
                        crate::snapmark::SNAP_INDICATOR_ROLE,
                        crate::snapmark::SNAP_COMMITTED_ROLE,
                    ])
                    .is_ok(),
                "{preset:?} draws a snap PROPOSAL and a COMMITTED selection in the same \
                 colour, so the cue that tells them apart is gone"
            );
        }
    }

    /// Every role the canvas asks for is defined, on every preset.
    #[test]
    fn every_role_the_canvas_reads_is_defined() {
        for preset in Preset::ALL {
            let overlays = overlays_for(&Theme::new(*preset));
            for role in [
                crate::snapmark::SNAP_INDICATOR_ROLE,
                crate::snapmark::SNAP_COMMITTED_ROLE,
            ] {
                assert!(
                    overlays.get(role).is_some(),
                    "{preset:?} defines no `{role}`, so the caller falls back and the \
                     absence is invisible"
                );
            }
        }
    }
}
