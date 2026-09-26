# `text::deleting` — the sentences a Delete that removed nothing shows

Four of them, for [`crate::canvas::deleting`], plus the rule that decides
which refusals get one at all.

## Why four and not eleven

[`crate::canvas::deleting::Refusal`] has eleven variants and seven of them
describe a state the operator put themselves in and can **see** on screen:
nothing is selected, they are inside an image that has no parts, they are at
the point rung on a line of text. `canvas::moving::decline` settled the rule
for its own eight and it holds unchanged here — *a bar that narrates the
obvious stops being read*, and a surface nobody reads is worse than no
surface, because the next real sentence lands in a place their eye has
learned to skip.

The four below are the ones an operator meets **having done nothing
wrong**: the shape they picked is inside a container pdfcer cannot cut into,
the file's own structure forbids removing this label before the next one,
they picked four points and pdfcer removes one at a time, or the page's
contents will not decompose at all. In every one of those the outline is on
screen, round the thing they want gone, and the key does nothing. From where
they sit, Delete is broken.


## The rule every sentence follows

**Name the thing the operator can see, never the thing pdfcer models.**
[`crate::text::resizing`]'s header states it and this catalogue obeys it:
they can see a **label**, a **line**, a **corner point** and a **group**;
they cannot see a "show operator", a "subpath", an "anchor" or a "form
XObject". A refusal phrased in the file format's vocabulary reads as an
internal error.

And **never a bare "dimension"** — R8b Rule 15. The labels on the
operator's drawings are *pdf dimensions*: page content pdfcer reads and must
not silently alter. The word on screen is **label**, which is what he calls
them and what he can see.

## Where they are shown

The status bar's disclosure row, through
`crate::app::actions::disclosure::record_notes`, stamped with the epoch
currently on screen — so the sentence stands until the operator's next real
edit moves past it. Deliberately **not** drawn on the page: R8b Rule 4 is
about *disclosure*, and applied content must render exactly as saved content
will. Nothing here marks the canvas.

## Item notes

### `fn every_sentence_is_finished_prose_with_no_baked_gap`

Not a formatting nicety: `check-string-gaps.sh` exists because a lost
line-continuation backslash bakes six spaces into the middle of a
wrapped literal, and the result *looks deliberate in the diff and wrong
in the window*. The gate greps the source; this asserts the value.

### `fn no_sentence_writes_a_bare_dimension`

The project has been corrected on this once already. A *pdf dimension*
is page content pdfcer reads; a *ce dimension* is what pdfcer authors;
and a bare "dimension" on an operator-facing surface is ambiguous
between the two at exactly the moment the operator is deciding whether
their drawing is about to be altered. The word on screen is "label".
