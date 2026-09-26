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

### `fn intro`

# Why this is the first thing on screen and not a footnote

Because it is the one belief the operator arrives with that is wrong. Every
"page size" control he has ever used — Word, LibreOffice, a print dialog —
reflows or scales. This one does not, and a window that let him discover
that from the result would be the *"fuzzy, never sneaky"* failure in its
purest form: he would get a file that is exactly what he asked for and
nothing like what he wanted.

### `fn now_uniform_unnamed`

Said out loud rather than shown as bare numbers with no comment. A CAD
exporter that writes 2,381.10 × 1,683.78 has produced an A1 sheet rounded to
two decimals by a units conversion, and an operator who sees "not a standard
size" learns something true about his own export pipeline.

### `fn now_mixed`

This is the state that makes a single "current size" readout a lie, and
the reason the dialog reads the operands rather than the current page. A
drawing set with one A3 detail sheet among nine A1s is the ordinary case,
not the exotic one.

### `fn size_name`

The wildcard arm is load-bearing. `PaperSize` is `#[non_exhaustive]` and
its own docs say the table will grow (ARCH, JIS B, ISO B/C); a size the
engine adds must appear in the list with its machine id rather than making
this module fail to compile or, worse, silently vanish from the picker.

### `fn fits`

This is the *only* wording in the window that promises anything, so it is
bounded exactly to what was measured — the union of the picked pages' drawn
vector extents, which is what `PageObjects::page_bbox` computes. See
[`overhang_unmeasurable`] for the case where even that could not be read,
and [`annots_not_counted`] for the boundary this sentence does not cover.

### `fn overhang`

# Why this names the EDGES and the AMOUNT

Because "content will be cropped" is a warning and this is a
**measurement**, and the operator's decision turns on the difference. On his
own A1 title-block sheet the overhang is 1,636 pt off the right edge — the
width of the whole title block — and a number that large tells him
immediately that he has picked the wrong size. A generic warning would read
identically for a 2 pt overhang he does not care about.

The four amounts are in points because that is the unit the sheet is in and
the unit the numbers beside it are in; a millimetre conversion here would
make the reader do arithmetic to compare this line with the one above it.

### `fn overhang_unmeasurable`

A stated boundary rather than a cheerful silence. A page whose content
stream will not decompose is exactly the page most likely to be a strange
export, and reporting "nothing falls off" for it would be a false negative
dressed as a measurement — which is the failure the engine's own
`MediaBoxChange` names as its residual and refuses to commit.

### `fn origin_differs`

# Why this exists at all

§7.7.3.3 does not require a media box to start at `(0, 0)`, and imposition
output and cropped scans really do carry offset ones. `set_media_boxes`
takes **one** rectangle for the whole selection, so when the picked sheets
sit at different corners no single rectangle can preserve all of them; the
dialog anchors at the origin and says so, rather than moving the paper
relative to the drawing without a word.

### `fn disclosure_lost_area`

The asymmetry is the part worth reporting and is quoted from the engine's
own reasoning: §14.11.2.1 says content outside the media box *"may safely be
discarded without affecting the meaning of the PDF file"* — an unusually
strong permission, because it asserts the discard is meaning-preserving. So
shrinking is reversible **in pdfcer**, by Undo, and is not reversible **in
the ecosystem**, once the file has been through any other tool.

### `fn disclosure_crop_outside`

Disclosed, not repaired, and the engine's argument for that is quoted
because the operator would otherwise reasonably expect a fix: a conforming
reader *"shall treat the box as its intersection with the media box"*, so
clamping the entry would change no reader's output while rewriting an entry
the operator did not name.

### `fn disclosure_inherited`

Worth a sentence rather than silence: the page is now sized by
inheritance, so a later change to the document's default size will move it
and a sibling's will not. That is a real, invisible difference in what the
next edit does.

### `fn refused_certified`

Worded rather than traced, because `RESUME.md`'s standing cross-cutting
defect is that *"every engine refusal reaches the operator as SILENCE"*.
Measured 2026-09-06 on `fixtures/certified-comments.pdf`: the engine refuses
with `CertificationForbidsChange` and this is what that has to read as.
