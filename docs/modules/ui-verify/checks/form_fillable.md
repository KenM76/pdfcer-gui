# `ui-verify/checks/form_fillable`

`use_these_pages_keeps_the_fields_fillable` — Form preview's *Use these
pages*, with *Keep the fields fillable* left at its default, writes the
form's fields as fillable fields, and one of them then fills in on the canvas
through the ordinary form-fill route.

# What it drives

A copy of `fixtures/dynamic-xfa.pdf` (three fields: Tenant, Rent, Deposit),
opened off-screen under `--no-input` by `form_pages::launch_form`.

1. Press `form-preview.use` without touching the switch, through
   `form_pages::press_use` (which judges the write, the disclosure and the
   window closing).
2. Require `form-pages-applied` with `fillable=true fields_written=3`.
3. Read the canvas's `form-box kind=text` census lines after that line, and
   take the first. Its `rect=` is in canvas space; the page is US letter.
4. Click its centre through `CanvasMapping`, require `form-focus`, press
   Ctrl+A, type `Ann`, press Enter.
5. Require a `form-fill-text` line after the Enter, carrying `commands=1`.

# What would make it fail

- Inverting the switch's `FieldOutput` gives `must carry fillable=true
  fields_written=3`; a switch defaulting off gives the same line.
