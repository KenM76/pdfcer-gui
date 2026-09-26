//! # `app::status::readmode` — the one line that says how to get the
//! application back
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/status/readmode.md`.

use egui::{Align, Layout, Vec2};

use super::ROW_HEIGHT_PTS;
use crate::text::window as t;

/// The share of the bar this line may occupy before eliding.
const EXIT_WIDTH_FRACTION: f32 = 0.60;

/// The region the line publishes for `ui-verify`.
///
/// A published region name is a cross-repo stability contract with the
/// harness: renaming it turns a check into a skip rather than a failure.
pub(super) const REGION_READ_MODE_EXIT: &str = "status-group:read-mode-exit"; // ui-text-exempt: trace region name, never displayed

/// The trace slot the line publishes, de-duplicated on the rendered sentence.
const EXIT_SLOT: &str = "read-mode-exit"; // ui-text-exempt: trace slot name, never displayed

/// Draw the exit statement, if read mode is on.
///
/// Returns nothing: like the rest of the left half this is a readout, and the
/// one case that is not — the unbound button — acts on `egui::Memory` rather
/// than raising an action (see the module header).
///
/// **Called before [`super::show`]'s `Status::Open` guard**, deliberately. Read
/// mode is per *window*, not per document (`app::window` §3), so an operator can
/// close their last file while in it — and a bar that only explained the way out
/// when a document happened to be open would go silent in the state where the
/// window has the least in it.
pub(super) fn show(ui: &mut egui::Ui) {
    let ctx = ui.ctx().clone();
    if !crate::app::window::read_mode(&ctx) {
        return;
    }

    // Both chords come from `app::window`'s published values, resolved once
    // per frame from the keymap that dispatches. Nothing in this file spells a
    // key, and `crate::text::window`'s own test forbids the catalog spelling one
    // either — so there is no place left for the sentence and the binding to
    // disagree.
    let exit = crate::app::window::exit_chord(&ctx);
    // Named only in the combined state. Full screen alone keeps the ribbon and
    // therefore keeps its own control; it is read mode that took the control
    // away, so it is read mode that owes the sentence.
    let full = crate::app::window::fullscreen(&ctx)
        .then(|| crate::app::window::fullscreen_chord(&ctx))
        .flatten();

    let width = (ui.available_width() * EXIT_WIDTH_FRACTION).max(0.0);
    let mut line = String::new();
    let rect = ui
        .allocate_ui_with_layout(
            Vec2::new(width, ROW_HEIGHT_PTS),
            Layout::left_to_right(Align::Center),
            |ui| match (&exit, &full) {
                (Some(exit), Some(full)) => {
                    line = t::status_read_mode_and_fullscreen(exit, full);
                    statement(ui, &line);
                }
                (Some(exit), None) => {
                    line = t::status_read_mode(exit);
                    statement(ui, &line);
                }
                // No chord is bound to `view.read_mode` in this build, so there
                // is nothing true to say about a key and the only honest surface
                // is the route itself. See the module header.
                (None, _) => {
                    line = t::status_read_mode_unbound().to_owned();
                    statement(ui, &line);
                    if ui.button(t::leave_read_mode_button()).clicked() {
                        let on = crate::app::window::toggle_read_mode(ui.ctx());
                        crate::diag::trace(|| {
                            // ui-text-exempt: diagnostic trace, never displayed
                            format!("read-mode on={on} by=status-bar")
                        });
                    }
                }
            },
        )
        .response
        .rect;

    crate::diag::ui_rect(REGION_READ_MODE_EXIT, rect);
    // Plain quoted strings rather than `Option`'s `Some("…")` debug form: the
    // harness's field parser gives `(` structural meaning, and an unbound chord
    // is expressed as the empty string. A trace shape a check has to
    // special-case is a trace shape a check gets wrong.
    let chord_field = exit.clone().unwrap_or_default();
    let full_field = full.clone().unwrap_or_default();
    crate::diag::trace_changed(EXIT_SLOT, || {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        //
        // `chord=` beside `line=` so a failing check can tell "the keymap
        // resolved nothing" from "the sentence dropped the chord it was
        // handed" — two different defects that produce the same missing text.
        // It is also the tie a check needs: `chord=` is what the KEYMAP
        // answered, `line=` is what the OPERATOR is shown, and asserting the
        // second quotes the first is the only external statement that a
        // re-introduced hard-coded chord would fail.
        format!("{EXIT_SLOT} chord={chord_field:?} fullscreen={full_field:?} line={line:?}")
    });
}

/// One elided label with the whole sentence on hover.
fn statement(ui: &mut egui::Ui, line: &str) {
    ui.add(egui::Label::new(line).truncate())
        .on_hover_text(line.to_owned());
    ui.separator();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::status::test_support::settled_bar_frame;
    use egui::Context;

    /// **Nothing is said when read mode is off**, and this is the half of
    /// the pair that is easy to get vacuously right.
    #[test]
    fn the_ordinary_state_says_nothing_about_read_mode() {
        let ctx = Context::default();
        assert!(!crate::app::window::read_mode(&ctx));
        let status = crate::app::status::test_support::opened();
        let before = settled_bar_frame(&ctx, &status).expect("the bar laid out");

        crate::app::window::toggle_read_mode(&ctx);
        let after = settled_bar_frame(&ctx, &status).expect("the bar laid out");

        assert!(
            after.1 > before.1,
            "turning read mode on must add shapes to the bar: {before:?} → {after:?}"
        );
        assert_eq!(
            after.0, before.0,
            "★ R128: the line must not change the bar's height, or a fit-to-viewport \
             zoom recomputes from a smaller canvas and the page visibly shrinks"
        );
    }

    /// **The line names the chord the keymap actually holds.**
    #[test]
    fn the_sentence_names_the_binding_the_keymap_holds() {
        let shell = crate::shell::manifest::built_in();
        let bound = crate::app::window::chord_for(shell.keymap.as_ref(), "view.read_mode")
            .expect("the built-in manifest binds a chord to view.read_mode");

        let ctx = Context::default();
        crate::app::window::publish_exit_chord(&ctx, Some(&shell));
        let published = crate::app::window::exit_chord(&ctx).expect("a published chord");
        assert_eq!(published, bound);

        let sentence = t::status_read_mode(&published);
        assert!(
            sentence.contains(bound),
            "the statement must name the live binding: {sentence:?} does not contain {bound:?}"
        );
    }

    /// **A context nothing has published into offers no chord**, and the line
    /// falls to the button rather than to a guess.
    #[test]
    fn an_unpublished_context_names_no_key() {
        let ctx = Context::default();
        assert_eq!(crate::app::window::exit_chord(&ctx), None);
        crate::app::window::publish_exit_chord(&ctx, None);
        assert_eq!(crate::app::window::exit_chord(&ctx), None);
    }
}
