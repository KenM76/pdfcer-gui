//! # `canvas::overlays` — the application's own colour roles, published per
//! frame
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/overlays.md`.

use egui_shell::theme::{Overlays, Theme};

/// Build the application's overlay roles from a resolved theme.
///
/// Pure, and takes the [`Theme`] rather than reading the context, so the test
/// below can run it against every preset without a frame. That is the same
/// shape `crate::panels::properties::font_embedded` takes and for the same
/// reason: a rule stated as a function is a rule that can be asserted.
#[must_use]
pub fn overlays_for(theme: &Theme) -> Overlays {
    Overlays::new()
        .with(
            crate::canvas::snap::SNAP_INDICATOR_ROLE,
            theme.palette.notice,
        )
        .with(
            crate::canvas::snap::SNAP_COMMITTED_ROLE,
            theme.palette.accent,
        )
}

/// Publish the roles for this frame.
///
/// Called beside `Theme::apply` in `crate::app::frame`, and per frame rather
/// than once at start-up for the theme's own reason: the operator can change
/// the preset from the Settings window, and a one-time install would mean a
/// restart to see the effect.
///
/// Cheap — a two-entry `BTreeMap` into an `Arc`, once per frame — and the
/// alternative is a cached copy that can disagree with the theme that is
/// actually applied, which is the class of bug `app::frame`'s own settings
/// snapshot exists to prevent.
pub fn install(ctx: &egui::Context, theme: &Theme) {
    Overlays::install(ctx, overlays_for(theme));
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_shell::theme::Preset;

    /// ★ **The preview role and the committed role are different colours, on
    /// every preset.**
    ///
    /// The check `egui-shell`'s `overlays.rs` says the application owes and the
    /// shell cannot write for it. Its own words for what it is protecting: *"a
    /// theme that merges two roles removes a cue that was doing work, and it
    /// would do so silently."*
    ///
    /// Over `Preset::ALL`, not the default. A preset is exactly where two roles
    /// collapse — a light theme reaching for the same mid-grey twice — and the
    /// two presets that are not the current default are the two nobody looks
    /// at until an operator switches to one.
    #[test]
    fn the_preview_and_committed_roles_are_distinct_on_every_preset() {
        for preset in Preset::ALL {
            let overlays = overlays_for(&Theme::new(*preset));
            assert!(
                overlays
                    .assert_distinct(&[
                        crate::canvas::snap::SNAP_INDICATOR_ROLE,
                        crate::canvas::snap::SNAP_COMMITTED_ROLE,
                    ])
                    .is_ok(),
                "{preset:?} draws a snap PROPOSAL and a COMMITTED selection in the same \
                 colour, so the cue that tells them apart is gone"
            );
        }
    }

    /// Every role the canvas asks for is defined, on every preset.
    ///
    /// ★ The failure this catches is the one `Overlays::get`'s `Option` makes
    /// possible: a role nobody defined returns `None`, the caller falls back,
    /// and **nothing looks broken** — the snap marker simply keeps drawing in
    /// the selection stroke, which is exactly the state this module was written
    /// to end and which survived unnoticed for a whole phase.
    #[test]
    fn every_role_the_canvas_reads_is_defined() {
        for preset in Preset::ALL {
            let overlays = overlays_for(&Theme::new(*preset));
            for role in [
                crate::canvas::snap::SNAP_INDICATOR_ROLE,
                crate::canvas::snap::SNAP_COMMITTED_ROLE,
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
