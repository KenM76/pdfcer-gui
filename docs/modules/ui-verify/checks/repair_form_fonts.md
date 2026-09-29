# `ui-verify/checks/repair_form_fonts`

`repair_form_fonts` — **Edit ▸ Forms ▸ Repair fonts on a form that keeps its
font inline moves it, and a second press says there is nothing left.**

## What it guards

Acrobat draws a filled text field blank when the field's `/DA` font resolves to
an inline entry in `/AcroForm /DR /Font`. `edit.form_repair_fonts` dispatches
`Action::RepairFormFonts`, which runs `EditSession::promote_inline_dr_fonts`
through the edit funnel. A count above zero is one undo entry and a status
line; zero is a worded decline and leaves the document alone.

## How it drives

1. It opens `fixtures/inline-dr-font.pdf` (built by its `PROVENANCE.py`): one
   filled text field, with `/Helv` held inline in `/DR /Font`.
2. It selects Edit mode, clicks `ribbon.tab.edit`, and finds
   `ribbon.item.edit.form_repair_fonts` on the band or in a collapsed group.
3. It presses it and requires `form-repair-fonts-applied fonts=1`.
4. It presses it again and requires no second `applied` line and a
   `form-repair-fonts-refused` line whose detail is `no inline /DR font`.

The second press is what tells a real repair from one that reports a count
and leaves the font inline: that build moves it again.

## Falsification

`promote_inline_dr_fonts` is replaced by a call answering `Ok(0)`. The check
then fails at step 3, quoting the refusal line.
