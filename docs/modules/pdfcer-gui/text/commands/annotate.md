# `text::commands::annotate` — the labels and tooltips of the **Markup** and
**Measure** tabs

## Why this is a file of its own, and what the seam actually is

**R2** (no `.rs` file over 1,500 lines) forced a split when the three
unblocked Phase 6 markup kinds arrived: [`super`] reached 1,520 lines. But a
line count only says *that* something had to move; it does not say what, and
`tools/gates/check-file-size.sh` says in its own header that shaving prose to
fit a threshold is the behaviour it exists to refuse. So the question was
which subject was separable, and this one is:

> **Markup and Measure are the tabs about what you ADD ON TOP of the page.
> Everything left in [`super`] is about the file, the view, the pages, or the
> content that is already there.**

That is not a line drawn here for convenience. It is the line
[`crate::app::modes::Capabilities`] already draws — `edit_content` on one
side, `author_markup` and `author_measure` on the other — and it is why
Review mode exists at all: a reviewer may add a comment and a dimension to a
drawing they may not otherwise touch. It is also the line
[`crate::shell::manifest`] already draws, which keeps `markup.rs` and
`measure.rs` as files of their own beside `edit.rs` and `pages.rs`. A reader
adding a Markup command now edits `text/commands/annotate.rs` and
`manifest/markup.rs`, which sit one directory apart and describe the same
band.

## Nothing else changed


The tests stay in [`super`], with the list they walk. They are about the
catalog as a whole — no two labels alike, every tooltip a sentence, every
command reachable — and splitting them across the two files would let a
duplicate label pass by being in the other half.

## Item notes

### `fn markup_cloud`

**"Revision cloud", not "Cloud".** The operator's own words, three times,
were *"still no revision cloud tool"* — never "cloud" alone — and in AEC the
two-word phrase is the term of art: it means *this area changed on this
revision*, which a one-word "Cloud" beside "Polygon" and "Freehand" does not
say. It is also the longest label in the Shapes band and that is accepted,
because a band of one-word labels with a two-word outlier reads as the
outlier being the specific one, which it is.

The tooltip repeats Polygon's gesture sentence almost verbatim, deliberately.
The two tools take the identical run of clicks and the identical ending, and
a reader who learns one has learned the other; wording it differently would
imply a difference that does not exist. What it adds is the last clause —
what makes it a cloud rather than a polygon is the border, which is the only
thing that differs in the file too.

### `fn markup_ink`

**Freehand, not Ink.** The type and the specification say `/Ink`
(§12.5.6.12) and the operator says freehand, which is the same split
`Rectangle`/`/Square` makes in the other direction — see
`canvas::markup`'s header on whose vocabulary the names follow.

### `fn markup_add_node`

The tooltip names the four shapes it works on rather than the one it
does not, because a `/Line`'s row is **absent** and not greyed — nobody
reads a tooltip for a row they cannot see. What it does have to explain is
where the new corner lands, since the answer is *on the outline*, not under
the pointer: the click is allowed to be several points off the line.

*"a freehand mark"* joined the list on 2026-09-09 with `pdfcer-core`
`Pass 278.0`. A tooltip that listed three shapes while the row appeared on
a fourth would be the surface disagreeing with itself.

### `fn markup_remove_node`

The tooltip carries **the floor**, and it is the reason this command is
greyed rather than absent when the shape is down to its last corners. R9
asks that a greyed control always explain itself on hover, and the
explanation has to say what would make it live again — *draw another
corner* — or greying is just a locked door.

The freehand floor is **per stroke** (`Pass 278.0`): a mark of three
strokes can lose points from a long stroke while a two-point stroke beside
it greys this row. The tooltip says *each stroke* so the operator is not
left counting the whole mark.

### `fn measure_perimeter`

The description names all three endings, because a tool with three ways
to stop has to say so before the first click. Discovering the closing
convention by accident works; discovering it after tracing thirty vertices
the wrong way does not.

It also names what the number IS - the whole way round, added up - because
the operator asked for exactly that ("it adds the distance of all the
segments together for the dimension display") and a label reading only
"Perimeter" leaves an open path looking like the wrong tool for a pipe run.

### `fn measure_length`

The operator's ask of 2026-08-20: *"add a length tool that works like the
perimeter tool without needing to close the profile."*

The label is `Length`, not `Path length` or `Open perimeter`: the operator
asked for a *length tool*, and the word they used is the word to put on it.
The description names what it is FOR - a run of something - because "click
along and add it up" describes the gesture and not the reason.

### `fn measure_finish`

# Why the tooltip names the double-click

Because the double-click is the ending most operators will actually use,
and a control that exists *because* a gesture has no natural end is the one
place the other ending has to be taught. A tooltip that said only "finish
the current dimension" would leave an operator reaching for the ribbon on
every circle they place — which works, and is slower than the tool is meant
to be.

It says *radius or diameter* rather than "the current measurement" because
this command is not general: Linear and Two-line finish themselves at a
known click count, so Finish is greyed while either is armed and an
operator who read a general promise here would be right to call that a bug.

### `fn measure_manage_groups`

The label is also the **dock tab caption**, because
`crate::app::PdfcerApp::new` builds the panel registry from the command
catalog — one string, so the tab and the ribbon control can never disagree
about what the surface is called. "Manage dimension groups…" was a
reasonable ribbon label and an unreadable tab; "Dimension groups" is both.
