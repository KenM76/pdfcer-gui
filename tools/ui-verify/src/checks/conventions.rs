//! **How a check in this suite is written, and what makes one admissible.**
//!
//! Doc-only: the rules a new check is held to, and the acceptance criterion
//! the harness itself answers to. It carries no code because the rules are
//! not enforceable by the compiler - they are enforced by whoever reviews the
//! check, and they live beside the roster in [`super`] so that reviewer finds
//! them without being told where to look.
//!
//! ## The acceptance criterion for the harness
//!
//! > The three assertions **fail** against the old GUI (proving they detect
//! > the real defects) and **pass** against the new one.
//! > — `PROJECT_PLAN.md` §4, stage S1
//!
//! That is the reason [`crate::profile::PDFCER_LEGACY`] exists. A check suite
//! that has only ever been seen to pass is not evidence of anything: it is
//! indistinguishable from a suite that cannot fail. This is the same argument
//! that put a `--self-test` in `tools/gates/check-ui-strings.sh`, and it comes
//! from the same recorded incident — a deliberately planted violation that a
//! gate failed to detect, briefly making it look as though the fix had
//! produced a gate that could only pass.
//!
//! ## Writing a new check
//!
//! Four rules, each of which exists because breaking it produced a real
//! problem in this codebase:
//!
//! 1. **Say what you are about to do, in a note, before you do it.** The notes
//!    are what make a SKIP diagnosable and a PASS believable.
//! 2. **Only ever write [`crate::coords::DocPoint`] or
//!    [`crate::geom::FracRect`] literals.** Never a screen coordinate, never a
//!    window coordinate. See [`crate::coords`].
//! 3. **Establish the precondition explicitly, and SKIP on it.** "The click
//!    selected something" must be *asserted* before "Delete removed it" can be
//!    a failure rather than a mystery.
//! 4. **Never treat an absence as evidence unless you have shown the thing
//!    that would have produced it was working.** The distinction is drawn in
//!    [`crate::report`] and applied in [`crate::checks::delete_key`].
//! 5. **A SKIP reason names the component that is actually blocked, and gets
//!    re-audited whenever the application gains a capability.** This one was
//!    earned at S2. `ribbon_group_captions_legible` spent a stage reporting
//!    *"the trace declared no `ui-rect` regions"* about a binary that declares
//!    three of them on every frame — because nothing in this crate parsed the
//!    event, so the reason described the harness's own blindness as though it
//!    were the application's silence. A reader following it would have gone to
//!    `diag.rs`, which was finished, and found no defect to fix.
//!
//!    The rule that prevents a repeat is mechanical: **a reason may only
//!    assert what the check actually looked at.** [`crate::checks::legibility::resolve_set`]
//!    takes the trace evidence as an argument and builds its reason from it,
//!    with a distinct sentence for "nothing was consulted", "the application
//!    said nothing" and "the application said these things and none of them is
//!    what I need" — because those three send a reader to three different
//!    files.
//!
//! ## Where a check's evidence comes from, in preference order
//!
//! Both apply to every new check, and both say the same thing in two domains:
//! **prefer the evidence the application produced this run.**
//!
//! | Domain | Preferred | Fallback |
//! |---|---|---|
//! | *Where* to measure | a `ui-rect` the application declared this frame | a calibrated fraction in [`crate::profile`] |
//! | *Whether* state changed | a count of the thing itself (`objects n=`) | the event for the verb that should have changed it |
//!
//! The first pair is argued in [`crate::checks::legibility`]; the second in [`crate::checks::delete_key`].
//! In both cases the fallback is kept, because a dated screenshot cannot
//! declare its regions and the old binary cannot count its objects — and in
//! both cases the check says in its own output which one it used.
