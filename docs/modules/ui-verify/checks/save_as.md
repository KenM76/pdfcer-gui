# `ui-verify/checks/save_as`

`save_as_rebinds_the_document_so_the_next_save_goes_to_the_new_file` —
**the half of Save As that is not the write.**

# The report


> *"we need a Save As option so that we are then making edits in the save as
> file instead of the original just like other programs have it."*

**The second half is the request.** `file.save_copy` already wrote the bytes
anywhere he pointed it; what it could not do was *move the document*, so the
next `Ctrl+S` went back to the file he was trying to leave.

# Why the oracle is the ORIGINAL file's digest

Every cheaper oracle passes against the defect this exists to catch.

| oracle | what a still-bound build does |
|---|---|
| the new file exists | **passes** — Save-a-copy has always written it |
| `save-as` was traced | **passes** — the line is emitted by the write |
| the title changed | passes only if the title is the thing that broke |

⇒ The question is *"where does the NEXT save go"*, and the only way to ask it
is to **do another save and see which file moved.** So this check saves
twice: once with Save As, once with `Ctrl+S`, and hashes the original both
before and after.

A build that wrote the copy and stayed bound to the original passes the
first three rows above and **fails phase D**, because the original changes
under it. That is a genuinely falsifying assertion rather than a confirming
one, and it is the same shape `checks::ocr` phase E uses for the same reason.

# The second edit, and why there has to be one

Between the two saves the check makes **another** edit. Without it the second
save has nothing to write, `save::has_unsaved_edits` is false, and a correct
build would legitimately do nothing at all — which is indistinguishable from
a broken one that did nothing because it was pointed at the wrong file.

# Nothing of the operator's is touched

Both files are in the run's own output directory: the fixture is **copied**
there first, and the Save As destination is a sibling. The check writes only
inside `--out`.

The native picker is never opened. `PDFCER_DIAG_SAVE_PATH` supplies its
answer, which is this project's established seam for a system dialog and is
what makes phase B an assertion about **a file on disk** rather than about a
button having been pressed.

# Every way this reports SKIP

No binary, `--no-input`, no diagnostic channel, the fixture missing, or a
ribbon control that was never declared or took no click.

## Item notes

### `const SAVE_AS`

The old path is on the line as well as the new one, and that is the point
of tracing it at all: *"the document moved"* and *"a copy was written"*
produce the same `save-copy` line, and only the pair says which file the next
`Ctrl+S` will reach.

### `fn digest`

`checks::ocr`'s, verbatim and for its stated reason: the question is *"did
this file change"*, the adversary is a bug rather than a forger, and the
**length is part of the digest** so a truncation cannot hide behind a
collision.

### `fn make_an_edit`

The cheapest edit that needs no canvas aim, no armed tool and no typing:
two ribbon clicks. This check's subject is *where a save goes*, so the edit
should be the least interesting thing in it.

### `fn the_digest_notices_a_single_changed_byte_and_a_truncation`

Phase D's whole verdict rests on this, so a digest that answered
"unchanged" for a modified file would turn the check's most important
assertion into a formality that always passes.

### `fn the_destination_is_not_the_original`

Trivial and load-bearing: if the destination resolved to the original,
phase D would compare a file with itself and pass against every possible
build, including one with no Save As at all.

### `fn nothing_outside_the_output_directory_is_written`

Both names are relative and are joined to `--out` by `CheckContext`.
This pins the intent: a future edit that reached for the repository's own
fixture directly would be writing to a tracked file, twice, on every run.
