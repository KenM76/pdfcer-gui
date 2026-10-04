# `ui-verify/checks/choice_defaults`

`a_multi_select_lists_defaults_can_be_chosen` — a multi-select list's default
checkboxes in the Properties panel record several default choices, read back
from the document.

# What it drives

A copy of `fixtures/all-field-kinds.pdf` (ignores `--pdf`), launched through
`properties_pane::launch_on_field` with `PDFCER_DIAG_SELECT_FIELD=ListMulti`.
`ListMulti` offers Mon–Fri, has a value (`/V [(Mon) (Wed)]`) and no `/DV`.

1. Before anything is pressed: `choice-defaults-read field=ListMulti defaults=-`.
2. `properties.choice_opts.default.1` → `defaults=Tue`.
3. `properties.choice_opts.default.3` → `defaults=Tue|Thu`.

Step 3 is the one that distinguishes a multi-select default from a single
one: the second pick must join the first, not replace it. The read line is
drawn from the field's `/DV` after the edit epoch, so each step proves the
change reached `edit_field`.

# Falsified

Clearing the picked list whenever a row changes fails step 3 with
`defaults=Thu`.

# Driven off-screen

Scripted pointer, window at `-4200,-4200`, `spec.place = false`; it runs under
`--no-input`.
