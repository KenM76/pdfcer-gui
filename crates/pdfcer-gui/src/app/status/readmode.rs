//! # `app::status::readmode` — the one line that says how to get the
//! application back
//!
//! One sentence, drawn only while `view.read_mode` is on, at the left end of
//! the status bar and ahead of everything else the bar has to say.
//!
//! # ★★★ The report, and why the status bar of all places
//!
//! The operator, 2026-09-05:
//!
//! > *"I didn't see a way to get back out of read mode. if there is a shortcut
//! > for this it should have a note what the key combo is in the top bar that
//! > holds the window controls."*
//!
//! He named the title bar, and the title carries it too — see
//! [`crate::text::doctabs::window_title`]. This module is the **second**
//! surface, and two surfaces for one fact is normally a smell, so the argument
//! has to be made rather than assumed.
//!
//! It is made in [`crate::app::window`]'s header and it is compositional:
//! **read mode composes with full screen.** With both on there is no ribbon
//! *and no title bar*, so a title-only hint would be missing in exactly the
//! state that has the least chrome left. The status bar is the one piece of
//! chrome `app::window` §2 deliberately keeps, and it survives both. In the
//! other direction the title is legible from the taskbar and from Alt-Tab,
//! which this bar is not, and it is legible on a maximised window whose bottom
//! edge the operator is not looking at. **Neither surface alone covers the
//! state space, and each is the other's blind spot** — the same two-channel
//! reasoning `ui-verify`'s `read_mode_hides_the_chrome` uses for the rect and
//! the pixels.
//!
//! # ★★ It is FIRST on the left, ahead of the page-drag caption
//!
//! `app::status`' header ranks the left half: the transient caption about a
//! gesture in progress outranks the disclosures about gestures already
//! finished, which outrank the narrator. This line outranks all three, and the
//! rule that puts it there is worth stating because it generalises:
//!
//! > **A sentence about how to reach the interface outranks every sentence
//! > about the document**, because an operator who cannot reach the interface
//! > cannot act on any of the others.
//!
//! # ★ It is never shed, and it never sheds anything else
//!
//! Two separate properties, and the bar's own machinery gives each:
//!
//! * **Never shed.** `fitting::SHED_ORDER` governs the fixed cluster on the
//!   *right*. This is a left-hand line, so no width pressure can remove it —
//!   which matters, because `fitting`'s reachability clause (*nothing it may
//!   drop is the operator's last route to that capability*) would forbid
//!   dropping this one anyway, and a rule enforced by construction beats a rule
//!   enforced by a list.
//! * **Sheds nothing.** It takes a bounded fraction of the width and elides,
//!   with the whole sentence on hover, exactly as the render notes do. An
//!   unbounded label here would push the page box and the zoom stepper off the
//!   right-hand end — which is defect O44's shape, and the shed mechanism
//!   exists because it has happened.
//!
//! [`EXIT_WIDTH_FRACTION`] is wider than the notes' 0.45 and the reason is the
//! ranking above: this is the sentence somebody is *hunting for*, where the
//! notes are volunteered.
//!
//! # ★ R128 — it cannot change the bar's height
//!
//! One label, on the row [`super::show`] has already allocated, of
//! [`super::ROW_HEIGHT_PTS`]. Nothing here wraps and nothing adds a line. The
//! measured failure R128 names — a status bar that grew by one line, a
//! fit-to-viewport zoom that recomputed from the smaller canvas, and a page
//! that visibly shrank across three frames — would arrive here the moment this
//! sentence were allowed to wrap.
//!
//! # ★★ The two shapes: a statement, and (rarely) a control
//!
//! | the keymap binds `view.read_mode` to… | what is drawn |
//! |---|---|
//! | a chord | **a statement** naming it. R9: this is not a placeholder and not a greyed control; it is a fact |
//! | nothing | **a button** that leaves read mode |
//!
//! The second row is not hedging. A build whose manifest binds no chord to
//! `view.read_mode` is legal — `SHELL_FRAMEWORK.md` §5 lets an operator rebind
//! keys and R8 lets a stripped build drop commands — and in that build the
//! ribbon control is hidden, the chord does not exist, and **there is no way
//! back at all short of restarting the application.** A statement has nothing
//! true to say there; the choice is between a control and a trap.
//!
//! With a chord bound, the statement is the better surface and the button would
//! be the worse one: a statement teaches the keyboard and leaves the bar a
//! readout, while a button invites the operator to keep coming back to the
//! mouse for something they now know a key for.
//!
//! ★ The button writes `egui::Memory` directly rather than raising an
//! [`crate::app::actions::Action`], which is the same licence the Find toggle
//! twenty lines away takes and for the identical reason `app::window`'s §4
//! gives: nothing here touches a document, so there is nothing for the undo log
//! to hold and nothing to order against.

use egui::{Align, Layout, Vec2};

use super::ROW_HEIGHT_PTS;
use crate::text::window as t;

/// The share of the bar this line may occupy before eliding.
///
/// Wider than [`super::NOTES_WIDTH_FRACTION`] (0.45) on purpose — see the
/// module header's ranking argument. Bounded all the same, because the controls
/// on the right are what an operator uses to *read* the document they are in
/// read mode to read.
const EXIT_WIDTH_FRACTION: f32 = 0.60;

/// The region the line publishes for `ui-verify`.
///
/// ★ A published region name is a cross-repo stability contract with the
/// harness: renaming it turns a check into a skip rather than a failure.
pub(super) const REGION_READ_MODE_EXIT: &str = "status-group:read-mode-exit"; // ui-text-exempt: trace region name, never displayed

/// The trace slot the line publishes, de-duplicated on the rendered sentence.
///
/// ★★ It carries the **sentence**, not a boolean, and that is what makes a
/// driven check able to fail correctly. `read-mode-exit shown=true` is
/// identical for a build that names the right chord, a build that names a chord
/// nothing is bound to, and a build that names no chord at all — so a check
/// reading it could only assert *something appeared*, which is the vacuous
/// shape this project has shipped before. The text is the claim; the trace
/// carries the claim.
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

    // ★ Both chords come from `app::window`'s published values, resolved once
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
    // ★ Plain quoted strings rather than `Option`'s `Some("…")` debug form: the
    // harness's field parser gives `(` structural meaning, and an unbound chord
    // is expressed as the empty string. A trace shape a check has to
    // special-case is a trace shape a check gets wrong.
    let chord_field = exit.clone().unwrap_or_default();
    let full_field = full.clone().unwrap_or_default();
    crate::diag::trace_changed(EXIT_SLOT, || {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        //
        // ★ `chord=` beside `line=` so a failing check can tell "the keymap
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
///
/// The same four rules `super::disclosure::disclosure_line` applies — bounded
/// width, fixed row, elide rather than wrap, full text on hover — written here
/// rather than reused because that helper draws `.small()`, which is right for
/// narration the operator did not ask for and wrong for the one sentence they
/// are hunting.
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

    /// ★★★ **Nothing is said when read mode is off**, and this is the half of
    /// the pair that is easy to get vacuously right.
    ///
    /// A check that only asserted the sentence *appears* would pass on a build
    /// that showed it permanently — which would be furniture, and would be a
    /// false statement for every minute the mode is off.
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
    ///
    /// The vacuous shape this forbids: a test that asserts a sentence exists
    /// passes on a sentence naming the wrong key. What is asserted is the
    /// identity of two derivations — the one this module draws, and the one
    /// taken from `shell::manifest::built_in`'s keymap — so a rebind moves both
    /// or fails here.
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
    ///
    /// The failure this forbids is a default: `Ctrl+H` as a fallback would be a
    /// second spelling of the binding wearing a fallback's clothes, and it would
    /// be wrong in exactly the case it was reached for.
    #[test]
    fn an_unpublished_context_names_no_key() {
        let ctx = Context::default();
        assert_eq!(crate::app::window::exit_chord(&ctx), None);
        crate::app::window::publish_exit_chord(&ctx, None);
        assert_eq!(crate::app::window::exit_chord(&ctx), None);
    }
}
