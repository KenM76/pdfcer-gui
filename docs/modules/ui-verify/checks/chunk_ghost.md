# `ui-verify/checks/chunk_ghost`

`dragging_a_chunk_shows_where_it_is_going` — **O215 ask 5: dragging a line
of a note previews where it will land, and dragging several previews all of
them.**

# The request

`OPERATOR_REQUESTS.md` **O215** ask 5 asks for a live preview — *"the chunk
follows the pointer, not a rectangle."* Both halves are measured here and
on ONE gesture: the **outline** that says where the set will land, and the
translucent **copy of the line's own pixels** that says what will land
there. A build with the first and not the second meets the floor and misses
the ask — which is a real state this program has been in — and the two
trace lines separate those cases by name rather than by degree.

# The defect it was written against

`overlay::draw_move_ghost` opened with `if !outline { return; }`, and
`canvas::pressing::grabbable` sets that flag only at the **object** rung. So
at the chunk rung the ghost was withheld, `draw_selection` was gated on the
same flag so no outline was drawn either, and `painting::draw_chunks` paints
the chunk boxes at their *undisplaced* positions. The operator's experience
of dragging a line of a note was therefore that **nothing whatsoever moved**
until he let go.

★★★ The gate was not arbitrary. `OPERATOR_REQUESTS.md` **O63** is *"it just
had a perimeter box around it"* — dragging a path **node** must not draw a
perimeter box, because `ShapePreview` already shows the real anchors
travelling and a box on top of that is noise. The correct condition is
therefore *is the real geometry already travelling*, not *is this an inner
rung*. Those two coincided exactly until a text chunk became selectable, and
a text chunk is an inner rung with **no path geometry at all**.

# The oracle — and the negative that names the defect exactly

```text
fixed build   canvas-move-ghost   boxes=3 rung=part suppressed=no
              canvas-raster-ghost drawn=3 clipped=0 reason=none
no feedback   canvas-move-ghost   boxes=0 rung=part suppressed=o63
empty boxes   canvas-raster-ghost drawn=0 clipped=0 reason=geometry
```

Both were measured by driving the release binary. They are mutually
exclusive and one field apart, so a failure here quotes the defect rather
than describing it.

| field | question it answers |
|---|---|
| `boxes=` | did the painter draw anything? It is the painter's claim about itself, counted in the loop that strokes |
| `rung=` | read from `SelectionState::level`, **not** from the flag being tested — a field that restated its own gate could not witness the gate being wrong |
| `suppressed=` | `o63` names the one correct withholding; `no` is every other frame |
| `drawn=` | how many pieces of the page texture were actually blitted — the copy's own claim about itself, counted in the loop that blits |
| `reason=` | why a zero. `geometry` is the one correct one; `no-raster` is a page with no picture yet |

★★ `boxes=` is asserted against `held=` on `status-rung`, because a preview
of *one* box while *three* lines are held is the ask failing in the way the
operator would actually meet it — he sweeps three labels, drags, and watches
one of them move.

# Where this check's reach ends

It reads what the painter wrote down, not the pixels. That the outline and
the travelling copy are **visible** — not clipped away, not blitted at an
alpha that vanishes against the page, not sampled out of the wrong part of
the texture above the pixmap ceiling — has one oracle, a rendered
screenshot, and it is not this row's subject.

`clipped=` is read and deliberately **not asserted**. A cropped copy is a
correct copy, and the number moves with the region tier rather than with
this feature: asserting zero would make a change in `render::strategy` fail
a row about dragging text.

It does not drive the object rung either. `overlay::ghost_is_owed` answers
`true` whenever `outline` is, so the narrowing cannot reach the object rung
at all: that arm is additive by the shape of the boolean rather than by
measurement, and a driven assertion would be theatre. Its truth table is
pinned by that function's own test.

The `suppressed=o63` arm needs a **path node** under the pointer, which this
fixture has none of; it is listed below as a plant instead.

# Fixture — pinned, and `--pdf` is ignored

`fixtures/paragraph.pdf`, six lines of one text object, baselines 16 pt
apart. The geometry table lives in
[`crate::checks::chunk_band`]'s header, which is where the bands were
derived; [`WIDE_BAND`] below is that check's wide band verbatim, and it
reaches lines 0, 1 and 2.

⚠ A missing fixture is a **FAIL**, not a SKIP: it is committed here, so its
absence is a broken checkout rather than an unavailable precondition.

# ⚠ HOW TO FALSIFY THIS CHECK — do this before believing a PASS

**Copy each file aside first** and restore from the byte copy; never revert
with git, because this project runs parallel tracks and a chained revert
discards another track's uncommitted work.

1. **Restore the defect.** Make `overlay::ghost_is_owed` answer `outline`
   alone. Steps C and F both go red quoting
   `boxes=0 … suppressed=o63`.

   ★★★ `a_rubber_band_inside_a_note_takes_its_lines` **passed under that
   build**, and so did every other row in the roster. It asserts what the
   band selected and what the release committed, and the defect lies
   entirely between the two. That is why this row is owed and why its
   absence was not visible as a gap.
2. **Break the count without breaking the draw.** Make `boxes` in
   `draw_move_ghost` a constant `1`. Step C stays green — one line is held —
   and step F goes red on `boxes=1` where `held=3`. The two arms are
   separately falsifiable because they hold different numbers of lines.
3. **Break the rung field.** Make `part_rung` in `draw_move_ghost` read
   `!outline` instead of `selection.level()`. Every step here stays green
   today, which is the point: the two agree on this build. Then apply plant
   1 on top and step C reports `rung=object` — a defective build describing
   itself as a healthy one. Restore plant 3 before believing anything.
4. **Prove the O63 arm still suppresses.** Select a path node on a vector
   drawing and drag it; `canvas-move-ghost boxes=0 … suppressed=o63` must
   appear. This check cannot drive that — its fixture has no path — so the
   arm is guarded by `overlay`'s
   `the_ghost_is_withheld_only_for_a_preview_that_has_something_in_it`,
   whose middle row is the trap: a preview that EXISTS and is empty, which
   is what a text object produces, must not count as geometry travelling.
5. **Remove the call site.** Delete the `overlay::draw_raster_ghost(…)`
   statement in `canvas::painting::draw`. Step C goes red with no
   `canvas-raster-ghost` line anywhere in the trace, while the outline still
   travels — which is exactly the half-met state this row exists to name.
6. **Blit only the first held chunk.** Put `.take(1)` on
   `selection.outlines()` inside `overlay::draw_raster_ghost`. Step F goes
   red. ⚠ It goes red through the ABSENCE arm rather than the count arm,
   and that is not a harness defect: the planted build writes `drawn=1` in
   the plural arm, which is what it already wrote in the singular one, and
   `diag::trace_changed` emits only on a change. The absence message names
   both causes for that reason.

⚠ **The `reason=geometry` arm cannot be falsified on this fixture, and
that is a property of the fixture rather than of the arm.**
`shapes::for_move_subject` answers `None` for every text-line subject, so a
chunk drag carries NO preview at all — and the wrong spelling
`already_travelling.is_none()` therefore agrees with the right one here and
the check stays green. That arm is pinned by
`the_travelling_copy_is_withheld_only_when_the_geometry_itself_moves`, which
hands the predicate the preview this fixture cannot produce. A driven row
reaches it only once a subject that carries geometry becomes draggable at an
inner rung.
7. **Prove the plant is in the artifact.**
   pdfcer-gui` AND `-p ui-verify`, then confirm the exe is newer than the
   source: a stale binary is the commonest cause of a falsification that
   "did not reproduce", and its tell is an **absent** trace line rather than
   a wrong one.
8. **Require the `[FAIL]` line**
   PASS does.
