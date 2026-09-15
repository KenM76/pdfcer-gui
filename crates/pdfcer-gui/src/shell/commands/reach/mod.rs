//! # `shell::commands::reach` — the sixth obligation: a registered command
//! must be **reachable**
//!
//! Registering a command creates five obligations, and every one of them fails
//! loudly: a count assertion, a group assertion, a `PLANNED` disjointness
//! test, a RON round-trip, a `KNOWN` lookup. **None of the five asks whether
//! the command does anything**, and that is the gap this module closes.
//!
//! ## What went wrong, and how many surfaces agreed with it
//!
//! `file.save_copy` was registered, drawn on the **quick-access toolbar**,
//! bound to `Ctrl+S`, listed in the shortcuts reference, and printed
//! "(Ctrl+S)" in its own tooltip — with **no dispatch arm**. Nothing this
//! shell built could be written to disk, for the whole life of the project,
//! and it was within an hour of being released that way. An audit the same
//! day found the identical shape in `edit.undo`/`edit.redo` (QAT, three
//! chords) and in **every** page operation, six of which the Pages panel's
//! context menu offered while `panels/pages/select.rs` maintained a
//! multi-select model to feed them.
//!
//! Five surfaces promise a command works — the registry, the ribbon, the
//! QAT, the keymap, the tooltip — and **not one of them is a dispatch arm.**
//! The only honest signal is a `command-unimplemented` line in the trace, and
//! nothing read it.
//!
//! ## The property asserted here
//!
//! > Every command in the registry is either named by a **literal arm** of
//! > `PdfcerApp::dispatch_command`'s `match`, or claimed by one of its
//! > **guard arms**, or listed in [`SCAFFOLDED`] with a written reason.
//!
//! It is a statement about **routing**, not about behaviour. An arm that
//! declines — `measure.finish` refusing because the mode cannot author
//! dimensions — is reachable, and correctly so: the operator's press produced
//! a decision instead of falling through to `command-unimplemented`. What this
//! catches is the *absence of a decision*.
//!
//! # ★ Why the arms are READ from the source rather than run
//!
//! Three mechanisms were available. The two that lost are worth recording,
//! because each lost for a reason a future session would otherwise re-derive.
//!
//! ## Rejected — dispatch every command in a test and assert it was handled
//!
//! The truest signal, and unavailable. `crate::app::files`' header states the
//! rule in its own words as **Rule 3: no test may dispatch `file.open`** —
//! "on the machine this is built on, dispatching `file.open` opens a **real
//! modal dialog** and blocks until a human dismisses it. A `cargo test` that
//! did that would hang the suite with an invisible window behind the
//! terminal."
//!
//! The escape hatch does not open either. `PDFCER_DIAG_OPEN_PATH` answers the
//! dialog without a human, but setting it needs `std::env::set_var`, which is
//! `unsafe` in edition 2024 while this crate is `#![forbid(unsafe_code)]` —
//! the same wall that leaves `files::from_env`'s environment read as its one
//! untested millimetre.
//!
//! So a dispatching test would have to **exempt `file.open`**, and that is
//! decisive rather than inconvenient: `file.open` is the command this defect
//! struck with the largest blast radius — registered, on the File tab, on the
//! QAT, bound to `Ctrl+O`, and armless, so the only way to open a document was
//! `argv`. A check that must skip the worst historical instance of the defect
//! it exists to prevent is not a check.
//!
//! (The other hazards are real but were *not* the deciding ones, and it is
//! worth saying which, so nobody re-opens this on the wrong grounds. Almost
//! every arm is safe in a test, because of this shell's first invariant:
//! **actions, not mutations.** `pages.delete` pushes
//! `Action::DeletePages` into a `Vec` the caller owns and deletes nothing;
//! `file.ocr` sets a dialog-open flag and starts no recogniser. The genuinely
//! effectful arms are the clipboard writes and the native picker, and of those
//! only the picker cannot be reached in a state where it does nothing.)
//!
//! ## Rejected — a `bash` gate that greps `dispatch.rs`
//!
//! Honest for a gate, and wrong for this file, for two independent reasons.
//!
//! **A `match` is not a regular language.** The failure that matters is a
//! *false pass*, and a grep for `"some.id" =>` has at least three ways to
//! produce one: a string inside a comment, the left-hand side of a **nested**
//! `match` inside an arm's body (`dispatch_command` contains four), and a doc
//! comment quoting an id whose arm has since been deleted — which is the D5
//! shape exactly, a list that agrees with itself about something that stopped
//! being true. `check-ui-strings.sh`'s header records the same class of error
//! from the other side: its first regex read `"svg" | "?xml"` as one literal
//! containing `" | "`, and "three of the four remaining hits… were exactly
//! that artefact — i.e. most of what was left after the real exclusions was
//! the detector misreading Rust, not the code violating the rule."
//!
//! **The guard arms are expressions, and a shell cannot evaluate them.** Six
//! arms have the shape `id if …_for_command(id).is_some()`, and the functions
//! behind them search enum tables in three other modules. A gate that decided
//! which ids they claim would have to re-derive `markup_command`,
//! `measure_command`, `chrome_command`, `page_display_command`,
//! `text_mark_command` and `Panel::command_id` from source — six more `match`
//! blocks to parse, and, worse, a **second table** of exactly the kind
//! [`super::mapping`]'s header exists to forbid: *"two hand-written tables can
//! disagree, and one table plus a derived search cannot."*
//!
//! ## Rejected — restructure so one `fn arm_for(id) -> Option<Arm>` is the
//! source of truth
//!
//! The best signal and the most invasive, and it contradicts the file it would
//! restructure. `app::dispatch`'s header states the property being protected:
//! **"the arms route; they do not compute"**, each arm one line that pushes an
//! `Action` or calls the one function that owns the rule. Interposing a table
//! turns every arm into two lookups — a variant, then a body — and the thing
//! a reader currently gets for free, that `"file.close" => actions.push(
//! Action::Close)` is the whole story, is precisely what would be lost. The
//! brief for this work says so in the same words: it must not turn one
//! readable `match` into a table nobody can follow.
//!
//! # ★ What is done instead, and why it is not the rejected grep
//!
//! **The literal arms are read from the abstract syntax tree**, with `syn` —
//! a real Rust parser, already in `D:\Dev\pdfcer`'s lockfile, taken as a
//! dev-dependency on the standard `flate2` and `rfd` are held to (see this
//! crate's `Cargo.toml`). Every objection above is an objection to treating
//! Rust as text, and none of them survives parsing it as Rust: an arm pattern
//! is an `Arm`'s `pat`, a comment is not in the tree at all, and a nested
//! `match` inside an arm's **body** is never visited, because only the arms of
//! the one `match` on `id` are read. `the_reader_does_not_see_a_nested_match`
//! pins that last one against a fixture, since it is the case a grep gets
//! wrong and the case nobody would notice.
//!
//! **The guard arms are not parsed. They are CALLED.** The tree says *which*
//! functions guard arms consult ([`Arms::guards`], the last path segment of
//! whatever is invoked with `id`); Rust then answers, for every registered id,
//! whether any of them claims it — by running the real
//! [`super::markup_for_command`] and its five siblings against the real
//! registry. There is no second table, because there is no table: the one
//! `match` in [`super::mapping`] is the only statement of each mapping, and
//! this reads it by executing it.
//!
//! The two halves are then held together by
//! [`tests::the_guards_the_checker_evaluates_are_the_guards_the_dispatcher_has`],
//! which asserts that the guard names found **in the source** and the guard
//! names this module **evaluates** are the same set. That closes the hole a
//! hand-kept list would otherwise open in both directions: a *seventh* guard
//! arm added to `dispatch.rs` fails by name rather than silently reporting its
//! whole family unreachable, and a guard arm *deleted* from `dispatch.rs`
//! stops making its family reachable rather than being vouched for by a
//! function that still exists.
//!
//! # Why this is a test and not a `tools/gates/` script
//!
//! Because both halves of the answer are Rust. The registry is built by
//! [`super::register`], the guards are functions with enum tables behind them,
//! and a shell script could reach neither without re-deriving both — which is
//! the failure the paragraphs above are about. What a gate script contributes
//! that a test does not is a *precondition* guarantee, and this has a stronger
//! one than any script can offer: [`DISPATCH_SRC`] is an `include_str!`, so a
//! dispatcher that has been moved, renamed or deleted **fails to compile**
//! rather than being quietly scanned as an empty tree. `run-all.sh`'s
//! three-state model exists because "found nothing" and "looked at nothing"
//! print the same thing; here the second state cannot be reached.
//!
//! # What the register holds, and what empties it
//!
//! [`SCAFFOLDED`] is the allow-list of registered commands with no dispatch
//! arm, each carrying the reason it is inert. **It is empty**, and
//! [`tests::the_p3_tension_is_counted`] pins both its length and the `★ P3`
//! subset at zero. A `★ P3` mark is this module's judgement that the honest
//! answer is *the control should not be drawn yet* — `RIBBON_IA.md` P3: "An
//! unavailable capability renders nothing, not a disabled stub".
//!
//! An entry leaves this list two ways, and only two: the command is **wired**,
//! or the command is **unregistered**, which is R9's answer — a capability that
//! is not built renders nothing — and is legitimate because of R8: *registering
//! a command is the only way the GUI may learn that a capability exists.* An
//! entry is never **reworded** when its reason expires;
//! `no_scaffolded_entry_is_stale`'s middle assertion exists to force that,
//! because a reason rewritten after a blocker clears gets none of the scrutiny
//! the original had.
//!
//! ## The kinds of reason, because only one kind can be worked on
//!
//! * a **blocker** — something is missing and somebody can build it. It expires
//!   when they do, and the entry is then deleted.
//! * a **decision** — a deferral. It expires only when the person who made it
//!   changes it. Whether a `★ P3` entry loses its control is a taxonomy
//!   decision and the operator's; nothing here removes one.
//! * **no reason at all**, which is not a deferral. An entry that admits it has
//!   no recorded reason is the FIRST one to re-derive, not the last: somebody
//!   has already established that nothing is defending it, and the confession
//!   reads like the output of a search that already happened.
//!
//! ⚠ A blocker can also be **correct for the wrong reason** — the id genuinely
//! has no arm, and the recorded cause is not the real one. Nothing about such
//! an entry looks wrong; the only thing that finds it is asking what the verb's
//! own REQUEST STRUCT requires rather than whether the verb exists.
//!
//! ★ The reliable half is identifiable in advance: **an entry whose truth
//! condition is inside THIS repository is the strong kind** — nothing makes a
//! missing window appear except somebody building it, so it cannot go stale by
//! accident. An entry that cites another document or another repository can,
//! and does. A reason that is a citation of a citation, with nothing re-reading
//! either, is the commonest way this list goes false.
//!
//! ⇒ **Re-derive the list on a schedule, not on a collision**, and when you
//! touch it for any purpose re-derive the reason of the entry beside the one
//! you came for. This assertion cannot help: it asks whether an id has an arm,
//! and an entry whose id has no arm and whose reason is nonsense is
//! indistinguishable from a correct one. A reason is prose; a reader is the
//! only instrument.
//!
//! ## ★★★ The honest verdict on this list
//!
//! It forces an explanation for every dead control. It has never forced a fix.
//! An entry can sit for weeks with a reason that is true about one thing and
//! false about the requirement, nine lines above the note naming that exact
//! failure mode, and survive an audit that re-derives its neighbours — and then
//! be found by the operator pressing the button.
//!
//! ⇒ The replacement is not a better list. It is a **driven check that presses
//! every registered id and fails on `command-unimplemented`** — a claim about
//! the running program, which no paragraph can satisfy. See `tools/ui-verify`.
//!
//! ★★ The empty list is **kept rather than deleted**, exactly as
//! [`UNREACHED_ARMS`] is kept at zero and for the same reason: an empty
//! allow-list is still a gate. A new entry cannot be added quietly — it has to
//! be written here with a reason, and the count assertion is what makes adding
//! one a visible act.
//!
//! ★ **And the mirror defect, which the same reader finds.** A literal arm can
//! name a command that is **not registered at all**, so no token can reach it
//! and no operator ever could. [`UNREACHED_ARMS`] is the allow-list for those.
//! It is empty and is kept as a gate: the first planted violation of this check
//! was one of those arms and the check said nothing.
//!
//! The gate discipline is kept in full. [`tests`] contains a self-test that
//! plants a violation in a fixture and proves the reader reports it, another
//! that proves the reader does **not** report a clean fixture, and two that
//! aim at the specific misreadings a grep would make — because, in
//! `check-file-size.sh`'s words, a gate that has never been observed to fail
//! is not evidence of anything.

/// ★ Which `handles`-style module claims an id — six guards and the
/// paragraph each carries. Split out under R2; its header records the
/// recurring lesson the six of them are evidence for.
mod guards;

use std::collections::{BTreeMap, BTreeSet};

/// The dispatcher's own source, embedded at **compile** time.
///
/// `include_str!` rather than a runtime `std::fs::read_to_string` of a path
/// built from `CARGO_MANIFEST_DIR`, and the difference is the whole
/// precondition story. A path that stops resolving is a *runtime* `Err` that
/// somebody has to remember to treat as a failure; `run-all.sh`'s header is
/// about exactly that ("SKIPPED is not PASSED"). A missing `include_str!`
/// target is a **compile error**, so the state in which this module checks
/// nothing and says so quietly does not exist.
/// ★ The register — every registered command with no dispatch arm, and why.
///
/// Split out at rule R2's ceiling, and the seam is a real one: this file is
/// the **check** and that one is the **data**. See its header for
/// why the data half is the one that grows, and why trimming a reason to fit is
/// the worst available response.
pub mod register;

pub(crate) use register::{SCAFFOLDED, UNREACHED_ARMS};

const DISPATCH_SRC: &str = include_str!("../../../app/dispatch.rs");

/// ★ The **second** file the routing table lives in.
///
/// # Why there are two, and why the checker had to learn about it
///
/// `app::dispatch` reaches R2's 1,500-line limit, so the Pages tab's arms live
/// in `app::dispatch::pages`, behind a guard arm (`id if pages::handles(id)`).
/// The parent file does not *contain* those six commands anywhere a `syn` walk
/// of it can see.
///
/// A checker that reads only the parent reports `pages.delete`,
/// `pages.extract`, `pages.move_up`, `pages.move_down` and both rotates as
/// unreachable — which is exactly the behaviour this module's header argues
/// for over a `bash` grep: **it fails closed.** A grep would find the string
/// `"pages.delete"` in either file and say nothing.
///
/// The lesson worth keeping is that a checker which reads ONE file is a
/// checker with a shelf life: R2 guarantees that any file it reads will
/// eventually be split, so *"where is the routing table?"* is a question with a
/// growing answer. Adding a source here is the cheap half; the expensive half
/// would have been discovering the blindness from an operator pressing a dead
/// control.
const DISPATCH_PAGES_SRC: &str = include_str!("../../../app/dispatch/pages.rs");

/// The measure dispatcher, split out of `dispatch.rs`.
///
/// ★ **The second source [`DISPATCH_PAGES_SRC`]'s own doc predicts**: a checker
/// which reads ONE file is a checker with a shelf life, because R2 guarantees
/// that any file it reads will eventually be split.
///
/// It fails closed again, and by name — six `measure.*` ids reported
/// unreachable the moment the arms move — which is the behaviour that
/// paragraph argues for over a `bash` grep. A grep would find the string
/// `"measure.set_scale"` in either file and say nothing.
///
/// ⇒ **The next split will need a line here too**, and the failure that costs
/// nothing to fix is this one rather than an operator pressing a dead control.
const DISPATCH_MEASURE_SRC: &str = include_str!("../../../app/dispatch/measure.rs");

/// This module's parent, read for the `&'static str` constants that arm
/// patterns may name.
///
/// One arm of the dispatcher is written `crate::shell::commands::FILE_RECENT
/// => …` rather than as a literal, for the reason that constant's own doc
/// comment gives: the id is spelled in four places that must agree, and "a
/// typo in any of them produces silence — a menu that draws and reports
/// nothing — rather than an error."
///
/// So the reader resolves constant patterns instead of ignoring them, and it
/// resolves them **by parsing the file that defines them** rather than by
/// carrying a copy of the value. A copy would be a hand-maintained mirror of
/// exactly one entry, which is still the thing `DEFECTS.md` D5 forbids: *"a
/// hand-maintained list with a comment telling you to hand-maintain it has
/// already failed once."*
const CONSTS_SRC: &str = include_str!("../mod.rs");

/// The method whose `match` is the routing table.
// ui-text-exempt: a Rust item name, matched against the parsed syntax tree.
const DISPATCHER: &str = "dispatch_command";

/// The name of the **free function** a split-out dispatcher file holds its
/// match in.
///
/// [`DISPATCHER`] is an inherent method on `PdfcerApp`; a file split out under
/// R2 has no `impl` block to hang one on, so its entry point is a plain
/// function. Two names rather than one loosened matcher, because "any function
/// containing a match" would start classifying helper functions as routing
/// tables the day somebody wrote one.
const SPLIT_DISPATCHER: &str = "dispatch";

/// The identifier the routing `match` scrutinises, and that its guard arms
/// pass to the mapping functions.
// ui-text-exempt: a Rust binding name, matched against the parsed syntax tree.
const SUBJECT: &str = "id";

// ===========================================================================
// READING THE ARMS
// ===========================================================================

/// What one `match` offers: the ids its literal arms name, and the guard
/// functions its guard arms consult.
///
/// Both are sets rather than lists because the question asked of them is only
/// ever membership, and because a duplicate arm is a `match` the compiler
/// already warns about.
#[derive(Debug, Default)]
pub(super) struct Arms {
    /// Every id named by an arm pattern — a string literal, an alternation of
    /// them, or a path naming a `&'static str` constant that resolves.
    pub(super) literals: BTreeSet<String>,
    /// The **last path segment** of each function a guard arm calls with the
    /// subject: `markup_for_command`, `from_command_id`, and so on.
    ///
    /// The last segment rather than the whole path, because the path is a
    /// spelling decision (`crate::shell::commands::markup_for_command` here,
    /// a `use` away from `markup_for_command` in a future edit) and the
    /// function is the fact.
    pub(super) guards: BTreeSet<String>,
    /// Whether a catch-all arm — a binding or `_` — is present.
    ///
    /// Asserted rather than used: the catch-all is where
    /// `command-unimplemented` is traced, so a `match` without one is not the
    /// `match` this module thinks it is reading.
    pub(super) catch_all: bool,
    /// ★★★ The ids named by an arm carrying `#[cfg(feature = "…")]` — a
    /// **conditional** arm, `SHELL_FRAMEWORK.md` §5b's dispatch-side twin of
    /// the manifest's `capability:` field.
    ///
    /// These ids are also in [`Self::literals`], so the *"is every registered
    /// command routed?"* direction sees them and is satisfied in the build
    /// where the command exists. They are listed separately so the **mirror**
    /// direction — *"does any arm name a command that is not registered?"* —
    /// can exempt them, because in a build with the capability compiled out
    /// that is precisely the intended state: the arm is in the source, the
    /// command is not in the registry, and no token can reach it on purpose.
    ///
    /// ⚠ The two directions must NOT be handled by one rule. Skipping a
    /// conditional arm entirely would report `file.sign` as *"registered and
    /// unrouted"* in the build where it IS registered — i.e. would call a
    /// working, dispatched, driven control an unwired one. That is what
    /// happened on the first attempt, and it is the reason this is a second
    /// set rather than a `continue`.
    pub(super) conditional: BTreeSet<String>,
}

/// Read the routing table out of `src`.
///
/// `src` is a parameter rather than a reach for [`DISPATCH_SRC`] for the
/// reason `crate::diag::record_if_changed` takes its map as an argument: the
/// **rule** is the interesting part and it has to be testable against a
/// fixture. A reader that can only be pointed at the real file cannot be shown
/// to bite.
///
/// # Errors
///
/// Returns the reason as a string when the source does not parse, when no
/// method named [`DISPATCHER`] holds a `match`, or when an arm pattern is a
/// shape this reader does not classify. **All three fail closed**: an
/// unreadable dispatcher reports *nothing* reachable rather than everything,
/// which is the direction that makes a caller notice.
pub(super) fn read_arms(src: &str, consts: &BTreeMap<String, String>) -> Result<Arms, String> {
    let file = syn::parse_file(src).map_err(|e| {
        // ui-text-exempt: a test failure message, never displayed to an operator.
        format!("the dispatcher does not parse as Rust: {e}")
    })?;
    let matched = find_routing_match(&file).ok_or_else(|| {
        // ui-text-exempt: a test failure message, never displayed to an operator.
        format!("no `match` was found in a method named `{DISPATCHER}`")
    })?;

    let mut arms = Arms::default();
    for (n, arm) in matched.arms.iter().enumerate() {
        // ★★★ A CONDITIONAL ARM IS SKIPPED, and this is `SHELL_FRAMEWORK.md`
        // §5b arriving in the one instrument that reads the dispatcher as text.
        //
        // An arm carrying `#[cfg(feature = "…")]` names a command that is
        // registered in some builds and not in others. This reader parses
        // source, so it sees the arm whichever build it is compiled into — and
        // in the build where the feature is off it would report
        // *"1 dispatch arm names a command that is not registered"* about an
        // arm that is correct, present on purpose, and unreachable exactly as
        // intended.
        //
        // ⚠ The alternative the failure message itself suggests — put it in
        // `UNREACHED_ARMS` with a reason — is **wrong here**, and the
        // difference matters: that list is for an arm waiting for a command
        // that does not exist yet, i.e. a promise. This arm is not waiting for
        // anything; in a default build it is live, dispatched and driven. A
        // permanent entry on the unreached list would make a working control
        // look like an outstanding work item forever.
        //
        // ★ It is skipped rather than recorded as a third category because
        // this reader's whole output is *"which ids can a token reach"*, and
        // the honest answer for a conditional arm is *"it depends on the
        // build"* — which the registry already answers, in the build that is
        // running, at `shell::tests::the_signing_command_is_registered_exactly
        // _when_the_feature_is_on`.
        let conditional = arm.attrs.iter().any(|a| a.path().is_ident("cfg"));
        // A guard arm is classified by what it CALLS, never by what it
        // matches: its pattern is the binding `id`, which names no command.
        if let Some((_, guard)) = &arm.guard {
            let name = guard_subject_fn(guard).ok_or_else(|| {
                // ui-text-exempt: a test failure message, never displayed to an operator.
                format!(
                    "arm {n} is guarded by an expression that calls nothing with `{SUBJECT}`; \
                     this reader cannot tell which commands it claims"
                )
            })?;
            arms.guards.insert(name);
            continue;
        }
        let before: BTreeSet<String> = if conditional {
            arms.literals.clone()
        } else {
            BTreeSet::new()
        };
        collect_pattern(&arm.pat, consts, n, &mut arms)?;
        if conditional {
            // Whatever this arm added, recorded a second time as conditional.
            // Read as a difference rather than by re-walking the pattern,
            // because `collect_pattern` recurses through `Pat::Or` and a second
            // walk would be a second implementation of the same classification.
            let added: Vec<String> = arms.literals.difference(&before).cloned().collect();
            arms.conditional.extend(added);
        }
    }
    Ok(arms)
}

/// Classify one arm pattern into [`Arms`].
///
/// Split from [`read_arms`] because `Pat::Or` recurses into it once per
/// alternative — `"pages.rotate_left" | "pages.rotate_right"` is one arm and
/// two ids — and writing that inline would put the classification rule in two
/// places.
fn collect_pattern(
    pat: &syn::Pat,
    consts: &BTreeMap<String, String>,
    n: usize,
    arms: &mut Arms,
) -> Result<(), String> {
    match pat {
        // `"file.new" => …`
        syn::Pat::Lit(lit) => match &lit.lit {
            syn::Lit::Str(s) => {
                arms.literals.insert(s.value());
                Ok(())
            }
            // ui-text-exempt: a test failure message, never displayed to an operator.
            _ => Err(unclassifiable(n, "a non-string literal pattern")),
        },
        // `"a" | "b" => …`
        syn::Pat::Or(or) => or
            .cases
            .iter()
            .try_for_each(|case| collect_pattern(case, consts, n, arms)),
        // `crate::shell::commands::FILE_RECENT => …`
        //
        // Resolved through the constant table, and **unresolvable is an
        // error rather than a shrug**: a path pattern this reader cannot
        // resolve is an arm whose id it does not know, and silently
        // dropping it is how a real arm comes to look like no arm at all.
        syn::Pat::Path(path) => {
            let last = path
                .path
                .segments
                .last()
                .map(|s| s.ident.to_string())
                .unwrap_or_default();
            match consts.get(&last) {
                Some(value) => {
                    arms.literals.insert(value.clone());
                    Ok(())
                }
                None => Err(format!(
                    // ui-text-exempt: a test failure message, never displayed to an operator.
                    "arm {n} matches the path `…::{last}`, which is not a `&str` constant \
                     this reader can resolve"
                )),
            }
        }
        // `other => …` / `_ => …`
        syn::Pat::Ident(_) | syn::Pat::Wild(_) => {
            arms.catch_all = true;
            Ok(())
        }
        _ => Err(unclassifiable(
            n,
            // ui-text-exempt: a test failure message, never displayed to an operator.
            "a pattern shape this reader does not know",
        )),
    }
}

/// The message for an arm this reader will not guess at.
///
/// Failing rather than ignoring is the whole discipline: an arm the reader
/// cannot classify is an arm whose ids it would otherwise report as
/// unreachable *or* miss entirely, and neither silence is acceptable in a
/// check whose only interesting failure is a false pass.
fn unclassifiable(n: usize, what: &str) -> String {
    // ui-text-exempt: a test failure message, never displayed to an operator.
    format!("arm {n} is {what}; teach `collect_pattern` about it rather than ignoring it")
}

/// The `match` that routes commands: the first one found directly in the body
/// of a method named [`DISPATCHER`].
///
/// Deliberately **not** a search for any `match` anywhere in the file.
/// `dispatch_command` contains four nested `match` expressions inside arm
/// bodies (the recent-file operand, the text-mark outcome, the page-move
/// refusal, the page-text failure), and one of them — the refusal — has string
/// literals on the *right* of its arrows. Reading the wrong one is the mistake
/// a grep makes; reading the right one is the reason this walks a tree.
fn find_routing_match(file: &syn::File) -> Option<&syn::ExprMatch> {
    // The method on `PdfcerApp` — the parent dispatcher's shape.
    let method = file.items.iter().find_map(|item| {
        let syn::Item::Impl(imp) = item else {
            return None;
        };
        imp.items.iter().find_map(|member| {
            let syn::ImplItem::Fn(f) = member else {
                return None;
            };
            if f.sig.ident != DISPATCHER {
                return None;
            }
            f.block.stmts.iter().find_map(|stmt| match stmt {
                syn::Stmt::Expr(syn::Expr::Match(m), _) => Some(m),
                _ => None,
            })
        })
    });
    if method.is_some() {
        return method;
    }
    // …or the free function a file split out under R2 uses instead, because it
    // has no `impl` block to hang a method on. See [`SPLIT_DISPATCHER`].
    file.items.iter().find_map(|item| {
        let syn::Item::Fn(f) = item else {
            return None;
        };
        if f.sig.ident != SPLIT_DISPATCHER {
            return None;
        }
        f.block.stmts.iter().find_map(|stmt| match stmt {
            syn::Stmt::Expr(syn::Expr::Match(m), _) => Some(m),
            _ => None,
        })
    })
}

/// The name of the function a guard arm calls with the subject binding.
///
/// Every guard arm in the dispatcher has the shape
/// `id if <path>(id).is_some()`, so this peels the wrappers — a method call, a
/// negation, a parenthesis — until it finds a call whose single argument is
/// the subject, and returns that call's last path segment.
///
/// Requiring the argument to be **exactly the subject** is what stops it
/// answering for an unrelated call in a more complicated guard. A guard that
/// does something this cannot read is an error at the call site above, not a
/// silent `None` treated as "claims nothing".
fn guard_subject_fn(expr: &syn::Expr) -> Option<String> {
    match expr {
        syn::Expr::MethodCall(m) => guard_subject_fn(&m.receiver),
        syn::Expr::Unary(u) => guard_subject_fn(&u.expr),
        syn::Expr::Paren(p) => guard_subject_fn(&p.expr),
        syn::Expr::Binary(b) => guard_subject_fn(&b.left).or_else(|| guard_subject_fn(&b.right)),
        syn::Expr::Call(call) => {
            let syn::Expr::Path(func) = &*call.func else {
                return None;
            };
            if call.args.len() != 1 {
                return None;
            }
            let Some(syn::Expr::Path(arg)) = call.args.first() else {
                return None;
            };
            if !arg.path.is_ident(SUBJECT) {
                return None;
            }
            func.path.segments.last().map(|s| s.ident.to_string())
        }
        _ => None,
    }
}

/// Every `&'static str` constant declared at the top level of `src`.
///
/// Only `const NAME: &str = "value";` is recognised, which is the one shape an
/// arm pattern can name. A constant built from an expression is not a pattern
/// Rust would accept either, so nothing is lost by not resolving one.
pub(super) fn string_consts(src: &str) -> BTreeMap<String, String> {
    let Ok(file) = syn::parse_file(src) else {
        return BTreeMap::new();
    };
    file.items
        .iter()
        .filter_map(|item| {
            let syn::Item::Const(c) = item else {
                return None;
            };
            let syn::Expr::Lit(lit) = &*c.expr else {
                return None;
            };
            let syn::Lit::Str(s) = &lit.lit else {
                return None;
            };
            Some((c.ident.to_string(), s.value()))
        })
        .collect()
}

// ===========================================================================
// ASKING THE GUARDS, BY RUNNING THEM
// ===========================================================================

/// [`super::mapping`]'s header warns about one level down.
pub(super) use guards::{EVALUATED_GUARDS, guard_claiming};

/// Whether `id` is routed by some arm of `arms`.
///
/// ★ The guard half consults **both** sides: a guard function may claim the
/// id, *and* the dispatcher must actually have an arm that consults that
/// function. Checking only the first would keep vouching for a family whose
/// guard arm had been deleted — the mapping would still answer and four ribbon
/// buttons would silently stop working, which is precisely the shape
pub(super) fn is_routed(id: &str, arms: &Arms) -> bool {
    arms.literals.contains(id)
        || guard_claiming(id).is_some_and(|guard| arms.guards.contains(guard))
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_shell::CommandRegistry;

    /// The live registry, built the way `PdfcerApp` builds it.
    ///
    /// Against **this** rather than against [`super::super::catalog::all`]'s
    /// literal text, for the reason every test in [`super::super::mapping`]
    /// gives: it is the difference between asserting that the code agrees with
    /// itself and asserting that the control exists.
    fn registry() -> CommandRegistry {
        let mut reg = CommandRegistry::new();
        super::super::register(&mut reg);
        reg
    }

    /// The real dispatcher's arms, or a panic naming why they could not be
    /// read.
    fn dispatcher() -> Arms {
        let consts = string_consts(CONSTS_SRC);
        let mut arms = read_arms(DISPATCH_SRC, &consts).expect("the dispatcher must be readable");
        // ★ …and every file it has been split into. See `DISPATCH_PAGES_SRC`:
        // the parent no longer contains the Pages tab's ids anywhere a `syn`
        // walk of it can see, and this checker reported all six as unreachable
        // the moment they moved — correctly, and loudly, which is the whole
        // argument its header makes for parsing over grepping.
        let split =
            read_arms(DISPATCH_PAGES_SRC, &consts).expect("the pages dispatcher must be readable");
        let measure_arms = read_arms(DISPATCH_MEASURE_SRC, &consts)
            .expect("the measure dispatcher must be readable");
        for part in [split, measure_arms] {
            arms.literals.extend(part.literals);
            arms.guards.extend(part.guards);
            arms.catch_all |= part.catch_all;
        }
        arms
    }

    /// Every registered id the dispatcher does not route.
    fn unrouted() -> Vec<String> {
        let arms = dispatcher();
        registry()
            .iter()
            .map(|c| c.id.clone())
            .filter(|id| !is_routed(id, &arms))
            .collect()
    }

    // -----------------------------------------------------------------
    // THE CHECK
    // -----------------------------------------------------------------

    /// ★★ **Every registered command is reachable, or argued for.**
    ///
    /// The one assertion this module exists to make. A failure here means a
    /// control is drawn, enabled and pressable and produces
    /// `command-unimplemented` — which is what `file.save_copy` did for the
    /// whole life of the project, agreed with by five surfaces and contradicted
    /// by none.
    #[test]
    fn every_registered_command_is_routed_or_argued() {
        let argued: BTreeSet<&str> = SCAFFOLDED.iter().map(|(id, _)| *id).collect();
        let orphans: Vec<String> = unrouted()
            .into_iter()
            .filter(|id| !argued.contains(id.as_str()))
            .collect();
        assert!(
            orphans.is_empty(),
            "{} registered command(s) have no dispatch arm and no argued exemption: {}\n\
             \n\
             Each one is a control an operator can press that traces \
             `command-unimplemented` and does nothing. Write the arm in \
             `app/dispatch.rs`, or add the id to `SCAFFOLDED` with the REASON it \
             is deliberately inert — and if the honest reason is that it should \
             not be drawn yet, say so there rather than here.",
            orphans.len(),
            orphans.join(", ")
        );
    }

    /// **No entry on the allow-list has rotted.**
    ///
    /// Three ways an exemption goes stale, and all three are silent:
    ///
    /// * the command is **no longer registered** — the entry then excuses an id
    ///   nothing has, and reads as a live promise that the control exists;
    /// * the command **has been wired** — the entry then states a reason that
    ///   is false, and the next reader believes it;
    /// * the reason has decayed into a restatement of the id, which is the
    ///   thing the brief for this list specifically forbids.
    ///
    /// The middle one is the important one. Without it this list is a place to
    /// park a command permanently, and an allow-list nobody ever has to shorten
    /// is `DEFECTS.md` D5's *"hand-maintained list with a comment telling you to
    /// hand-maintain it"* wearing a different hat.
    #[test]
    fn no_scaffolded_entry_is_stale() {
        let reg = registry();
        let arms = dispatcher();
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        for (id, reason) in SCAFFOLDED {
            assert!(
                seen.insert(id),
                "`{id}` is on the allow-list twice; one command, one reason"
            );
            assert!(
                reg.get(id).is_some(),
                "`{id}` is on the allow-list and is not registered. \
                 An exemption for a command that does not exist excuses nothing \
                 and misleads the next reader; delete it, or, if the command was \
                 renamed, follow it."
            );
            assert!(
                !is_routed(id, &arms),
                "`{id}` is on the allow-list AND has a dispatch arm. \
                 The entry now states a reason that is false — the work landed. \
                 Delete the entry."
            );
            assert!(
                reason.len() >= 40 && !reason.contains(id),
                "`{id}`'s allow-list entry must carry the REASON, not the name. \
                 Cite the place the reason already lives — a registration's doc \
                 comment, `app::dispatch`'s own table, a `SALVAGE.md` class, or a \
                 `FEATURES.md` row — rather than writing a second wording that can \
                 drift from the first."
            );
        }
    }

    /// ★ **No literal arm names a command that is not registered.**
    ///
    /// The mirror of the check above, and it was not planned — it fell out of
    /// planting the first violation. `app::dispatch`'s `format.delete` arm
    /// states the rule it enforces: an arm for an unregistered id is *"an arm
    /// no token can ever reach — dead code wearing a design pattern, which is
    /// what the no-placeholders invariant forbids"*.
    ///
    /// The failure is quieter than the one this module was written for, and in
    /// one way nastier: an inert control at least *looks* wrong when pressed,
    /// while a dead arm reads as working code and will be maintained, reviewed
    /// and reasoned about by everyone who passes it.
    ///
    /// Guard arms are deliberately not checked here. They claim ids by
    /// computing over an enum, and [`super::mapping`]'s own tests already
    /// assert in both directions that every kind has a registered command.
    ///
    /// ★★★ **CONDITIONAL arms are exempt, and `UNREACHED_ARMS` is the wrong
    /// place for them.**
    ///
    /// An arm carrying `#[cfg(feature = "…")]` names a command that is
    /// registered in some builds and not in others, and this reader parses
    /// SOURCE, so it sees the arm in both. In a `--no-default-features` build
    /// it would report *"an arm no token can ever reach"* about an arm that is
    /// correct, present on purpose, and live in the default build.
    ///
    /// ⚠ The remedy this test's own failure message suggests — put it on
    /// `UNREACHED_ARMS` with a reason — is wrong here, and the difference is
    /// worth stating because the message will suggest it again. That list is
    /// for an arm **waiting for a command that does not exist yet**: a promise,
    /// which becomes false the day the command arrives, and which
    /// `every_allow_list_entry_still_earns_its_place` deletes when it does.
    /// A conditional arm is not waiting for anything. Listing it would make a
    /// working control look like an outstanding work item **for ever**, and the
    /// allow-list test would then fail in the default build because the arm is
    /// both listed and routed.
    ///
    /// ★ The exemption is narrow in the right way: it does not weaken the
    /// mirror direction. `every_registered_command_is_routed_or_argued` still
    /// sees a conditional arm in [`Arms::literals`], so a build that registers
    /// `file.sign` and forgets its arm still fails.
    #[test]
    fn no_literal_arm_names_an_unregistered_command() {
        let reg = registry();
        let tolerated: BTreeSet<&str> = UNREACHED_ARMS.iter().map(|(id, _)| *id).collect();
        let arms = dispatcher();
        let dead: Vec<&String> = arms
            .literals
            .iter()
            .filter(|id| {
                reg.get(id).is_none()
                    && !tolerated.contains(id.as_str())
                    && !arms.conditional.contains(*id)
            })
            .collect();
        assert!(
            dead.is_empty(),
            "{} dispatch arm(s) name a command that is not registered, so no token can \
             ever reach them: {dead:?}\n\
             \n\
             Delete the arm, or register the command it is waiting for — and if it is \
             deliberate, put it in `UNREACHED_ARMS` with the reason.",
            dead.len()
        );
        // …and the tolerated list itself must not rot: an entry that HAS been
        // registered since is an arm that now works, and the note excusing it
        // has become false.
        for (id, reason) in UNREACHED_ARMS {
            assert!(
                reg.get(id).is_none(),
                "`{id}` is listed as unreachable and is now registered; delete the entry"
            );
            assert!(reason.len() >= 40 && !reason.contains(id));
        }
    }

    /// **The allow-list and `PLANNED` describe different states and must not
    /// overlap.**
    ///
    /// `manifest::PLANNED` is for commands that are **not registered**: named
    /// by `RIBBON_IA.md`, absent from this build, drawn nowhere. Everything
    /// here is registered and drawn. An id in both lists would mean one of the
    /// two is wrong about whether the command exists, and the registry
    /// assertion in [`no_scaffolded_entry_is_stale`] already says which.
    #[test]
    fn no_scaffolded_command_is_also_planned() {
        let planned: BTreeSet<&str> = crate::shell::manifest::PLANNED
            .iter()
            .map(|(id, _)| *id)
            .collect();
        for (id, _) in SCAFFOLDED {
            assert!(
                !planned.contains(id),
                "`{id}` is both PLANNED (not registered) and SCAFFOLDED (registered, \
                 no arm). Those are different states and it cannot be in both."
            );
        }
    }

    /// ★ **The guards the checker runs are the guards the dispatcher has.**
    ///
    /// The seam between the two halves of this module, asserted as a set
    /// equality in both directions:
    ///
    /// * a **new** guard arm in `dispatch.rs` that [`guard_claiming`] cannot
    ///   run would otherwise report that arm's whole family unreachable, and
    ///   the reader would go looking in the wrong place;
    /// * a **deleted** guard arm would otherwise keep being vouched for by a
    ///   mapping function that still exists, which is a false pass on the
    ///   exact defect this module is about.
    #[test]
    fn the_guards_the_checker_evaluates_are_the_guards_the_dispatcher_has() {
        let in_source = dispatcher().guards;
        let evaluated: BTreeSet<String> =
            EVALUATED_GUARDS.iter().map(|s| (*s).to_owned()).collect();
        assert_eq!(
            in_source, evaluated,
            "`dispatch_command`'s guard arms and `guard_claiming` have diverged. \
             Add the missing function to `guard_claiming` and to `EVALUATED_GUARDS`, \
             or remove the one the dispatcher no longer consults."
        );
    }

    /// The dispatcher still has the catch-all that traces
    /// `command-unimplemented`.
    ///
    /// A sanity check on the **reading**, not on the code: a `match id` with no
    /// catch-all would not compile, so a run in which this is false means the
    /// reader found something other than the routing table and every other
    /// assertion in this module is measuring the wrong thing.
    #[test]
    fn the_reader_found_the_routing_table() {
        let arms = dispatcher();
        assert!(
            arms.catch_all,
            "no catch-all arm: this is not the routing table"
        );
        assert!(
            arms.literals.len() > 20,
            "only {} literal arm(s) were read; the reader has lost the routing table",
            arms.literals.len()
        );
    }

    // -----------------------------------------------------------------
    // THE SELF-TEST — the reader proves it bites
    // -----------------------------------------------------------------
    //
    // `check-file-size.sh`'s header states the rule these four keep: a gate
    // that has never been observed to fail is not evidence. Each fixture below
    // is a miniature dispatcher, and between them they plant every misreading
    // that would turn this check green while the defect shipped.

    /// A fixture dispatcher carrying one of each arm shape, plus every trap a
    /// text scan falls into.
    const CLEAN_FIXTURE: &str = r####"
impl PdfcerApp {
    pub(super) fn dispatch_command(&mut self, id: &str) {
        // "fx.in_a_comment" must not be read as an arm.
        match id {
            "fx.literal" => self.one(),
            "fx.left" | "fx.right" => self.pair(),
            crate::shell::commands::FX_CONST => self.constant(),
            id if crate::shell::commands::fx_for_command(id).is_some() => self.guarded(),
            other => {
                let _ = "fx.in_a_body";
                match other {
                    "fx.in_a_nested_match" => self.nested(),
                    _ => self.unimplemented(other),
                }
            }
        }
    }
}
"####;

    /// The constant table the fixture's path pattern resolves through.
    const FIXTURE_CONSTS: &str = r####"
pub const FX_CONST: &str = "fx.constant";
"####;

    fn fixture_arms(src: &str) -> Arms {
        read_arms(src, &string_consts(FIXTURE_CONSTS)).expect("the fixture must be readable")
    }

    /// **A. The reader finds every arm shape the dispatcher actually uses.**
    ///
    /// Without this, assertion B below could pass by finding nothing at all —
    /// which is the failure mode `run-all.sh`'s three-state model exists for,
    /// arriving inside a test instead of inside a script.
    #[test]
    fn the_reader_finds_every_arm_shape() {
        let arms = fixture_arms(CLEAN_FIXTURE);
        assert!(arms.literals.contains("fx.literal"), "a plain literal arm");
        assert!(
            arms.literals.contains("fx.left"),
            "the left of an alternation"
        );
        assert!(
            arms.literals.contains("fx.right"),
            "the right of an alternation"
        );
        assert!(
            arms.literals.contains("fx.constant"),
            "a path pattern, resolved through the constant table — this is how \
             `crate::shell::commands::FILE_RECENT` is reached"
        );
        assert!(
            arms.guards.contains("fx_for_command"),
            "the guard's function"
        );
        assert!(arms.catch_all, "the catch-all");
    }

    /// **B. A planted unreachable command is reported.**
    ///
    /// The fixture is `CLEAN_FIXTURE` with the `"fx.literal"` arm deleted and
    /// nothing else changed — which is exactly the shape of the real defect:
    /// the command stays registered, the ribbon keeps drawing it, and only the
    /// arm is gone.
    #[test]
    fn a_deleted_arm_is_reported_unreachable() {
        let planted = CLEAN_FIXTURE.replace(r#"            "fx.literal" => self.one(),"#, "");
        assert_ne!(
            planted, CLEAN_FIXTURE,
            "the plant must actually change the fixture"
        );

        let before = fixture_arms(CLEAN_FIXTURE);
        let after = fixture_arms(&planted);
        assert!(
            is_routed("fx.literal", &before),
            "with its arm present the command must be reachable, or assertion B \
             proves nothing"
        );
        assert!(
            !is_routed("fx.literal", &after),
            "the reader did not notice a deleted arm — it cannot detect its own \
             planted violation, and its verdict on the real dispatcher is worth \
             nothing"
        );
        // …and only that one moved.
        assert!(is_routed("fx.left", &after));
        assert!(is_routed("fx.constant", &after));
    }

    /// **C. Neither a comment nor a string in an arm's body is an arm.**
    ///
    /// The two false passes a grep produces. `"fx.in_a_comment"` is a quoted id
    /// inside a `//` line — the exact shape of the doc comments in
    /// `app::dispatch`, which quote ids constantly — and `"fx.in_a_body"` is a
    /// string literal in executable code. A check that counted either would go
    /// green over a command whose arm had been deleted while the prose about it
    /// stayed.
    #[test]
    fn the_reader_does_not_see_comments_or_body_strings() {
        let arms = fixture_arms(CLEAN_FIXTURE);
        assert!(
            !arms.literals.contains("fx.in_a_comment"),
            "a quoted id in a comment is not an arm"
        );
        assert!(
            !arms.literals.contains("fx.in_a_body"),
            "a string literal in an arm's body is not an arm"
        );
    }

    /// **D. A nested `match`'s arms are not the routing table's arms.**
    ///
    /// `dispatch_command` contains four nested `match` expressions inside arm
    /// bodies. A text scan cannot tell their arrows from the outer ones, and
    /// one of them has string literals on the right-hand side — so a grep would
    /// credit the dispatcher with routing ids it has never heard of. The tree
    /// walk visits the arms of one `match` and never descends into a body.
    #[test]
    fn the_reader_does_not_see_a_nested_match() {
        let arms = fixture_arms(CLEAN_FIXTURE);
        assert!(
            !arms.literals.contains("fx.in_a_nested_match"),
            "an arm of a `match` inside an arm's BODY routes nothing at the top \
             level, and crediting it is the false pass a grep produces"
        );
    }

    /// **E. An arm shape the reader does not understand is an error, not a
    /// shrug.**
    ///
    /// Failing closed is what keeps a future edit honest: a pattern nobody
    /// taught this reader about must stop the suite and be classified, rather
    /// than silently taking its ids out of the check.
    #[test]
    fn an_unreadable_arm_is_refused() {
        let odd = CLEAN_FIXTURE.replace(
            r#""fx.literal" => self.one(),"#,
            "crate::shell::commands::NOT_A_KNOWN_CONST => self.one(),",
        );
        let err = read_arms(&odd, &string_consts(FIXTURE_CONSTS))
            .expect_err("an unresolvable path pattern must be refused");
        assert!(
            err.contains("NOT_A_KNOWN_CONST"),
            "the error must name the arm: {err}"
        );
    }

    /// **F. A source with no dispatcher is refused rather than reported
    /// clean.**
    ///
    /// The "zero files scanned" failure, closed here as an `Err`. In the real
    /// module it cannot even arise: [`DISPATCH_SRC`] is an `include_str!` and a
    /// missing dispatcher file is a compile error.
    #[test]
    fn a_source_with_no_dispatcher_is_refused() {
        let err = read_arms("fn main() {}", &BTreeMap::new())
            .expect_err("a source with no dispatcher must not read as an empty routing table");
        assert!(
            err.contains(DISPATCHER),
            "the error must say what was missing: {err}"
        );
    }

    /// The constant reader resolves the shape an arm pattern can name.
    #[test]
    fn string_constants_are_resolved_from_their_defining_file() {
        let consts = string_consts(CONSTS_SRC);
        assert_eq!(
            consts.get("FILE_RECENT").map(String::as_str),
            Some(super::super::FILE_RECENT),
            "the reader must resolve `FILE_RECENT` to the same value Rust does, \
             or the one arm written as a constant reads as no arm at all"
        );
    }

    // -----------------------------------------------------------------
    // WHAT THE ALLOW-LIST SAYS ABOUT THE RIBBON
    // -----------------------------------------------------------------

    /// **★ How many drawn controls do nothing, and how many of those breach
    /// P3.**
    ///
    /// Not a rule — a **published number**, in the shape
    /// `the_icon_coverage_split_adds_up_to_the_registry` established: an
    /// arithmetic identity plus the two literals a reader actually consults, so
    /// that shortening this list is a visible act rather than a silent one.
    ///
    /// `RIBBON_IA.md` P3 says an unavailable capability renders **nothing**.
    /// Every entry marked `★ P3` in its reason is a control this module's
    /// author believes should not be drawn yet; removing one is a taxonomy
    /// decision and is the operator's. The count moving *down* is the project
    /// working.
    #[test]
    fn the_p3_tension_is_counted() {
        let total = SCAFFOLDED.len();
        let p3 = SCAFFOLDED
            .iter()
            .filter(|(_, reason)| reason.contains("\u{2605} P3"))
            .count();
        // ★ The literal, and it is the ONLY copy of this number.
        //
        // A failure message that sends a reader off to update prose is the
        // shape this project has corrected repeatedly — a gate runner's header,
        // `README.md`'s test count, `catalog.rs`'s icon split, the print
        // dialog's paper sentence. **When prose and a measurement disagree,
        // delete the prose's copy rather than correcting it**; where the prose
        // is already gone, stop telling people to update it.
        //
        // Failing here means the allow-list changed, and the two directions
        // mean opposite things. An entry ADDED is a command drawn and left
        // unwired. An entry REMOVED is work that landed.
        //
        // ★★★ THE LIST IS EMPTY, AND AN EMPTY LIST IS STILL A GATE. A new entry
        // cannot be added quietly: it has to be written in `register.rs` with a
        // reason, and this assertion is what makes adding one a visible act.
        //
        // ⚠ What this assertion CANNOT do, stated because the whole value of
        // the list depends on somebody knowing it: it asks whether an id has an
        // arm. An entry whose id has no arm and whose *reason* is nonsense is
        // indistinguishable from a correct one, and no mechanism can tell them
        // apart — a reason is prose. The header above carries the failure modes
        // a reader has to look for, and the practical rule that comes out of
        // them: **when you touch this list for any purpose, re-derive the
        // reason of the entry beside the one you came for**, and re-derive the
        // whole list on a schedule rather than on a collision.
        //
        // ⇒ The instrument that does not have this hole is a **driven check
        // that presses every registered id and fails on
        // `command-unimplemented`**. See `tools/ui-verify`.
        assert_eq!(
            total, 0,
            "the allow-list holds {total} entries — a command was scaffolded or wired"
        );
        assert_eq!(
            // Same rule as the total above: one copy of the number, here.
            //
            // A `★ P3` entry is a control drawn on the ribbon that does
            // nothing. The count exists to make the direction visible: it goes
            // down when such a control is wired, and it goes down when the
            // command is UNREGISTERED instead, which is R9's answer.
            //
            // ★ Why P3 is a cost and not a cosmetic. `RIBBON_IA.md` groups the
            // commands of Edit ▸ Content so the answer to *"what can I change
            // on this page?"* is one group; a group of three where one does
            // nothing reads as a broken program rather than as a missing
            // feature, which is precisely what P3 names.
            //
            // ★★ A caution for whoever reads a zero here. This census counts
            // `reason.contains("★ P3")`, i.e. **self-assigned prose**, and it
            // has never seen the worst breaches in this build: a command can be
            // enabled at application startup with no document, drawn on the
            // ribbon, and inert — the most severe form of P3 available — and
            // carry no mark at all. The census reports the state of the
            // ANNOTATIONS, not the state of the ribbon. Zero here means the
            // list is empty, and nothing more.
            p3,
            0,
            "{p3} entries are marked as breaching P3 by being drawn at all; the \
             report to the operator quotes the figure, so move both together"
        );
        assert!(p3 <= total, "the P3 subset must be a subset");
        // ★ …and the mirror list's length, pinned for the same reason and in
        // the same place. It is **zero**: an arm is tolerated only when its
        // verb has two live routes that are not the dispatcher, and none does.
        // A dead arm is still possible and still has to be argued — this
        // assertion is what makes adding one a visible act rather than a quiet
        // one, and what stops the header above going stale about it.
        assert_eq!(
            UNREACHED_ARMS.len(),
            0,
            "`UNREACHED_ARMS` is documented as empty. If an arm genuinely has \
             to be tolerated, add it there WITH its reason and move this number \
             — do not move this number alone."
        );
    }
}
