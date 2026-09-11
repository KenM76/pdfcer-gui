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
use pdfcer_core::annot_author::{StampName, StickyIcon};
use pdfcer_core::page_tree::Rect;

use crate::app::actions::Action;
use crate::canvas::textannot::{
    DEFAULT_STAMP, DEFAULT_STAMP_SIZE, DEFAULT_STICKY_ICON, MAX_TEXT_CHARS, STAMP_SIZES, STAMPS,
    STICKY_ICONS, StampSize, TextAnnotKind,
};
use crate::stamps::lastused::LastStamp;
use crate::stamps::library::{CustomStamp, Library};
use crate::text::stamps as st;
use crate::text::textannot as t;

/// The region the whole window publishes.
pub const REGION_BODY: &str = "dialog:text-annot"; // ui-text-exempt: trace region name, never displayed
/// The region the text field publishes, so a driven check can type into it.
pub const REGION_TEXT: &str = "text-annot.text"; // ui-text-exempt: trace region name, never displayed
/// The region the Accept control publishes.
pub const REGION_ACCEPT: &str = "text-annot.accept"; // ui-text-exempt: trace region name, never displayed
/// **Cancel**, and it is declared for exactly the reason Accept is.
///
/// The operator's report of 2026-09-10 named both: *"I can't see the add or
/// cancel button. Those buttons should always be available."* A harness that
/// could see one of the two would report the answer row as reachable on a build
/// where half of it had been clipped away.
// ui-text-exempt: trace region name, never displayed
pub const REGION_CANCEL: &str = "text-annot.cancel";
/// The region the sticky note's icon chooser publishes, so a driven check can
/// find it and press one of the seven.
///
/// ★ Its own region rather than sharing [`REGION_BODY`], for the reason every
/// other region in this shell is separate: a check that asserts *"the icon
/// chooser is on screen"* against the window's own rectangle would pass on a
/// window with no chooser in it at all.
pub const REGION_ICON: &str = "text-annot.icon"; // ui-text-exempt: trace region name, never displayed
/// The region the stamp's label-size chooser publishes, so a driven check
/// can find it and open it.
///
/// ★ Its own region for [`REGION_ICON`]'s reason exactly, and that reason is
/// sharper here: the stamp body ALREADY had a gallery before this chooser
/// existed, so a check written against [`REGION_BODY`] would have passed on
/// every build since 2026-08-28 — including all of the ones with no size
/// control in them at all.
pub const REGION_STAMP_SIZE: &str = "text-annot.stamp-size"; // ui-text-exempt: trace region name, never displayed

/// The region the **operator's own** stamps publish, one enclosing box for the
/// whole custom half of the gallery.
///
/// ★ Separate from [`REGION_BODY`] and from [`REGION_STAMP_SIZE`] for the
/// reason the latter's comment states and this case makes sharper still: this
/// half of the gallery is **conditionally present**. It renders nothing at all
/// on a machine with no stamps folder (R9), so a driven check keyed on the
/// body region cannot tell *"his stamps are offered"* from *"this build draws
/// a stamp window"* — the two are the same observation there and different
/// observations here.
pub const REGION_CUSTOM_STAMPS: &str = "text-annot.custom-stamps"; // ui-text-exempt: trace region name, never displayed

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
    /// ☑ **The label size selected in the stamp's size chooser** (engine
    /// `Pass 287.0`). Meaningless for the other kinds and carried anyway,
    /// exactly as [`Self::stamp`] and [`Self::icon`] are.
    stamp_size: StampSize,
    /// The icon selected in the sticky note's chooser. Meaningless for the
    /// other kinds and carried anyway, exactly as [`Self::stamp`] is.
    icon: StickyIcon,
    /// ★★★ **The operator's own stamps, as they were on disk when this window
    /// opened** (`OPERATOR_REQUESTS.md` O172).
    ///
    /// # Why it is scanned per open, and not once per session
    ///
    /// Because Acrobat rewrites that folder. It adds a collection the first
    /// time somebody makes a stamp, replaces one when they edit it, and does
    /// both while pdfcer is running. A list cached at startup would show him a
    /// stamp he deleted an hour ago and hide the one he made two minutes ago,
    /// and the failure would look like pdfcer being broken rather than stale.
    ///
    /// The cost is one `Document::load` per collection file, on the frame the
    /// window opens — on this machine, one file of 78 KB. If that ever becomes
    /// a stall it is a *measurement* that says so, not this comment.
    ///
    /// ⚠ Empty for the sticky note and the text box: [`Self::open`] scans only
    /// for the stamp kind, because the other two kinds cannot reach a gallery
    /// and paying for a filesystem walk to fill a field nothing reads is the
    /// kind of cost that never shows up in a profile attributed to its cause.
    library: Library,
    /// **Which of his own stamps is selected**, or `None` for a standard one.
    ///
    /// ★ This field, rather than a `StampName`-shaped enum with a `Custom`
    /// arm, and the reason is the type: `StampName` is the engine's spelling
    /// of §12.5.6.12's **closed vocabulary**, and a custom stamp is by
    /// definition not in it. Widening that enum here would be this shell
    /// asserting something about the standard that is not true.
    ///
    /// ⇒ So the gallery's selection is *"`custom` is `Some`, or else
    /// [`Self::stamp`]"*, and the two radio groups keep that invariant by
    /// clearing the other on click. The invariant is asserted in this module's
    /// tests, because it is held by two call sites rather than by a type.
    custom: Option<CustomStamp>,
    /// Set by Accept, consumed after the window's closure returns.
    /// **What to remember, once this window has committed** -- `None` until it
    /// does, and `None` forever on any window that is not a stamp.
    ///
    /// ★★ Written at the commit, read by the host after [`Self::show`] has
    /// returned `false`, and that ordering is the whole design. The alternative
    /// -- the host reading `self.stamp` / `self.custom` when the window closes
    /// -- cannot tell Add from Cancel, so dismissing a window would set the
    /// memory to a stamp the operator explicitly declined to place. A field
    /// only the accept path writes makes that unrepresentable rather than
    /// merely avoided.
    committed: Option<LastStamp>,
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

/// **How much taller the sticky note's window opens**, in points.
///
/// ★★ Because that one body is genuinely longer than the other two, and by a
/// known amount: seven radio rows, a heading and the icon disclosure, added
/// under the text field on 2026-09-06. 240 pt held a four-line field and two
/// buttons comfortably; it does not hold those as well, and a dialog whose
/// Accept button is below its own bottom edge is a window an operator cannot
/// finish.
///
/// # ★ A constant added to [`WINDOW_PTS`], not a size measured from the body
///
/// `print/layout.rs`' rule and `Host::fit`'s: **a size measured from the
/// content it sizes is R128**, and this project has met that three times. So
/// the number is derived from what is being added — seven rows at roughly the
/// row height plus a heading and a wrapped small line — and stated as a
/// constant that can be read and argued with, rather than queried from a `Ui`
/// that is being laid out inside the window this decides the height of.
///
/// ⚠ It is deliberately generous. Over-tall costs the operator nothing on a
/// dialog they can resize and drag; under-tall costs them the Accept button.
const STICKY_EXTRA_PTS: f32 = 190.0;

/// **How much taller the stamp's window opens**, in points.
///
/// ★★ [`STICKY_EXTRA_PTS`]'s argument, applied to a smaller addition. The
/// stamp body was seven radio rows and a wrapped disclosure inside 240 pt;
/// `Pass 287.0`'s size chooser adds a heading, one combo row and a second
/// wrapped disclosure under them. That is roughly three rows' worth, not
/// ten — the ten sizes live inside the combo's popup, which is drawn in its
/// own layer and costs the window no height at all.
///
/// ★ That is the concrete reason the chooser is a combo and the gallery is
/// radios, stated here rather than only in [`TextAnnotDialog::sizes`]: ten
/// radio rows would have needed roughly 250 pt and made the stamp the
/// tallest of the three dialogs by a wide margin, for a control every other
/// program on his desk draws as a dropdown.
///
/// ⚠ Derived from what is being ADDED and stated as a constant, never
/// measured from the `Ui` being laid out inside the window this sizes —
/// `print/layout.rs`' rule and `Host::fit`'s, which this project has met
/// three times as R128.
const STAMP_EXTRA_PTS: f32 = 70.0;

/// **How much taller the stamp's window opens for each CATEGORY of the
/// operator's own stamps**, in points.
///
/// A category costs a 6 pt space and one `.small()` heading. 26 pt is that,
/// rounded up.
const CUSTOM_CATEGORY_PTS: f32 = 26.0;

/// **…and for each of his stamps**, in points.
///
/// Measured, not guessed: the driven run of 2026-09-10 published three custom
/// radios at content y 298, 326 and 354, so the pitch of a radio row in this
/// window is exactly 28 pt.
///
/// ⚠ That is a measurement of a ROW's pitch, taken once, from a trace — not a
/// size queried from the `Ui` this function helps to size. The distinction is
/// R128's: a constant a reader can argue with, versus a feedback loop.
const CUSTOM_STAMP_ROW_PTS: f32 = 28.0;

/// **…and for the dynamic-stamp disclosure**, when the collection holds one.
///
/// The sentence appears the moment he selects a dynamic stamp. Counted up
/// front rather than when it appears, because a window that grows on a click
/// moves the control that was clicked, which is a worse behaviour than being
/// 22 pt taller than it strictly needs.
const CUSTOM_DYNAMIC_NOTE_PTS: f32 = 22.0;

/// **The most the operator's own stamps may add to the window**, in points.
///
/// Roughly eleven rows. Past that the body scrolls, which is what a scroll
/// area is for.
///
/// ★ A cap is needed even though [`window_size`] already clamps to the
/// application window. Without one a collection of forty stamps opens a dialog
/// the full height of the window it belongs to, standing over the drawing he
/// is annotating — the exact thing this module's header argues against for the
/// dialog's POSITION. Scrolling for the fortieth stamp is a smaller cost than
/// losing sight of the sheet for all forty.
const CUSTOM_EXTRA_MAX_PTS: f32 = 320.0;

/// **How much taller the stamp window opens because the operator has stamps of
/// his own.**
///
/// # ★★★ Why this exists, and it is a defect report — 2026-09-10
///
/// `Pass O172` added the custom half to the stamp gallery and did not change
/// the window's height. The first driven run of
/// `custom_stamp_reaches_the_page` captured the result: a dialog showing the
/// seven standard stamps and the Add/Cancel row, with **none** of the
/// operator's three stamps on the screen. They were laid out below the
/// scrolled body's fold, published as rectangles in the scrolled content, and
/// invisible.
///
/// It was not a clipping bug and it was not a layout bug. [`window_size`] adds
/// a per-kind constant to [`WINDOW_PTS`] and the stamp's constant was written
/// for a body that did not yet have this section in it. **A guessed size is a
/// claim about the content, and the content changed under the claim.**
///
/// # ★★ Why counting the library is NOT the R128 feedback loop
///
/// The rule this module states three times — *a size measured from the content
/// it sizes is R128* — is about querying a `Ui` that is being laid out inside
/// the window whose size is being decided. This function queries no `Ui`. It
/// reads two integers off a [`Library`] that was scanned off the disk when the
/// dialog opened, before any layout ran, and multiplies them by constants
/// stated above. The result is the same kind of number as
/// [`STICKY_EXTRA_PTS`] — derived from what is being added, and arguable by a
/// reader — except that "what is being added" is data rather than a fixed list
/// of seven radios.
///
/// ⇒ There is no loop, because nothing about the laid-out window can change
/// the count of files in his stamps folder.
#[must_use]
fn custom_extra_pts(library: &Library) -> f32 {
    if library.is_empty() {
        return 0.0;
    }
    let categories = library.categories.len();
    let stamps: usize = library.categories.iter().map(|c| c.stamps.len()).sum();
    // ★ `saturating` arithmetic is not available on f32 and is not needed: the
    // counts come from a directory scan and the cap below bounds the result
    // whatever they are.
    #[allow(clippy::cast_precision_loss)]
    let wanted = categories as f32 * CUSTOM_CATEGORY_PTS
        + stamps as f32 * CUSTOM_STAMP_ROW_PTS
        + if library
            .categories
            .iter()
            .any(|c| c.stamps.iter().any(|s| s.dynamic))
        {
            CUSTOM_DYNAMIC_NOTE_PTS
        } else {
            0.0
        };
    wanted.min(CUSTOM_EXTRA_MAX_PTS)
}

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
fn window_size(screen: egui::Rect, kind: TextAnnotKind, custom_extra: f32) -> egui::Vec2 {
    // ★ The height is per-KIND as of 2026-09-06, and the width is not. The
    // three bodies are the same width by construction — a field, a gallery and
    // a chooser all stretch to the window — and two of the three grew
    // downwards. Making the width vary too would be a second number with no
    // reason behind it.
    //
    // ⚠ The text box is now the ONLY kind with no addition, so this match no
    // longer folds two arms into one. Written out rather than left as
    // `_ => 0.0`, so a fourth kind is a compile error here instead of a
    // silently unsized dialog.
    let extra = match kind {
        TextAnnotKind::Sticky => STICKY_EXTRA_PTS,
        // ★ `custom_extra` is added to the stamp's arm and nowhere else. The
        // caller computes it from the library, and the other two kinds have no
        // gallery to put one in — see [`custom_extra_pts`] for why counting the
        // operator's stamps is not the feedback loop this file forbids three
        // times.
        TextAnnotKind::Stamp => STAMP_EXTRA_PTS + custom_extra,
        TextAnnotKind::TextBox => 0.0,
    };
    egui::vec2(
        WINDOW_PTS
            .x
            .min(screen.width() - SCREEN_MARGIN_PTS)
            .max(MIN_WINDOW_PTS.x),
        // ★ Clamped to the application window, minus the same margin the width
        // leaves, so a tall dialog on a short screen is squeezed rather than
        // running off the bottom — and floored at `MIN_WINDOW_PTS.y` for the
        // reason `window_size`'s width floor exists: an unreachable negative
        // size becomes reachable the day somebody adds a monitor at 250 %.
        (WINDOW_PTS.y + extra)
            .min(screen.height() - SCREEN_MARGIN_PTS)
            .max(MIN_WINDOW_PTS.y),
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
    ///
    /// `last` is the stamp the operator most recently committed **this
    /// session**, or `None` when he has not placed one yet -- O172's *"and it
    /// remembers the last one used"* clause. See
    /// [`crate::stamps::lastused`] for why the memory is a name that is
    /// re-resolved here rather than a stored stamp.
    ///
    /// ★★ It is a REQUIRED argument rather than one with a
    /// keep-the-old-behaviour default. A defaulted parameter silently declines
    /// the feature at every call site written before it existed, the compiler
    /// goes quiet about it, and the tests stay green -- this project has been
    /// bitten by exactly that. Every caller now has to say what it means.
    #[must_use]
    pub fn open(page: usize, kind: TextAnnotKind, rect: Rect, last: Option<&LastStamp>) -> Self {
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "text-annot-open kind={kind:?} page={page} w={:.2} h={:.2}",
                rect.urx - rect.llx,
                rect.ury - rect.lly
            )
        });
        // Scanned here, guarded on the kind -- see the `library` field's own
        // note for why it is per-open rather than per-session, and why the
        // other two kinds do not pay for it.
        let library = if matches!(kind, TextAnnotKind::Stamp) {
            let found = crate::stamps::library::scan();
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!(
                    "custom-stamp-library categories={} stamps={} unreadable={} unplaceable={} folder={}",
                    found.categories.len(),
                    found.len(),
                    found.unreadable,
                    found.unplaceable,
                    found.folder.is_some()
                )
            });
            found
        } else {
            Library::default()
        };
        // ⚠ Resolved against the library that was just scanned, NOT against the
        // one that existed when the memory was taken. That is the entire
        // contract of `LastStamp::resolve`: a collection the operator has since
        // edited in Acrobat has renumbered pages, and a remembered index would
        // place different artwork under the right label.
        //
        // A memory that no longer resolves falls back to the default and says
        // nothing on screen -- it is indistinguishable to him from the first
        // stamp of a session. The trace is what tells the two apart, so a
        // driven check can assert the difference the operator cannot see.
        let (stamp, custom) = last.map_or((DEFAULT_STAMP, None), |l| {
            l.resolve(&library, DEFAULT_STAMP)
        });
        if matches!(kind, TextAnnotKind::Stamp) {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!(
                    "stamp-gallery-opens remembered={} restored={}",
                    last.map_or_else(|| "none".to_owned(), LastStamp::token),
                    if custom.is_some() {
                        "custom"
                    } else if last.is_some() {
                        "standard"
                    } else {
                        "default"
                    }
                )
            });
        }
        Self {
            page,
            kind,
            rect,
            text: String::new(),
            stamp,
            // ⚠ The named constant, never `StampSize::default()`, even though
            // the two are the same value today. The constant is where the
            // argument lives for why a fresh gallery offers the DERIVED size
            // rather than the engine's flat 12 pt, and a `default()` call
            // would let that argument be silently overturned by an edit to a
            // `#[derive]` attribute three files away.
            //
            // ★★ It is deliberately NOT remembered alongside the stamp. The
            // size is a property of the box he is drawing right now, and
            // `FitTheBox` already derives it from that box; carrying a literal
            // point size over from a stamp placed on a different sheet would
            // make the next one silently the wrong size. Acrobat does not
            // remember it either.
            stamp_size: DEFAULT_STAMP_SIZE,
            icon: DEFAULT_STICKY_ICON,
            library,
            custom,
            committed: None,
            accept_requested: false,
            close_requested: false,
            focused_once: false,
            focus_attempts: 0,
        }
    }

    /// Draw one frame. Returns `false` when it should close.
    pub fn show(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) -> bool {
        let screen = ctx.input(egui::InputState::content_rect);
        let size = window_size(screen, self.kind, custom_extra_pts(&self.library));
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
            // ★★★ The body scrolls; the buttons do not. See
            // [`crate::dialogs::host::Host::scrolled`] for the operator's rule
            // this enacts and why it is structural rather than best-effort.
            //
            // This dialog is the one that made the rule: its height is a
            // GUESS — `WINDOW_PTS` plus a per-kind constant — deliberately not
            // measured from the content it sizes, because measuring it is
            // R128's feedback loop. A guess that is too small used to clip the
            // bottom of the window, and the bottom of this window is Add and
            // Cancel.
            crate::dialogs::host::Host::scrolled(
                ui,
                self,
                |ui, this| this.body(ui),
                |ui, this| this.footer(ui),
            );
        });
        let open = !frame.closed;

        if self.accept_requested {
            self.accept_requested = false;
            // Taken BEFORE the action is pushed and before `text` is moved
            // out, so the memory describes the choice that is actually being
            // committed. Only meaningful for a stamp; the other two kinds have
            // no gallery and nothing to remember.
            if self.kind.uses_gallery() {
                self.committed = Some(LastStamp::taken(self.stamp, self.custom.as_ref()));
            }
            actions.push(Action::CommitTextAnnot {
                page: self.page,
                kind: self.kind,
                rect: self.rect,
                text: std::mem::take(&mut self.text),
                stamp: self.stamp,
                stamp_size: self.stamp_size,
                icon: self.icon.clone(),
                custom: self.custom.clone(),
            });
            return false;
        }
        // ★ The window's own close button counts as Cancel, and authors
        // nothing. That is the honest reading: the operator dismissed a
        // question, and a dismissed question is not an answer.
        !(self.close_requested || !open)
    }

    /// **The memory this window earned**, or `None` if it was cancelled, was
    /// not a stamp, or has not committed yet.
    ///
    /// Takes the value rather than borrowing it: the host calls this exactly
    /// once, on the frame the window closes, and is about to drop the window.
    /// A borrow would invite a second read of a value that has already been
    /// stored somewhere else.
    pub fn remembered(&mut self) -> Option<LastStamp> {
        self.committed.take()
    }

    /// The field or the gallery — everything above the button row, and the
    /// only part of this window that scrolls.
    fn body(&mut self, ui: &mut Ui) {
        ui.label(t::intro(self.kind));
        ui.add_space(8.0);

        if self.kind.uses_gallery() {
            self.gallery(ui);
        } else {
            self.field(ui);
        }
        // ★★★ **The icon chooser — the sticky note's own gallery**, and it sits
        // BELOW the text field rather than above it.
        //
        // The field is the question the window opened to ask (*"what is the
        // note?"*), and it is the control that takes focus on the first frame.
        // A chooser above it would put seven radio buttons between the title and
        // the caret, so the operator's eye and the keyboard would start in
        // different places — which is the same defect
        // `TextAnnotDialog::field`'s retry exists to prevent, arrived at from
        // the layout instead of from the focus race.
        //
        // ★ It is drawn for the sticky kind alone, and absent — not greyed —
        // for the other two. R9, and the engine agrees in writing: an `icon` on
        // anything but a `/Text` is `EditError::StylePropertyNotApplicable`,
        // refused by name rather than swallowed, because a `/Stamp`'s face
        // comes from its own `/Name` vocabulary and a `/FreeText` has no icon
        // at all.
        self.icons(ui);
    }

    /// **The two buttons, pinned to the bottom of the window.**
    ///
    /// Separated from [`Self::body`] on 2026-09-10, on the operator's report
    /// that the second stamp he placed opened a window whose Add and Cancel
    /// were below its own bottom edge. They were the last items in a linear
    /// layout, so they were the first thing an under-tall window clipped —
    /// and a dialog whose Accept is off-screen is a transaction that cannot be
    /// finished, only abandoned with the title bar's X.
    fn footer(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            // ★ Accept is greyed when there is nothing to author, with the
            // reason on hover. That is the one place this shell greys rather
            // than hides: the control is *temporarily* unavailable — a
            // keystroke makes it live — which is exactly what greying is
            // reserved for.
            let ready = self.kind.uses_gallery() || !self.text.trim().is_empty();
            let accept = ui.add_enabled(ready, egui::Button::new(t::accept()));
            // ★★★ `ui_rect_visible`, NOT `ui_rect`, and the difference is the
            // whole of the operator's report of 2026-09-10.
            //
            // `ui_rect` publishes a rectangle whether or not it is on the
            // screen, so a check that read it would have found Accept
            // "declared" on the very build where he could not see it, and would
            // then have clicked a point below the window's own bottom edge —
            // a plausible number, no error anywhere, and a defect reported as
            // working. `ui_rect_visible` stays SILENT when the rect is outside
            // its clip rect, which turns "was it declared?" into "was it on the
            // screen?" and removes the last size anybody has to guess at.
            crate::diag::ui_rect_visible(REGION_ACCEPT, accept.rect, ui.clip_rect());
            if accept.clicked() {
                self.accept_requested = true;
            }
            if !ready {
                accept.on_disabled_hover_text(t::accept_disabled(self.kind));
            }
            let cancel = ui.button(t::cancel());
            crate::diag::ui_rect_visible(REGION_CANCEL, cancel.rect, ui.clip_rect());
            if cancel.clicked() {
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

    /// **The sticky note's icon chooser**, for the one kind that has a `/Name`
    /// picture.
    ///
    /// ★ Radios over a combo box, matching [`Self::gallery`] one function down
    /// and for its stated reason: seven entries is a set an operator reads at a
    /// glance, and a combo would hide six of them behind a click for no saving.
    /// Using the *same* control for the two choosers is deliberate — they are
    /// the same act (*pick one of seven*) and a window that answered it two
    /// ways would be teaching the operator a distinction that does not exist.
    ///
    /// ★★ The disclosure under it is not optional. `annot_author::sticky_note`
    /// (`pdfcer-core` `annot_author.rs:3712`) passes the icon to `/Name` at
    /// `:3759` and **nowhere else** — the marker artwork is the same dog-eared
    /// page glyph for all seven, by a trade-dress decision the engine records
    /// at `annot_author.rs:2794`. So an operator who picks *Key* sees no change
    /// here and a key in another reader, and
    /// [`crate::text::textannot::sticky_icon_bound`] is where they are told
    /// that before it happens rather than after.
    fn icons(&mut self, ui: &mut Ui) {
        if self.kind != TextAnnotKind::Sticky {
            return;
        }
        ui.add_space(8.0);
        ui.label(t::sticky_icon_heading());
        let top = ui.cursor().min;
        for icon in STICKY_ICONS {
            ui.radio_value(&mut self.icon, icon.clone(), t::sticky_icon_label(icon));
        }
        crate::diag::ui_rect(REGION_ICON, egui::Rect::from_min_max(top, ui.cursor().min));
        ui.label(egui::RichText::new(t::sticky_icon_bound()).small().weak());
    }

    /// ☑ **The stamp's label-size chooser** — engine `Pass 287.0`'s operator
    /// half, and the answer to *"I have to draw the size before it gets
    /// applied"*.
    ///
    /// # ★★ A combo, where the two galleries beside it are radios
    ///
    /// [`Self::icons`] argues for radios and gives the reason — *seven entries
    /// is a set an operator reads at a glance* — and then says using the same
    /// control for both choosers is deliberate because *"they are the same act
    /// (pick one of seven)"*. This is a different act, and the difference is
    /// the point rather than an exception to that rule:
    ///
    ///   * A stamp face and a note icon are **named things from a closed set**
    ///     the operator wants to see all of. A size is a **number on a scale**,
    ///     where the list is a convenience and the operator already knows what
    ///     18 pt is without reading the other nine.
    ///   * Every program on his desk — Word, Acrobat, every CAD annotation
    ///     dialog — draws a size as a dropdown. Standing rule: **use the
    ///     conventional interaction, never invent one.** The convergence of a
    ///     product class is the specification, and a vertical stack of ten
    ///     size radios would be pdfcer's own invention.
    ///   * It costs the window ~70 pt instead of ~250. See
    ///     [`STAMP_EXTRA_PTS`].
    ///
    /// # ★ What the disclosure under it is, and what it is NOT
    ///
    /// [`crate::text::textannot::stamp_size_bound`] tells the operator the box
    /// will widen if the words need it. That is **not** an R8b rule 4
    /// disclosure of an inference — a grown box is visible on the canvas as
    /// itself, and a screenshot of it matches a screenshot of the saved file.
    /// It is told before the act, so the drag means what the operator thinks
    /// it means.
    fn sizes(&mut self, ui: &mut Ui) {
        ui.add_space(8.0);
        ui.label(t::stamp_size_heading());
        let top = ui.cursor().min;
        let response = egui::ComboBox::from_id_salt("text-annot-stamp-size") // ui-text-exempt: internal widget id
            .selected_text(t::stamp_size_label(self.stamp_size))
            .show_ui(ui, |ui| {
                for size in STAMP_SIZES {
                    let entry = ui.selectable_value(
                        &mut self.stamp_size,
                        *size,
                        t::stamp_size_label(*size),
                    );
                    // ★★★ **Each open entry declares its own region, and this is
                    // what makes the chooser DRIVABLE at all.**
                    //
                    // The alternative a harness is pushed into without this is
                    // arithmetic: press the combo, then click `row_height × n`
                    // below its top-left corner. That is a guess about egui's
                    // interior spacing dressed as a coordinate, it silently
                    // selects the WRONG SIZE when a theme changes a row's
                    // padding by two points — and a check that selected 18 pt
                    // while asking for 24 pt reports *"the operator's choice
                    // did not reach the engine"*, which is a defect report
                    // about the application written by a defect in the
                    // harness. This project has already lost a day to that
                    // exact shape.
                    //
                    // ★★ The name is keyed on
                    // [`StampSize::trace_token`](crate::canvas::textannot::StampSize::trace_token)
                    // — the same token the commit's `stamp-style size=` field
                    // carries — so the check presses `…stamp-size.24` and then
                    // looks for `size=24`. **One vocabulary, both ends.** A
                    // drift that renamed one and not the other could otherwise
                    // leave the check pressing a control that exists and
                    // matching a field that no longer says what it pressed.
                    //
                    // ⚠ These are declared only while the popup is OPEN. A
                    // region that stops being declared emits `ui-rect-gone`, so
                    // a check must read the frame *after* the press that opens
                    // it, not a cached one — the standing lesson that a
                    // whole-capture `last()` on a surface that has since closed
                    // returns a fossil.
                    crate::diag::ui_rect(
                        // ui-text-exempt: diagnostic region name, never displayed.
                        &format!("{REGION_STAMP_SIZE}.{}", size.trace_token()),
                        entry.rect,
                    );
                }
            })
            .response;
        // ★ The region covers the CONTROL, not the popup. The popup is drawn
        // in its own layer and exists only while it is open, so a driven check
        // has to press this rectangle first and read the popup's own entries
        // afterwards. The union with the cursor is taken because the heading
        // above belongs to the chooser, and a check looking for the words
        // should find them inside the region that names them.
        crate::diag::ui_rect(
            REGION_STAMP_SIZE,
            egui::Rect::from_min_max(top, ui.cursor().min).union(response.rect),
        );
        // ★★★ **The control reports its own state, and this is a separate fact
        // from the size reaching the engine.**
        //
        // A combo that stores the operator's pick correctly and goes on
        // displaying the previous one is a control he cannot trust: he has no
        // way to tell an accepted choice from an ignored click, and the
        // document he gets is right for a reason he could not have predicted.
        // `canvas::textannot`'s `stamp-style` line proves the *commit* carried
        // the number; nothing there proves the *window* ever admitted it.
        //
        // ★★ [`crate::diag::trace_changed`] rather than `trace`, because this
        // is a frame-loop call site: an unchanged value re-reported sixty times
        // a second answers the question no better and buries every other line
        // in the capture. The slot de-duplicates on the formatted line, so the
        // first draw emits the default and nothing else is emitted until the
        // operator changes it — which is precisely the two observations a
        // driven check wants and no third one.
        //
        // ⚠ The value is [`StampSize::trace_token`], **not** `t::stamp_size_label`.
        // The label is operator copy — it is allowed to be reworded on a
        // Tuesday, it is translatable in principle, and a machine keyed on it
        // would break silently on an edit that improved it.
        crate::diag::trace_changed("stamp-size-chooser", || {
            let token = self.stamp_size.trace_token();
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("stamp-size-chooser selected={token}")
        });
        ui.label(egui::RichText::new(t::stamp_size_bound()).small().weak());
    }

    /// **Select a standard stamp**, and clear whatever custom one was live.
    ///
    /// # ★★★ Why this is a method and not two lines at the call site
    ///
    /// Because the gallery's selection spans **two fields**, and the invariant
    /// *"exactly one of them is the selection"* is the only thing that stops a
    /// click on `Approved` from placing the operator's signature. `radio_value`
    /// cannot express it — it writes one variable and knows nothing about the
    /// other — so the two writes have to travel together, and a pair of writes
    /// that must travel together is a function.
    ///
    /// ⇒ The payoff is that the invariant becomes **testable without a
    /// window**. Held at two inline call sites it could only be checked by
    /// laying out a frame and synthesising a click, which is a driven check's
    /// job and not a unit test's; held here it is two calls and an assertion.
    fn select_standard(&mut self, stamp: StampName) {
        self.stamp = stamp;
        self.custom = None;
    }

    /// **Select one of the operator's own stamps.**
    ///
    /// ★ [`Self::stamp`] is deliberately left alone rather than reset. It is
    /// not read on this route — `app::actions::apply` forks on `custom` before
    /// it looks at anything else — so clearing it would buy nothing, and it
    /// means a click back onto a standard stamp restores the one he had
    /// chosen before rather than snapping to `Approved`.
    fn select_custom(&mut self, stamp: CustomStamp) {
        self.custom = Some(stamp);
    }

    /// The stamp gallery: the seven standard stamps, then the operator's own.
    ///
    /// # ★★ Why `ui.radio(..).clicked()` and not `ui.radio_value(..)`
    ///
    /// Because the selection spans **two** fields. `radio_value` writes one
    /// variable and knows nothing about the other, so a click on `Approved`
    /// would set [`Self::stamp`] and leave [`Self::custom`] holding his
    /// signature — and the commit path reads `custom` first, so the operator
    /// would have picked `Approved` and got his signature. Silent, and
    /// reproducible on the first click of a second choice.
    ///
    /// ⇒ Each arm therefore writes both halves. The invariant *"exactly one of
    /// the two is the selection"* is held here, at two call sites, and asserted
    /// in this module's tests because two call sites is not a type.
    fn gallery(&mut self, ui: &mut Ui) {
        // A vertical list of radios rather than a combo box: seven entries is
        // a set an operator reads at a glance, and a combo would hide six of
        // them behind a click for no saving — this window has the room.
        for stamp in STAMPS {
            let selected = self.custom.is_none() && self.stamp == *stamp;
            if ui.radio(selected, t::stamp_label(*stamp)).clicked() {
                self.select_standard(*stamp);
            }
        }
        ui.add_space(4.0);
        ui.label(egui::RichText::new(t::stamp_bound()).small().weak());
        let _ = self.custom_stamps(ui);
        // ★ The size chooser is drawn by the gallery rather than by
        // `Self::body`, which is where `Self::icons` is called from. The
        // difference is that `icons` guards on the kind itself and returns
        // early for the other two; this function is ALREADY the stamp-only
        // branch, so a second guard would be a condition that can never be
        // false — and a condition that cannot be false is a line a reader has
        // to prove harmless.
        //
        // ★★ It IS guarded on the custom selection, and that guard is R9 rather
        // than tidiness. The size chooser sets the point size of the **label
        // text the engine draws** for a standard stamp; a custom stamp has no
        // label — its words are pixels in somebody's artwork — so the control
        // governs nothing. R9: an unavailable capability renders **nothing**.
        // A greyed size box beside his signature would be a control whose
        // absence of effect he would have to discover.
        if self.custom.is_none() {
            self.sizes(ui);
        }
    }

    /// **The operator's own stamps**, grouped by the category their collection
    /// declares — which is the same heading Acrobat's stamp menu shows.
    ///
    /// Renders **nothing at all** when he has none: no heading, no empty
    /// group, no *"you can add your own"* invitation. R9. A machine with no
    /// Acrobat, or with an Acrobat nobody has ever made a stamp in, gets the
    /// seven standard entries and no evidence that a second half exists.
    ///
    /// # Returns
    ///
    /// **Whether anything was drawn.** Not used by the caller, and that is the
    /// point: it exists so R9 can be asserted by a unit test rather than only
    /// by a driven check. [`crate::diag::ui_rect`] is write-only and silent
    /// unless the diagnostic channel is on, so *"the region was not emitted"*
    /// is not a question a test in this crate can ask — and *"the operator
    /// sees no custom half"* is exactly the claim R9 makes. A returned `bool`
    /// is the smallest thing that makes the claim checkable in-process.
    ///
    /// ⚠ It is a claim about this function only. That the *gallery* calls it,
    /// and that a real folder produces a real list, are separate claims and
    /// the driven check owes both.
    fn custom_stamps(&mut self, ui: &mut Ui) -> bool {
        if self.library.is_empty() {
            return false;
        }
        let top = ui.cursor().min;
        let mut index = 0usize;
        // ★★ The click is recorded here and applied after the loop, because
        // the loop holds `&self.library` and `Self::select_custom` needs
        // `&mut self`. The alternative — indexing `self.library.categories[i]`
        // so each borrow ends at the statement — reads worse and clones the
        // label on every frame rather than on the frame he clicks.
        //
        // ⇒ The visible consequence is that the radio he just pressed draws
        // unselected for the remainder of *this* frame. That is not a defect
        // to work around: a click is an input event, egui repaints on input,
        // and the next frame is drawn before anything reaches the screen.
        let mut picked: Option<CustomStamp> = None;
        for category in &self.library.categories {
            ui.add_space(6.0);
            let heading = if category.name.is_empty() {
                st::gallery_category_unnamed().to_owned()
            } else {
                st::gallery_category(&category.name)
            };
            // ⚠ Not `RichText::strong()` — D11, and the gate that enforces it.
            // There is no colour `.strong()` can resolve to that is correct on
            // both an accent fill and a panel.
            ui.label(egui::RichText::new(heading).small());
            for stamp in &category.stamps {
                let selected = self
                    .custom
                    .as_ref()
                    .is_some_and(|c| c.file == stamp.file && c.page_index == stamp.page_index);
                let response = ui.radio(selected, &stamp.label);
                // ★ Keyed on the stamp's ORDINAL, not on its label. The label
                // is the operator's own words — his signature is called `Ken`
                // — and a driven check keyed on it would be a check that only
                // runs on this machine. `library::scan` sorts the file list
                // and the categories, so the ordinal is stable between runs
                // for an unchanged folder, which is what a check needs.
                // ★★★ `ui_rect_visible`, not `ui_rect` — 2026-09-10, and the
                // reason is a driven failure rather than a preference.
                //
                // The first driven run of `custom_stamp_reaches_the_page`
                // found these three rows published at content y 298, 326 and
                // 354 inside a window whose body ended at 270. They were
                // rectangles in the SCROLLED CONTENT, not positions on the
                // screen, and the harness clicked the first of them — into
                // the dialog's own drop shadow, twenty-odd points below its
                // bottom edge. Every number involved was correct and the
                // operator could not see a single one of his stamps.
                //
                // `ui_rect_visible` stays silent when the rect falls outside
                // its clip rect, which collapses *"was the row published?"*
                // and *"was the row on the screen?"* into one question — the
                // same property `REGION_ACCEPT` was given for O171, for the
                // same reason and one day earlier.
                crate::diag::ui_rect_visible(
                    // ui-text-exempt: diagnostic region name, never displayed.
                    &format!("{REGION_CUSTOM_STAMPS}.{index}"),
                    response.rect,
                    ui.clip_rect(),
                );
                index += 1;
                if response.clicked() {
                    picked = Some(stamp.clone());
                    crate::diag::trace(|| {
                        // ui-text-exempt: diagnostic trace, never displayed in the UI
                        format!(
                            "custom-stamp-chosen name={} page={} dynamic={}",
                            stamp.label, stamp.page_index, stamp.dynamic
                        )
                    });
                }
            }
        }
        // Applied before the note below, so the disclosure for a dynamic stamp
        // appears on the same frame the operator chooses one.
        if let Some(stamp) = picked {
            self.select_custom(stamp);
        }
        // ★★ The pre-commit disclosure for a dynamic stamp — R8b rule 4's
        // *affordance* half. It appears only once one is chosen, sits in the
        // window rather than on the canvas, blocks nothing, and does not tell
        // him to stop. Its whole job is that the choice is still reversible
        // while he is reading it.
        if self.custom.as_ref().is_some_and(|c| c.dynamic) {
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(st::gallery_dynamic_note())
                    .small()
                    .weak(),
            );
        }
        // The enclosing region, taken from the cursor's travel: everything
        // this function drew, headings included. A check asking *"are his
        // stamps offered"* reads this one; a check pressing a particular stamp
        // reads the ordinal-keyed ones above.
        crate::diag::ui_rect(
            REGION_CUSTOM_STAMPS,
            egui::Rect::from_min_max(top, ui.cursor().max),
        );
        true
    }
}

#[cfg(test)]
#[path = "textannot_tests.rs"]
mod tests;
