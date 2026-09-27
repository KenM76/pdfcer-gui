# `pdfcer-gui/text/redact/tests`

## Item notes

### `fn nothing_on_the_marking_surface_claims_a_removal`

Rule 1 of the module header, asserted rather than trusted. The failure
this catches is a copy pass tightening *"marked for redaction"* into
*"redacted"* — which reads better, is shorter, and is the exact
misunderstanding that ships marked documents.

### `fn only_the_verification_line_and_the_clean_outcome_say_verified`

Rule 2, and it is a test rather than a doc comment because the word is
the single most valuable one on this surface: it is the difference
between a report and a claim, and it costs nothing to sprinkle it
somewhere it is not earned.

### `fn the_staged_outcome_names_its_residuals_and_says_nothing_has_happened_yet`

**REWRITTEN 2026-09-05.** Its predecessor asserted that both forms
said *"Nothing is on disk yet"*, which was the deferred route's own hazard
under `Pass 250.1`: the content had been removed from the document, the
file had not been written, and an operator who handed over the original had
redacted nothing.

Under `Pass 250.2` that sentence is no longer sufficient, because a second
thing is now also true and is the more surprising of the two: **nothing has
been removed either.** The page is unchanged. So both forms must say both
facts, and this test enumerates them rather than sampling — a form that
said only "nothing is on disk" would leave him believing the document in
front of him was already redacted, which is the marked-file failure this
whole feature exists to prevent.

### `fn the_staged_save_outcome_says_the_window_is_stale_and_the_removal_is_still_armed`

1. **the file has the content removed** — the receipt, and the only one of
   the three the operator would guess;
2. **the window is stale** — the session was never mutated, so the canvas
   goes on drawing the marks and the content while the file holds neither.
   Without this he concludes the save did not work, or worse, that a page
   still showing a name is a page whose name was removed;
3. **the removal is still armed** — `save_applying_redaction` takes
   `&self`, so the next save does it again and the ordinary modes stay
   refused. *"I saved it, so it is done"* is the assumption that would
   otherwise stand.

### `fn the_cancel_sentence_says_the_marks_are_still_there`

The one misreading available at this control is *"never mind, that is
dealt with"* — and it is not: the marks are still on the document and the
content is still in the file, which is exactly what the operator asked for
and exactly what he must not be allowed to forget.

### `fn no_post_apply_sentence_mentions_undo_as_a_way_back`

The distinction that replaced it: undo reaches the **arming**, and never
reaches the **removal**. So this sweep's membership list is the
load-bearing half, and it is drawn on exactly that line:

* **In** — every sentence about content that is gone from a file, and
  every sentence about what applying will permanently do.
* **Out** — the marking strings (taking a mark off genuinely is undoable),
  and the staging strings, which are about a state undo *does* reach and
  which have their own test above.

### `fn the_staging_copy_says_undo_still_works`

Before a save, undo genuinely reaches everything: the marks, the edits
around them, and — through the *call it off* control — the arming itself.
A catalog that stayed silent about that out of habit would leave the
operator believing a staged removal is as irreversible as a written one,
which is the wrong lesson in the *cautious* direction and costs him the
whole capability `Pass 250.2` bought.

So this test is a **positive** one: at least one string the operator reads
while choosing the deferred destination must tell him undo still works.

### `fn the_two_outcomes_read_differently_and_the_residual_one_names_its_count`

Rule 1 mechanically: the residual form must name the leftover count in
the same sentence as the success, and must not be reachable by softening
the clean form.

### `fn the_suggested_name_differs_from_the_original`

The suffix is the mechanism; `crate::dialogs::redact` asserts the
resulting path. This asserts the half that lives in the catalog, in the
shape `crate::text::ocr`'s equivalent test established.

### `fn each_named_refusal_says_something_different`

The entry read `reason: "hybrid".to_owned()`, and `refusal_message`
selected its sentence with `reason.contains("hybrid-reference")`. The
literal `"hybrid"` does not contain `"hybrid-reference"`, so this set
exercised the GENERIC arm and the specific one was never called by any
test — while the fixture's own word made it read as though it were.

That is the shape worth remembering: a payload chosen because it was
short and evocative, in a test whose subject is which sentence comes
out. The word `hybrid` was doing the reader's convincing and none of
the assertion's work.

The selector is a `bool` on the variant now, so the two cases are two
entries and cannot collapse into one by accident of wording. The
distinctness assertion below is what proves the split earns its keep.

### `fn no_residual_line_shows_the_operator_an_engine_key`

The fourth wording rule, asserted over the whole vocabulary rather than over
a sample. Until 2026-09-09 the sentence the operator read was literally
*"⚠ struct_tree: present in this document…"* — an identifier the engine
documents as being *"for the carrier"*, i.e. for a program, printed into a
report written for a person.

**The assertion is "no underscore", not "does not contain the key", and
the difference is a measurement.** The blunt substring form was written
first and went red on `thumbnails`, whose English name is *"the page
thumbnails stored in the file"* — the key and the operator's own word for
the thing are the same word, and there is nothing wrong with that sentence.
The defect was never "a carrier's name appears in its sentence"; it was
**a machine identifier appearing in prose**, and what makes an identifier
visible as one is the underscore. Twelve of the fourteen keys are
`snake_case`, so this catches every one of them and every future one,
without forcing a perfectly good English word out of a sentence to satisfy
a test.

The second assertion covers the two-word-free remainder from the other
side: an identity mapping — the failure mode where somebody deletes an arm
and the `other => other` fallback silently takes over — leaves the sentence
*equal* to the key, which no translated name ever is.

### `fn a_carrier_this_shell_has_never_heard_of_is_still_named`

The open-vocabulary case, and the one an over-tidy edit would break: the
obvious "fix" for the test above is to return an empty string for an unknown
key, which silently drops a residual the engine went to the trouble of
reporting. On this surface a dropped disclosure is the worst available
outcome, so an unknown carrier reads awkwardly and is *there*.

`CarrierStatus::carrier` is not a closed set and the engine may add one at
any release — `residual_sweep` itself arrived that way — so this is a real
state, not a hypothetical.

### `fn the_whole_file_sweep_does_not_get_the_generic_carrier_sentence`

Every other carrier is a *place that holds content*, and the generic
sentence says so. `residual_sweep` is not a place: it is the engine's search
of every other object in the file, and it reports `DisclosedNotScrubbed`
when that **search** could not finish. Telling the operator that a search
"is present in this document and pdfcer cannot scrub it" is not jargon — it
is a false sentence, in the residual list, on the one surface where rule 1
forbids a comfortable one.

The last assertion is the load-bearing half. Without it the test would
pass on a build where **both** sentences had been rewritten into the sweep's
wording, which discloses nothing about the other twelve carriers.

### `fn the_sweep_sentence_points_somewhere_that_exists`

A promise kept across two modules: [`super::residual_sweep_line`] tells the
operator that pdfcer's own notes *"at the foot of this report"* say which
objects were left, and `dialogs::redact::disclosures::engine_notes` is what
puts them there. Before 2026-09-09 `RedactionReport::notes` was read by
nothing in this crate, so a sentence like this one would have pointed at an
empty part of the screen.

### `fn the_drawing_instruction_clause_is_conditional`

The engine counts `residual_content_streams_blanked` apart from the sweep's
total for one stated reason: it is the only member of the sweep that edits
**drawing instructions**. Everything else removes a metadata string nobody
looks at; this changes what a page would paint. A report that said "and 0
drawing-instruction streams" on every ordinary redaction would train the
operator to skip the clause on the day it reads 3.

### `fn the_clean_census_number_matches_its_list`

The number and the list come from one argument, so they cannot disagree —
this pins that both are actually derived from it, which a `format!` that
hard-coded either would not be.

### `fn the_clean_census_claims_the_places_and_not_the_document`

Rule 1's hardest case: this is the only sentence in the report that exists
to reassure, and the report's whole purpose is to prevent a comfortable one.
It is safe because of what it claims — pdfcer *checked* these places and
found nothing *in them* — and this test pins the scope that keeps it narrow.
A rewrite to "no trace of it anywhere in the file", or to "the document is
clean", would be the defect.

### `fn the_declined_matches_are_named_and_said_to_survive`

This is rule 1 at its sharpest. Every other sentence in the report body
describes something pdfcer is about to remove; this one describes text it
found, can remove, and will not. A copy pass that softened it into "some
metadata was left unchanged" would be technically true and would cost the
operator the one fact he needs — that the marked words are still readable in
the file he is about to keep.

### `fn the_declined_matches_read_as_a_setting_and_not_as_a_fault`

The engine keeps `FoundNotScrubbed` and `DisclosedNotScrubbed` apart because
collapsing them would make a deliberate scope look like a failure and a real
failure look like a preference. That distinction only survives into the
product if the two sentences read differently — so this one must not borrow
the vocabulary of the other, and must point at the setting a failure could
not point at.
