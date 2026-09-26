//! # `tabnav` — Tab moves through what the operator clicked on
//!
//! `OPERATOR_REQUESTS.md` O204:
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/tabnav.md`.

use egui::Id;

/// Which ring a request is for.
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
pub fn publish(ctx: &egui::Context, scope: Scope, id: Id) {
    ctx.data_mut(|d| d.insert_temp(Id::new(OWNER_KEY), Owner { scope, id }));
}

/// **Give up ownership**, so no further press is claimed for it.
pub fn release(ctx: &egui::Context) {
    ctx.data_mut(|d| d.remove::<Owner>(Id::new(OWNER_KEY)));
}

/// The published owner, if it still holds egui's keyboard focus.
fn owner(ctx: &egui::Context) -> Option<Owner> {
    let owner = ctx.data(|d| d.get_temp::<Owner>(Id::new(OWNER_KEY)))?;
    (ctx.memory(|m| m.focused()) == Some(owner.id)).then_some(owner)
}

/// **Whether `scope`'s canvas ring currently holds the keyboard.**
#[must_use]
pub fn owns_focus(ctx: &egui::Context, scope: Scope) -> bool {
    owner(ctx).is_some_and(|owner| owner.scope == scope)
}

/// **Take the Tab press, if the canvas owns it.**
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
pub fn discard(ctx: &egui::Context) {
    ctx.data_mut(|d| d.remove::<Request>(Id::new(REQUEST_KEY)));
}

/// **Where a Tab press lands, given per-page rings.**
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
