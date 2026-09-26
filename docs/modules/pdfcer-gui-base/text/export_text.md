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

### `fn intro`

Leads with what a text file **is not**, on [`crate::text::export_dxf`]'s
rule: the operator opening one expects the page, and what arrives is the
words in content order with every trace of the page removed. The sentence
that saves a support question is the one about what was left behind.

### `fn pages_range_hint`

The same syntax the Print window, the Insert-pages window, the OCR window
and the image export all accept, because they all call
`dialogs::print::tabs::parse_page_range`. An operator who learned it in one
window is entitled to it in the next.

### `fn pages_range_invalid`

Says the document's own count, because the commonest cause is a range that
runs past the end and the operator cannot check that against a number they
have not been given.

### `fn separator_marker_hint`

The operator is asking for something readable and getting something the
document does not contain. That is worth one clause, because a later reader
of the file has no way to tell the marker from a line that was on the page —
and on a drawing whose title block genuinely says `Page 2 of 6`, they would
be right not to be able to.

### `fn page_marker`

Catalogued rather than formatted at the call site even though it lands in
a file rather than on screen, because it is **prose an operator reads** and
the whole point of the catalog is that such prose lives in one place. The
blank line before it is part of the string: without it the marker runs on
from whatever the previous page's last line was.

### `fn encoding_line`

A CAD drawing carries degree signs, diameter marks, plus-or-minus and
occasionally a Greek letter, and every one of those is multi-byte in UTF-8
and mangled by anything that guesses a code page. Saying so costs one line
and answers the question an operator asks after the mangling, not before.

### `fn loses_breaks`

`text_extract`'s negative result S5: line breaks are **always** derived,
even in Tagged PDF, because a PDF content stream records where glyphs were
painted and nowhere records that two of them are in the same word. Saying so
matters because the operator is about to diff, grep or re-import this file,
and every one of those acts treats a line break as a fact about the source.

### `fn no_text_at_all`

The most important string in this catalog, and the reason the export refuses
before the save picker opens rather than after it.

A scanned drawing is a **picture of** text. There is no text layer, so the
extraction is correct, complete, and empty — and a zero-byte `.txt` on disk
is indistinguishable from a successful export of a page that happened to be
blank. The operator would find out when they opened it, or worse, when
whoever they sent it to did.

So it says three things in order: that nothing was written, **why** (the
page is a picture, which is a fact about their file rather than a pdfcer
failure), and the command that fixes it — named exactly as it appears on the
ribbon, because a remedy the operator cannot find is not a remedy.

### `fn wrote`

The lead-in, so it is the sentence an operator reads if they read only one —
`super::super::app::actions::record_notes`' own rule. Characters rather than
bytes, because the operator asked for words and a byte count of UTF-8 is a
number about the encoding.

### `fn wrote_with`

Reported only when a departure was chosen, on the image export's rule that a
bar which narrates non-events stops being read. UTF-8 without a mark is what
the window promised and what the clipboard already carries.

### `fn marker_lines_added`

Repeated here even though the window said it, because the window is gone
and the file is not. This is the one added-text disclosure that survives the
act — an operator who sends the file on has sent lines pdfcer wrote.

### `fn pages_without_text`

Named rather than counted, because *which* page came out empty is the whole
of what the operator does next with this sentence: an empty page 4 in a
six-page set is a scanned insert, and they can go and look at it.

Capped, because a fifty-page scan set would otherwise put fifty numbers in a
status bar. The cap is stated rather than silent — a trailing "and N more"
is a count the operator can act on; a truncated list they were not told was
truncated is a wrong answer.

### `fn unreadable_fonts`

`text_extract`'s two dead ends, and the engine is emphatic that neither is a
pdfcer shortfall: a **Type 3** font names its glyphs with arbitrary
`/CharProcs` keys and an **Identity-H** font with no `/ToUnicode` publishes
no mapping at all, so ISO 32000-1 §9.10.2's own answer is that no Unicode
exists to be recovered. Acrobat is gated on the identical entry.

⇒ Which is exactly why it must be said. The page looks right, the export
looks like it worked, and the words are missing — and Acrobat's answer to
this case is to give up silently, which rule 4 forbids.

### `fn undecodable_characters`

The engine's headline honesty metric, and it is reported as a **fraction**
rather than a bare count: 40 failures out of 200 characters is a broken
export and 40 out of 400,000 is a stray glyph, and the two need different
reactions from the operator.

It **describes** the replacement character rather than printing one.
`text::glyphs` proves the font stack cannot draw U+FFFD, so a literal one
here would render as a substitution box — and a sentence explaining that
unreadable characters became a box, in which the box is itself unreadable,
is a joke at the operator's expense. The name is also what they can search
their text editor for.

### `fn pages_unreadable`

A different fact from [`pages_without_text`] and kept apart from it: an
empty page is a page pdfcer read successfully and found nothing on; this is a
page pdfcer could not read. Rolling the two together would let a damaged
file present as a scan.

### `fn pages_resources_defaulted`

# Why this is one of the few `TextDiagnostics` counters worth a sentence

[`honesty_notes`](crate::app::actions::export) takes three of roughly
thirty, on the test *"does it change what the operator should do next?"*.
This one passes that test for a reason that is easy to miss: §7.8.3 lets a
**Type 3 font** omit its own `/Resources` and inherit **the page's**, so a
page with no dictionary of its own can be the reason text on it decoded to
nothing. An operator reading a short export and no explanation concludes
the file is a scan.
