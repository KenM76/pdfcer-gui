//! # `dialogs::host` — a dialog is an OS WINDOW
//!
//! ## The operator's report, 2026-08-20
//!
//! > *"Print dialogue box doesn't pop up in its own movable window. It is
//! > locked within the boundaries of the program's window. Like, I just assume
//! > you've been trained on a million lines of code and software that pops it
//! > up in its own window."*
//!
//! He is right, and the last sentence is the diagnosis. `ui-conventions/dialogs.md`
//! G1 states the rule and states why immediate-mode toolkits get it wrong:
//!
//! > **Why immediate-mode toolkits get this wrong.** Their in-viewport "window"
//! > widget is the path of least resistance — it looks like a dialog and is one
//! > function call. A real OS window needs a viewport/second-window API that is
//! > newer and more awkward. The default is wrong and nothing pushes back.
//!
//! Every retained toolkit makes the OS window the **default** and the inline
//! panel the special case — `QDialog`, `NSWindow`, `ContentDialog` vs `Window`.
//! That default ordering is itself the guidance. This module restores it here:
//! calling [`Host::show`] is now as easy as calling `egui::Window::new`, so the
//! path of least resistance and the right answer are the same path.
//!
//! ## ★ What an OS window buys, concretely
//!
//! Not aesthetics. Four things an operator does with a print dialog:
//!
//! 1. **Move it off the document** to read the page underneath while choosing a
//!    range. An in-viewport window can be dragged to the edge and no further.
//! 2. **Put it on the second monitor**, which is what a two-screen desk is for.
//! 3. **Find it in the taskbar / Alt-Tab** when it has gone behind something.
//! 4. **Resize it past the application window**, which matters for the print
//!    preview specifically: the preview is the point of the dialog and it is
//!    the first thing the 520 pt floor squeezes.
//!
//! ## ★★ It degrades, and that is deliberate rather than incidental
//!
//! `Context::show_viewport_immediate` falls back to an **embedded** window —
//! literally the `egui::Window` this replaces — when the backend has no
//! multi-viewport renderer, which is the case on **web**. `MODES_AND_PANELS.md`
//! records the web fork as a live target, so a dialog host that only worked
//! natively would be a surface that vanishes on one of the two platforms.
//!
//! The fallback is egui's, not ours: one code path, two renderings, and
//! [`Frame::class`] says which so a caller that genuinely must know can ask.
//! Nothing in this module branches on it except the position memory, which has
//! nothing to remember when the OS is not placing the window.
//!
//! ## G4 — Enter accepts, Escape cancels, and the default is VISIBLY the default
//!
//! The second half of the operator's item, and the failure mode is the one
//! everybody has met:
//!
//! > *"The operator types into the last field, presses Enter out of habit, and
//! > nothing happens — or worse, something other than what they expected."*
//!
//! [`Host::buttons`] draws the pair and owns all three obligations, because a
//! caller that had to remember them would forget one:
//!
//! - **Enter** activates the affirmative action — but **not while a text field
//!   has focus and wants the key**, which is why the check asks
//!   `ctx.text_edit_focused()` first. A multi-line field would otherwise lose
//!   the ability to type a newline the moment it sat in a dialog.
//! - **Escape** is equivalent to Cancel *and* to the close button, so all three
//!   routes out are one outcome.
//! - The affirmative button is **drawn** as the default, from the theme's
//!   **accent** and the foreground the theme pairs with it
//!   (`Theme::accent_pair`), so the operator knows what Enter will do before
//!   pressing it. A default nobody can see is not a default; it is a surprise.
//!   ★ This sentence said *"the theme's own selection fill"* until 2026-09-03,
//!   and that was the defect rather than a description of it: `selection_fill`
//!   is a 27 %-opacity canvas tint, so the default button rendered **paler than
//!   an ordinary button** and read as disabled. See [`Host::buttons`].
//!
//! ## G6 — it remembers where it was left
//!
//! Position is held per dialog, keyed on the string [`Host::new`] was given,
//! and re-applied on the next open through `ViewportBuilder::with_position`.
//! **Nothing is remembered in the embedded fallback**, because egui places that
//! window and the OS does not.
//!
//! ★ It is stored in memory rather than on disk, deliberately. A position that
//! survived a restart would have to be validated against the *current* monitor
//! layout — G6 says so in the same breath — and a dialog that opens on a
//! monitor which is no longer attached is worse than one that opens centred.
//! Persisting it is a real feature with a real check to write; the session-long
//! version is the nine-tenths of it that costs nothing and cannot strand
//! anybody.
//!
//! ## ★★ THE HOST OWNS NO STATE, AND THAT IS WHAT MADE THE OTHER THIRTEEN
//! ## DIALOGS ONE LINE EACH
//!
//! The first version of this module kept the remembered position in a `Host`
//! **struct field**, which meant every dialog that wanted an OS window had to
//! grow a `host: Option<Host>` field, a `new_host()` constructor, and a
//! `take()`/put-back dance around the borrow — because `show` needed `&mut` on
//! the host while the closure it was handed needed `&mut` on the dialog.
//!
//! That is fine for one dialog and it is a tax on thirteen. Worse, it made
//! `ShortcutsDialog` — a **unit struct**, deliberately stateless — impossible
//! to convert without giving it state it does not otherwise need.
//!
//! So the position lives in `egui::Memory` keyed on the dialog's own id string,
//! `show` takes `&self`, and a `Host` is a **description** built fresh each
//! frame rather than an object with a lifetime. Three consequences, all wanted:
//!
//! 1. A conversion is one expression: `Host::new(…).show(ctx, |ui| …)`.
//! 2. The memory **survives close-and-reopen**, which is what G6 actually asks
//!    for. The struct version lost the position the moment the dialog closed,
//!    and called that correct because it had no way to be otherwise.
//! 3. There is no second copy of the position to go stale.
//!
//! ★ It is `insert_temp`, so it is session-scoped and never written to disk —
//! see the paragraph above for why that is the feature and not a shortcut.
//!
//! ## What this does NOT fix, said so it is a decision
//!
//! ## G3 — OWNED BY THE APPLICATION WINDOW, as of 2026-08-21
//!
//! This section read *"filed as a gap rather than papered over"* for a day. The
//! gap said: `eframe 0.35`'s `ViewportBuilder` has no owner or parent option —
//! `grep with_` over `egui/src/viewport.rs` returns thirty builders and none of
//! them is one — and `egui-winit` never passes down the parent relationship
//! egui itself tracks in `viewport_parents`. So a dialog could fall behind the
//! main window, which is *the* classic Windows bug.
//!
//! ★★ **What closed it was a second symptom, not a second look.** The dialog
//! also **lost the keyboard a third of a second after opening**, measured with
//! both windows reporting their own focus, with the application asking for none
//! of it. The operator's version: *drag out a note box, type without clicking
//! the field first, and the words go nowhere.* Asking for focus again does not
//! work and that was tried — Windows refuses the foreground to a process that
//! does not already hold it.
//!
//! **Ownership is not a request.** An owned window is by definition above its
//! owner and keeps activation as a property of the relationship. It is what
//! every native dialog on the machine already uses, which is why none of them
//! has either problem. `crate::dialogs::host` now sets it through
//! [`native_window::own_window`], finding the child by title among **this
//! process's** windows — the crate that call lives in exists so that this
//! crate's `#![forbid(unsafe_code)]` survives.
//!
//! ★ `with_always_on_top` remains refused, and the reason is worth keeping now
//! that it is not the only option: this project's own RAG records an
//! always-on-top window swallowing the driven harness's clicks with
//! `SetForegroundWindow` still reporting success. Trading a rare confusion for
//! a class of undiagnosable harness failure was always a bad trade. Ownership
//! gets the z-order guarantee without the input trap.
//! - **G5, focus trapping and tab order.** An OS window gets keyboard focus of
//!   its own, which is most of what G5 asks for and is strictly better than the
//!   in-viewport version had. Ordered tab traversal and a focus trap are not
//!   asserted by anything and are still a gap.
//!
//! ## ★ The diagnostic channel had to learn about viewports, and why
//!
//! `crate::diag::ui_rect` publishes a named region's rectangle so a driven
//! check can aim at a control without guessing. Those rectangles are **relative
//! to the viewport that drew them**, and until this module existed there was
//! only one viewport, so the harness could add the main window's client origin
//! and be right.
//!
//! A dialog in its own window breaks that silently — the coordinates stay
//! plausible and land somewhere else entirely, which is the exact shape of
//! defect `D:/dev/rag/egui/` records twice already. So [`Host::show`] publishes
//! `viewport-inner`, the child's own client rectangle in **desktop**
//! coordinates, and `ui_rect` tags every region it publishes with the viewport
//! that drew it. The harness then has both halves and can convert; it is also
//! the only way a check can *tell* that a dialog opened in its own window,
//! which is what makes G1 assertable rather than a matter of looking.

use egui::{Pos2, Vec2, ViewportBuilder, ViewportClass, ViewportId};

/// How far in from the application window a dialog opens when it has no
/// remembered position.
///
/// Not centred on the parent, and not at the OS's own default. Centring puts a
/// dialog exactly over the thing it is asking about, which is the one place it
/// must not be for a *print* dialog whose preview the operator is comparing
/// against the page behind it. A small inset reads as "this belongs to that
/// window" without covering its middle.
const OPEN_INSET_PT: f32 = 48.0;

/// How much a dialog's body must overflow its window before the window is
/// grown to fit it. See [`Host::fit`], whose first version had no such floor
/// and grew the About window from 560 px to 1,624 px in a few frames.
/// Where the application window's handle is kept for [`Host::show`] to find.
const OWNER_KEY: &str = "dialog-host-owner"; // ui-text-exempt: a memory key, never displayed.

/// **Tell the dialog host which window owns its dialogs.** Called once a frame
/// by the application, before any dialog draws.
///
/// # ★★ Why a channel through `egui::Memory` and not an argument
///
/// Because the alternative is a fourteenth argument on thirteen call sites to
/// carry one fact that never varies. `eframe` hands the application window's
/// handle out **exactly once**, to `PdfcerApp` at start-up, and every dialog in
/// the program wants the same one; threading it by hand would be thirteen
/// opportunities to pass `None` and one dialog that quietly kept G3's
/// symptoms.
///
/// ★ It is safe as a hidden channel for the reason most hidden channels are
/// not: **it has exactly one writer**, `app::frame`, on the frame path, and the
/// value is a constant for the life of the process. There is no ordering to get
/// wrong and no second producer to disagree with.
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

const FIT_MARGIN: f32 = 8.0;

/// How many times one dialog may be grown to fit its content before the host
/// concludes the measurement is circular and stops.
///
/// See the growth budget in [`Host::fit`]. The legitimate case settles in one
/// round trip and two covers a body that re-flows in response to the first;
/// three is one more than has ever been needed, and it is the difference
/// between a bounded nuisance and a window that grows for as long as it is
/// open.
const FIT_BUDGET: usize = 3;

/// **The size a window should be grown to**, or `None` to leave it alone.
///
/// # Why this is a free function
///
/// Because the growth branch could not otherwise be tested. `Host::fit` needs
/// a live viewport to read `inner_rect` and to issue the resize, so the whole
/// decision was reachable headlessly only in its no-op half — and the adjacent
/// test said as much in its own doc comment: *"only the no-op half is
/// reachable headlessly … the growing branch is asserted by the driven check,
/// which is the only place it can be."*
///
/// ★ That was true of the *resize*, and it was never true of the *arithmetic*.
/// Splitting them costs one function and buys the convergence test that would
/// have caught the print dialog's runaway before an operator did: feed this
/// its own output and it must reach a fixed point.
///
/// # The contract
///
/// * `None` when the content already fits within [`FIT_MARGIN`] on both axes.
///   The margin is a floor on what is worth acting on — below it the
///   difference is noise between `min_rect` and a client size the window
///   manager reports, and acting on noise is what creep is made of.
/// * Otherwise a size that is **never smaller than the current window** on
///   either axis, and never smaller than `min_size`. Growth only: shrinking to
///   content would fight the operator every time they enlarged a window, and
///   would shrink a scrollable body to its own scroll viewport, which is
///   circular by construction.
///
/// ★★ Note what this function cannot do, and why the budget in [`Host::fit`]
/// exists as well. Its answer is **idempotent** — feed it a window that has
/// already been grown to its content and it returns `None` — but idempotence
/// only holds if `content` stays put when the window changes. When the content
/// is measured *from* the window, every answer is new and correct in
/// isolation, and the sequence still runs away. No pure function of
/// `(inner, content)` can detect that; only a count of how often it has been
/// asked can.
fn fit_target(inner: Vec2, content: Vec2, min_size: Vec2) -> Option<Vec2> {
    if content.x <= inner.x + FIT_MARGIN && content.y <= inner.y + FIT_MARGIN {
        return None;
    }
    Some(Vec2::new(
        content.x.max(inner.x).max(min_size.x),
        content.y.max(inner.y).max(min_size.y),
    ))
}

/// How many passes from opening a dialog goes on asking for the keyboard.
///
/// # ★★★ Measured, twice, and the second measurement moved it
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
/// # ★★★ THE NUMBER STAYED AT EIGHT, AND THE HUNT THAT NEARLY CHANGED IT IS
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
/// ★ Two things are worth carrying out of that:
///
/// - **A knob must not sit at a value chosen to fix something it does not
///   fix.** Left at a hundred and twenty, a future reader would have believed
///   the problem was tuning, and a dialog would have re-seized the foreground
///   for two seconds after every opening for no benefit at all.
/// - **A measurement that moves with the knob is not proof the knob is the
///   subject.** Focus really did follow the requests; the requests really were
///   irrelevant to the failure. Both were true at once.
///
/// ★ The bound is not the only guard, and on its own it would be a bad one —
/// see [`ENGAGED`]. A dialog stops asking the instant the operator touches it,
/// so the only case this can fight is *clicking away within half a second of a
/// dialog appearing without having interacted with it*, whose worst outcome is
/// the dialog coming back to the front once.
const FOCUS_FRAMES: u64 = 8;

/// Marker written when a dialog first receives input of its own.
///
/// ★★ **The real terminator of the focus request**, with [`FOCUS_FRAMES`] as
/// its backstop rather than the other way round. *"Keep asking for the keyboard
/// until the operator has used this window"* is the rule that matches intent;
/// a pass count is only there so a dialog nobody touches stops asking.
///
/// Once set it is never cleared for the life of that opening, so a dialog the
/// operator clicked into and then left never re-seizes the foreground.
const ENGAGED: bool = true;

/// One dialog's window: what it is called, how big it opens, and where the
/// operator last left it.
///
/// Held by the dialog it belongs to, so its lifetime is the dialog's — which is
/// what makes the position memory correct without anything having to clear it.
/// A dialog that is closed and reopened gets a fresh `Host` and therefore opens
/// where the platform puts it; a dialog that stays open across frames keeps the
/// position it has been dragged to.
pub struct Host {
    /// The viewport id, stable for this dialog across frames.
    ///
    /// ★ Derived from a caller-supplied string rather than counted, because
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

impl Host {
    /// The padding between a dialog's content and its window edge, in points.
    ///
    /// # Why a constant, and why it lives on the host
    ///
    /// The operator's 2026-09-03 report — *"the print button that is so far off
    /// in the corner it is touching the edge the window"* — was true of all
    /// fourteen dialogs, because the `Ui` egui hands a viewport callback is the
    /// window's root and nothing pads it. The main window never showed it: its
    /// `CentralPanel` brings egui's own inner margin.
    ///
    /// One number, owned here, so that fourteen dialogs cannot pick fourteen
    /// values and so that nobody has to remember to pad theirs.
    ///
    /// ★ 12 pt rather than egui's default 8: this shell's own `Metrics` use
    /// `panel_padding` of 8-12 depending on preset, and a **dialog** is the one
    /// surface where the window edge is a hard boundary rather than a seam onto
    /// the next panel. Windows' own dialogs are roomier at the frame than at
    /// internal gutters for the same reason.
    ///
    /// ★★ It is deliberately NOT read from `Theme::of(ctx).metrics`, and that
    /// is a real decision. This value is fed into [`Self::fit`], which sizes the
    /// window; a metric that changes with the preset would change the window
    /// size on a theme switch, and `fit` is the function whose doc comment
    /// records an unbounded growth loop. A constant cannot participate in a
    /// feedback loop.
    const BODY_MARGIN_PTS: f32 = 12.0;

    /// A dialog window.
    ///
    /// `id` must be unique and stable per dialog — `"print"`, `"insert-image"`.
    /// It keys the OS window, and it is also what the diagnostic channel
    /// publishes, so a driven check names the same string the code does.
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
        }
    }

    /// Where this dialog was last left, in desktop coordinates.
    fn remembered(&self, ctx: &egui::Context) -> Option<Pos2> {
        ctx.data(|d| d.get_temp::<Pos2>(self.key))
    }

    /// Record where the OS has put this dialog.
    fn remember(&self, ctx: &egui::Context, at: Pos2) {
        ctx.data_mut(|d| d.insert_temp(self.key, at));
    }

    /// **Grow the window until the body fits**, at most once per size.
    ///
    /// # ★★ Why this exists: `.resizable(false)` was a SIZE, and an OS window
    /// # has to be given one
    ///
    /// Nine of the thirteen dialogs converted on 2026-08-21 were
    /// `egui::Window::…resizable(false)` with **no** `default_size`, which
    /// means egui sized them to their content every frame. There is no number
    /// written down anywhere for how big those dialogs are — the layout *is*
    /// the number.
    ///
    /// An OS window must be created at some size, so a naive conversion means
    /// **guessing thirteen numbers**, and a guess that is too small does not
    /// look wrong: it clips the bottom of the dialog, which on a confirmation
    /// is the row with the buttons on it. That is exactly the class of defect
    /// `D:/dev/rag/egui/` records as *"panels that shipped unreachable in real
    /// builds with every gate green"*.
    ///
    /// So the window is created at a stated size and then **asks the content
    /// how big it actually is**, growing to fit. The stated size stops being a
    /// promise and becomes an opening bid.
    ///
    /// # ★★★ It only ever GROWS, it grows by a MEANINGFUL amount, and it
    /// # never asks twice for the same size
    ///
    /// Three guards, and every one of them is here because of R128 — the
    /// fit-zoom feedback loop this project has already been bitten by, where a
    /// measurement fed a size that changed the measurement.
    ///
    /// **The first version of this function had that exact defect, and a driven
    /// run found it in one launch.** It padded the measured content by an item
    /// spacing before comparing — so `want` was always larger than `inner`,
    /// every frame asked for eight more pixels than the last, and the
    /// once-per-size guard did not help because *every* size was a new one.
    /// The About window opened at 560 x 480 and was 1624 x 746 by the time the
    /// trace was read. Monotonic creep is a loop; a guard that only stops
    /// *repetition* does not stop it.
    ///
    /// 1. **Grow only.** Shrinking to content would fight the operator every
    ///    time they enlarged a window, and would shrink a scrollable body to
    ///    its own scroll viewport, which is circular by construction.
    /// 2. **Grow by something worth growing by.** [`FIT_MARGIN`] is the floor
    ///    on how much overflow is worth a resize. Below it the difference is
    ///    measurement noise between `min_rect` and a client size the window
    ///    manager reports, and acting on noise is what creep is made of.
    /// 3. **Never ask twice for the same size**, so a body that genuinely does
    ///    respond to its window settles after one round trip instead of
    ///    oscillating for the life of the dialog.
    ///
    /// ★ The content is measured RAW, with nothing added. A margin added here
    /// is indistinguishable from real overflow, which is the whole of the bug
    /// above: the padding an eye would want belongs in the *layout*, not in the
    /// question "is the layout bigger than its window".
    ///
    /// ★ A scrollable body cannot trigger this at all: a `ScrollArea` reports
    /// the size it was *given*, not the size of what is inside it. That is why
    /// the print dialog — the one dialog that already had a measured size and a
    /// scrollbar — is unaffected by a mechanism written for the other twelve.
    fn fit(&self, child: &egui::Context, content: Vec2) {
        let Some(inner) = child.input(|i| i.viewport().inner_rect).map(|r| r.size()) else {
            return;
        };
        let Some(want) = fit_target(inner, content, self.min_size) else {
            return;
        };
        if child.data(|d| d.get_temp::<Vec2>(self.fit_key)) == Some(want) {
            return;
        }

        // ★★★ THE GROWTH BUDGET — the guard that turns a layout mistake into a
        // stopped dialog instead of one that grows without limit.
        //
        // Added 2026-08-25 after the third instance of R128's shape in this
        // project, and the first one an operator had to report: the print
        // dialog's footer overflowed its row by a fixed width every frame, so
        // every requested size was NEW, the once-per-size guard was satisfied
        // every time, and the window grew in steps for as long as it was open.
        //
        // ★ The point that took three instances to learn: **a guard against
        // repetition is not a guard against monotonic creep.** Creep never
        // repeats. Anything that only asks *"have I asked for this before?"*
        // is blind to it by construction, and so is anything that only asks
        // *"is the difference big enough to be real?"* — the step here was a
        // whole label wide and entirely real. The only property that separates
        // a legitimate fit from a loop is HOW MANY TIMES it happens.
        //
        // The legitimate case is bounded and small, and the doc above says so
        // in its own terms: the window opens at a stated bid, measures its
        // content once, and settles "after one round trip". Two rounds covers
        // a body that re-flows in response to the first. [`FIT_BUDGET`] is
        // three, which is one more than has ever been needed.
        //
        // Exceeding it is not recoverable by trying harder, so the dialog
        // stops resizing and keeps whatever size it reached — a window that is
        // slightly too small for its content is a nuisance the operator can
        // fix with the mouse, and a window that grows for ever is not.
        // ★ It is also RECORDED rather than merely suppressed: silently
        // capping would leave the underlying layout defect invisible, which is
        // how a bounded bug survives to become somebody else's afternoon.
        let spent = child
            .data(|d| d.get_temp::<usize>(self.budget_key))
            .unwrap_or(0);
        if spent >= FIT_BUDGET {
            if spent == FIT_BUDGET {
                child.data_mut(|d| d.insert_temp(self.budget_key, spent + 1));
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    format!(
                        "dialog-fit-runaway title={:?} budget={FIT_BUDGET} at={:.0}x{:.0} wanted={:.0}x{:.0} \
                         (content is being measured from the window it sets — a layout defect, not a size)",
                        self.title, inner.x, inner.y, want.x, want.y
                    )
                });
            }
            return;
        }
        child.data_mut(|d| {
            d.insert_temp(self.budget_key, spent + 1);
            d.insert_temp(self.fit_key, want);
        });
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "dialog-fit title={:?} from={:.0}x{:.0} to={:.0}x{:.0}",
                self.title, inner.x, inner.y, want.x, want.y
            )
        });
        child.send_viewport_cmd_to(self.id, egui::ViewportCommand::InnerSize(want));
    }

    /// **Draw one frame of this dialog in its own OS window.**
    ///
    /// `add` is handed a `Ui` inside the window and may do anything an
    /// `egui::Window`'s closure could. What it returns comes back untouched, so
    /// a dialog that computes something while drawing does not need a field to
    /// carry it out.
    ///
    /// # ★ Why the close signal comes back rather than through an `&mut bool`
    ///
    /// `egui::Window::open(&mut bool)` is the idiom this replaces, and it has a
    /// property worth losing: the flag is written *during* the draw, so a
    /// caller reading it afterwards cannot tell whether the operator closed the
    /// window or the caller's own code did. [`Frame::closed`] is a report about
    /// this frame only, and the caller decides what closing means — which for
    /// a dialog that is mid-transaction is not always "stop".
    pub fn show<R>(&self, ctx: &egui::Context, add: impl FnOnce(&mut egui::Ui) -> R) -> (Frame, R) {
        let owner = owner(ctx);
        // ★ `show_viewport_immediate` takes `FnMut`, because egui reserves the
        // right to call a viewport's callback more than once. `add` is `FnOnce`
        // — the honest signature for a dialog body, which draws once per frame
        // and may consume what it captures — so it is moved into an `Option`
        // and taken. A second call would `expect` here rather than silently
        // drawing nothing, because "the dialog was blank" is a symptom nobody
        // could trace back to this line.
        let mut add = Some(add);
        let mut builder = ViewportBuilder::default()
            .with_title(self.title.clone())
            .with_inner_size(self.default_size)
            .with_min_inner_size(self.min_size)
            // ★ No maximize and no minimize. A dialog is one transaction; the
            // operator finishes it or abandons it, and a minimised dialog is a
            // transaction that has been left open with no surface saying so.
            // Every platform's dialog chrome makes the same choice.
            .with_minimize_button(false)
            .with_maximize_button(false)
            // It IS in the window list, deliberately, and that is the half of
            // the operator's report that a borderless window would not fix:
            // *"find it when it has gone behind something"*. With G3
            // unavailable (see the module header) this is the only route back
            // to a dialog that has fallen behind the parent.
            .with_taskbar(true);
        // ★★ A SHORT WINDOW, not a single frame, and the reason is measured.
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
            // ★ A gap means it was closed and reopened: a fresh opening. The
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
        }
        ctx.data_mut(|d| d.insert_temp(self.seen_key, (opened_at, last)));
        let engaged = ctx.data(|d| d.get_temp::<bool>(self.engaged_key)) == Some(ENGAGED);
        let opening = !engaged && now.saturating_sub(opened_at) < FOCUS_FRAMES;
        // The very first pass of this opening. See the position clause below.
        let placing = now == opened_at;

        // ★★★ A POSITION IS ASSERTED ONCE, ON THE PASS THE DIALOG OPENS, and
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
        // ★ Two things that costs, and the second is the one that was hunted
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
                // G6: back where it was left, including across a close.
                Some(at) => builder.with_position(at),
                // First open of the session: inset from the application window
                // rather than centred on it — see `OPEN_INSET_PT`.
                None => match ctx.input(|i| i.viewport().outer_rect) {
                    Some(parent) => builder.with_position(parent.min + Vec2::splat(OPEN_INSET_PT)),
                    // No parent rect means egui has not been told where the
                    // application window is, which happens on the first frame
                    // and in a headless harness. Letting the platform place it
                    // is the right answer and not a fallback: it is what every
                    // dialog does when nothing better is known.
                    None => builder,
                },
            };
        }

        // ★★★ A DIALOG THAT OPENS TAKES THE KEYBOARD, and it stopped doing
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
        // ★ ONLY ON THE FRAME IT OPENS. A `Focus` command sent every frame
        // would seize the foreground back from anything the operator switched
        // to while the dialog was open — including another application — which
        // is the behaviour of the worst software on the machine. The pass
        // number of the last frame this dialog drew tells an opening from a
        // continuation; a gap of more than one frame means it was closed and
        // reopened.

        let mut frame = Frame {
            // ★ `EmbeddedWindow`, not `Root`, as the value before egui
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

            // ★ Remember where the OS has put it, every frame, so a drag is
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
                // ★★ The child's own client rectangle, in DESKTOP coordinates,
                // for the harness. See the module header: every `ui-rect` this
                // dialog publishes is relative to THIS origin and not to the
                // application window's, and the two are plausible-looking
                // numbers that differ by hundreds of pixels.
                if let Some(inner) = inner {
                    crate::diag::viewport_inner(self.id, inner);
                }
            }

            // ★★ Every `ui-rect` this dialog publishes is tagged with THIS
            // viewport for the rest of the callback. See
            // `crate::diag::ViewportScope`: without it the harness reads the
            // dialog's rectangles as if they were the application window's,
            // and they are plausible numbers naming a different place on the
            // desktop.
            let _regions = crate::diag::ViewportScope::enter(self.id);

            // ★★★ OWNED BY THE APPLICATION WINDOW. See the module header's G3
            // section: this is what makes the dialog stay in front of the
            // window it belongs to AND keep the keyboard, neither of which a
            // request can guarantee.
            //
            // ★ Attempted every frame, deliberately, and cheap by construction:
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
                // ★ ONLY AN OPERATOR'S OWN EVENTS COUNT, and the first
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
                    crate::diag::trace(|| {
                        // ui-text-exempt: diagnostic trace, never displayed.
                        format!(
                            "TMPASK title={:?} now={now} opened_at={opened_at}",
                            self.title
                        )
                    });
                }
            }
            // ★ Whether the PLATFORM has given this window the keyboard, which
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
            // typing-guard-exempt: this asks whether a WIDGET holds Escape, not
            // whether anybody is composing. A canvas draft is not reachable from
            // inside a dialog.
            let escape =
                !child.text_edit_focused() && child.input(|i| i.key_pressed(egui::Key::Escape));
            frame.closed = escape || child.input(|i| i.viewport().close_requested());

            // egui may in principle call a viewport's callback more than once
            // in a frame; a dialog body is `FnOnce` and may consume what it
            // captures, so the second call panics rather than silently drawing
            // nothing. "The dialog was blank" is a symptom nobody could trace
            // back to this line.
            //
            // ui-text-exempt: a panic message for an egui contract violation,
            // never displayed to an operator and never reachable from one.
            // ★★★ THE WINDOW'S OWN BACKGROUND, and its absence was a defect
            // that shipped for an hour on 2026-08-21.
            //
            // The `Ui` egui hands a viewport callback is the child window's
            // ROOT — the same position `eframe::App::ui` occupies for the main
            // window — and nothing has painted it. In the application window
            // `app::frame` adds a `CentralPanel`, which fills it; a dialog had
            // no such thing, so every one of the thirteen converted that day
            // rendered its controls over the **clear colour**: dark text on
            // near-black, legible only as an outline.
            //
            // ★ It is invisible to every non-pixel oracle. `viewport-inner` was
            // published, every `ui-rect` was declared, the driven check that
            // asserts *"a dialog opens in its own OS window"* passed on all
            // eight — and a screenshot showed a black rectangle. That is
            // `D:/dev/rag/egui/`'s standing rule arriving again: **a layout or
            // rendering defect has exactly one oracle, and it is a rendered
            // screenshot.**
            ui.painter()
                .rect_filled(ui.max_rect(), 0.0, ui.visuals().panel_fill);
            // ★★★ THE INNER MARGIN, AND ITS ABSENCE WAS THE OPERATOR'S
            // "TOUCHING THE EDGE" — 2026-09-03.
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
            // ★★ It is applied HERE rather than in each dialog, because
            // fourteen dialogs applying their own margin is fourteen chances to
            // pick a different number and one guarantee that somebody forgets.
            // The host already owns the window, the background and the button
            // pair for exactly that reason.
            //
            // ★ `Frame::NONE` with only an inner margin, not
            // `Frame::window` or `central_panel`: those bring a fill and a
            // stroke, and the fill would paint over the background this function
            // just established while the stroke would draw a second border
            // inside the OS window's own. The margin is the only part wanted.
            //
            // ★★ AND IT MUST NOT REACH `fit`. `Self::fit` grows the window to
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
            let inner = framed.show(ui, |ui| {
                // ui-text-exempt: a panic message for an egui contract violation.
                let draw = add.take().expect("viewport callback ran twice");
                (draw(ui), ui.min_rect().size())
            });
            let (out, content) = inner.inner;

            // ★ Measured AFTER the body has drawn, which is the only moment
            // the answer exists in an immediate-mode toolkit. See [`Self::fit`]
            // for the two guards that keep this from becoming a feedback loop.
            if class == ViewportClass::Immediate {
                // ★ The CONTENT's own size plus the margin twice, as a
                // constant. NOT `ui.min_rect()` of the outer ui, which already
                // includes the margin and would therefore be a measurement
                // containing the thing being added — see above.
                self.fit(&child, content + egui::vec2(margin * 2.0, margin * 2.0));
            }
            out
        });
        (frame, result)
    }

    /// **Draw a dialog's affirmative and cancelling buttons**, with Enter and
    /// Escape wired and the default drawn as the default.
    ///
    /// Returns `(accepted, cancelled)`. Both can be `false`; neither pair of
    /// them is ever `true` together, because Enter and Escape are different
    /// keys and the two buttons are different rectangles.
    ///
    /// # ★ The order is Cancel then Accept, right-aligned
    ///
    /// Which is Windows' order and the order every dialog on this operator's
    /// machine uses. It is not a preference: a button's meaning is learned by
    /// position long before it is read, and a dialog that reverses the pair is
    /// a dialog whose Cancel gets clicked by muscle memory aimed at OK.
    ///
    /// # ★★ Enter is refused while a text field wants it
    ///
    /// `ctx.text_edit_focused()` — the same predicate `canvas::textedit`
    /// enforces one copy of, and `tools/gates/check-typing-guard.sh` fails the
    /// build on a second. Without it, a dialog with a multi-line field would
    /// accept on the first newline the operator typed, which is worse than
    /// having no Enter at all: it commits a half-written transaction.
    ///
    /// A **single-line** field is the case this deliberately gives up. Enter in
    /// a one-line box should accept the dialog, and here it does not, because
    /// egui reports "a text edit has focus" without saying whether it is
    /// multi-line. Recorded as a known limit rather than guessed at — the fix
    /// is per-field and belongs with the field.
    pub fn buttons(ui: &mut egui::Ui, accept: &str, cancel: &str) -> (bool, bool) {
        let ctx = ui.ctx().clone();
        // ★ THIS ASKS WHETHER A WIDGET IN THIS DIALOG HOLDS THE KEYBOARD, not
        // whether the operator is composing anywhere in the application, and
        // the two genuinely differ here.
        //
        // `crate::canvas::textedit::composing` - the predicate this gate
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
        let enter = !ctx.text_edit_focused() && ctx.input(|i| i.key_pressed(egui::Key::Enter));

        let mut accepted = false;
        let mut cancelled = false;
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // ★★★ THE ACCENT, NOT THE SELECTION FILL — and the difference is
            // the operator's 2026-09-03 report about Print.
            //
            // His words: *"it looks greyed out as though it doesn't do anything
            // even when I hit print — but it is working, so after many clicks I
            // checked the printer and of course there was a dozen jobs there."*
            //
            // This used to read `visuals.selection.bg_fill` +
            // `strong_text_color()`, sourced from the theme rather than from a
            // literal — so it satisfied `check-theme-colors.sh` and looked
            // correct in review. **It was the right rule applied to the wrong
            // role.** `egui-shell`'s theme sets
            //
            //     v.selection.bg_fill = p.selection_fill
            //                         = Color32::from_rgba_unmultiplied(90, 140, 220, 70)
            //
            // — a **27 %-opacity** wash, because that role's real job is tinting
            // selected objects on the CANVAS, where translucency is the whole
            // point. Composited over a light panel it becomes roughly
            // `rgb(193, 207, 230)`: a pale blue-grey, *paler than
            // `widgets.inactive.weak_bg_fill`*, which is the opaque fill every
            // ordinary button gets. So the affirmative button rendered **less
            // solid than the Cancel button beside it** — which is exactly what
            // a disabled control looks like.
            //
            // ★★ THIS IS DEFECT D2 AGAIN, in the one place its fix did not
            // reach. When the active ribbon tab had the identical bug it was
            // moved to `accent` + `on_accent`; `FEATURES.md` records that fix
            // and even says the mode selector beside it "always did" paint the
            // accent. `Host::buttons` was written from the same wrong role and
            // nobody noticed, because a *dialog* button is not a *ribbon* tab
            // and no test compares the two.
            //
            // ★ `on_accent` rather than `strong_text_color()` is not a detail.
            // `accent` is a saturated blue in the light presets and a lighter
            // blue in the dark one, and `strong_text_color()` follows
            // `override_text_color` — which is the body text colour, near-black
            // in a light theme. Black on a saturated blue is poor; in the dark
            // preset it would be near-black text on a light blue, which is
            // fine, and in a future preset with a dark accent it would be black
            // on black. `Palette::on_accent` exists precisely so that pairing is
            // decided by the theme and moves with it — its own doc comment says
            // so, and says the two must never be welded together.
            //
            // Read through `egui-shell`'s accessor rather than reconstructed, so
            // there is one claimant for "what colour is this application's
            // accent" and `check-theme-colors.sh` still has nothing to object
            // to.
            let (fill, text) = egui_shell::Theme::accent_pair(ui.ctx());
            let default = egui::Button::new(egui::RichText::new(accept).color(text)).fill(fill);
            if ui.add(default).clicked() || enter {
                accepted = true;
            }
            if ui.button(cancel).clicked() {
                cancelled = true;
            }
        });
        (accepted, cancelled)
    }
}

#[cfg(test)]
mod tests {
    /// How many frames the divergence demonstration runs for. Must exceed
    /// [`FIT_BUDGET`]; see the compile-time assertion below.
    const DIVERGENCE_ROUNDS: usize = 12;

    use super::*;

    /// ★ **Two dialogs get two windows**, which is the whole reason the id is
    /// derived from a caller-supplied string rather than counted.
    ///
    /// A shared `ViewportId` is a shared OS window: the second dialog would
    /// draw into the first one's frame, and which one you saw would depend on
    /// draw order. Cheap to assert, and the failure is invisible until two
    /// dialogs are open at once.
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

    /// ★ **A host with nothing remembered reports nothing**, so the first open
    /// is placed rather than restored.
    ///
    /// Asserted against a real `Context` rather than a field, because as of
    /// 2026-08-21 the position is not a field: it lives in `egui::Memory`, and
    /// the property worth holding is *what the host answers*, not where it
    /// keeps it. See the module header for why the memory moved.
    #[test]
    fn a_fresh_host_remembers_no_position() {
        let ctx = egui::Context::default();
        let h = Host::new("print", "Print", Vec2::splat(100.0), Vec2::splat(10.0));
        assert!(h.remembered(&ctx).is_none());
    }

    /// …and once it has been told, it answers with what it was told — for
    /// **that dialog only**.
    ///
    /// ★ The second half is the one worth a test. Two dialogs sharing a memory
    /// key would drag each other around the desktop, and the key is derived
    /// from the same string as the viewport id, so a mistake there is a
    /// mistake in both places at once and invisible in either.
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

    /// ★★ **A window already big enough for its body is left alone**, which is
    /// the guard that keeps [`Host::fit`] from being a feedback loop.
    ///
    /// Only the no-op half is reachable headlessly — issuing the resize needs a
    /// live viewport — and the no-op half is the one with the hazard in it.
    /// Named rather than claimed: the growing branch is asserted by the driven
    /// check, which is the only place it can be.
    #[test]
    fn fitting_a_window_that_already_fits_asks_for_nothing() {
        let ctx = egui::Context::default();
        let h = Host::new("print", "Print", Vec2::new(400.0, 300.0), Vec2::splat(10.0));
        h.fit(&ctx, Vec2::new(100.0, 100.0));
        assert!(
            ctx.data(|d| d.get_temp::<Vec2>(h.fit_key)).is_none(),
            "no resize may be requested when the body already fits"
        );
    }

    /// **Growing to fit reaches a fixed point in one step, and stays there.**
    ///
    /// The half the test above says it cannot reach. It can, now that the
    /// arithmetic is a free function: grow once, then feed the result back and
    /// require silence.
    #[test]
    fn growing_to_fit_settles_after_one_step() {
        let min = Vec2::splat(10.0);
        let inner = Vec2::new(400.0, 300.0);
        let content = Vec2::new(520.0, 300.0);

        let want =
            fit_target(inner, content, min).expect("content wider than its window must grow");
        assert_eq!(want, Vec2::new(520.0, 300.0));
        assert!(
            fit_target(want, content, min).is_none(),
            "a window already grown to its content must ask for nothing further"
        );
    }

    /// **A window is never shrunk, on either axis.**
    #[test]
    fn fitting_only_ever_grows() {
        let inner = Vec2::new(800.0, 600.0);
        // Taller than its window, and much narrower.
        let want = fit_target(inner, Vec2::new(100.0, 900.0), Vec2::splat(10.0)).unwrap();
        assert_eq!(
            want.x, 800.0,
            "the axis that already fits must be left exactly as it was"
        );
        assert_eq!(want.y, 900.0);
    }

    /// ★★★ **The print dialog's runaway, reproduced as arithmetic — and the
    /// proof that no pure function could have stopped it.**
    ///
    /// Operator report, 2026-08-25: the print dialog *"keeps expanding its size
    /// in little steps to infinity"* after pressing Print. The cause was a
    /// footer row whose right-to-left button block reached the right edge of
    /// whatever width it was offered, with a status label appended AFTER it —
    /// so the row overflowed by the label's width no matter how wide the
    /// window became.
    ///
    /// This test models exactly that: content that is always `OVERFLOW` wider
    /// than its window. Every individual answer [`fit_target`] gives is
    /// correct, every one is a size it has never returned before, and the
    /// sequence still diverges — which is precisely why the fix is a **count**
    /// in [`Host::fit`] and not a smarter comparison here.
    ///
    /// It is written as a test rather than a comment so that anyone tempted to
    /// replace the budget with "just check the size is different" has to delete
    /// an assertion that says why it will not work.
    #[test]
    fn content_measured_from_its_own_window_diverges_and_never_repeats() {
        const OVERFLOW: f32 = 24.0;
        let min = Vec2::splat(10.0);
        let mut inner = Vec2::new(800.0, 600.0);
        let mut seen = Vec::new();

        for _ in 0..DIVERGENCE_ROUNDS {
            // The defect in one line: the content is a function of the window.
            let content = Vec2::new(inner.x + OVERFLOW, inner.y);
            let want = fit_target(inner, content, min)
                .expect("content wider than its window always asks to grow");
            assert!(
                !seen.contains(&want.x),
                "every requested size is NEW — which is why a once-per-size guard cannot see this, and why FIT_BUDGET counts instead"
            );
            seen.push(want.x);
            inner = want;
        }

        assert_eq!(
            inner.x,
            800.0 + OVERFLOW * DIVERGENCE_ROUNDS as f32,
            "unbounded, in steps of exactly the overflow — the operator's              'little steps to infinity'"
        );
    }

    /// The loop above runs further than the budget allows, on purpose: if
    /// [`FIT_BUDGET`] were ever raised past it the divergence demonstration
    /// would stop demonstrating anything, so the relationship is asserted at
    /// **compile time** rather than inside a test where clippy correctly points
    /// out that a comparison between two constants is not an assertion.
    const _: () = assert!(
        DIVERGENCE_ROUNDS > FIT_BUDGET,
        "the divergence test must run more rounds than the budget permits"
    );

    /// ★★★ **The layout half of the print dialog's runaway, measured in a real
    /// laid-out frame — and it fails on the old ordering.**
    ///
    /// The test above models the consequence; this one reproduces the CAUSE,
    /// which is the part a reader will not believe on assertion alone:
    ///
    /// > A `Layout::right_to_left` child inside a left-to-right `horizontal`
    /// > is anchored to the RIGHT EDGE of the space it was offered, and its
    /// > `min_rect` reaches that edge **whether or not it needed the room**.
    /// > Anything appended after it is therefore placed past the edge.
    ///
    /// Both orderings are laid out here in the same width, and the assertion is
    /// on the resulting row width against the width the row was given. Buttons
    /// first overflows; the status label first does not. That difference is the
    /// entire fix, and this is where it is proved rather than argued.
    ///
    /// ★ Run it against the pre-fix ordering — swap the two blocks in
    /// [`super::super::print`]'s `footer` — and the first assertion fails. That
    /// is the falsification, and without it this test would only be describing
    /// the code it sits next to.
    ///
    /// ★ Measured, so the size of the thing is on the record: in a 400 pt row
    /// the pre-fix ordering produces **481.9 pt** and the fixed ordering
    /// produces **exactly 400.0**. That 81.9 pt is the step the window grew by
    /// on every single frame the dialog was open after a print — which is what
    /// "little steps to infinity" was.
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
