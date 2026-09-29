# `redact::disclosures` — the parts of the engine's report that
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

### `fn sweep`

Placed immediately after `info_scrubbed` in the body, because the two answer
the same question about two different halves of the file: that one says what
the trailer's document-information dictionary carried, this one says what
everything else carried. The engine counts them separately for exactly that
reason, and its own words on the field are *"A single total would hide the
fact that the second number is the one nobody expected to be non-zero."*

Drawn only when the sweep actually edited something. `objects` is the
engine's total — dictionaries scrubbed, plus metadata packets blanked, plus
content streams blanked — so it is zero exactly when the sweep changed
nothing, and a "0 objects will be scrubbed" line on an ordinary redaction
would be noise in a report whose warnings have to keep their force.

### `fn checked_clean`

Drawn with the proof, under [`crate::text::redact::verified_line`], because
it is the same kind of statement: evidence that a check happened. It is
muted rather than plain, one step below the verification line, because the
verification line is this shell's own sweep of the finished bytes and this
is the engine's sweep of the carriers — related, and not equally strong.

The whole argument for its existence is the engine's, on the variant:

> *"a shell that tells an operator 'nothing to do' when the truth is
> 'checked, clean' has taken away the one thing that distinguishes a
> diligence sweep from a no-op."*

The derivation is [`checked_clean_names`], separated from the painting so a
test can observe it: a `&mut Ui` is not an oracle, and a check that had to
read pixels to prove a filter reads the right variant would be measuring the
wrong thing.

### `fn left_by_choice`

[`CarrierAction::FoundNotScrubbed`] — the carrier holds a copy of the marked
text, pdfcer can remove it, and the operator's reach setting says not to.
This block is what makes offering a narrow reach honest: the setting changes
what pdfcer may *change*, never what it *finds* or *reports*, so the operator
sees the full census either way and chooses with it in front of him.

**Notice weight, and the choice of colour is the substance of the block.**
Not muted like the clean census, because this is live text surviving into
the saved file rather than evidence that a check ran. Not
`palette.danger` like the residual section, because nothing failed — the
engine keeps this verdict apart from `DisclosedNotScrubbed` precisely so a
deliberate scope does not read as a fault, and painting both red would
collapse in the product the distinction the engine maintains in the report.

Drawn **above** the residual section so the danger colour stays last and
closest to the confirm control.

It gates nothing. It cannot enter [`crate::redact::residual_count`], cannot
be acknowledged, and cannot block the confirm control — an operator who set
a narrow reach and is then refused the save he asked for has been given a
setting that does not work.

### `fn left_by_choice_names`

An `==` filter for the same reason [`checked_clean_names`] is one:
`CarrierAction` is `#[non_exhaustive]`, a `match` here could only be written
with a catch-all, and a catch-all is how a new verdict becomes invisible.
The tripwire for a new variant is `tools/gates/check-engine-api-drift.sh`,
not this file.

### `fn engine_notes`

These were being discarded, and one of them is load-bearing: when the
residual sweep cannot scrub a stream object, the **object numbers** exist
only in a note. [`crate::text::redact::residual_sweep_line`] tells the
operator to look here for them, which is a promise this function keeps.

**Collapsed by default**, and the reason is `OPERATOR_REQUESTS.md` O160 —
his report that this dialog's warnings had become something to click past.
These notes are the engine's prose: they cite ISO 32000-1 by table number
and there can be a dozen on one sheet. Open by default they would bury the
residual section under spec citations, which is the same failure in a new
place. Closed, they cost one line and lose nothing.

**Not styled as a warning**, even though some of them are about residuals.
The residuals that matter are already lifted out into the danger-coloured
section above by their own derivations; painting this section red as well
would double-count them and dilute the colour that means "read this".

### `struct RemovedText`

The state is carried out of the derivation rather than re-derived by the
painter, so the trace reports what this function chose rather than what a
second reading of the same report would choose. Those are the same value
only while the two readings agree, and a trace that can disagree with the
screen is worse than none.

### `fn removed_text_lines`

**It always returns at least one line.** An empty return would collapse
three different states — text found, no text present, and codes removed with
no text reported — into one blank area, which reads as *"pdfcer has nothing
to say about this"* in all three. The first two are findings and the third
is a disclosure; none of them is silence.

Neither cap cuts silently: [`t::MAX_ENTRIES`] is followed by
[`t::removed_text_more`] naming the remainder, and [`t::MAX_CHARS`] is
applied inside [`t::removed_text_entry`], which says how many characters of
that one region it did not print.
