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

## Item notes

### `struct Harvest`

# Why a struct rather than five `&mut` parameters

[`rows`] is recursive, so every output it collects is threaded through every
level. Five out-parameters would be five places to transpose two of the same
type — and two of these *are* the same type
(`Option<pdfcer_core::object::ObjId>`: the row that was clicked and the row a
drag began on), which is exactly the pair a reader cannot check by eye at a
call site.

[`crate::panels::pages::grid_rows`] takes them loose under a
`clippy::too_many_arguments` waiver, on the argument that a struct there
would name a type whose only purpose is to be destructured immediately. That
holds for a flat grid and not for a tree, and the difference is the
recursion: a bundle passed down four levels is written once, and four loose
parameters are written at every level.

### `const DISCLOSURE_WIDTH_PTS`

Reserved on a **leaf** as well, with `add_space`, so every title at one
level starts at one x. A tree whose rows step in and out by the width of a
triangle depending on whether they have children reads as a rendering fault,
and it is the first thing an eye notices in a list of names.

### `fn rows`

Indentation carries the structure. See the module docs on why the indent
is keyed by the item's object id rather than by its index.

# It recurses only when the row is OPEN

A walk that recursed unconditionally would draw every child of every
bookmark whatever `/Count`'s sign said, which would leave the disclosure
triangle writing the sign into the file and changing **nothing on screen** —
a control that appears not to work, and the operator's next act is to press
it again. Honouring the sign here is also what makes three sentences
elsewhere in the panel literally true:
[`crate::text::panels::bookmark_add_under_collapsed`], its move counterpart,
and [`edit`]'s subtree warning are all about a branch the operator cannot
see.

**The count above the list is a different number.**
`outline.diagnostics.items` is every item pdfcer read, at every level,
collapsed branches included — the document's real size — and the number of
rows drawn here is what is visible. They are allowed to differ: the panel's
summary is about the document and its list is about the screen.

# Every row is an enabled control, and only navigation is withheld

A row has four jobs — navigate, select, drag, expand — and three of them work
perfectly on a bookmark that leads nowhere. So the row is enabled and it is
**navigation alone** that is withheld: the click raises no
[`Action::GoToPage`], the label is drawn weak, and the tooltip says which of
the two unclickable kinds it is. The three-state distinction the module
header sets out is carried by the label's colour and its words rather than
by a dead widget, because a disabled `egui::Button` reports no click at all
and a **heading** that cannot be clicked cannot be selected as the parent
for an add — which is the likeliest thing an operator wants of one.

`enabled=` in the row's trace line means **navigable**, which is what every
reader of it assumes and what `tools/ui-verify`'s `bookmark_edit` check
skips on.

### `fn disclosure`

# A leaf gets no triangle, and that is R83 rather than tidiness

§12.3.3 Table 153 requires `/Count` only of an item that has descendants,
so an item without them carries none and has no open-or-closed state to set.
`EditSession::set_outline_open` answers `Ok(false)` for a leaf rather than
refusing — a *collapse all* sweep asks every row it walks, and refusing would
make the sweep's caller filter first for no gain — so a triangle on a leaf
would be a control that reaches the engine and correctly does nothing. Never
offer a control for something that cannot work.

The width is reserved anyway. See [`DISCLOSURE_WIDTH_PTS`].

# The hover text says the state is saved into the document

The one genuinely surprising fact about this control, and the reason it is
disclosed **before** the press rather than after: every other tree an
operator has used treats expand and collapse as a window setting, and here
it is a byte in the file. See
[`crate::text::panels::bookmarks::bookmark_expand_tooltip`].

### `fn a_resolved_destination_navigates_zero_based_and_prints_one_based`

The off-by-one that would otherwise be invisible: `page_index` is
already 0-based into `pages`, and [`Action::GoToPage`] takes the same
0-based index — so the raw value travels, and the `+ 1` happens only
where a human reads it.

Getting that backwards produces a panel that navigates one page past
every bookmark, which looks like a document defect.

### `fn any_destination_that_is_not_a_resolved_page_is_not_navigable`

`Destination` is `#[non_exhaustive]`, so core can add a variant
without this crate changing. The match must therefore *fail closed*:
anything that is not a resolved page is a row pdfcer declines to
offer, never a row it guesses at.

Asserted against a real fixture whose destinations pdfcer genuinely
cannot resolve, using the same expression the panel uses, so the two
cannot come apart. Constructing `Destination` values by hand would
prove only that `matches!` works.

### `fn the_two_disabled_row_kinds_explain_themselves_differently`

A heading and an unresolved destination are both disabled rows. If
they read the same, an operator cannot tell a perfectly ordinary
document from one whose outline is damaged.
