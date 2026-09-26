# `dialogs::redact::disclosures` — the parts of the engine's report that
reached nobody


## Why a file of its own, and where the seam is

The blocks below plus their arguments would put `dialogs/redact.rs` over
rule R2's ceiling, so a split is compulsory; what follows is why *this*
split rather than a mechanical one.

The seam is **"a disclosure derived from the engine's report, drawn into the
report body, that gates nothing."** Every block here reads
[`RedactionReport`] and paints; none of them contributes to
[`super::residual_lines`], none is acknowledgeable, none can block the
confirm control. That is a genuinely different job from the rest of the
dialog, which is a transaction: a destination choice, two acknowledgements
and an irreversible verb. Keeping the two apart means an edit to the wording
of a disclosure cannot reach the transaction by accident.

**The derivations live here too**, and that is deliberate but conditional.
`crate::redact` is where a derivation belongs when two surfaces must agree
on it — [`crate::redact::residual_count`]'s doc comment argues that at length
and it is right. Nothing else consumes these; the moment something does,
they move down into `crate::redact` and this file keeps only the painting.
Putting them there today would have been a layer for one caller.

## Rule 1 and the reassuring sentence

[`crate::text::redact`]'s rule 1 — *never say "removed" without
qualification when anything was left* — is the reason the census in
[`checked_clean`] is safe to draw at all. It is not an "all clear". It is
drawn from `CheckedClean`, the residual section is drawn from
`DisclosedNotScrubbed`, and the two are different verdicts on different
carriers: there is no report in which showing one suppresses the other.
A future edit that made this block conditional on the residual list being
empty would break that property, and would be the defect this note exists to
prevent.

## What guards the wiring, and it is not a test

Everything below paints into a `&mut Ui`, which is not an oracle, so no unit
test in this crate can observe that any of it is **drawn**. The derivations
are tested — [`checked_clean_names`] and [`left_by_choice_names`] are
separated out precisely so they can be — but a build where every call site in
[`super::RedactDialog::report`] had been deleted would pass every test in the
workspace, which is the exact shape of the defect this module closes.


⚠ A driven assertion is still owed regardless: dead-code analysis proves the
call exists, not that the label reaches a pixel the operator can read. That
is the same gap the project's own note about panels shipping unreachable was
written for.

Section 5 is the only block that closes it. It publishes its rect through
[`crate::diag::ui_rect_visible`], so a driven check can tell *drawn and
readable* from *drawn behind the fold* from *not drawn at all*; the other
four publish nothing and the gap is open for them. The asymmetry is
deliberate rather than unfinished: section 5 is the one block whose absence
costs content rather than context.
