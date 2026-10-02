//! # `canvas::textedit::edits` — one keystroke's effect on the open draft
//!
//! [`Keys::event`] takes one `egui::Event` and changes the draft, pushes the
//! action it finishes in, or ignores it. It answers [`Flow::Done`] when the
//! draft has been committed or handed on, so the caller must not store it
//! again. Every change to the text records the draft's state first
//! ([`super::history`]), so `Ctrl+Z` takes back keystrokes before it reaches
//! the document.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/textedit/keys.md`.

use egui::{Event, Key, Modifiers};
use pdfcer_gui_base::editmodel::history::{EditKind, tab_spaces};
use pdfcer_gui_base::text::draftnote::DraftNote;

use super::caret::{self, backspace, delete_forward, insert, word_left, word_right};
use super::history::{self, Step};
use super::keys::{EnterMeans, copy_selection, enter_means, take_selection};
use super::{Anchor, Draft, abandon, blocks, commit_into, note};
use crate::app::actions::Action;
use crate::app::state::OpenDoc;

/// Tab stops are every half inch, in PDF points.
const TAB_STOP_PT: f32 = 36.0;

/// What [`claim_tab`] turns a Tab press into.
const TAB: &str = "\t";

/// **Take Tab away from egui's focus walk while a draft is open.** A bare Tab
/// press becomes a typed [`TAB`], which [`Keys::event`] turns into spaces;
/// Shift+Tab and every release are dropped. Nothing happens unless a draft is
/// open on the root viewport with no text field focused.
pub fn claim_tab(ctx: &egui::Context, input: &mut egui::RawInput) {
    if input.viewport_id != egui::ViewportId::ROOT
        // typing-guard-exempt: this asks whether an egui text field holds the
        // keyboard, which keeps its own Tab; the draft is asked on the next line.
        || ctx.text_edit_focused()
        || super::read(ctx).is_none()
    {
        return;
    }
    input.events = std::mem::take(&mut input.events)
        .into_iter()
        .filter_map(|ev| match ev {
            Event::Key {
                key: Key::Tab,
                pressed,
                modifiers,
                ..
            } if !modifiers.command && !modifiers.alt => {
                (pressed && !modifiers.shift).then(|| Event::Text(TAB.to_owned()))
            }
            other => Some(other),
        })
        .collect();
}

/// Whether the draft is still the caller's to store.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flow {
    Continue,
    Done,
}

/// One frame's keystrokes against the open draft.
pub struct Keys<'a> {
    pub ctx: &'a egui::Context,
    pub doc: &'a OpenDoc,
    pub draft: &'a mut Draft,
    pub actions: &'a mut Vec<Action>,
    /// The frame's own Shift state; see [`caret::shifted`].
    pub frame_shift: bool,
    /// Whether the draft changed and must be stored.
    pub changed: bool,
}

impl Keys<'_> {
    /// Apply `ev` to the draft.
    pub fn event(&mut self, ev: Event) -> Flow {
        match ev {
            Event::Text(t) if t == TAB => self.tab(),
            Event::Text(t) if !t.is_empty() => self.type_text(&t, EditKind::Typing),
            // `Event::Copy`/`Cut`/`Paste`, never `Key::C`/`X`/`V`: egui-winit
            // turns the three chords into these and pushes no key event.
            Event::Copy => {
                copy_selection(self.ctx, self.draft);
            }
            Event::Cut => {
                if copy_selection(self.ctx, self.draft) {
                    self.record(EditKind::Other);
                    self.draft.caret = take_selection(self.draft);
                    self.changed = true;
                }
            }
            Event::Paste(p) if !p.is_empty() => self.paste(&p),
            Event::Key {
                key,
                pressed: true,
                modifiers,
                ..
            } => return self.key(key, modifiers),
            _ => {}
        }
        Flow::Continue
    }

    fn key(&mut self, key: Key, m: Modifiers) -> Flow {
        match key {
            Key::Backspace | Key::Delete => self.remove(key == Key::Delete, m.command),
            Key::A if m.command => {
                self.draft.mark = Some(0);
                self.draft.caret = self.draft.text.chars().count();
                self.moved();
            }
            Key::Z if m.command && m.shift => return self.step(Step::Redo),
            Key::Z if m.command => return self.step(Step::Undo),
            Key::Y if m.command => return self.step(Step::Redo),
            Key::S if m.command => return self.save(),
            // The global chords yield while a draft is open, so the draft
            // routes the three it shares with the ribbon itself.
            Key::B | Key::I | Key::U if m.command && !m.alt && !m.shift => {
                self.actions
                    .push(Action::Command(style_command(key).into()));
            }
            Key::Tab if !m.command && !m.alt => self.tab(),
            Key::ArrowLeft | Key::ArrowRight => self.horizontal(key == Key::ArrowRight, m),
            Key::ArrowUp | Key::ArrowDown => return self.vertical(key == Key::ArrowUp, m),
            Key::Home | Key::End => return self.home_end(key == Key::End, m),
            Key::Enter => return self.enter(m),
            _ => {}
        }
        Flow::Continue
    }

    fn record(&self, kind: EditKind) {
        history::record(self.ctx, self.draft, kind);
        note::forget(self.ctx);
    }

    /// A caret movement: the draft is stored, and the next edit is its own
    /// undo entry.
    fn moved(&mut self) {
        history::break_run(self.ctx);
        self.changed = true;
    }

    /// Insert `t` at the caret, replacing the selection. On an existing run
    /// the run's font sieves it first; keys it lacks are named by the
    /// refused-keys notice, and a keystroke refused whole leaves the selection
    /// standing.
    fn type_text(&mut self, t: &str, kind: EditKind) {
        let draft = &*self.draft;
        let (kept, refused) = match &draft.anchor {
            Anchor::Run { run, .. } => {
                let s = super::repertoire::sieve(self.ctx, self.doc, draft.page, *run, t);
                (s.kept, s.refused)
            }
            Anchor::Origin { .. } | Anchor::Box { .. } => (t.to_owned(), None),
        };
        if let (Anchor::Run { run, .. }, Some((missing, base_font))) = (&draft.anchor, &refused) {
            if let Some(face) = super::reface::plan(self.ctx, self.doc, draft, missing, base_font) {
                self.record(kind);
                self.draft.caret = take_selection(self.draft);
                self.draft.caret = insert_lines(&mut self.draft.text, self.draft.caret, t);
                self.changed = true;
                super::refused::planned(self.ctx, self.draft, missing, base_font, &face);
                return;
            }
            self.key_refused(*run, missing[0], base_font.clone());
        }
        if !kept.is_empty() {
            self.record(kind);
            self.draft.caret = take_selection(self.draft);
            self.draft.caret = insert_lines(&mut self.draft.text, self.draft.caret, &kept);
            self.changed = true;
        }
        if let Some((missing, base_font)) = &refused {
            super::refused::note(self.ctx, self.draft, missing, base_font, !kept.is_empty());
        }
    }

    fn key_refused(&mut self, run: usize, character: char, base_font: String) {
        let page = self.draft.page;
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "text-edit-key-refused page={page} run={run} character='{character}' \
                 character_font={base_font}"
            )
        });
        self.actions.push(Action::Text(
            crate::app::actions::text::TextAction::KeyRefused {
                page,
                run,
                character,
                base_font,
            },
        ));
    }

    /// Paste through the same sieve as typing. A line already on the page
    /// holds no line break, so pasted lines join with spaces and say so; new
    /// text keeps them.
    fn paste(&mut self, pasted: &str) {
        let text = pdfcer_gui_base::clippaste::textbox::normalise(pasted);
        let joined = matches!(self.draft.anchor, Anchor::Run { .. }) && text.contains('\n');
        let text = if joined {
            text.replace('\n', " ") // ui-text-exempt: a typed character, not prose
        } else {
            text
        };
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "text-edit-paste chars={} joined={}",
                text.chars().count(),
                u8::from(joined)
            )
        });
        self.type_text(&text, EditKind::Other);
        if joined {
            note::raise(self.ctx, DraftNote::LinesJoined);
        }
    }

    /// Backspace or Delete; with `word`, to the word boundary. A selection is
    /// removed instead, by either key.
    fn remove(&mut self, forward: bool, word: bool) {
        self.record(EditKind::Deleting);
        let d = &mut *self.draft;
        d.caret = if caret::range(d.mark, d.caret).is_some() {
            take_selection(d)
        } else if word {
            let other = if forward {
                word_right(&d.text, d.caret)
            } else {
                word_left(&d.text, d.caret)
            };
            caret::delete_range(&mut d.text, d.caret.min(other), d.caret.max(other))
        } else if forward {
            delete_forward(&mut d.text, d.caret)
        } else {
            backspace(&mut d.text, d.caret)
        };
        self.changed = true;
    }

    /// Undo or redo inside the draft; with nothing there to step, the draft is
    /// committed and the document's own history steps instead.
    fn step(&mut self, way: Step) -> Flow {
        let in_draft = history::step(self.ctx, self.draft, way)
            || (way == Step::Redo && history::has_edits(self.ctx));
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "text-edit-history step={} owner={} len={}",
                if way == Step::Undo { "undo" } else { "redo" },
                if in_draft { "draft" } else { "document" },
                self.draft.text.chars().count()
            )
        });
        if in_draft {
            note::forget(self.ctx);
            self.changed = true;
            return Flow::Continue;
        }
        commit_into(self.ctx, self.draft, self.actions);
        abandon(self.ctx);
        self.actions.push(match way {
            Step::Undo => Action::Undo,
            Step::Redo => Action::Redo,
        });
        Flow::Done
    }

    /// Commit the draft, then save: in place when the document has a file,
    /// otherwise as a copy, as the Save command does.
    fn save(&mut self) -> Flow {
        commit_into(self.ctx, self.draft, self.actions);
        abandon(self.ctx);
        let in_place = crate::app::save::has_a_file(self.doc);
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("text-edit-save in_place={}", u8::from(in_place))
        });
        self.actions.push(if in_place {
            Action::Save
        } else {
            Action::SaveCopy
        });
        Flow::Done
    }

    /// Tab: spaces to the next half-inch stop from the start of the draft's
    /// text, measured on the editor box as last drawn.
    fn tab(&mut self) {
        let n = super::hit::read(self.ctx).map_or(1, |l| {
            let pt_per_px = l.body_canvas.width() / l.body.width().max(f32::EPSILON);
            let x = l.x_at(self.draft.caret) - l.x_at(0);
            let space = space_px(&l, &self.draft.text);
            tab_spaces(x * pt_per_px, space * pt_per_px, TAB_STOP_PT)
        });
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("text-edit-tab spaces={n}")
        });
        self.type_text(&" ".repeat(n), EditKind::Other); // ui-text-exempt: typed spaces, not prose
        note::raise(self.ctx, DraftNote::TabAsSpaces(n));
    }

    fn horizontal(&mut self, right: bool, m: Modifiers) {
        let d = &mut *self.draft;
        d.mark = caret::moved(d.mark, d.caret, caret::shifted(m.shift, self.frame_shift));
        d.caret = match (right, m.command) {
            (false, true) => word_left(&d.text, d.caret),
            (false, false) => d.caret.saturating_sub(1),
            (true, true) => word_right(&d.text, d.caret),
            (true, false) => (d.caret + 1).min(d.text.chars().count()),
        };
        self.moved();
    }

    /// Up and Down: the draft's own lines first, then the page's lines, which
    /// commit this draft and open the next ([`blocks::step`]). Shifted past
    /// the draft's first or last line, they extend the selection to its start
    /// or end and the draft stays open.
    fn vertical(&mut self, up: bool, m: Modifiers) -> Flow {
        let shift = caret::shifted(m.shift, self.frame_shift);
        let d = &mut *self.draft;
        let to = if up {
            super::lines::up(&d.text, d.caret)
        } else {
            super::lines::down(&d.text, d.caret)
        };
        let to = to.or_else(|| shift.then(|| if up { 0 } else { d.text.chars().count() }));
        if let Some(to) = to {
            d.mark = caret::moved(d.mark, d.caret, shift);
            d.caret = to;
            self.moved();
            return Flow::Continue;
        }
        let dir = if up {
            blocks::Vertical::Up
        } else {
            blocks::Vertical::Down
        };
        if blocks::step(self.ctx, self.doc, self.draft, dir, self.actions) {
            return Flow::Done;
        }
        Flow::Continue
    }

    /// Home and End: the draft's own line when it has several, else the line
    /// the operator sees, which may be several runs wide ([`blocks::line`]).
    /// Shifted, they stay inside the draft.
    fn home_end(&mut self, end: bool, m: Modifiers) -> Flow {
        let shift = caret::shifted(m.shift, self.frame_shift);
        let d = &mut *self.draft;
        let to = if super::lines::is_multi_line(&d.text) {
            if end {
                super::lines::end_of_line(&d.text, d.caret)
            } else {
                super::lines::start_of_line(&d.text, d.caret)
            }
        } else {
            if !shift && blocks::line(self.ctx, self.doc, d, end, self.actions) {
                return Flow::Done;
            }
            if end { d.text.chars().count() } else { 0 }
        };
        d.mark = caret::moved(d.mark, d.caret, shift);
        d.caret = to;
        self.moved();
        Flow::Continue
    }

    /// Enter: see [`enter_means`].
    fn enter(&mut self, m: Modifiers) -> Flow {
        let means = enter_means(&self.draft.anchor, m.command);
        // ui-text-exempt: diagnostic trace, never displayed.
        crate::diag::trace(|| format!("text-edit-enter means={means:?}"));
        match means {
            EnterMeans::Commit => {
                commit_into(self.ctx, self.draft, self.actions);
                abandon(self.ctx);
                return Flow::Done;
            }
            EnterMeans::CannotSplit => {
                self.actions.push(Action::Text(
                    crate::app::actions::text::TextAction::EnterCannotSplit,
                ));
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    "text-edit-enter-declined reason=run-cannot-hold-a-newline".to_owned()
                });
            }
            EnterMeans::NewLine => {
                self.record(EditKind::Other);
                self.draft.caret = take_selection(self.draft);
                self.draft.caret = caret::newline(&mut self.draft.text, self.draft.caret);
                self.changed = true;
            }
        }
        Flow::Continue
    }
}

/// `caret::insert` for text that may hold line breaks: each break goes in by
/// `caret::newline`, as `insert` drops control characters.
fn insert_lines(text: &mut String, mut at: usize, s: &str) -> usize {
    for (i, piece) in s.split('\n').enumerate() {
        if i > 0 {
            at = caret::newline(text, at);
        }
        at = insert(text, at, piece);
    }
    at
}

/// A space's width on screen in the drawn box: measured at a space in `text`
/// when there is one, else a quarter of the box's height.
fn space_px(l: &super::hit::Layout, text: &str) -> f32 {
    text.chars()
        .position(|c| c == ' ')
        .map(|i| l.x_at(i + 1) - l.x_at(i))
        .filter(|w| *w > 0.0)
        .unwrap_or(l.body.height() * 0.25)
}

/// The ribbon command Ctrl plus `key` stands for inside a draft.
fn style_command(key: Key) -> &'static str {
    // ui-text-exempt: registered command ids, never displayed.
    match key {
        Key::B => "format.bold",
        Key::I => "format.italic",
        _ => "format.underline",
    }
}

#[cfg(test)]
mod tests {
    use super::insert_lines;

    #[test]
    fn pasted_lines_keep_their_breaks_in_new_text() {
        let mut t = String::from("ab");
        let at = insert_lines(&mut t, 1, "x\ny");
        assert_eq!(t, "ax\nyb");
        assert_eq!(at, 4);
    }
}
