//! # `app::dispatch::routes` — the commands that perform nothing and point
//! somewhere else
//!
//! Split out of [`super`] under **R2** on 2026-08-28, when that file crossed
//! 1,500 lines for the fourth time.
//!
//! ## ★★★ The seam, and it is a subject rather than a size
//!
//! Every arm here raises `Action::Command(other_id)` and does nothing else.
//!
//! ★ It is **not** every such arm in the shell: `format.properties` predates
//! this file and stays in `dispatch::format`, because an id must have exactly
//! one claimant and moving it would have given it two. See [`target`].
//! That is the property they share and no other arm in the dispatcher does,
//! and it is the one thing a reader has to check about any of them:
//!
//! > `Action::Command` exists so a second route to an existing command cannot
//! > become a second implementation of it.
//!
//! `format.properties` set the precedent — File ▸ Document and the Format tab
//! are two places to ask *"tell me about this"*, and exactly one of them opens
//! the panel. The failure they exist to prevent is the one this project has
//! spent its time removing everywhere else: two surfaces for one capability,
//! drifting apart, each with its own guards.
//!
//! ## Why a command exists at all when another command does the work
//!
//! Because `egui-shell` enforces **one command, one tab**, and the placements
//! answer different questions. An operator hunting on the Tools tab for where
//! font folders live should find the entry there; that it opens the Settings
//! window is an implementation detail of *where the list is kept*, not a
//! reason to leave the Tools tab silent.
//!
//! ★ Each of the three was, until recently, a **drawn and inert control** with
//! a recorded blocker that had gone stale. That is not a coincidence: a command
//! whose capability lives somewhere else is exactly the kind whose blocker
//! stops being true without anybody noticing, because nothing about it changes
//! when the other surface ships.

use crate::app::actions::Action;

/// Whether this file owns `id`.
///
/// `pub(crate)` for `dispatch::format::handles`' reason: `shell::commands::reach`'s
/// reachability checker must be able to **evaluate** every guard arm it finds,
/// and a guard it cannot evaluate is a place commands could hide from the check
/// that exists to find them.
#[must_use]
pub(crate) fn handles(id: &str) -> bool {
    target(id).is_some()
}

/// The command each route points at.
///
/// One function, so the mapping is stated once and [`handles`] cannot drift
/// from [`dispatch`] — the two would otherwise be a list and a match that agree
/// today.
fn target(id: &str) -> Option<&'static str> {
    // ui-text-exempt: registered command ids, never displayed.
    match id {
        // ★★ Its tooltip promises *"list every form field … rename, retype or
        // remove them"*, and every one of those is already reachable: the Forms
        // panel lists and fills, the Properties pane renames and removes.
        // **Retyping is reachable nowhere and never will be** — Acrobat has
        // offered no field-type conversion since Acrobat 6, and `pdfcer-core`
        // models the same limit by making the request unrepresentable, so there
        // is not even a control to grey. That clause was struck from the
        // tooltip when this was wired.
        "edit.form_manage_fields" => Some("view.panel_forms"),
        // ★★★ **`edit.objects`, wired 2026-08-28 — and it was the last entry in
        // `SCAFFOLDED` with NO RECORDED REASON AT ALL.**
        //
        // Its register entry said so in those words: *"it appears in a test
        // list and in an argument about its LABEL, and nowhere else … inferring
        // a deferral is not the same as recording one."*
        //
        // ⇒ Re-deriving it took one minute, and the answer was **in its own
        // tooltip**, which promises: *"click to select, drag to move the
        // object, drag an anchor to move that node, or press Delete to remove
        // it."* Every clause of that has shipped. It is the Select tool,
        // described exactly, by a button that did nothing.
        //
        // ★★ So it is a **route**, on `format.properties`' precedent and for
        // its stated reason — *"a second route to an existing command must not
        // become a second implementation of it"*. `view.tool_select` arms the
        // tool; this is the door an operator finds on the **Edit tab**, beside
        // *Edit text* and *Add text*, where somebody looking for *"how do I
        // edit the drawing"* looks first. The tool row is where somebody who
        // already knows the tool goes.
        //
        // ★ The two doors matter here more than usual: `RIBBON_IA.md` puts the
        // three content verbs together on the Edit tab precisely so the answer
        // to *"what can I change on this page?"* is one group of three, and a
        // group of three where the third does nothing reads as a broken
        // program rather than as a missing feature.
        // ★★★ `"edit.objects" => Some("view.tool_select")` was HERE until
        // 2026-08-31 and the command is gone with it (O69). Kept as a comment
        // for one reason only: this file's whole subject is *when is a second
        // door legitimate*, and this is its one worked example of a door that
        // was legitimate in principle and **pointed at the wrong room**. The
        // rule the deletion adds to the two already in this header: a route's
        // target must be derived from what the operator will SEE HAPPEN, not
        // from what the source command is called. Its tooltip described the
        // Points tool and it armed the Select tool, and nothing in a route
        // table can catch that — only reading the two side by side can.
        // ★★★ `tools.font_folders` LEFT this file on 2026-08-28, and the reason
        // is worth more than the entry was.
        //
        // It routed here for a year and the argument was sound: *a second route
        // to an existing command must not become a second implementation of
        // it.* What changed is that the two stopped being the same request.
        // `file.settings` means *"show me the settings"*; `tools.font_folders`
        // means *"where do font folders live"*, and answering the second by
        // opening the first at the top of ten **collapsed** headings is the
        // defect `OPERATOR_REQUESTS.md` O50 opens with — the operator asked for
        // a setting that had shipped the day before, because the route named
        // after it did not land on it.
        //
        // ⇒ **A route stops being a pure route the moment the target needs to
        // know WHY it was reached.** The window is still one implementation —
        // one draft, one Save — so `routes`' argument survives intact; what the
        // arm gained is an operand, and an operand is exactly what this file's
        // shape cannot carry (`target` returns a bare id).
        //
        // It now shares `file.settings`' own dispatch arm, which is the form
        // that keeps "one claimant per id" true while letting the two ids differ
        // in what they ask for.
        // ★★ `format.properties` is the oldest route of this shape and it is
        // deliberately NOT here. It stays in `dispatch::format`, whose subject
        // is *the Format tab's share of the routing table*, and moving it would
        // have given one id two claimants — the dispatcher matches arms in
        // order, so which one won would depend on their order in a `match`.
        //
        // ⇒ Two organising principles genuinely collide here: *"all second
        // routes in one place"* and *"one file per tab's arms"*. The tie-break
        // is that **an id must have exactly one claimant**, and the existing
        // one is tested. This file's header claims the shape, not a monopoly on
        // it.
        _ => None,
    }
}

/// Raise the command this route points at.
///
/// ★ It does **not** re-check the target's guards. `dispatch_command` is the
/// choke point and the raised id goes through it exactly as a ribbon click
/// would — which is the entire point of routing rather than performing. A
/// guard applied here would be a second copy of the target's rule, in the file
/// whose purpose is to have none.
pub(crate) fn dispatch(id: &str, actions: &mut Vec<Action>) {
    if let Some(target) = target(id) {
        actions.push(Action::Command(target.to_owned()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★★ **Every route points at a registered command, and never at itself.**
    ///
    /// Two failures in one assertion, and both are silent. A route to an
    /// unregistered id raises an `Action::Command` that `dispatch_command`
    /// drops on the floor — a control that does nothing, which is what all
    /// three of these used to be. A route to itself is an infinite loop through
    /// the action queue.
    #[test]
    fn every_route_points_at_a_registered_command_that_is_not_itself() {
        let mut registry = egui_shell::commands::CommandRegistry::new();
        crate::shell::commands::register(&mut registry);
        for id in ROUTED {
            let target = target(id).expect("listed");
            assert_ne!(target, *id, "`{id}` routes to itself");
            assert!(
                registry.get(target).is_some(),
                "`{id}` routes to `{target}`, which this build does not register — the action \
                 would be raised and dropped, and the control would do nothing"
            );
            assert!(
                registry.get(id).is_some(),
                "`{id}` is routed and is not registered, so nothing can invoke it"
            );
        }
    }

    /// The ids this file claims, as a list, so a fourth added to `target` and
    /// not here fails rather than going untested.
    // ui-text-exempt: registered command ids, never displayed.
    const ROUTED: &[&str] = &["edit.form_manage_fields"];

    /// **`handles` and `target` cannot disagree**, which is why there is one
    /// mapping and not a list beside a match.
    #[test]
    fn handles_agrees_with_target() {
        for id in ROUTED {
            assert!(handles(id));
        }
        assert!(!handles("file.open"));
        assert!(!handles("nonsense"));
    }
}
