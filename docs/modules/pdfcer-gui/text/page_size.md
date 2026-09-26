# `text::page_size` — the words for **changing the paper an open drawing
sits on**

The catalogue for [`crate::dialogs::page_size`] and for the disclosures
[`crate::app::actions::pagesize`] raises after the commit. R1: every string
a human can read is defined here and nowhere else.

## The one thing this whole catalogue exists to say

**Changing a `/MediaBox` changes the paper. It does not move, scale or
shrink anything drawn on the page.**


So this catalogue states the rule three times over, in three registers, and
that repetition is deliberate rather than sloppy:

1. [`intro`] — the standing rule, at the top of the window, before any
   choice is made.
2. [`fits`] / [`overhang`] — the **measured consequence for these sheets and
   this size**, recomputed as the operator changes the size. This is the one
   that does the work: a rule is a thing to agree with, a measurement is a
   thing to act on.
3. [`disclosure_lost_area`] — after the commit, in the status bar, because
   rule 4 says a consequence the operator cannot see on the page is owed a
   sentence off it.

## Rule 15 — "dimension" is never written bare here

Two different things on a CAD sheet are called dimensions and a paper change
affects them differently, so R8b rule 15 forbids the bare word:


The second row is *why the honest answer is "nothing moves"* rather than a
hedge. A verb that scaled the drawing to fit would have to rescale every ce
dimension group's calibration to keep its numbers true, and would silently
falsify every one it missed. This verb cannot get that wrong because it does
not touch them — and [`intro`] says so in the operator's own terms rather
than in that argument.

## Register

Short, and assumes competence, matching [`crate::text::new_document`]. He
drafts in SolidWorks; he knows what A1 is. What he does not know — because
no other application behaves this way — is which of *crop* and *shrink* he
is about to get, so that is the sentence that gets the words.

## Item notes

### `fn the_intro_says_the_drawing_does_not_move_and_is_not_scaled`

The single most important property of this whole catalogue, and the one
a reword could quietly destroy: a future editor tidying [`intro`] for
length could drop the clause that says nothing is scaled, and every
test in the workspace would stay green while the window started lying
by omission. This fails instead.

### `fn the_overhang_line_names_exactly_the_edges_that_overhang`

Both directions. A line that named all four edges regardless would read
as alarming nonsense on a sheet that overhangs only to the right; one
that named the first would silently under-report a drawing hanging off
two edges, which is what a wrongly-oriented sheet produces.

⚠ **The absence assertions match `"past the top"`, not `"top"`**, and
the reason is recorded because it cost a red run: the sentence contains
the word *"s-top-s"*, so a bare substring test reported a line naming
one edge as naming two. A negative assertion over prose has to match the
**phrase the code emits**, or it is asserting something about English
rather than about the program.

### `fn no_string_in_this_catalogue_writes_a_bare_dimension`

A CAD sheet has two kinds and a paper change affects them differently:
a **pdf dimension** is the printed measurement the exporter drew — page
content pdfcer reads and must not silently alter — and a **ce
dimension** is the one pdfcer authored. Both are unaffected here, for
different reasons, and a sentence that said "dimensions" would be
telling the operator something about a category that does not exist.

Swept over **every** string this module can produce rather than over the
one that happens to use the word today, because the rule binds the
catalogue and not a line of it — and a future sentence added by someone
who has not read R8b is exactly what this is for.

### `fn no_shipped_size_falls_through_to_its_machine_id`

The wildcard arm of [`size_name`] falls back to `PaperSize::id`, which is
machine-facing (`ansi-d`). Every size the engine ships today must have a
real name here; the fallback is for one it adds tomorrow, and this is
what makes that distinction hold rather than drift.
