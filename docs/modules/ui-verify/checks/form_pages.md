# `ui-verify/checks/form_pages`

`use_these_pages_makes_the_form_the_document` — Form preview's *Use these
pages* writes a dynamic XFA form's laid-out pages into the document in place
of its stand-in page, says so on the status row, and closes the window.

# What it drives

A copy of `fixtures/dynamic-xfa.pdf`, opened off-screen under `--no-input`
with a `ScriptedPointer`. The form opens Form preview by itself.

1. Wait for the `form-preview` trace line; without it the check fails.
2. Find `form-preview.use` among the preview's declared regions and click it.
3. Require `form-pages-applied` with `written=1 replaced=1`: the fixture's one
   stand-in page became the form's one computed page.
4. Require a `form-pages` funnel line carrying an `epoch=` and a disclosure
   other than `none`: the edit committed and the status row said something.
5. Require that `dialog:form-preview` is no longer declared.

Pictures before and after the press go to the output folder; the verdict is
the trace.

# What would make it fail

The button pushing no action, the applier writing nothing, the funnel
dropping the disclosure, or the window staying open. Disconnecting the
button's push gives `pressing Use these pages traced no form-pages-applied
line`.
