//! # `canvas::moving::nudge` — the arrow keys, and the one point they move
//!
//! Every drawing program moves a selected thing with the arrow keys, and until
//! today this one did not: an operator who had placed a revision cloud two
//! points too far left had to grab it with the pointer and drag, at whatever
//! precision the zoom happened to give them. At 100 % that is a two-pixel drag
//! nobody can make reliably, and the correction is the one edit an operator
//! makes most often.
//!
//! ## ★★★ Why this is small, and why it is in `canvas::moving` rather than in
//! `canvas::keys`
//!
//! **Because the verb already exists and is already called.** A nudge is a
//! **delta**, and `EditSession::move_annotation` takes a delta —
//! [`crate::app::actions::annot::AnnotAction::Move`] is raised by
//! [`crate::canvas::annotdrag`] on the release of a pointer drag and this raises
//! the identical action with the identical operands. There is no second verb, no
//! second arithmetic and no second set of refusals; what is new is a second way
//! to say *how far*.
//!
//! That is also why it lives here. [`super`] owns the one function in `canvas/`
//! that crosses into PDF space ([`super::page_delta`]) and owns the module docs
//! that argue the crossing. A nudge that did its own trigonometry in the key
//! handler would be a **second derivation of the page transform**, which is the
//! precise failure `viewer`'s header warns about:
//!
//! > *"PDF user space is y-UP; canvas and screen are y-DOWN. The failure is
//! > silent — the page looks perfect until someone selects a line and gets a
//! > different one."*
//!
//! ## ★★★ THE Y SIGN, and how it is got right without deciding it here
//!
//! An operator's **Up arrow means up on the screen.** PDF user space has y
//! increasing *upward* from the bottom-left corner (§8.3.2.3), canvas space has
//! it increasing *downward*, and a page may additionally carry `/Rotate 90`, in
//! which case screen-up is page-**left**.
//!
//! Three facts, and this module knows exactly one of them: **screen-up is
//! negative y in canvas space.** So Up builds the canvas-space vector
//! `(0, -step)` and hands it to [`super::page_delta`], which is the function the
//! pointer drag already uses and which composes the flip and the rotation
//! together by inverting the page's own device transform. Nothing here writes a
//! minus sign into a `dy`, nothing here reads `/Rotate`, and a page turned on
//! its side nudges in the direction the operator pressed **for free**.
//!
//! ⇒ The precedent is [`crate::canvas::annotdrag::drag`], which converts its
//! pointer travel the same way and for the same reason, one line before it
//! raises the same action. A wrong sign here would be invisible in a unit test
//! that asserted `dy > 0.0` and obvious in one second of use; this arrangement
//! makes the assertion *"Up produces the same delta a one-point upward drag
//! produces"*, which is a claim about agreement between two surfaces rather than
//! about arithmetic.
//!
//! ## ★★★ The step, and which program it is borrowed from
//!
//! **One PDF point bare, a quarter point with Ctrl. That is Acrobat's
//! convention**, and it is chosen over the drawing programs' for one reason that
//! is local to this canvas.
//!
//! | program | bare arrow | modifier |
//! |---|---|---|
//! | **Acrobat** | 1 pt | a modifier gives a **smaller** step |
//! | Illustrator / InDesign | the keyboard increment | **Shift** gives a **10×** step |
//! | Inkscape | 2 px | Shift 10×, Alt 1 px |
//!
//! ⇒ **Shift is spoken for on this canvas and must not be given a second
//! meaning.** It constrains a drag to one axis
//! ([`crate::canvas::constrain`], applied above both branches of the annotation
//! fork), it locks a node drag to one axis
//! ([`crate::canvas::annotnodes`]), and it makes a resize uniform
//! ([`crate::canvas::scaling`]). Three gestures, one meaning — *this movement
//! shall be along one axis* — and a fourth gesture in which it meant *ten times
//! further* would be the chord that means two things, which is worse than a
//! missing chord.
//!
//! ★ There is a tempting counter-argument and it is worth writing down so it is
//! not re-made: an arrow key is **already** axis-locked by construction, so
//! Shift's existing meaning is vacuous for a nudge and the chord is "free". That
//! is true and it is not enough. What the operator learns is *Shift constrains*;
//! a Shift that multiplied would teach them that Shift means whatever the
//! current gesture felt like, which is the thing a convention exists to prevent.
//!
//! ★★ **Alt is spoken for too**, and mechanically rather than by convention:
//! the built-in keymap binds `Alt+Up` and `Alt+Down` to `pages.move_up` and
//! `pages.move_down`. So this module refuses **any** modifier shape but the two
//! it claims, and does so by reading the modifiers itself rather than trusting
//! [`egui::InputState::consume_key`] — that function matches with
//! `Modifiers::matches_logically`, which **ignores extra Shift and Alt**, so a
//! pattern of `NONE` would have fired on `Alt+Up` and nudged a mark while
//! reordering a page. See [`step_for`].
//!
//! ## ★ One undo entry per keypress, including auto-repeat
//!
//! `egui`'s `key_pressed` counts key-repeat events, and this raises one
//! `AnnotAction::Move` per press — so holding the key walks the mark across the
//! sheet a point at a time and leaves one undo entry per point. That is what
//! Illustrator, InDesign and Acrobat all do, and the alternative (coalescing a
//! held key into one entry) needs a notion of *gesture end* that a keyboard does
//! not offer without a timer. It is a nice-to-have and it is deliberately not
//! built: correctness first, and one press → one entry is the correct half.
//!
//! ## conventions: drag-moves
//!
//! Corpus: `ui-conventions/drag-moves.md`. A keystroke is not a drag, and five
//! of the nine rows are about the interval between press and release that a
//! keystroke does not have. Each is waived with its reason rather than being
//! skipped, because a waiver that is not written down is indistinguishable from
//! an oversight — which is the whole argument of `tools/gates/conventions.list`.
//!
//! - D1 live-preview: WAIVED — there is no interval to preview. The press *is*
//!   the commit; the mark is redrawn one point over, which is the whole change.
//!   A ghost would be a preview of a thing that has already happened.
//! - D2 derived-from-commit: satisfied trivially and for the same reason — there
//!   is one code path, and it is the commit. Nothing is drawn that the commit
//!   did not do.
//! - D3 escape-cancels: WAIVED — there is nothing in flight for Escape to
//!   cancel. Escape's six claimants are listed in [`crate::canvas::keys`] and a
//!   nudge is not among them; the way back is `Ctrl+Z`, which is what every
//!   editor offers for a completed edit.
//! - D4 one-gesture-one-undo: satisfied. One press raises exactly one
//!   `AnnotAction::Move`, which `EditSession::move_annotation` records as one
//!   command. See the section above on auto-repeat, which is the one place this
//!   row needed a decision rather than an observation.
//! - D5 modifiers-constrain: **satisfied by having nothing to constrain.** An
//!   arrow is one axis by construction, so Shift is deliberately unbound here
//!   rather than given the drawing programs' 10× meaning — the argument is in
//!   full above, and it is the row this module thought hardest about.
//! - D6 snapping: WAIVED — a nudge is a fixed step in the page's own units, not
//!   a pointer looking for something to land on. Snapping the *result* to a grid
//!   would make the step size depend on where the mark already was, which is the
//!   opposite of what an operator asks a nudge for.
//! - D7 no-op-is-not-an-edit: satisfied by construction. Every accepted press
//!   has a non-zero step, so there is no zero-travel case to filter; a press
//!   that refuses raises no action at all.
//! - D8 grab-point: WAIVED — there is no grab. The delta is relative, so the
//!   mark keeps its relationship to everything around it, which is what this row
//!   protects.
//! - D9 disclosure: **satisfied, and it is the row this module adds to.** A
//!   translation changes no measured value and the operator can see where the
//!   mark went, exactly as [`super`] argues for a drag — but a *refused* nudge
//!   shows nothing at all, so all four refusals put a sentence on the status row
//!   through [`crate::text::arrange`]. A drag that refuses has a pointer the
//!   operator can see not working; a key that refuses has nothing.

use egui::Key;
use pdfcer_core::page_tree::Page;

use crate::app::actions::Action;
use crate::app::actions::annot::AnnotAction;
use crate::app::modes::Capabilities;
use crate::canvas::selection::{AnnotKind, SelectionState};
use crate::text::arrange::NudgeRefusal;

/// **The bare-arrow step, in PDF points.**
///
/// One point, which is Acrobat's and is also the unit PDF user space is defined
/// in — so an operator who nudges four times has moved four points, and the
/// number they would type into a properties box is the number of presses they
/// made. A step defined in *pixels* would be zoom-dependent and would move a
/// different distance on the page at every magnification, which is the property
/// [`super::page_delta`]'s own header is at pains to avoid for the pointer.
pub(crate) const STEP_PT: f32 = 1.0;

/// **The Ctrl-arrow step, in PDF points.**
///
/// A quarter point — Acrobat's *smaller with a modifier*, given a value. A
/// quarter rather than a tenth because a tenth of a point is below what any
/// renderer will show as a difference at ordinary zoom, so it would read as a
/// key that did nothing; a quarter is visible at 400 % and is the finest step
/// that is honest.
pub(crate) const FINE_STEP_PT: f32 = 0.25;

/// The trace line a nudge writes.
///
/// ★ Suffixed rather than bare, per `tools/gates/check-trace-names.py`: the edit
/// funnel writes `move-annotation …` for the same edit, and two lines sharing a
/// first token means `ui-verify`'s `last(name)` returns the wrong one.
// ui-text-exempt: diagnostic trace name, never displayed
const TRACE: &str = "annot-nudge";

/// **Which step this frame's modifiers ask for**, or `None` for a modifier
/// shape this module does not claim.
///
/// # ★★★ Why the modifiers are read rather than matched by `consume_key`
///
/// [`egui::InputState::consume_key`] matches with
/// [`egui::Modifiers::matches_logically`], whose documented behaviour is that
/// **extra Shift and Alt modifiers are ignored**. So `consume_key(NONE,
/// ArrowUp)` fires for `Shift+Up`, for `Alt+Up` and for `Ctrl+Alt+Shift+Up`.
///
/// `Alt+Up` is bound in the built-in keymap to `pages.move_up`. A nudge written
/// the obvious way would therefore have moved the selected mark **and** the page
/// it is on, from one press, and the second effect would have been invisible in
/// any unit test that injected a bare arrow.
///
/// ⇒ So the shapes are enumerated here, exhaustively and exclusively, and
/// anything else declines. `command` rather than `ctrl` for the fine step: it is
/// Ctrl everywhere and Cmd on macOS, which is `crate::app::keyboard`'s standing
/// rule for every chord this shell reads.
#[must_use]
fn step_for(modifiers: egui::Modifiers) -> Option<f32> {
    if modifiers.shift || modifiers.alt {
        // Both are spoken for — Shift by the axis constraint, Alt by
        // `pages.move_up`. See the module header.
        return None;
    }
    if modifiers.command {
        Some(FINE_STEP_PT)
    } else {
        Some(STEP_PT)
    }
}

/// **Which direction an arrow means, in CANVAS space.**
///
/// The one place a sign is written in this module, and it is a screen fact
/// rather than a PDF one: canvas space is y-down, so *up* is negative. The flip
/// into PDF's y-up and any page rotation are [`super::page_delta`]'s, which is
/// the whole point of routing through it — see the module header's section on
/// the Y sign.
#[must_use]
const fn direction(key: Key) -> Option<(f32, f32)> {
    match key {
        Key::ArrowUp => Some((0.0, -1.0)),
        Key::ArrowDown => Some((0.0, 1.0)),
        Key::ArrowLeft => Some((-1.0, 0.0)),
        Key::ArrowRight => Some((1.0, 0.0)),
        _ => None,
    }
}

/// Everything a nudge needs that is not the keyboard.
///
/// A struct for [`crate::canvas::keys::Keys`]' stated reason: the list reached
/// five, three of them are `bool`-ish or reference-shaped, and transposing two
/// would compile.
pub(crate) struct Frame<'a> {
    /// The page the selection is on, for the coordinate crossing. `None` for a
    /// frame with no page, which declines rather than guessing.
    pub page: Option<&'a Page>,
    /// What the mode permits. `author_markup` is the gate, matching where the
    /// pointer drag's gate is and where the Delete key's is.
    pub caps: Capabilities,
    /// The revision on screen, for stamping a refusal's sentence. See
    /// [`crate::app::actions::record_note`] on why the *current* epoch is the
    /// right one for a non-edit.
    pub edit_epoch: u64,
}

/// **Read this frame's arrow keys and nudge the selected markup.**
///
/// Returns the number of `AnnotAction::Move`s raised, which is zero on every
/// frame that carries no arrow — the overwhelmingly common case, and it costs
/// one modifier read and four event scans.
///
/// # ★★★ The guard, and it is the worst key in the application to get wrong
///
/// `DEFECTS.md` D1 is *"I can't even click on an object and delete it by hitting
/// the delete key"*, and its cause was a guard that asked **"is any widget
/// focused?"** where it meant **"is a text field focused?"**. Its second
/// instance cost the operator the space bar while typing on the canvas.
///
/// An arrow key is the worst key on which to repeat that, because moving a caret
/// is *what an arrow key is for*. There are two claimants and this module must
/// yield to both:
///
/// | claimant | seen by | where it is asked |
/// |---|---|---|
/// | a real `egui::TextEdit` — a form field, the page box, the Find bar | `Context::text_edit_focused` | [`crate::canvas::keys::canvas_keys`]'s first line, which returns before this is reached |
/// | the **canvas caret**, which is deliberately not a widget | nothing egui offers | [`crate::canvas::textedit::composing`], asked here |
///
/// ⇒ So the predicate asked here is `composing`, the wide one — the single
/// implementation `tools/gates/check-typing-guard.sh` exists to keep single —
/// and **not** `text_edit_focused`, which would be the founding defect's
/// spelling and would answer `false` for an operator who is visibly mid-word.
///
/// ★ It is asked here rather than relied on from the caller even though
/// `canvas_keys` has already returned for the *widget* half. The two halves are
/// different questions with different answers, the caller's guard is
/// deliberately the narrow one so that Escape can still reach the draft-abandon
/// rung, and a module that assumed otherwise would be reading a comment in
/// another file as a contract.
pub(crate) fn keys(
    ctx: &egui::Context,
    frame: &Frame<'_>,
    selection: &SelectionState,
    actions: &mut Vec<Action>,
) -> usize {
    // ★ The canvas caret owns the arrow keys while a draft is in flight. See
    // this function's own table for why the predicate is `composing` and not
    // `text_edit_focused`, and why asking it here is not a second copy of the
    // caller's guard.
    if crate::canvas::textedit::composing(ctx) {
        return 0;
    }

    let modifiers = ctx.input(|i| i.modifiers);
    let Some(step) = step_for(modifiers) else {
        return 0;
    };

    // ★ Counted per key rather than answered yes/no, because `key_pressed`
    // includes key-repeat events and a held arrow must walk the mark rather
    // than move it once. `count_and_consume_key` does both halves in one call:
    // it reports how many presses arrived and takes them out of the queue, so
    // nothing behind this — a scroll area, a focused segment — acts on the same
    // press. `egui-shell`'s mode selector states the rule in its own words:
    // *"an arrow that moved the selector must not also scroll whatever is
    // behind it."*
    let mut raised = 0;
    for key in [
        Key::ArrowUp,
        Key::ArrowDown,
        Key::ArrowLeft,
        Key::ArrowRight,
    ] {
        let Some((ux, uy)) = direction(key) else {
            continue;
        };
        // ★ The pattern is the modifiers actually held, not a constant. See
        // `step_for` for why a constant `NONE` would have matched `Alt+Up`.
        let count = ctx.input_mut(|i| i.count_and_consume_key(modifiers, key));
        if count == 0 {
            continue;
        }
        for _ in 0..count {
            if nudge_once(frame, selection, egui::vec2(ux * step, uy * step), actions) {
                raised += 1;
            } else {
                // A refusal is a property of the selection and the mode, not of
                // this particular press, so the remaining repeats of a held key
                // would refuse identically. Reporting once is the honest count
                // and stops a held arrow writing the same sentence forty times.
                return raised;
            }
        }
    }
    raised
}

/// One press: decide, refuse with a sentence, or raise the move.
///
/// Returns whether an action was raised, which is what lets the caller stop
/// repeating a refusal.
fn nudge_once(
    frame: &Frame<'_>,
    selection: &SelectionState,
    canvas: egui::Vec2,
    actions: &mut Vec<Action>,
) -> bool {
    let Some(annot) = selection.annot() else {
        // ★★ Two states share this branch and only one of them is silent, which
        // is the split `crate::text::arrange::NudgeRefusal`'s own doc argues:
        //
        // * **nothing selected** — silent. The arrow keys are pressed
        //   constantly for reasons that are not about a selection, and a bar
        //   that said "select something first" on every stray press would stop
        //   being read.
        // * **page content selected** — a sentence. The operator picked an
        //   object out of the drawing and pressed a key every drawing program
        //   binds; silence there is the shape this project keeps finding.
        if selection.is_empty() {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!("{TRACE}-declined reason=nothing-selected")
            });
        } else {
            refuse(frame, selection, NudgeRefusal::NotAMarkup);
        }
        return false;
    };

    // ★ Rule 15, guarded by the `AnnotKind` match the compiler checks rather
    // than by a `/Subtype` string. A **ce dimension** is pdfcer-authored, it is
    // a `/Line` carrying `/IT /LineDimension` with a record in `/PieceInfo`, and
    // moving one may have to RE-MEASURE it — which `move_annotation` does not do
    // and refuses by name (`AnnotationMoveWrongVerb`, naming `move_dimension`).
    // A **pdf dimension** is CAD-exported page content and is not an annotation
    // at all, so it cannot reach this branch: it has no `AnnotTarget`.
    //
    // The pointer's fork makes the same distinction one module over —
    // `canvas::dimdrag` claims a ce dimension and `canvas::annotdrag` claims
    // everything else — so this is that fork restated for the keyboard rather
    // than a new rule. Routing a ce dimension here instead would be the
    // "silently does less under this name" defect the engine's own comment
    // refuses.
    match annot.target.kind {
        AnnotKind::Markup => {}
        AnnotKind::CeDimension => {
            refuse(frame, selection, NudgeRefusal::Dimension);
            return false;
        }
    }

    // §12.5.3 Table 165 bit 8. Honoured HERE rather than left to the engine —
    // which does not check it for a move at all — for `annotdrag::eligible`'s
    // stated reason: a key and a drag must not disagree about what locked means.
    if annot.target.locked {
        refuse(frame, selection, NudgeRefusal::Locked);
        return false;
    }

    // ★ The mode gate, and it is `author_markup` rather than `edit_content`.
    //
    // **Review** is the markup stance — `edit_content: false, author_markup:
    // true` — and it is the mode an operator is in *because* they are working on
    // comments. `canvas::keys`' Delete rung learned this the hard way: it sat
    // below an `edit_content` gate and Review was the one mode in which Delete
    // could not remove a comment. One predicate per capability.
    if !frame.caps.author_markup {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("{TRACE}-declined reason=mode-cannot-author-markup")
        });
        return false;
    }

    // ★ The coordinate crossing, through the one function that owns it. See the
    // module header on the Y sign: everything about the flip and the page's
    // `/Rotate` is inside `page_delta`, and a page whose transform will not
    // invert declines here exactly as a drag on it declines.
    let Some(delta) = frame.page.and_then(|page| super::page_delta(canvas, page)) else {
        refuse(frame, selection, NudgeRefusal::DegeneratePage);
        return false;
    };

    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "{TRACE} id={} dx={:.3} dy={:.3}",
            annot.target.id.num, delta.dx, delta.dy
        )
    });
    actions.push(Action::Annot(AnnotAction::Move {
        id: annot.target.id,
        dx: delta.dx,
        dy: delta.dy,
    }));
    true
}

/// Say why a nudge did nothing — on the status row **and** on the trace.
///
/// ★ Both, not one. The trace is what a driven check reads and what a harness on
/// a machine nobody can see reports from; the status row is what the operator
/// reads. They carry the same fact in two registers, and neither substitutes for
/// the other — `crate::canvas::deleting::decline` is the same shape and states
/// the same reason.
fn refuse(frame: &Frame<'_>, selection: &SelectionState, why: NudgeRefusal) {
    if let Some(sentence) = crate::text::arrange::nudge_refusal(why) {
        // ★ The CURRENT epoch, not a new one. The sentence stands from now until
        // the next real edit moves past it, and is retired without anything
        // having to remember to — `app::actions::disclosure::record_note`'s
        // contract for a non-edit.
        crate::app::actions::record_note(frame.edit_epoch, sentence.to_owned());
    }
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "{TRACE}-declined level={:?} sel={} reason={why:?}",
            selection.level(),
            selection.len(),
        )
    });
}

#[cfg(test)]
mod tests;
