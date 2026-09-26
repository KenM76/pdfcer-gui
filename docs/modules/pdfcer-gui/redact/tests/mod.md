# `pdfcer-gui/redact/tests/mod`

## Item notes

### `fn scratch`

`std::env::temp_dir` rather than a path in the repository, exactly as
`crate::app::save`'s tests do it: a test that writes beside the fixtures
leaves a file somebody eventually commits.

### `fn applied_redaction_leaves_no_recoverable_trace_in_the_saved_bytes`

After apply-and-save through [`prepare_redaction_apply`], the redacted
text must not be recoverable from the saved bytes by any means pdfcer
itself offers. Three independent measures, because a single one could be
satisfied by a build that merely hid the text:

1. **`extract-text`** — the very tool `pdfcer extract-text` and this
   shell's Copy-text both use — finds nothing;
2. **every decoded stream** (content streams, XObjects, object-stream
   containers, metadata) contains no occurrence;
3. **the raw file bytes** contain no occurrence.

And the negative control: `KEEPTHIS`, which was never marked, is still
extractable. Without it, a build that emitted an empty page would pass
all three assertions above while destroying the document.

### `fn a_mark_that_was_never_saved_is_still_applied`

The un-saved-mark trap §1.2 names: passing `session.document()` to
`apply_redactions` would apply nothing and report success. The assertion
that makes it bite is `marks_applied` — a build with that bug produces
`NothingToApply` or a zero count, never a removal.

### `fn the_output_is_one_revision_with_no_prior_revision_to_walk_back_to`

A `/Prev` in the trailer would mean a prior revision is reachable in the
saved file, which for a redaction is the un-redacted content one hop
away — R35's whole point, and the reason §1.1 forbids the incremental
writer this shell otherwise uses for every save.

### `fn writing_over_the_source_replaces_it_and_leaves_no_temporary`

It read `a_region_over_an_image_refuses_the_whole_apply` until
2026-09-03 and asserted that the engine declined the entire document —
which was true, was the operator's headline complaint
(`OPERATOR_REQUESTS.md` O103, *"every time I've tried the redact feature
it tells me it can't"*), and stopped being true with `pdfcer-core`
v0.26.0 the same day.

**Writing over the source replaces it, and leaves no temporary
behind.**


Three assertions, and the third is the one a `write`-based build passes
by accident and an unclean temp-file build fails:

1. the target holds the **redacted** bytes afterwards, not the original;
2. the original content is **gone** from it;
3. no `.pdfcer-tmp` file is left beside it — and that file would contain
   a complete redacted document, which is the last kind of stray
   artefact this feature should scatter around somebody's job folder.

### `fn a_real_drawing_sheet_with_an_embedded_font_is_applied_rather_than_refused`

# Why this exists, and why every other test in this module missed it

Everything above runs on [`assemble`]d fixtures: a handful of objects,
uncompressed streams, `/Helvetica`, no embedded font, no compression, no
object streams. Those fixtures are right for what they assert — *"every
byte in this file is one the suite put there"* — and they share one
property that turned out to matter more than any of them: **there is
nothing in them for a coincidence to hide in.**

`tools/ui-verify`'s `checks::redaction` — the one end-to-end check that
drives the real binary — generates its own fixture too, and its header
says what it is: *"Two pages, uncompressed"*, drawing two ASCII strings
with a Base-14 font. So the redaction feature had a unit suite and a
driven check and **neither had ever seen a document with an embedded
font in it**, which is to say neither had ever seen a document a person
would open.

The result was a feature that passed every test and refused every real
file. `fixtures/a1-titleblock.pdf` — a drawing sheet in this repository,
with JetBrains Mono embedded — refused with
`VerificationFailed { survivors: [" construction"] }` because the font's
`name` table describes its own ligatures as *"Classic construction"*.

⇒ **The fixture that exercises the feature and the fixture that
resembles the operator's work are not the same fixture, and a suite
needs both.** This is the second.

# What it asserts, and the negative control

1. the apply **is not refused** — the operator's complaint, as a
   boolean;
2. characters were actually **removed** (`glyphs_removed > 0`), so a
   build that "passed" by doing nothing fails;
3. the residual **is** disclosed and names
   [`ResidualSite::FontProgram`] — not refusing must not mean not
   telling;
4. the file is **written** once the acknowledgement is given, which is
   the operator's actual demand: *"still make the changes it could"*;
5. the negative control: `FOUNDATION`, a word on the same sheet that
   was never marked, is **still extractable** from the written file. A
   build that emptied the page would satisfy 1–4 and fail here.

### `fn a_region_over_an_image_destroys_the_samples_and_says_so`

What it asserts now is the pair the operator cares about: the apply
SUCCEEDS, and the report says the image was dealt with rather than
quietly stepped over.

### `fn an_unacknowledged_residual_refuses_the_write`

§2.3, asserted rather than described. The dialog greys its confirm
control until the box is ticked, and **a greyed control is a drawing
decision, not a mechanism** — this is the mechanism. The failure it
catches is the one that matters most: a partially-redacted file handed
over as a complete one.

The fixture builds the residual by hand rather than hunting for a
document that happens to produce one, because the point under test is
the *gate*, not the classification (which [`proof`]'s own tests cover).

### `fn a_clean_report_writes_with_the_acknowledgement_withheld`

The other direction of the gate, and the one that would make the feature
unusable if it were wrong: a redaction with nothing to disclose must not
demand a tick nobody can give.

### `fn a_write_that_cannot_happen_is_a_named_refusal`

`crate::app::save`'s equivalent test, for the writer that matters more:
a redaction the operator believes landed, at a path that does not exist,
is a file they will look for and not find at the moment they need it.

### `fn the_debug_impl_reports_a_length_rather_than_the_bytes`

§2.1's hand-written [`std::fmt::Debug`], pinned. The failure it prevents
is silent and total: a `#[derive(Debug)]` restored during a routine
tidy-up would put a whole redacted PDF into any trace, panic or test
failure that formatted this value — a log file nobody thinks of as
containing document content.

### `fn both_ordinary_save_modes_are_refused_by_name_while_staged`

This is `request_apply_redactions_into_the_session.md` §4.1 — the property
the request marked and asked the engine to enforce by refusal. `Pass
250.1` declined to refuse, on the argument that its collapse removed the
hazard at the root. `Pass 250.2` cannot make that argument, because it
preserves the un-redacted session on purpose, so it ships the refusal — and
this test is why that refusal is believed rather than quoted.

**The leak is measured as well as the refusal.** It would be possible for
the engine to refuse `to_incremental_bytes` and not `to_full_bytes`, or to
refuse both and for this shell to be reaching for some third serialiser, so
the test does not stop at the error type: it asserts that **no bytes came
back at all** from either mode, which is the only form of "cannot leak"
that does not depend on reading the engine's source.

The positive control is the fixture itself: the same session's staged save
path DOES produce bytes, in `the_staged_save_removes_the_text_and_leaves_no_prior_revision`
below. Without that, this test would pass on a build in which the session
could not be serialised by any means whatsoever.

### `fn the_staged_save_removes_the_text_and_leaves_no_prior_revision`

The other half of the headline, and the positive control for it: the save
that IS permitted while a removal is armed must actually produce bytes, must
not contain the removed text, and must be a single revision.

The `/Prev` assertion is the sharp one. A staged save is a full rewrite by
construction (`crate::redact::save_applying_pending`), so a `/Prev` in the
trailer would mean the un-redacted document is reachable one `startxref` hop
away in a file this shell has told the operator is redacted — R35's whole
point.

The scan is over the **raw bytes** rather than over decoded streams, and
on this fixture that is legitimate: the content stream is uncompressed and
the font is Base-14, so there is no encoding under which the text could be
present-but-unfindable, and no font program in which it could be
present-but-innocent. `a_real_drawing_survives_the_staged_route` answers the
compressed, embedded-font case.

### `fn staging_preserves_the_undo_log`

The route this replaced cleared the log outright, and the operator accepted
that with a *"for now"* attached. This is the assertion that the *for now*
is over, and it is deliberately three separate claims because a build that
had silently reverted to the collapsing verb would fail a different one of
them depending on how it reverted:

1. **the depth is unchanged** — the log is not merely non-empty, it is the
   same size it was before the staging;
2. **an undo actually works** and takes the marks back off, which is the
   operator-visible form of the same claim;
3. **`has_applied_redaction()` is false** — this shell never collapses, and
   if that verb ever answers true here, something is calling the engine's
   other apply and `redact::sealed`'s count has been wrong.

### `fn a_staged_removal_can_be_called_off_and_saving_works_again`

*A stageable operation that cannot be un-staged is a trap*, asserted. The
trap has teeth here rather than being a matter of taste: while a removal is
armed the engine refuses both ordinary save modes, so an operator who
changed his mind and had no way to say so could not save his document at
all.

The second assertion is the one that makes the first mean something. A
cancel that cleared the flag and left the session unable to serialise would
satisfy *"the flag is off"* and leave him exactly where he was.

And the third: the marks survive. Un-arming is not un-marking, and a
cancel that silently removed the operator's marks would destroy work while
claiming to be the safe button.

### `fn a_staged_document_with_no_marks_left_can_still_be_called_off`

The sequence, and every step of it is something a reasonable person does:

1. mark, then *Review & apply* ▸ *this document* — the removal is armed;
2. change his mind about the marks and take them off in the panel, one
   Remove at a time (not an undo — an ordinary edit);
3. press `Ctrl+S`.

The save is refused, because the armed removal has nothing to remove and
the engine refuses both ordinary modes while it stands. So he goes back to
*Review & apply* to call the removal off — **and if the pipeline asked the
mark census before the pending flag, he would be told `NothingToApply`,
which the dialog draws as a refusal with no control on it.** The document
would then be unsaveable by every route in the program, with the one button
that frees him behind a phase he cannot reach.

Two things keep it open and both are asserted here: `AlreadyStaged` is
answered ahead of the census, and the command that opens the window is
`enabled_when("doc.pages")` rather than on a marks predicate — so the
ribbon control stays live on a document with none.

### `fn a_second_staging_and_a_second_report_are_both_refused_by_name`

Two reachable causes and one refusal: the operator opens *Review & apply* a
second time on a document he has already staged, or a second `Stage` action
arrives before the first frame after the first one.

The refusal has to be **this shell's**, not the engine's, and that is the
assertion. `EditSession::apply_redactions_deferred` would happily run a
second preview and set an already-set flag; what makes the second open
legible is `prepare_redaction_apply` naming the state, because otherwise the
engine's `to_full_bytes` refusal would surface as *"this document cannot be
rewritten in full"* — a true sentence about the wrong subject, at the one
surface where a wrong diagnosis costs most.

### `fn a_refused_staging_leaves_the_session_untouched`

The engine's own guarantee — *"on any error the pending flag is NOT set"* —
asserted from this side rather than quoted. `NothingToApply` is the one
refusal a test can produce without breaking the engine, and it is also the
one this shell can actually reach (a mark undone in the frame between the
panel enabling its button and the action running).

### `fn the_staging_and_the_undo_log_both_survive_the_save`

`save_applying_redaction` takes `&self`. It does not mutate the session and
it does not clear the flag, so a saved document is still armed — which is
the fact `crate::text::redact::saved_applying_redaction` tells the operator
and which nothing else on screen would.

The reason it is a test rather than a sentence is the assumption it
contradicts: *"I saved it, so it is done."* A build that cleared the flag on
save would look correct for one save and then quietly write the un-redacted
document on the second one — with no refusal, because the flag would be off.

Undo across the save is asserted in the same test, because the two facts
have the same cause (`&self`) and a build that broke one would break both.

### `fn a_real_drawing_survives_the_staged_route`

Every other fixture in this file is uncompressed with a Base-14 font, which
is a document with nothing for a coincidence to hide in. This one is a CAD
title block with compressed content streams and an embedded, subsetted
font — and it is the file whose font `name` table describes its ligatures as
*"Classic construction"*, which is what made the shell refuse every real
redaction until the proof was corrected on 2026-09-04.

It asserts through pdfcer's own text extraction as well as through the raw
bytes, because on a compressed document the raw scan alone would pass on a
build that had not removed anything at all — and it asserts the refusal of
the ordinary modes on the same document, because the leak surface this pass
introduces is the un-redacted base, and a compressed real document is where
a partial guard would hide.

### `fn the_save_time_proof_is_free_when_there_is_nothing_to_prove_and_bites_when_there_is`

[`super::prove_saved_bytes`] is the check `crate::app::save` runs between
the bytes and the syscall on every save, and
[`super::save_applying_pending`] runs it once more before handing the bytes
over. It is expected to pass forever, and a check that is expected to pass
is exactly the kind this project keeps finding was never wired. So it is
falsified in both directions: an empty claim list returns `Ok` without
decoding anything, and a claim that IS present in a decoded stream comes
back as a survivor.

### `fn the_residual_count_matches_the_disclosed_list_except_for_promotion`

`crate::dialogs::redact::residual_lines` builds the list the operator
acknowledges; [`super::residual_count`] produces the number the staged
outcome sentence quotes. They must count the same things, and the
differences — promotion, which only the write-now route can observe, and the
absence proof, which the staging route has not run — are pinned here rather
than left to be rediscovered.

The `None` case is the important half and it is a claim rather than a
convenience: the staging verb discards its bytes, so no sweep has run, and a
caller passing a default `AbsenceVerification` would have told the operator
that one had and found nothing.

### `fn assemble`

The same fixture shape `pdfcer-core`'s own redaction tests use —
synthetic, so that every byte in the file is one this suite put there.
`pub(super)` so [`super::proof`]'s tests share it rather than growing a
second, subtly different assembler.

### `fn assemble_with_trailer`

The trailer is where a carrier that is on no page lives — `/Info` above
all — so a fixture that needs one needs this rather than a second
assembler. `extra` is inserted verbatim before the closing `>>`.
