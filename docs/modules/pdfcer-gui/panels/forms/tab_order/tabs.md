# `pdfcer-gui/panels/forms/tab_order/tabs`

## Item notes

### `fn tabs_name`

`Dict::get` collapses a null-valued entry to `None` (§7.3.7/§7.3.9), so
`/Tabs null` reads as absent without a second check — which is right: a null
value is the standard's way of saying the key is not there.

A `/Tabs` whose value is not a name (a string, a number, an array) reads as
absent too. That is a malformation, and the honest reading of a malformed
entry is that the file has not named a tab order — inventing one from a
string that happened to say "R" would be pdfcer deciding what the file meant.

### `enum TabsMode`

Five named values (ISO 32000-2 Table 31; `A` and `W` are PDF 2.0) plus a
verbatim catch-all. The catch-all is modelled rather than folded into one of
the five for the reason `pdfcer-core` gives for keeping an unrecognised `/RT`
name: *"a name pdfcer does not recognise is a document fact and flattening it
to the default would make the model claim the file said something it did
not."*

### `fn from_name`

Byte comparison against the five names the standard defines. Anything
else — including a lower-case `r`, which is a *different name* in PDF
and not a spelling of `R` — is [`Self::Unrecognised`].

### `enum Sequence`

The one thing an operator must not be left to guess. A list that silently
showed the wrong sequence would be worse than no list at all, which is why
this is a modelled answer with a sentence per value rather than a footnote.

### `fn page_tabs`

`slot` is `pdfcer_core::page_tree::PageSlot`, whose `ancestors` are **root
first** and exclude the page itself — so the nearest ancestor is the *last*
element, and this walks them in reverse.

# Why the ancestors come from `PageSlot` rather than from a `/Parent` walk

`pdfcer-core`'s own (private) `page_uses_structure_tab_order` chases
`/Parent` from the page, bounded by `page_tree::MAX_TREE_DEPTH`. That is the
obvious implementation and it has two properties this one does not want.

1. **It trusts `/Parent`.** `/Parent` is Required on a page, but a file that
   omits it, or that points it somewhere other than the node whose `/Kids`
   actually holds the page, still has a perfectly good page tree read from
   the top. `PageSlot::ancestors` is that top-down walk's own record of how
   it reached this page, so it cannot disagree with the page numbering this
   view is indexing by.
2. **It needs its own depth guard.** The downward walk is already bounded
   (`MAX_TREE_DEPTH`, and a visited set), so `ancestors` is a finite vector
   by construction and there is no cycle left to guard against. A second
   bound here would be a second place for the two to disagree.

Nothing is lost: on every conformant file the two walks visit the same
nodes in the same order.
