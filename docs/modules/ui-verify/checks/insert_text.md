# `ui-verify/checks/insert_text`

`a_caret_marks_an_insertion` — Markup ▸ Insert text puts a `/Caret` on the
page carrying the typed words and, when asked, a paragraph mark.

# What it drives

Its own fixture, `fixtures/layer-assign.pdf` (ignores `--pdf`; an 800 × 600
page).

1. Review mode, `ribbon.tab.markup`, `ribbon.item.markup.insert_text`:
   `markup-tool tool=TextAnnot(Caret)`.
2. A click at (150, 450), clear of the fixture's box and annotation:
   `text-annot-open kind=Caret`.
3. `inserted words ` typed into the window's text field (it takes focus on
   open), then `text-annot.caret-paragraph`, then `text-annot.accept`:
   `caret-annot-read … chars=14 paragraph=true`, then
   `caret-annot-placed … id=<non-zero>`.

The character count is the oracle that the typed words, trimmed, reached the
action; `paragraph=true` is the oracle that the checkbox's choice survives the
window.

# Driven off-screen

Scripted pointer, window at `-4200,-4200`, `spec.place = false`; it runs under
`--no-input`.
