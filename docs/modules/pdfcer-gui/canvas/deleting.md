# `canvas::deleting` — **which delete verb the rung the operator is on reaches**

The delete twin of [`crate::canvas::moving::eligible`], and it exists for
the same reason that one does: a selection ladder with three rungs addresses
three different things, `pdfcer-core` has a different verb for each, and the
decision about *which* must be made in one pure function that both the
keyboard and the ribbon ask — or the key and the command act on different
things, which `app::keyboard`'s header calls the defect the single
dispatcher exists to make impossible.

## What this closes, and why it is one defect three times


```text
canvas-delete-declined level=Part reason=no-verb-for-rung
```

…and that trace line was the whole of the response. Nothing on screen, no
sentence, no sound. The operator's own words about the same shape of failure
elsewhere: *"nothing happens"*.

The refusal was honest when it was written — `SelectionState::deletable_
objects_on` carries the argument, and it is a good one: at the Part rung the
selection names one subpath or one label *inside* an object, while
`delete_objects` removes **whole objects**, and on the measured CAD export
one path object holds **1,194 subpaths** and one text object holds **all 237
pdf dimension labels**. Borrowing the Object rung's verb would delete a whole
drawing view because the operator asked to remove one line of it.

What was wrong is that the engine had shipped the three verbs that do the
right thing and nothing here called them:

| rung | what is selected | verb |
|---|---|---|
| Part, on a **path** | one subpath | `EditSession::delete_subpath` (Pass 25.2) |
| Part, on **text** | one visual line — one label | `EditSession::delete_text_run`, once per fragment |
| Node | one anchor | `EditSession::delete_node` (Pass 36.1) |

Each of the three had its **move** twin wired — `move_subpath` through
`VectorAction::MoveSubpath`, `move_node` / `move_nodes` through
`VectorAction::MoveNode` / `MoveNodes` — so for a fortnight a line could be
entered, selected and **dragged**, and could not be removed.

## The vocabulary, because getting it wrong here is a documented cost

**R8b Rule 15.** The 237 labels on the operator's SolidWorks export are
**pdf dimensions** — page content pdfcer reads and must not silently alter.
A **ce dimension** is the thing pdfcer itself authors, and this module never
touches one: `Action::Dimension` is a different family entirely. Nothing
here is ever a bare "dimension".

## Why this is pure, and what it deliberately cannot do

No egui, no pointer, no document, no `&mut` anything. It takes the selection
and the page's object model and returns either the one verb to raise or the
one reason none applies. That is what lets every rule below be a unit test
rather than something to be hoped for in a running window — and it is what
lets `canvas::keys` and `app::dispatch::format` ask the identical question,
which is the property the ribbon's Delete had already been found to have
lost once.

## R83: a refusal the model can see BEFORE the press is a refusal that
names its remedy

One refusal here is asked ahead of the verb rather than reported after it:
[`Refusal::RunWouldMoveNext`]. `ObjectModelProvider::text_line_delete_would_
move_next` answers from the same `positioned_by` flag
`EditSession::delete_text_run` refuses on (§9.4.2 — a following run with no
positioning operator of its own starts wherever this one ends, so removing
this one would slide it). Both answers are therefore the same answer, and
asking first buys the one thing the engine's own refusal cannot deliver to
an operator: **the remedy**. `EditError` is `Display` output and
`check-ui-strings.sh` exclusion 3 forbids routing it to a surface, so a
refusal that arrives from the engine reaches the operator as
`text::status::edit_declined_by_engine`, which by design names no cause.
Asked here, the sentence can say *delete the later label first*, which is
true, always works, and is what they need.

Every other refusal below is left to the engine, deliberately: it is the
engine's judgement, it is made against the bytes rather than against a
model, and `apply::vector_edit`'s `Err` arm already routes it to the decline
channel with a worded sentence (`OPERATOR_REQUESTS.md` O116). Duplicating
those predicates here would be a second statement of a destructive rule,
which is the drift `deletable_objects_on`'s own header refuses.
