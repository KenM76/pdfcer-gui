# `pdfcer-gui/panels/objects/provider/line`

**The Part rung's unit for a text object: one visual LINE, not one show
operator.**

# Why the two are not the same thing

A PDF text object is a list of show operators (`Tj`/`TJ`/`'`/`"`), each with
its own origin. What the operator sees as one line of a title block is
however many of those its producer chose to write. Measured on
`SW41177.pdf` page 0, object 5871 — a SolidWorks notes column — that is
**237 show operators forming 144 lines**, and the single line
`#2 USE SPACERS 8 9 10 11 IF REQUIRED.` is **nine** of them.

So a Part rung indexed in show operators tells the operator he is on *line
54 of 237* when the block has 144 lines, draws a selection box round a 19 pt
fragment in the middle of the words he clicked, and moves that fragment
alone when he drags it. `OPERATOR_REQUESTS.md` O214, in his words: *"take a
line like this that is part of a larger block and … relocate it by dragging
and moving as if it wasn't part of a larger block."*

# The grouping is the engine's, not a second opinion

[`pdfcer_core::vector::edit::text_object_split_points`] at
[`SplitGranularity::Line`] decides where the lines are, and this module
calls it rather than comparing baselines itself. `pdfcer-core` already has
to answer this question to plan a split; a shell that answered it a second
way would one day draw a box round one grouping and move another.

⚠ **It is an inference and the engine says so** — §14.8 does not record
where a text object's lines are. The grouping is *consecutive runs sharing a
baseline, in stream order*, which is what makes it usable on CAD output: a
sheet has dozens of unrelated labels sharing a y-coordinate across its
width, and grouping by baseline alone would weld them into one line.

# What follows from the grouping, and it is the whole of the move design

A run whose `positioned_by` is
[`Inherited`](pdfcer_core::vector::RunPositioning::Inherited) starts wherever
the previous show operator's pen stopped — so for horizontal text its
baseline is its predecessor's, and it is therefore **always in its
predecessor's line group**. Two consequences:

1. **A line's first fragment is the only one that can lack a position of its
   own**, and only when it is run 0 of the whole object.
2. **An inherited fragment inside a line needs no move of its own** — it
   follows the fragment before it. A line is therefore moved as one SET
   through `EditSession::move_text_runs`, and
   [`ObjectModelProvider::text_line_move_refusal_of`] asks the engine's set
   guard, which lets an inherited run through when its predecessor moves
   too.

## Item notes

### `fn lines_of`

Always covers every run exactly once and is never empty for a non-empty
object, because [`text_object_split_points`](pdfcer_core::vector::edit::text_object_split_points)
never returns 0 and never returns an index past the last run.

### `fn text_object`

**What keeps it from being an oracle written by the thing it
measures:** every field below is fixed by sub-clause 9.4.2 rather than
chosen. `text_matrix` is `Tm` as it stood at the show operator, so two
`Tj`s with no positioning operator between them share one, and a `Tm`
in between gives the second a new `f`. The same shape was measured on
a real decomposition in `tests/ken_sw41177_line_move_probe.rs`, where
nine consecutive runs of one visual line came back sharing baseline
927.23. The driven proof over a real file is `ui-verify`'s.

### `fn a_line_whose_first_fragment_inherits_names_the_line`

Built by declaring run 0 `Inherited`. That is what a text object
whose first show operator relies on the text-state carried in from
before `BT` looks like, and the engine refuses to move it for the same
reason it refuses any other: there is no operand to rewrite.

### `fn the_local_fixture_gives_all_four_line_move_answers`

Every other test in this module builds its runs by hand, which makes
them a calibration of the grouping rule and not a measurement of it:
the fixture and the code under test were written from the same reading
of 9.4.2, so both can be wrong together. This one decomposes
`fixtures/inherited-runs.pdf` — a file on disk, written by a generator
that knows nothing about `runs_share_a_line` — and asserts the table in
that generator's header.

**The rotated pair is the load-bearing half.** An inherited run
advances along the text direction, so a HORIZONTAL one always lands on
its predecessor's baseline and is always inside its predecessor's line
group. Rotation is the only way a line can BEGIN with an inherited run,
and without it `NoPositionOfItsOwn` and `WouldMoveNextRun` are sentences
no document could produce at line granularity — which would leave a
build that had deleted them passing every check.

Line 1 is the CONTROL. Without an answer of `None` somewhere on the
page, a build that refused every line move would satisfy the other
three assertions.

### `fn the_aims_driven_at_this_fixture_land_one_per_line`

# Why a unit test owns the harness's coordinates

`AIMS` asserts an ANSWER per aim, never a line index, because
`canvas-selection` carries no part index for it to read back. That
makes the aims self-checking only while the four lines give four
different answers — and silently wrong the moment two of them agree.
Here the index IS visible, so the mapping from point to line can be
stated outright.

The exclusivity half is the load-bearing one. The rotated pair is
stacked along one narrow column and meets at a single y, so a point
that fell in both boxes would still satisfy a containment-only
assertion while aiming at whichever of the two the hit test happened
to return first.

PDF user space, y-up, straight off the engine's decomposition. No
canvas transform is involved: `AIMS` is in page coordinates and the
harness maps it at drive time, so converting here would introduce the
one step this is meant to hold still.

### `fn text_line_runs_of`

`None` for a non-text object or a line index the object does not have —
which is a stale selection, and the caller's cue to say nothing rather
than to name line 0.

### `fn text_line_hits`

Order comes from [`pdfcer_core::vector::hit_test_text_runs`], which
answers nearest-first in run indices; several runs of one line under the
pointer collapse to that line's first sighting, so the caller's
`first()` is still the nearest thing to the pointer.

Page objects only. A text object painted from inside a form XObject has
no hit test at any granularity — [`Self::text_run_hits`] indexes the
page's own list, so answering a leaf from it would return another
object's runs entirely — and this inherits that hole rather than
papering over it. `canvas::target`'s `part_hits_of` records it in the
same terms.

### `fn text_line_bounds_canvas_of`

The union of its fragments' boxes. Drawing one fragment's box instead is
the visible half of O214: the operator clicks the middle of a phrase,
gets a rectangle round nineteen points of it, and concludes the
selection is broken.

### `fn text_line_delete_would_move_next_of`

Asks about the run *after the line*, not about the run after the one the
operator clicked. Deleting a line removes every fragment of it, so the
only run whose origin can be orphaned is the first one left standing.

`false` when the line is the whole object: deleting every run deletes
the text object, which the engine allows unconditionally.

This one reads `positioned_by` directly, because the delete-side guard
has no exported twin to call — the standing hazard
[`ObjectModelProvider::text_run_delete_would_move_next`] records.
