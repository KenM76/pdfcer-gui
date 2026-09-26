//! # `app::status::decline::floor` — **the funnel's last word, and only if the
//! verb had none**
//!
//! One type, and it is here rather than in [`super`] for **R2**: that file
//! reached 1,530 lines when `OPERATOR_REQUESTS.md` O116's variant, its
//! retirement argument and this guard arrived together, and the gate's own
//! ruling is that the response to it firing is to split the module, not to
//! shrink the prose.
//!
//! ## Why THIS is the seam, out of everything that file holds
//!
//! Because it is the one part of `decline` that is not about *what a decline
//! is*. [`super`] answers three questions — what may be declined, how long a
//! sentence lives, and what it says — and every one of its fifteen recorders is
//! a different answer to the first. This answers a fourth question that belongs
//! to a **different module's protocol**: *when `crate::app::actions::funnel`
//! runs a verb and the verb refuses, who gets to speak?* It grows when that
//! protocol changes, which is roughly never, and it is the only thing in the
//! file with a lifetime — a value held across somebody else's call.
//!
//! It reaches into the parent for `LAST` and [`Declined`], which is exactly
//! what a child module is for and is why this is a submodule rather than a
//! sibling: the store stays private to `decline`, and nothing outside it can
//! write the slot without going through a recorder or through this.
//!
//! [`Declined`]: super::Declined
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/status/decline/floor.md`.

use super::{Declined, LAST};

/// **The floor under every edit: the verb speaks first, and if it says
/// nothing the funnel says the un-categorised thing** — `OPERATOR_REQUESTS.md`
/// O116.
#[derive(Debug)]
pub(crate) struct BeforeTheVerb(Option<Declined>);

/// Take the slot and hold it for the duration of the verb call.
#[must_use]
pub(crate) fn before_the_verb() -> BeforeTheVerb {
    BeforeTheVerb(LAST.with_borrow_mut(Option::take))
}

impl BeforeTheVerb {
    /// **The verb succeeded.** Put back whatever was live before it ran, unless
    /// the verb itself recorded something.
    pub(crate) fn granted(self) {
        LAST.with_borrow_mut(|slot| {
            if slot.is_none() {
                *slot = self.0;
            }
        });
    }

    /// **The verb refused.** Say the un-categorised sentence, unless the verb
    /// already said something better.
    pub(crate) fn refused(self) {
        LAST.with_borrow_mut(|slot| {
            if slot.is_none() {
                *slot = Some(Declined::EditRefused);
            }
        });
    }
}
