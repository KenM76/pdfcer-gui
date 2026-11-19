# `ui-verify/checks/form_pages`

`use_these_pages_makes_the_form_the_document` — Form preview's *Use these
pages* writes a dynamic XFA form's laid-out pages into the document in place
of its stand-in page, says so on the status row, and closes the window. It
turns *Keep the fields fillable* off first, so the pages are flat.

# What it drives

A copy of `fixtures/dynamic-xfa.pdf`, opened off-screen under `--no-input`
with a `ScriptedPointer`. The form opens Form preview by itself.

1. Wait for the `form-preview` trace line; without it the check fails.
2. Click `form-preview.fillable` (the switch, on by default), then find
   `form-preview.use` among the preview's declared regions and click it.
3. Require `form-pages-applied` with `written=1 replaced=1`: the fixture's one
   stand-in page became the form's one computed page.
4. Require a `form-pages` funnel line carrying an `epoch=` and a disclosure
   other than `none`: the edit committed and the status row said something.
5. Require that `dialog:form-preview` is no longer declared.
6. Require `fillable=false fields_written=0` on `form-pages-applied`, and no
   `form-box` canvas census line after it.

`launch_form`, `launch_fixture` and `press_use` are shared with
`form_fill`, `form_fillable` and `form_widgets`.

Pictures before and after the press go to the output folder; the verdict is
the trace.

# What would make it fail

The button pushing no action, the applier writing nothing, the funnel
dropping the disclosure, or the window staying open. Disconnecting the
button's push gives `pressing Use these pages traced no form-pages-applied
line`. Inverting the switch's `FieldOutput` gives `must carry fillable=false
fields_written=0`.
