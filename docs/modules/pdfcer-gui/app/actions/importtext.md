# `app::actions::importtext` — a text file becomes pages, and the receipt
says what the import decided

`Action::ImportText`'s body. Reached from `dialogs::import_text`'s Import
button and from nowhere else.

## THIS MODULE IS MOSTLY A DISCLOSURE, AND THAT IS THE DESIGN

`EditSession::place_text` does the work in one call. What takes the space
here is `PlaceTextReport` — **23 fields**, six of which are judgements the
import made about the operator's own file:

| field | the judgement |
|---|---|
| `paragraphs_split_across_pages` | a paragraph he wrote as one block is now on two sheets |
| `tabs_collapsed` | his column alignment is gone; a tab became a space |
| `chars_dropped_control` | bytes in his file were not printable and are not in the PDF |
| `dropped_unmappable_chars` | characters the face could not write, dropped because he opted in |
| `explicit_page_breaks` | U+000C in the source became a page break, which he may not know he wrote |
| `overlong_words` | a word wider than the column, which will overflow rather than wrap |

⇒ **Rule 4's surviving half, six times over.** Every one of these is an
inference the operator cannot see by looking at the result: a paragraph
split across a page break looks like a paragraph he wrote that way, and a
collapsed tab looks like a space he typed. Render normally; report
separately. Both.

And `box_overflow_lines` is not in that table because it is **not** a
judgement — it is the engine's own self-check, and it must be zero. It is
reported only when it is not, in the language of a defect rather than of a
disclosure.

## The undo promise is READ, never assumed

The engine's own doc is explicit that the one-undo-entry fold is *checked,
not assumed*: an import needing more commands than `MAX_UNDO_DEPTH` still
places **every** page and simply fails to group them, reporting
`coalesced == false` with `undo_entries` giving the real count.

⇒ A surface that promised one `Ctrl+Z` without reading `coalesced` would be
promising something the engine has already said may not be true. This one
reads it, and says *"this will take N presses of undo"* when it is false —
which is the only moment an operator can act on the fact, because after the
first press the rest look like a program undoing things by itself.

## Three refusals are named, and the reason is that two of them are
answerable BEFORE the press

`PlaceTextError` has ten variants. `NoColumn` and `PageTooShort` both mean
*the chosen margins leave no room on the chosen sheet* — which is a
**chooser** problem, so their sentences name the two controls that fix it
rather than describing a failure. `NoPageToInsertBeside` means an import
into a document with no page at all; the command is greyed on `doc.pages`
for exactly that, so reaching it is a surprise and says so.

The remaining seven fall to a catch-all carrying the engine's own message,
which is the posture `annots::refusal_for` argues at length: a catch-all
that says *the page is exactly as it was* is honest, where a catch-all that
shrugged would not be.
