# `panels::bookmarks::clip` — **cut, copy and paste a bookmark and everything under it**

`OPERATOR_REQUESTS.md` **O59**, item 3, and the last of the three.

## This is the one Acrobat cannot do

`pdfcer-core`, 2026-08-29: *"Acrobat cannot do this between two files at all;
Adobe's own documentation says so by name."* Copying a chapter's bookmark
subtree out of one drawing set and into another is a thing an operator with
a template has always had to do by hand, one bookmark at a time.

Worth stating plainly because it changes how the controls should read: this
is not a parity feature catching up with a reference implementation, so
there is no established wording to borrow and the sentences below are
written from what the operation actually does.

## Why the controls are in this panel and not on the ribbon

Every other bookmark verb is here — Add, Rename, Remove, the reorder arrows
and the disclosure triangles. A bookmark is edited where it is *seen*,
because the tree is the only thing that says which one is selected and what
is filed under it.

⇒ So Copy and Paste join them, and there is no ribbon entry and no chord.
`app::dispatch::pageclip`'s header carries the general form of that argument
for pages; here it is simpler, because there was never a competing claimant:
`Ctrl+C` belongs to the canvas and no bookmark has ever wanted it.

## The one question that must be asked BEFORE the press

**Does the destination document have the pages these bookmarks point at?**

The engine flagged this itself and it is the third of its three
*"produces a document that looks right and is not"* cases:

> A destination naming a page this document does not have is **DROPPED, not
> clamped**. A dropped-destination bookmark still shows, still has its
> title, and does nothing when clicked, with nothing on screen to
> distinguish it.

`OutlineClip::deepest_page()` against this document's page count answers it,
and it is asked **while the operator can still choose** — before the paste,
beside the button, rather than as a report afterwards.

Dropped rather than clamped is the right engine behaviour and worth
understanding before writing the sentence: clamping would send the operator
to *some* page, confidently and wrongly, which is worse than a bookmark that
plainly does nothing. §12.3.3 permits an item with no destination — a pure
grouping entry — so a destination-less bookmark is a legal, honest shape.

## Where a paste lands

**As the last child of the selected bookmark**, or at the top level when
nothing is selected. That is `add`'s rule exactly, and reusing it is the
point: an operator who has learned where a new bookmark appears already
knows where a pasted one will.
