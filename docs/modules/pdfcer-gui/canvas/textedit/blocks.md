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

★★ **The reassembly is `pdfcer-core`'s and always was.**
`EditableTextModel::recognize` groups a page's show operators into lines and
lines into `Block`s by column band, and `caret_up` / `caret_down` walk
*lines* rather than runs — so a caret at the end of one paragraph's last line
steps into the next paragraph without anything here knowing what a paragraph
is. The old shell's whole contribution was **asking**.

This shell had not been asking. Its caret is a character index into a
one-run draft, so Up and Down had no meaning and were not bound at all: there
is no line above a single run.

## ★ Why a page-space model rather than the draft's own string

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
