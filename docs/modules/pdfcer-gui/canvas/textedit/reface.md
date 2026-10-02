# `canvas::textedit::reface` — planning a face for keys the font lacks

At the keystroke, `plan` runs the same pre-flight the Properties face chooser
reads with the missing characters plus up to `PLACEHOLDERS` (3) of the run's
own characters as candidates, keeps the faces that take them all, and picks
the nearest to the run's face (`editmodel::nearface::nearest`). On a pick the
keys go into the draft and the plan is held in egui memory (`textedit-reface`)
for the page and run; `refused` shows it as `Planned`.

`take` at the commit turns the held plan into the `Reface` the action
carries, keeping only characters still in the draft. `forget` drops it when
the draft is abandoned. `is_planned` lets the preview report
`PreviewFallback::Reface` instead of a shaping failure.

The diagnostic seed (`PDFCER_DIAG_TYPE`) bypasses `plan`.

Trace: `text-edit-reface-planned page= run= characters= face=`.
