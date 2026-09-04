//! # `dialogs::textannot` — the words half of a text-bearing annotation
//!
//! The second half of the place-then-type gesture. The canvas has taken a
//! rectangle (or a point); this asks what goes in it, and **nothing reaches
//! the document until Accept.**
//!
//! ## ★ Why a dialog, when markup authors on release
//!
//! `crate::dialogs`' header draws the line: *"a dialog is a single transaction
//! with a start and an end… a panel is somewhere an operator dips in and out
//! of while working."* Typing a callout is unmistakably the first — it begins
//! when the box is drawn, it ends when the words are accepted or abandoned,
//! and there is nothing to dip back into afterwards.
//!
//! The alternative — an in-place editor drawn over the page, the way a word
//! processor would — was rejected on the standing rule that **nothing floats
//! over the canvas** except the Find bar, which is a documented exception the
//! operator granted for one surface. It would also have needed a caret, a
//! selection and a hit test over text this shell does not own, which is a text
//! editor rather than a dialog.
//!
//! ## ★ It is deliberately NOT modal to the document
//!
//! The reference line stays drawn, the page stays where it was, and the dialog
//! is `default_pos` rather than anchored so it can be dragged aside. An
//! operator writing a callout is usually looking at the thing they are calling
//! out, and a window pinned over it would make them close the window to read
//! what they were annotating.
//!
//! ## The three kinds meet three different questions
//!
//! | kind | what this asks |
//! |---|---|
//! | text box | *what should it say?* — a multi-line field, because a callout wraps |
//! | sticky note | *what is the note?* — the same field; the words live in a popup rather than on the page, and the window says so |
//! | stamp | *which stamp?* — a gallery, and **no text field at all** |
//!
//! The stamp's absence of a field is the important one. `manifest/markup.rs`
//! recorded the blocker as *"a stamp with no chooser has no operand"*, and the
//! converse is just as true: a stamp with a free-text field is a text box with
//! a border, and offering both would be two controls for one feature with no
//! way for an operator to tell which they wanted.

use egui::Ui;
use pdfcer_core::annot_author::StampName;
use pdfcer_core::page_tree::Rect;

use crate::app::actions::Action;
use crate::canvas::textannot::{DEFAULT_STAMP, MAX_TEXT_CHARS, STAMPS, TextAnnotKind};
use crate::text::textannot as t;

/// The region the whole window publishes.
pub const REGION_BODY: &str = "dialog:text-annot"; // ui-text-exempt: trace region name, never displayed
/// The region the text field publishes, so a driven check can type into it.
pub const REGION_TEXT: &str = "text-annot.text"; // ui-text-exempt: trace region name, never displayed
/// The region the Accept control publishes.
pub const REGION_ACCEPT: &str = "text-annot.accept"; // ui-text-exempt: trace region name, never displayed

/// One open text-annotation dialog.
pub struct TextAnnotDialog {
    /// The page the annotation will land on, captured when the gesture
    /// completed.
    ///
    /// **Not re-read per frame.** The operator drew a box on the sheet they
    /// were looking at; a page change while this window is open must not
    /// redirect the annotation, which is the same rule the Set-scale dialog
    /// applies to its group.
    page: usize,
    /// Which kind is being authored.
    kind: TextAnnotKind,
    /// The rectangle, in PDF user space, captured with the page.
    rect: Rect,
    /// What the operator has typed.
    text: String,
    /// The stamp selected in the gallery. Meaningless for the other kinds and
    /// carried anyway — see `Action::CommitTextAnnot`'s field of the same name.
    stamp: StampName,
    /// Set by Accept, consumed after the window's closure returns.
    accept_requested: bool,
    /// Set by Cancel, consumed by [`Self::show`].
    close_requested: bool,
    /// Whether the text field has been **observed holding** focus.
    ///
    /// ★ It exists because a dialog that asks a question should put the caret
    /// where the answer goes. Without it the operator draws a box, a window
    /// appears asking what it should say, and they have to click into the field
    /// before they can type — which is a step the window itself created.
    ///
    /// ★★ Note what it records: that the field **has** focus, not that focus
    /// was **requested**. Those were conflated, and the difference is the whole
    /// defect — see [`Self::field`].
    focused_once: bool,
    /// How many frames have asked for focus without getting it.
    ///
    /// Bounds the retry, so a field that can never take focus cannot fight the
    /// operator for the rest of the dialog's life. See
    /// [`FOCUS_ATTEMPT_FRAMES`].
    focus_attempts: u8,
}

/// **How many frames may ask for the text field's focus before giving up.**
///
/// The retry exists because the dialog's first frame races the pointer release
/// that opened it (see [`TextAnnotDialog::field`]); it is bounded because
/// asking forever would take focus back from Cancel and from the stamp gallery,
/// and a window that cannot be dismissed is worse than one that cannot be typed
/// into.
///
/// Eight frames is a shade over a tenth of a second at 60 Hz — longer than any
/// number of frames a release takes to resolve, and far shorter than a human
/// noticing the window and reaching for the mouse. Nothing here depends on the
/// exact value; it only has to sit inside that gap, which is two orders of
/// magnitude wide.
const FOCUS_ATTEMPT_FRAMES: u8 = 8;

/// The size the note window opens at, before it is squeezed by a narrow
/// application window.
///
/// Wide enough for a four-line callout at the body text size without the field
/// wrapping every sentence, and short enough that the window reads as a
/// question rather than as a second document. The stamp gallery is the taller
/// of the two bodies and fits inside it.
const WINDOW_PTS: egui::Vec2 = egui::vec2(420.0, 240.0);

/// The smallest the note window may be, by resize or by squeeze.
///
/// The same floor handed to `Host` as its `min_size`, read from one constant so
/// the two cannot disagree. A dialog squeezed below the size it refuses to be
/// dragged to would be a window in a state the operator could not return it to.
const MIN_WINDOW_PTS: egui::Vec2 = egui::vec2(320.0, 200.0);

/// How much of the application window is left clear on either side when the
/// note window has to be squeezed to fit it.
///
/// Purely so a squeezed dialog does not sit edge-to-edge with the window it
/// belongs to, which reads as a rendering fault rather than as a dialog.
const SCREEN_MARGIN_PTS: f32 = 40.0;

/// **How big the note window opens**, given the application window's content
/// rectangle.
///
/// Derived from [`WINDOW_PTS`], [`MIN_WINDOW_PTS`] and the *outer* rectangle —
/// never from anything the body lays out. That is `print/layout.rs`'s rule and
/// `Host::fit`'s: a size measured from the content it sizes is R128, which this
/// project has met three times.
///
/// ★ The floor is new as of 2026-09-04 and the previous expression had none:
/// `420.0.min(screen.width() - 40.0)` goes **negative** on an application
/// window narrower than 40 pt. Unreachable in practice and free to close, and
/// an unreachable negative size is the kind of thing that becomes reachable
/// when somebody adds a second monitor at 250 % scaling.
#[must_use]
fn window_size(screen: egui::Rect) -> egui::Vec2 {
    egui::vec2(
        WINDOW_PTS
            .x
            .min(screen.width() - SCREEN_MARGIN_PTS)
            .max(MIN_WINDOW_PTS.x),
        WINDOW_PTS.y,
    )
}

/// **Where the note window opens**, in the application window's own screen
/// coordinates.
///
/// Centred horizontally and a **third** of the way down, not half — the same
/// placement the Set-scale dialog uses, and for the same reason: a window
/// centred vertically sits exactly over the middle of the page, which on a
/// drawing sheet is where the content is.
///
/// # ★★ This is not, and must not become, a click-relative position
///
/// The review that found A16c described the discarded computation as
/// *"click-relative"*. It never was, and making it so would contradict this
/// module's own header: *"an operator writing a callout is usually looking at
/// the thing they are calling out, and a window pinned over it would make them
/// close the window to read what they were annotating."* A dialog that opened
/// on top of the note would be a worse answer than the corner, not a better
/// one. What A16c is about is the dialog reaching **the position it computed**
/// instead of the corner.
///
/// ★ The clamp onto the application window lives in
/// `dialogs::host::placement`, not here. This function's job is to say where
/// the dialog belongs; keeping it free of edge cases is what lets it be a
/// three-line expression that can be read at a glance and tested without a
/// window.
#[must_use]
fn opening_position(screen: egui::Rect, size: egui::Vec2) -> egui::Pos2 {
    egui::pos2(
        ((screen.width() - size.x).max(0.0) / 2.0).max(0.0),
        ((screen.height() - size.y).max(0.0) / 3.0).max(0.0),
    )
}

impl TextAnnotDialog {
    /// Open for a placed annotation.
    #[must_use]
    pub fn open(page: usize, kind: TextAnnotKind, rect: Rect) -> Self {
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "text-annot-open kind={kind:?} page={page} w={:.2} h={:.2}",
                rect.urx - rect.llx,
                rect.ury - rect.lly
            )
        });
        Self {
            page,
            kind,
            rect,
            text: String::new(),
            stamp: DEFAULT_STAMP,
            accept_requested: false,
            close_requested: false,
            focused_once: false,
            focus_attempts: 0,
        }
    }

    /// Draw one frame. Returns `false` when it should close.
    pub fn show(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) -> bool {
        let screen = ctx.input(egui::InputState::content_rect);
        let size = window_size(screen);
        // ★★★ ITS OWN OS WINDOW as of 2026-08-21, AND IT OPENS WHERE IT SAYS —
        // the second half restored 2026-09-04, review finding A16c.
        //
        // A note is typed *about* something on the page, so the one window that
        // must be movable off the document is this one. When it became an OS
        // window the computed position had nowhere to go — `Host` placed every
        // dialog at a fixed inset from the application window's corner — and
        // this line read:
        //
        // > `let _ = pos;` — *"The computed opening position is retired with
        // > the `egui::Window` it fed."*
        //
        // It was not retired. It was **discarded**, and an outside review found
        // the consequence by opening one note: the dialog appears in the
        // top-left corner of the window rather than where the operator is
        // looking. That is a small cost paid on **every note**, and a markup
        // session places dozens — the dialog is opened and dismissed so often
        // that it almost never has a remembered position to restore, so the
        // corner is very nearly the only place it ever appeared.
        //
        // ★ [`Host::opening_near`] clamps this onto the application window, so
        // the arithmetic below may stay a statement about where the dialog
        // *should* go without also having to be a statement about monitors.
        let (frame, ()) = crate::dialogs::host::Host::new(
            "text-annot", // ui-text-exempt: a viewport key, never displayed.
            t::title(self.kind),
            size,
            MIN_WINDOW_PTS,
        )
        .opening_near(opening_position(screen, size))
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui);
        });
        let open = !frame.closed;

        if self.accept_requested {
            self.accept_requested = false;
            actions.push(Action::CommitTextAnnot {
                page: self.page,
                kind: self.kind,
                rect: self.rect,
                text: std::mem::take(&mut self.text),
                stamp: self.stamp,
            });
            return false;
        }
        // ★ The window's own close button counts as Cancel, and authors
        // nothing. That is the honest reading: the operator dismissed a
        // question, and a dismissed question is not an answer.
        !(self.close_requested || !open)
    }

    /// The field or the gallery, then the two buttons.
    fn body(&mut self, ui: &mut Ui) {
        ui.label(t::intro(self.kind));
        ui.add_space(8.0);

        if self.kind.uses_gallery() {
            self.gallery(ui);
        } else {
            self.field(ui);
        }

        ui.add_space(10.0);
        ui.separator();
        ui.horizontal(|ui| {
            // ★ Accept is greyed when there is nothing to author, with the
            // reason on hover. That is the one place this shell greys rather
            // than hides: the control is *temporarily* unavailable — a
            // keystroke makes it live — which is exactly what greying is
            // reserved for.
            let ready = self.kind.uses_gallery() || !self.text.trim().is_empty();
            let accept = ui.add_enabled(ready, egui::Button::new(t::accept()));
            crate::diag::ui_rect(REGION_ACCEPT, accept.rect);
            if accept.clicked() {
                self.accept_requested = true;
            }
            if !ready {
                accept.on_disabled_hover_text(t::accept_disabled(self.kind));
            }
            if ui.button(t::cancel()).clicked() {
                self.close_requested = true;
            }
        });
    }

    /// The free-text field, for the two kinds whose words the operator writes.
    fn field(&mut self, ui: &mut Ui) {
        let response = ui.add(
            egui::TextEdit::multiline(&mut self.text)
                .desired_rows(4)
                .desired_width(f32::INFINITY)
                .hint_text(t::hint(self.kind))
                .char_limit(MAX_TEXT_CHARS),
        );
        crate::diag::ui_rect(REGION_TEXT, response.rect);
        // ★★ **Ask until the field actually HOLDS focus — not once.**
        //
        // This used to latch on having *asked*: `request_focus(); focused_once
        // = true;`. Asking and holding are different facts, and the gap between
        // them is a window the operator types into and nothing happens.
        //
        // The dialog's first frame is the frame **after** the gesture that
        // opened it — `Action::BeginTextAnnot` is raised by the canvas and
        // applied when the queue drains, so the pointer release that finished
        // the drag is still being resolved around the request. A request that
        // loses that race was never retried, because the latch had already been
        // set by the asking; the field then sat there looking like the place to
        // type while every keystroke went somewhere else. The operator's report
        // was *"it doesn't type anything in the box when I type"*.
        //
        // ★ Why it is bounded, and not simply "ask whenever unfocused". The
        // original comment's objection is still correct — `request_focus` every
        // frame would fight anything the operator clicked, including Cancel, and
        // a dialog that cannot be cancelled is worse than one that cannot be
        // typed into. So the retry is limited to [`FOCUS_ATTEMPT_FRAMES`], which
        // is long enough to outlast the release being resolved and far short of
        // a human reaching for the mouse.
        //
        // ★ And it latches on `has_focus()` rather than counting down, so the
        // common case costs exactly one request: the frame after a successful
        // one observes focus and stops asking for good.
        // ★★★ AND THE BUDGET ONLY RUNS WHILE THE WINDOW ITSELF IS FOCUSED —
        // 2026-08-21, when this dialog became a real OS window.
        //
        // The retry above is eight frames, chosen to outlast a pointer release
        // being resolved. That was the whole race while the dialog drew inside
        // the application's window and inherited its focus. An OS window has
        // focus of its own, granted by the **platform**, and the grant can take
        // longer than eight frames or not arrive at all — Windows refuses the
        // foreground to a process that does not already have it.
        //
        // Spending the budget during that wait means every attempt is made at a
        // window that cannot hold focus, the counter reaches its bound, and the
        // field is never focused **at the moment it becomes possible**. The
        // measured symptom: `text_annot_takes_the_keyboard_unclicked` typed two
        // characters, the Accept control stayed disabled because the field was
        // empty, and pressing it authored nothing — the operator's version being
        // *"I dragged out a note box and typing did nothing."*
        //
        // So an attempt is only counted while the window is focused. The bound
        // keeps its original meaning — *don't fight the operator's own click* —
        // and stops being consumed by a wait that has nothing to do with them.
        let window_focused = ui.ctx().input(|i| i.viewport().focused) != Some(false);
        // ★★ Published because a field that never takes focus is a whole
        // defect class in this shell — *"it doesn't type anything in the box
        // when I type"* — and it is invisible from outside: the box is drawn,
        // the caret blinks, and the characters go somewhere else. The four
        // numbers are the whole state machine above, so a driven check or a
        // reader of a trace can tell "never asked", "asked and lost the race",
        // and "held it and then the WINDOW lost focus" apart. All three were
        // suspected on 2026-08-21 and the trace is what ruled two of them out.
        crate::diag::trace_on_change("text-annot-field", || {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                // ui-text-exempt: diagnostic trace, never displayed.
                "has_focus={} once={} attempts={} window_focused={window_focused}",
                response.has_focus(),
                self.focused_once,
                self.focus_attempts
            )
        });
        if !self.focused_once {
            if response.has_focus() {
                self.focused_once = true;
            } else if window_focused && self.focus_attempts < FOCUS_ATTEMPT_FRAMES {
                self.focus_attempts += 1;
                response.request_focus();
            }
        }
        ui.label(egui::RichText::new(t::bound(self.kind)).small().weak());
    }

    /// The stamp gallery, for the one kind whose words come from `/Name`.
    fn gallery(&mut self, ui: &mut Ui) {
        // A vertical list of radios rather than a combo box: seven entries is
        // a set an operator reads at a glance, and a combo would hide six of
        // them behind a click for no saving — this window has the room.
        for stamp in STAMPS {
            ui.radio_value(&mut self.stamp, *stamp, t::stamp_label(*stamp));
        }
        ui.add_space(4.0);
        ui.label(egui::RichText::new(t::stamp_bound()).small().weak());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect() -> Rect {
        Rect {
            llx: 0.0,
            lly: 0.0,
            urx: 100.0,
            ury: 40.0,
        }
    }

    /// **A `RawInput` describing a real screen at a deterministic time.**
    ///
    /// Two fields of `RawInput::default()` are wrong for driving this dialog,
    /// and each cost a debugging round when it was left alone.
    ///
    /// **`screen_rect` is `None`.** This dialog sizes itself from
    /// `content_rect` — `420.min(width - 40)` — so a default input hands
    /// `egui::Window` a degenerate size and the field inside it a width nothing
    /// can be focused in. A test that lays out differently from the application
    /// is measuring a different program.
    ///
    /// **`time` is `None`, and egui then fills it from the wall clock.** That
    /// makes frame timing depend on how loaded the machine is, so a test that
    /// drives several frames is reproducible when run alone and intermittent
    /// when run beside a thousand others — which is precisely the flake that
    /// gets re-run until it is green and then believed. Time is supplied here,
    /// one 60 Hz tick per frame, so the sequence is the same every time.
    fn on_screen(frame: u32) -> egui::RawInput {
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1280.0, 800.0),
            )),
            time: Some(f64::from(frame) / 60.0),
            predicted_dt: 1.0 / 60.0,
            ..Default::default()
        }
    }

    /// The application window this dialog's geometry is computed against.
    fn screen() -> egui::Rect {
        egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1280.0, 800.0))
    }

    /// ★★★ **The note window does not open in the corner** — review finding
    /// A16c.
    ///
    /// The whole of the defect in one assertion. `dialogs/textannot.rs`
    /// computed this position and then wrote `let _ = pos;`, so the dialog was
    /// placed by `Host`'s corner inset instead — on every open, dozens of times
    /// in a markup session, because a dialog dismissed that often almost never
    /// has a remembered position to restore.
    ///
    /// The position is asserted as a **relationship** rather than as two
    /// numbers: centred across the window and between a fifth and half of the
    /// way down it. Pinning the exact pixels would fail the next time the
    /// window's size changed for an unrelated reason, which is how a test stops
    /// being read and starts being edited.
    #[test]
    fn the_note_window_does_not_open_in_the_corner() {
        let screen = screen();
        let size = window_size(screen);
        let at = opening_position(screen, size);

        assert!(
            at.x > 0.0 && at.y > 0.0,
            "the note dialog opened at {at:?} — the top-left corner of the window is \
             precisely what A16c reported"
        );
        let centre_gap = (at.x + size.x / 2.0) - screen.center().x;
        assert!(
            centre_gap.abs() < 1.0,
            "the window must be centred across the application window; its centre is \
             {centre_gap} pt off"
        );
        let down = at.y / screen.height();
        assert!(
            (0.2..0.5).contains(&down),
            "a third of the way down, not half and not the top: got {down}"
        );
    }

    /// **The window is squeezed to fit a narrow application window, and never
    /// below the size it refuses to be dragged to.**
    ///
    /// The floor is the half that was missing: the expression used to be
    /// `420.min(width - 40)` with no `max`, which is **negative** for an
    /// application window under 40 pt wide. Unreachable today and free to
    /// close.
    #[test]
    fn the_note_window_is_squeezed_but_never_below_its_own_floor() {
        let roomy = window_size(screen());
        assert_eq!(roomy, WINDOW_PTS, "a wide window gets the size asked for");

        let narrow = window_size(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(380.0, 800.0),
        ));
        assert!(narrow.x < WINDOW_PTS.x, "a narrow window squeezes it");
        assert!(narrow.x >= MIN_WINDOW_PTS.x);

        let absurd = window_size(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(10.0, 10.0),
        ));
        assert!(
            absurd.x >= MIN_WINDOW_PTS.x,
            "a window narrower than the margin must not produce a size of {absurd:?}"
        );
    }

    /// A fresh dialog carries no words and the gallery's default.
    #[test]
    fn a_fresh_dialog_is_empty_and_defaulted() {
        let d = TextAnnotDialog::open(0, TextAnnotKind::TextBox, rect());
        assert!(d.text.is_empty(), "no words are invented for the operator");
        assert_eq!(d.stamp, DEFAULT_STAMP);
        assert!(!d.accept_requested);
    }

    /// ★ The page and the rect are captured, not re-read.
    ///
    /// The property that stops a page change under an open window redirecting
    /// the annotation. Asserted on the stored values because there is nothing
    /// else to assert it on — the whole point is that nothing re-reads them.
    #[test]
    fn the_page_and_rect_are_captured_at_open() {
        let d = TextAnnotDialog::open(7, TextAnnotKind::Sticky, rect());
        assert_eq!(d.page, 7);
        assert!((d.rect.urx - 100.0).abs() < f64::EPSILON);
    }

    /// ★ Accept is live for a stamp with no typed text, and dead for the
    /// others.
    ///
    /// The readiness rule, which is the gallery exception stated once more at
    /// the control that depends on it. A stamp whose Accept required typing
    /// could never be authored; a callout whose Accept did not would author an
    /// empty box.
    #[test]
    fn readiness_follows_the_gallery_rule() {
        let ready = |d: &TextAnnotDialog| d.kind.uses_gallery() || !d.text.trim().is_empty();

        let stamp = TextAnnotDialog::open(0, TextAnnotKind::Stamp, rect());
        assert!(ready(&stamp), "a stamp needs no typed words");

        let mut box_ = TextAnnotDialog::open(0, TextAnnotKind::TextBox, rect());
        assert!(!ready(&box_), "an empty callout must not be authorable");
        box_.text = "   ".to_owned();
        assert!(!ready(&box_), "whitespace is not words");
        box_.text = "note".to_owned();
        assert!(ready(&box_));
    }

    /// **The oracle for *"it doesn't type anything in the box when I type"*.**
    ///
    /// Every test above asserts on the struct's fields, which is exactly the
    /// blind spot `DEFECTS.md` D1 was: they all pass on a build whose window
    /// accepts no keystrokes, because none of them ever draws one. This drives
    /// a real `egui::Context` through two frames — one to build the field and
    /// take its one-shot focus, one carrying a real `Event::Text` — and asserts
    /// the words arrived.
    #[test]
    fn typing_into_the_open_window_reaches_the_draft() {
        let ctx = egui::Context::default();
        let mut d = TextAnnotDialog::open(0, TextAnnotKind::TextBox, rect());
        let mut actions = Vec::new();

        // Frame 0: the field is created and requests focus.
        let _ = ctx.run_ui(on_screen(0), |ui| {
            d.show(ui.ctx(), &mut actions);
        });

        // Frame 1: a real keystroke, the way a keyboard delivers one.
        let mut input = on_screen(1);
        input.events.push(egui::Event::Text("h".to_owned()));
        let _ = ctx.run_ui(input, |ui| {
            d.show(ui.ctx(), &mut actions);
        });

        assert_eq!(d.text, "h", "the window took the keystroke");
    }

    /// ★★ **The regression test: focus LOST on the opening frame is re-taken.**
    ///
    /// The defect this replaced latched on having *asked* for focus rather than
    /// on holding it, so a request that lost its frame was never retried and
    /// the field sat there looking typeable while every keystroke went
    /// elsewhere. That is unreachable in a bare `egui::Context` — the request
    /// always wins when nothing competes — which is why the test above passed
    /// on the broken build and why this one takes the focus away by hand.
    ///
    /// The theft models what the real frame does: the dialog's first draw is
    /// the frame AFTER the gesture that opened it, so the pointer release that
    /// finished the drag is still being resolved around the request.
    #[test]
    fn focus_stolen_on_the_opening_frame_is_taken_back() {
        let ctx = egui::Context::default();
        let mut d = TextAnnotDialog::open(0, TextAnnotKind::TextBox, rect());
        let mut actions = Vec::new();
        let thief = egui::Id::new("whatever-won-the-release");

        // Frame 0: the dialog draws and asks for focus...
        let _ = ctx.run_ui(on_screen(0), |ui| {
            d.show(ui.ctx(), &mut actions);
        });
        // ...and loses it, the way a release being resolved would take it.
        ctx.memory_mut(|m| m.request_focus(thief));

        // The retry frames. Bounded by the budget rather than assuming one
        // frame is enough: when two widgets ask for focus in the same pass egui
        // keeps the earlier request, so the field can need a second attempt to
        // win it back. The claim under test is *"within the budget"*, which is
        // what the production code promises - not *"on the very next frame"*.
        for frame in 1..=u32::from(FOCUS_ATTEMPT_FRAMES) {
            let _ = ctx.run_ui(on_screen(frame), |ui| {
                d.show(ui.ctx(), &mut actions);
            });
        }

        // The keystroke, which is the assertion that matters -- "focus was
        // requested" is the very claim that shipped broken.
        let mut input = on_screen(u32::from(FOCUS_ATTEMPT_FRAMES) + 1);
        input.events.push(egui::Event::Text("h".to_owned()));
        let _ = ctx.run_ui(input, |ui| {
            d.show(ui.ctx(), &mut actions);
        });

        assert_eq!(
            d.text, "h",
            "the field lost focus on its opening frame and never took it back, so the operator \
             types into a window that is ignoring them"
        );
    }

    /// ★ ...and the retry is BOUNDED, so Cancel stays clickable.
    ///
    /// The objection the original one-shot latch was written to answer, and it
    /// is still correct: a field that asks for focus every frame takes it back
    /// from whatever the operator clicked, and a window that cannot be
    /// dismissed is worse than one that cannot be typed into.
    ///
    /// The competitor is a **real drawn button**, not a bare `Id`. egui drops
    /// focus for an id no widget registered that frame, so focusing an invented
    /// id proves nothing about who won — it only proves egui tidied up.
    #[test]
    fn the_focus_retry_gives_up_so_another_control_can_hold_it() {
        let ctx = egui::Context::default();
        let mut d = TextAnnotDialog::open(0, TextAnnotKind::TextBox, rect());
        let mut actions = Vec::new();
        let mut other = None;

        // A real button, drawn every frame beside the dialog, taking focus the
        // way a control the operator clicked would. Its id is read back from
        // the `Response` rather than invented, so the assertion names the
        // widget egui actually registered.
        let mut n = 0;
        let mut frame = |steal: bool, d: &mut TextAnnotDialog, other: &mut Option<egui::Id>| {
            n += 1;
            let _ = ctx.run_ui(on_screen(n), |ui| {
                d.show(ui.ctx(), &mut actions);
                let r = ui.button("Cancel");
                *other = Some(r.id);
                if steal {
                    r.request_focus();
                }
            });
        };

        // Outlast the budget, taking focus back every single frame.
        for _ in 0..(FOCUS_ATTEMPT_FRAMES as usize + 2) {
            frame(true, &mut d, &mut other);
        }
        // One more frame with nobody competing: the field must NOT grab it.
        frame(false, &mut d, &mut other);

        assert_eq!(
            ctx.memory(|m| m.focused()),
            other,
            "the field kept grabbing focus back, so nothing else in the window can be used"
        );
    }
}
