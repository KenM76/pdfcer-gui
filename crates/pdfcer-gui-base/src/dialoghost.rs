//! # `dialoghost` — a dialog is an OS WINDOW
//!
//!
//! > *"Print dialogue box doesn't pop up in its own movable window. It is
//! > locked within the boundaries of the program's window. Like, I just assume
//! > you've been trained on a million lines of code and software that pops it
//! > up in its own window."*
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/dialoghost.md`.

use egui::{Pos2, Vec2, ViewportBuilder, ViewportClass, ViewportId};

/// **Where a dialog's window opens.** See that module's header for the seam and
/// for A16c, the defect it was split out to fix.
mod placement;

/// **Window auto-sizing** — the arithmetic that grows a dialog to its body,
/// the two guards that keep that from becoming a feedback loop, and the budget
/// that stops it when the measurement is circular. Split out of this file on
/// 2026-09-10 under R2, at 1,600 lines; its header carries the runaway it was
/// written after and why the arithmetic is a free function.
mod fit;

/// Where the application window's handle is kept for [`Host::show`] to find.
const OWNER_KEY: &str = "dialog-host-owner"; // ui-text-exempt: a memory key, never displayed.

/// **Tell the dialog host which window owns its dialogs.** Called once a frame
/// by the application, before any dialog draws.
pub fn set_owner(ctx: &egui::Context, window: Option<isize>) {
    let key = egui::Id::new(OWNER_KEY);
    match window {
        Some(w) => {
            ctx.data_mut(|d| d.insert_temp(key, w));
        }
        None => ctx.data_mut(|d| d.remove::<isize>(key)),
    }
}

/// The application window's handle, if [`set_owner`] has been called.
fn owner(ctx: &egui::Context) -> Option<isize> {
    ctx.data(|d| d.get_temp::<isize>(egui::Id::new(OWNER_KEY)))
}

/// How many passes from opening a dialog goes on asking for the keyboard.
///
/// # Measured, twice, and the second measurement moved it
///
/// One request on the opening pass was **granted** — the dialog traced
/// `focused=Some(true)` — and then lost again. Eight passes was tried next, on
/// the theory that the window manager was still settling. Also not enough. The
/// trace says exactly when, with both windows reporting:
///
/// ```text
/// text-annot-open kind=TextBox …
/// dialog-focus  title="Text box" focused=Some(true)     <- the dialog gains it
/// root-focus    focused=Some(false)
/// …17 idle passes, no resize, no reposition, no input…
/// root-focus    focused=Some(true)                      <- and the ROOT takes it back
/// dialog-focus  title="Text box" focused=Some(false)
/// ```
///
/// The application asks for none of that: no `Focus` command, no
/// `SetWindowPos`, nothing between the two. The platform hands the foreground
/// back to the owner-less main window about a third of a second after the child
/// appears.
///
/// # THE NUMBER STAYED AT EIGHT, AND THE HUNT THAT NEARLY CHANGED IT IS
/// # THE LESSON
///
/// A driven check reported that a note dialog *"does not take the keyboard
/// when it opens"*, and this constant was raised to forty passes and then to
/// a hundred and twenty chasing it. Each raise was justified by a measurement
/// — the dialog visibly held the foreground while the requests were going out
/// and lost it when they stopped — and **every one of them was fixing the
/// wrong thing.**
///
/// The check was clicking its Accept button through the APPLICATION window's
/// coordinates while the dialog had its own. It typed correctly, the dialog
/// received the characters correctly, and then the click that should have
/// committed them landed on a page. Converting that one call site made the
/// check pass **with this constant back at eight**, which is how the raises
/// were shown to have bought nothing.
///
/// Two things are worth carrying out of that:
///
/// - **A knob must not sit at a value chosen to fix something it does not
///   fix.** Left at a hundred and twenty, a future reader would have believed
///   the problem was tuning, and a dialog would have re-seized the foreground
///   for two seconds after every opening for no benefit at all.
/// - **A measurement that moves with the knob is not proof the knob is the
///   subject.** Focus really did follow the requests; the requests really were
///   irrelevant to the failure. Both were true at once.
///
/// The bound is not the only guard, and on its own it would be a bad one —
/// see [`ENGAGED`]. A dialog stops asking the instant the operator touches it,
/// so the only case this can fight is *clicking away within half a second of a
/// dialog appearing without having interacted with it*, whose worst outcome is
/// the dialog coming back to the front once.
const FOCUS_FRAMES: u64 = 8;

/// Marker written when a dialog first receives input of its own.
const ENGAGED: bool = true;

/// One dialog's window: what it is called, how big it opens, and where the
/// operator last left it.
pub struct Host {
    /// The viewport id, stable for this dialog across frames.
    ///
    /// Derived from a caller-supplied string rather than counted, because
    /// `ViewportId` is what egui keys the OS window on: two dialogs sharing one
    /// would be two dialogs sharing one window, and a counter would give a
    /// dialog a different window depending on what else was open when it was
    /// created.
    id: ViewportId,
    /// Where the last size this host ASKED FOR is kept, so it asks once.
    /// See [`Self::fit`].
    fit_key: egui::Id,
    /// How many times [`Host::fit`] has already grown this window. See the
    /// growth budget in `fit` for why a count, and not a size, is the thing
    /// that distinguishes a legitimate fit from a feedback loop.
    budget_key: egui::Id,
    /// Where the pass number of the last frame this dialog was drawn on is
    /// kept, so a **fresh opening** can be told from a continuing one. See
    /// [`Self::show`]'s focus request.
    seen_key: egui::Id,
    /// Where [`ENGAGED`] is recorded — whether the operator has yet used this
    /// dialog, which is what stops it asking for the keyboard.
    engaged_key: egui::Id,
    /// Where the remembered position is kept in `egui::Memory`.
    ///
    /// Derived from the same string as [`Self::id`] and salted, so it cannot
    /// collide with anything else keyed on the dialog's name. See the module
    /// header for why the position is not a field.
    key: egui::Id,
    /// Whether the title bar offers maximise. Off for a dialog — see
    /// [`Self::show`] — and on for a window that is a view rather than a
    /// transaction, such as the popped-out print preview.
    maximizable: bool,
    /// The window's title bar text. Owned rather than `&'static str` because it
    /// may carry a document name.
    title: String,
    /// The size it opens at.
    default_size: Vec2,
    /// The smallest it may be dragged to.
    ///
    /// A floor, not a preference — the reason is `print`'s and it generalises:
    /// a resizable window with no floor can be dragged down to a title bar and
    /// a scrollbar, which is a state with no way back except closing the
    /// dialog and losing what was typed into it.
    min_size: Vec2,
    /// Where the caller would **prefer** this dialog to open, in the
    /// application window's own egui screen coordinates. See
    /// [`Host::opening_near`].
    ///
    /// `Option`, and the `None` case is not "no opinion, use zero" — it is
    /// *"place it the way every dialog without an opinion is placed"*.
    /// Collapsing the two into a `Pos2` with a sentinel would make "the corner"
    /// and "somebody asked for the corner" the same value, and the whole of
    /// A16c is the difference between those two.
    preferred: Option<Pos2>,
}

/// What one frame of a hosted dialog reported back.
pub struct Frame {
    /// Whether egui drew a real OS window or fell back to an embedded one.
    ///
    /// Carried rather than hidden because it is the honest answer to *"did G1
    /// actually happen"*, and because the embedded case has no position to
    /// remember. No caller is expected to branch on it.
    pub class: ViewportClass,
    /// The operator asked to close it — the OS close button, or Escape.
    ///
    /// Both, together, deliberately: G4 says Escape *is* Cancel and is *is* the
    /// close button, so a caller that treated them differently would give one
    /// of the three routes out a different meaning from the other two.
    pub closed: bool,
}

// ---------------------------------------------------------------------------
// The footer's region names
// ---------------------------------------------------------------------------
//
// Consumed by `tools/ui-verify`, which presses controls by name. They are
// declared here, beside their only publisher, and they are GENERIC rather than
// per-dialog on purpose -- see [`Host::footer`], which argues both that choice
// and the one case it gives up.

/// The region the affirmative footer button publishes -- Print, OK, Save.
pub const REGION_ACCEPT: &str = "dialog.buttons.accept";

/// The region the cancelling footer button publishes.
pub const REGION_CANCEL: &str = "dialog.buttons.cancel";

/// The region the optional third footer button publishes, when there is one.
pub const REGION_KEEP: &str = "dialog.buttons.keep";

/// `response`, with `hover` attached to it when there is a sentence to attach.
fn explained(response: egui::Response, hover: &str) -> egui::Response {
    if hover.is_empty() {
        response
    } else {
        response.on_hover_text(hover)
    }
}

impl Host {
    /// The padding between a dialog's content and its window edge, in points.
    const BODY_MARGIN_PTS: f32 = 12.0;

    /// A dialog window.
    #[must_use]
    pub fn new(id: &str, title: impl Into<String>, default_size: Vec2, min_size: Vec2) -> Self {
        Self {
            id: ViewportId::from_hash_of(id),
            // ui-text-exempt: a memory key, never displayed.
            key: egui::Id::new(("dialog-host-position", id)),
            // ui-text-exempt: a memory key, never displayed.
            fit_key: egui::Id::new(("dialog-host-fit", id)),
            budget_key: egui::Id::new(("dialog-host-fit-budget", id)),
            // ui-text-exempt: a memory key, never displayed.
            seen_key: egui::Id::new(("dialog-host-seen", id)),
            // ui-text-exempt: a memory key, never displayed.
            engaged_key: egui::Id::new(("dialog-host-engaged", id)),
            title: title.into(),
            default_size,
            min_size,
            preferred: None,
            maximizable: false,
        }
    }

    /// Offer maximise in the title bar. For a window that is a view onto
    /// something rather than a transaction; minimise stays off regardless.
    #[must_use]
    pub fn maximizable(mut self) -> Self {
        self.maximizable = true;
        self
    }

    /// **Open near `at`** — a position in the application window's own egui
    /// screen coordinates, the space `ctx.input(InputState::content_rect)`
    /// reports and every `ui_rect` in the main window is published in.
    #[must_use]
    pub fn opening_near(mut self, at: Pos2) -> Self {
        self.preferred = Some(at);
        self
    }

    /// Where this dialog was last left, in desktop coordinates.
    fn remembered(&self, ctx: &egui::Context) -> Option<Pos2> {
        ctx.data(|d| d.get_temp::<Pos2>(self.key))
    }

    /// Record where the OS has put this dialog.
    fn remember(&self, ctx: &egui::Context, at: Pos2) {
        ctx.data_mut(|d| d.insert_temp(self.key, at));
    }

    /// Where *a field held the keyboard when the last pass ended* is kept — in
    /// the **child** window's memory, not the parent's.
    fn field_focus_key(&self) -> egui::Id {
        self.key.with("field-focus")
    }
}

/// Where [`Host::show`] tells [`Host::footer`] that a field held the keyboard
/// when the last pass ended. Keyed on the viewport, because `footer` is handed
/// a `Ui` and not the `Host`. Call it OUTSIDE `data`/`data_mut`:
/// `viewport_id` takes the context lock those hold, and nesting deadlocks.
fn enter_grace_key(ctx: &egui::Context) -> egui::Id {
    egui::Id::new("dialog-enter-grace").with(ctx.viewport_id())
}

impl Host {
    /// **Draw one frame of this dialog in its own OS window.**
    pub fn show<R>(
        &self,
        ctx: &egui::Context,
        mut add: impl FnMut(&mut egui::Ui) -> R,
    ) -> (Frame, R) {
        let owner = owner(ctx);
        //
        // `show_viewport_immediate` takes `impl FnMut` because egui reserves
        // the right to call a viewport's callback **more than once in a
        // frame** — and it exercises that right routinely, whenever anything
        // in the body calls `Context::request_discard` to re-run a pass at a
        // size it has just learned. A `Grid` or a wrapped table doing its own
        // sizing is enough.
        //
        //
        // > the honest signature for a dialog body, which draws once per frame
        // > and may consume what it captures [...] a second call would `expect`
        // > here rather than silently drawing nothing, because "the dialog was
        // > blank" is a symptom nobody could trace back to this line.
        //
        // Every clause is reasonable and the conclusion turned a routine egui
        // behaviour into a **process abort**. An outside reviewer found it in
        // the first ten minutes: `pdfcer ▸ Keyboard shortcuts` panicked
        // instantly, on a fresh launch, taking the open documents with it.
        //
        // The choice between "blank" and "crash" was a FALSE ONE. The third
        // option is the correct one and it is what egui means: **draw again.**
        // A second pass exists precisely because the first is being discarded,
        // so re-running the body is not a workaround, it is the contract. The
        // result of the last call is the one returned, exactly as egui's own
        // `*out = Some(...)` keeps the last.
        //
        // What this costs: a body must now be re-runnable within a frame.
        // That is already true of every dialog here — they draw from state they
        // borrow rather than consume — and it is the same requirement
        // immediate mode places on every other widget in the process. If a
        // future body genuinely cannot be run twice, the fix is to move what it
        // consumes out of the closure, not to reinstate the panic.
        let mut builder = ViewportBuilder::default()
            .with_title(self.title.clone())
            .with_inner_size(self.default_size)
            .with_min_inner_size(self.min_size)
            // No minimize, and no maximize unless `maximizable` asked for it.
            // A dialog is one transaction; the
            // operator finishes it or abandons it, and a minimised dialog is a
            // transaction that has been left open with no surface saying so.
            // Every platform's dialog chrome makes the same choice.
            .with_minimize_button(false)
            .with_maximize_button(self.maximizable)
            // It IS in the window list, deliberately, and that is the half of
            // the operator's report that a borderless window would not fix:
            // *"find it when it has gone behind something"*. With G3
            // unavailable (see the module header) this is the only route back
            // to a dialog that has fallen behind the parent.
            .with_taskbar(true);
        // A SHORT WINDOW, not a single frame, and the reason is measured.
        //
        // One `Focus` on the opening frame was sent, granted — the dialog
        // traced `focused=Some(true)` — and **lost again a few frames later**,
        // before any keystroke arrived. A window that has just been created is
        // still settling with the window manager, and a single request lands in
        // the middle of that.
        //
        // So the request is repeated for [`FOCUS_FRAMES`] passes from the
        // opening and then stops for good. Bounded, because a `Focus` sent
        // forever would seize the foreground back from anything the operator
        // switched to while the dialog was open — including another
        // application, which is the behaviour of the worst software on the
        // machine. `dialogs::textannot`'s own field-focus retry is bounded for
        // the same reason and says so in the same words.
        let now = ctx.cumulative_pass_nr();
        let opened_at = ctx.data(|d| d.get_temp::<(u64, u64)>(self.seen_key));
        let (opened_at, last) = match opened_at {
            // A gap means it was closed and reopened: a fresh opening. The
            // gap is measured against a couple of passes rather than against
            // `FOCUS_FRAMES`, which is a different quantity and would make a
            // dialog reopened within half a second look like a continuation.
            Some((_, last)) if now.saturating_sub(last) > 2 => (now, now),
            Some((opened, _)) => (opened, now),
            None => (now, now),
        };
        if opened_at == now {
            // A fresh opening has not been engaged with yet.
            ctx.data_mut(|d| d.remove::<bool>(self.engaged_key));
            // AND IT HAS NOT BEEN FITTED YET EITHER — the operator's
            // *"the second time I place a stamp the window is too small to
            // show the Add button"*, 2026-09-10, and it was this line's
            // absence.
            //
            // `fit_key` and `budget_key` live in `egui::Memory`, keyed on the
            // dialog's id string, exactly as the remembered POSITION does. The
            // position is meant to outlive a close — that is G6. **The fit
            // state is not**, and nothing said so:
            //
            //   * `fit_key` holds the last size this dialog ASKED FOR. On the
            //     second opening the window is created afresh at
            //     `default_size`, the body measures the same overflow it
            //     measured the first time, and `fit` computes the same `want`
            //     — which equals the remembered one, so the once-per-size
            //     guard swallows it and **the resize is never sent**. The
            //     dialog opens at its opening bid and stays there, with
            //     whatever is at the bottom of the body clipped off. On a
            //     dialog whose bottom row is Accept/Cancel that is a window
            //     the operator cannot finish.
            //   * `budget_key` is worse, because it is cumulative: after
            //     [`FIT_BUDGET`] growths across the whole session, no opening
            //     of that dialog ever grows again.
            //
            // The guards were right and their SCOPE was wrong. Both exist
            // to stop a measurement feeding the size it measures **within one
            // opening** (R128, met three times here). A new window is a new
            // measurement of a new window; carrying the old answer into it is
            // not caution, it is a stale cache. So the budget is spent per
            // opening and the once-per-size memory starts empty, which is what
            // makes the second opening behave exactly like the first — the
            // property the operator noticed the absence of.
            self.forget_fit(ctx);
        }
        ctx.data_mut(|d| d.insert_temp(self.seen_key, (opened_at, last)));
        let engaged = ctx.data(|d| d.get_temp::<bool>(self.engaged_key)) == Some(ENGAGED);
        let opening = !engaged && now.saturating_sub(opened_at) < FOCUS_FRAMES;
        // The very first pass of this opening. See the position clause below.
        let placing = now == opened_at;

        // A POSITION IS ASSERTED ONCE, ON THE PASS THE DIALOG OPENS, and
        // never again while it is open.
        //
        // `show_viewport_immediate` DIFFS the builder against the previous
        // frame's and turns each changed property into a `ViewportCommand`. A
        // position clause that runs every frame therefore re-asserts a position
        // every frame — and the position it asserts comes from
        // [`Self::remembered`], which is written from the window's own
        // `outer_rect` inside the callback, one frame behind. Any wobble in
        // that round trip is a `SetWindowPos` per frame at the platform.
        //
        // Two things that costs, and the second is the one that was hunted
        // for an hour. It is G6's original defect in a new form — the window
        // being dragged back toward where the program thinks it is rather than
        // where the operator put it — and, because `SetWindowPos` participates
        // in window ACTIVATION, it is a live suspect for a dialog that was
        // granted the keyboard on opening and lost it a few passes later.
        //
        // Asserting once is also the honest statement of intent: the program
        // chooses where a dialog OPENS, and after that the window belongs to
        // the operator. `remembered` is still written every frame, because what
        // it feeds is the *next* opening.
        if placing {
            builder = match self.remembered(ctx) {
                // G6: back where it was left, including across a close. This
                // arm is FIRST and stays first: a position the operator dragged
                // the window to outranks any position the program computed,
                // including one a caller asked for through
                // [`Self::opening_near`]. A dialog that re-centred itself on
                // every open would undo the operator's placement once per
                // opening, which is G6's original defect wearing a new hat.
                Some(at) => builder.with_position(at),
                // First open of the session. Where that is depends on whether
                // the caller expressed a preference — see
                // `placement::opening`, which owns the whole of that decision
                // and is where A16c was fixed.
                None => match placement::app_window(ctx) {
                    Some(app) => builder.with_position(placement::opening(
                        app,
                        self.default_size,
                        self.preferred,
                    )),
                    // egui has not been told where the application window is,
                    // which happens on the first frame and in a headless
                    // harness. Letting the platform place it is the right
                    // answer and not a fallback: it is what every dialog does
                    // when nothing better is known.
                    None => builder,
                },
            };
        }

        // A DIALOG THAT OPENS TAKES THE KEYBOARD, and it stopped doing
        // that the day it became an OS window.
        //
        // In the embedded era a dialog drew inside the application's window, so
        // it inherited that window's focus and `request_focus()` on its first
        // field was the whole of the job. An OS window has focus of its own,
        // and whether the platform grants it on creation is **not reliable**:
        // Windows refuses to hand the foreground to a process that does not
        // currently have it, silently, which is the same rule
        // `tools/ui-verify` documents at length about `SetForegroundWindow`.
        //
        // The observable cost is exact. `text_annot_takes_the_keyboard_unclicked`
        // types into the note dialog **without clicking it**, *"the way an
        // operator does"*, and after the conversion the characters went to the
        // page instead: the Accept control is gated on the field being
        // non-empty, so it stayed disabled and pressing it authored nothing.
        // The operator's version of that is *"I dragged out a note box and
        // typing did nothing."*
        //
        // ONLY ON THE FRAME IT OPENS. A `Focus` command sent every frame
        // would seize the foreground back from anything the operator switched
        // to while the dialog was open — including another application — which
        // is the behaviour of the worst software on the machine. The pass
        // number of the last frame this dialog drew tells an opening from a
        // continuation; a gap of more than one frame means it was closed and
        // reopened.

        let mut frame = Frame {
            // `EmbeddedWindow`, not `Root`, as the value before egui
            // answers. It is the CONSERVATIVE default: it claims the fallback
            // rather than the OS window, so a path that somehow never reaches
            // the callback reports "G1 did not happen" instead of asserting it
            // did. A default that over-claims is how a gate goes green on a
            // build that regressed.
            class: ViewportClass::EmbeddedWindow,
            closed: false,
        };
        let result = ctx.show_viewport_immediate(self.id, builder, |ui, class| {
            frame.class = class;
            let child = ui.ctx().clone();

            // Remember where the OS has put it, every frame, so a drag is
            // captured without a drag handler. `inner_rect` is desktop
            // coordinates; `with_position` takes the OUTER position, so the
            // outer rect is what is stored — using the inner one would walk the
            // window up-left by the title bar's height on every reopen.
            if class == ViewportClass::Immediate {
                let (outer, inner) =
                    child.input(|i| (i.viewport().outer_rect, i.viewport().inner_rect));
                if let Some(outer) = outer {
                    self.remember(&child, outer.min);
                }
                // The child's own client rectangle, in DESKTOP coordinates,
                // for the harness. See the module header: every `ui-rect` this
                // dialog publishes is relative to THIS origin and not to the
                // application window's, and the two are plausible-looking
                // numbers that differ by hundreds of pixels.
                if let Some(inner) = inner {
                    crate::diag::viewport_inner(self.id, inner);
                }
            }

            // Every `ui-rect` this dialog publishes is tagged with THIS
            // viewport for the rest of the callback. See
            // `crate::diag::ViewportScope`: without it the harness reads the
            // dialog's rectangles as if they were the application window's,
            // and they are plausible numbers naming a different place on the
            // desktop.
            let _regions = crate::diag::ViewportScope::enter(self.id);

            // OWNED BY THE APPLICATION WINDOW. See the module header's G3
            // section: this is what makes the dialog stay in front of the
            // window it belongs to AND keep the keyboard, neither of which a
            // request can guarantee.
            //
            // Attempted every frame, deliberately, and cheap by construction:
            // the call is idempotent and returns early when the relationship
            // already holds. There is no "the viewport was just created" event
            // to hang it on — the platform window comes into existence DURING a
            // frame, so the first attempt after a dialog opens can legitimately
            // find nothing, and the honest shape is to try again next frame
            // rather than to guess how many frames to wait.
            if class == ViewportClass::Immediate
                && let Some(owner) = owner
            {
                let owned = native_window::own_window(owner, &self.title);
                crate::diag::trace_on_change("dialog-owned", || {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    format!("title={:?} owned={owned}", self.title)
                });
            }

            // See the note above `opening`, and [`ENGAGED`].
            if class == ViewportClass::Immediate {
                // ONLY AN OPERATOR'S OWN EVENTS COUNT, and the first
                // version of this test said `!i.events.is_empty()` — which is
                // true on almost every pass, because a viewport receives
                // `WindowFocused`, pointer motion and screen-rect changes it
                // never asked for. The dialog marked itself engaged
                // immediately and stopped asking for the keyboard on the pass
                // after it opened, which is the defect this rule exists to
                // prevent, reintroduced by its own guard.
                let used = child.input(|i| {
                    i.events.iter().any(|e| {
                        matches!(
                            e,
                            egui::Event::Key { pressed: true, .. }
                                | egui::Event::Text(_)
                                | egui::Event::PointerButton { pressed: true, .. }
                        )
                    })
                });
                if used {
                    // The operator has used this window. Stop asking, for good.
                    child.data_mut(|d| d.insert_temp(self.engaged_key, ENGAGED));
                } else if opening {
                    child.send_viewport_cmd_to(self.id, egui::ViewportCommand::Focus);
                    // THE ASK, which is a different fact from the grant the
                    // `dialog-focus` line below reports. One line per frame for as long
                    // as the window is inside its opening grace period and the operator
                    // has not yet touched it, so the two read together say whether the
                    // platform answered and after how many frames. That is the only way
                    // to tell *focus was never requested* from *focus was requested and
                    // refused* — which look identical from the operator's chair and have
                    // opposite fixes.
                    //
                    crate::diag::trace(|| {
                        // ui-text-exempt: diagnostic trace, never displayed.
                        format!(
                            "dialog-refocus title={:?} now={now} opened_at={opened_at}",
                            self.title
                        )
                    });
                }
            }
            // Whether the PLATFORM has given this window the keyboard, which
            // is a different fact from whether it was asked to and the only one
            // a driven check can act on. A dialog that never reports `true` is
            // a dialog an operator has to click before typing — the thing the
            // conversion to an OS window must not have cost.
            crate::diag::trace_on_change("dialog-focus", || {
                // ui-text-exempt: diagnostic trace, never displayed.
                let focused = child.input(|i| i.viewport().focused);
                // ui-text-exempt: diagnostic trace, never displayed.
                format!("title={:?} focused={focused:?}", self.title)
            });

            // G4's Escape half, read from the CHILD's input. Reading the
            // parent's would answer about a key pressed into the application
            // window, which is a different window and, once G3 lands, a
            // different focus.
            //
            // **The question is asked of the PREVIOUS pass, because egui has
            // already destroyed the answer by the time this line runs.**
            // `Focus::begin_pass` clears `focused_widget` on Escape before any
            // widget code executes, so `child.text_edit_focused()` is `false`
            // here on exactly the frame where it matters. A guard written the
            // obvious way is not a weak guard; it is inert, and the dialog
            // cancels on the same press that leaves the field — up to a text
            // annotation's worth of typing gone on one key.
            //
            // So the rung is: **first Escape leaves the field, second cancels
            // the dialog**, which is what Acrobat, Word and every options window
            // in Windows do, and therefore the spec under this project's
            // use-the-conventional-interaction rule.
            //
            // The flag is not written on an Escape pass. egui may run this
            // callback more than once per frame, and a flag cleared by the first
            // run would let the second run of the *same* press close the window
            // — the grace press consumed and spent inside one keystroke.
            //
            // typing-guard-exempt: this asks whether a WIDGET held the keyboard,
            // not whether anybody is composing. A canvas draft is not reachable
            // from inside a dialog.
            let field_was_focused = child
                .data(|d| d.get_temp::<bool>(self.field_focus_key()))
                .unwrap_or(false);
            let escape = child.input(|i| i.key_pressed(egui::Key::Escape));
            // Enter has the same rung, for the same reason: a single-line
            // field or a `DragValue` surrenders focus on the Enter that
            // commits it, so by the time the footer asks, the field has let
            // go. Handed to [`Self::footer`] through the child's memory.
            let enter = child.input(|i| i.key_pressed(egui::Key::Enter));
            let grace = enter_grace_key(&child);
            child.data_mut(|d| d.insert_temp(grace, field_was_focused));
            frame.closed =
                (escape && !field_was_focused) || child.input(|i| i.viewport().close_requested());

            //
            // The `Ui` egui hands a viewport callback is the child window's
            // ROOT — the same position `eframe::App::ui` occupies for the main
            // window — and nothing has painted it. In the application window
            // `app::frame` adds a `CentralPanel`, which fills it; a dialog had
            // no such thing, so every one of the thirteen converted that day
            // rendered its controls over the **clear colour**: dark text on
            // near-black, legible only as an outline.
            //
            // It is invisible to every non-pixel oracle. `viewport-inner` was
            // published, every `ui-rect` was declared, the driven check that
            // asserts *"a dialog opens in its own OS window"* passed on all
            // eight — and a screenshot showed a black rectangle. That is
            // `D:/dev/rag/egui/`'s standing rule arriving again: **a layout or
            // rendering defect has exactly one oracle, and it is a rendered
            // screenshot.**
            ui.painter()
                .rect_filled(ui.max_rect(), 0.0, ui.visuals().panel_fill);
            //
            // His words, about Print: *"the print button that is so far off in
            // the corner it is touching the edge the window."* It was, and so
            // was everything else: the trace showed `print.properties` declared
            // at `y = 0.0`, i.e. the first control flush against the top of the
            // client area.
            //
            // The `Ui` egui hands a viewport callback is the child window's
            // ROOT, whose `max_rect` is the whole client area. In the main
            // window `app::frame` adds a `CentralPanel`, and a `CentralPanel`
            // brings `Frame::central_panel`'s inner margin with it — which is
            // why nothing in the application looked like this and every dialog
            // did. The background paint above was added when that difference
            // was first noticed; it fixed the colour and left the geometry.
            //
            // It is applied HERE rather than in each dialog, because
            // fourteen dialogs applying their own margin is fourteen chances to
            // pick a different number and one guarantee that somebody forgets.
            // The host already owns the window, the background and the button
            // pair for exactly that reason.
            //
            // `Frame::NONE` with only an inner margin, not
            // `Frame::window` or `central_panel`: those bring a fill and a
            // stroke, and the fill would paint over the background this function
            // just established while the stroke would draw a second border
            // inside the OS window's own. The margin is the only part wanted.
            //
            // AND IT MUST NOT REACH `fit`. `Self::fit` grows the window to
            // its content, and its doc comment records a run in which an added
            // margin turned that into an unbounded growth loop — R128's shape,
            // met three times in this project. A `Frame`'s `inner_margin`
            // enlarges the `min_rect` it returns by exactly the margin, every
            // frame, so feeding that back would grow the window by 16 pt per
            // frame for ever. The measurement below therefore takes the INNER
            // ui's `min_rect`, captured before the frame closes, and adds the
            // margin ONCE as a constant — a constant, not a measurement, which
            // is the distinction that makes it safe.
            let margin = Self::BODY_MARGIN_PTS;
            let framed = egui::Frame::NONE.inner_margin(egui::Margin::same(margin as i8));
            // Called directly. See this function's signature for why there is
            // no longer an `Option` and a `take()` here: egui may run this
            // callback more than once per frame, and the right answer to a
            // second run is to draw again.
            let inner = framed.show(ui, |ui| (add(ui), ui.min_rect().size()));
            let (out, content) = inner.inner;

            // Recorded for the NEXT pass's Escape, per the rung above. Asked
            // after the body has drawn, which is the only moment in an
            // immediate-mode toolkit at which a field that has the keyboard has
            // said so. Read `field_focused` out before taking the write lock:
            // `text_edit_focused` reads the same memory `data_mut` holds.
            //
            // typing-guard-exempt: this asks whether an egui WIDGET held the
            // keyboard — the fact the next pass's Escape needs in order to
            // choose between leaving the field and cancelling the window. It is
            // not a question about whether the operator is composing: a canvas
            // text draft is not reachable from inside a dialog, and `composing`
            // would answer about the application window rather than this one.
            if !escape && !enter {
                let field_focused = child.text_edit_focused();
                child.data_mut(|d| d.insert_temp(self.field_focus_key(), field_focused));
            }

            // Measured AFTER the body has drawn, which is the only moment
            // the answer exists in an immediate-mode toolkit. See [`Self::fit`]
            // for the two guards that keep this from becoming a feedback loop.
            if class == ViewportClass::Immediate {
                // The CONTENT's own size plus the margin twice, as a
                // constant. NOT `ui.min_rect()` of the outer ui, which already
                // includes the margin and would therefore be a measurement
                // containing the thing being added — see above.
                self.fit(&child, content + egui::vec2(margin * 2.0, margin * 2.0));
            }
            out
        });
        (frame, result)
    }

    /// **Put the body in its own scrolling space and pin the footer to the
    /// bottom of the window**, so the buttons are reachable at any size.
    pub fn scrolled<S, R>(
        ui: &mut egui::Ui,
        state: &mut S,
        body: impl FnOnce(&mut egui::Ui, &mut S),
        footer: impl FnOnce(&mut egui::Ui, &mut S) -> R,
    ) -> R {
        // The footer FIRST. `Panel::bottom` takes its height out of
        // the ui's rectangle before anything else is laid out, which is the
        // whole of the guarantee: whatever the body does afterwards, it is
        // working inside what is left over.
        //
        // `id_salt` from the ui rather than a constant: two dialogs are two
        // viewports, but a panel id must still be unique within the context
        // that hosts them, and a constant here would collide the day two
        // dialogs are open at once — which this shell allows.
        let out = egui::Panel::bottom(ui.id().with("dialog-footer"))
            .resizable(false)
            // The `ui.separator()` below is the rule the eye reads; a panel
            // separator as well would draw two.
            .show_separator_line(false)
            // No fill and no stroke. The window's background is already painted
            // by `show`, and a panel frame here would draw a second surface
            // inside the OS window's own border.
            .frame(egui::Frame::NONE.outer_margin(egui::Margin {
                top: 8,
                ..egui::Margin::ZERO
            }))
            .show(ui, |ui| {
                ui.separator();
                footer(ui, state)
            })
            .inner;
        egui::ScrollArea::vertical()
            // `auto_shrink` on the Y axis so a short body does not leave the
            // footer floating halfway down an over-tall window; off on X so the
            // body still gets the full width to wrap into.
            .auto_shrink([false, true])
            .show(ui, |ui| body(ui, state));
        out
    }

    /// **Draw a dialog's affirmative and cancelling buttons**, with Enter and
    /// Escape wired and the default drawn as the default.
    pub fn buttons(ui: &mut egui::Ui, accept: &str, cancel: &str) -> (bool, bool) {
        let (accepted, cancelled, _) = Self::footer(ui, (accept, ""), (cancel, ""), None);
        (accepted, cancelled)
    }

    /// **Every route out of a dialog**, laid out right-to-left, with Enter
    /// wired to the affirmative button and every button's rectangle published
    /// for the driven harness.
    pub fn footer(
        ui: &mut egui::Ui,
        accept: (&str, &str),
        cancel: (&str, &str),
        keep: Option<(&str, &str)>,
    ) -> (bool, bool, bool) {
        let ctx = ui.ctx().clone();
        // THIS ASKS WHETHER A WIDGET IN THIS DIALOG HOLDS THE KEYBOARD, not
        // whether the operator is composing anywhere in the application, and
        // the two genuinely differ here.
        //
        // `pdfcer_gui::canvas::textedit::composing` - the predicate this gate
        // normally requires - answers `true` while a canvas draft is live, and
        // a canvas draft SURVIVES the opening of a dialog: it is committed by
        // clicking away on the page, not by a print window appearing. So using
        // it would mean that an operator who had a caret on the page, opened
        // Print and pressed Enter got nothing, with no surface saying why -
        // which is `dialogs.md` G4's stated failure mode reintroduced by the
        // guard against a different one.
        //
        // The hazard the gate exists for cannot occur here in either
        // direction. A dialog is a separate OS window with its own keyboard
        // focus, so an Enter arriving in it was aimed at it; and nothing in
        // this function can steal a key from the canvas, because the canvas is
        // not being drawn inside this callback.
        //
        // What IS wanted is the half `text_edit_focused` answers: a multi-line
        // field inside the dialog must keep the ability to type a newline. See
        // this function's own docs for the single-line case, which is
        // deliberately given up rather than guessed at.
        //
        // typing-guard-exempt: the four paragraphs above are the reason. In one
        // line: a dialog is a separate OS window with its own focus, so
        // "somebody is composing on the canvas" is not a fact about this key.
        let grace = enter_grace_key(&ctx);
        let field_had_it = ctx.data(|d| d.get_temp::<bool>(grace)).unwrap_or(false);
        let enter = !ctx.text_edit_focused()
            && !field_had_it
            && ctx.input(|i| i.key_pressed(egui::Key::Enter));

        let mut accepted = false;
        let mut cancelled = false;
        let mut kept = false;
        // Laid right to left, so the first button drawn is the rightmost.
        // Windows puts the affirmative first: [Accept] [Keep] [Cancel].
        // macOS and the Linux desktops put it last: [Keep] [Cancel] [Accept].
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // The default action is a solid accent plate, never the 27 % canvas
            // selection wash, which made it look disabled (DEFECTS.md D2).
            let mut draw_accept = |ui: &mut egui::Ui| {
                let (fill, text) = egui_shell::Theme::accent_pair(ui.ctx());
                let default =
                    egui::Button::new(egui::RichText::new(accept.0).color(text)).fill(fill);
                let response = explained(ui.add(default), accept.1);
                crate::diag::ui_rect(REGION_ACCEPT, response.rect);
                if response.clicked() || enter {
                    accepted = true;
                }
            };
            let mut draw_cancel = |ui: &mut egui::Ui| {
                let response = explained(ui.button(cancel.0), cancel.1);
                crate::diag::ui_rect(REGION_CANCEL, response.rect);
                if response.clicked() {
                    cancelled = true;
                }
            };
            let mut draw_keep = |ui: &mut egui::Ui| {
                if let Some((label, hover)) = keep {
                    let response = explained(ui.button(label), hover);
                    crate::diag::ui_rect(REGION_KEEP, response.rect);
                    if response.clicked() {
                        kept = true;
                    }
                }
            };
            if cfg!(target_os = "windows") {
                draw_cancel(ui);
                draw_keep(ui);
                draw_accept(ui);
            } else {
                draw_accept(ui);
                draw_cancel(ui);
                draw_keep(ui);
            }
        });
        (accepted, cancelled, kept)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Two dialogs get two windows**, which is the whole reason the id is
    /// derived from a caller-supplied string rather than counted.
    #[test]
    fn each_dialog_gets_its_own_viewport() {
        let a = Host::new("print", "Print", Vec2::splat(100.0), Vec2::splat(10.0));
        let b = Host::new(
            "insert-image",
            "Insert image",
            Vec2::splat(100.0),
            Vec2::splat(10.0),
        );
        assert_ne!(a.id, b.id);
    }

    /// …and the same dialog gets the same window every time, so a reopen is the
    /// same window rather than a second one beside it.
    #[test]
    fn one_dialog_keeps_one_viewport_across_constructions() {
        let a = Host::new("print", "Print", Vec2::splat(100.0), Vec2::splat(10.0));
        let b = Host::new("print", "Print", Vec2::splat(900.0), Vec2::splat(90.0));
        assert_eq!(a.id, b.id, "the id must key on the NAME, not on the size");
    }

    /// **A host with nothing remembered reports nothing**, so the first open
    /// is placed rather than restored.
    ///
    #[test]
    fn a_fresh_host_remembers_no_position() {
        let ctx = egui::Context::default();
        let h = Host::new("print", "Print", Vec2::splat(100.0), Vec2::splat(10.0));
        assert!(h.remembered(&ctx).is_none());
    }

    /// …and once it has been told, it answers with what it was told — for
    /// **that dialog only**.
    #[test]
    fn a_position_is_remembered_per_dialog() {
        let ctx = egui::Context::default();
        let print = Host::new("print", "Print", Vec2::splat(100.0), Vec2::splat(10.0));
        let about = Host::new("about", "About", Vec2::splat(100.0), Vec2::splat(10.0));
        print.remember(&ctx, Pos2::new(320.0, 240.0));
        assert_eq!(print.remembered(&ctx), Some(Pos2::new(320.0, 240.0)));
        assert!(
            about.remembered(&ctx).is_none(),
            "one dialog's position must not answer for another's"
        );
    }

    /// **The layout half of the print dialog's runaway, measured in a real
    /// laid-out frame — and it fails on the old ordering.**
    #[test]
    fn a_status_label_after_a_right_to_left_block_overflows_the_row() {
        const GIVEN: f32 = 400.0;

        fn row_width(ctx: &egui::Context, status_first: bool) -> f32 {
            let mut measured = 0.0;
            let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
                ui.set_max_width(GIVEN);
                let r = ui.horizontal(|ui| {
                    if status_first {
                        ui.label("Sent 3 pages");
                    }
                    // The shape `Host::buttons` uses. The layout is what
                    // matters here, not which widgets are inside it.
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let _ = ui.button("Print");
                        let _ = ui.button("Close");
                    });
                    if !status_first {
                        ui.label("Sent 3 pages");
                    }
                });
                measured = r.response.rect.width();
            });
            measured
        }

        let ctx = egui::Context::default();
        // Warm one frame: egui sizes some widgets from the previous pass.
        let _ = row_width(&ctx, false);

        let buttons_first = row_width(&ctx, false);
        let status_first = row_width(&ctx, true);

        assert!(
            buttons_first > GIVEN,
            "the pre-fix ordering must overflow the row it was given (got {buttons_first} in {GIVEN}) — if this does not overflow, the mechanism behind the operator's runaway has changed and the fix below needs re-deriving, not just keeping"
        );
        assert!(
            status_first <= GIVEN + 0.5,
            "with the status drawn first the row must fit exactly the width it was given (got {status_first} in {GIVEN}); anything wider is an overflow that `Host::fit` will chase"
        );
    }
}
