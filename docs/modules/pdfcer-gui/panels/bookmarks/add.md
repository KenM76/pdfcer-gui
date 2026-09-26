# `panels::bookmarks::add` — writing a bookmark, and the `/Count` trap that
comes with it

## The gap this closes


## The `/Count` trap, and why nothing here diffs a number

The engine flagged it as *"not a footnote … the entire difficulty of the
feature"*, and it is the one thing that would produce a wrong disclosure:

| | root `/Outlines` | an item |
|---|---|---|
| `/Count` counts | visible items at **every** level, including top-level | visible **descendants**, excluding itself |
| absent means | no open items | the item is a **leaf** |

On an item the **sign is the open/closed flag** — §12.3.3 defines no `/Open`
key, so the sign is the only carrier. And the consequence:

> Adding a bookmark under a **collapsed** ancestor does not change the
> document's total, because the new item is not visible.

So a surface reporting *"added N bookmarks"* by diffing the root count
reports **zero for a correct save**. Nothing here diffs anything: one call
adds one bookmark, and that is what is said.

## And the collapsed case is DISCLOSED, not merely survived

Getting the count right is the low bar. The operator's actual problem is
that they will add a bookmark under a collapsed parent, look at the panel,
and **not see it** — because it genuinely is not visible, and the panel is
correct to show it that way.

`OutlineItem::open` is read from the parent before the add, so the sentence
can be said. It is the same posture the ce-dimension group window takes
about re-measuring on a move: state the surprising consequence before the
press, not after the operator has gone looking.

## Why only the current page, and only `Fit`

Because those are the two things the engine authors today and the only ones
it authors **without refusing**. `Destination::Named` and `Remote` are
refused by name, and `DestView::Unknown` is *"the one that looks writable
and is not"* — the reader keeps an extension's fit name and discards its
parameters, so re-emitting it writes a view that is not the one the source
had.

A destination chooser offering fits pdfcer cannot write would be a control
whose options are mostly refusals, which is R9 at the level of a combo box.
The page the operator is looking at is the destination every other
page-scoped surface in this application uses, and it needs no chooser.
