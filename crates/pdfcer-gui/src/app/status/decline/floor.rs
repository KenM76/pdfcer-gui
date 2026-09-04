//! # `app::status::decline::floor` — **the funnel's last word, and only if the
//! verb had none**
//!
//! One type, and it is here rather than in [`super`] for **R2**: that file
//! reached 1,530 lines when `OPERATOR_REQUESTS.md` O116's variant, its
//! retirement argument and this guard arrived together, and the gate's own
//! ruling is that the response to it firing is to split the module, not to
//! shrink the prose.
//!
//! ## ★ Why THIS is the seam, out of everything that file holds
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
//! ★ It reaches into the parent for `LAST` and [`Declined`], which is exactly
//! what a child module is for and is why this is a submodule rather than a
//! sibling: the store stays private to `decline`, and nothing outside it can
//! write the slot without going through a recorder or through this.
//!
//! [`Declined`]: super::Declined

use super::{Declined, LAST};

/// ★★★ **The floor under every edit: the verb speaks first, and if it says
/// nothing the funnel says the un-categorised thing** — `OPERATOR_REQUESTS.md`
/// O116.
///
/// Held across `crate::app::actions::funnel`'s call to the verb closure, and
/// it exists because that arm has a problem none of the parent's fifteen
/// recorders has: **it runs after the verb, over every verb.** A plain
/// `record_edit_refused()` in the error arm would be correct for
/// `edit_text` and destructive for six others — `record_rotate`,
/// `record_unshare`, `record_resize_not_rebuildable`,
/// `record_bookmark_move_refused`, `record_text_style` and
/// `record_adopt_refusal` are all called from **inside** the closure, so their
/// sentence is already in the slot by the time the arm runs, and an
/// unconditional write would replace *"turn on Scale line weight and this will
/// be exact"* with *"that change was refused"*.
///
/// ⇒ The rule is a **precedence**, not a suppression: a decline the verb can
/// name is always better than one it cannot, so the funnel is the last speaker
/// and must yield to anything the verb said.
///
/// # ★★ Why it takes the slot rather than merely reading it
///
/// [`before_the_verb`] **empties** the slot and holds the contents.
/// Comparing before-and-after values instead would have been simpler and
/// wrong in two directions at once:
///
/// - a verb that recorded *the same* refusal twice in a row (two rotate
///   refusals with no command between them) would compare equal, and the funnel
///   would overwrite a correct specific sentence with the vague one;
/// - a **stale** sentence left over from an earlier gesture would compare equal
///   too, and the funnel would stay silent — leaving the operator reading
///   *"that rotation was refused"* after a text commit. Two canvas gestures in
///   a row reach that state, because [`super::retire`] runs at the dispatcher
///   and a canvas-raised `Action` never passes through it.
///
/// Taking the slot makes the question exact: **is anything in here now?** can
/// only be answered *yes* by a write that happened while the guard was held.
///
/// # ★★★ Repeatability, which the take is what makes mechanical
///
/// Pressing commit twice on the same unsupported text is **two events, and the
/// second registers** — the property the module header's reason 2 is about, and
/// the one an `edit_epoch` key cannot express. It falls out of the take rather
/// than needing a rule of its own: on the second press
/// [`before_the_verb`] takes the *first* press's sentence out of the
/// slot, the verb refuses again and writes nothing, the slot is empty, and
/// [`Self::refused`] writes a genuine second record. There is no
/// "already shown, skip" gate anywhere in the path, which is exactly what
/// `crate::canvas::zoom::trace_outcome` rules for the trace channel and for the
/// same reason.
///
/// # ★ The one thing a future recorder must not do
///
/// Record a decline **before** calling `vector_edit` and expect it to survive a
/// refusal: [`before_the_verb`] will have taken it, and
/// [`Self::refused`] will replace it with the un-categorised sentence. No
/// caller does this today — the two that record before the funnel
/// (`xobject::fanout`, `actions::history`) both `return` without calling it —
/// and the placement rule the module already states is what keeps it that way:
/// *an engine-side refusal is recorded from inside the closure*, because
/// whether the engine will refuse is not knowable before the call.
#[derive(Debug)]
pub(crate) struct BeforeTheVerb(Option<Declined>);

/// Take the slot and hold it for the duration of the verb call.
///
/// ★ A free function rather than `BeforeTheVerb::new` or
/// `BeforeTheVerb::before_the_verb`, and the reason is only partly that clippy's
/// `self_named_constructors` rejects the second: the call site reads
/// `decline::before_the_verb()`, which says **when** it happens, and *when* is
/// the entire contract. `new` would say nothing at all about the one property a
/// caller has to get right.
///
/// `#[must_use]` because dropping the value without calling either
/// [`BeforeTheVerb::granted`] or [`BeforeTheVerb::refused`] would silently
/// retire whatever sentence was live — a decline vanishing on an unrelated
/// edit, with nothing anywhere to point at.
#[must_use]
pub(crate) fn before_the_verb() -> BeforeTheVerb {
    BeforeTheVerb(LAST.with_borrow_mut(Option::take))
}

impl BeforeTheVerb {
    /// **The verb succeeded.** Put back whatever was live before it ran, unless
    /// the verb itself recorded something.
    ///
    /// ★ Restoring rather than clearing keeps a successful edit's behaviour
    /// **exactly** what it was before this guard existed: nothing about an
    /// `Ok` has ever retired a decline, and this is not the place to decide
    /// that it should. `retire` owns that question and answers it at the
    /// dispatcher.
    pub(crate) fn granted(self) {
        LAST.with_borrow_mut(|slot| {
            if slot.is_none() {
                *slot = self.0;
            }
        });
    }

    /// **The verb refused.** Say the un-categorised sentence, unless the verb
    /// already said something better.
    ///
    /// What was live before is deliberately **not** restored: a refusal that
    /// has just happened supersedes a sentence about an earlier gesture, and
    /// leaving the older one up would answer this press with someone else's
    /// answer.
    pub(crate) fn refused(self) {
        LAST.with_borrow_mut(|slot| {
            if slot.is_none() {
                *slot = Some(Declined::EditRefused);
            }
        });
    }
}
