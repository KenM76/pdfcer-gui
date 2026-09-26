# `pdfcer-gui/app/save/tests`

## Item notes

### `fn scratch`

`std::env::temp_dir` rather than a path in the repository: a test that
writes beside the fixtures leaves a file somebody eventually commits,
which is the exact hazard `tools/ui-verify`'s OCR check records having
hit.

### `fn the_suggested_name_is_never_the_source_file`

The shipped tooltip's promise as a **default** rather than as a warning.
An operator who accepts the suggestion without reading it must not
overwrite the drawing they were working on, and this is the assertion
that says so — the tooltip says it in words, and words are not a
mechanism.

### `fn a_created_document_is_suggested_its_own_name`

The other half of `stored_under`, and the interesting failure is not
"the suffix was skipped" but "the guard was written the wrong way round",
after which every opened document would be offered its own path as the
default — turning the tooltip's promise into a trap. Both directions are
therefore asserted, here and in the test above.

### `fn a_saved_copy_begins_with_the_original_file_byte_for_byte`

The assertion this whole module exists for, and the one that fails
against the plausible wrong implementation. A copy produced by
`to_full_bytes` would satisfy everything else a reader might check — it
is a valid PDF, it has the right pages, it carries the edit — and it
would have rewritten the file from scratch, destroying every digital
signature (§12.8.1) and discarding the previous revision that
`file.save_copy`'s shipped tooltip promises *"stays intact inside the
file"*.

A §7.5.6 incremental update cannot do that by construction: the original
revision is left untouched and the new one is appended after it. So
`output[..input.len()] == input` is a **property of the save mode**, and
it is the cheapest possible test that tells the two modes apart.

With no edits made, the engine's own contract goes further and the
output *is* the input — asserted too, because a build that appended an
empty revision to an untouched document would still pass the prefix
check while quietly growing every file an operator copied.

### `fn an_edit_survives_the_round_trip_through_a_saved_copy`

The round trip, in the smallest form a unit test can hold: rotate a page
through the engine, save a copy, and re-open the copy from disk with a
fresh `Document::load` — the same call `PdfcerApp::open_path` makes — and
read the rotation back.

Two assertions and both are load-bearing. **The edit is present**, which
is what separates "a file was written" from "the operator's work was
written"; a build that wrote `Document::bytes()` instead of the session's
update would produce a perfectly good PDF with the edit missing and would
pass every check that only asks whether a file appeared. **And the
original's bytes are still its prefix**, which is what separates an
incremental save from a full rewrite that would also carry the edit.

`set_page_rotation` rather than a markup annotation because it is the
cheapest engine verb that changes a page object and is readable back
through `EditSession::pages` without a decomposition — the *shape* of the
proof is what matters here, and `tools/ui-verify`'s `save_copy_round_trip`
makes the same claim about a real annotation placed by a real drag.

### `fn saving_a_copy_changes_nothing_about_the_open_document`

§3, asserted rather than described. Three failures this catches, each of
which looks like tidying up:

* bumping `edit_epoch` — dissolves the canvas selection, discards the
  decomposition and the page-text cache, and retires a rule-4 disclosure
  the operator may not have read, to record an event that changed nothing
  on screen;
* **zeroing** `edit_epoch` — puts it below `saved_epoch`, so
  `actions::acrobat`'s `edit_epoch.saturating_sub(saved_epoch)` reports
  **zero** unsaved changes in a dialog that is on screen precisely because
  there are some;
* writing `path`/`origin` — that is Save **As**, a command this build
  does not have, and doing it here would rename the operator's open
  document because they asked for a copy.

### `fn a_created_document_saves_and_the_copy_opens`

The case the shipped `file.new` tooltip now promises and that nothing
else covers: `tools/ui-verify`'s round trip drives an *opened* document,
because it needs a page with content to drag a rectangle across.

It is worth its own test rather than being assumed from the opened case,
because a created document is the one whose `path` is **not a file**. A
save that reached for `doc.path` anywhere — to read base bytes, to
resolve a directory, to decide anything — would work perfectly on every
opened document and fail here alone, with `Untitled 1.pdf` as the error.

### `fn a_saved_document_is_not_dirty_and_an_undone_edit_is_not_either`

Five states, and each of the two terms is load-bearing in a different
one of them, which is why the test walks the whole table rather than
asserting the headline case:

- **edited, then saved → clean** is what a build with only
  `session.is_modified()` gets wrong. That is the one the operator hit:
  the tab kept its unsaved dot, and the next Close asked a question
  whose only save button opened a picker and then closed the document.
- **edited, then undone → clean** is what a build with only the epoch
  comparison gets wrong, because an undo bumps `edit_epoch` like every
  other edit.

So a build that drops either term passes half of this and fails the
other half, which is exactly what a truth-table test is for.

### `fn a_write_that_cannot_happen_is_a_named_refusal`

A directory that does not exist is the commonest real failure — the
operator typed a path, or a network share went away between the dialog
and the write — and it is the one that must not be silent. Asserted as
the `Write` variant specifically, because the two variants send a reader
to two different subsystems and collapsing them into one would throw that
away at the last step.

### `fn stage`

Three lines in four tests, and it is a helper rather than repetition
because the last line is the one that gets forgotten: `edit_epoch += 1` is
what `crate::app::actions::apply::vector_edit` does after every successful
edit, and [`has_unsaved_edits`] reads it. A test that staged without it
would be asserting over a state the running program never reaches.

### `fn a_document_with_a_staged_redaction_has_unsaved_edits`

* **Under `Pass 250.1`** the session *collapsed*, so `is_modified()`
  answered false immediately afterwards and the predicate inherited it.
* **Under `Pass 250.2`** nothing is mutated, so `is_modified()` answers
  whatever the marks made it — true when the operator has just made them,
  and false when the marks were already in the file he opened. That second
  case is the one a two-term predicate still gets wrong, and it has a test
  of its own directly below.

This one walks the same table the O65 test does, on the staged branch:
dirty after the staging, clean after the save, dirty again after a further
edit. The middle row is what stops the fix from being *"always answer
dirty once a removal has been armed"*.

### `fn an_armed_removal_alone_is_enough_to_make_a_document_dirty`

The assertion above is the headline and it would still pass on a two-term
predicate, because marking is itself an edit and `is_modified()` sees it.
This is the state where it would not: a session whose only difference from
its base is an **armed removal**.

It is built by staging and then undoing back to a clean session — which is
possible only because `Pass 250.2` preserves undo, and which reaches
exactly the state an operator gets by opening a drawing that already
carries its `/Redact` marks and arming it. `is_modified()` is false there,
the epochs differ because the funnel bumped one, and without
`has_pending_redaction()` the answer is **clean**: no tab marker, no
question on Close, and the arming discarded in silence.

### `fn a_staged_document_saves_through_the_redaction_writer_with_the_content_gone`

The end-to-end assertion for `OPERATOR_REQUESTS.md` O125's second half:
the operator arms the removal in the open document, then presses Save, and
the file that lands has the content gone. [`write_copy`] is the one
function every save verb goes through, and the fork inside it is the whole
of §1.1.

**The `Written::RedactionApplied` assertion is not decoration.** A build
that failed to fork would not leak — the engine refuses both ordinary
modes — it would simply stop saving, and the failure would arrive as a
refusal rather than as a wrong file. What this pins is the *route*, so a
future "simplification" that removed the fork fails here with a sentence
naming what it removed rather than in three unrelated tests about
`WriteError`.

`crate::redact::tests` proves the same thing about the session's bytes.
This proves it about **the file on disk**, through the shell's own save
path, with the shell's own settings applied.

### `fn a_save_whose_bytes_still_hold_the_redacted_text_is_refused_and_writes_nothing`

The falsification for the check above. [`write_copy`] is handed a claim
that is demonstrably still in the document — no removal is armed, so the
ordinary writer runs and the claim is a lie — and must refuse **by name**
and leave no file behind.

It is the check nobody expects to fire, which is exactly the kind this
project keeps discovering was never wired, which is why its bite is
asserted rather than assumed.

### `fn undoing_the_marks_under_an_armed_removal_refuses_the_save_by_name`

The trap `Pass 250.2` brings with it, and the one sequence in this feature
an operator reaches by doing something entirely reasonable:

1. mark, then *Review & apply* ▸ *this document* — the removal is armed;
2. **undo the marks**, which now works, and is the point of the whole pass;
3. press `Ctrl+S`.

There is nothing left to remove, so the removal refuses; and the ordinary
modes are refused too while the arming stands. The document cannot be saved
at all until it is called off.

The failure this test is looking for is the *tempting* repair: falling
back to the ordinary writer when the removal reports `NothingToApply`. That
build saves successfully, looks correct, and writes a document whose
`redaction_absence_claims` still say text was removed from it — so the very
next save of the same session refuses, inexplicably, having already written
the file. The refusal here is the honest outcome, and the sentence names
the remedy.

### `fn open_at`

`open_local_fixture`'s body against a path the test wrote, because one of
the assertions below needs a document that is **not** in `fixtures/` — a
deliberately damaged one, which must never be committed.

### `fn a_healthy_nested_document_still_saves`

The negative control, and it is not a formality: a guard wired the wrong way
round — or one whose `is_consistent` is inverted — refuses every save in the
program, and every *other* assertion about the guard would still pass. The
fixture is the nested one on purpose, so the control is taken on the shape
the guard was written for rather than on the flat shape it is trivially
right about.

### `fn a_document_whose_page_tree_disagrees_with_itself_is_refused_and_writes_nothing`

The bite. The corruption is planted in the **base file's own bytes** rather
than produced by an engine verb, and that is deliberate for two reasons:

1. **It cannot rot.** The day `pdfcer-core` fixes `delete_pages` this test
   keeps testing the same thing, because an incremental save keeps the base
   revision verbatim (§7.5.6) and appends — so a base whose root `/Count` is
   wrong produces an output whose root `/Count` is wrong, whatever the
   writer does. A test built on the engine's *current defect* would invert
   on good news, which is the one thing an assertion must never do.
2. **It proves the guard reads the OUTPUT.** The session is never asked to
   change a page here. A guard that only inspected what an edit did would
   see nothing and pass.

The plant is one digit: the root node's `/Count 12` becomes `/Count 13`,
the same byte length, so every cross-reference offset in the file stays
valid and the document still opens. Twelve pages are reachable, the root
claims thirteen, and Acrobat would show a blank thirteenth.

### `fn deleting_a_page_from_a_nested_document_is_caught_at_the_save`

`delete_pages` on the nested fixture, then a save through `write_copy` — the
exact path `file.save_copy` takes. This is the assertion that says the guard
catches the defect *he reported*, rather than a shape chosen because it was
easy to construct.

It **skips loudly** rather than failing if `pdfcer-core` is ever fixed.
The day the engine walks the ancestor chain this save succeeds, and a test
that went red on the repair would turn good news into a broken build. The
guard's own bite stays under permanent assertion in
`a_document_whose_page_tree_disagrees_with_itself_is_refused_and_writes_nothing`,
which depends on no engine behaviour at all. That split is the standing
answer to `check-stale-blockers`' subject: a claim about what the engine
cannot do has a shelf life measured in hours, so it goes where its expiry is
harmless.
