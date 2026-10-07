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

## Item notes

### `fn object_rung`

The page's own
paint order wins when both are present, because `delete_objects` is the verb
with the erase preview and the leaf list is the fallback for a selection
made **entirely** of form-interior targets — which is the state an ordinary
click has been able to produce since the deep hit test landed.

### `fn part_rung`

The kind decides the verb and the address space decides which one: a part of
a page object reaches `delete_subpath` or `delete_text_run`, a part of a form
leaf reaches `delete_subpath_in_form` or `delete_text_run_in_form`. `Address`
carries that second choice so the kind match is written once.

### `fn entered`

[`crate::canvas::moving`]'s `entered_entry` in every respect; kept separate
rather than made public there because the two modules' refusal enums are
different types and a shared helper would have to be generic over the error
to save four lines.

### `enum DeleteSubject`

One variant per `EditSession` delete verb this shell can address, and no
variant without one — R9, applied to a routing enum: a case that renders
nothing must not be representable as though it did something.

### `enum Refusal`

Every variant is reported **by name** on the diagnostic channel, and the
three that an operator can meet without having made a mistake carry a
sentence in [`crate::text::deleting`]. That split is the whole design: a bar
that narrates the obvious stops being read, and a program that says nothing
when a key does nothing is the founding defect of this project.

### `fn subject`

Asked from exactly two places — `canvas::keys`' Delete/Backspace and
`app::dispatch::format`'s `format.delete` — because a destructive rule
stated twice is a rule that drifts, and the drift here removes a drawing
view instead of a line.

# Why `provider` is an `Option` and the Object rung does not need it

The Object rung's operand list comes from the selection alone: an entry
already holds a resolved `TargetId`, and `object_indices_on` is a filter
over four integers. The deeper rungs need the object model to answer *what
kind of part is this* — a subpath and a show operator wear the same
`subpath: Some(n)` field on [`Selection`] and reach different verbs — so
they and only they decline [`Refusal::NoObjectModel`] when it is absent.

Making the whole function require a model would have made a page that
cannot decompose un-deletable at the rung where deletion needs no
decomposition at all, which is a limit invented by a signature rather than
by a fact.

### `fn action`

A separate function from [`subject`] for [`crate::canvas::moving::action`]'s
reason: the decision is the part worth unit-testing exhaustively, and the
translation is a five-arm match that cannot fail. Keeping them apart is also
what lets the two call sites share the decision and differ in what they do
with it — the ribbon's arm holds the erase preview, the key's does not.

Infallible. Every variant of [`DeleteSubject`] names a verb this shell
calls; there is no arm that can decline here, which is R9 read as a type:
a routing enum must not be able to represent a case that renders nothing.

### `fn decline`

# Which refusals get a sentence, and the rule behind the split

Four do, and they are the four an operator meets **without having made a
mistake**:

* [`Refusal::RunWouldMoveNext`] — they picked a label, pressed Delete, and
  the file's own structure forbids it. There is a remedy and it always
  works.
* [`Refusal::ManyNodes`] — they Shift-clicked four anchors and watched four
  highlight. Removing one silently would be worse than refusing.
* [`Refusal::ManyLines`] — the same press one rung up, and the operator has
  more reason to be surprised: the identical set can be dragged as one.
* [`Refusal::NoObjectModel`] — the page will not decompose, so nothing
  inside an object can be named. The Object rung still works and the
  sentence says so.

The rest describe states the operator put themselves in and can see —
nothing selected, an image with no parts, the Node rung on a line of text —
and `moving::decline`'s argument applies unchanged: *a surface that narrates
the obvious stops being read*.

# Why the sentence travels as a note and not as a decline

`app::status::decline` is written by the one dispatcher and read by the one
bar; `record_notes` is the channel already used for *"a limit with nowhere
else to be said"* — `canvas::interact` raises one from the canvas when a
caret cannot be placed, which is the identical shape: no edit happened, no
epoch moved, and the operator is owed a sentence anyway. The epoch passed is
the **current** one, so the sentence stands until the next real edit moves
past it, which is what retires it without anything having to remember to.
# `model_attempted`, and why a refusal carries how it was reached

[`Refusal::NoObjectModel`] is raised for two causes that look identical from
here: the page genuinely would not decompose, or **this frame never asked**
for the decomposition. The second is a defect that has shipped four times
(see `canvas::modelneed`), and for one commit it was reported in the first's
words — `reason=NoObjectModel`, with nothing to say which.

So the flag travels onto the trace as `asked=`. A `debug_assert` at
`canvas::keys`' call site turns the bad case into a panic under test; this
is the half that survives into a release build, where a driven check reads
it. **It is not shown to the operator** — from their chair both causes are
the same event and both are answered by the same sentence.
