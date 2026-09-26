# `text::export_text` — every word the Export-text window shows, and every
sentence a text export owes afterwards


> *"also the engine can export PDFs as text. we should have export/import
> for that."*

Half of that sentence is buildable and half of it is not, and this catalog
is the buildable half's copy. See
[`crate::app::actions::exporttext`]'s header for the whole finding on the
import side; the short version is that **`pdfcer-core` has no verb that
turns a text file into PDF page content**, and a request has been filed
rather than a round trip faked.

## The one sentence this whole window is arranged around

> **A scanned drawing has no text layer, so exporting it writes an empty
> file — and an empty file looks exactly like a successful export.**

That is this feature's version of the DXF export's *"a 1:2 detail arrives at
half size and looks plausible"*: the failure is silent, the artifact opens
cleanly, and the person who finds out is whoever needed the words. So the
export **refuses** rather than writing nothing — [`no_text_at_all`] — and it
names the remedy, which is `File ▸ Recognise text`.

## What "the text of this document" means, and why it is not decided here


That is not laziness, it is the whole point. Two answers to *"what is the
text of this document"* inside one program is worse than either answer on
its own, because both of them look like text and nothing on screen would
ever say which one you have. `app::dispatch::textcopy`'s header already
makes this argument for the two clipboard verbs sharing one extraction; this
is the same argument with a file on the end of it.

⇒ Every control in the window that departs from that string
([`separator_marker`], [`line_endings_windows`], [`bom`]) is **opt-in**,
and every one of them is named in the receipt afterwards. The default is the
clipboard's own bytes.

## Why the losses are said TWICE — in the window and in the receipt

They are different losses.

* **In the window**: the *standing* truths, which are true of every text
  export of every document and are therefore knowable before the press —
  layout is gone, a table becomes lines, columns may interleave, nothing
  about position or font or colour travels.
* **In the receipt**: the *counted* ones, which are facts about **this**
  document and cannot be known until the extraction has run — pages that
  came out empty, fonts carrying text that was never recoverable as Unicode
  at all, pages whose content stream would not walk.

Rule 4 as narrowed by decision 059 puts the second set **off-canvas and
after the fact**: the status bar, never a mark drawn on the page.

## Item notes

### `fn the_empty_scan_sentence_points_at_the_command_that_fixes_it`

This is the assertion the whole feature's honesty rests on. A refusal
that says *"nothing to export"* and stops is a dead end; one that names
`Recognise text` is a next step. The label is asserted **literally**, so
that renaming the command on the ribbon without renaming it here fails
here rather than in front of an operator hunting for a control that no
longer has that name.
