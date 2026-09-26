//! # `dialogs::about` — the attribution surface an operator can actually reach
//!
//! The dispatch target for `file.about`, on **File ▸ pdfcer** beside Settings
//! and Keyboard shortcuts.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/about.md`.

use egui_shell::theme::Theme;

use crate::text::about as t;

/// The region this window's body publishes, so a driven check can find it.
pub const REGION_BODY: &str = "dialog:about"; // ui-text-exempt: trace region name, never displayed

/// The About dialog. Its existence is its "open" state — see
/// [`super::DialogsState`]'s header for why there is no `open: bool`.
///
/// It holds **no configuration at all**, which is unusual for a dialog and is
/// the honest shape here: everything it shows is a constant, so there is
/// nothing for the operator to change and nothing for closing it to forget.
/// It is still a struct rather than a bare `bool` so that it sits in
/// [`super::DialogsState`] under the same idiom as every other dialog; a
/// second, simpler mechanism for one surface is how two ways to do one thing
/// get started.
#[derive(Debug, Default)]
pub struct AboutDialog {
    /// Set by the Close button, consumed by [`Self::show`].
    ///
    /// The same two-step the print dialog uses: a widget inside the window's
    /// closure cannot drop the state it is being drawn from, so it records
    /// the request and the caller acts on it after the closure returns.
    close_requested: bool,
}

impl AboutDialog {
    /// Build the dialog.
    ///
    /// Takes nothing, because it shows nothing that varies. Kept as a
    /// constructor rather than letting callers write `AboutDialog::default()`
    /// so that the day it *does* need something — a build identity, a
    /// packaged-build marker — the call sites do not have to change.
    #[must_use]
    pub(super) fn open() -> Self {
        Self::default()
    }

    /// Draw one frame of the dialog. Returns `false` when it should close.
    pub(super) fn show(&mut self, ctx: &egui::Context) -> bool {
        // Its own OS window, like every dialog in this directory. This is the
        // case that makes the reason obvious: About carries the third-party
        // ATTRIBUTIONS, the one surface in this program with a legal obligation
        // behind it, and it has to be movable off the document so it can be read
        // beside something else.
        //
        // An OS window is anchored to the DESKTOP, so it needs no screen anchor
        // of its own — and must not grow one. Re-applying a centring anchor on
        // every frame drags the window back to the middle the moment the
        // operator moves it.
        let (frame, ()) = crate::dialogs::host::Host::new(
            "about", // ui-text-exempt: a viewport key, never displayed.
            t::title(),
            egui::vec2(560.0, 480.0),
            // A floor, for the reason the print dialog records: a resizable
            // window with no minimum can be dragged down to a title bar and a
            // scrollbar, which is a state with no way out but closing.
            egui::vec2(420.0, 300.0),
        )
        .show(ctx, |ui| {
            // Declared like every other dialog's body, so a driven check can
            // find this window.
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui);
        });
        !frame.closed && !std::mem::take(&mut self.close_requested)
    }

    /// Everything inside the window.
    fn body(&mut self, ui: &mut egui::Ui) {
        let theme = Theme::of(ui.ctx());

        ui.label(egui::RichText::new(t::product()).heading());
        // The RELEASE version, from the git tag by way of `build.rs` — not
        // the crate manifest. See [`version_label`] for the whole argument; the
        // short form is that `CARGO_PKG_VERSION` is pinned at `0.1.0` on
        // purpose and is not a release version at all.
        ui.label(version_label(
            env!("PDFCER_RELEASE_VERSION"),
            env!("PDFCER_RELEASE_DISTANCE"),
            env!("PDFCER_RELEASE_MODIFIED") == "1",
        ));
        ui.add_space(6.0);
        ui.label(t::summary());
        ui.add_space(6.0);
        ui.label(egui::RichText::new(t::licence_line()).color(theme.palette.text_muted));

        ui.add_space(10.0);
        build_block(ui, &theme);

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(6.0);

        ui.label(egui::RichText::new(t::attributions_heading()).heading());
        ui.add_space(6.0);

        // Scrolled, and the scroll starts HERE rather than around the whole
        // body: the product identity and the licence line are the two things
        // that must be legible without scrolling, and a scroll area over
        // everything would let them be dragged out of sight.
        //
        // The `max` is not defensive tidying. `available_height()` minus the
        // footer's reservation goes NEGATIVE in a window shorter than its own
        // header, and a negative `max_height` is neither a compile error nor a
        // panic — it is a scroll area that silently draws nothing. The
        // attribution list would then be empty on a small screen and correct
        // everywhere else, which is precisely the class of defect that only
        // ever shows up on somebody else's laptop. The floor keeps at least
        // one entry reachable; `tests::a_window_too_small_for_its_own_header_still_draws`
        // is what stops the clamp being removed as noise.
        const FOOTER_RESERVE: f32 = 44.0;
        const LIST_FLOOR: f32 = 48.0;
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .max_height((ui.available_height() - FOOTER_RESERVE).max(LIST_FLOOR))
            .show(ui, |ui| {
                for (i, a) in t::attributions().iter().enumerate() {
                    if i > 0 {
                        ui.add_space(10.0);
                    }
                    attribution(ui, &theme, a);
                }
                ui.add_space(12.0);
                ui.label(egui::RichText::new(t::full_texts_note()).color(theme.palette.text_muted));
            });

        ui.separator();
        ui.horizontal(|ui| {
            if ui.button(t::close()).clicked() {
                self.close_requested = true;
            }
        });
    }
}

/// **Which of the three version sentences this build is entitled to.**
fn version_label(version: &str, distance: &str, modified: bool) -> String {
    if version.is_empty() {
        return t::version_unreleased().to_owned();
    }
    let commits: u32 = distance.parse().unwrap_or(0);
    if commits == 0 && !modified {
        t::version_line(version)
    } else {
        t::version_line_after(version, commits, modified)
    }
}

/// **When this was built, and what is inside it.**
fn build_block(ui: &mut egui::Ui, theme: &Theme) {
    // The provenance, traced as well as drawn.
    //
    // A screenshot proves the block rendered; it does not prove the values are
    // the ones that were compiled in, and reading four fields out of a PNG is
    // not something this harness can do. The trace is the assertable half, and
    // it carries exactly what the labels carry — so a check can fail on an
    // EMPTY stamp, which is the failure mode that would otherwise look like a
    // layout glitch rather than a missing value.
    //
    // Quoted, for the reason `app::keyboard`'s chord trace is quoted: a value
    // with a space or a bracket in it corrupts the rest of the line otherwise.
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI.
        format!(
            "about-build stamp={:?} rev={:?} engine={:?} engine_rev={:?} iccce={:?} \
             release={:?} release_distance={:?}",
            env!("PDFCER_BUILD_TIME"),
            env!("PDFCER_GUI_REV"),
            env!("PDFCER_ENGINE_VERSION"),
            env!("PDFCER_ENGINE_REV"),
            env!("PDFCER_ICCCE_VERSION"),
            // This line grows by ADDING keys, never by reordering or
            // renaming: `tools/ui-verify`'s about check reads named keys off it
            // and asserts on a fixed list of them, so a new key is available to
            // a future driven check without disturbing the one that exists.
            // Traced EMPTY rather than omitted when there is no
            // release version, for the same reason the stamp is — an absent
            // key and an unset value would be indistinguishable.
            env!("PDFCER_RELEASE_VERSION"),
            env!("PDFCER_RELEASE_DISTANCE"),
        )
    });
    // `.strong()` AND an explicit colour, which is the sanctioned pairing.
    //
    // `RichText::strong()` has no colour role of its own — it resolves to
    // `widgets.active.fg_stroke`, the foreground of the accent-FILLED widget
    // state — so on an ordinary panel it is pale text on a pale background, and
    // a heading rendered that way is barely visible beside the legible labels
    // under it. `tools/gates/check-strong-text.sh` refuses the unpaired form.
    ui.label(
        egui::RichText::new(t::build_heading())
            .strong()
            .color(theme.palette.text),
    );
    ui.add_space(4.0);
    ui.label(t::build_line(
        env!("PDFCER_BUILD_TIME"),
        env!("PDFCER_GUI_REV"),
    ));

    component(
        ui,
        "pdfcer",
        env!("PDFCER_ENGINE_VERSION"),
        env!("PDFCER_ENGINE_REV"),
        env!("PDFCER_ENGINE_TIME"),
    );
    component(
        ui,
        "iccce",
        env!("PDFCER_ICCCE_VERSION"),
        env!("PDFCER_ICCCE_REV"),
        env!("PDFCER_ICCCE_TIME"),
    );

    ui.add_space(4.0);
    ui.label(
        egui::RichText::new(t::components_note())
            .small()
            .color(theme.palette.text_muted),
    );
}

/// One component row: its revision and commit date, or a statement that it is
/// not in this build.
fn component(ui: &mut egui::Ui, name: &str, version: &str, rev: &str, committed: &str) {
    if version.is_empty() {
        ui.label(t::component_absent(name));
    } else {
        ui.label(t::component_line(name, version, rev, committed));
    }
}

/// Draw one attribution: what it is, who made it, where from, and on what
/// terms.
fn attribution(ui: &mut egui::Ui, theme: &Theme, a: &crate::text::about::Attribution) {
    ui.label(a.component);
    ui.label(egui::RichText::new(a.creator).color(theme.palette.text_muted));
    ui.label(egui::RichText::new(a.origin).color(theme.palette.text_muted));
    ui.label(a.licence);
    if let Some(url) = a.licence_url {
        // `hyperlink` rather than a label: a licence that requires a LINK
        // requires one the recipient can follow, and a URL they have to
        // retype is a link in appearance only. Drawn only when the licence
        // family asks for it — see `Attribution::licence_url`.
        ui.hyperlink(url);
    }
    ui.label(egui::RichText::new(a.changes).color(theme.palette.text_muted));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fresh dialog is not asking to close.
    #[test]
    fn a_new_dialog_does_not_immediately_close() {
        let d = AboutDialog::open();
        assert!(!d.close_requested);
    }

    /// Run one real frame and assert the dialog stays open.
    #[test]
    fn one_frame_draws_and_leaves_the_dialog_open() {
        let mut dialog = AboutDialog::open();
        let ctx = egui::Context::default();
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(500.0, 340.0),
            )),
            ..Default::default()
        };
        ctx.begin_pass(input);
        let stayed_open = dialog.show(&ctx);
        let _ = ctx.end_pass();
        assert!(
            stayed_open,
            "the dialog closed itself on the frame it opened"
        );
    }

    /// A frame drawn at a size that leaves almost no room still composes.
    #[test]
    fn a_window_too_small_for_its_own_header_still_draws() {
        let mut dialog = AboutDialog::open();
        let ctx = egui::Context::default();
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(200.0, 60.0),
            )),
            ..Default::default()
        };
        ctx.begin_pass(input);
        let stayed_open = dialog.show(&ctx);
        let _ = ctx.end_pass();
        assert!(stayed_open, "a cramped window closed the dialog");
    }

    // =======================================================================
    // The version headline
    // =======================================================================

    /// A clean tree sitting exactly on the tag names the release, bare.
    ///
    /// The only state entitled to `Version 0.5.0` with nothing after it.
    #[test]
    fn a_build_on_the_tag_names_the_release() {
        assert_eq!(version_label("0.5.0", "0", false), "Version 0.5.0");
    }

    /// A build past the tag says so, and does not pass for the release.
    #[test]
    fn a_build_past_the_tag_does_not_pass_for_the_release() {
        let label = version_label("0.5.0", "23", false);
        assert!(
            label.contains("0.5.0") && label.contains("23"),
            "a development build must name both the release it is past and how \
             far past it is; got {label:?}"
        );
        assert_ne!(
            label,
            version_label("0.5.0", "0", false),
            "a build 23 commits past v0.5.0 renders identically to the release itself"
        );
    }

    /// An uncommitted change is enough to lose the bare line, at distance 0.
    ///
    /// The tag is the release; a tree with edits in it is not, even when the
    /// commit underneath is exactly the tagged one.
    #[test]
    fn a_modified_tree_on_the_tag_is_not_the_release() {
        let label = version_label("0.5.0", "0", true);
        assert_ne!(label, "Version 0.5.0");
        assert!(label.contains("uncommitted"), "got {label:?}");
    }

    /// **With no version available, nothing numeric is drawn.**
    #[test]
    fn the_unavailable_case_invents_no_number() {
        let label = version_label("", "", false);
        assert!(
            !label.chars().any(|c| c.is_ascii_digit()),
            "About showed a number when no release version could be derived. \
             The only numbers in reach there are invented ones — the crate \
             manifest's {manifest:?}, which is pinned by O110 and is not a \
             release version, or a literal. Got {label:?}.",
            manifest = env!("CARGO_PKG_VERSION")
        );
        assert!(
            !label.is_empty(),
            "an absent release version must be STATED, not left as a gap: a \
             missing line is indistinguishable from a layout fault"
        );
    }

    /// The modified flag cannot resurrect a version that does not exist.
    ///
    /// Guards the branch order: the emptiness test has to come first, or a
    /// dirty tarball build renders `Version , with uncommitted changes`.
    #[test]
    fn no_version_beats_every_other_fact() {
        assert_eq!(version_label("", "7", true), version_label("", "", false));
    }

    /// **`build.rs` actually emitted the fields, and they are well formed.**
    #[test]
    fn the_build_script_emitted_a_usable_release_field() {
        let version = env!("PDFCER_RELEASE_VERSION");
        let distance = env!("PDFCER_RELEASE_DISTANCE");
        if version.is_empty() {
            assert!(
                distance.is_empty(),
                "no release version but a distance of {distance:?} — build.rs \
                 emitted half a fact"
            );
            return;
        }
        assert!(
            version.starts_with(|c: char| c.is_ascii_digit()),
            "the release version is {version:?}; the tag's leading `v` should \
             have been stripped in build.rs, not shown to an operator"
        );
        assert!(
            distance.bytes().all(|b| b.is_ascii_digit()) && !distance.is_empty(),
            "the release distance is {distance:?}, which is not a commit count"
        );
    }

    /// The catalog the dialog draws from is not empty.
    #[test]
    fn the_dialog_has_something_to_attribute() {
        assert!(
            !crate::text::about::attributions().is_empty(),
            "About is on the ribbon with an empty attribution list"
        );
    }
}
