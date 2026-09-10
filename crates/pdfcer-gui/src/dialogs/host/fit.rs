//! # `dialogs::host::fit` — growing a window to its body, without a loop
//!
//! Split out of [`super`] on 2026-09-10 when that file reached 1,600 lines and
//! tripped R2. The seam is a real subject rather than a convenient cut: every
//! item here exists because **an OS window has to be created at some size**,
//! and nine of the thirteen dialogs converted on 2026-08-21 had no size written
//! down anywhere — their layout *was* the number.
//!
//! ## ★★★ The defect this module is shaped by
//!
//! The first version of the grow-to-fit rule padded the measured content before
//! comparing it to the window, so `want` was always larger than `inner`, every
//! frame asked for eight more pixels than the last, and the once-per-size guard
//! did not help **because every size was a new one**. The About window opened at
//! 560 x 480 and was 1624 x 746 by the time the trace was read.
//!
//! ⇒ The standing lesson, recorded across this project: *a guard that stops
//! repetition does not stop creep*. A measurement fed back into a size needs a
//! **direction bound** ([`fit_target`] only ever grows) and a **floor**
//! ([`FIT_MARGIN`]) — and, because neither can detect a content size that moves
//! with the window, a **budget** ([`FIT_BUDGET`]) that stops asking.
//!
//! ## Why the arithmetic is a free function
//!
//! [`Host::fit`] needs a live viewport to read `inner_rect` and to issue the
//! resize, so the whole decision used to be reachable headlessly only in its
//! no-op half. That was true of the *resize* and never true of the
//! *arithmetic*: [`fit_target`] is pure, so the convergence test below can feed
//! it its own output and require a fixed point.
//!
//! ## Rule 15
//!
//! Every size in this module is a **window** size in points — neither a ce
//! dimension nor a pdf dimension appears anywhere in it.

use egui::Vec2;

use super::Host;

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

impl Host {
    /// **Forget everything [`Self::fit`] learned about the last opening.**
    ///
    /// Called from [`Self::show`] on the pass a dialog opens. See the call site
    /// for the operator report that made it necessary; the short version is
    /// that `fit`'s two guards are scoped to *one opening* and were living in
    /// memory that outlives the window.
    pub(super) fn forget_fit(&self, ctx: &egui::Context) {
        ctx.data_mut(|d| {
            d.remove::<Vec2>(self.fit_key);
            d.remove::<usize>(self.budget_key);
        });
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
    pub(super) fn fit(&self, child: &egui::Context, content: Vec2) {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    /// How many frames the divergence demonstration runs for. Must exceed
    /// [`FIT_BUDGET`]; see the compile-time assertion below.
    const DIVERGENCE_ROUNDS: usize = 12;

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

    /// ★★★ **A reopened dialog is fitted from scratch** — the operator's
    /// *"the second stamp's window is too small to show Add"*, 2026-09-10.
    ///
    /// Both of [`Host::fit`]'s guards live in `egui::Memory` keyed on the
    /// dialog's id, and both are statements about **one opening**:
    ///
    ///   * `fit_key` says *"I have already asked for this size"*, which on a
    ///     brand-new window at its opening bid is false and suppressed the
    ///     only resize that mattered;
    ///   * `budget_key` counts growths, and a cumulative count means the
    ///     fourth opening of a dialog in a session can never grow at all.
    ///
    /// The remembered POSITION deliberately survives a close (G6). The fit
    /// state deliberately does not, and this is the line that says which is
    /// which.
    ///
    /// ⚠ What this test does NOT cover: that [`Host::show`] actually calls
    /// [`Host::forget_fit`] on the opening pass. That needs a live viewport,
    /// so it belongs to the driven check. The mechanism is asserted here; the
    /// wiring is asserted there.
    #[test]
    fn a_reopened_dialog_forgets_the_last_opening_s_fit() {
        let ctx = egui::Context::default();
        let h = Host::new("print", "Print", Vec2::new(400.0, 300.0), Vec2::splat(10.0));

        // Stand where the end of a previous opening left this dialog: a size
        // already asked for, and the whole growth budget spent.
        ctx.data_mut(|d| {
            d.insert_temp(h.fit_key, Vec2::new(520.0, 300.0));
            d.insert_temp(h.budget_key, FIT_BUDGET);
        });

        h.forget_fit(&ctx);

        assert!(
            ctx.data(|d| d.get_temp::<Vec2>(h.fit_key)).is_none(),
            "a new window has not asked for any size yet"
        );
        assert!(
            ctx.data(|d| d.get_temp::<usize>(h.budget_key)).is_none(),
            "the growth budget is per opening, not per session"
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
}
