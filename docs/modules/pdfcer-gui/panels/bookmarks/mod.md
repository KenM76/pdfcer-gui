# `panels::bookmarks` — the document's outline, as navigation

It navigates — [`Action::GoToPage`] — and it authors: add, rename, remove,
reorder, expand, and cut/copy/paste of a whole branch. Every one of those
leaves as an [`Action`]; nothing here touches the document.

"Bookmarks", not "Outline": the PDF specification calls the structure an
outline (§12.3.3) and every other reader calls the things in it
bookmarks. The operator-facing word is the one operators use; the spec's
word stays in the code and the doc comments.

# Why the tree is read fresh each frame rather than cached

[`pdfcer_core::outline::read_outline`] takes an object graph, not `&mut
self`, so it can run inside the draw closure — and the outline is a
property of the document that page edits can change (deleting a page can
leave a bookmark pointing nowhere). A cache would need invalidating on
every edit and undo, which is a correctness problem traded for a parse of
a structure that is a few hundred items at most.

Measure before trading back. Note the contrast with
[`crate::panels::objects`], whose decomposition **is** cached: that one
walks every content stream on the page and there is no cache anywhere in
`pdfcer-core`. The two panels differ because the work does, not because
one of them was optimised and the other forgotten.

# A bookmark with no destination is NOT an error

Three distinct states, and collapsing them would mislead:

| State | Row | Why |
|---|---|---|
| points at a page pdfcer resolved | full-strength label, tooltip names the page | the only one worth a click |
| a **heading** with no destination at all | weak label, tooltip says so | legal, common, groups its children |
| a destination pdfcer could not resolve | weak label, tooltip says so | the document meant something and pdfcer could not follow it |

Only the third is a problem. Rendering the second and third alike would
send an operator hunting for damage in a perfectly ordinary document; not
showing the third at all would hide a real defect.

## The two unnavigable kinds are LIVE controls, not disabled widgets

A row has four jobs — navigate, select, drag, expand — and three of them work
perfectly on a bookmark that leads nowhere. So the row is a live control and
it is **navigation alone** that is withheld: no [`Action::GoToPage`] is
raised, the label is drawn weak, and the tooltip says which of the two kinds
it is.

Disabling the whole row would be R83 applied too coarsely. A disabled
`egui::Button` reports no click at all, so a **heading** could not be
selected and could therefore never be the parent for an add — and a heading
is the likeliest parent there is, since a heading is what an operator files
things under. What R83 requires withheld is the thing that cannot work, not
everything that shares a widget with it.

Neither unnavigable kind is a navigation *affordance*. Both are still drawn,
because a heading's children hang off it and omitting the parent would show
them at the wrong depth, silently misrepresenting the document's structure.

[`pdfcer_core::outline::Destination`] is `#[non_exhaustive]` with six
variants; only `Page { page_index, .. }` is navigable and the match below
says so by naming it and treating everything else as unresolved. That is
deliberate: a variant added to core must default to *"pdfcer could not
follow this"*, never to a guess.

# The list honours `/Count`'s SIGN

A **collapsed** bookmark's children are not drawn, although `read_outline`
resolves the whole tree whatever the sign says — `OutlineItem::children` is
populated for a closed item exactly as for an open one. The panel has to
honour the sign because [`reorder`]'s disclosure triangle writes it: a
control that changes the file and changes **nothing on screen** is a control
that appears not to work, and the operator's next act is to press it again.

Three sentences elsewhere in this panel are about rows that are genuinely
not on the screen, and depend on this:

* [`crate::text::panels::bookmark_add_under_collapsed`], which promises the
  new bookmark will not appear until the parent is expanded;
* [`crate::text::panels::bookmarks::bookmark_move_into_collapsed`], its
  counterpart for the move;
* [`edit`]'s subtree warning, which is about a branch the operator cannot
  see.

**The count above the list is a different number from the number of rows,
and that is correct.** `outline.diagnostics.items` counts every item pdfcer
read at every level, collapsed branches included — the document's real size.
The rows are what is visible. The summary is about the document and the list
is about the screen, so they are allowed to differ.

# The truncation disclosure sits ABOVE the list

An operator who scrolls a short list and stops has already drawn a
conclusion by the time a footnote would reach them. Same reasoning as the
Signatures caveat and the Fonts coverage note; three panels, one rule.

# Indentation is keyed by object id, not by index

`ui.indent` takes an id source, and two siblings at the same index in
different subtrees would collide in egui's id space — which shows up as
the wrong row responding to a hover. The item's `ObjId` (`num`,
`generation`) is unique across the document, so it cannot.
