# `dialogs::offpage` — **which of my drawings have marks outside the sheet?**


## ★★★ The third of the operator's question that was still open

**Ken, 2026-09-10:** *"how do I view and edit objects that are off of the
page? we added this feature but I didn't see how to enable it."*

The **view** and **edit** halves shipped the same day: the canvas learned to
rasterize a halo past the sheet edge, and the cull learned that a page can be
visible when its own rectangle is not. An operator who knows an object is out
there can now scroll to it, select it, drag it back and delete it.

**That leaves the half nobody could do anything about: KNOWING.** Off-page
content is invisible by construction — it does not render, it does not print,
and no amount of looking at a document discloses it. Before this window the
only way to find it was to already suspect it and go hunting at 8 % zoom on
every sheet of a thirty-six-sheet set.

## ★★★ Why it is a PROTECT control and not a view option

Because off-page content is a **leak class**, and a textbook one. Every
instance this project has seen on a real CAD export is a thing the sender
believed was gone:

- a title-block border cropped by changing the page size rather than the
  geometry, with the old revision table still sitting past the left edge;
- a superseded revision note dragged off the sheet instead of deleted;
- a customer's name moved out of the frame for a drawing about to go to a
  different customer.

None of it renders. All of it is still in the content stream, still
extractable by every PDF library on earth, still returned by a text search,
and still emailed. That is the same sentence the redaction panel exists for,
which is why this window sits in the same ribbon group and marks with the
same mechanism.

## ★★★ The scan runs ONE PAGE PER FRAME, and that is the whole design

`crate::app::cache` records the measurement this is built around:

```text
page-objects-built page=0 objects=129758 leaves=10256 ms=469
```

469 ms to decompose **one** sheet of the operator's benchmark drawing. A
synchronous `scan_document` over a thirty-six-sheet set is a **seventeen
second freeze** with no window, no cursor and no way to cancel — and the
house precedent (`crate::dialogs::redact` runs its whole removal on open)
was judged not to scale to this, because that one runs once over a document
the operator has already decided to rewrite and this one runs on a question.

So the window opens **immediately, empty**, scans exactly one page per frame,
requests a repaint while it has work left, and states its progress. The
consequences are all good and all deliberate:

| property | why it follows |
|---|---|
| the title bar works | the frame loop never stops |
| Close works mid-scan | it is an ordinary button on an ordinary frame |
| findings appear as they are found | the list is drawn from what has been scanned |
| the rest of the program keeps drawing | one page of work per frame is ~½ a frame on the worst sheet measured |

★ The one thing it costs: the answer is not instant on a large set. Hence
[`crate::text::offpage::scanning`], which exists so that "still working" and
"found nothing" cannot look the same — the failure this window would
otherwise have.

## ★★ Why the window opens even when the answer is "nothing"

[`crate::dialogs::unembed`] returns `None` rather than opening over a
document with nothing to do, and that is right for a command that *offers an
operation*. This is not one. The operator pressed a control that asks a
**question**, and *"nothing is drawn outside any page boundary in this
document"* is the answer they came for — arguably the more valuable one,
because it is the one that lets them send the file.

A command that answers a question by doing nothing visible is the defect
class this project keeps finding.

## Rule 4 — "fuzzy, never sneaky"

Nothing here marks the canvas, and the temptation to do so is real: it would
be easy, and wrong, to tint the off-page halo or outline each finding on the
page view. Applied content renders exactly as saved content will, and a
second rendering path for "content pdfcer is suspicious of" is two paths that
drift. Every word of the disclosure is in this window or in the status line.

The **surviving half** is honoured in full: the census reports the recovered
text of off-page text objects (see [`crate::text::offpage::object_row`]), the
marking action discloses what Undo will do, and a page whose content would
not decode is reported as *unchecked* rather than folded into a clean bill.

## What it does NOT do
