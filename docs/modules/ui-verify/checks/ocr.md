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
