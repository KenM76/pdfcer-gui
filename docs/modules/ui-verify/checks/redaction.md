# `ui-verify/checks/redaction`

`checks::redaction` — mark a page, apply the redaction, and prove the text
is **gone from the bytes**

# The defect this exists to catch

**A shell that calls `redact::apply_redactions` directly and writes the
bytes ships an unverified redaction and will not know.** A caller can exit
SUCCESS on a file it never verified.

*"Will not know"* is the whole problem, and it is why this check exists in a
harness rather than only in the crate's own test suite. A build that marked
a page, reported four regions removed, wrote a file and removed **nothing**
would:

* pass every unit test that asserts the pipeline is called;
* emit a `redact-written` trace line indistinguishable from a correct one,
  because a trace line is written by the code under test, about itself;
* produce a file that opens, has the right page count, and looks right.

The only thing that separates it from a correct build is **what is in the
bytes on disk**, read by something that is not the program that wrote them.

# The falsification, and where it is

A test that checks a relation rather than a magnitude is satisfied by any
absurdity in the right direction. An absence check is the extreme case — it
passes on an empty
file, on a file that was never written, and on a grep looking for the wrong
string.

So the same byte scan is run **three** times and only one of the three is
the actual verdict:

| run | over | must be | what a wrong answer means |
|---|---|---|---|
| 1 | the **fixture**, before anything | `SECRET` **present** | the instrument can register the secret at all. This is the falsifying phase: a build that marks and reports success without removing anything fails at run 3, and a *check* that could not see the secret in the first place fails here rather than passing vacuously |
| 2 | the **output** | `SURVIVOR` **present** | the scan is still a valid instrument on *this* file. Page 2 is untouched, so its text must still be findable — and if the writer had compressed the streams, this fails and says so, rather than letting run 3 pass because a deflate stream hid everything |
| 3 | the **output** | `SECRET` **absent** | **the verdict** |


# …and a fourth oracle, in a second process

A raw byte scan is the harness's own reading of the file. The other honest
question is what **pdfcer** makes of it, and `checks::save_copy` established
the shape: re-open the written file in a second process and read the answer
out of its trace.

Here that is `file.copy_page_text`, which runs the same extraction
`pdfcer extract-text` does, over the page the redaction covered. On the
fixture it traces `text-copied source=page chars=N`; on the redacted output
the same click must trace `text-copy-declined source=page
reason=nothing-to-copy`, because there is nothing left on that page to copy.

That is the assertion an operator would actually make — *"I opened the file
and tried to copy the text out"* — made by a process that did not perform
the redaction and holds none of its state.

# The fixture is generated, and every byte in it is the harness's

Two pages, uncompressed, one distinctive string on each:

* page 1 draws [`SECRET`], and is the page that gets marked;
* page 2 draws [`SURVIVOR`], and is never touched.

Generated into the run's output directory rather than committed, for
`checks::save_copy`'s reason about stray files — and **two pages rather than
one**, which is the design decision that makes this check falsifiable
without a keyboard. A one-page fixture with the whole page marked has no
survivor, so a build that emitted a blank document would pass every absence
assertion. The second page is the negative control, and it is reached
without typing anything: the mark covers page 1 only.

**No keyboard is used anywhere in this check**, by choice rather than by
limitation — [`crate::checks::add_text`] types real characters and asserts
they land. The marking route driven here is the one that needs no text
entry: *Mark whole page*. The search-and-mark route — which needs a query
typed
into a field — is therefore **not covered here**, and that is stated rather
than left to be discovered. Its rule is unit-tested in
`crates/pdfcer-gui/src/app/actions/apply.rs`; what is not verified by driving
is the field itself.

# Two gaps a reader should know about

* **A region name is shared between two crates and enforced by neither.**
  `REGION_STAGING_NOTE` here must spell the same string the application
  publishes. If the two disagree, `declared` returns `None`, phase E3
  reports THE STAGING DISCLOSURE IS MISSING, and the message is about a
  sentence that is on screen. Compare the two constants before believing
  the application is at fault.
* **The `Phase::Staged` surface is reached by nothing in this suite.** See
  phase E4's own note: `redact-apply-cancel-staged` is published by the
  application and is clicked by no check.

# What each phase fails on

| phase | fails when |
|---|---|
| A | the fixture's own text is not extractable by pdfcer, so the check has no baseline |
| B | `edit.redact` opens no panel, or the panel's controls publish no rects |
| C | marking traces no mark, so nothing after it means anything |
| D | the apply report never appears, or reports `verified=false` on a clean fixture |
| E | **the confirm control is live before the acknowledgement is given** — the gate that stands between an operator and the one irreversible operation in the program |
| E2 | *note only* — the *replace the original* choice is not drawn. A note rather than a failure because the application draws it only when the source is still a file on disk |
| E3 | **the default destination is not on screen**, or **the staging disclosure is missing or sits below the confirm control**. The default destination removes nothing at the click and clears no undo log — it ARMS the next save, and the page does not change. That is the fact the sentence carries, and it is the one thing an operator cannot work out by looking. Geometry is the one thing the headless suite cannot assert |
| E4 | switching to *a new file* leaves the button dead (the overwrite acknowledgement being demanded by a destination that overwrites nothing), or leaves the staging disclosure on screen (which claims nothing is written, on the destination that writes) |
| F | no file was written |
| G | **the source file changed** — a redaction that wrote over the document it came from |
| H | the secret survives in the output, or the survivor does not |
| I | a second process can still extract the redacted page's text |

## Item notes

### `const DESTINATION_REPLACE_REGION`

Declared by the application **only while the document has an original to
replace** — `RedactDialog::can_replace_original`, which asks the file system
— so its absence is ambiguous between "this build does not draw the control"
and "the fixture is no longer on disk". Phase E2 says so rather than
choosing.

### `const DESTINATION_INTO_DOCUMENT_REGION`

Declared **unconditionally**, unlike its two siblings: every document can
be redacted into, including one created in this session with no file to
replace. So its absence is unambiguous and is a **failure** rather than a
SKIP — there is no innocent reading of it.

### `const DESTINATION_INTO_DOCUMENT_NOW_REGION`

# Why its absence is a FAILURE and not a SKIP

The deferred destination is the default and, by its own doc's terms, **the
page does not change** when it is chosen. Offer that alone and the operator
presses the only button there is and watches nothing happen — which reads
as the feature having regressed to *"just the 'don't apply yet' button"*.

⇒ This row is the other half. Its absence is exactly that state, so there
is no innocent reading of it — unconditional, like its *this
document* sibling above, and for the same reason: every document can be
redacted into, including one created in this session with no file to
replace.

### `const STAGING_NOTE_REGION`

Declared only while the deferred destination is selected, which is what
makes the geometric assertion in phase E3 possible: the sentence must be
**above** the confirm control, because it is the one thing the operator
cannot work out by looking — he presses a control about permanent removal
and *the page does not change*.

The name says **staging**, and it has to: the sentence at this region is
about the write being deferred, not about undo history. A region name that
described a different sentence would aim this check at one thing and find
another — a check that passes while measuring something else, which is this
harness's own worst outcome.

### `fn digest`

`checks::save_copy`'s, for its reason: the question is *"did this file
change"*, the adversary is a bug rather than a forger, and the length is
part of the digest so a truncation cannot hide behind a collision.

### `fn invokes`

A count rather than a presence: this check clicks four different controls
across two processes, and *"has it ever been invoked?"* would be answered
`true` by a click made ten seconds earlier.

### `fn extracted_chars`

`Ok(Some(chars))` when it copied something, `Ok(None)` when it declined
because there was nothing to copy. Used **twice** — once on the fixture and
once on the redacted output in a second process — which is the whole reason
it is a function: the two answers have to come from the identical sequence,
or the comparison at the end is between two different measurements.

### `fn the_generated_fixture_contains_both_strings`

The falsifying phase, asserted at build time as well as at run time.
Phase 1 of the byte scan reports a harness defect if this is ever false;
this catches the same thing without launching anything, which is where a
developer who has just edited [`fixture_bytes`] will see it.

### `fn the_secret_and_the_survivor_are_unrelated_strings`

A shared prefix would make the survivor's presence satisfy a scan for
the secret, which would turn the check's verdict into its own negative
control. Asserted rather than eyeballed because the two constants sit
four lines apart and are deliberately similar in shape.

### `fn the_byte_scan_answers_both_ways`

`contains` is the whole verdict of phases 1–3, and a scan that always
answered `false` would make the check pass against every build. Both
directions, plus the empty-needle case, which must be `false` — the
mathematically correct answer (`true`) would make every run report a
leak.

### `fn the_selectors_match_the_applications_own_names`

Spelling, and it is not a formality: these strings are matched literally
against the application's `ui-rect` declarations, so a rename on either
side silently un-aims every click this check makes and the check reports
a missing feature that is merely spelled differently.
