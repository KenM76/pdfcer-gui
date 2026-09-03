//! # `panels::tool` — what the pointer does, what is armed, and where the
//! tools live
//!
//! ## The two complaints this closes, and why they are one complaint
//!
//! The operator, 2026-08-19, with *"I bring them up over and over again and
//! they are still not dealt with"*:
//!
//! > 4. *"no side bar area showing what tool is active and its options"*
//! > 5. *"no text editing or adding text on the canvas"*
//!
//! **The second is false in the literal sense and that is the interesting
//! part.** `edit.text` and `edit.add_text` are registered, drawn on Edit ▸
//! Content, bound to `Ctrl+E` and `Ctrl+Shift+E`, and two driven checks pass on
//! them. The feature works. He could not find it.
//!
//! So #5 is a **discoverability defect**, which is this project's founding
//! failure wearing different clothes, and it was marked green for three weeks
//! because *"the tests pass"* is not a report of working software. #4 is the
//! same gap named directly: eleven panels can be mounted in Edit and **not one
//! of them is about what an operator can do.**
//!
//! ## ★ This is NOT the "Tool options" pane, and the two text tools are the
//! proof
//!
//! `RIBBON_IA.md` §6 commissions this surface and calls it the *Tool Options
//! pane*, per P2: *"The ribbon picks the activity; the sidebar holds that
//! activity's controls."* The tab caption is nevertheless **Tool**, not *Tool
//! options*, and the difference is not cosmetic.
//!
//! `CanvasTool::TextEdit`'s entire state is a `Draft { page, kind, anchor,
//! text, seeded }` whose anchor is **derived from the click**. There is no
//! font, no size, no alignment — `RIBBON_IA.md` §5.8 gives all of those to the
//! Format tab and the Properties panel. **The two tools whose absence started
//! this have exactly zero options.** A panel shaped around options would render
//! nothing for them and complaint #5 would survive its own fix.
//!
//! A caption is a promise. *Options* is a promise this panel cannot keep for
//! the hand tool, the text sweep, or either text tool. *Tool* promises to name
//! what is armed, and that question always has an answer.
//!
//! ## ★★ What it must never become: a second ribbon
//!
//! The pull is real and has a specific shape — somebody will put all eight
//! `MarkupKind`s and all three `MeasureKind`s here as a palette, because that
//! makes the panel look full. Three rules, and they are load-bearing:
//!
//! 1. **It never lists a tool the ribbon has no control for.** R8: registering
//!    a command is the only way this GUI learns a capability exists.
//! 2. **When a tool is armed it shows only that tool.** No sibling kinds, no
//!    *"switch to Ellipse"*.
//! 3. **When nothing is armed the list is one row per tool FAMILY**, capped at
//!    seven rows. If it ever exceeds seven, the design has failed and somebody
//!    has rebuilt the ribbon in a 320 pt column.
//!
//! The test that matters: **a good Tool panel makes the operator need it less
//! over time**, because every row names the ribbon tab the command lives on. A
//! second ribbon makes them need the *ribbon* less, which is the opposite, and
//! is how a project ends up maintaining two information architectures.
//!
//! ## The three blocks, and why the unarmed state is the hard one
//!
//! `CanvasTool::Select` is `Default` and the application opens in it. That is
//! the state the operator is in most of the time, and it is where three obvious
//! answers do real damage:
//!
//! | answer | what it costs |
//! |---|---|
//! | an **empty panel** | it gets closed on day two, and the dock **persists** that — a panel that earns being closed is closed for ever |
//! | *"No tool selected."* | R9 violation, and it is the operator's own complaint about the shell this replaces: *"the nagging and red flagging … made for a lot of extra bugs in the visibility when editing"* |
//! | a full **tool palette** | a second ribbon, and worse than the defect it replaces: now there are two places to look that disagree at the edges |
//!
//! So: three blocks, every line live, nothing dead.
//!
//! - **A — the pointer.** What a press means *in this mode*, right now. Not a
//!   placeholder: the identical drag marquees objects in Edit and sweeps text
//!   in Read, decided by `canvas::textsel::takes_the_press` reading the mode,
//!   and **no surface in this application has ever said so.**
//! - **B — the tools this mode has.** One row per family, each naming its
//!   ribbon tab and its chord, each arming through `Action::Command(id)` so
//!   there is no second dispatch path and every guard the ribbon control has
//!   applies unchanged.
//! - **C — what pdfcer last worked out.** The disclosure block, rendered only
//!   when there is something in it.
//!
//! ## ★★ Block C is where the text-edit refusal finally has somewhere to go
//!
//! `crate::text::textedit::refusal` writes three good sentences, is tested by
//! `every_refusal_says_something`, and its own module records the trap:
//! they were aimed at the **status bar**, and *"it shares the status row with
//! everything else and R128 forbids that row growing."* `Refusal::SpansRuns` is
//! 47 words. It has never been readable.
//!
//! **This is very likely the actual cause of complaint #5.** On a dense CAD
//! sheet the first click of `edit.text` lands where the operator *wants* text
//! rather than where text *is*, so `Refusal::NoRun` is the likely first
//! outcome — and a decline nobody can read teaches an operator that the feature
//! does not exist. A dock panel's width is the dock's, decided before the body
//! draws, so R128 does not apply here at all. This is the same property
//! [`super::dimension_groups`] used to retire the Manage-groups window's growth
//! loop.
//!
//! ## Layout, inherited whole from `panels::dimension_groups`
//!
//! One `ScrollArea`, **nothing after it**, no footer, no reserve constant.
//! `CONTINUE.md` §7's pattern recurred four times in one day and every instance
//! was a control placed after an unbounded scroll region. The panel also never
//! influences its own width — `super::content_width` — because a content-driven
//! width beside a fit-to-viewport zoom is R128 from the other direction.
//!
//! ## What is deliberately NOT here yet, and why saying so matters
//!
//! **The markup pen's colour and width.** They belong here and are not built,
//! for a reason that is a design decision rather than a shortfall:
//! `Panel::show` is handed `(ui, Option<&OpenDoc>, &mut PanelsState,
//! Option<&MenuHost>, &mut Vec<Action>)` and **the pen is on `PdfcerApp`**. A
//! naive build draws a colour swatch that accepts a click and discards it —
//! precisely the control `panels::properties` refused to ship, in its own
//! words *"not a harmless placeholder but a control that silently loses an
//! operator's work."*
//!
//! The route is `Action::SetPen`, raised like every other panel's intent, which
//! keeps the funnel's fourth property intact: *"what can change what is drawn?"*
//! stays greppable. Until that lands the panel shows the pen **not at all**,
//! rather than showing it read-only — a swatch that looks like a control and is
//! not is worse than no swatch.
//!
//! Also absent, each for a recorded reason: any property of a **placed**
//! annotation (that is the Format tab and `panels::properties` — this panel is
//! about the *next* gesture), a second *Draw into* group picker (one copy, in
//! the panel that owns it), arrowhead length and angle (`NO_SURFACE.md` §1's
//! **MIS-FILED** verdict — they are the cursor, and a control whose readback is
//! real and whose effect is imaginary is worse than none), and the ink
//! simplification tolerance (it must follow the pen; a field re-opens the
//! 2026-08-17 defect where a 0.25 pt pen got four times its half-width).

mod armed;
mod idle;

use egui::Ui;

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::shell::menus::MenuHost;

/// The region the panel body publishes.
pub const REGION_BODY: &str = "panel:tool"; // ui-text-exempt: trace region name, never displayed
/// The region Block A publishes.
pub const REGION_POINTER: &str = "tool.pointer"; // ui-text-exempt: trace region name, never displayed
/// The region Block B publishes.
pub const REGION_TOOLS: &str = "tool.tools"; // ui-text-exempt: trace region name, never displayed
/// The region the armed block publishes.
pub const REGION_ARMED: &str = "tool.armed";
/// The region the text pen's controls publish.
pub const REGION_TEXT_PEN: &str = "tool.text_pen"; // ui-text-exempt: trace region name, never displayed
/// The region the Select tool's three scale switches publish.
///
/// ★ Its own name rather than sharing `REGION_TEXT_PEN`: a driven check aiming
/// at "the options row" would find whichever tool happened to be armed, and the
/// two option sets belong to different tools.
pub const REGION_SCALE_SWITCHES: &str = "tool.scale_switches"; // ui-text-exempt: trace region name, never displayed
/// The *Scale line weight* switch's own rect.
pub const REGION_SCALE_STROKE: &str = "tool.scale.stroke"; // ui-text-exempt: trace region name, never displayed
/// The *Keep the inner margins* switch's own rect.
pub const REGION_SCALE_INSETS: &str = "tool.scale.insets"; // ui-text-exempt: trace region name, never displayed
/// The *Allow the artwork to distort* switch's own rect.
pub const REGION_SCALE_DISTORT: &str = "tool.scale.distort"; // ui-text-exempt: trace region name, never displayed
/// The region the radius/diameter tool's picked-point list publishes.
pub const REGION_MEASURE_POINTS: &str = "tool.measure_points"; // ui-text-exempt: trace region name, never displayed
/// The prefix of one picked point's row; its index in the set is appended.
///
/// ★ Per ROW rather than one rect for the list, because the whole capability
/// `OPERATOR_REQUESTS.md` O107 asks for is *removing a particular point*, and a
/// check that could only find "the list" could not press one. The index is the
/// same number the row shows the operator, minus the one-based display offset.
pub const REGION_MEASURE_POINT_PREFIX: &str = "tool.measure_point."; // ui-text-exempt: trace region name, never displayed
/// The region Block C publishes.
pub const REGION_DISCLOSURES: &str = "tool.disclosures"; // ui-text-exempt: trace region name, never displayed
/// The prefix of one arming row's region; the command id is appended.
pub const REGION_ROW_PREFIX: &str = "tool.row."; // ui-text-exempt: trace region name, never displayed

/// Draw the panel.
///
/// # ★ Why a region is published per BRANCH rather than one at the root
///
/// `CONTINUE.md` §7's second rule: *"an instrument that can only return one
/// answer cannot detect the thing it was added to detect. Put diagnostics at
/// the entry of a function with early returns, naming each gate."*
///
/// This body has two top-level states and four blocks. A single `panel:tool`
/// region would tell a driven check that the panel drew and **nothing** about
/// which of them it drew — so an armed panel with an empty armed block and an
/// unarmed panel with a full tool list would be indistinguishable from outside.
/// Each block publishes its own name, and a check asserts on the one it is
/// about.
pub fn body(ui: &mut Ui, doc: &OpenDoc, host: Option<&MenuHost<'_>>, actions: &mut Vec<Action>) {
    let ctx = ui.ctx().clone();
    let armed = crate::canvas::tool::selected(&ctx);

    egui::ScrollArea::vertical()
        .id_salt("tool-panel-scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());

            // ★ Block A FIRST and unconditionally, armed or not.
            //
            // It is the only block that is true in every state, and it is the
            // literal answer to *"no side bar area showing what tool is
            // active"*. Putting it under the armed block would make the panel's
            // first line vary with the tool, which is the volatility rule this
            // panel applies to its own layout: the row an operator's eye lands
            // on must not move.
            idle::pointer(ui, &ctx);

            if armed == crate::canvas::tool::CanvasTool::Select {
                // Nothing armed. The tool list is the body, and it is what
                // makes complaint #5 findable at frame one with no clicks.
                ui.separator();
                idle::tools(ui, &ctx, host, actions);
                // ★★★ **The Select tool's own options, and they live HERE
                // rather than in `armed::options` — which is where they were
                // first written, and where they were unreachable.**
                //
                // This panel's model is *armed versus idle*, and **Select is
                // the idle state**: the branch above is entered precisely when
                // nothing is armed. So `armed::block` — and the `options` row
                // inside it — is never called for Select, and an option row
                // added there for Select is dead code that compiles, reads
                // correctly, and draws nothing.
                //
                // ⇒ Caught by `the_line_weight_switch_reaches_the_resize` on
                // its first driven run, reporting *"THE SWITCH IS NOT DRAWN"*
                // and listing the nine `tool.*` regions that did appear. Every
                // unit test in the chain passed: the store round-trips, the
                // mapping to `ResizeOptions` is exhaustive, the defaults are
                // asserted. **Nothing tested that the control is on screen**,
                // which is R1's whole argument in one afternoon.
                //
                // ★ It sits after the tool list rather than before it, because
                // the list is the answer to *"what can I do"* and these modify
                // a gesture the operator has not made yet. Same ordering rule
                // the armed block uses: identity, then stage, then options.
                ui.separator();
                armed::scale_switches(ui, &ctx);
            } else {
                ui.separator();
                armed::block(ui, &ctx, doc, armed, host);
            }

            // ★ Block C LAST, and it is the one block that may render nothing.
            //
            // R9: an unavailable capability renders nothing, not a stub. A
            // heading over an empty region is exactly the placeholder that rule
            // forbids, so the heading is inside the `if` rather than above it.
            disclosures(ui, doc);
        });
}

/// **Block C — what pdfcer last worked out, in full, at panel width.**
///
/// # Why this is the highest-value block in the panel
///
/// It renders `crate::app::actions::disclosure::last_edit_disclosure`, which is
/// the same slot the status bar's one elided line reads — and which carries,
/// among other things, **the text tools' refusal sentences**. Those were
/// written well, are tested, and have never been readable: 47 words in a row
/// R128 forbids growing.
///
/// A dock panel's width is the dock's, decided before the body draws. So the
/// sentence that has been telling operators *why their click was declined*
/// finally has somewhere it fits, and the most likely cause of *"no text
/// editing on the canvas"* stops being invisible.
///
/// # It renders NOTHING when there is nothing, heading included
///
/// Not *"No notes."*, not an empty heading. R9, and also honesty: a heading
/// that is present on every frame trains an operator to stop reading the
/// region under it, which would waste the one surface a disclosure has.
fn disclosures(ui: &mut Ui, doc: &OpenDoc) {
    let Some(disclosure) = crate::app::actions::disclosure::last_edit_disclosure(doc.edit_epoch)
    else {
        return;
    };
    if disclosure.notes.is_empty() {
        return;
    }
    ui.separator();
    ui.label(crate::text::tool::disclosures_heading());
    crate::diag::ui_rect(REGION_DISCLOSURES, ui.min_rect());
    for note in &disclosure.notes {
        // ★ VERBATIM, and wrapped rather than elided. `ui-spec` §6's standing
        // rule and the whole reason this block is in a panel rather than in the
        // status row: a disclosure that has been shortened to fit is a
        // disclosure that has been edited by the program disclosing it.
        ui.label(egui::RichText::new(note).small());
    }
}

#[cfg(test)]
mod tests {
    /// ★ The panel's regions are all distinct, and every one is a `tool.`
    /// name or the body.
    ///
    /// Two regions sharing a name would make a driven check aim at whichever
    /// drew last, silently — and the two states this panel has draw different
    /// blocks, so the check would be measuring the state it was not testing.
    #[test]
    fn every_region_is_its_own_name() {
        let names = [
            super::REGION_BODY,
            super::REGION_POINTER,
            super::REGION_TOOLS,
            super::REGION_ARMED,
            super::REGION_TEXT_PEN,
            super::REGION_SCALE_SWITCHES,
            super::REGION_SCALE_STROKE,
            super::REGION_SCALE_INSETS,
            super::REGION_SCALE_DISTORT,
            super::REGION_MEASURE_POINTS,
            super::REGION_MEASURE_POINT_PREFIX,
            super::REGION_DISCLOSURES,
            super::REGION_ROW_PREFIX,
        ];
        let mut seen = std::collections::BTreeSet::new();
        for name in names {
            assert!(seen.insert(name), "{name} is declared twice");
        }
        assert_eq!(super::REGION_BODY, "panel:tool");
        for name in &names[1..] {
            assert!(
                name.starts_with("tool."),
                "{name} does not share the panel's region prefix, so a check sweeping \
                 `tool.` will not find it"
            );
        }
    }
}
