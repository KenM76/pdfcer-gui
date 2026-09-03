//! # `app::conditions::armed` — **which control renders pressed**
//!
//! One question, split out of [`super`] on 2026-08-31 under R2 when
//! `view.smart_select` (`OPERATOR_REQUESTS.md` O70) took that file to 1,513
//! lines.
//!
//! ## ★★ Why this is a seam and not an arbitrary cut
//!
//! Everything [`super`] publishes answers *"may this control be pressed?"* —
//! it reads the document, the selection, the undo log, the mode. Everything
//! here answers a different question: **"is this control ALREADY in the state
//! it names?"** The two differ in three ways at once, which is the test this
//! project uses for a seam:
//!
//! | | enable conditions | pressed conditions |
//! |---|---|---|
//! | source | `&self` — the app and its document | `egui::Context` — memory, and the viewport |
//! | scope | mostly inside `Status::Open` | **outside** it, deliberately: an armed tool survives closing a file |
//! | shape | `set.set("doc.pages")` | `set.set(selected_condition(id))` |
//!
//! ## ★★★ The recurring defect this file is the home of
//!
//! **Adding a tool is five changes, and the fifth has no unit test to remind
//! you.** Phase 7 shipped `CanvasTool::Measure`, `arm_measure`,
//! `measure_command` and a dispatch arm — every one with a passing test — and
//! no call here. So Measure ▸ Linear armed the tool, placed a dimension the
//! engine accepted, and **the button never lit up**. `ui-verify` found it in a
//! running window, because the missing link was a *call site*, and a call
//! site's effect is only observable in one.
//!
//! Collecting them in one file is the mitigation: the next person adding a tool
//! meets every existing one in a single screen rather than finding four of them
//! scattered through a thousand lines of enable logic.

use egui_shell::commands::ConditionSet;

impl crate::app::PdfcerApp {
    /// Publish the pressed state of every control that has one.
    ///
    /// Called from [`super::PdfcerApp::conditions`] with the set it is building,
    /// so there is one `ConditionSet` per frame and this adds to it rather than
    /// returning a second one to be merged — a merge being a place two answers
    /// about one control could both be present.
    pub(super) fn armed_conditions(&self, ctx: &egui::Context, set: &mut ConditionSet) {
        // ★ **The two toggles whose state lives in `egui::Memory`.**
        //
        // These were the last controls in the ribbon with no pressed state,
        // and the reason was structural rather than an oversight: this
        // function took `&self` and no `egui::Context`, so a toggle whose
        // state is in egui's own memory had no route here at all. Three
        // separate pieces of work recorded the gap and declined to invent a
        // second mechanism for it, which was right — the fix is to hand this
        // function the context, not to keep a shadow copy of the tool on
        // `PdfcerApp` that the canvas would then have to remember to update.
        //
        // A shadow copy is worth naming as the road not taken, because it is
        // the obvious one: it would put the truth about which tool is armed in
        // two places, and the failure mode is a ribbon that says Hand while
        // the canvas selects — a disagreement no test would catch, because
        // each half would be self-consistent.
        //
        // **Outside the `Status::Open` arm on purpose.** The armed tool and
        // the armed zoom survive closing a document, so a ribbon that forgot
        // which tool you were in the moment you closed a file would be
        // reporting something untrue about its own state. The commands
        // themselves are gated on `doc.pages`, so they still grey out with
        // nothing open — greyed and pressed is exactly right for "this is the
        // tool you are in, and there is nothing to use it on".
        if crate::canvas::tool::selected(ctx) == crate::canvas::tool::CanvasTool::Hand {
            set.set(egui_shell::ribbon::selected_condition("view.tool_hand"));
        }
        // ★ **The text tool's pressed state**, published exactly as the hand's
        // is, from the same `egui::Memory`-backed value and outside the
        // `Status::Open` arm for the same reason.
        //
        // # This is the step that was forgotten once, and the reason it has a
        // test of its own
        //
        // Phase 7 shipped `CanvasTool::Measure`, `arm_measure`, `measure_command`
        // and a dispatch arm using its inverse — every one with a passing unit
        // test — and did **not** publish the condition here, so Measure ▸ Linear
        // armed the tool, placed a dimension the engine accepted, and the button
        // never lit up. `ui-verify` found it in a running window, because the
        // missing link was a *call site*.
        //
        // The text tool is more exposed to that failure than either family
        // before it, and the reason is worth stating: arming it changes the
        // **cursor and nothing else**. A markup tool at least draws a band the
        // moment you use it; an armed text tool that did not light its control
        // would leave an operator with no on-screen evidence of the mode they are
        // in at all — and a captured window does not carry the pointer, so not
        // even a screenshot would show it.
        //
        // `selected` rather than `active`, matching the hand: a held space bar
        // borrows the hand for as long as it is down, and a control that
        // un-pressed itself under the operator's thumb every time they panned
        // would be reporting a tool they did not choose.
        if crate::canvas::tool::selected(ctx).is_text() {
            set.set(egui_shell::ribbon::selected_condition("view.tool_text"));
        }
        // ★ **The two View ▸ Window toggles' pressed state.**
        //
        // Outside the `Status::Open` arm, and more obviously so than the armed
        // tools above: these describe the **application's own shape**, which has
        // nothing to do with whether a document is loaded. A ribbon that
        // un-pressed Full screen because the operator closed a file would be
        // reporting something untrue about the window it is drawn in.
        //
        // The two read their state from different places and that asymmetry is
        // argued in `crate::app::window` §3, not here. In one sentence: read
        // mode is created by nothing but its own command, so `egui::Memory` is
        // its home and the same route the armed tool takes; full screen has an
        // owner **outside this program** — a window manager can grant or revoke
        // it unasked — so it is read back off the viewport rather than shadowed
        // on `PdfcerApp`, where the two would drift and the control would render
        // pressed over a windowed application.
        //
        // ★ Note what that costs, so it is not mistaken for a defect: the
        // full-screen control lights up on the frame **after** the press,
        // because a viewport command is answered by the backend. That is the
        // honest lag — the alternative is a control that reports a request as a
        // fact, which is wrong precisely when the backend refuses.
        if crate::app::window::read_mode(ctx) {
            set.set(egui_shell::ribbon::selected_condition("view.read_mode"));
        }
        if crate::app::window::fullscreen(ctx) {
            set.set(egui_shell::ribbon::selected_condition("view.fullscreen"));
        }
        if crate::canvas::zoom::region_zoom_armed(ctx) {
            set.set(egui_shell::ribbon::selected_condition("view.zoom_region"));
        }
        // ★★ **Smart select's pressed state** — `OPERATOR_REQUESTS.md` O70.
        //
        // Read from `egui::Memory` through `canvas::smart`, which is the same
        // value the click path resolves with — so the control and the canvas
        // cannot disagree about which way the switch is set. The persisted copy
        // on `Prefs` is deliberately NOT what is read here: it is written by the
        // same dispatch arm, but reading it would make the ribbon report what
        // will be true after the next restart rather than what is true now.
        //
        // Outside the `Status::Open` arm, like the armed tools above and for
        // their reason: it is a statement about how this operator works, not
        // about a file. The command is gated on `mode.edit_content`, so with
        // nothing open it renders greyed and pressed — which is exactly right
        // for *"this is how selection behaves, and there is nothing to select"*.
        if crate::canvas::smart::enabled(ctx) {
            set.set(egui_shell::ribbon::selected_condition("view.smart_select"));
        }
        // ★ The armed markup tool, published the same way and outside the
        // `Status::Open` arm for the same reason as the two above.
        //
        // **At most one**, because `CanvasTool::Markup` carries the kind
        // rather than there being one variant per shape — so the four
        // controls behave as a radio without anything having to enforce it.
        // That is the payoff of the enum shape the canvas chose: a tool that
        // could be two kinds at once is unrepresentable, so a ribbon showing
        // two pressed shape buttons is unrepresentable too.
        if let Some(kind) = crate::canvas::tool::selected(ctx).markup_kind() {
            set.set(egui_shell::ribbon::selected_condition(
                crate::shell::commands::markup_command(kind),
            ));
        }
        // ★ …and the armed **measure** tool, for the identical reason.
        //
        // # This arm was missing, and `ui-verify` is what found it
        //
        // Phase 7 shipped `CanvasTool::Measure(MeasureKind)`, `arm_measure`,
        // `measure_command` — the exact twin of `markup_command` — and a
        // dispatch arm that uses its inverse. Every one of those has a passing
        // unit test. What nothing tested is that *this function* hands the
        // second to the first, because that is a property of a **call site**,
        // and a call site's effect is observable only in a running window.
        //
        // So Measure ▸ Linear armed the tool, placed a dimension the engine
        // accepted, and **the button never lit up**. That is `HANDOFF.md`
        // defect 2's shape one layer up: the thing works, and the surface the
        // operator looks at does not say so. It was found by
        // `ui_verify::checks::measure_linear`, which compares the control's fill
        // against its sibling's *in one capture* — a differential nothing that
        // happens to both controls can satisfy.
        //
        // The lesson worth keeping: adding a tool is not four changes, it is
        // five, and the fifth is the one with no unit test to remind you.
        if let Some(kind) = crate::canvas::tool::selected(ctx).measure_kind() {
            set.set(egui_shell::ribbon::selected_condition(
                crate::shell::commands::measure_command(kind),
            ));
        }
    }
}
