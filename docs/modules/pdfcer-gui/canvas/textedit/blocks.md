# `canvas::textedit::blocks` — the page's lines, reassembled into paragraphs

## What this is, and where it came from


> *"there was an acrobat feature in the original pdfcer-gui that attempted to
> reassemble individual lines into paragraphs and the cursor would move to
> the next block of text using the navigation keys."*

**This is salvage.** It existed in the shell this project is replacing, and
the first act was to read it there rather than to design it. What it did,
from `D:\Dev\pdfce\crates\pdfce-gui\src\main.rs`:

```text
egui::Key::ArrowUp   => model.caret_up(cur, model.caret_x(cur).unwrap_or(0.0)),
egui::Key::ArrowDown => model.caret_down(cur, model.caret_x(cur).unwrap_or(0.0)),
egui::Key::Home      => model.line_range_at(cur).map_or(cur, |(s, _)| s),
egui::Key::End       => model.line_range_at(cur).map_or(cur, |(_, e)| e),
```

**The reassembly is `pdfcer-core`'s and always was.**
`EditableTextModel::recognize` groups a page's show operators into lines and
lines into `Block`s by column band, and `caret_up` / `caret_down` walk
*lines* rather than runs — so a caret at the end of one paragraph's last line
steps into the next paragraph without anything here knowing what a paragraph
is. The old shell's whole contribution was **asking**.

This shell had not been asking. Its caret is a character index into a
one-run draft, so Up and Down had no meaning and were not bound at all: there
is no line above a single run.

## Why a page-space model rather than the draft's own string

Because *"the next block of text"* is a fact about the **page**, not about
what is being typed. A draft knows one run's characters and nothing about
what is above or below it on the sheet — and the answer has to come from
geometry, because two runs adjacent in content order can be at opposite
corners of a drawing.

That is why every function here takes the document and rebuilds the model.
It is not cheap (see [`neighbour`]'s note) and it is not on a per-frame path.

## What this deliberately does not do

- **It does not move within a text BOX.** A box draft is multi-line in its
  own right and its lines are the shell's wrap, not the page's — so Up and
  Down there are a different question with a different answer, and answering
  it with this model would move the caret to a run somewhere else on the page
  mid-paragraph. Named rather than left to be discovered.
- **It does not draw the paragraph.** Showing which block the caret is in is
  worth doing and is a separate surface; this is the navigation.

## Item notes

### `fn land`

The order is load-bearing: **commit first**. A caret that walks out of a
run with unsaved keystrokes in it silently discards them, which is this
project's defining defect class — and `commit_into` writes nothing when the
text is unchanged, so an operator merely *reading* with the navigation keys
puts nothing on the undo stack.

### `fn characters_and_bytes_round_trip_through_an_accent`

Everything else here is `pdfcer-core`'s recognition, which has its own
tests and needs a real page. What is *this* module's own is the
character ⟷ byte hop, and it is exactly the kind of arithmetic that
compiles either way round and puts the caret inside a multi-byte
character on the first document with an accent in it.

Asserted on a string that has one: `"café"` is five bytes and four
characters, so a caret index of 4 is a byte offset of 5 and any
implementation that confused them would be off by one at the end.

### `enum Vertical`

Named for what the operator pressed rather than for a sign, because "up" on
screen is a **larger** baseline y in PDF user space and a `-1` here would be
a number whose meaning depends on which space the reader has in mind. This
project has met that confusion four times in coordinate arithmetic; a
two-variant enum cannot have it.

### `fn neighbour`

`None` when there is nothing there — the top line of the topmost block, a
page whose text will not extract, a run the model does not recognise. The
caller leaves the caret where it is, which is what every editor does at the
end of a document.

# It crosses paragraphs without knowing what one is

`caret_up` and `caret_down` walk the model's **lines**, and a `Block` is a
group of lines. So a caret on the last line of one paragraph steps to the
first line of the next, and nothing in this function had to look at a block
to make that happen. That is the whole of the operator's *"the cursor would
move to the next block of text"*, and it is `pdfcer-core`'s recognition doing
the work.

# The desired column is preserved, which is what makes repeated presses
# behave

`caret_x` is the caret's page-space x; passing it to `caret_up` asks for the
nearest slot in the same column on the line above. Without it a caret
stepping through lines of unequal length would drift toward whichever end the
implementation happened to clamp to, and three presses down and three back up
would not return it to where it started.

Not carried across presses, deliberately: a true "desired column" is
remembered from the *first* vertical press and survives short lines in
between, which is what a text editor does. That is a second piece of state
and it is not built — recorded here so it is a decision rather than an
oversight. What is built is right for one press at a time.

# Cost

One provenance-free extraction and one recognition of the whole page, per
press. Measured elsewhere in this crate at **336 ms on the benchmark CAD
sheet** for the extraction alone — which is why this is on a *keystroke*
path and not a frame path, and why the caller must not call it speculatively.

### `fn line_end`

Home and End. Salvaged from the same four lines as [`neighbour`] and using
the same model, so a line means the same thing to both.

A *line* here is the page's, not the run's — a line drawn as four separate
show operators (a CAD title block's row, which is the shape this operator's
documents are full of) is one line to the model, so End reaches the end of
what he can see rather than the end of the fragment he happens to be in.
That is the same recognition that made `Reason::SharesTheLine` necessary, put
to a second use.

### `fn step`

# Why this is a function and not four lines in the match arm

Because the arm has to do three things in a fixed order and two of them are
easy to leave out: trace the outcome, **commit the draft it is leaving**,
and seed the new one from the page rather than from the draft it just left.
A caret that walks out of a run with unsaved keystrokes in it silently
discards them, which is this project's defining defect class — and
`commit_into` writes nothing when the text is unchanged, so an operator
merely *reading* with the arrow keys puts nothing on the undo stack.

# The NOWHERE outcome is traced too, and that came from a driven run

The first live run of `arrow_keys_walk_between_blocks` failed with *"the
arrow keys moved the caret nowhere"*, and the trace could not say which of
the two causes the check itself had named it was:

| cause | where it lives | what it means |
|---|---|---|
| the keys never reached this arm | the shell — an earlier arm ate them | a **defect** |
| [`neighbour`] answered `None` | the page — no line that way | a **fact about the document** |

One trace line covered only the success, so both failures looked identical
from outside: silence. `text-caret-nowhere` is the other half. That is
`DEFECTS.md` D14's rule in its less obvious form — *a trace must be able to
say the thing did not happen*, not only that it did, because a check that
cannot tell a build defect from a fixture fact will eventually accuse the
wrong one.

And `None` is genuinely ordinary here, which is why it must not read as
an error: `caret_up`/`caret_down` never cross a **column band**
(`pdfcer-core`'s reading order), so a lone label in the middle of a drawing
has nothing above or below it by construction.

### `fn line`

# A LINE IS THE PAGE'S, NOT THE RUN'S — which is the whole point

A row of a CAD title block is drawn as four or five separate show
operators, and the operator sees **one line**. So End belongs at the end of
what he can see rather than at the end of whichever fragment he happened to
click in, and reaching it means landing in a **different run** exactly as
[`step`] does. That is the same recognition that made `Reason::SharesTheLine`
necessary, put to a second use.

`false` is the ordinary answer, not a failure: on a line that is one run,
[`line_end`] reports there is nowhere new to go and the caller falls back to
moving within the draft — which lands in the same place, one allocation
cheaper and without touching the undo stack.
