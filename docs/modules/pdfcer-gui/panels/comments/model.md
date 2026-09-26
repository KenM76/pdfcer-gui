# `panels::comments::model` — turning a document into a comment list

The whole of the Comments panel that is not drawing. [`collect`] walks the
session's pages, applies the exclusion rule, classifies what survives, and
hands back a [`Listing`] the body renders row for row. Nothing here touches
`egui`.

## Why the panel is split this way

Because every interesting decision this panel makes is a *classification*,
and a classification is only testable if it is separable from the widget
that shows it. The list of things that have to be right —

- which annotations are excluded, and how many of each,
- whether a `/Line` is a **ce dimension** or a genuine `/Line` markup,
- whether `/Contents` is a note or an accessibility description,
- whether the row is a reply, a group subordinate, neither, or something
  `/RT` named that pdfcer has never heard of,
- whether the annotation is suppressed on screen,
- whether its appearance state could be resolved,
- and the ordering of the whole thing

— is exactly the list `tests` below sweeps against real engine fixtures.
`crate::panels::objects` is split on the same seam and for the same reason
(`provider.rs` and `summary.rs` beside its `mod.rs`), and this module's
`Vec<CommentRow>` is that pattern at a much smaller scale.

## The ordering is `pdfcer list-annotations`', reused by name

**Page order, then `/Annots`-array order.** Reused rather than reinvented:
a second, GUI-only ordering rule could disagree with the CLI's, and an
operator comparing a panel against a command's output on the same file
would have no way to tell which of the two had drifted. `/Annots` order is
whatever [`pdfcer_core::annot::page_annotations`] returns, which is the
array order the file itself carries — not a sort pdfcer imposes.

There is deliberately **no sort by date**. `/M` is stored raw because
§12.5.2 makes it *"date or text string"* and requires a reader to accept
any format, so ordering by it would mean parsing a value the standard says
may not parse — and any such feature owns that decision itself rather than
inheriting it from a list nobody asked to be sorted.

## Read the SESSION, not the file on disk

[`collect`] takes an [`ObjectGraph`], and the body hands it
`doc.session.view()` — the base revision with **every unsaved edit
applied**, which is the same thing the canvas rasterizes. An operator who
has just drawn three shapes must see three rows without saving first.
`crate::panels::forms`' body carries the same rule and the same sentence.
