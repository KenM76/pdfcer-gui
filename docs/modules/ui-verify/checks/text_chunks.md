# `ui-verify/checks/text_chunks`

`chunk_boxes_show_what_a_text_block_is_made_of` — **the boxes O215 asks
for, driven: they appear on a click, the switch turns them off, and the
switch turns them back on.**

# The request

`OPERATOR_REQUESTS.md` **O215**, ask 3, in his words:

> *"Ideally we'd have a way to click on a text block and it would show us
> boxes around all the blocks contained within it, then let us use our usual
> mouse selection methods to move the chunks."*

— with the switch his own sentence asks for: *"a new selector option we can
turn on or off in the sidebar, navigate, and content edit tools."*

**The boxes are not decoration and the row records why.** Selection
already descends to the chunk; what decides whether a drag moves the chunk
or the whole block is `canvas::presspick::covers`, asking on **press**
whether the pointer is inside the current selection's outline — a rectangle
nothing draws. So the operator is aiming at an invisible target, and *"it
sometimes takes the whole block"* is what aiming at an invisible target
feels like. Drawing the boxes is what makes ask 1 — a repeatable gesture —
possible at all.

# Why a unit test cannot stand in for this

`canvas::chunks::tests` asserts the switch remembers its answer and that the
decline reasons are distinct. Both are true and neither is evidence (**R1**):
they call the functions. Four things sit between them and the operator, and
every one of them has been the defect in this project before —

| link | what could be wrong with a green unit suite |
|---|---|
| the painter calls `chunks::outlines` at all | a call site that was never added |
| the ribbon declares the toggle | a registered command with no item naming it |
| the dispatch arm runs | an id in `handles` with no `match` arm |
| the painter honours the switch | the state read from the persisted home, a restart behind |

# The oracle — three subsystems, per press

```text
shell    ribbon-command-invoked id=view.text_chunks     the pointer reached the control
app      text-chunks enabled=false                      the dispatch arm ran
app      canvas-chunks-declined reason=switched-off     the painter honoured it
```

Each line is written by a different subsystem, separated by the exact
boundaries the wiring crosses. A build whose ribbon item is missing writes
none of them; one whose dispatch arm is missing writes the first only; one
that reads the persisted home rather than the live one writes the first two
and not the third — and that third failure is invisible until tomorrow,
which is why it is asserted today.

# Reading the chunk state: anchored after a gesture, unanchored before

`canvas-chunks` and `canvas-chunks-declined` are written through
`diag::trace_changed` under **one** slot, so the channel is a change log:
identical repeats collapse and an alternation survives. Two consequences,
and this check depends on both.

- **After a gesture**, read with `Trace::last_after` anchored on a mark
  taken before it. Every step below changes the state, so an anchored `None`
  means *the painter said nothing since*, which is a different verdict from
  *it said the same thing* and is reported differently.
- **Before the first gesture**, read the whole capture. The painter runs
  every frame from launch, so the newest of the two lines **is** the current
  state — this is the one case where `last` is right rather than a fossil.

# Why it reads the state before it starts

The toggle is **persisted**, and the preference lives beside the exe:
`settings::resolve_store` prefers a writable `userdata/` next to
`current_exe`. So *which* directory the launched process resolves decides
what this check starts from.

Under the default isolation it is a fresh one — [`crate::sandbox::Sandbox`]
removes and recreates a per-check directory on every run — so the starting
state is the shipped default and step A finds the boxes already on. Under
`--shared-profile`, or a hand run pointed straight at a real `--exe`, it is
whatever the previous run left: a run that ended with the boxes off would
make the next run's first assertion fail for a reason with nothing to do
with the build.

So the starting state is **read rather than assumed**, turned on with a
note if it is found off, and left **on** however this check ends. That costs
one trace read in the common case and removes an entire class of articulate
failure about the wrong subject in the other.

# Fixture — pinned, and `--pdf` is ignored

`fixtures/paragraph.pdf` at page 0, (120, 704), through
[`crate::fixture::text_point_target`]. One `BT`…`ET` holding six `Tj`
operators, each with its own `Tm` on its own baseline: **one text object of
six chunks**, which is the shape this check needs and the reason the count
below is an equality rather than a floor.

A floor would pass on a build that had lost five of the six. The number is
a property of the committed document, so it is assertable exactly, and a run
that reports a different one has found either a broken box walk or an engine
whose line granularity has moved — both worth a red line rather than a pass.

A one-chunk object is declined by design (`single-chunk`: its one box
would sit on the selection outline already drawn there), so a fixture whose
text object held a single line would measure nothing while looking green.

⚠ A missing fixture is a **FAIL**, not a SKIP: it is committed to this
repository, so its absence is a broken checkout.

# ⚠ What this check can see, and where its reach ends

It reads the **count the painter returned**, not the pixels. `draw_chunk_boxes`
answers with the number of rectangles it handed to the painter, from inside
its own loop, and `draw_chunks` traces that — so a build that never calls it
does not compile, and one that enters it and draws nothing traces `drawn=0`.

What it cannot see: a stroke made transparent, a colour equal to the page,
or a mapping that puts every box off-screen. Those count as drawn and have
one oracle, which is a rendered screenshot.

# ⚠ HOW TO FALSIFY THIS CHECK — do this before believing a PASS

1. **Copy the file aside first.** `cp crates/pdfcer-gui/src/canvas/overlay.rs
   $SCRATCH/overlay.rs.bak`. **Never revert it with git** — this project runs
   parallel tracks and a chained revert discards another track's uncommitted
   work; restore from the byte copy.
2. **Plant the defect no unit test can see**: in `draw_chunk_boxes`, make the
   loop `for page_rect in boxes.iter().take(0)`. It compiles, every unit test
   stays green, and the canvas gets nothing. Step B goes red on `drawn=0`.

   The plant that would NOT be caught is returning `boxes.len()` instead
   of the loop's own tally — a deliberate lie, and the single change that
   would quietly disarm this check. It is why the count is taken inside the
   loop, and why that is argued where the count is produced.
3. **A second, sharper plant, for step C alone**: make `draw_chunks` read
   `app.prefs.text_chunks` rather than `canvas::chunks::enabled`. The ribbon
   press still traces, the switch still flips, and the boxes stay on until
   the next launch.
4. **A third, for the ribbon link**: delete the `view.text_chunks` row from
   `shell::manifest::view`. The command stays registered and the ledger stays
   green; the switch becomes unreachable.
5. **Prove the plant is in the artifact.** `cargo build --release -p
   pdfcer-gui`, then confirm the exe is newer than the source: a stale binary
   is the commonest cause of a falsification that "did not reproduce", and
   its tell is an **absent** trace line rather than a wrong one.
6. **Require the `[FAIL]` line**, not the exit code — a SKIP exits the way a
   PASS does.
7. **Restore from the byte copy**, rebuild, confirm the PASS returns.
