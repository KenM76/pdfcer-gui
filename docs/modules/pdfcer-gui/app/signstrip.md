# `pdfcer-gui/app/signstrip`

The signing strip (O269 step 3): a fixed-height bar under the document tab strip, reading
*Signed N of M* with a **Next** button, Acrobat's guided-signing bar in the
form the rest of this program already uses.

## When it is drawn

Exactly when the canvas draws the red signature tags: a document is open,
`canvas::forms::offers_signing` holds for the active tool (annotations shown,
no fill refusal such as a certification signature, a tool that fills forms),
and the form has at least one unsigned signature widget. Drawn in Read mode
too, like the remote-control banner: a signing task the operator cannot see
the end of is the thing the strip exists to prevent.

`exact_size`, never content-driven: a chrome bar whose height follows its
content above a fit-to-viewport page is the recorded zoom feedback loop.

## What it counts

Fields, not widgets. A signature field with two widgets on two pages is one
signature, so `progress` takes the distinct field names in `Placed::unsigned`
and asks `OpenDoc::hand_signed` which are signed. A box signed with a digital
ID is not in `unsigned` at all and so is in neither number.

`OpenDoc::hand_signed` is measured from the document's own hand-signature
tags (`app::handsigned`), so a box signed by hand, saved and reopened still
counts as signed.

## What Next does

It never signs. It brings the next box still to sign into view and leaves the
click to the operator — the same box, tag and *Sign here* window as clicking
it directly.

- Order is `Placed::unsigned`: page, then `/Annots` order.
- The strip remembers the index it last went to, per document, in egui temp
  memory. `next` starts after it and wraps; signed boxes are skipped; a stale
  index past the end wraps to the start.
- Navigation is the Tab-between-fields pattern: `GoToPage` when the page
  differs, then `RevealRect` with `why=sign-next`, which outranks the page
  command's own scroll.

When every box is signed the button goes and the line reads *All N signature
boxes are signed*.

## Trace

- `sign-strip signed= total=` on change.
- `sign-next page= index=` per press. No field names: they are text from the
  operator's document.
- Regions `signstrip` and `signstrip.next`. The canvas publishes each drawn tag
  as `form.sign-box.<index>`, the same index, so a check can see which box Next
  brought on screen.
