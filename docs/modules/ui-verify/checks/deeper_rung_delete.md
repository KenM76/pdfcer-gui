# `ui-verify/checks/deeper_rung_delete`

`deeper_rung_delete` — **Delete removes ONE line, ONE label or ONE corner
point, and leaves the rest of the object standing.**

Four rungs, three verbs. The fourth — [`Rung::Chunk`] — is the label's verb
reached through the operator's own gesture rather than through the Points
tool, because a verb and the door to it fail separately and only a driven
gesture can tell which one is shut. Its argument is on
[`Rung::narrows_by_clicking_a_chunk_box`].


# THE ASSERTION THAT MATTERS IS "THE OTHERS SURVIVE"

Read this before changing anything below.

A check that asserted *"something was deleted"* **would pass on the exact
bug this feature was built to prevent.** `Pass 32.0`'s own words:

> *"on the operator's drawing **one text object holds all 237 dimension
> labels**, so deleting 'a label' deleted every one of them."*

Those 237 are **pdf dimensions** — page content pdfcer reads and must not
silently alter (R8b Rule 15) — and a build that routed the Part rung to
`delete_objects` removes every label on the sheet, reports success, drops
the object count by one, and traces a perfectly healthy deletion. So the
verdict here is a **pair**:

| must hold | what a wrong build does |
|---|---|
| `objects_after == objects_before` | drops by one — the whole object went |
| `parts_after == parts_before - 1` | unchanged (nothing happened) or collapses to 0 |

Either half alone is satisfiable by a wrong build. `parts_` alone is
ambiguous after a whole-object delete, because deletion **renumbers**: the
index the caller held then names a different object, and asking it for a run
count answers about whatever moved into the slot.

# The oracle

One line, written by `app::actions::vector` after each of the three verbs:

```text
delete-text-line-applied page=0 object=12 part=3 \
  objects_before=41 objects_after=41 text-lines_before=144 text-lines_after=143
```

…plus the funnel's own `delete-text-line page=0 n=9 epoch=8 disclosures=none`,
which is what says the **engine** accepted it. Both are required and they
answer different questions: the first says the page still looks right, the
second says an edit really landed. A build that computed the census and never
reached `EditSession` writes the first and not the second.

The `-applied` suffix is not decoration. `check-trace-names.py` exists
because a module line sharing its first token with a funnel label is the one
`Trace::last` returns — three recorded instances, each of which made a driven
check report *"the verb did nothing"* about a verb that worked.

# ⚠ HOW TO FALSIFY THESE CHECKS — do this before believing a PASS

A check that has never been seen to fail is not evidence. The plant, and the
proof that the plant landed, in order:


A falsification that produces a SKIP has proved nothing. If step 4 finds
`[SKIP]`, the fixture is wrong before the plant is wrong — see the table
below.

# Fixtures — pinned in [`Rung::fixture`], not passed on the command line


| check | fixture | point | why |
|---|---|---|---|
| label | `fixtures/paragraph.pdf` | `0,120,704` | needs a text object holding **several** runs; on a one-run object `delete_text_run` correctly deletes the object and the check cannot tell right from wrong. Measured by walking its content stream: one `BT`…`ET` block, **six** `Tj` operators, 12 pt, on a 612 × 792 page — so its text is also legible at fit zoom, which the A1 sheet's is not |
| line | `fixtures/hole-in-a-big-object.pdf` | `0,336,500` | needs a path object holding **several** subpaths. Measured: **41** — a circle and forty unrelated segments in ONE object, which is the shape of the operator's own export |
| point | `fixtures/polyline-nodes.pdf` | `0,150,260` | needs a subpath with **three or more** anchors — `delete_node` refuses one that would leave fewer than two, correctly. Measured: **6** |
| chunk | `fixtures/paragraph.pdf` | `0,100,704` | the **same line of the same document as the label rung**, borrowed from `crate::fixture::text_chunk_point` so the two cannot drift. The rungs differ in the door, not the target, which is the whole reason both exist |


**The line rung's fixture was WRONG in the first version of this table
and the check said so rather than passing.** It named `polyline-nodes.pdf`
at `0,150,260`; that page is one path object holding **one** subpath, so the
delete committed correctly, took the whole object with it (which is right —
a path with no subpaths is not a smaller object but a meaningless one), and
the check SKIPPED with *"the object under --doc-point held 1 line(s), and
this check needs at least 2"*. That is the fixture guard doing its job: the
discrimination this check exists for is unavailable on a one-part object,
and reporting a PASS there would have been reporting nothing.

Every one of them SKIPs rather than FAILs when it does not find what it
needs, which is the standing rule: a check that cannot establish its
precondition must not report on the property beyond it.


All three **PASS**. What the first run found, in the order it found it:

1. **The two shape rungs FAILED** — `canvas-delete-declined level=Part
   sel=1 reason=NoObjectModel`. Not *"no verb for the rung"*: the routing
   was there and the frame had simply never asked for the page's
   decomposition, because `canvas::interact` gated it on a hand-maintained
   list of **gesture outcomes** and Delete is a keystroke. Fixed by
   `canvas::modelneed`, whose header carries all four recurrences of that
   defect and the 531 ms measurement behind the fix's shape.
2. **The label rung SKIPPED, and the check was the thing that was wrong** —
   it double-clicked, and a double-click on text opens a caret by the
   operator's own ruling (O70). See [`Rung::arms_the_points_tool`] for the
   trace lines and the route that does exist.
