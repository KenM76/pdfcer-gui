//! # `canvas::textsel::clipboard` — the two chords, and the one place this
//! shell writes the clipboard
//!
//! Split out of [`super`] on 2026-08-14, when the text-markup work pushed that
//! file past **R2** (no `.rs` file over 1,500 lines). The seam is not an
//! arbitrary cut at a line number; it is the one the module had already drawn
//! in prose, and the two halves change for different reasons:
//!
//! | half | answers | changes when |
//! |---|---|---|
//! | [`super`] | *what is selected, and what geometry describes it* | a gesture, a hit rule or a derivation changes |
//! | this file | *what the operator can do to what is selected, with a key* | a chord, a guard or the clipboard contract changes |
//!
//! There is a second, sharper reason this is the right seam rather than the
//! convenient one: **[`copy`] is not about a canvas selection at all.** Three
//! verbs reach it — the canvas selection's `Ctrl+C`, `file.copy_page_text` and
//! `file.copy_document_text` — and the last two arrive from
//! `crate::app::dispatch` with a whole page's or a whole document's extraction
//! and no selection anywhere in sight. A function two ribbon commands call
//! belongs beside the clipboard contract it enforces, not inside the module
//! that resolves ranges.
//!
//! Everything here is re-exported flat from [`super`], so every existing call
//! site still writes `textsel::copy` and `textsel::pending_key` and nothing
//! outside `canvas/` learns that the module was split — the same courtesy
//! `shell::commands::mapping`'s split extended to its callers, and for the same
//! reason: a file-size rule that rewrites unrelated call sites is a rule that
//! manufactures diffs.

use super::{PageContext, TextSelection, select_all};

/// One of the two keyboard verbs a text selection has, as read off the frame's
/// input **before** anything expensive is fetched.
///
/// See [`pending_key`] for why the read is split from the act.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextKey {
    /// `Ctrl+A` — select every character on the page.
    SelectAll,
    /// `Ctrl+C` — copy what is selected.
    Copy,
}

/// ★ **Whether either text chord was pressed this frame** — the cheap half,
/// asked before the page's extraction is fetched.
///
/// # The defect this split closes, and it was found by driving the binary
///
/// The first version of this feature had one function that took a
/// [`PageContext`], read the two chords, and acted. `canvas::interact` therefore
/// had to build that context — which means calling
/// [`crate::app::state::OpenDoc::page_text`] — **on every frame in a reading
/// mode**, in order to discover that no chord had been pressed.
///
/// The cache made that one extraction rather than sixty a second, so no test
/// noticed and the trace showed exactly one `page-text` line, as designed. What
/// driving the real binary showed was *when* that line appeared: at **open**,
/// before the operator had touched anything, at a measured **392 ms** on
/// `ncored-benchmark-cad-drawing.pdf`. A reader opening a dense drawing paid
/// four tenths of a second for a gesture they had not made and might never make.
///
/// It is the same failure [`crate::app::state::OpenDoc::page_objects`]'s own
/// gate exists for, in that method's own words: *"asking for it on a frame that
/// has no hit test to do would decompose the page the first time the operator
/// merely zoomed"*. The difference is that the decomposition's gate is in its
/// caller, and this one was missing — because the expensive value was being
/// fetched in order to answer a question that did not need it.
///
/// So the read is split from the act. This function touches nothing but
/// `egui::InputState`; `canvas::interact` fetches the extraction only when it
/// answers `Some`, and a reading canvas nobody is typing at costs one input read
/// per frame. **The extraction now happens on the first text gesture rather than
/// on the first frame** — which is when the operator asked for it.
///
/// # The guard
///
/// `text_edit_focused()`, the `DEFECTS.md` D1 predicate, and this is the
/// sharpest instance of D1's own reason in the product: `Ctrl+A` and `Ctrl+C`
/// are what an operator presses **inside** the Find field or the status bar's
/// page box, and a canvas that took them would make the two most reflexive
/// keystrokes in the application select and copy a page instead of the text they
/// were typing. It is `text_edit_focused()` and never
/// `egui_wants_keyboard_input()` — see `app::keyboard`'s header for why that
/// distinction is not a nicety.
///
/// # Why Copy wins a frame carrying both
///
/// Unreachable from a keyboard, and answered anyway: the **narrower** verb wins,
/// so a synthetic frame carrying both copies what was selected rather than
/// copying the whole page it selected a microsecond earlier. A rule that is
/// stated cannot be got wrong by a later reader who reaches that state from a
/// script.
#[must_use]
pub fn pending_key(ui_ctx: &egui::Context) -> Option<TextKey> {
    // ★★ A canvas draft claims these chords too, 2026-08-20 — widened from
    // `text_edit_focused()`, which is false for an operator typing on the page.
    //
    // Ctrl+C mid-word must not copy the page's text selection: the operator is
    // composing, and the selection they made before the caret landed is not
    // what those two keys mean any more. Same reasoning as the space bar, which
    // was taken by the hand tool for the same reason on the same day.
    if crate::canvas::textedit::composing(ui_ctx) {
        return None;
    }
    ui_ctx.input(|i| {
        // ★★★ COPY IS READ AS AN EVENT, NOT AS A KEY, AND THAT DISTINCTION IS
        // THE WHOLE OF DEFECT O18.
        //
        // This function used to ask `i.key_pressed(egui::Key::C)`. In a real
        // window that is **permanently false**, because of fifteen lines of
        // `egui-winit-0.35.0/src/lib.rs`:
        //
        // ```rust
        // if is_cut_command(modifiers, active_key)   { events.push(Event::Cut);   return; }
        // if is_copy_command(modifiers, active_key)  { events.push(Event::Copy);  return; }
        // if is_paste_command(modifiers, active_key) { … events.push(Event::Paste(c)); return; }
        // events.push(Event::Key { … });
        // ```
        //
        // The `return` comes **before** the `Event::Key` push, so `Ctrl+C`
        // yields `Event::Copy` and no key event at all. Every unit test below
        // passed throughout, because a test injects the key event winit never
        // sends — which is exactly how a dead path stays certified, and is why
        // those tests now inject `Event::Copy` instead.
        //
        // ★ `app::keyboard` recorded this same finding on 2026-08-20, under a
        // heading in capitals, and fixed itself. Nobody asked who ELSE read the
        // signal — the answer was one grep away, and it was this file. The
        // operator lost a working Ctrl+C for a day because a lesson was written
        // down instead of being applied.
        if i.events.iter().any(|e| matches!(e, egui::Event::Copy)) {
            return Some(TextKey::Copy);
        }
        // ★ Ctrl+A is NOT intercepted by winit and does arrive as a key event,
        // so it is still read as one. The asymmetry is winit's, not ours, and
        // collapsing the two into one style would break whichever half was
        // made to match the other.
        if i.modifiers.command && i.key_pressed(egui::Key::A) {
            return Some(TextKey::SelectAll);
        }
        None
    })
}

/// **Act on the chord [`pending_key`] found.**
///
/// Split from the read for the cost reason that function's header records; this
/// half is the one that needs the page's extraction, and it is reached only on
/// the frames where there is something to do with it.
///
/// Handled here rather than in [`crate::canvas::keys`] because both verbs need
/// that extraction, and `canvas_keys` is deliberately a *document-free* function
/// a headless `egui::Context` can drive end to end. Escape **is** in
/// `canvas_keys`, because clearing needs nothing but the field, and because it
/// has a precedence question to answer that these two do not.
///
/// Traces its own outcome: the caller cannot see which verb fired, and a
/// selection is otherwise invisible from outside the process — see
/// [`crate::canvas::trace::text_selection`] for that argument.
pub fn apply_key(
    ui_ctx: &egui::Context,
    ctx: &PageContext<'_>,
    key: TextKey,
    current: &mut Option<TextSelection>,
) {
    match key {
        TextKey::Copy => {
            if let Some(selection) = current.as_ref().filter(|s| s.live(ctx.epoch)) {
                copy(ui_ctx, &selection.text, "selection");
            } else {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed in the UI.
                    // Distinct from `copy`'s own `nothing-to-copy`, which is
                    // about an empty string: this is "there was no selection to
                    // read one from", a different fact and the likelier one.
                    "text-copy-declined source=selection reason=no-live-selection".to_owned()
                });
            }
        }
        TextKey::SelectAll => {
            *current = select_all(ctx);
            crate::canvas::trace::text_selection(ctx.index, current.as_ref(), "all");
        }
    }
}

/// **Put text on the clipboard, and say so.**
///
/// The one place this shell writes the clipboard. Three verbs reach it — the
/// canvas selection's Ctrl+C, `file.copy_page_text` and
/// `file.copy_document_text` — and routing all of them through one function is
/// what makes the trace line below a complete record of what pdfcer has copied
/// rather than one of three partial ones.
///
/// It raises no [`crate::app::actions::Action`], and that is the same call
/// `file.print` makes for the same reason: the funnel exists for work that
/// touches a document or that must not happen mid-frame, and a clipboard write
/// is neither. `egui::Context::copy_text` queues an output command that the
/// backend spends after the frame anyway.
///
/// An **empty** string is refused rather than written. Copying nothing would
/// silently destroy whatever the operator had on their clipboard — which is
/// their data, from another application, and not pdfcer's to discard — and the
/// decline is traced so the difference between "copied nothing" and "did not
/// run" is on the record.
pub fn copy(ui_ctx: &egui::Context, text: &str, source: &str) {
    if text.is_empty() {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("text-copy-declined source={source} reason=nothing-to-copy")
        });
        return;
    }
    ui_ctx.copy_text(text.to_owned());
    // Not de-duplicated: two copies are two events, and a harness must be able
    // to tell a second Ctrl+C from a silence.
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("text-copied source={source} chars={}", text.len())
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★ **A frame with no chord costs one input read and nothing else.**
    ///
    /// The regression test for the defect that shipped and was caught by driving
    /// the binary: `canvas::interact` used to fetch the page's extraction in
    /// order to *discover* that no chord had been pressed, which built it on the
    /// first frame of every reading canvas — 392 ms at open on the benchmark
    /// drawing. [`pending_key`] is the cheap half that made the gate possible,
    /// and its `None` on an idle frame is the whole of what the gate rests on.
    ///
    /// It is asserted at the level of the **predicate** rather than by counting
    /// extractions, because that is where the property lives: the caller's `if
    /// let` cannot fetch anything when this answers `None`, whatever else it
    /// does.
    #[test]
    fn an_idle_frame_asks_for_no_text_chord() {
        let ctx = egui::Context::default();
        let mut found = Some(TextKey::Copy);
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            found = pending_key(ui.ctx());
        });
        assert_eq!(
            found, None,
            "an idle canvas must not make the caller reach for the page's text"
        );
    }

    /// ★ **A focused text field keeps Ctrl+A and Ctrl+C** — `DEFECTS.md` D1's
    /// guard, at the sharpest instance of it in the product.
    ///
    /// These are the two chords an operator presses *inside* the Find field. A
    /// canvas that took them would select and copy the page instead of the text
    /// being typed, which is the same failure D1 produced with Delete and would
    /// be more surprising, because the operator can see the field they are in.
    ///
    /// Built against a **real** `TextEdit`, for the reason
    /// `canvas::keys::a_focused_text_field_keeps_delete_for_itself` gives:
    /// `text_edit_focused()` resolves the focused id and looks for a
    /// `TextEditState` under it, so a hand-requested focus on a bare id would
    /// pass vacuously.
    #[test]
    fn a_focused_text_field_keeps_the_text_chords() {
        let ctx = egui::Context::default();
        let mut buffer = String::from("total");

        // Frame 1: build the field and take focus.
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            ui.add(egui::TextEdit::singleline(&mut buffer))
                .request_focus();
        });

        // Frame 2: the field holds focus and Ctrl+C is pressed.
        //
        // ★ `Event::Copy`, which is what winit actually sends — not the key
        // event this test used to inject. See `pending_key`.
        let input = egui::RawInput {
            events: vec![egui::Event::Copy],
            modifiers: egui::Modifiers::COMMAND,
            ..Default::default()
        };
        let mut typing = false;
        let mut found = Some(TextKey::SelectAll);
        let _ = ctx.run_ui(input, |ui| {
            ui.add(egui::TextEdit::singleline(&mut buffer));
            // typing-guard-exempt: a TEST asserting the harness actually reached
            // the focused state. Reading the raw egui answer is the point - a
            // test that asked `composing()` could not tell a focused widget from
            // a canvas draft, and the thing being proved is that the widget half
            // is reachable at all. D1 shipped because its test could not reach it.
            typing = ui.ctx().text_edit_focused();
            found = pending_key(ui.ctx());
        });

        assert!(
            typing,
            "the test is vacuous unless a TEXT field really holds focus"
        );
        assert_eq!(
            found, None,
            "a focused field must keep the two chords an operator uses inside it"
        );
    }

    /// …and with nothing focused, the same chord really does reach the canvas —
    /// without which the test above would pass on a build where the chords never
    /// worked at all.
    #[test]
    fn the_text_chords_reach_an_unfocused_canvas() {
        // ★★ THE TWO CHORDS ARRIVE IN TWO DIFFERENT SHAPES, and injecting the
        // wrong one is how O18 shipped: this loop used to send `Event::Key {
        // key: C }` for copy, which winit never sends, so it certified a path
        // that could not fire in the running application for a single frame.
        //
        // Copy is intercepted by `egui-winit` and arrives as `Event::Copy`
        // with no key event; Ctrl+A is not intercepted and arrives as an
        // ordinary key event. The asymmetry is winit's and the test has to
        // mirror it exactly, because a test that normalises the two is a test
        // that has stopped describing the program.
        let cases: [(egui::Event, TextKey); 2] = [
            (egui::Event::Copy, TextKey::Copy),
            (
                egui::Event::Key {
                    key: egui::Key::A,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::COMMAND,
                },
                TextKey::SelectAll,
            ),
        ];
        for (event, want) in cases {
            let ctx = egui::Context::default();
            let input = egui::RawInput {
                events: vec![event.clone()],
                modifiers: egui::Modifiers::COMMAND,
                ..Default::default()
            };
            let mut found = None;
            let _ = ctx.run_ui(input, |ui| found = pending_key(ui.ctx()));
            assert_eq!(found, Some(want), "{event:?}");
        }

        // ★ And the regression itself, stated as its own assertion: a bare
        // `Ctrl+C` KEY EVENT must no longer be what copy listens for. If a
        // future edit reinstates `key_pressed(Key::C)` this fails, and the
        // failure names the reason rather than leaving somebody to rediscover
        // fifteen lines of a dependency.
        //
        // clipboard-chord-exempt: this test injects the DEAD form deliberately,
        // to prove it is dead. It is the one place in the crate that should
        // mention `Key::C`, and `tools/gates/check-clipboard-chords.sh` exists
        // to make sure it stays the only one.
        let ctx = egui::Context::default();
        let input = egui::RawInput {
            events: vec![egui::Event::Key {
                key: egui::Key::C,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::COMMAND,
            }],
            modifiers: egui::Modifiers::COMMAND,
            ..Default::default()
        };
        let mut found = Some(TextKey::SelectAll);
        let _ = ctx.run_ui(input, |ui| found = pending_key(ui.ctx()));
        assert_eq!(
            found, None,
            "copy must listen for Event::Copy, not for a Ctrl+C key event that \
             egui-winit never emits — that mistake is defect O18"
        );

        // …and the same letters **unmodified** are not chords at all. `A` and
        // `C` are ordinary keys; a canvas that selected the page when the
        // operator pressed `a` would be unusable.
        let ctx = egui::Context::default();
        let input = egui::RawInput {
            events: vec![egui::Event::Key {
                key: egui::Key::A,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
            ..Default::default()
        };
        let mut found = Some(TextKey::Copy);
        let _ = ctx.run_ui(input, |ui| found = pending_key(ui.ctx()));
        assert_eq!(found, None, "a bare `A` is a letter, not Select All");
    }
}
