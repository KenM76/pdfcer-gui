# `ui-verify/checks/move_line_of_text`

`move_line_of_text` — **a drag on one line inside a block of text MOVES
it, or says why it cannot.**

The driven half of `OPERATOR_REQUESTS.md` **O188**, changes (B) and (C). It
is the only thing that can say the feature works, because the defect O188
names is a *silence*, and a silence is exactly what a green unit-test suite
looks like.

# A CHECK THAT PINS AN ABSENCE HAS A SHELF LIFE IN DAYS

This check first asserted that a single line could NOT be moved, because
on the day it was written the engine had no verb for it. The verb landed,
and the assertion became a confident statement of the opposite of the
truth — green the whole time.

⇒ **A driven check that pins a capability's absence has a shelf life
measured in days.** Nothing in this repository changed; a dependency did,
and a green suite became a liar. The defence is not to avoid such checks —
this one caught a real silence — but to write the absence down where the
backlog gate can see it (`ENGINE_BACKLOG.md`) so that the delivery arrives
as a failing gate rather than as a quietly wrong assertion.

# What it asserts: four drags, three answers, one document

A line move succeeds on most lines, including one written in pieces whose
joins carry no position, and refuses on two shapes, each with its own
sentence. So the check drives four drags against one fixture:

| the line under the pointer | what must happen |
|---|---|
| one written in two pieces, the join having no position of its own | **it MOVES**, both pieces as one set |
| one that states its own position, with no successor | **it MOVES** |
| one the NEXT line's position is measured from | refused, *moving it would drag that line along too* |
| one whose position this document does not state | refused, *there is no position here to change* |

**The rows that MOVE are not a bonus, they are what make the other two
mean something.** A table of refusals alone passes for ever against the
build this check was originally written for — the one that refused every
line move. A check that cannot fail on the dangerous build is not a check.

# WHY THIS CHECK EXISTS WHEN FOUR UNIT TESTS ALREADY COVER IT

`canvas::moving::tests` asserts, at the seam, that `decline` pushes the
right `Action::DeclineOnCanvas`, and that a movable line produces
`VectorAction::MoveTextLine`. Those tests are right and they are not
evidence, for this project's founding
reason (**R1**): they call the function. They cannot see the chain in front
of it — whether a real drag at a real rung reaches `decline` at all,
whether the apply phase has an arm for the action, whether the status bar
draws what the store holds, or whether a mode gate swallows the gesture two
layers earlier. Eight green tests once sat in front of a feature that did
one step of fourteen.

So this drives the OS: a real click into Edit mode, a real chord to arm the
Points tool, a real click to select one line, a real press-move-release,
and then it reads what three independent subsystems wrote down.

# The oracle — three lines, three subsystems, in order (per aim)

```text
canvas       canvas-move-declined level=Part sel=1 reason=run-would-move-next …
apply phase  canvas-decline-recorded what=text-run-would-drag-next-line
status bar   ui-rect name=status-group:decline rect=…
```

and, for the row that commits, one line from a fourth subsystem:

```text
apply phase  move-text-line page=0 n=1 epoch=3 disclosures=none
```

`n=` is the number of show operators the line is written in. The one
aim that commits here is a one-piece line, so it is `1`; on his own sheet
the same line reports `n=9`.

| line | question it answers | who writes it |
|---|---|---|
| `canvas-move-declined` | did the gesture reach the move rules, and refuse for the reason this check is about? | `crate::canvas::moving`, holding `&OpenDoc` |
| `canvas-decline-recorded` | did the refusal's **sentence** cross the `Action` boundary and reach the store? | `crate::app::status::decline::canvas`, holding `&mut` |
| `status-group:decline` | was it **drawn**, on a frame, where he could read it? | `crate::app::status::disclosure` |

**That is a chain measurement, not the application agreeing with
itself.** The three writers are three subsystems separated by the exact
boundary O188's design is about — a canvas gesture holds no `&mut` and can
only *ask*. A shell that raised the action and had no apply arm for it
writes the first line and not the second. A shell that recorded the
sentence into a store the bar never reads writes the first two and not the
third.

## Why the region alone would have been worthless

`status-group:decline` is **one region shared by every decline in the
application** — a save that failed, a bookmark that would not move, a zoom
with nothing to frame. A check asserting only *"that region is on screen
after the drag"* is satisfied just as well by a build that raised the wrong
sentence, and by a stale sentence left on the bar by an earlier gesture.

> An assertion both outcomes satisfy is not a measurement of which one
> shipped. Name what the WRONG mechanism cannot produce.

`canvas-decline-recorded what=…` is that name. It carries a **stable
token**, not a `Debug` rendering, for the reason this project has recorded
twice: a `{:?}` field is a property of how a variant is *spelled*, so a
rename turns a driven check into a confident false negative that quotes the
truth in its own failure message.

# ⚠ HOW TO FALSIFY THIS CHECK — do this before believing a PASS

A check that has never been seen to fail is not evidence.

1. **Copy the file aside first.**
   `cp crates/pdfcer-gui/src/canvas/moving/mod.rs $SCRATCH/moving.rs.bak`.
   **Never `git checkout` to undo it** — this project runs parallel tracks
   and that discards another track's uncommitted work.
2. **Plant the silence Ken reported**: in `Refusal::worded`, change
   `Self::TextRunCannotMove(RunMoveBlock::NoPositionOfItsOwn)` from
   `Some(CanvasDecline::TextRunHasNoPositionOfItsOwn)` to `None`. The
   refusal still happens and still traces; it just says nothing, which is
   the shape of the defect O188 names.
3. **Prove the plant is in the artifact.** `cargo build --release -p
   pdfcer-gui`, then confirm the exe is newer than the source — a stale
   binary is the commonest cause of a falsification that "did not
   reproduce". Do not grep the exe for the token: `token()` is untouched
   by this plant, so the string is still in there and its presence proves
   nothing either way.
4. **Require the `[FAIL]` line**, not the exit code — a SKIP exits the way
   a PASS does. It must fail on **the last aim only**, naming *the refusal
   happened and raised no sentence* and quoting the `canvas-move-declined`
   line it saw. The other three aims must still pass: a plant that reddens
   all four has broken the drag, not the sentence, and has measured
   nothing about this check's discrimination.
5. **Restore from the byte copy**, rebuild, confirm the PASS returns.

A second, cheaper plant exercises the third link on its own: in
`app::status::decline::show`, return before `disclosure_line` publishes
`REGION_DECLINE`. All three declining aims should then fail on the
*region* while still reporting the right `canvas-decline-recorded` token —
which is the one outcome that separates "recorded but never drawn" from
"never recorded".

# Fixture — pinned here, not passed on the command line

`fixtures/inherited-runs.pdf` at page 0, four aims in PDF user space.

**NOT `paragraph.pdf`, which every other line-of-text check in this
harness uses** — and the reason is the whole argument for a second
fixture. `paragraph.pdf` writes a `Tm` in front of all six of its show
operators, so every one of its lines states its own position, so
`text_run_move_refusal` answers `None` six times out of six and **neither
refusal can be reached on it**. A check written against it alone would pass
on a build that had deleted the pre-check entirely, for ever.

`inherited-runs.pdf` is one `BT`…`ET` block holding **five** `Tj`
operators at 12 pt on a 612 × 792 page — three horizontal, then a pair
turned a quarter turn — with a positioning operator in front of three of
them and **none** in front of the other two. Grouped into visual lines that
is **four** lines: two that move and one for each of the engine's two
line-move refusals, in one document, which is what [`AIMS`] drives.
`tools/gen-inherited-runs-fixture.py` builds it and carries the reasoning;
`fixtures/inherited-runs.PROVENANCE.md` carries the measured spans the aims
in [`AIMS`] were computed from.


⚠ A missing fixture is a **FAIL**, not a SKIP: it is committed to this
repository, so its absence is a broken checkout rather than an unavailable
precondition, and a SKIP would say the opposite.

# The Points tool, and why there is no double-click here

The Part rung on a **text** object is not reachable by double-click, by
design: `canvas::clicking`'s O70 arm opens the caret instead, which is the
operator's own ruling (*"double-clicking inside the bounding box should
edit the text"*). The route that exists is the **Points** tool — chord
`A`, labelled *Points* because a draughtsman says point — whose branch
takes the click before every other claimant and calls
`SelectionState::click_direct`, landing on the Part rung whenever the probe
found a part. On a text object a "part" **is** a visual line — which may
be written in any number of show operators, and on his own sheet usually
is.

`deeper_rung_delete::Rung::arms_the_points_tool` carries the measurement
that established this, including the trace lines it was read out of.

The chord is pressed **before** the click, not after: with the arrow
armed, the first click would select the whole text object and the Points
tool's own branch would then be entering an object it did not pick.

It also does the check a second favour, and the check depends on it.
Arming the tool is a **command**, and `app::status::decline::retire` runs
at the top of `dispatch_command` — so any decline left on the bar by an
earlier gesture is cleared before this one starts. That is what makes the
"nothing on the bar before the drag" control below assertable rather than
hopeful.

## Item notes

### `const MODE`

The shell's default is Read, where a canvas click on content is refused BY
DESIGN. A check that skipped this step would report the mode gate as a
selection defect — which `delete_key`'s own header records as having
happened.

### `const MOVE_DECLINED_EVENT`

Not per frame. An in-flight drag is re-evaluated 60 times a second and a
refusal traced per frame would bury every other event on the channel — the
lesson `canvas-pointer` taught when a stationary pointer emitted fifty
identical lines in nine seconds.

### `const RECORDED_EVENT`

**The per-refusal tokens live in [`AIMS`], not in constants here.** A
token hoisted to module scope reads as a property of the check, so it
survives the disappearance of the refusal it names — the assertion stays
green while the constant becomes the name of nothing. A token that only one
row needs belongs in that row, where the answer it describes is visible
beside it and a changed answer changes them together.

### `const DECLINE_REGION`

⚠ Read through [`driving::declared`], never with `Trace::last`. The
application's `ui-rect` channel is a **change log** — it emits when a rect
moves and emits a `ui-rect-gone` when a region stops being drawn — so
`.last()` returns a fossil for a region that has been retired, and a caller
cannot tell it from a live one. That misreading once produced a confident,
detailed, entirely wrong layout-defect report about eighteen ribbon
controls.

### `const AIMS`

One launch, one fixture, four drags — one per visual LINE of
`inherited-runs.pdf`. Every field here is measured; the spans are recorded
in `fixtures/inherited-runs.PROVENANCE.md` and pinned by
`provider::line::tests::the_local_fixture_gives_all_four_line_move_answers`,
which decomposes the same file.

**The aim is not asserted directly, and it does not need to be.** The
two refusing lines produce different answers and differ from the two that
move, so an aim that landed on the wrong one mostly produces the wrong
answer and this check fails — loudly, naming what it got. That is the
property a table of four buys that four separate
checks against four separate fixtures could not: the discriminating power
is in the document, not in the harness's arithmetic.

**Two aims are on rotated text, and that is not decoration.** An
inherited show operator advances along the text direction, so a horizontal
one always lands on its predecessor's baseline and is always inside its
predecessor's line. A line can only BEGIN with an inherited piece when the
text is turned — so without rows three and four, `run-has-no-position` and
`run-would-move-next` are tokens no document could produce at line
granularity, and this check would pass on a build that had deleted both.

### `const MOVED_EVENT`

`n=` is the number of show operators the line is written in, not `1`.
The pieces go to one `move_text_runs` as a set, one undo step, so a line
a producer wrote as nine `Tj`s reports `n=9`.

Asserted instead of `canvas-move`, and the difference is the whole
point of asserting it. `canvas-move` is written by the canvas when it
RAISES the action; this one is written by the apply phase after the engine
has accepted the edit and the epoch has moved. A shell that raised a move
the engine then refused writes the first and not the second — and that is
precisely the build a pre-check regression produces.

### `const DRAG_PX`

Comfortably past any drag threshold and past
`canvas::moving::Refusal::NoTravel`'s floor, and small enough that the
pointer stays well inside the canvas on a 612 × 792 page at fit zoom. The
destination does not matter: the release is refused before any geometry is
computed, so this only has to be *a drag* rather than *a click*.

### `fn drive`

The three-way return is the SKIP/FAIL/PASS rule made structural: `Err` is a
precondition that was absent (SKIP), `Ok(Some(_))` is an assertion that did
not hold (FAIL), `Ok(None)` is a pass.

### `fn one_aim`

Same three-way return as [`drive`]: `Err` SKIP, `Ok(Some(_))` FAIL,
`Ok(None)` pass.

# Why the control is INSIDE this function and not once at the top

Because `status-group:decline` is one region shared by every decline in the
application, and two of the three rows put a sentence in it. A control taken
once, before the loop, would be a baseline for row 1 and a fossil for rows 2
and 3 — the exact shape of *an absence assertion is only as good as when
its baseline was taken*, which cost this project a green check over a
planted defect. Each row establishes its own.

The clearing gesture is a press of **Escape**. It ascends the selection
ladder, which is what this row needs: every row starts from no selection and
descends to the Part rung by its own click, so no row inherits a rung.


This function's first shape treated an empty status-bar slot as a
**precondition** and skipped the row when the slot was still on screen,
reasoning that *"Escape is a command, and `decline::retire` runs at the top
of `dispatch_command`."* The first driven run refuted it: row 2 skipped with
the slot still live, and the trace carries no `ui-rect-gone
name=status-group:decline` anywhere — so the sentence really was still
drawn.

The cause is that **Escape is claimed by the canvas before it can become a
command.** With a selection standing, `canvas::keys` spends it ascending the
ladder, so `dispatch_command` never runs and neither does `retire`.
`canvas::keys`' own header states the contract plainly — *"pressing Escape
twice puts the tool down; pressing it once corrects a mis-aimed pick"* —
and neither of those is a command.

**That is behaviour, not a defect, and the fix was to stop needing it.**
Both of this check's sentences are `still_true` for the reason
`decline::fresh` calls *"the FILE cannot change under it"*: they report a
property of the document, so no later moment makes them stale and they are
meant to outlive a deselection. A check that demanded an empty slot was
asserting a retirement policy this project deliberately does not have.

**So the per-row link is a MARK, not an absence.** Every assertion below
is anchored at a `Trace::mark` taken immediately before this row's drag and
reads `last_after`. A sentence left standing by an earlier row cannot
satisfy them, because the events that carry it are behind the mark. The
slot's live-ness is still read — and reported — but as a note, which is
the difference between measuring a thing and gating on it.

### `fn moved`

**This is the row that proves the other two are measuring something.**
It is also the whole of O188's move half as the operator experiences it —
he drags one label in a title block and the label moves.
