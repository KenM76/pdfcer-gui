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

## Item notes

### `const CHUNK_AIM`

The first line, which is also [`Rung::Label`]'s target — so the two doors
are measured against the same line of the same document and a difference
between them is a difference in the route and nothing else.

### `enum Rung`

One enum rather than three copies of `drive`, because the three differ in
exactly four things — how deep to descend, which `-applied` label to read,
which count field carries the parts, and what to say when the fixture cannot
exercise it — and everything else is one sequence. Three copies would be
three places for the census pair to drift.

### `fn unit`

The text rung's prefix is `text-lines` and not `lines`, because
`Rung::Line` — a subpath — already owns `lines`, and two rungs writing
one field name into one trace is how a check comes to read the wrong
census and report a pass.

### `fn stem`

⇒ **An artefact path must be a function of the check, never of the
module.** `tools/gates/check-artifact-paths.sh` enforces that no two
roster entries can collide on one, because a rule described in prose is
a rule that will be approximated.

### `fn descents`

One double-click enters the Part rung; a second descends to the Node
rung. `canvas::selection::descend` is the rule.

**Zero for the label, and that is a fact about the program rather
than a shortcut.** See [`Self::arms_the_points_tool`].

Zero for the chunk too, for the same underlying reason and by a
different route: its descent is a second **plain** click, not a
double-click, so it is counted by
[`Self::narrows_by_clicking_a_chunk_box`] rather than here. A
double-click anywhere in this rung's sequence would open the caret and
the ladder would never move.

### `fn narrows_by_clicking_a_chunk_box`

# Why a second row for a verb that already has one

[`Self::Label`] and [`Self::Chunk`] end at the same verb, on the same
fixture, on the same line of it. Everything between the pointer and
that verb is different:

| | label | chunk |
|---|---|---|
| preparation | arm the Points tool (chord `A`) | switch the chunk boxes on |
| gesture | one click | click the block, then click a box |
| what the operator was told | nothing — the tool is not advertised for text | O215 built the boxes so he could aim at them |

So a build can pass `label` and leave O216 ask 2 unmet, and that is not
hypothetical: it is the state this row was added into. **A capability
the operator cannot reach is not a capability**, and the only
instrument that can tell the two apart is a driven gesture.

# The sequence, and why each step is where it is

1. **Boxes on, before the first click.** The chunk rung is offered
   exactly where a box is drawn — `canvas::clicking` sets
   `ClickHit::chunk` from `chunks::boxed`, which reads the preference.
   The preference is persisted beside the exe, so a previous run that
   left it off is a fact about the machine and not about the build, and
   a run that began with it off would measure the switch and file the
   result as a delete defect.
2. **First click: the block.** Not asserted here —
   `clicking_a_chunk_selects_that_chunk` owns that rule, including the
   R6 half that a build descending on first contact has made dragging a
   whole block unreachable.
3. **Second click, same point, plain left button: one chunk.** The part
   index comes from `ClickHit::part`, the same probe the Points tool
   reads, so the two doors select the identical unit and this row's
   census is comparable with the label row's.

# ⚠ Where this row's reach ends

The census says *one text line went and the block survived*. It cannot
say *the line under the pointer* went: both halves of the pair are
counts, and the `part=` on the applied line is the selection's own
number passed through, so comparing them would assert nothing. Which
line was removed has one oracle, a rendered page, and it is not this
check's subject.

### `fn needs_parts`

**Two, not one, and for the Point rung three.** On a one-part object
every one of these verbs correctly deletes the whole object — a painting
operator with no path, or a `BT`…`ET` that shows nothing, is not a
smaller object but a meaningless one — so a right build and a wrong build
produce the *identical* census and the check has no discrimination at
all. `delete_node` additionally refuses a subpath that would be left with
fewer than two anchors, so its floor is three.

### `fn fixture`

⇒ **Knowledge a check cannot run without belongs in the check.** A
fixture table in a doc comment is a note to a human about to type a
command line. It is not a precondition, and against a runner that does
not read doc comments it is not even a note.

# The three, and what was measured about each


⚠ A rung whose fixture is missing must FAIL, not SKIP. All three are
committed to this repository; an absent one is a broken checkout, not
an unavailable precondition, and a SKIP would say the opposite.

### `fn drive`

The three-way return is the SKIP/FAIL/PASS rule made structural: `Err` is a
precondition that was absent (SKIP), `Ok(Some(_))` is an assertion that did
not hold (FAIL), `Ok(None)` is a pass.
