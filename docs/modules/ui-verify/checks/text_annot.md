# `ui-verify/checks/text_annot`

`text_annot_places_and_authors` — a text box drawn on the page, typed into,
and actually written to the document.

# The gap this closes

The operator asked on 2026-08-18 to *"finish adding all of the standard
revisioning tools."* Three were registered, drawn on the Markup tab, and
had no dispatch arm for the whole life of the project: text box, sticky note
and stamp.

Their recorded reason was accurate — `canvas::markup`'s own table calls them
*"text-bearing, not geometric. A different gesture (place, then type) and a
different spec type"* — and building them meant a fourth `CanvasTool`
family, a `DragKind` whose release does **not** author, a dialog, two
actions and a click path.

# Why the release-does-not-author part needs driving

It is the whole difference between this family and the seven geometric
kinds, and **no unit test in the workspace can see it**. The chain is:

1. a ribbon press arms `CanvasTool::TextAnnot(kind)`;
2. a drag on the canvas rubber-bands a rectangle;
3. the release raises `BeginTextAnnot` and **authors nothing**;
4. a dialog opens and the operator types;
5. Accept raises `CommitTextAnnot`, which reaches the engine.

Step 3 is the one worth a harness. A build where the release authored
directly would pass every unit test in `canvas::textannot` — the spec
builder is pure and correct either way — and would put an **empty box** on
the operator's drawing every time they let go of the mouse.

So this check asserts, in order: the tool arms, the drag places, **the page
is unchanged at that moment**, the dialog appears, and only after typing and
accepting does the annotation count go up.

# Why the text box and not all three

It is the one that exercises every link. The sticky differs only in its
placing gesture (a click rather than a drag) and the stamp only in where its
words come from (a gallery rather than a field) — both are covered by unit
tests over predicates the production code itself branches on
(`is_dragged`, `uses_gallery`), which is the shape that cannot drift. What
neither of those can cover is the frame-level chain, and that is identical
for all three.

Stated rather than left implied, because "we drove one of three" read as
"we drove them" is exactly the kind of coverage claim this suite exists to
stop.

## Item notes

### `const BOX_PT`

Big enough that the drag is unambiguously a drag rather than a click the
gesture machine might round to one, and small enough to stay on the page
from any `--doc-point` that is itself on it.

### `fn annot_count`

Counted from the application's own `add-text-annot` trace lines — the label
`vector_edit` stamps on the commit — rather than from the file, because the
annotation has not been saved and only the session knows about it. That is
the same reason `panels::redact`'s census reads the session graph.

It counts COMMITS, which is exactly the question this check asks: "did
the release author?" and "did Accept author?" are both about whether the
funnel ran, not about what the page contains. A page census would also
answer, and would additionally move if some unrelated arm authored
something — a looser oracle for no gain.

Returns `0` when the trace cannot be read, which is safe here: every
comparison is a *difference* between two reads taken the same way, so a
build that reports nothing produces equal counts and FAILS the assertions
rather than passing them.
