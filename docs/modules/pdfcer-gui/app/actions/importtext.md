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

## Item notes

### `fn import`

# Errors are worded, never dropped

Three of them: the file cannot be read, its bytes are not UTF-8, and the
engine refused. All three reach the status row, because a File ▸ Import that
does nothing and says nothing is this project's founding defect wearing a
different hat.

### `fn disclosures`

Ordered by what an operator would act on, not by the struct's field
order. The count of pages comes first because it is the answer to *"did that
work?"*; the undo warning comes second because it is the only one with a
deadline on it; the six judgements follow, and the engine's self-check is
last because it is a defect report rather than a disclosure and should not
be read as one of the six.

### `fn refusal_for`

See the module header for why exactly three are named. The catch-all carries
the engine's own message rather than a shrug, which is
`annots::refusal_for`'s posture and its argument applies here unchanged.

### `enum FileAction`

# Why this enum exists, and it is R2 arriving on time for once

`Action` already carries four sub-enums — `Annot`, `Vector`, `Field` and
the write family — and adds a fifth here. The pattern is the same one
`AnnotAction`'s own note argues: a family of related verbs shares one
dispatch arm, one apply arm and one module, so the shared files carry a line
each and the family's reasoning lives with the family.

**It has one member, and that is the honest shape rather than premature
structure.** Spelled directly on `Action` instead, one feature's argument
would be written **three times in three files nobody owns** — `action.rs`,
`apply.rs` and `dispatch.rs`, each already at R2's 1,500-line ceiling. The
line count is the symptom; the duplication is the defect, and R2's rule is
*"when a file approaches the limit, that is the signal to find the seam"*.
This is the seam.
