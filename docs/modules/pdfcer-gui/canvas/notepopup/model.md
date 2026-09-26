# `canvas::notepopup::model` — what a note says, and where its window goes

The **pure** half of the note pop-up: it reads the document and answers
three questions, and it draws nothing, stores nothing and decides nothing
about the interface.

| question | answer |
|---|---|
| *what notes are on this page, and where?* | [`notes_on`] |
| *which one is under the pointer?* | [`under`] |
| *what replies hang off this one?* | [`replies_to`] |

It is separate from [`super`] for the reason every `model` module in this
crate is: a `Ui` cannot be driven from a unit test here, so anything that
needs a `Ui` is untestable by construction. Everything in this file takes an
[`ObjectGraph`] and a [`Page`] and returns data, which means every rule it
states can be asserted.


> *"I could add a yellow sticky note but even in read mode I don't think I
> could figure out how to read it."*

He was right, and the measurement was worse than the report. Before this
module the **only** route to a note's `/Contents` in the whole shell was
the Comments panel, which is mounted from the `markup` tab — and
`crate::app::modes::defaults`' `"read"` arm gives Read the tab list
`["file", "view"]`. So in Read mode there was **no route at all**, which is
the posture exactly backwards: Acrobat *Reader* is a read-only product and
reading comments is its whole purpose.

A pop-up on the canvas is the fix that cannot regress that way, because it
is **canvas behaviour rather than a ribbon item** — it is mode-independent
by construction, and no future tab-list edit can take it away.

## What the file already contained, and what it did not

`pdfcer-core`'s sticky author — the private `sticky_note` behind
`annot_author::TextAnnotSpec::Sticky` — writes a `/Popup` companion for
every sticky note it authors, carrying the note's own `/Open` state and a
rectangle 150 pt wide placed to the right of the note. So the data is in
the operator's files already; this module is what draws it.

### `/Open` is READ from the file, never defaulted

§12.5.6.4 Table 172 gives `/Open` on a `/Text` annotation as *"a flag
specifying whether the annotation shall initially be displayed open"*, and
§12.5.6.14 Table 183 gives the same key the same meaning on the `/Popup`.
A note authored open must therefore **open on load**, with no click.

**The workaround that was here is GONE — 2026-09-06.** This paragraph
read: *"`pdfcer_core::annot::Annotation` does not model `/Open`. Confirmed
by audit on 2026-09-05: `b"Open"` appears exactly twice in the whole crate,
both write sites in `annot_author.rs`. So this module reads the raw
dictionary through `ObjectGraph::value`."* It was reported as a workaround
under pdfcer decision 058 — *anything the GUI has to work around is a place
the crate boundary was drawn wrong* — and filed as
`request_popup_open_state_cannot_be_read.md`.

`Pass 253.3` shipped [`pdfcer_core::annot::Annotation::open`], and
[`read_open`] is now two field reads. ⇒ **The shell's copy was deleted the
day the engine's existed**, which is the whole point of having reported it:
the request's own words were *"the day `Annotation` grows the field, two
places will answer the same question and one of them will be ours"*, and
the only way that never happens is to remove ours immediately rather than
leaving it beside the real one as a fallback nobody re-reads.

⚠ The `/Popup` companion's `/Open` is read from the **same walk** rather
than by a second lookup: [`notes_on`] already gathers every `/Popup` on the
page to find its rectangle, and `page_annotations` returns each one with its
own `open`. Table 170 gives geometric markup no `/Open` of its own, so for a
`/Square` the companion is the entire answer.

## Where a pop-up is drawn, in priority order

1. **The `/Popup`'s own `/Rect`**, when it has one. The file said where the
   window goes and honouring it is what makes a document look the same here
   as it does in the reader that wrote it.
2. **Beside the note**, when there is no `/Popup` or its rect is unusable —
   to the right, top-aligned, which is where every reader in the class puts
   one and where `pdfcer-core`'s own author places it.

[`PopupBox::rect`] is `None` in case 2 and [`super`] does the placing,
because case 2 needs the *drawn* size of a window this module cannot see.

## Rule 15: a ce dimension is a `/Line` and it is NOT excluded here

`crate::panels::comments`' header settles this and the argument is not
re-derived: **ce dimensions** (the ones pdfcer authors, `/Line` with `/IT
/LineDimension` and a `/PieceInfo` sidecar) are annotations on the
document, so hiding them by subtype would also hide a genuine `/Line`
markup an operator drew.

What *is* different here is that a ce dimension's `/Contents` is
**regenerated from its measurement** by `author_dimension`, so it is never
a note somebody wrote. [`NoteView::contents`] carries whatever the file
says and [`super`] captions it; this module does not filter on it.

## Cost, stated rather than discovered

[`notes_on`] walks one page's `/Annots` — one array read plus one
dictionary per entry, bounded by `pdfcer_core::annot::MAX_ANNOTS_PER_PAGE`,
decomposing nothing. It is the same walk
`crate::canvas::selection::annot::under_pointer` already pays per click.

[`replies_to`] walks **every page**, because §12.5.6.2 permits a reply to
live on a different page from the comment it replies to and `pdfcer-core`'s
own `locate_annotation` scans every page for exactly that reason. It is
called only while a pop-up is open, never otherwise.
`crate::panels::comments` pays the same walk every frame it is visible and
states so in its own header; this is the same bound with a stricter gate.
