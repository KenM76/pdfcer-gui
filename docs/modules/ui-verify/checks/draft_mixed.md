# `a_mixed_paragraph_opens_whole_and_keeps_its_looks`

**Defect it guards.** O295: a paragraph with a bold or coloured word opens one
line at a time (`reason=mixed-looks`), or an edit of it sets every word in the
first run's look.

**Fixture.** `fixtures/mixed-paragraph.pdf`, from
`python tools/gen-mixed-paragraph-fixture.py`: the six lines of
`paragraph.pdf`, with `notes` on line 2 in Helvetica-Bold filled red. The
look changes inside one line, so the recogniser has no new reason to split the
block. Opened through the scripted pointer, in Edit mode with the Text tool.

**Steps.**

1. A click at (90, 703) on the first line must trace `text-edit-widened` with
   `lines=6` and `looks=` of 2 or more.
2. Type `Z`, press Ctrl+Enter, and wait for `edit-block-text-applied`.
3. Click the same point again: the re-opened paragraph's `looks=` must equal
   step 1's.

`looks=` counts (font resource, `Tf` size, fill) from provenance, with an
unset fill read as default black. The rewrite emits a colour only where the
inherited one differs, so without that reading a paragraph that kept its
looks re-opens one look higher.

**Falsified** against the v0.82.0 build: step 1 fails with
`text-edit-widen-declined reason=mixed-looks`. Before the unset-fill reading,
the current build failed step 3 with `looks=3`.

**What it does not prove.** That the bold red word is still bold red where it
was: a rewrite that swapped the two looks between words keeps the count. That
the inserted `Z` took its neighbour's look. A mixed size, or a look change
between lines, is not driven.
