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

## Item notes

### `const REGION_ENGINE_NOTES`

A stable salt rather than the label, because the label carries the note
**count** and egui derives a `CollapsingHeader`'s open/closed state from its
id: without this, a report with three notes and a report with four would be
two different widgets, and the section would spring back open every time the
count changed.

### `fn only_the_checked_clean_carriers_are_called_clean`

The defect this whole module closes was not a wrong filter — it was **no
filter**, so a test that only asserted `CheckedClean` is picked up would
pass on a build that picked up everything. Both halves are asserted, and
the second half is the one with teeth: `DisclosedNotScrubbed` appearing
here would put a residual into a sentence that says pdfcer found no
trace of it, which is rule 1 broken in the worst available direction.

### `fn a_report_with_nothing_clean_says_nothing`

Without this, a `checked_clean_names` that returned every carrier would
still satisfy the test above's first assertion if the counts happened to
line up, and a reassuring sentence would appear on a report that had
earned nothing.

### `fn a_declined_match_and_a_real_failure_are_never_the_same_list`

This is the assertion with teeth in the whole module. The engine keeps
`FoundNotScrubbed` — *told not to* — apart from `DisclosedNotScrubbed`
— *tried and could not* — and warns that collapsing them makes a
deliberate scope look like a failure and a real failure look like a
preference. Both directions are wrong and both are one `==` away, so
both are asserted: a declined match must not enter the blocking
residual section, and a real failure must not be excused as a setting.

### `fn text_report`

Built by assignment rather than a struct literal because
[`RedactionReport`] is `#[non_exhaustive]`, the same reason
[`report_with`] above is shaped this way.

### `fn a_line_is_drawn_whatever_the_report_says`

The property the whole block rests on. A state that produced no line
would paint blank space under the heading, and blank space reads as
*"nothing to report"* — true in one of these three cases and false in
the other two.

### `fn codes_removed_without_any_text_reported_are_disclosed`

`glyphs_removed` is the only thing separating this from the case above,
and getting that branch backwards would tell the operator a region full
of text contained none — the worst sentence this dialog could print.

### `fn a_long_entry_is_cut_on_a_char_boundary_not_a_byte_one`

Written with a two-byte character on purpose: the strings arriving here
are whatever the document's fonts decoded to, and a byte cut inside a
multi-byte sequence panics the dialog rather than truncating it. The
character count is the assertion; the panic is the failure this test
exists to catch.

### `fn each_branch_reports_a_different_state_to_the_trace`

`state` is what a driven check reads to tell *"the block found text"*
from *"the block found none"*, and two branches sharing a token would
make that check pass on either — an assertion both outcomes satisfy.

### `fn the_traced_character_count_measures_what_goes_not_what_is_shown`

It is the only field of the trace a driven check can compare against
something it knows — the length of the string its own fixture puts on
the page — so counting bytes would make every non-ASCII document
disagree with the check by the length of its encoding, and counting the
drawn lines would make a capped report understate what is going.
