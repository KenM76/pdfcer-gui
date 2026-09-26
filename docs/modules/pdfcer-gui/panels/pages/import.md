# `panels::pages::import` — **a drawing dropped on the thumbnails becomes
# pages in this one**


> *"I should be able to drag and drop documents into the thumbnails section
> of another pdf to import the pages."*

This is the Pages panel's half of [`crate::app::filedrag`]'s claim
protocol: the panel resolves the gap under the pointer with the same code
that resolves it for a page drag, and this module decides whether the file
that landed there is one it can act on.

## Every refusal here is a FALL-THROUGH, never a message

There are four reasons this module declines, and not one of them tells the
operator anything:

| it declines when | and then |
|---|---|
| the drop was not on this panel | the file opens, or inserts, as always |
| the platform gave no cursor position | the file opens — the position-blind behaviour |
| the file is **not a PDF** | an image dropped here still goes to the placement window |
| the file has **no readable pages** | the file opens, and the *parser* says why |

That last row is the one worth stating plainly. A corrupt or encrypted file
dropped on the thumbnails produces `pdfcer-core`'s own error, which names the
actual problem, rather than this module's guess at one — the same argument
`app::dropped::classify` makes for not sniffing bytes it is about to hand to
a parser that will.

⇒ **Declining costs a feature and never a file.** That asymmetry is the
whole reason the claim protocol exists rather than this module deciding what
every drop means.

## Why the whole panel accepts, not only the tiles

The operator wrote *"into the thumbnails section"* — a region, not a
target. A drop on the grid's empty space below the last row, or on the
*Previews* checkbox, is unambiguously aimed at this panel, and refusing it
because it missed a tile by four points would teach that drops work
*sometimes*, which `app::dropped`'s header already names as worse than
never.

So the panel's rectangle accepts, and the **gap** comes from the tile under
the pointer when there is one and from the end of the document when there
is not. That is also the conventional answer: a file dropped past the last
page goes after the last page.

## Several files at once, and why the positions are computed up front

*"documents"*, plural. Each file becomes its own
[`PageAction::InsertPagesFromFile`] — one undoable command each, which is
what the engine's `insert_pages` records — and their positions are worked
out **before** any of them is applied:

```text
A (3 pages) and B (2 pages) dropped in the gap before page 5
  A → Before(5)
  B → Before(8)          5 + 3, because A's pages are in front of B's by then
```

Deriving each position from the live page count at apply time would be the
obvious alternative and is wrong in a way that is hard to see: the actions
are applied in sequence within one frame, so the second would read a count
that already includes the first, and the two files would interleave.

## Item notes

### `struct Source`

The count is read **once**, here, and then used twice — for the page list
and for the next file's position. Reading it twice would be two parses of
the same file with no guarantee they agreed.

### `fn readable_documents`

# Why the page count is read here rather than at apply time

Because a file with **no** readable pages must not be claimed at all — it
has to reach `Action::Open` so the parser can say what is wrong with it —
and "does this file have pages?" cannot be answered without opening it.
`dialogs::insert_pages` opens the file for the same reason and says the same
thing about the cost: *"the load is cheap relative to what follows"*, since
the insert itself opens it again anyway.

### `fn page_count_of`

`0` for every failure, deliberately: the caller's only question is *"can
this be imported?"*, and a `Result` here would make three different
unreadable-ness reasons into three call-site branches that all end in the
same fall-through.

### `fn a_landing_with_no_position_is_not_claimed`

The platform gives no cursor position on a locked workstation, and the
tempting default — the panel's own centre, or the end of the document —
would import pages at a place nobody pointed at. Falling through opens
the file instead, which is visible and undoable.

### `fn an_unreadable_document_is_left_for_the_parser_to_explain`

The path does not exist, so it cannot be read — the same outcome as a
corrupt file, and the one that matters: the drop is NOT claimed, so
`app::dropped` still runs and the operator gets the parser's own
sentence rather than silence.

### `fn several_files_stack_in_the_order_they_were_dropped`

The arithmetic from the module header, asserted without touching the
disk: A has three pages and goes in the gap before page 5, so B's own
position has to be 8 rather than 5. Deriving each position at apply
time would produce 5 twice and interleave the two documents.

### `fn claim`

`panel` is the panel body's rectangle in screen points, `gap` the boundary
the grid resolved under the pointer (`None` when the pointer was over no
tile — see the module header), and `page_count` this document's length.

Returns `true` when the drop was claimed, which the caller does not need but
a test does: it is the difference between *"this panel acted"* and *"the
fallback will"*, and that is the property worth asserting.
