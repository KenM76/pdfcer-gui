# `ui-verify/checks/chunk_multi_move`

`shift_click_builds_a_chunk_set_the_whole_program_honours` — **O215 ask 4,
driven: several lines of one text block are selected together, move
together, and come back together on one press of undo.**

# The request

`OPERATOR_REQUESTS.md` **O215** ask 4 — *multi-select*. The operator holds
four labels of a note and expects the next drag to take all four, the way
every drawing program he uses takes them.

# The two halves, and why a check that drives only the first is worthless

A multi-chunk selection was **representable and reachable** before any of
this: `SelectionState::pick_within` has pushed a Shift-clicked part in as
its own entry since the Part rung landed, and `normalise` collapses to the
Object rung only when entries differ by object or page. So the set could be
built by hand and the outlines drew. What no consumer did was *read* it —
the drag moved the first entry, and the status line said *1 line of 6* over
four outlined lines.

⇒ So this check is in two halves, and the second is the one with teeth:

| half | gesture | what it would catch |
|---|---|---|
| the set is built | Shift-click a second chunk | `pick_within` replacing instead of pushing |
| the set is **honoured** | drag it, then undo it | a consumer reading `entries[0]` and moving one line of four |

# The gap press — the assertion the narrowing exists for

Step G presses on the line **between** the two selected ones, which is not
selected, and drags.

`Grabbable::bounds` at the Part rung is the union of the selected chunks'
outlines, so a set built from lines 0 and 2 spans line 1. A `covers`
predicate that asks only *"is the topmost object one of mine?"* answers yes
for a press anywhere in that span — the whole note is one object — so the
press is claimed as a move of the set and `presspick` never gets to re-pick
the line the operator aimed at. The operator's experience of that build:
he aims at line 1, drags, and lines 0 and 2 move while line 1 stays put.

That is **O215 ask 1 failing through a multi-chunk selection**, and no other
step here can see it: every other press in this check lands on a chunk that
is already held, where the old predicate and the new one agree.

# The oracle — four subsystems, and none of them is the canvas agreeing
with itself

```text
canvas      canvas-selection via=click mod=true sel=2 level=Part first=object:0 part=0
status bar  status-rung kind=text part=0 held=2 of=6
apply phase move-text-lines page=0 n=2 epoch=1 disclosures=none
history     undo kind=MoveTextRun undo_depth=1
press pick  selection-set page=0 object=0 part=1 level=part via=press
```

| line | question it answers |
|---|---|
| `canvas-selection mod=` | did the modifier reach the application, or was it a plain click? |
| `canvas-selection sel=` | did the Shift-click **add** rather than replace? |
| `status-rung held=` | does the surface that tells him what he is holding say **2**? |
| `move-text-lines` | did the PLURAL verb run — a singular one here is a set that moved one line |
| `undo … undo_depth=1` | did N engine calls fold into **one** undo entry? |
| `selection-set … via=press` | did the gap press re-pick the line under it? |

The last row is on a **different channel** from the first, and it has to
be. `canvas-selection` is written by the click path, which runs on the
release; a chunk chosen on the press and then dragged never produces a click
and never appears there. Reading the gap press off `canvas-selection` sees
silence, and reports the fix as missing on the build that has it.

`move-text-lines` and `move-text-line` are distinct event names and the
trace matches them exactly, so *"the set moved"* and *"the first entry
moved"* cannot be confused. A build that collapsed the set mid-gesture
writes the singular line, and this check quotes it.

`undo_depth=` is read on the press **before** the pop, so it is the
depth the operator is acting on. Step C establishes that the log is empty
before the drag — without that control, `undo_depth=1` would be satisfied
by a build that left one entry of four behind and had four on the log all
along.

# Fixture — pinned, and `--pdf` is ignored

`fixtures/paragraph.pdf` through [`crate::fixture::text_chunk_point`]: one
text object of six lines on baselines 16 pt apart, every one of them
stating its own position — so the engine refuses none of them and a refusal
anywhere in this run is this shell's.

⚠ A missing fixture is a **FAIL**, not a SKIP: it is committed here, so its
absence is a broken checkout.

# ⚠ What this check can see, and where its reach ends

It reads what four subsystems wrote down, not the pixels. That both lines
*drew* an outline has one oracle — a rendered screenshot — and it is
`text_chunks`' subject rather than this one's.

That the ghost travelled over both while the drag was in flight is
[`crate::checks::chunk_ghost`]'s, which reads the painter's own count of
the outlines it stroked. It is a separate row because the defect it names
lies entirely between what this check asserts — the set that was built, and
the move the release committed — and passed here undetected.

# ⚠ HOW TO FALSIFY THIS CHECK — do this before believing a PASS

Three plants, one per half. **Copy each file aside first** and restore from
the byte copy; never revert with git, because this project runs parallel
tracks and a chained revert discards another track's uncommitted work.

1. **The set is not built.** In `SelectionState::pick_within`, make the
   `shift` arm assign `self.entries = vec![entry]` like the plain one. Step
   D goes red on `sel=1`, and everything after it is unreachable.
2. **The set is built and not honoured.** In `canvas::moving::eligible`,
   delete the `None if lines.len() > 1` arm of the page-object text branch.
   Step D still passes; step E goes red naming `move-text-line` — the
   singular verb over a selection of two, which is the defect in one line.
3. **The narrowing.** In `canvas::pressing::body_under`, return `true` as
   soon as the topmost hit is one of the selection's objects — i.e. delete
   the Part-rung chunk test. Steps A–F stay green, because every press in
   them is on a held chunk. Step G goes red: the press in the gap is
   claimed as a move of the set, so the trace carries `move-text-lines`
   where a singular move of the aimed-at line was required.
4. **Prove the plant is in the artifact.** `cargo build --release -p
   pdfcer-gui`, then confirm the exe is newer than the source: a stale
   binary is the commonest cause of a falsification that "did not
   reproduce", and its tell is an **absent** trace line rather than a wrong
   one.
5. **Require the `[FAIL]` line**, not the exit code — a SKIP exits the way a
   PASS does.
