//! # `canvas::textedit::keys` — what every key means inside a draft
//!
//! ## What this is
//!
//! One function, [`typing`], and the rules it enforces. It is the whole of the
//! keyboard's contract with a text draft: which keys insert, which move, which
//! commit, which abandon, and — since 2026-08-21 — which select.
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
//! ## ★★ The four selection rules, and where each is enforced
//!
//! | # | rule | enforced by |
//! |---|---|---|
//! | 1 | a selection is the range between the mark and the caret | [`caret::range`] |
//! | 2 | typing replaces it | the `Text` arm, via [`take_selection`] |
//! | 3 | Backspace and Delete remove it and nothing else | their two arms |
//! | 4 | any movement without Shift drops it | [`caret::moved`], called by every movement arm |
//!
//! Rule 4 is the one that looks like a detail and is not: without it a
//! highlight stays on screen after the caret has walked out of it, and the next
//! keystroke deletes text the operator is no longer looking at.
//!
//! ## ★ What is NOT here, named rather than left to be discovered
//!
//! **Drag-select and double-click-to-select-a-word.** The draft is drawn in an
//! editor box in *screen* space by [`super::paint`], and hit-testing a pointer
//! into it needs that laid-out galley published where the click ladder can
//! reach it. Real work, not a line — and until it exists, a selection is made
//! with the keyboard only.

use egui::Ui;

use super::caret::{self, backspace, delete_forward, insert, word_left, word_right};
use super::{Anchor, DIAG_TYPE, Draft, abandon, blocks, commit_into, hit, read, store};
use crate::app::state::OpenDoc;

/// **What the pointer does inside the editor box** — place the caret, sweep a
/// selection, or take a word on a double click.
///
/// Returns `true` when the draft changed and must be written back.
///
/// # ★★★ The three gestures, and why they are one function
///
/// | gesture | result |
/// |---|---|
/// | press | the caret goes where the pointer is, and any selection is dropped |
/// | drag | the mark stays at the PRESS and the caret follows the pointer |
/// | double click | the word under the pointer is selected |
///
/// They share a hit test and they share the rule that all three are only
/// meaningful **inside** the box, so splitting them would be three copies of
/// the containment check and three chances to drift on which coordinate space
/// is being asked about.
///
/// # ★★ The press origin, not the current position, anchors the drag
///
/// `PointerState::press_origin` is where the button went down, so the mark is
/// recomputed from it every frame rather than stored. That is deliberate: a
/// stored mark would have to be cleared on every way a drag can end — released,
/// Escaped, interrupted by focus loss, interrupted by the space bar — and
/// `canvas::markup::ink`'s header records what forgetting one of those four
/// costs. Derived state cannot go stale.
///
/// # ★ Why this reads raw pointer input rather than a `Response`
///
/// Because the draft is not a widget. It is painted into the canvas and the
/// canvas's own `Response` covers the whole page; a second `interact` over the
/// editor box would put an invisible widget in the canvas's hit-test order and
/// change what the page under it receives. The box is a rectangle this module
/// drew and this module knows where it is — asking egui to tell it back would
/// be a second derivation of a fact it already has.
fn pointer(ui: &Ui, ctx: &egui::Context, draft: &mut Draft) -> bool {
    let Some(layout) = hit::read(ctx) else {
        return false;
    };
    let (pressed, down, double, pos, origin) = ui.input(|i| {
        (
            i.pointer.primary_pressed(),
            i.pointer.primary_down(),
            i.pointer
                .button_double_clicked(egui::PointerButton::Primary),
            i.pointer.interact_pos(),
            i.pointer.press_origin(),
        )
    });
    let Some(pos) = pos else {
        return false;
    };
    // ★ The PRESS decides whether this gesture belongs to the box, not the
    // current position. A sweep that starts inside and runs out over the page
    // keeps selecting to the end of the text, which is what every text field
    // does; a press that starts outside never becomes the draft's business no
    // matter where it is dragged to.
    let began_inside = origin.is_some_and(|o| layout.body.contains(o));

    if double && began_inside {
        // The word under the pointer. `word_left`/`word_right` are the same
        // two functions `Ctrl+Left`/`Ctrl+Right` use, so a double click and a
        // chord agree about where a word ends.
        let at = layout.index_at(pos);
        let from = word_left(&draft.text, (at + 1).min(draft.text.chars().count()));
        let to = word_right(&draft.text, at);
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
        let to = layout.index_at(pos);
        if from != to {
            draft.mark = Some(from);
            draft.caret = to;
            return true;
        }
    }
    false
}

/// **Remove whatever is selected**, and answer the caret.
///
/// Answers `draft.caret` unchanged when nothing is selected, so a caller may
/// call it unconditionally — which the `Text` arm does, because *"replace the
/// selection if there is one"* and *"insert here"* is one act.
///
/// ★ It clears the mark, and that is not tidying: a mark left pointing into
/// text that no longer exists is an index past the end of the string, and the
/// next Shift+Left would select a range that is not there.
fn take_selection(draft: &mut Draft) -> usize {
    let Some((from, to)) = caret::range(draft.mark, draft.caret) else {
        return draft.caret;
    };
    draft.mark = None;
    caret::delete_range(&mut draft.text, from, to)
}

/// **Consume this frame's keystrokes into the draft.**
///
/// Returns `true` when the draft was committed by Enter, so the caller knows the
/// caret is gone.
///
/// # Why the events are read raw rather than through a `TextEdit` widget
///
/// Because the caret is painted in PDF space, on the page, at the glyphs' own
/// scale — which is what *"just edit the existing box"* means. An `egui`
/// `TextEdit` would be a second box floating over the first, and the old shell's
/// one virtue here is worth keeping: it had a real caret in the page, and no
/// widget in the typing path.
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
    // ★★ THE POINTER FIRST, and before the seam, because a press that lands
    // in the box is the operator saying *where* the next keystroke goes — and
    // a keystroke arriving in the same frame must land at the new caret rather
    // than the old one.
    let mut changed = pointer(ui, ctx, &mut draft);
    // The diagnostic seam, consumed exactly once per draft. See [`DIAG_TYPE`].
    if !draft.seeded {
        draft.seeded = true;
        changed = true;
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
    }
    if focused {
        // ★★ Read once, outside the loop: see [`caret::shifted`] for why the
        // frame's own modifier state is consulted at all, and why ignoring it
        // cost this shell its whole first driven run of Shift+arrow.
        let frame_shift = ui.input(|i| i.modifiers.shift);
        for ev in ui.input(|i| i.events.clone()) {
            match ev {
                // ★★ TYPING REPLACES THE SELECTION. Rule 2 of the four in
                // `caret`'s selection section, and the one an operator notices
                // first: select a word, type a word, and the old one is gone.
                egui::Event::Text(t) if !t.is_empty() => {
                    draft.caret = take_selection(&mut draft);
                    draft.caret = insert(&mut draft.text, draft.caret, &t);
                    changed = true;
                }
                // ★ Rule 3: with a selection, Backspace and Delete remove
                // THAT and nothing else — they stop being different keys, which
                // is what every text field does and is why both arms are the
                // same two lines.
                egui::Event::Key {
                    key: egui::Key::Backspace,
                    pressed: true,
                    ..
                } => {
                    draft.caret = if caret::range(draft.mark, draft.caret).is_some() {
                        take_selection(&mut draft)
                    } else {
                        backspace(&mut draft.text, draft.caret)
                    };
                    changed = true;
                }
                egui::Event::Key {
                    key: egui::Key::Delete,
                    pressed: true,
                    ..
                } => {
                    draft.caret = if caret::range(draft.mark, draft.caret).is_some() {
                        take_selection(&mut draft)
                    } else {
                        delete_forward(&mut draft.text, draft.caret)
                    };
                    changed = true;
                }
                // ★★ SELECT ALL. `Ctrl+A` is not in the keymap and must not be:
                // the application's own Select-all acts on OBJECTS, and while a
                // draft is live the operator means the text they are typing.
                // The draft takes the chord first and the event is consumed, so
                // the two never both fire.
                egui::Event::Key {
                    key: egui::Key::A,
                    pressed: true,
                    modifiers,
                    ..
                } if modifiers.command => {
                    draft.mark = Some(0);
                    draft.caret = draft.text.chars().count();
                    changed = true;
                }
                // ★★★ THE DRAFT'S CLIPBOARD — copy, cut and paste. Defect O18.
                //
                // All three were absent until 2026-08-21, and the absence was
                // not an oversight so much as a half-finished thought.
                // `textsel::clipboard::pending_key` was widened that same week
                // to STOP answering Ctrl+C while a draft is composing, with a
                // correct argument: *"the operator is composing, and the
                // selection they made before the caret landed is not what those
                // two keys mean any more"*. True — and it left the chord with no
                // owner at all, so it fell through to the ribbon keymap, reached
                // `edit.copy`, and copied an OBJECT. The operator pasted into
                // Notepad and got *"1 object copied from pdfcer"*.
                //
                // The lesson is the general one: taking a chord away from a
                // handler is only half a decision. The other half is naming who
                // gets it, and a chord with no owner does not go quiet — it goes
                // to whoever claims it next.
                //
                // ★ These arrive as `Event::Copy` / `Event::Cut` /
                // `Event::Paste`, never as key events: `egui-winit` intercepts
                // the three chords and returns before pushing an `Event::Key`.
                // Matching on `Key::C` here would compile, read correctly, pass
                // a unit test that injected a key event, and never fire once in
                // the running application. That is exactly how O18 shipped.
                egui::Event::Copy => {
                    // Copy leaves the draft alone — `changed` stays as it was.
                    // A copy is not an edit, so it must not mark the draft dirty
                    // and must not cost an undo entry.
                    copy_selection(ctx, &draft);
                }
                egui::Event::Cut => {
                    // ★ Cut is copy-then-delete, in that order, and it is a
                    // no-op with no selection rather than a cut of the whole
                    // draft. Some editors cut the current line when nothing is
                    // selected; a text box on a drawing is not a code editor,
                    // and silently removing everything the operator had typed on
                    // a stray Ctrl+X is not a behaviour worth borrowing.
                    if copy_selection(ctx, &draft) {
                        draft.caret = take_selection(&mut draft);
                        changed = true;
                    }
                }
                egui::Event::Paste(pasted) if !pasted.is_empty() => {
                    // ★ Replaces the selection, exactly as typing does — rule 2
                    // of the four in `caret`'s selection section. Reusing
                    // `take_selection` rather than repeating its two lines is
                    // what keeps paste and typing from drifting apart on a rule
                    // the operator experiences as one behaviour.
                    //
                    // `caret::insert` filters control characters, so a multi-
                    // line paste arrives as one line. That is a real limitation
                    // and it is the RIGHT one until the draft is multi-line
                    // (O15): inserting a newline the draft cannot represent
                    // would either be dropped silently later or committed as a
                    // literal control byte into a content stream.
                    draft.caret = take_selection(&mut draft);
                    draft.caret = insert(&mut draft.text, draft.caret, &pasted);
                    changed = true;
                }
                // ★★ **Caret movement**, 2026-08-20, on the operator's report
                // that *"the cursor just sits at the end of a text line. It
                // can't be moved to the center of an existing text block."*
                //
                // These five arms are what makes the caret a caret. Before
                // them the draft had no position at all: text was appended and
                // Backspace popped, so changing `SHEET 1 OF 4` to `SHEET 2 OF
                // 4` meant deleting back to `SHEET ` and retyping the rest.
                //
                // ★ `changed` is set for a pure movement, and that is
                // deliberate rather than sloppy. It is the flag that decides
                // whether the draft is written back to `egui::Memory`, and a
                // moved caret IS a changed draft - without this the arrow keys
                // would appear to work for one frame and then snap back on the
                // next load. It does NOT put anything on the undo stack:
                // `commit_into` compares the TEXT with the original, so a draft
                // whose caret moved and whose characters did not still pushes
                // no action.
                egui::Event::Key {
                    key: egui::Key::ArrowLeft,
                    pressed: true,
                    modifiers,
                    ..
                } => {
                    // ★ Rule 4, applied by one function so every movement arm
                    // agrees: Shift plants or keeps the mark, no Shift drops it.
                    draft.mark = caret::moved(
                        draft.mark,
                        draft.caret,
                        caret::shifted(modifiers.shift, frame_shift),
                    );
                    draft.caret = if modifiers.command {
                        word_left(&draft.text, draft.caret)
                    } else {
                        draft.caret.saturating_sub(1)
                    };
                    changed = true;
                }
                egui::Event::Key {
                    key: egui::Key::ArrowRight,
                    pressed: true,
                    modifiers,
                    ..
                } => {
                    draft.mark = caret::moved(
                        draft.mark,
                        draft.caret,
                        caret::shifted(modifiers.shift, frame_shift),
                    );
                    let end = draft.text.chars().count();
                    draft.caret = if modifiers.command {
                        word_right(&draft.text, draft.caret)
                    } else {
                        (draft.caret + 1).min(end)
                    };
                    changed = true;
                }
                // ★★★ UP AND DOWN WALK THE PAGE'S OWN LINES, AND CROSS INTO
                // THE NEXT PARAGRAPH.
                //
                // The operator, 2026-08-21: *"there was an acrobat feature in
                // the original pdfcer-gui that attempted to reassemble
                // individual lines into paragraphs and the cursor would move to
                // the next block of text using the navigation keys."*
                //
                // **Salvage.** `canvas::textedit::blocks` carries the four
                // lines it came from and the argument; the short form is that
                // the reassembly is `pdfcer-core`'s — `caret_up` walks the
                // model's *lines*, and a block is a group of lines, so the
                // caret steps into the next paragraph without anything here
                // knowing what a paragraph is. The old shell's whole
                // contribution was **asking**, and this shell had not been.
                //
                // ★ It was not bound at all before today, and that was right at
                // the time: the caret is a character index into ONE run, and a
                // single run has no line above it. What changed is not the
                // draft — it is that the *page* is now the thing being
                // navigated.
                //
                // ★★ THE DRAFT IS COMMITTED ON THE WAY OUT. A caret that left
                // a run with unsaved keystrokes in it would silently discard
                // them, which is the defect class this whole module exists
                // against — and `commit_into` writes nothing when the text is
                // unchanged, so an operator who is merely reading with the
                // arrow keys puts nothing on the undo stack.
                //
                // ★ A BOX draft is deliberately excluded. Its lines are the
                // shell's wrap rather than the page's, so this model would move
                // the caret to a run somewhere else on the sheet mid-paragraph.
                // Named in `blocks`' header rather than left to be discovered.
                egui::Event::Key {
                    key: key @ (egui::Key::ArrowUp | egui::Key::ArrowDown),
                    pressed: true,
                    ..
                } => {
                    let dir = if key == egui::Key::ArrowUp {
                        blocks::Vertical::Up
                    } else {
                        blocks::Vertical::Down
                    };
                    if blocks::step(ctx, doc, &draft, dir, actions) {
                        return true;
                    }
                }
                // ★★ HOME AND END REACH THE ENDS OF THE LINE THE OPERATOR CAN
                // SEE, which on a CAD sheet is usually several show operators
                // wide. `blocks::line` answers `false` when the line is this
                // run — the common case, and the cheap one — and the two
                // assignments below are what happens then.
                egui::Event::Key {
                    key: egui::Key::Home,
                    pressed: true,
                    modifiers,
                    ..
                } => {
                    // ★ Shift+Home selects to the start of the draft and stays
                    // in it, rather than walking to another run: a selection
                    // that spanned two show operators would be a selection this
                    // shell cannot commit, and offering it would be a gesture
                    // whose result is a refusal.
                    let shift = caret::shifted(modifiers.shift, frame_shift);
                    if !shift && blocks::line(ctx, doc, &draft, false, actions) {
                        return true;
                    }
                    draft.mark = caret::moved(draft.mark, draft.caret, shift);
                    draft.caret = 0;
                    changed = true;
                }
                egui::Event::Key {
                    key: egui::Key::End,
                    pressed: true,
                    modifiers,
                    ..
                } => {
                    let shift = caret::shifted(modifiers.shift, frame_shift);
                    if !shift && blocks::line(ctx, doc, &draft, true, actions) {
                        return true;
                    }
                    draft.mark = caret::moved(draft.mark, draft.caret, shift);
                    draft.caret = draft.text.chars().count();
                    changed = true;
                }
                // ★★★ ENTER MEANS TWO THINGS, AND THE ANCHOR DECIDES WHICH.
                //
                // The operator, 2026-08-21: *"I should be able to make it multi
                // line."*
                //
                // | anchor | plain Enter | Ctrl+Enter |
                // |---|---|---|
                // | a **box** | a paragraph break | commit |
                // | a point, or an existing run | commit | commit |
                //
                // ★ This is the old shell's own split, carried across verbatim:
                // *"in box mode a plain Enter is a paragraph break; Ctrl+Enter
                // accepts. In point mode Enter accepts (single line)."* It is
                // also what every program in the class does, which is the
                // standing tie-breaker.
                //
                // ★★ And it is why `Anchor::Box` is a variant rather than an
                // `Option<Rect>` on `Origin`. Enter cannot mean *insert* and
                // *commit* in one draft, so the keystroke handler has to know
                // which gesture started it — and asking the TEXT ("does it
                // already contain a newline?") would make the first Enter
                // commit and every one after it insert, which is the worst
                // possible answer.
                //
                // ★ A newline in an EXISTING run is refused by construction
                // rather than by a check: `Anchor::Run` is not a box, so plain
                // Enter commits there. That is correct and not a limitation
                // being hidden — `edit_text` replaces the text of ONE show
                // operator, and a show operator cannot contain a line break. A
                // run that should become two lines is a *reflow*, which is a
                // different verb with its own preconditions.
                egui::Event::Key {
                    key: egui::Key::Enter,
                    pressed: true,
                    modifiers,
                    ..
                } => {
                    crate::diag::trace(|| {
                        // ui-text-exempt: diagnostic trace, never displayed.
                        //
                        // ★ Enter is the one keystroke in this handler with TWO
                        // meanings, so its ARRIVAL is worth reporting separately
                        // from its effect. The multi-line work spent a driven
                        // run on *"did the key arrive, or did the branch pick
                        // wrong?"*, which the `text-edit-typing` line cannot
                        // answer: it reports a length, and both failures leave
                        // the length unchanged.
                        format!(
                            "text-edit-enter boxed={} command={}",
                            u8::from(matches!(draft.anchor, Anchor::Box { .. })),
                            u8::from(modifiers.command),
                        )
                    });
                    if matches!(draft.anchor, Anchor::Box { .. }) && !modifiers.command {
                        // ★ `newline`, NOT `insert` — see its docs. `insert`
                        // drops control characters, correctly, and ate this
                        // exact keystroke for one driven run.
                        draft.caret = caret::newline(&mut draft.text, draft.caret);
                        changed = true;
                    } else {
                        commit_into(ctx, &draft, actions);
                        abandon(ctx);
                        return true;
                    }
                }
                _ => {}
            }
        }
    }
    if changed {
        // The selection, published for the harness.
        //
        // ★ A TRACE RATHER THAN A MUTATION, and that is the point of it. The
        // honest way to prove Shift+Right selected three characters is to type
        // over them and see the text shrink — but a driven check runs on the
        // operator's own drawing, and proving a *selection* by making an
        // *edit* is a bad trade. This line carries the two numbers a wrong
        // build would get wrong, so nothing has to be changed to read them.
        //
        // ★★ It reports the EMPTY case too, in its own words rather than by
        // going quiet. Rule 4 — an unshifted move drops the selection — is
        // exactly as important as the selecting, and an absent line cannot be
        // told from a build where the trace stopped being emitted.
        crate::diag::trace_on_change("text-select", || {
            // ui-text-exempt: diagnostic trace, never displayed.
            match caret::range(draft.mark, draft.caret) {
                // ★ The KEY IS NOT REPEATED in the value. `trace_on_change`
                // prints `pdfcer-diag <key> <value>`, so a value beginning with
                // the key produces `text-select text-select from=0 …` — which
                // parses, reads as a typo, and was one until this line.
                Some((from, to)) => {
                    let n = to - from;
                    // ui-text-exempt: diagnostic trace, never displayed.
                    format!("from={from} to={to} n={n}")
                }
                // ui-text-exempt: diagnostic trace, never displayed.
                None => format!("none caret={}", draft.caret),
            }
        });
        store(ctx, draft);
    }
    false
}

/// **Put the draft's selected text on the clipboard**, reporting whether there
/// was any.
///
/// The return value is what makes [`Event::Cut`](egui::Event::Cut) safe: cut is
/// copy-then-delete, and it must not delete when the copy found nothing to take.
/// Returning a `bool` rather than having the caller re-ask `caret::range` means
/// the two halves of a cut cannot disagree about whether a selection existed.
///
/// Routed through [`crate::canvas::textsel::clipboard::copy`] — **the** one
/// place this shell writes text to the clipboard — rather than calling
/// `egui::Context::copy_text` directly. That function's header carries why:
/// three verbs reach it, and routing all of them through one function is what
/// makes its trace line a complete record of what pdfcer has copied rather than
/// one of several partial ones. It also refuses an empty string there, which is
/// the guard that stops a copy silently destroying whatever the operator had on
/// their clipboard from another application.
fn copy_selection(ctx: &egui::Context, draft: &Draft) -> bool {
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
    ///
    /// ★ A REAL GALLEY, from the same font stack the shell draws with, because
    /// the whole claim of `hit` is that the layout which is hit-tested is the
    /// layout which was drawn. A stub that mapped x to an index would test the
    /// arithmetic of a stub.
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
                origin,
                galley: galley.clone(),
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

    /// ★★★ **CTRL+C IN A TEXT BOX COPIES THE SELECTED TEXT** — defect O18, the
    /// operator's report of 2026-08-21.
    ///
    /// The event injected is `Event::Copy`, which is what `egui-winit` actually
    /// sends, and injecting anything else is how this defect shipped: a test
    /// feeding `Event::Key { key: C }` certifies a path the running application
    /// can never take.
    ///
    /// ★ It asserts the draft is UNCHANGED as well. A copy that quietly moved
    /// the caret or dropped the selection would pass a "did it copy" check and
    /// still be wrong — the operator's next Shift+Left would select the wrong
    /// range.
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

    /// ★★ **CTRL+X removes what it copied**, and copy runs first.
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

    /// ★★★ **A CUT WITH NO SELECTION MUST DESTROY NOTHING.**
    ///
    /// Some editors cut the whole current line when nothing is selected. A text
    /// box on a drawing is not a code editor, and a stray Ctrl+X silently
    /// removing everything the operator had typed is not a behaviour worth
    /// borrowing — they would have to notice it to undo it.
    #[test]
    fn ctrl_x_with_no_selection_destroys_nothing() {
        let ctx = egui::Context::default();
        draft_of(&ctx, "SHEET 1 OF 4");
        clipboard_frame(&ctx, egui::Event::Cut);

        let after = read(&ctx).expect("the draft survives");
        assert_eq!(after.text, "SHEET 1 OF 4", "a cut with nothing selected");
    }

    /// ★★ **CTRL+V pastes at the caret**, and replaces a selection if there is
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

    /// ★ **A multi-line paste arrives as one line**, because the draft is
    /// single-line. Named as a test rather than left to be discovered: the
    /// filtering is `caret::insert`'s and it is deliberate — a newline the draft
    /// cannot represent would otherwise be dropped later or committed as a
    /// literal control byte into a content stream.
    #[test]
    fn a_multi_line_paste_arrives_as_one_line() {
        let ctx = egui::Context::default();
        draft_of(&ctx, "");
        clipboard_frame(&ctx, egui::Event::Paste("one\ntwo".to_owned()));

        let after = read(&ctx).expect("the draft survives");
        assert_eq!(after.text, "onetwo");
    }

    /// ★★★ **A DRAG ACROSS THE TEXT SELECTS WHAT IT CROSSED** — the pointer
    /// half of `OPERATOR_REQUESTS.md` O14 item 11.
    ///
    /// Driven through the same function the keyboard goes through, with a real
    /// galley published the way `paint` publishes one, so the hit test is the
    /// inverse of the caret painter rather than a second guess at it.
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

    /// ★★ **A press with no travel places the caret and clears any selection**,
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

    /// ★ **A press that begins OUTSIDE the box is not the draft's business**,
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
    ///
    /// Every existing text-edit check seeds the draft through `PDFCER_DIAG_TYPE`,
    /// which is the ONE path that bypasses the event loop — so all of them pass
    /// on a build where real typing is dead. This one drives a real
    /// `egui::Context` with a real `Event::Text` and asserts the draft grew.
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
        // ★ A real document, because `typing` now takes one: Up and Down ask
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

    /// ★★ **Shift+Right selects, and a plain Right drops it** — the two halves
    /// of the selection, driven through the same event loop the keyboard uses.
    ///
    /// A unit test rather than only a driven one, because the driven check
    /// cannot tell "the shell ignored Shift" from "the harness never sent it",
    /// and those live in different repositories. This one is unambiguous: the
    /// event carries `shift: true` by construction.
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
}
