//! # `app::toolstatus` — the one line that replaced the Tool panel
//!
//! `OPERATOR_REQUESTS.md` **O123**, verbatim:
//!
//! > *"I never understood why there is a tool dock when everything can be in
//! > object and properties. … The Tool panel becomes a one-line tool status
//! > (name, one sentence, 'Put this tool down'); its buttons duplicate the
//! > ribbon and go."*
//!
//! ## ★★★ What moved where, and why nothing was deleted
//!
//! `SHELL_LAYOUT_PROPOSAL.md` §3 ranked this proposal fourth of four and said
//! **do not build it**, on the ground that it deletes the armed block's live
//! controls and orphans a disclosure slot. That analysis was right about the
//! cost and wrong about the remedy, and the operator's first sentence is why:
//! the controls were never the tool panel's to hold. They are properties of
//! what is selected or about to be drawn.
//!
//! | what the Tool panel held | where it is now |
//! |---|---|
//! | the armed tool's **name** and **stage** | here, on one line |
//! | **Put this tool down** | here, at the end of that line |
//! | the pointer sentence (Block A) | here, as the sentence for the resting tool |
//! | every stage's **second** sentence | here, in the strip's hover — see [`sentence`] |
//! | the **tool list** (Block B) | **gone**, on the operator's instruction — every row was a route to a ribbon command |
//! | the text pen's **font, size and colour** | `crate::panels::properties::tool` |
//! | the circular measure's **pick list** | `crate::panels::properties::tool` |
//! | the three **scale switches** | `crate::panels::properties::tool` |
//! | the **disclosure block** (Block C) | `crate::panels::properties::disclose` |
//!
//! ★ The tool list is the only genuine subtraction, and it is the one he asked
//! for by name. It is worth writing down that it was the *answer to a
//! discoverability defect* — `panels::tool` existed because *"The feature
//! works. He could not find it."* — and that removing it is his call to make
//! and not this module's. What survives of that argument is the sentence on
//! this strip: it is permanent chrome, it names what is armed at frame one with
//! no clicks, and it cannot be closed, which is more than the panel could say.
//!
//! ## ★★ Why a dock banner and not a status-bar item
//!
//! The strip has to be **beside the document**, permanently, and it has to have
//! somewhere to put a button. The status bar is under R128 — its row must not
//! grow — and it already carries an *elided* copy of the same disclosure slot
//! this change re-homes ([`crate::app::status::disclosure`]). A second, wider
//! claimant on that row is the exact feedback loop R128 exists to forbid.
//! [`egui_shell::dock::banner`] gives the right dock a reserved strip whose
//! height is a constant the dock takes off the top before it resolves the
//! columns, so nothing here can drive a width or a height.
//!
//! ## The resting state draws no button, and that is R9 rather than an omission
//!
//! `CanvasTool::Select` **is** the resting state; putting a tool down returns
//! to it. A *Put this tool down* button beside `Select` would be a control
//! whose press changes nothing — `SHELL_LAYOUT_PROPOSAL.md` §3.3 caught the
//! mock shipping exactly that — and R9 forbids a dead control. So the button
//! appears when something is armed and is absent otherwise.
//!
//! ## ★★★ And the strip draws its sentence even when it cannot name the tool
//!
//! `OPERATOR_REQUESTS.md` **O66**. A `CanvasTool::Place` is armed from inside a
//! dialog that then hides itself, so there is no ribbon control to name and
//! [`command_for`] answers `None` — the old identity row was absent in exactly
//! that case. But the *stage* line still drew, and its own comment says why:
//! *"This one is the ONLY place the gesture and the way out are stated …
//! Deleting this line would strand an operator who has forgotten what they
//! armed."*
//!
//! ⇒ So a missing name suppresses the **name**, never the sentence. Written as
//! its own early-return rather than folded into the format string, because the
//! tempting shape — one `format!` with an empty name — silently ships a line
//! beginning with an em dash.

use egui::Ui;

use crate::app::state::OpenDoc;
use crate::canvas::measure::MeasureKind;
use crate::canvas::tool::CanvasTool;
use crate::shell::menus::MenuHost;
use crate::text::tool as t;
use crate::text::toolstatus as ts;

/// The height the right dock reserves for the strip, in points.
///
/// One row of text plus the padding a button needs around it.
/// [`egui_shell::dock::banner::resolve_height`] clamps this, so a window too
/// short to afford it gets no strip rather than a sliver — see that function's
/// ★ section.
pub const BANNER_HEIGHT_PTS: f32 = 26.0;

/// The region the strip publishes when it has drawn.
///
/// ★ Distinct from `egui-shell`'s own `dock.right.banner`, which reports the
/// **compartment**. This one reports the **content**, and the two answer
/// different questions: the dock's says *the strip is on screen*, this one says
/// *the application put something in it*. A check that asserted only the former
/// would pass against a build whose handler drew nothing at all.
pub const REGION: &str = "toolstatus"; // ui-text-exempt: trace region name, never displayed
/// The region the *Put this tool down* button publishes.
pub const REGION_PUT_DOWN: &str = "toolstatus.put_down"; // ui-text-exempt: trace region name, never displayed

/// Draw the strip into the banner `Ui` the dock reserved.
///
/// Draws **nothing at all** with no document open: the whole line is about a
/// gesture on a page, and a strip that named a tool with nothing to use it on
/// would be the placeholder R9 forbids, sitting in permanent chrome where it
/// could never be dismissed.
pub fn banner(ui: &mut Ui, doc: Option<&OpenDoc>, host: Option<&MenuHost<'_>>) {
    let Some(doc) = doc else {
        return;
    };
    let ctx = ui.ctx().clone();
    let armed = crate::canvas::tool::selected(&ctx);
    let (primary, secondary) = sentence(&ctx, doc, armed);
    let line = match name_of(armed, host) {
        Some(name) => ts::status_line(name, &primary),
        None => primary.clone(),
    };
    // The hover carries what the line could not: the second sentence of the
    // stages that have one, then the note saying where the controls went.
    let hover = match &secondary {
        Some(extra) => format!("{line}\n\n{extra}"),
        None => line.clone(),
    };

    ui.horizontal_centered(|ui| {
        // ★★ `truncate`, never `wrap`. The strip's height is a constant the
        // dock has already taken off the side; a second line would be drawn
        // over the first stack's tab bar and clipped away, which reads as a
        // rendering fault. The full text is on hover, which is the same
        // elide-and-defer discipline [`crate::app::status::disclosure`] applies
        // to the bar's single row and for the same reason.
        let response = ui
            .add(egui::Label::new(egui::RichText::new(&line).small()).truncate())
            .on_hover_text(&hover)
            .on_hover_text(ts::status_tooltip());
        crate::diag::ui_rect_visible(REGION, response.rect, ui.clip_rect());

        if armed != CanvasTool::Select {
            // Right-aligned, so the button does not move when the sentence
            // changes length. A control that moves is a control you cannot aim
            // at — the armed block's own layout rule, applied to a row instead
            // of a column.
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                put_down(ui, &ctx);
            });
        }
    });
}

/// The *Put this tool down* button, moved verbatim from the armed block.
///
/// It still writes `canvas::tool::select` directly rather than raising an
/// `Action`, and the argument is unchanged and worth repeating because a move
/// is exactly when somebody would "fix" it: **the armed tool is not document
/// state.** It contributes nothing to the undo log and has nothing to order
/// against, so routing it through the action funnel would add a variant `apply`
/// could only answer by writing the same memory slot.
fn put_down(ui: &mut Ui, ctx: &egui::Context) {
    let response = ui
        .button(t::put_down_button())
        .on_hover_text(t::put_down_hint());
    crate::diag::ui_rect_visible(REGION_PUT_DOWN, response.rect, ui.clip_rect());
    if response.clicked() {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            "tool-panel-put-down".to_owned()
        });
        crate::canvas::tool::select(ctx, CanvasTool::Select);
    }
}

/// The armed tool's name, read from the command registry.
///
/// **Never a string of this module's own.** A second copy of a label compiles,
/// reads identically the day it is written, and drifts the first time either is
/// reworded — invisibly, because nothing renders both at once. The rule is
/// inherited from the armed identity row this replaces.
fn name_of<'a>(tool: CanvasTool, host: Option<&'a MenuHost<'_>>) -> Option<&'a str> {
    host?.label(command_for(tool)?)
}

/// The one sentence, plus whatever the old stage said on a **second** line.
///
/// # ★★★ Why this returns a pair instead of one string
///
/// Six of the armed stages drew two labels, and the second was never
/// decoration. `t::text_annot_release` is described in its own module as *"The
/// sentence that stops a working tool reading as broken"*; `t::hand_borrow`
/// says how the borrowed hand is given back; `t::node_shift` names the modifier
/// that makes the node tool usable. A one-row strip has nowhere to put them,
/// and **dropping them would be the content regression this whole change is
/// under instruction not to commit.**
///
/// So they go to the hover, which is where this project sends a sentence that
/// has been elided rather than shortened — the discipline
/// [`crate::app::status::disclosure`] states as *"eliding defers rather than
/// loses"*.
///
/// # ★ The primary is the LIVE stage where there is one
///
/// One slot, two contents — the armed block's rule, and it applies here with
/// more force rather than less. *"3 vertices placed"* is worth more than
/// *"click each corner"* the moment the operator has clicked one, and it
/// collapses back to the instruction when the run ends.
fn sentence(ctx: &egui::Context, doc: &OpenDoc, tool: CanvasTool) -> (String, Option<String>) {
    match tool {
        // The resting state, and the one sentence in the application that says
        // what a plain drag MEANS in this mode — it marquees objects in Edit
        // and sweeps text in Read, decided by `canvas::textsel::takes_the_press`
        // reading the mode. Block A of the old panel carried it; nothing else
        // ever has.
        CanvasTool::Select => {
            let caps = crate::canvas::tool::capabilities(ctx);
            let line = if caps.edit_content {
                t::pointer_edit()
            } else {
                t::pointer_reading()
            };
            (line.to_owned(), None)
        }
        CanvasTool::Node => (
            t::node_instruction().to_owned(),
            Some(t::node_shift().to_owned()),
        ),
        CanvasTool::Hand => (
            t::hand_instruction().to_owned(),
            Some(t::hand_borrow().to_owned()),
        ),
        CanvasTool::Text => {
            // ★ The second sentence is rendered only where it is TRUE. In Read
            // and Review the select tool already swept text, so arming this
            // takes nothing away and the sentence would be describing a change
            // that did not happen. Absent rather than reworded — R9 applied to
            // a sentence, and carried across unchanged from the armed block.
            let extra = crate::canvas::tool::capabilities(ctx)
                .edit_content
                .then(|| t::text_select_takes_the_press().to_owned());
            (t::text_select_instruction().to_owned(), extra)
        }
        CanvasTool::Form(kind) => (
            t::form_instruction().to_owned(),
            Some(t::form_kind_hint(kind).to_owned()),
        ),
        // ★★★ O66 — the ONLY surface that states this gesture and its way out,
        // because the window that asked for the placement has hidden itself.
        CanvasTool::Place(_) => (crate::text::placing::armed_instruction().to_owned(), None),
        CanvasTool::TextAnnot(kind) => (
            t::text_annot_instruction(kind).to_owned(),
            Some(t::text_annot_release().to_owned()),
        ),
        CanvasTool::TextEdit(kind) => {
            // Live when there is a caret, the instruction before there is one.
            let live = matches!(crate::canvas::textedit::read(ctx), Some(d) if d.kind == kind);
            let line = if live {
                t::text_edit_live().to_owned()
            } else {
                t::text_edit_instruction(kind).to_owned()
            };
            (line, None)
        }
        CanvasTool::Markup(kind) => {
            let line = match crate::canvas::markup::vertex::read(ctx) {
                Some(run) if run.kind == kind && run.in_progress() => {
                    t::vertices_placed(run.vertices.len())
                }
                _ => t::markup_instruction(kind).to_owned(),
            };
            (line, None)
        }
        CanvasTool::Measure(MeasureKind::Perimeter) => (perimeter_stage(ctx, doc), None),
        CanvasTool::Measure(MeasureKind::Circular) => (circular_stage(ctx, doc), None),
        CanvasTool::Measure(kind) => (t::measure_instruction(kind).to_owned(), None),
    }
}

/// The perimeter tool's live sentence: the instruction before the first click,
/// the running total after it.
///
/// Moved unchanged from the armed block, and the reasoning moves with it,
/// because it is the reason this function is not two lines of arithmetic:
/// [`pdfcer_core::dimension::format_measurement`] is the ENGINE's own
/// formatter — the same one the committed label goes through — so the running
/// total and the final label cannot disagree about scale, unit, precision,
/// fraction style or decimal marker. The operator's ask was that the tool
/// behave *"the same as the other dimensioning tools"*, and a live readout in
/// points beside a committed dimension in metres would be two numbers for one
/// measurement.
///
/// Falls back to the instruction when the group cannot be read: a total whose
/// scale is unknown is not a total.
fn perimeter_stage(ctx: &egui::Context, doc: &OpenDoc) -> String {
    let instruction = || t::measure_instruction(MeasureKind::Perimeter).to_owned();
    let Some(st) = crate::canvas::measure::read(ctx) else {
        return instruction();
    };
    let picked = st.perimeter.points().len();
    if picked == 0 {
        return instruction();
    }
    let model = doc.session.dimension_model();
    let Some(group) = model.group(st.group) else {
        return instruction();
    };
    let shown = pdfcer_core::dimension::format_measurement(
        st.perimeter.length_points(),
        group.scale,
        group.format,
    );
    t::measure_perimeter_live(picked, &shown.text)
}

/// The radius/diameter tool's live sentence: the instruction before the first
/// click, the count and the current fit after it.
///
/// `OPERATOR_REQUESTS.md` O105 — *"selecting more points around a hole doesn't
/// always get it to narrow down to the size of the hole."* An operator adding
/// points to a fit is watching a number converge, and before this existed there
/// was no number to watch: every correction was a commit and an undo.
///
/// ★ **Radius or diameter follows the pick set's own display toggle**, so the
/// number the strip shows is the number the placed dimension will show. A
/// readout that always reported the radius would disagree with a committed
/// diameter label by a factor of two, silently.
fn circular_stage(ctx: &egui::Context, doc: &OpenDoc) -> String {
    let Some(st) = crate::canvas::measure::read(ctx) else {
        return t::measure_instruction(MeasureKind::Circular).to_owned();
    };
    let picked = st.circular.point_count();
    if picked == 0 {
        return t::measure_instruction(MeasureKind::Circular).to_owned();
    }
    let Some(fit) = st.circular.fit() else {
        return t::measure_circular_needs_more(picked);
    };
    let model = doc.session.dimension_model();
    let Some(group) = model.group(st.group) else {
        return t::measure_circular_needs_more(picked);
    };
    let value = if st.circular.show_diameter {
        fit.radius * 2.0
    } else {
        fit.radius
    };
    let shown = pdfcer_core::dimension::format_measurement(value, group.scale, group.format);
    t::measure_circular_live(picked, &shown.text)
}

/// The command that arms `tool`, if one does.
///
/// # ★ Derived from the existing id maps, never written a second time
///
/// `shell::commands::markup_command` and `measure_for_command`'s inverse are
/// the single binding between an id and a kind, exactly as
/// `Panel::from_command_id` is for panels. Re-listing them here would be a
/// second table to keep in step, and the failure when it drifted would be a
/// strip naming the wrong tool — which is the one thing it exists to get right.
///
/// Moved verbatim from the armed block; the `None` arms are the interesting
/// ones and each keeps its reason.
fn command_for(tool: CanvasTool) -> Option<&'static str> {
    match tool {
        // ui-text-exempt: command ids, never displayed
        CanvasTool::Select => Some("view.tool_select"),
        CanvasTool::Node => Some("view.tool_node"),
        CanvasTool::Hand => Some("view.tool_hand"),
        CanvasTool::Text => Some("view.tool_text"),
        CanvasTool::Markup(kind) => Some(crate::shell::commands::markup_command(kind)),
        // ★ Each kind names its own command, which is what lets the strip show
        // the armed field type. The mapping lives on the kind rather than here
        // so the two cannot drift.
        CanvasTool::Form(kind) => Some(kind.command_id()),
        // ★ **None** — a placement is armed from inside a dialog and has no
        // ribbon control to name, so the NAME is absent. The sentence is not;
        // see this module's O66 section. Written as its own arm rather than
        // folded into a `_` so that a second `PlaceKind` has to be ruled on
        // rather than inheriting this silently.
        CanvasTool::Place(_) => None,
        // ★ The empty string is `MeasureKind::Scale`'s id, and it is not a
        // command — that kind is armed from inside the Set-scale window and
        // deliberately maps to nothing.
        CanvasTool::Measure(kind) => {
            let id = crate::shell::commands::measure_command(kind);
            (!id.is_empty()).then_some(id)
        }
        CanvasTool::TextAnnot(kind) => Some(kind.command()),
        CanvasTool::TextEdit(crate::canvas::textedit::TextEditKind::Edit) => Some("edit.text"),
        CanvasTool::TextEdit(crate::canvas::textedit::TextEditKind::Add) => Some("edit.add_text"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The two regions are distinct and neither collides with a panel's.
    #[test]
    fn the_regions_are_their_own_names() {
        assert_ne!(REGION, REGION_PUT_DOWN);
        assert!(REGION_PUT_DOWN.starts_with(REGION));
        assert!(!REGION.starts_with("panel:"));
    }

    /// ★ The reserved height is inside the band `egui-shell` will honour at a
    /// realistic window, so the strip cannot silently resolve to nothing.
    ///
    /// Falsifiable in one edit: drop `BANNER_HEIGHT_PTS` below the shell's
    /// floor and this goes red rather than the strip quietly disappearing at
    /// run time.
    #[test]
    fn the_reserved_height_survives_the_shells_clamp() {
        let resolved = egui_shell::dock::banner::resolve_height(BANNER_HEIGHT_PTS, 800.0);
        assert!(
            (resolved - BANNER_HEIGHT_PTS).abs() < f32::EPSILON,
            "the dock would clamp the strip to {resolved} pt"
        );
    }

    /// ★★★ **Every tool that had a name has one still, and the two that never
    /// did still do not.**
    ///
    /// The table moved modules, and a move is where an arm gets dropped. This
    /// asserts the `None` arms are exactly the two documented ones — a
    /// placement, and the scale kind that is armed from inside a window — so a
    /// tool silently losing its name shows up as a red test rather than as a
    /// strip that renders a sentence with no subject.
    #[test]
    fn only_the_two_dialog_armed_tools_have_no_command() {
        use crate::canvas::textedit::TextEditKind;
        let named = [
            CanvasTool::Select,
            CanvasTool::Node,
            CanvasTool::Hand,
            CanvasTool::Text,
            CanvasTool::TextEdit(TextEditKind::Add),
            CanvasTool::TextEdit(TextEditKind::Edit),
            CanvasTool::Measure(MeasureKind::Linear),
            CanvasTool::Measure(MeasureKind::Perimeter),
            CanvasTool::Measure(MeasureKind::Circular),
        ];
        for tool in named {
            assert!(
                command_for(tool).is_some_and(|id| !id.is_empty()),
                "{tool:?} lost the command that names it"
            );
        }
        assert_eq!(
            command_for(CanvasTool::Measure(MeasureKind::Scale)),
            None,
            "the scale kind is armed from inside a window and names no command"
        );
    }
}
