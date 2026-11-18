# `ui-verify/checks/form_fill`

`typing_into_a_form_preview_field_fills_the_form` — a click on a field in
Form preview opens it for typing, and Enter writes the value into the form's
data, says so on the status row, and redraws the preview.

# What it drives

A copy of `fixtures/dynamic-xfa.pdf`, opened off-screen under `--no-input`
with a `ScriptedPointer` (launched by `form_pages::launch_form`). The form
opens Form preview by itself.

1. Wait for the `form-preview` trace line, and keep its `digest=`.
2. Click the region `form-preview.field.0.1` (Rent, holding 1250), press
   Ctrl+A, type `975`, press Enter.
3. Require `form-fill-applied` with `chars=3`.
4. Require a `form-fill` funnel line carrying an `epoch=` and a disclosure
   other than `none`.
5. Require a `form-preview` line after the fill whose `digest=` differs from
   the first. The digest hashes the drawn pages' pixels. The preview is an
   immediate viewport, and the screenshot seam cannot capture one.
6. Require `form-preview.field.0.1` still declared.

# What would make it fail

- Disconnecting the overlay's push gives `traced no form-fill-applied line`.
- Removing the redraw on the edit epoch gives `the preview drew no page after
  form-fill-applied`.
