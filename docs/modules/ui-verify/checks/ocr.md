# `ui-verify/checks/ocr`

`ocr_recognises_a_page_and_the_document_keeps_it` — the check for a feature
whose whole product is **text that was not in the document before**, and
whose whole risk is that it might have touched a file nobody saved.


It was called `ocr_recognises_a_page_and_writes_a_new_file`, and every
assertion in it was about a **Save-a-copy**: a picker was answered through
an environment seam, a file appeared where the harness asked, and the source
was hashed before and after to prove it had not been overwritten.

All of that existed because `ocr::layer::add_ocr_layer` took an immutable
`&Document` and returned a complete PDF. Recognition was the one capability
in pdfcer that was not an edit, so a shell holding an open session could only
offer *"here is a different file, somewhere else"*. The operator's verdict,
2026-08-26: *"Why do I have to save a copy instead of just go back into my
pdf and save over it?"*


# What no unit test in this workspace observes

Four links, and only the first is testable off the binary:

| # | Link | Its own test |
|---|---|---|
| 1 | recognition produces words, and applying them yields extractable text | yes — `ocr::fixture::tests::recognises_the_synthetic_page` |
| 2 | a ribbon click on `file.ocr` reaches the dialog and the dialog's controls exist | **no** |
| 3 | the completed run reaches `vector_edit` and the **session** takes it | **no** |
| 4 | **nothing is written to disk**, because nobody saved | **no** |

Link 3 is the new one and it is the one with a plausible silent failure: the
dialog raises `Action::ApplyOcr` into a queue, and an action that is raised
and dropped leaves a window saying *"the text is now in this document"* over
a document with no text in it. Two trace lines are asserted rather than one
— `ocr-applied` says *I asked*, `ocr-layer` says *it happened* — and only
the pair distinguishes those two states.

# The falsifying phase, and what it is aimed at

Phases A–D could all be passed by a build that recognised correctly and then
wrote the result out to the operator's file on its own initiative. That is
not a hypothetical shape — it is what this feature did for its first two
weeks, and it is the shape somebody restores while "making OCR persist".

So **phase E hashes the fixture before the run and after it**, and the
verdict rests on the digests being equal. Nothing in the run saves, so
nothing may have been written. It is a genuinely falsifying assertion rather
than a confirming one: it fails against the plausible wrong implementation
and there is no way to satisfy it accidentally.

# Why the fixture is `synthetic-image-only.pdf`

Because a document with no extractable text is the only kind on which OCR's
result is unambiguous: any text in the output came from the recogniser.
`crates/pdfcer-gui-base/src/ocr/fixture.rs` generates it, and **its header is
required reading before believing anything here**. The short version, and it
is stated in this check's own report so a green result cannot be misread:
the fixture is a *rendered page*, not a scan. It has no scanner noise, no
skew and no JPEG ringing, so this check establishes the **plumbing** and
establishes **nothing** about recognition quality on real scanned material.

# Mouse only, and one consequence that matters

## Item notes

### `const READ`

**Read, deliberately, and it is half of what this check is for.** The
operator's instruction is that OCR be available in Read, and the first
implementation of this feature put the command on the **Tools** tab —
`RIBBON_IA.md` §5.7's placement — where Read cannot reach it at all, Read
being shown `["file", "view"]` alone. Driving the ribbon in Read is what
turns that from an argument into an observation.

### `const EDIT_EVENT`

Both are asserted, and the pair is the point. The dialog's line says *I
asked*; this one says *it happened*. A build where the action was raised and
dropped emits the first and not the second, and that is a state with a
dialog claiming the text is in the operator's document while the document
has none.

### `const RECOGNITION_FRAMES`

Generous. Recognition of one page measured about one second in a release
build and twenty in a debug one, and this harness drives whichever binary it
was pointed at. A wait that was too short would report "recognition did not
happen" about a build that was still working, which is the worst available
failure message.

### `fn default_fixture`

`tools/ui-verify/` → up two → the workspace root. Stable whatever the
harness was invoked from and whatever `--source-root` says, which is the
property the first two attempts at this lacked — see [`drive`].

### `fn digest`

Not cryptographic and does not need to be: the question is *"did this file
change"*, the adversary is a bug rather than a forger, and carrying a SHA-2
implementation into this crate to answer it would be a dependency for
nothing. The **length is part of the digest** so a truncation cannot be
hidden by a hash collision, which is the only failure mode a 64-bit hash
realistically has here.

### `fn the_digest_notices_a_single_changed_byte_and_a_truncation`

Phase E's whole verdict rests on this function, so a digest that answered
"unchanged" for a modified file would turn the check's most important
assertion into a formality that always passes.

### `fn the_check_drives_read_and_a_tab_read_actually_has`

Pinned because the two together *are* the finding: `RIBBON_IA.md` §5.7
puts OCR on Tools, Read is shown `["file", "view"]`, and a check that
quietly drove Edit instead would pass against a build in which OCR is
unreachable in Read.

### `fn the_output_path_is_not_beside_the_fixture`

A stray recognised copy beside the fixture would be committed by
somebody eventually, and a repository that gains a file every time the
harness runs is a repository whose `git status` stops being read.

### `fn click_command`

The same shape as [`driving::click_mode_segment`], for the other half of the
ribbon. Not folded into that module because it is the first check to need
it: a second caller is the moment to move it, and moving it on the first
would leave `driving` with an untested function.

# The OCRcer variant

`ocrcer_recognises_a_page_and_the_document_keeps_it` drives the same chain
with `ocr_engine = ocrcer` seeded in the sandbox's preferences, and fails
unless `ocr-started engine=` names `ocrcer`. The first check takes the
build's default recogniser, ocrs, so without this one OCRcer's model path and
adapter are never driven. It needs a packaged build: OCRcer's model sits in
`models/ocrcer/` beside the binary.

Falsified by seeding `ocrs` under it: the check fails and names the
recogniser that ran.
