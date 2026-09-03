//! # `app::dispatch::navigate` — the five controls in View ▸ Navigate
//!
//! The arrow, the white arrow, the type tool, the hand, and — since
//! 2026-08-31 — the Smart-select switch beside them.
//!
//! ## Why they are one module
//!
//! Because they are one **row**, and the row is a claim: *these are the things
//! a press on the page can mean*. Four of them arm a tool and the fifth changes
//! what the first one selects, which makes it the odd one out in mechanism and
//! the obvious one out in placement — the operator asked for it *"in
//! navigate"* by name.
//!
//! They were arms in `app::dispatch` until R2's 1,500-line gate refused the
//! file on the day the fifth arrived. That is the gate doing its job: the seam
//! was already there and nobody had needed to find it.
//!
//! ## ★★ The two shapes in here, and why the difference is not an
//! ## inconsistency
//!
//! | control | capability check | why |
//! |---|---|---|
//! | Select, Hand | none | they author nothing, and `tool::retire_forbidden` permits them in every mode |
//! | Text | none | *"copying is not authoring"* — the operator's own ruling, 2026-08-14 |
//! | **Points** | `edit_content`, **and it says so** | an anchor is selected in order to be DRAGGED |
//! | **Smart select** | `edit_content`, and it says so | it governs a substitution that only happens where content is selectable |
//!
//! ★ Both declining arms exist because these controls have a **second door**.
//! The ribbon item is withheld outside Edit, so the route that still reaches
//! them there is a bare chord — and a chord that silently does nothing offers
//! no control to hover and no explanation anywhere. That is the P3 decline this
//! project keeps removing, which is why each decline both traces (for a reader
//! of a machine they cannot see) and records a sentence (for the operator in
//! front of one).

/// The ids this module answers for.
///
/// ★ A `matches!` over literals rather than a prefix test, for the reason every
/// `handles` in this directory gives: a prefix would silently claim the next
/// `view.*` command somebody adds, and the failure would be a command that
/// reaches this file's `match` and falls out of it doing nothing.
pub(crate) fn handles(id: &str) -> bool {
    // ui-text-exempt: registered command ids, never displayed.
    matches!(
        id,
        "view.tool_select"
            | "view.tool_hand"
            | "view.tool_text"
            | "view.tool_node"
            | "view.smart_select"
    )
}

/// Arm a tool, or flip the switch.
pub(crate) fn dispatch(app: &mut crate::app::PdfcerApp, ctx: &egui::Context, id: &str) {
    match id {
        "view.tool_select" => {
            crate::canvas::tool::select(ctx, crate::canvas::tool::CanvasTool::Select);
        }
        // Toggles, and returns the tool now chosen — which is discarded here
        // because the pressed state is published from `conditions` by asking
        // `tool::selected`, not from a copy kept in the app. A shadow copy is
        // how a ribbon comes to say Hand while the canvas marquees.
        "view.tool_hand" => {
            let _ = crate::canvas::tool::toggle_hand(ctx);
        }
        // ★ **The text tool**, the hand's twin down to the discarded return.
        //
        // ★★ **No capability check, and the absence is the decision** rather
        // than an oversight — worth saying because both arms below have one.
        // `markup_for_command` declines on `author_markup`, `measure_for_command`
        // on `author_measure`, and the obvious symmetry would be a third. There
        // is nothing to put there: selecting text authors nothing, so
        // `canvas::tool::retire_forbidden` permits this tool in every mode and
        // a decline here would contradict it. In one line, it is the operator's
        // own *copying is not authoring* ruling of 2026-08-14, which already
        // moved both text-copy verbs off the authoring tab.
        //
        // It therefore has no decline trace either, and that is consistent
        // rather than lax: a trace line exists to say *which* nothing happened,
        // and there is no state in which pressing this does nothing.
        "view.tool_text" => {
            let _ = crate::canvas::tool::toggle_text(ctx);
        }
        "view.tool_node" => {
            // Declines by name in a mode that cannot author, exactly as
            // `edit.text` does and for the identical reason: an anchor is
            // selected in order to be dragged, and a mode that refuses the drag
            // must refuse the tool rather than arm it and then say no to every
            // gesture. `tool::retire_forbidden` closes the same gap from the
            // other end when the mode changes underneath.
            if app.capabilities().edit_content {
                crate::canvas::tool::select(ctx, crate::canvas::tool::CanvasTool::Node);
            } else {
                decline(id);
            }
        }
        // ★★★ **Smart select** — `OPERATOR_REQUESTS.md` O70.
        //
        // A toggle that reads its own state rather than being handed one: the
        // ribbon's pressed look comes from a condition published out of
        // `app::conditions`, which reads the same memory value this arm writes.
        // One truth, so the control cannot disagree with the canvas about which
        // way the switch is set.
        //
        // ★ It writes BOTH homes and that is the whole of the persistence
        // design: `egui::Memory` is where the click path can read it, `Prefs` is
        // where it survives a restart, and `app::frame` mirrors the second into
        // the first every frame. This is the only writer of the persisted
        // answer.
        "view.smart_select" => {
            if app.capabilities().edit_content {
                let on = !crate::canvas::smart::enabled(ctx);
                crate::canvas::smart::set_enabled(ctx, on);
                app.prefs.smart_select = on;
                // Persisted immediately, like the pick filter and for its
                // reason: this is one discrete operator decision, so the correct
                // number of writes is one, now. A failure is not worth a modal —
                // losing a preference across a restart is an inconvenience, and
                // `app::pickstore`'s header argues it at length.
                let _ = app.prefs.save();
            } else {
                decline(id);
            }
        }
        // Unreachable while `handles` and this `match` agree; a bare fall-out
        // would be a command that reached its own module and did nothing, which
        // is the failure `handles`' doc comment is about.
        other => crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI.
            format!("command-unimplemented id={other} in=navigate")
        }),
    }
}

/// Say no, twice: once to the trace and once to the operator.
///
/// ★ Both, always, because they answer different people. The trace is read by
/// somebody debugging a machine they are not sitting at; the status sentence is
/// read by the operator who just pressed a key and saw nothing happen. Either
/// one alone has been a defect in this project — a silent decline, and a
/// sentence with no way to tell which decline it was.
fn decline(id: &str) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI.
        format!("command-declined id={id} reason=mode-cannot-edit-content")
    });
    crate::app::status::decline::record_node_tool_needs_edit_mode();
}

/// ★ Note the signature: **no `actions`**. Every control here changes how the
/// next gesture is READ and none of them changes the document, so there is
/// nothing to push. An arm that needed one would be a control that does not
/// belong in this row.
#[cfg(test)]
mod tests {
    use super::*;

    /// Every id this module answers for, and nothing else.
    #[test]
    fn it_claims_exactly_the_navigate_row() {
        for id in [
            "view.tool_select",
            "view.tool_hand",
            "view.tool_text",
            "view.tool_node",
            "view.smart_select",
        ] {
            assert!(handles(id), "{id} is in the Navigate row");
        }
        for id in [
            "view.zoom_in",
            "view.show_points",
            "edit.text",
            "view.smart",
            "view.tool",
        ] {
            assert!(!handles(id), "{id} is not");
        }
    }

    /// ★ The type is `fn(&str) -> bool` and nothing here allocates, which is
    /// what lets `app::dispatch` ask it in a guard arm on every command.
    #[test]
    fn the_claim_is_a_cheap_question() {
        let claimed = ["view.tool_select", "view.smart_select"]
            .into_iter()
            .filter(|id| handles(id))
            .count();
        assert_eq!(claimed, 2);
    }
}
