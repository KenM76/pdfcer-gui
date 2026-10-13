//! # `canvas::textedit::keys` — what every key means inside a draft
//!
//! ## What this is
//!
//!
//! ## Why it is its own file
//!
//! R2. `textedit/mod.rs` reached 1,571 lines the day the selection landed, and
//! the seam was already drawn: everything else in that module is about *what a
//! draft is* and *where it came from*, and this is about *what happens when a
//! key goes down*. The old shell's 25,005-line `main.rs` is the argument, and
//! the rule that prevents it is to split at the seam rather than to raise the
//! limit.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/textedit/keys.md`.

use egui::Ui;

use pdfcer_gui_base::editmodel::history::EditKind;

use super::caret::{self, insert};
use super::edits::{Flow, Keys};
use super::{Anchor, DIAG_TYPE, Draft, hit, read, store};
use crate::app::state::OpenDoc;

/// **What the pointer does inside the editor box** — place the caret, sweep a
/// selection, take a word on a double click or a line on a triple click.
fn pointer(ui: &Ui, ctx: &egui::Context, draft: &mut Draft) -> bool {
    let Some(layout) = hit::read(ctx) else {
        return false;
    };
    let (pressed, down, clicks, pos, origin) = ui.input(|i| {
        let primary = egui::PointerButton::Primary;
        let clicks = if i.pointer.button_triple_clicked(primary) {
            3
        } else if i.pointer.button_double_clicked(primary) {
            2
        } else {
            1
        };
        (
            i.pointer.primary_pressed(),
            i.pointer.primary_down(),
            clicks,
            i.pointer.interact_pos(),
            i.pointer.press_origin(),
        )
    });
    let Some(pos) = pos else {
        return false;
    };
    // The PRESS decides whether this gesture belongs to the box, not the
    // current position. A sweep that starts inside and runs out over the page
    // keeps selecting to the end of the text, which is what every text field
    // does; a press that starts outside never becomes the draft's business no
    // matter where it is dragged to.
    let began_inside = origin.is_some_and(|o| layout.body.contains(o));

    // A double or triple click is reported on its release, when egui has
    // already dropped the press origin, so it is the box's by where it ends.
    if clicks > 1 && layout.body.contains(pos) {
        let (from, to) = clicked_span(&draft.text, layout.index_at(pos), clicks);
        draft.mark = Some(from);
        draft.caret = to;
        return true;
    }
    if pressed && began_inside {
        draft.caret = layout.index_at(pos);
        draft.mark = None;
        return true;
    }
    if down
        && began_inside
        && let Some(origin) = origin
    {
        let from = layout.index_at(origin);
        let to = layout.swept_index_at(pos, draft.text.chars().count());
        if from != to {
            draft.mark = Some(from);
            draft.caret = to;
            return true;
        }
    }
    false
}

/// The span a multiple click at character `at` takes: on a double click the
/// word under it without its trailing space (or the run of spaces under it),
/// so typing over it keeps the separator; on a triple the draft's line.
fn clicked_span(text: &str, at: usize, clicks: u8) -> (usize, usize) {
    if clicks >= 3 {
        return (
            super::lines::start_of_line(text, at),
            super::lines::end_of_line(text, at),
        );
    }
    let chars: Vec<char> = text.chars().collect();
    let Some(last) = chars.len().checked_sub(1) else {
        return (0, 0);
    };
    let at = at.min(last);
    let blank = chars[at].is_whitespace();
    let same = |i: usize| chars[i].is_whitespace() == blank;
    let mut from = at;
    while from > 0 && same(from - 1) {
        from -= 1;
    }
    let mut to = at + 1;
    while to < chars.len() && same(to) {
        to += 1;
    }
    (from, to)
}

/// **Remove whatever is selected**, and answer the caret.
pub(super) fn take_selection(draft: &mut Draft) -> usize {
    let Some((from, to)) = caret::range(draft.mark, draft.caret) else {
        return draft.caret;
    };
    draft.mark = None;
    caret::delete_range(&mut draft.text, from, to)
}

/// **What pressing Enter does**, as three named outcomes.
///
/// `OPERATOR_REQUESTS.md` **O127**, defect 2 — the operator: *"can the enter key
/// create new lines when we are editing or creating text?"*
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnterMeans {
    /// Insert a line break at the caret. What Enter means, everywhere it can.
    NewLine,
    /// Finish the draft and write it. **`Ctrl+Enter`, in every draft.**
    Commit,
    /// A line break cannot go here, and the operator is told so by name. The
    /// draft is left alive.
    CannotSplit,
}

/// **Enter's whole contract, as a pure function.**
#[must_use]
pub const fn enter_means(anchor: &Anchor, command: bool) -> EnterMeans {
    if command {
        // Asked FIRST, so the chord is unconditional. An operator should not
        // have to know which gesture started the draft they are in to know how
        // to finish it — and O127's brief is explicit that commit must not be
        // reachable only by mouse.
        return EnterMeans::Commit;
    }
    match anchor {
        Anchor::Run { .. } => EnterMeans::CannotSplit,
        Anchor::Origin { .. } | Anchor::Box { .. } | Anchor::Block { .. } => EnterMeans::NewLine,
    }
}

/// **Consume this frame's keystrokes into the draft.** Answers `true` when the
/// draft was committed or handed on this frame.
pub fn typing(
    ui: &Ui,
    ctx: &egui::Context,
    doc: &OpenDoc,
    focused: bool,
    actions: &mut Vec<crate::app::actions::Action>,
) -> bool {
    let Some(mut draft) = read(ctx) else {
        return false;
    };
    // The pointer first: a press in the box moves the caret a keystroke in the
    // same frame must land at.
    let mut changed = pointer(ui, ctx, &mut draft);
    if changed {
        super::history::break_run(ctx);
    }
    // Keys held by the refused-keys notice go in once their face has landed.
    if let Some(back) = super::refused::resume(ctx, doc, &draft) {
        super::history::record(ctx, &draft, EditKind::Other);
        draft.caret = take_selection(&mut draft);
        draft.caret = insert(&mut draft.text, draft.caret, &back);
        changed = true;
    }
    changed |= seed(&mut draft);
    if focused {
        let mut keys = Keys {
            ctx,
            doc,
            draft: &mut draft,
            actions,
            // Read once: see [`caret::shifted`].
            frame_shift: ui.input(|i| i.modifiers.shift),
            changed,
        };
        for ev in ui.input(|i| i.events.clone()) {
            if keys.event(ev) == Flow::Done {
                return true;
            }
        }
        changed = keys.changed;
    }
    if changed {
        publish_selection(&draft);
        store(ctx, draft);
    }
    false
}

/// The diagnostic seam ([`DIAG_TYPE`]), consumed once per draft; whether it
/// changed the draft.
fn seed(draft: &mut Draft) -> bool {
    if draft.seeded {
        return false;
    }
    draft.seeded = true;
    if let Ok(seed) = std::env::var(DIAG_TYPE)
        && !seed.is_empty()
    {
        draft.text.clear();
        draft.caret = insert(&mut draft.text, 0, &seed);
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("text-edit-seeded len={}", draft.text.chars().count())
        });
    }
    true
}

/// The selection, traced for the harness as `text-select`, including the empty
/// case, so a check reads it without making an edit.
fn publish_selection(draft: &Draft) {
    crate::diag::trace_on_change("text-select", || {
        // ui-text-exempt: diagnostic trace, never displayed.
        match caret::range(draft.mark, draft.caret) {
            // ui-text-exempt: diagnostic trace, never displayed.
            Some((from, to)) => format!("from={from} to={to} n={}", to - from),
            // ui-text-exempt: diagnostic trace, never displayed.
            None => format!("none caret={}", draft.caret),
        }
    });
}

/// **Put the draft's selected text on the clipboard**, reporting whether there
/// was any.
pub(super) fn copy_selection(ctx: &egui::Context, draft: &Draft) -> bool {
    let Some((from, to)) = caret::range(draft.mark, draft.caret) else {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            "text-copy-declined source=draft reason=no-selection".to_owned()
        });
        return false;
    };
    let text: String = draft.text.chars().skip(from).take(to - from).collect();
    if text.is_empty() {
        return false;
    }
    crate::canvas::textsel::clipboard::copy(ctx, &text, "draft");
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::actions::Action;
    use crate::canvas::textedit::TextEditKind;

    /// A draft holding `text`, with the caret at the end and nothing selected.
    fn draft_of(ctx: &egui::Context, text: &str) {
        store(
            ctx,
            Draft {
                page: 0,
                kind: TextEditKind::Add,
                anchor: Anchor::Origin { x: 10.0, y: 10.0 },
                text: text.to_owned(),
                caret: text.chars().count(),
                mark: None,
                seeded: true,
            },
        );
    }

    /// Lay `text` out and publish it as the editor box, the way `paint` does.
    fn publish_layout(ctx: &egui::Context, text: &str) -> std::sync::Arc<egui::Galley> {
        let galley = ctx.fonts_mut(|f| {
            f.layout_no_wrap(
                text.to_owned(),
                egui::FontId::proportional(14.0),
                // NOT A THEME COLOUR: a test fixture. This galley is laid out
                // to be MEASURED, never drawn, and the colour is the one
                // argument `layout_no_wrap` will not let us omit. Taking it
                // from the theme would make the test depend on the palette to
                // assert an arithmetic property of text layout.
                egui::Color32::BLACK,
            )
        });
        let origin = egui::pos2(100.0, 100.0);
        hit::publish(
            ctx,
            hit::Layout {
                body: egui::Rect::from_min_size(origin, galley.rect.size() + egui::vec2(8.0, 8.0)),
                body_canvas: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1.0, 1.0)),
                caret: hit::Caret::Galley {
                    origin,
                    galley: galley.clone(),
                },
            },
        );
        galley
    }

    /// Where, on screen, the caret slot before character `i` sits.
    fn slot_x(galley: &egui::Galley, i: usize) -> f32 {
        100.0 + galley.pos_from_cursor(egui::text::CCursor::new(i)).min.x
    }

    /// Run one frame of `typing` with the given raw input.
    fn frame(ctx: &egui::Context, input: egui::RawInput) {
        let doc = crate::app::state::open_fixture(crate::app::state::FOUR_PAGES);
        let inner = ctx.clone();
        let mut actions = Vec::new();
        let _ = ctx.run_ui(input, move |c| {
            egui::CentralPanel::default().show(c, |ui| {
                typing(ui, &inner, &doc, true, &mut actions);
            });
        });
    }

    /// Raw input placing the pointer at `x` on the editor box's line, with the
    /// primary button in the given state.
    fn at(x: f32, down: bool) -> egui::RawInput {
        let mut input = egui::RawInput::default();
        let pos = egui::pos2(x, 108.0);
        input.events.push(egui::Event::PointerMoved(pos));
        input.events.push(egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: down,
            modifiers: egui::Modifiers::NONE,
        });
        input
    }

    /// A draft holding `text` with `from..to` selected.
    fn draft_selecting(ctx: &egui::Context, text: &str, from: usize, to: usize) {
        store(
            ctx,
            Draft {
                page: 0,
                kind: TextEditKind::Add,
                anchor: Anchor::Origin { x: 10.0, y: 10.0 },
                text: text.to_owned(),
                caret: to,
                mark: Some(from),
                seeded: true,
            },
        );
    }

    /// One frame carrying a single clipboard event.
    fn clipboard_frame(ctx: &egui::Context, event: egui::Event) {
        let mut input = egui::RawInput {
            modifiers: egui::Modifiers::COMMAND,
            ..Default::default()
        };
        input.events.push(event);
        frame(ctx, input);
    }

    /// **CTRL+C IN A TEXT BOX COPIES THE SELECTED TEXT** — defect O18, the
    /// operator's report of 2026-08-21.
    #[test]
    fn ctrl_c_in_a_text_box_copies_the_selection_and_changes_nothing() {
        let ctx = egui::Context::default();
        draft_selecting(&ctx, "SHEET 1 OF 4", 0, 5);
        clipboard_frame(&ctx, egui::Event::Copy);

        let after = read(&ctx).expect("a copy must not end the draft");
        assert_eq!(after.text, "SHEET 1 OF 4", "a copy is not an edit");
        assert_eq!(after.caret, 5, "a copy must not move the caret");
        assert_eq!(after.mark, Some(0), "a copy must not drop the selection");
    }

    /// **CTRL+X removes what it copied**, and copy runs first.
    #[test]
    fn ctrl_x_in_a_text_box_cuts_the_selection() {
        let ctx = egui::Context::default();
        draft_selecting(&ctx, "SHEET 1 OF 4", 0, 6);
        clipboard_frame(&ctx, egui::Event::Cut);

        let after = read(&ctx).expect("a cut must not end the draft");
        assert_eq!(after.text, "1 OF 4");
        assert_eq!(after.caret, 0, "the caret lands where the cut text began");
        assert_eq!(after.mark, None, "a stale mark would index past the string");
    }

    /// **A CUT WITH NO SELECTION MUST DESTROY NOTHING.**
    #[test]
    fn ctrl_x_with_no_selection_destroys_nothing() {
        let ctx = egui::Context::default();
        draft_of(&ctx, "SHEET 1 OF 4");
        clipboard_frame(&ctx, egui::Event::Cut);

        let after = read(&ctx).expect("the draft survives");
        assert_eq!(after.text, "SHEET 1 OF 4", "a cut with nothing selected");
    }

    /// **CTRL+V pastes at the caret**, and replaces a selection if there is
    /// one — rule 2, the same rule typing obeys.
    #[test]
    fn ctrl_v_replaces_the_selection_the_way_typing_does() {
        let ctx = egui::Context::default();
        draft_selecting(&ctx, "SHEET 1 OF 4", 0, 5);
        clipboard_frame(&ctx, egui::Event::Paste("PLAN".to_owned()));

        let after = read(&ctx).expect("the draft survives a paste");
        assert_eq!(after.text, "PLAN 1 OF 4");
        assert_eq!(after.caret, 4, "the caret lands after what was pasted");
    }

    /// A paste with nothing selected inserts at the caret rather than appending.
    #[test]
    fn ctrl_v_with_no_selection_inserts_at_the_caret() {
        let ctx = egui::Context::default();
        store(
            &ctx,
            Draft {
                page: 0,
                kind: TextEditKind::Add,
                anchor: Anchor::Origin { x: 10.0, y: 10.0 },
                text: "SHEET 4".to_owned(),
                // Index 5 is between "SHEET" and the space before "4".
                caret: 5,
                mark: None,
                seeded: true,
            },
        );
        clipboard_frame(&ctx, egui::Event::Paste("S 1 OF".to_owned()));

        let after = read(&ctx).expect("the draft survives");
        assert_eq!(after.text, "SHEETS 1 OF 4");
    }

    /// New text keeps a pasted line break; a pasted tab is a space.
    #[test]
    fn a_multi_line_paste_into_new_text_keeps_its_lines() {
        let ctx = egui::Context::default();
        draft_of(&ctx, "");
        clipboard_frame(&ctx, egui::Event::Paste("one\r\ntwo\tthree".to_owned()));

        let after = read(&ctx).expect("the draft survives");
        assert_eq!(after.text, "one\ntwo three");
    }

    /// One frame carrying `key` with `modifiers`; the actions it pushed.
    fn key_frame(ctx: &egui::Context, key: egui::Key, modifiers: egui::Modifiers) -> Vec<Action> {
        let mut input = egui::RawInput {
            modifiers,
            ..Default::default()
        };
        input.events.push(egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        });
        frame_actions(ctx, input)
    }

    /// [`frame`], answering the actions the frame pushed.
    fn frame_actions(ctx: &egui::Context, input: egui::RawInput) -> Vec<Action> {
        let doc = crate::app::state::open_fixture(crate::app::state::FOUR_PAGES);
        let inner = ctx.clone();
        let out = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let sink = out.clone();
        let _ = ctx.run_ui(input, move |c| {
            egui::CentralPanel::default().show(c, |ui| {
                let mut actions = Vec::new();
                typing(ui, &inner, &doc, true, &mut actions);
                sink.lock().unwrap().extend(actions);
            });
        });
        std::mem::take(&mut *out.lock().unwrap())
    }

    fn type_frame(ctx: &egui::Context, text: &str) {
        let mut input = egui::RawInput::default();
        input.events.push(egui::Event::Text(text.to_owned()));
        frame(ctx, input);
    }

    #[test]
    fn ctrl_z_takes_back_a_run_of_typing_then_ctrl_y_puts_it_back() {
        let ctx = egui::Context::default();
        draft_of(&ctx, "AB");
        type_frame(&ctx, "c");
        type_frame(&ctx, "d");
        let none = key_frame(&ctx, egui::Key::Z, egui::Modifiers::COMMAND);
        assert!(none.is_empty(), "the draft's undo pushes no action");
        assert_eq!(read(&ctx).unwrap().text, "AB");
        key_frame(&ctx, egui::Key::Y, egui::Modifiers::COMMAND);
        assert_eq!(read(&ctx).unwrap().text, "ABcd");
    }

    /// Tab in a draft is taken out of the raw input before egui's focus walk
    /// can see it, and reaches the draft as spaces; Shift+Tab is dropped.
    #[test]
    fn a_tab_in_a_draft_never_reaches_the_focus_walk() {
        let ctx = egui::Context::default();
        draft_of(&ctx, "AB");
        let tab = |pressed, modifiers| egui::Event::Key {
            key: egui::Key::Tab,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers,
        };
        let mut input = egui::RawInput {
            events: vec![
                tab(true, egui::Modifiers::SHIFT),
                tab(true, egui::Modifiers::NONE),
                tab(false, egui::Modifiers::NONE),
            ],
            ..Default::default()
        };
        crate::canvas::textedit::claim_tab(&ctx, &mut input);
        assert_eq!(input.events, vec![egui::Event::Text("\t".to_owned())]);
        frame(&ctx, input);
        assert_eq!(read(&ctx).unwrap().text, "AB ");
    }

    /// With no draft open, Tab is left for egui's focus walk.
    #[test]
    fn a_tab_with_no_draft_is_left_alone() {
        let ctx = egui::Context::default();
        let mut input = egui::RawInput {
            events: vec![egui::Event::Key {
                key: egui::Key::Tab,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
            ..Default::default()
        };
        crate::canvas::textedit::claim_tab(&ctx, &mut input);
        assert_eq!(input.events.len(), 1);
    }

    #[test]
    fn ctrl_z_with_nothing_typed_undoes_the_document() {
        let ctx = egui::Context::default();
        draft_of(&ctx, "AB");
        let pushed = key_frame(&ctx, egui::Key::Z, egui::Modifiers::COMMAND);
        assert!(read(&ctx).is_none(), "the draft is settled first");
        assert_eq!(pushed.last(), Some(&Action::Undo));
    }

    #[test]
    fn ctrl_backspace_removes_the_word_before_the_caret() {
        let ctx = egui::Context::default();
        draft_of(&ctx, "SHEET 1 OF 4");
        key_frame(&ctx, egui::Key::Backspace, egui::Modifiers::COMMAND);
        key_frame(&ctx, egui::Key::Backspace, egui::Modifiers::COMMAND);
        assert_eq!(read(&ctx).unwrap().text, "SHEET 1 ");
    }

    #[test]
    fn ctrl_s_commits_the_draft_then_saves() {
        let ctx = egui::Context::default();
        draft_of(&ctx, "NEW");
        let pushed = key_frame(&ctx, egui::Key::S, egui::Modifiers::COMMAND);
        assert!(read(&ctx).is_none());
        assert!(matches!(pushed.first(), Some(Action::CommitAddText { .. })));
        assert!(matches!(
            pushed.last(),
            Some(Action::Save | Action::SaveCopy)
        ));
    }

    /// **A DRAG ACROSS THE TEXT SELECTS WHAT IT CROSSED** — the pointer
    /// half of `OPERATOR_REQUESTS.md` O14 item 11.
    #[test]
    fn a_drag_across_the_editor_box_selects_what_it_crossed() {
        let ctx = egui::Context::default();
        // One frame to warm the font stack, so `fonts()` has a real atlas.
        let _ = ctx.run_ui(egui::RawInput::default(), |_| {});
        let galley = publish_layout(&ctx, "SHEET 1 OF 4");
        draft_of(&ctx, "SHEET 1 OF 4");

        // Press before character 0, then drag to just past character 5.
        frame(&ctx, at(slot_x(&galley, 0) + 1.0, true));
        publish_layout(&ctx, "SHEET 1 OF 4");
        let mut input = egui::RawInput::default();
        input.events.push(egui::Event::PointerMoved(egui::pos2(
            slot_x(&galley, 5),
            108.0,
        )));
        frame(&ctx, input);

        let after = read(&ctx).expect("the draft survives a pointer gesture");
        assert_eq!(
            caret::range(after.mark, after.caret),
            Some((0, 5)),
            "the sweep must select the characters it crossed, not place a caret"
        );
    }

    /// `n` clicks at `x` on a box showing `text`, one frame per press or
    /// release, the box republished before each as `paint` does.
    fn clicks_at(ctx: &egui::Context, text: &str, x: f32, n: usize) {
        for _ in 0..n {
            for down in [true, false] {
                publish_layout(ctx, text);
                frame(ctx, at(x, down));
            }
        }
    }

    #[test]
    fn a_double_click_in_the_box_selects_the_word_under_it() {
        let ctx = egui::Context::default();
        let _ = ctx.run_ui(egui::RawInput::default(), |_| {});
        let galley = publish_layout(&ctx, "SHEET 1 OF 4");
        draft_of(&ctx, "SHEET 1 OF 4");
        clicks_at(&ctx, "SHEET 1 OF 4", slot_x(&galley, 2) + 1.0, 2);
        let after = read(&ctx).expect("the draft survives a double click");
        assert_eq!(caret::range(after.mark, after.caret), Some((0, 5)));
    }

    #[test]
    fn shift_down_on_the_last_line_selects_to_the_end_and_keeps_the_draft() {
        let ctx = egui::Context::default();
        draft_of(&ctx, "SHEET 1 OF 4");
        let mut d = read(&ctx).unwrap();
        d.caret = 3;
        store(&ctx, d);
        let actions = key_frame(&ctx, egui::Key::ArrowDown, egui::Modifiers::SHIFT);
        assert!(
            actions.is_empty(),
            "Shift+Down must not commit: {actions:?}"
        );
        let after = read(&ctx).expect("the draft stays open");
        assert_eq!(caret::range(after.mark, after.caret), Some((3, 12)));
        key_frame(&ctx, egui::Key::ArrowUp, egui::Modifiers::SHIFT);
        let after = read(&ctx).expect("the draft stays open");
        assert_eq!(caret::range(after.mark, after.caret), Some((0, 3)));
    }

    #[test]
    fn a_triple_click_in_the_box_selects_its_line() {
        let ctx = egui::Context::default();
        let _ = ctx.run_ui(egui::RawInput::default(), |_| {});
        let galley = publish_layout(&ctx, "SHEET 1 OF 4");
        draft_of(&ctx, "SHEET 1 OF 4");
        clicks_at(&ctx, "SHEET 1 OF 4", slot_x(&galley, 2) + 1.0, 3);
        let after = read(&ctx).expect("the draft survives a triple click");
        assert_eq!(caret::range(after.mark, after.caret), Some((0, 12)));
    }

    /// **A press with no travel places the caret and clears any selection**,
    /// which is the gesture that makes a sweep undoable by clicking.
    #[test]
    fn a_press_inside_the_box_places_the_caret_and_drops_the_selection() {
        let ctx = egui::Context::default();
        let _ = ctx.run_ui(egui::RawInput::default(), |_| {});
        let galley = publish_layout(&ctx, "SHEET 1 OF 4");
        draft_of(&ctx, "SHEET 1 OF 4");
        // Start with something selected, so the clearing is observable.
        let mut d = read(&ctx).unwrap();
        d.mark = Some(0);
        d.caret = 5;
        store(&ctx, d);

        frame(&ctx, at(slot_x(&galley, 3) + 1.0, true));
        let after = read(&ctx).expect("the draft survives a press");
        assert_eq!(after.caret, 3, "the caret goes where the pointer is");
        assert_eq!(
            caret::range(after.mark, after.caret),
            None,
            "a press drops the selection - rule 4 in the pointer's dialect"
        );
    }

    /// **A press that begins OUTSIDE the box is not the draft's business**,
    /// however far it is dragged into one. That is what keeps a marquee on the
    /// page from turning into a text selection when it happens to cross the
    /// editor.
    #[test]
    fn a_press_outside_the_box_never_becomes_a_selection() {
        let ctx = egui::Context::default();
        let _ = ctx.run_ui(egui::RawInput::default(), |_| {});
        let galley = publish_layout(&ctx, "SHEET 1 OF 4");
        draft_of(&ctx, "SHEET 1 OF 4");

        // Press well to the left of the box, then drag into it.
        frame(&ctx, at(10.0, true));
        publish_layout(&ctx, "SHEET 1 OF 4");
        let mut input = egui::RawInput::default();
        input.events.push(egui::Event::PointerMoved(egui::pos2(
            slot_x(&galley, 5),
            108.0,
        )));
        frame(&ctx, input);

        let after = read(&ctx).expect("the draft survives");
        assert_eq!(
            caret::range(after.mark, after.caret),
            None,
            "a gesture that began off the box must not select inside it"
        );
    }

    /// **The oracle for *"it doesn't type anything in the box when I type"*.**
    #[test]
    fn a_real_text_event_lands_in_the_draft() {
        let ctx = egui::Context::default();
        store(
            &ctx,
            Draft {
                page: 0,
                kind: TextEditKind::Add,
                anchor: Anchor::Origin { x: 10.0, y: 10.0 },
                text: String::new(),
                caret: 0,
                mark: None,
                seeded: true,
            },
        );
        let mut input = egui::RawInput::default();
        input.events.push(egui::Event::Text("h".to_owned()));
        let mut actions = Vec::new();
        let inner = ctx.clone();
        // A real document, because `typing` now takes one: Up and Down ask
        // the PAGE where the next line is (see `blocks`). This test's own event
        // is a `Text`, which never reaches that path — the document is here to
        // satisfy the signature, and passing a real one rather than inventing a
        // stub is what keeps the test honest if the typing path ever grows a
        // second document read.
        let doc = crate::app::state::open_fixture(crate::app::state::FOUR_PAGES);
        let _ = ctx.run_ui(input, move |c| {
            egui::CentralPanel::default().show(c, |ui| {
                typing(ui, &inner, &doc, true, &mut actions);
            });
        });
        assert_eq!(read(&ctx).map(|d| d.text), Some("h".to_owned()));
        assert_ne!(
            TextEditKind::Edit.command_id(),
            TextEditKind::Add.command_id()
        );
    }

    /// **Shift+Right selects, and a plain Right drops it** — the two halves
    /// of the selection, driven through the same event loop the keyboard uses.
    #[test]
    fn shift_right_selects_and_a_plain_right_drops_it() {
        let ctx = egui::Context::default();
        store(
            &ctx,
            Draft {
                page: 0,
                kind: TextEditKind::Add,
                anchor: Anchor::Origin { x: 10.0, y: 10.0 },
                text: "abcdef".to_owned(),
                caret: 0,
                mark: None,
                seeded: true,
            },
        );
        let doc = crate::app::state::open_fixture(crate::app::state::FOUR_PAGES);

        let press = |ctx: &egui::Context, doc: &crate::app::state::OpenDoc, shift: bool| {
            // ui-text-exempt: nothing below is displayed; this is a driver.
            let mut input = egui::RawInput::default();
            let modifiers = egui::Modifiers {
                shift,
                ..Default::default()
            };
            input.modifiers = modifiers;
            input.events.push(egui::Event::Key {
                key: egui::Key::ArrowRight,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers,
            });
            let mut actions = Vec::new();
            let inner = ctx.clone();
            let _ = ctx.run_ui(input, move |c| {
                egui::CentralPanel::default().show(c, |ui| {
                    typing(ui, &inner, doc, true, &mut actions);
                });
            });
        };

        press(&ctx, &doc, true);
        press(&ctx, &doc, true);
        let after = read(&ctx).expect("the draft survives a movement");
        assert_eq!(after.caret, 2);
        assert_eq!(
            caret::range(after.mark, after.caret),
            Some((0, 2)),
            "two shifted presses select two characters, from where the caret started"
        );

        press(&ctx, &doc, false);
        let after = read(&ctx).expect("the draft survives a movement");
        assert_eq!(after.caret, 3);
        assert_eq!(
            caret::range(after.mark, after.caret),
            None,
            "an unshifted move drops the selection - rule 4"
        );
    }

    /// **Enter means a NEW LINE, and it means it everywhere it can.**
    #[test]
    fn enter_makes_a_new_line_in_both_authoring_drafts() {
        for anchor in [
            Anchor::Origin { x: 10.0, y: 20.0 },
            Anchor::Box {
                llx: 0.0,
                lly: 0.0,
                urx: 100.0,
                ury: 50.0,
            },
            Anchor::Block {
                block: 0,
                run: 3,
                llx: 0.0,
                lly: 0.0,
                urx: 100.0,
                ury: 50.0,
                original: "a paragraph".to_owned(), // ui-text-exempt: test fixture text
            },
        ] {
            assert_eq!(
                enter_means(&anchor, false),
                EnterMeans::NewLine,
                "{anchor:?} can hold a line break, so Enter must break the line"
            );
        }
    }

    /// **Ctrl+Enter commits every draft there is.**
    #[test]
    fn control_enter_commits_whatever_the_draft_is() {
        for anchor in [
            Anchor::Origin { x: 10.0, y: 20.0 },
            Anchor::Box {
                llx: 0.0,
                lly: 0.0,
                urx: 100.0,
                ury: 50.0,
            },
            Anchor::Run {
                run: 3,
                original: "TITLE".to_owned(),
            },
        ] {
            assert_eq!(
                enter_means(&anchor, true),
                EnterMeans::Commit,
                "Ctrl+Enter must finish {anchor:?} — the chord is the keyboard route out"
            );
        }
    }

    /// **Enter in text already on the page DECLINES rather than
    /// committing.**
    #[test]
    fn enter_in_an_existing_run_declines_instead_of_committing() {
        let anchor = Anchor::Run {
            run: 3,
            original: "TITLE".to_owned(),
        };
        assert_eq!(
            enter_means(&anchor, false),
            EnterMeans::CannotSplit,
            "a show operator cannot hold a line break, and the operator is owed the sentence saying so rather than an edit finishing under them"
        );
        assert_ne!(
            enter_means(&anchor, false),
            EnterMeans::Commit,
            "committing here is what made the answer invisible — it looks like success"
        );
    }

    /// **The three outcomes are distinct, and the modifier is what separates
    /// the two that share an anchor.**
    #[test]
    fn the_modifier_is_the_only_thing_that_changes_a_run_draft() {
        let anchor = Anchor::Run {
            run: 0,
            original: String::new(),
        };
        assert_ne!(enter_means(&anchor, false), enter_means(&anchor, true));
        let boxed = Anchor::Box {
            llx: 0.0,
            lly: 0.0,
            urx: 10.0,
            ury: 10.0,
        };
        assert_ne!(enter_means(&boxed, false), enter_means(&boxed, true));
    }
}
