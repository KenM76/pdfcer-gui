# `ui-verify/checks/tagged_underline`

`underlining_part_of_a_tagged_paragraph_says_it_is_not_recorded` — Ctrl+U on
one word of a tagged paragraph underlines the word, and the status bar says the
structure tree does not record that underline, because pdfcer does not split a
structure element to give the word one of its own.

# What it drives

`fixtures/tagged-paragraph.pdf` (ignores `--pdf`; built by
`tagged-paragraph.PROVENANCE.py`): one page holding an `<H1>` at MCID 0 and a
`<P>` at MCID 1, with the StructTreeRoot's `/ParentTree` mapping the page's
`/StructParents 0` to both elements. The caret goes into "rose" in the `<P>`
line, at page point (113, 693).

1. Scripted click on `ribbon.tab.format`, then Ctrl+U.
2. Owed: `text-decorate-applied ... applied=1`.
3. Owed: the last `text-style-disclosed` line after the press carries the
   engine's sentence (`text_edit::decoration::tagged`) — matched on its tail
   "pdfcer does not split structure elements" — and names `<P>`.
4. Owed: the status bar declares `status-group:edit-disclosure`.

# Why not `tagged-report.pdf`

It has a structure tree but no `/ParentTree`. The engine finds the elements a
page's text belongs to through `/ParentTree` keyed by the page's
`/StructParents`, so on that file it finds none and says nothing — correctly.
It also feeds `export_tables_scripted` and `export_word_scripted`, so it was
not altered.

# Falsified

- On `tagged-report.pdf` (no `/ParentTree`) step 3 fails: no sentence.
- With both recorders of the edit's disclosures suppressed — the action
  funnel's and `textstyle::emit_carried`'s `record_edit_disclosure` — step 4
  fails. Suppressing only one still passes, because either records it.

# Driven off-screen

Scripted pointer and keys, window at `-4200,-4200`; it runs under
`--no-input`.
