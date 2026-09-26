# `ui-verify/checks/bookmark_move`

`a_bookmark_can_be_dragged_and_a_branch_collapsed` — **the panel that could
create, rename and delete, and could not REORGANISE.**

# What this proves

`pdfcer-core` `Pass 161.0` shipped `move_outline_item` and
`set_outline_open`. Until then a bookmark could be written, retitled and
removed, and an outline in the wrong **order** could only be fixed by
deleting a branch and re-authoring it — which loses every destination,
colour and style on it. Two verbs, one gesture apiece, and both of them
reach the operator through the row list rather than through a button:

* **drag a row onto the middle of another** and it is filed inside it;
* **press the triangle** on a row with children and the branch folds away.

⇒ One check, because the second gesture is the only honest oracle for the
first half of the third phase and because they cannot be exercised
separately without paying twice for a launch, a mode click, a panel open and
two authored bookmarks.

# The collapse oracle is the DISAGREEMENT between two numbers

This is the assertion the whole check is built around, and it is the one no
unit test in the workspace can make:

> after the triangle is pressed, the panel says the document holds exactly
> as many bookmarks as it did a moment ago, and draws **one row fewer**.


A build that wrote the sign and kept drawing the children passes every unit
test about the tree, passes the funnel-line assertion below, and fails this.
A build that "collapsed" by deleting the subtree fails the item count. There
is no third build that passes both.

**Every count in this check is a DELTA**, measured against the outline the
fixture arrived with. The corpus is not uniform — `fixtures/four-pages.pdf`
ships with a six-item outline and the CAD exports have none — and a check
that hard-coded either number would SKIP on half of it while blaming the
fixture.

# The move oracle is the LEVEL, not the order

`bookmark-row level=` is `OutlineItem::level` — `0` for a top-level
bookmark, `1` for its child. The drag in phase B drops TAIL on the middle of
DETAIL, which is [`OutlinePlacement::LastChild`], so TAIL's level must go
**0 → 1**.

Asserting the *order* instead would pass on a build that reordered where it
should have re-parented, which is precisely the defect a three-band drop
model can produce by mis-reading the pointer's y. The level cannot be
reached by a reorder at all.

And `bookmark-move-report reparented=` is asserted beside it, from the
engine's own report. Two independent witnesses to one fact: the shell's
read of the tree afterwards, and the engine's account of what it did. They
agree on a correct build and a build that lies has to lie twice.

# Two rectangles per row, and using the wrong one aims at nothing

`bookmark-row` carries `rect=` (the **label**) and `row=` (the **full-width
strip**). They are different questions:

| to | aim with | why |
|---|---|---|
| press or lift a row | `rect=` | only the label is a widget; the strip's centre is empty space |
| drop onto a landing band | `row=` | the band test is over the strip, and the pointer must be inside it |

Getting that backwards produces a silent failure in each direction: a press
at the strip's centre lands on nothing and starts no drag, and a drop aimed
at the label's centre is over the row but tells you nothing about whether
the strip was the thing being tested.

Neither is a `ui_rect`. Both come from the panel's own per-row diagnostic
line, which is written for **every** row whether or not it is on screen —
see [`visible_rows`] and the incident it carries.

# Phases

| Phase | Does | Expected |
|---|---|---|
| A | author DETAIL, then TAIL, both at the top level | `items` up by 2, two more rows, both `level=0` |
| B | drag TAIL onto the middle band of DETAIL | `bookmark-drag-released placement=last-child`, `bookmark-move-report moved=1 reparented=1`, `move-bookmark`, and TAIL at `level=1` |
| C | press DETAIL's triangle | `bookmark-disclosure open=0`, `set-bookmark-open`, and **the item count unmoved with one row fewer drawn** |

[`OutlinePlacement::LastChild`]: https://docs.rs/pdfcer-core

## Item notes

### `const PARENT_KEYS`

Spelled from the letters `crate::sys::vk` actually publishes. That module
adds virtual-key constants **one at a time, with a reason**, deliberately —
its own note says so — so a check invents a word from the alphabet that is
there rather than widening a shared file for a fixture name. `DETAIL` is
also the word `bookmark_edit` renames to, so a reader comparing the two
traces sees a name they recognise.

### `const CHILD_KEYS`

A **different length** from the first, deliberately, so a trace that
reported only a character count could still tell them apart. Nothing below
needs that today; it costs nothing, and it is the property `bookmark_edit`
had to go back and add after the fact.

### `fn visible_rows`

# Why both filters, and why each was paid for

**The frame filter.** The trace holds every frame the application drew, and
this check needs to count rows — *"one row is drawn"* is its central
assertion. Counting `bookmark-row` lines across the whole trace counts
hundreds. The panel traces its census line at the **top** of its body,
before any row, so the lines after the last census are exactly the last
frame's rows.


⇒ The general form, and this suite has now met it three times: **a trace
line written for every item is not a list of the items you can click.** The
`ui-rect` census answers *what is on screen*; a per-item diagnostic answers
*what was computed*.

### `fn author`

Factored because it happens twice and the second time must be identical to
the first — a set-up that differed between the two bookmarks would leave the
check unable to say which difference mattered.
