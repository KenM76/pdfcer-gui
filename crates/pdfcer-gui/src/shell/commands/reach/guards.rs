//! # `shell::commands::reach::guards` — which `handles`-style module claims an id
//!
//! One function and one list, and between them they are the reason this file
//! exists rather than being three paragraphs in its parent.
//!
//! ## Why this is its own file
//!
//! R2, and the shape of the growth is the point. `super` is a **checker**: it
//! parses the dispatcher's `match`, classifies its arms and reports any
//! registered command no arm can reach. That logic has been stable for weeks.
//!
//! What grows is this: every time a family of commands is split out of
//! `app/dispatch.rs` into its own module — six times now — a guard is added
//! here, with the paragraph explaining what moved and why. Each is short; six
//! of them, with the argument each carries, took the parent past 1,500 lines.
//!
//! ⇒ Two subjects, two rates of change, which is this project's test for a
//! seam. The checker changes when the *checking* changes; this changes when the
//! *dispatcher* is reorganised.
//!
//! ## ★★★ The recurring lesson, recorded once here instead of six times below
//!
//! **A hand-written list inside a completeness test is the gap it was built to
//! find.** Every entry below was added *after* the checker went red — the split
//! happened, this file did not know about it, and every command in the moved
//! module was reported unreachable at once.
//!
//! That is the mechanism working, and it is worth being clear about why it is
//! survivable rather than merely annoying: the failure is **loud and total**. A
//! new `dispatch::*` module is invisible here until somebody adds a line, and
//! while it is invisible *all* of its commands go red together — not one of them
//! quietly. A checker that failed open would have shipped four dead controls
//! with a green suite, which is the outcome this whole module exists to prevent.

/// The guard function that claims `id`, if any — **by calling it**.
///
/// This is the half a shell script could not have written. Each name returned
/// is the same string [`read_arms`] extracts from the guard arm that consults
/// it, so the two halves can be compared as sets; and each answer comes from
/// the real mapping rather than from a re-derivation of it, so there is no
/// second table to drift. [`super::super::mapping`]'s header states the property this
/// preserves: *"two hand-written tables can disagree, and one table plus a
/// derived search cannot."*
///
/// Order is irrelevant here even though it is load-bearing in the dispatcher,
/// where `match` takes the first arm that matches. Reachability asks only
/// whether **some** arm claims the id; which one wins is asserted, in both
/// directions, by the disjointness tests in [`super::super::mapping`].
pub(crate) fn guard_claiming(id: &str) -> Option<&'static str> {
    // ui-text-exempt: Rust function names, compared against the parsed syntax tree.
    if super::super::measure_for_command(id).is_some() {
        return Some("measure_for_command");
    }
    if super::super::text_mark_for_command(id).is_some() {
        return Some("text_mark_for_command");
    }
    if super::super::markup_for_command(id).is_some() {
        return Some("markup_for_command");
    }
    // ★ The five form-field commands, claimed by the same shape as markup's.
    // This checker MIRRORS the dispatcher rather than trusting it, so a guard
    // added there and not here is caught by
    // `the_guards_the_checker_evaluates_are_the_guards_the_dispatcher_has` —
    // which is exactly how this line came to be written.
    if super::super::form_for_command(id).is_some() {
        return Some("form_for_command");
    }
    // ★ The text-bearing markup kinds, whose guard is an associated function
    // on the kind rather than a free function in `mapping`.
    //
    // The checker reads the NAME the dispatcher calls, and the dispatcher calls
    // `TextAnnotKind::from_command`, so that is the string — the path is not
    // part of what it matches. It sits beside `markup_for_command` because a
    // reader asking "what claims a `markup.*` id?" needs both answers together.
    if crate::canvas::textannot::TextAnnotKind::from_command(id).is_some() {
        return Some("from_command");
    }
    if super::super::page_display_for_command(id).is_some() {
        return Some("page_display_for_command");
    }
    if super::super::chrome_for_command(id).is_some() {
        return Some("chrome_for_command");
    }
    if crate::panels::Panel::from_command_id(id).is_some() {
        return Some("from_command_id");
    }
    // ★ The Pages tab's arms, which live in `app::dispatch::pages` since that
    // file was split out under R2 on 2026-08-18.
    //
    // This one differs from every entry above in a way worth naming: the others
    // guard on a *mapping* that also produces the operand (a `MarkupKind`, a
    // `Panel`), so evaluating the guard and dispatching are the same question
    // asked once. `handles` produces nothing — it is a membership test, and its
    // partner `dispatch` matches the id again.
    //
    // That is two statements of one set, which is the shape this crate usually
    // refuses. It is accepted here because the two sit adjacent in one small
    // file and because `dispatch`'s fall-through is `unreachable!` naming the
    // id — so a member of `handles` missing from `dispatch` panics loudly in a
    // developer build rather than silently doing nothing. Were they to grow
    // apart, the fix is to make `handles` return an operand like its siblings.
    if crate::app::dispatch::pages::handles(id) {
        return Some("handles");
    }
    // ★ The third membership-test guard, added 2026-08-28 with
    // `dispatch::routes` — the commands that perform nothing and raise
    // `Action::Command` at something that does. Everything the paragraph above
    // says about `pages::handles` applies unchanged, with one improvement worth
    // naming: `routes` has **one** mapping rather than a list beside a match,
    // and its `handles` is defined as `target(id).is_some()`. So the two
    // statements that could grow apart are one statement, which is what that
    // paragraph asks for as the eventual fix.
    if crate::app::dispatch::routes::handles(id) {
        return Some("handles");
    }
    // ★ The font commands, in `app::dispatch::fonts` since 2026-08-28, when
    // wiring `tools.embed_fonts` took `dispatch.rs` past 1,500 lines for the
    // fifth time. Everything the `pages::handles` paragraph says applies.
    //
    // ★★ **It failed closed here too, and by name**, which is now the fourth
    // time this checker has caught a split the moment it happened: moving the
    // arm out of the parent made `tools.embed_fonts` report as having no
    // dispatch arm at all, one test run after it was wired. A checker that
    // reads the parent's syntax tree cannot see a child's arm until it is told,
    // and being told is exactly this line.
    if crate::app::dispatch::fonts::handles(id) {
        return Some("handles");
    }
    // ★ `dispatch::batch`, split out 2026-08-31 when `tools.merge_files` was
    // wired (`OPERATOR_REQUESTS.md` O68). Everything the `pages::handles`
    // paragraph says applies unchanged, mitigation included: `batch::dispatch`
    // ends in `unreachable!` naming the id.
    //
    // ★★ Added in the SAME commit as the arm, deliberately. This checker has
    // failed closed five times on exactly this — a new `dispatch::*` module is
    // invisible here until somebody adds a line, and while it is invisible all
    // of its commands report as unreachable. Writing the entry with the module
    // rather than after the test goes red is the only version of this that does
    // not cost a diagnosis.
    if crate::app::dispatch::batch::handles(id) {
        return Some("handles");
    }
    // ★ `dispatch::navigate`, split out 2026-08-31 when `view.smart_select`
    // (`OPERATOR_REQUESTS.md` O70) took `dispatch.rs` past 1,500 lines for the
    // seventh time. It carries the five controls of View ▸ Navigate.
    //
    // ★★ **Written in the same edit as the module**, per the paragraph above —
    // and this one still went red first, because the arms were MOVED before
    // this line was added. Five commands that had worked for weeks reported as
    // unreachable in one run, which is precisely the loud-and-total failure
    // that makes a hand-kept list survivable here. It is not a list of what
    // exists; it is a list of what has been noticed, and the test is what does
    // the noticing.
    if crate::app::dispatch::navigate::handles(id) {
        return Some("handles");
    }
    // ★ The Settings commands, in `app::dispatch::settings` since 2026-08-28,
    // when this file crossed 1,500 lines for the third time in one session.
    //
    // ★★ It carries an id `routes` used to own — `tools.font_folders` — so this
    // is also the one split where a command MOVED between two guard-arm modules
    // rather than out of the parent. `routes`' `ROUTED` list shrank to one in
    // the same commit, and `every_route_points_at_a_registered_command_that_is_
    // not_itself` is what would have failed had it not.
    if crate::app::dispatch::settings::handles(id) {
        return Some("handles");
    }
    // ★ `dispatch::text`, split out 2026-08-28 with paragraph reflow. Same
    // shape, same reason, one difference worth naming: this one's `dispatch`
    // cannot trace an "unrouted" line for a member it does not know, because it
    // has exactly one member. If it grows a second, it owes that trace.
    if crate::app::dispatch::text::handles(id) {
        return Some("handles");
    }
    // ★ The second membership-test guard, added 2026-08-20 with
    // `dispatch::textcopy`. Everything the paragraph above says about
    // `pages::handles` applies to it unchanged — including the mitigation: its
    // partner `dispatch` traces `textcopy-unrouted` on a member it does not
    // know, rather than doing nothing.
    //
    // Both resolve to the string `"handles"` because that is the **guard's own
    // spelling** in `dispatch.rs`, which is what
    // `the_guards_the_checker_evaluates_are_the_guards_the_dispatcher_has`
    // reads out of the syntax tree. Two modules sharing one predicate name is
    // therefore one entry here, not two.
    if crate::app::dispatch::textcopy::handles(id) {
        return Some("handles");
    }
    // ★★ The guard added 2026-08-29 with `dispatch::clipboard`, when
    // `edit.paste_duplicate` (O58) took `dispatch.rs` past 1,500 lines for the
    // FOURTH time. Everything the `pages::handles` paragraph says applies
    // unchanged.
    //
    // ★★★ **And it failed closed for the fifth time, which is the point of
    // recording each one.** The instant the three clipboard arms moved out of
    // the parent, this checker reported `edit.cut`, `edit.copy`, `edit.paste`
    // and `edit.paste_duplicate` as registered controls with no dispatch arm —
    // four commands the operator presses daily, reported as unreachable, by a
    // test that had no idea a new module existed.
    //
    // ⇒ That is the failure mode this project has named before and keeps
    // meeting: **a hand-written list inside a completeness test is the gap it
    // was built to find.** The list is still hand-written, and the mitigation
    // is that it fails LOUDLY rather than silently — a new `dispatch::*` module
    // is invisible to this checker until someone adds a line here, and what
    // makes that survivable is that every command in the moved module goes red
    // at once. A checker that failed open would have shipped four dead
    // controls with a green suite.
    if crate::app::dispatch::clipboard::handles(id) {
        return Some("handles");
    }
    // ★★ The guard added 2026-08-29 with `dispatch::pageclip` — O59 item 2, the
    // page clipboard. Sixth `handles` module, and **it failed closed for the
    // sixth time**: the instant `pages.copy`, `pages.cut` and `pages.paste`
    // were registered, this checker reported all three as controls an operator
    // can press that trace `command-unimplemented` and do nothing.
    //
    // ⇒ Which is the mechanism working exactly as it should, and is also the
    // recurring cost of it: **a hand-written list inside a completeness test is
    // the gap it was built to find.** The mitigation is unchanged and is what
    // makes it survivable — every command in a newly split module goes red at
    // once, loudly, rather than one of them going quietly dead.
    if crate::app::dispatch::pageclip::handles(id) {
        return Some("handles");
    }
    // ★ The third membership-test guard, added 2026-08-24 with
    // `dispatch::zoom`, when O29's third fit mode took `dispatch.rs` past
    // 1,500 lines. Everything the `pages::handles` paragraph above says
    // applies unchanged, mitigation included: its partner `dispatch` ends in
    // an `unreachable!` naming the id, so a member of `handles` missing from
    // the match panics loudly rather than silently doing nothing.
    if crate::app::dispatch::zoom::handles(id) {
        return Some("handles");
    }
    // ★ The fourth membership-test guard, added 2026-08-27 with
    // `dispatch::format`, when the form-XObject work took `dispatch.rs` past
    // 1,500 lines for the third time. Everything the `pages::handles`
    // paragraph above says applies unchanged, mitigation included: its partner
    // `dispatch` ends in an `unreachable!` naming the id.
    //
    // ★★ **It failed closed, again, and by name.** The moment the three
    // `format.*` arms moved out of the parent, this checker reported
    // `format.delete`, `format.properties` and `format.select_form` as
    // registered controls with no dispatch arm — which is the fourth time the
    // prediction in `DISPATCH_PAGES_SRC`'s doc has come true, and the fourth
    // time the failure was a correct report rather than a false alarm. A grep
    // for the id strings would have found them in the new file and said
    // nothing.
    if crate::app::dispatch::format::handles(id) {
        return Some("handles");
    }
    None
}

/// Every guard [`guard_claiming`] knows how to run.
///
/// **Not a mirror of the dispatcher**, and the distinction is the one `D5`
/// turns on: this list is *asserted equal* to the set read out of
/// `dispatch.rs`'s syntax tree by
/// [`tests::the_guards_the_checker_evaluates_are_the_guards_the_dispatcher_has`],
/// so it cannot drift without a named failure. A hand-maintained list that
/// nothing checks is the defect; a hand-written list that a test pins against
/// the source is a declaration.
pub(crate) const EVALUATED_GUARDS: &[&str] = &[
    // ui-text-exempt: Rust function names, compared against the parsed syntax tree.
    "measure_for_command",
    // ui-text-exempt: Rust function names, compared against the parsed syntax tree.
    "text_mark_for_command",
    // ui-text-exempt: Rust function names, compared against the parsed syntax tree.
    "markup_for_command",
    // ui-text-exempt: Rust function names, compared against the parsed syntax tree.
    "form_for_command",
    // ui-text-exempt: Rust function names, compared against the parsed syntax tree.
    "from_command",
    // ui-text-exempt: Rust function names, compared against the parsed syntax tree.
    "page_display_for_command",
    // ui-text-exempt: Rust function names, compared against the parsed syntax tree.
    "chrome_for_command",
    // ui-text-exempt: Rust function names, compared against the parsed syntax tree.
    "handles",
    // ui-text-exempt: Rust function names, compared against the parsed syntax tree.
    "from_command_id",
];
