# `viewer::remembered` — the page-display choice, per document, on disk


> *"Mode persists **per document**, not globally — opening a drawing set
> must not inherit a report's setting."*

One file, `page-display.txt`, holding one line per document. Nothing else
is stored and nothing else should be.

## ★ Why this is a third file rather than a field in one of the two

`PROJECT_PLAN.md`'s brief for this work named the two existing stores and
asked which fits. Neither does, and both say so themselves:

| store | what it is | lifetime | verdict |
|---|---|---|---|
| [`crate::app::persistence`] `layout.ron` | the dock arrangement and named workspaces | **per operator**, outlives every document | wrong axis entirely — one layout serves all documents, and a per-document key in it would make the arrangement a document property |
| [`crate::app::recent`] `recent.txt` | the ten documents most recently opened | per operator, keyed by document | *nearly* right, and explicitly refused |

`recent.rs`'s own header settles the second, in a section headed *"What is
deliberately NOT here"*:

> **No per-document view state** (last page, last zoom). That is a
> different feature with a different lifetime and a different file; a
> recent list that quietly became a session store would be the thing
> nobody could later separate.

That paragraph was written before there was any per-document view state to
store. There is now, and the conclusion it reached is the one taken here:
**a different file.** Three concrete consequences follow, and each is a
reason rather than a restatement:

1. **The lifetimes genuinely differ.** The recent list is capped at ten
   because it is *drawn* — a menu taller than ten rows is a scroll view.
   This list is never drawn, so its cap is about disk and nothing else, and
   [`CAP`] is twenty times larger. Sharing a file would force one cap to
   serve two purposes, and the drawn one would win.
2. **Forgetting means different things.** A "clear recent files" command
   (in `PLANNED`) must not silently reset every document's display mode,
   and a document whose remembered mode is evicted must not vanish from the
   recent menu. Two files make that true by construction rather than by a
   rule somebody has to honour.
3. **The format cannot serve both.** `recent.txt` is *"one path per line,
   newest first … nothing else is legal, so nothing else has to be
   parsed"*. Adding a field to it makes every line ambiguous with the
   format it replaced, and a half-upgraded file would read old paths as
   mode ids.

## ★ Why it lives in `viewer/` rather than beside the other two in `app/`

Because what it persists is [`PageDisplay`], and the on-disk spelling of
that enum is [`PageDisplay::id`]. Keeping the reader and the writer beside
the type means a variant added to the enum without a spelling is a
compile error and a variant added with a colliding spelling is a test
failure in the same module — see
`viewer::display::tests::every_mode_round_trips_through_its_on_disk_spelling`.
Put the store in `app/` and the enum's spelling has two homes, which is the
shape of every drift this project writes headers about.

The other two stores are in `app/` because what they persist —
`egui_shell::layout::LayoutDocument` and a list of paths — has no module of
its own to sit in.

## The format

```text
continuous\tD:\Drawings\job-4471\sheet-set.pdf
single\tC:\Users\ken\Documents\report.pdf
```

One line per document, **most recently written first**, UTF-8, no header
and no comments. The separator is a **tab**, chosen because it is the one
ASCII character a Windows path cannot contain (`< > : " / \ | ? *` and the
control range are all reserved) and because it needs no escaping. A line
with no tab, an unknown mode id, or an empty path is **dropped** — a
corrupt file degrades into a shorter list, exactly as `recent.txt` does,
rather than into an error the operator has to dismiss about a preference.

Like `recent.txt`, this is a flat text file rather than RON because
**this crate cannot serialize**: `serde` and `ron` are dependencies of
`egui-shell`, not of `pdfcer-gui`, and `Cargo.toml` is not this work's to
edit.

## ★ Why there is no in-memory store held on the application

Because there is nothing to hold. The file is read **once per document
open** and written **once per mode change** — two of the rarest events in
the application, against a file of at most [`CAP`] short lines. A cached
copy would buy a few microseconds on events that happen seconds apart, and
would cost a field on `PdfcerApp`, a load at start-up, and a staleness
question ("what if another pdfcer window wrote it?") that not caching
answers for free: **two windows open on two documents each write their own
line and read the other's**, because every write is a read-modify-write of
the whole file.

The honest limit of that: two windows changing the mode at the same instant
can lose one of the two writes. The loss is one document's display
preference, the window that lost it still shows what the operator chose,
and the next change writes it again. A lock file would be a larger
mechanism than the thing it protects.
