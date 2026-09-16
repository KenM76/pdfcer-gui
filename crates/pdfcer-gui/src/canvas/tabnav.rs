//! # `canvas::tabnav` — Tab moves through what the operator clicked on
//!
//! `OPERATOR_REQUESTS.md` O204:
//!
//! > *"when I press tab while in a form I end up tabbing through the menus
//! > instead of the form items. The tab should tab through whatever space I
//! > have clicked on (example if I have an object selected on the canvase it
//! > should tab through to the next object as expected, and if I've clicked on
//! > a form item it should tab forward and shift-tab backwards to the next
//! > one."*
//!
//! ## Contract
//!
//! Two halves, and they are in different parts of the frame.
//!
//! 1. **Before egui sees the input.** [`claim`] runs from
//!    `eframe::App::raw_input_hook`, removes the Tab key event from the raw
//!    input, and parks a [`Request`]. It claims only when the widget that egui
//!    says currently holds focus is the one the canvas [`publish`]ed last pass.
//! 2. **During the canvas's own drawing.** The owning surface calls [`take`],
//!    gets the [`Request`], and moves its own focus.
//!
//! A surface that wants the ring therefore does exactly two things: publish the
//! `egui::Id` of the widget it has focused, every pass it draws it; and take
//! and act on the request.
//!
//! ## Why the seam is `raw_input_hook` and nothing else can work
//!
//! egui latches the focus move in `Focus::begin_pass` **from the `RawInput`
//! events**, before any application `ui` code runs (`egui::memory`, the
//! `Key::Tab` arms of `begin_pass`). By the time a widget could call
//! `ctx.input_mut(|i| i.consume_key(…))`, `focus_direction` is already set and
//! the walk to the next focusable widget is already going to happen. Consuming
//! the key later removes the *evidence* and not the *effect* — which is the
//! shape of fix that passes a unit test and leaves the operator tabbing through
//! the ribbon.
//!
//! ## Why ownership is an identity test and not a flag
//!
//! [`claim`] asks whether `memory.focused()` **is** the published id. That is
//! stronger than "the canvas thinks it has a field open", and it is stronger in
//! the two ways that matter: a dialog, a panel text box or a ribbon search
//! field that has taken focus makes the test fail, so Tab goes where the
//! operator is actually typing; and a canvas that stopped being drawn — a dock
//! tab switched, the panel collapsed — cannot leave a stale claim behind,
//! because egui's own end-of-pass dead-man's switch drops the focus of an id
//! that was not used. **The identity test is also the freshness test**, which
//! is why no pass counter appears in [`Owner`].

use egui::Id;

/// Which ring a request is for.
///
/// Carried so the two canvas rings can share one seam without either acting on
/// the other's press: a form field and a selected page object are focused by
/// different code and move by different rules (O204 decision 3 — the field ring
/// crosses pages, the object ring wraps within one).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// A form field being filled on the page.
    Field,
    /// An object selected on the page.
    Object,
}

/// The canvas widget that currently holds focus, as the canvas sees it.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Owner {
    scope: Scope,
    id: Id,
}

/// A Tab press the canvas has taken off egui.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Request {
    /// Which ring asked for it — always the scope that was published.
    pub scope: Scope,
    /// `true` for Shift+Tab.
    pub backwards: bool,
}

/// `ctx.data` key for the published owner.
const OWNER_KEY: &str = "pdfcer-canvas-tabnav-owner"; // ui-text-exempt: internal data key, never displayed

/// `ctx.data` key for a claimed press waiting to be taken.
const REQUEST_KEY: &str = "pdfcer-canvas-tabnav-request"; // ui-text-exempt: internal data key, never displayed

/// **Declare that `id` is the canvas widget holding keyboard focus.**
///
/// Called every pass the widget is drawn. Cheap by construction: one
/// `ctx.data_mut` insert of a `Copy` value, which is the same cost as the
/// focus-tracking every other surface in this shell does.
pub fn publish(ctx: &egui::Context, scope: Scope, id: Id) {
    ctx.data_mut(|d| d.insert_temp(Id::new(OWNER_KEY), Owner { scope, id }));
}

/// **Give up ownership**, so no further press is claimed for it.
///
/// Published ownership is otherwise dropped by egui itself: a widget that
/// stops being drawn stops being focused, and [`owner`]'s identity test then
/// rejects the stale entry. This exists for the case that test cannot see —
/// a surface that is still drawn, still focused, and has handed the key to
/// somebody else. The measure tools are that case: Tab cycles the snap mode
/// while one is armed, and the owner is the same page id either way, so
/// merely declining to re-publish would leave the hook swallowing a press
/// with nothing to spend it on.
pub fn release(ctx: &egui::Context) {
    ctx.data_mut(|d| d.remove::<Owner>(Id::new(OWNER_KEY)));
}

/// The published owner, if it still holds egui's keyboard focus.
///
/// The identity test the module header argues for, in one place because two
/// callers need it: [`claim`], and [`owns_focus`] for the surfaces that have to
/// stand aside from a key the ring is about to read.
fn owner(ctx: &egui::Context) -> Option<Owner> {
    let owner = ctx.data(|d| d.get_temp::<Owner>(Id::new(OWNER_KEY)))?;
    (ctx.memory(|m| m.focused()) == Some(owner.id)).then_some(owner)
}

/// **Whether `scope`'s canvas ring currently holds the keyboard.**
///
/// For the one place outside this module that must ask: the space bar is the
/// canvas's hand-tool modifier, and a focused form button reads Space as
/// *toggle me*. Without this the bar pans the paper and a checkbox cannot be
/// ticked from the keyboard at all — the same shape as the operator's
/// *"it doesn't accept spaces"* about the text caret.
///
/// Not a second spelling of the typing guard
/// (`crate::canvas::textedit::composing`): that one answers *is the operator
/// composing text*, this one answers *does this canvas ring own the keyboard*,
/// and a focused push button is the case where those differ.
///
/// # Why it takes a scope rather than answering for the canvas as a whole
///
/// A focused PAGE owns Tab and nothing else — the space bar is still the hand
/// tool, which is the gesture the operator uses most on a drawing. Asking the
/// unscoped question would hand Space to the object ring, which has no use for
/// it, and stop the paper panning the moment a page was clicked.
#[must_use]
pub fn owns_focus(ctx: &egui::Context, scope: Scope) -> bool {
    owner(ctx).is_some_and(|owner| owner.scope == scope)
}

/// **Take the Tab press, if the canvas owns it.**
///
/// Called from `eframe::App::raw_input_hook` — see the module header for why
/// that and only that. Removes every `Key::Tab` event from `raw_input` when it
/// claims, so egui never sees one and `Focus::begin_pass` never latches a
/// direction.
///
/// Ctrl+Tab and every other modified Tab are left alone: they belong to the
/// dock, and a ring that swallowed them would take a chord it was never asked
/// for.
pub fn claim(ctx: &egui::Context, raw_input: &mut egui::RawInput) {
    let Some(owner) = owner(ctx) else {
        return;
    };
    let mut backwards = None;
    raw_input.events.retain(|event| {
        let egui::Event::Key {
            key: egui::Key::Tab,
            pressed,
            modifiers,
            ..
        } = event
        else {
            return true;
        };
        if modifiers.any() && !modifiers.shift_only() {
            return true;
        }
        if *pressed {
            backwards = Some(modifiers.shift_only());
        }
        false
    });
    let Some(backwards) = backwards else {
        return;
    };
    // A request still sitting here is one the owning surface did not take,
    // which means it published an id it does not act on. Traced rather than
    // silently overwritten: the symptom of the alternative is a Tab that does
    // nothing, which nobody can debug from the outside.
    let id = Id::new(REQUEST_KEY);
    let stale = ctx.data_mut(|d| {
        let previous = d.get_temp::<Request>(id);
        d.insert_temp(
            id,
            Request {
                scope: owner.scope,
                backwards,
            },
        );
        previous
    });
    if let Some(previous) = stale {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("tab-claim-unconsumed backwards={}", previous.backwards)
        });
    }
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "tab-claim scope={} backwards={backwards}",
            match owner.scope {
                Scope::Field => "field",
                Scope::Object => "object",
            }
        )
    });
}

/// **Take this pass's claimed press, if it is for `scope`.**
///
/// Consumes unconditionally when the scope matches. A request is parked at the
/// start of a frame by [`claim`], which only claims when the owning surface is
/// drawn and focused — so the surface that could take it is guaranteed to run
/// in that frame, and a request that outlives the frame would be a bug rather
/// than a press to replay.
pub fn take(ctx: &egui::Context, scope: Scope) -> Option<Request> {
    let id = Id::new(REQUEST_KEY);
    let request = ctx.data(|d| d.get_temp::<Request>(id))?;
    if request.scope != scope {
        return None;
    }
    ctx.data_mut(|d| d.remove::<Request>(id));
    Some(request)
}

/// Drop any claimed press without acting on it.
///
/// For the surface that took ownership and then found it had nothing to move
/// to — an empty ring, a field that vanished under an undo. Leaving the request
/// parked would let the *next* frame's surface act on a press aimed at this
/// one.
pub fn discard(ctx: &egui::Context) {
    ctx.data_mut(|d| d.remove::<Request>(Id::new(REQUEST_KEY)));
}

/// **Where a Tab press lands, given per-page rings.**
///
/// `rings` is `(page index, number of stops on that page)`, sorted ascending by
/// page, with no empty entries. `at` is `(page, index within that page's
/// ring)`. `cross` is O204 decision 3: `true` walks off the end of one page's
/// ring onto the next page's, `false` wraps within the page.
///
/// Returns `None` only when `at` names a page that has no ring — a focus that
/// has gone, which the caller settles rather than moves.
///
/// Wrapping is unconditional in both modes: the last stop leads to the first.
/// A ring that stopped at its end would make the operator's recovery from an
/// over-press a mouse gesture, and the convention across every program that
/// tabs through fields is that it does not.
#[must_use]
pub fn step(
    rings: &[(usize, usize)],
    at: (usize, usize),
    backwards: bool,
    cross: bool,
) -> Option<(usize, usize)> {
    let here = rings.iter().position(|(page, _)| *page == at.0)?;
    let len = rings[here].1;
    if !backwards {
        if at.1 + 1 < len {
            return Some((at.0, at.1 + 1));
        }
        if !cross {
            return Some((at.0, 0));
        }
        let next = (here + 1) % rings.len();
        return Some((rings[next].0, 0));
    }
    if at.1 > 0 {
        // A stop past the end of a shrunken ring steps to the ring's last.
        return Some((at.0, at.1.saturating_sub(1).min(len.saturating_sub(1))));
    }
    if !cross {
        return Some((at.0, len.saturating_sub(1)));
    }
    let prev = (here + rings.len() - 1) % rings.len();
    Some((rings[prev].0, rings[prev].1.saturating_sub(1)))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Three pages carrying two, one and three stops.
    const RINGS: &[(usize, usize)] = &[(0, 2), (2, 1), (5, 3)];

    #[test]
    fn a_tab_within_a_page_takes_the_next_stop() {
        assert_eq!(step(RINGS, (0, 0), false, true), Some((0, 1)));
        assert_eq!(step(RINGS, (5, 1), false, true), Some((5, 2)));
    }

    #[test]
    fn a_shift_tab_within_a_page_takes_the_previous_stop() {
        assert_eq!(step(RINGS, (0, 1), true, true), Some((0, 0)));
        assert_eq!(step(RINGS, (5, 2), true, true), Some((5, 1)));
    }

    /// O204 decision 3, the field half: the ring crosses pages.
    #[test]
    fn a_crossing_ring_walks_onto_the_next_page() {
        assert_eq!(step(RINGS, (0, 1), false, true), Some((2, 0)));
        assert_eq!(step(RINGS, (2, 0), false, true), Some((5, 0)));
        // …and off the last page back onto the first.
        assert_eq!(step(RINGS, (5, 2), false, true), Some((0, 0)));
    }

    #[test]
    fn a_crossing_ring_walks_backwards_onto_the_previous_pages_last_stop() {
        assert_eq!(step(RINGS, (2, 0), true, true), Some((0, 1)));
        assert_eq!(step(RINGS, (5, 0), true, true), Some((2, 0)));
        assert_eq!(step(RINGS, (0, 0), true, true), Some((5, 2)));
    }

    /// O204 decision 3, the object half: the ring stays on the page.
    #[test]
    fn a_wrapping_ring_never_leaves_its_page() {
        assert_eq!(step(RINGS, (0, 1), false, false), Some((0, 0)));
        assert_eq!(step(RINGS, (0, 0), true, false), Some((0, 1)));
        // A single-stop page is a ring of one, and lands on itself.
        assert_eq!(step(RINGS, (2, 0), false, false), Some((2, 0)));
        assert_eq!(step(RINGS, (2, 0), true, false), Some((2, 0)));
    }

    /// The focus's page has no ring at all — an undo removed every field on it.
    #[test]
    fn a_page_with_no_ring_has_no_next_stop() {
        assert_eq!(step(RINGS, (1, 0), false, true), None);
        assert_eq!(step(&[], (0, 0), false, true), None);
    }

    /// An index past the end of a ring that shrank between frames steps to a
    /// real stop rather than off it.
    #[test]
    fn an_index_past_the_end_of_a_shrunken_ring_lands_inside_it() {
        assert_eq!(step(RINGS, (2, 7), false, true), Some((5, 0)));
        assert_eq!(step(RINGS, (2, 7), true, true), Some((2, 0)));
    }
}
