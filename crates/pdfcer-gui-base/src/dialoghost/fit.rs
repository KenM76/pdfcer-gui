//! # `dialoghost::fit` — growing a window to its body, without a loop
//!
//!
//! ## The defect this module is shaped by
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
//!
//! ## Rule 15
//!
//! Every size in this module is a **window** size in points — neither a ce
//! dimension nor a pdf dimension appears anywhere in it.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/dialoghost/fit.md`.

use egui::Vec2;

use super::Host;

/// How much a dialog's body must overflow its window before the window is
/// grown to fit it.
const FIT_MARGIN: f32 = 8.0;

/// How many times one dialog may be grown to fit its content before the host
/// concludes the measurement is circular and stops.
const FIT_BUDGET: usize = 3;

/// **The size a window should be grown to**, or `None` to leave it alone.
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
    pub(super) fn forget_fit(&self, ctx: &egui::Context) {
        ctx.data_mut(|d| {
            d.remove::<Vec2>(self.fit_key);
            d.remove::<usize>(self.budget_key);
        });
    }

    /// **Grow the window until the body fits**, at most once per size.
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

        // THE GROWTH BUDGET — the guard that turns a layout mistake into a
        // stopped dialog instead of one that grows without limit.
        //
        //
        // The point that took three instances to learn: **a guard against
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
        // It is also RECORDED rather than merely suppressed: silently
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

    /// **A window already big enough for its body is left alone**, which is
    /// the guard that keeps [`Host::fit`] from being a feedback loop.
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

    /// **A reopened dialog is fitted from scratch** — the operator's
    /// *"the second stamp's window is too small to show Add"*, 2026-09-10.
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

    /// **The print dialog's runaway, reproduced as arithmetic — and the
    /// proof that no pure function could have stopped it.**
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
