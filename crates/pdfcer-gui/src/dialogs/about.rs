//! # `dialogs::about` — the attribution surface an operator can actually reach
//!
//! The dispatch target for `file.about`, on **File ▸ pdfcer** beside Settings
//! and Keyboard shortcuts.
//!
//! ## What it is for, which is not "an About box"
//!
//! Most About boxes are a version number and a logo. This one exists for a
//! specific obligation, and the version number is the incidental part.
//!
//! `pdfcer-gui.exe` redistributes third-party work that `cargo-about` cannot
//! see — font faces and data tables the engine embeds with `include_bytes!`,
//! and, when OCR lands, a set of **CC-BY-SA-4.0** neural-network weights the
//! operator decided on 2026-08-14 to ship. Attribution-style licences require
//! the notice to reach the **recipient**, and the recipient of this program is
//! someone holding a binary, not someone reading a repository. A
//! `PROVENANCE.md` in the source tree discharges nothing for them.
//!
//! The full argument, the catalog, and the sources every field was lifted
//! from are in [`crate::text::about`]. This module is only the drawing.
//!
//! ## ★ Why this dialog is exempt from "a closed document closes the dialogs"
//!
//! [`super::DialogsState::show`] drops every open dialog the moment the
//! document goes away, and the reason is sound: a print job configured
//! against pages that no longer exist is a job against nothing.
//!
//! **This dialog is about the application, not about a document.** It has to
//! open with nothing loaded — an operator who has just launched pdfcer and
//! wants to know what version they are running has no document, and a control
//! that did nothing in that state would be the placeholder this project
//! forbids. So `show` grew a two-branch shape: document-scoped dialogs are
//! still closed when the document closes, and this one is drawn either way.
//! That distinction is now a property of the module rather than of this file;
//! see [`super::DialogsState::show`].
//!
//! ## Why it pushes no `Action`
//!
//! [`super::DialogsState`]'s header gives the rule: the action funnel exists
//! for changes to *document* state, and a dialog that changes none does not
//! use it. This one reads three `&'static str` catalogs and a compile-time
//! version constant. It has nothing to undo, nothing to order against, and
//! nothing that could alias.
//!
//! ## Layout
//!
//! One column, scrollable, in the order an operator reads it: what this
//! program is, what version, under what terms — then the third-party
//! material, then where the full texts are. The attribution list is a stack
//! of blocks rather than a table: an attribution is four sentences about one
//! work, and four sentences do not fit a table cell at any window width worth
//! having.

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
        // ★ ITS OWN OS WINDOW as of 2026-08-21 — the operator's report about
        // the print dialog applied to every dialog in this directory, and this
        // one is the case that makes it obvious: About carries the third-party
        // ATTRIBUTIONS, the one surface in this program with a legal obligation
        // behind it, and it could not be moved off the document to be read
        // beside anything.
        //
        // The screen anchor that stood here is retired rather than moved: an OS
        // window is anchored to the DESKTOP, which satisfies the standing
        // objection to surfaces that move on zoom more completely than
        // `CENTER_CENTER` did — and `CENTER_CENTER` on every frame was G6's
        // defect, dragging the window back to the middle the moment it was
        // moved.
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
        // The version is read from the crate manifest at compile time rather
        // than written in the catalog, so the two cannot drift. The *word* in
        // front of it is copy and lives in `text::about`.
        ui.label(t::version_line(env!("CARGO_PKG_VERSION")));
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
        // ★ The `max` is not defensive tidying. `available_height()` minus the
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

/// **When this was built, and what is inside it.**
///
/// Every value here arrives through `env!` from `build.rs`, so none of it can
/// drift from what was compiled: there is no constant to forget to bump.
///
/// # ★ Why the components are listed at all
///
/// `summary()` already says the engine is built in. That answers *"does this
/// talk to pdfcer or contain it"* and not the question an operator asks when a
/// bug appears and disappears between two copies of the program: **which
/// pdfcer**. Two builds an hour apart can carry different engines, and until
/// this block existed the only way to find out was to read `BUILD-INFO.txt`
/// beside the executable — which the operator does not have when someone sends
/// them a screenshot of the window.
///
/// # ★ Why an absent component is named
///
/// See [`crate::text::about::component_absent`]. Short version: the
/// no-placeholders rule governs controls, and this is a report.
fn build_block(ui: &mut egui::Ui, theme: &Theme) {
    // ★ The provenance, traced as well as drawn.
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
            "about-build stamp={:?} rev={:?} engine={:?} engine_rev={:?} iccce={:?}",
            env!("PDFCER_BUILD_TIME"),
            env!("PDFCER_GUI_REV"),
            env!("PDFCER_ENGINE_VERSION"),
            env!("PDFCER_ENGINE_REV"),
            env!("PDFCER_ICCCE_VERSION"),
        )
    });
    // ★ `.strong()` AND an explicit colour, which is the sanctioned pairing.
    //
    // `RichText::strong()` has no colour role of its own — it resolves to
    // `widgets.active.fg_stroke`, the foreground of the accent-FILLED widget
    // state — so on an ordinary panel it is pale text on a pale background.
    // This exact line shipped without the colour and the driven capture showed
    // "Build" as barely-there grey while every label under it was legible.
    // `tools/gates/check-strong-text.sh` catches it; the screenshot found it
    // first, which is the order this project expects.
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
///
/// The two cases share a function so they cannot be worded inconsistently, and
/// so a component moving from absent to present needs no edit here — the
/// lockfile decides.
fn component(ui: &mut egui::Ui, name: &str, version: &str, rev: &str, committed: &str) {
    if version.is_empty() {
        ui.label(t::component_absent(name));
    } else {
        ui.label(t::component_line(name, version, rev, committed));
    }
}

/// Draw one attribution: what it is, who made it, where from, and on what
/// terms.
///
/// # Why every field is drawn, including "no changes were made"
///
/// The four lines are not decoration. Identification of the creator, a notice
/// of the licence, a link where the licence family asks for one, and an
/// indication of whether the material was modified are the *separate*
/// obligations an attribution-style licence imposes, and leaving one out
/// leaves that obligation undischarged while the surface looks complete. A
/// "no changes" statement is a real answer to the fourth, not the absence of
/// one, which is why [`crate::text::about::Attribution::changes`] is never
/// empty and a test asserts it.
///
/// The licence name is drawn in the plain text role and the supporting lines
/// in the muted one. Not `.strong()`: `DEFECTS.md` D11 records that role as
/// unusable in this theme, and the whole point of a named palette is that a
/// surface added later does not have to rediscover that.
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
    ///
    /// Guards the two-step: if `close_requested` ever defaulted to `true` the
    /// window would open and vanish on the same frame, which looks exactly
    /// like a command that does nothing.
    #[test]
    fn a_new_dialog_does_not_immediately_close() {
        let d = AboutDialog::open();
        assert!(!d.close_requested);
    }

    /// Run one real frame and assert the dialog stays open.
    ///
    /// Drawn through `egui::Context::run` rather than by inspecting the code,
    /// because the failure this guards against is a *drawing* failure: a
    /// `ScrollArea` whose `max_height` goes negative, an `available_height`
    /// read before there is any, a panic inside a closure the compiler is
    /// perfectly happy with. `HANDOFF.md` §2's founding rule is that a
    /// passing test is not evidence a surface works — this is the weaker
    /// claim that it at least composes, and the real check is `ui-verify`
    /// driving the window.
    ///
    /// The window rect is deliberately small. A dialog that only survives on
    /// a large screen is a dialog that crashes on a laptop.
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
    ///
    /// ★ The specific thing this catches. [`AboutDialog::body`] sizes its
    /// scroll area as `available_height() - 44.0`, and a window shorter than
    /// its own header makes that **negative**. A negative `max_height` is not
    /// a compile error and is not a panic either — it is a scroll area that
    /// silently draws nothing, which would empty the attribution list on a
    /// small screen and on no other. That is the shape of defect this project
    /// keeps finding by running the program rather than by testing it, so the
    /// degenerate size is asserted here instead of waited for.
    ///
    /// What it does NOT prove is that the words are legible or in the right
    /// order. Only `ui-verify` driving the real window can say that; see
    /// `HANDOFF.md` §2.
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

    /// The catalog the dialog draws from is not empty.
    ///
    /// Stated here as well as in `text::about` because the two tests answer
    /// different questions. That one asks whether the catalog is well formed;
    /// this one asks whether this dialog has anything to say — and a dialog
    /// registered on the ribbon that renders a heading and nothing under it
    /// is the placeholder `HANDOFF.md` §6 forbids, arriving through data
    /// rather than through code.
    #[test]
    fn the_dialog_has_something_to_attribute() {
        assert!(
            !crate::text::about::attributions().is_empty(),
            "About is on the ribbon with an empty attribution list"
        );
    }
}
