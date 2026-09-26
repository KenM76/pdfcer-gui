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
