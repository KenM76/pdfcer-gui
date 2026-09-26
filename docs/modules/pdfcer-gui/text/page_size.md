# `text::page_size` — the words for **changing the paper an open drawing
sits on**

The catalogue for [`crate::dialogs::page_size`] and for the disclosures
[`crate::app::actions::pagesize`] raises after the commit. R1: every string
a human can read is defined here and nowhere else.

## ★★★ The one thing this whole catalogue exists to say

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

## ★★ Rule 15 — "dimension" is never written bare here

Two different things on a CAD sheet are called dimensions and a paper change
affects them differently, so R8b rule 15 forbids the bare word:


★ The second row is *why the honest answer is "nothing moves"* rather than a
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
