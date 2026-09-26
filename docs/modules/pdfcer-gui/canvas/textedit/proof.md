# `canvas::textedit::proof` — **the tail did not move, proved in the bytes**

`crate::redact::proof`'s shape applied to `DEFECTS.md` **D4b**: a claim about
what an edit does to a *file*, asserted against the file, with the falsifying
run beside it.

## What is being proved, and why nothing else in the suite proves it

D4b says two things are wrong on commit, and both are about **text the
operator did not touch**:

1. a right-aligned / centred / justified tail is pushed off the edge it is
   flush against;
2. a rotated line's tail is displaced along user-space x, which is not the
   direction its baseline runs.

Neither is visible in any assertion about the edit itself. The edited run
comes out correct in both builds — the right glyphs, the right place, the
right font. What differs is a `Tm` belonging to something else, twenty bytes
further down the content stream, and the only oracle that can see it is the
stream.

## ★ The falsifying run is the point of the module

Every assertion here is made **twice**: once through
[`super::disposition::options`], which is the shipped decision, and once
through `EditOptions::default()`, which is what a shell that made no
decision at all would pass. The second run is not decoration — it is what
makes the first one evidence:

* a build with the rule removed produces exactly `EditOptions::default()`,
  so the falsifying run *is* the broken build, executed;
* a fixture that does not actually exercise the defect makes both runs
  agree, and the `assert_ne!` between them then fails — so this cannot pass
  by flattering the thing it measures.

## Why the fixture's content stream is uncompressed

Because the oracle is a byte scan for a `Tm` operand triple, and a
Flate-compressed stream would answer "absent" for a correct build and a
broken one alike — a false pass in the only direction that matters. See
`tools/gen-textedit-fixtures.py`, which says the same thing from the
producing side.

## Where the *second process* proof lives

Not here. This module proves the arithmetic against the written bytes in one
process; `tools/ui-verify`'s `text_edit_pins_an_aligned_tail` drives the real
binary, saves a copy through the real command, and re-opens it in a second
process — which is the only honest proof that an edit reached the bytes *by
the route an operator takes*. The two answer different questions and neither
substitutes for the other.
