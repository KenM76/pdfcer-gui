# `canvas::textedit::reface` — planning a face for keys the font lacks

At the keystroke, `plan` runs the same pre-flight the Properties face chooser
reads with the missing characters plus up to `PLACEHOLDERS` (3) of the run's
own characters as candidates, keeps the faces that take them all, and picks
the nearest to the run's face (`editmodel::nearface::nearest`). On a pick the
keys go into the draft and the plan is held in egui memory (`textedit-reface`)
for the page and run; `refused` shows it as `Planned`.

`take` at the commit turns the held plan into the `Reface` the action
carries, keeping only characters still in the draft. `forget` drops it when
the draft is abandoned. `face` hands the preview the planned face, which it
passes as `EditOptions::with_fallback`, so the keys preview in that face when
the run is one show operator. `is_planned` lets the preview report
`PreviewFallback::Reface` when the engine still cannot lay it out (a run
across several operators).

The diagnostic seed (`PDFCER_DIAG_TYPE`) bypasses `plan`.

## Choosing the face

`nearest` first asks for the face nearest the run's own among those the
font preflight says take the keys, and confirms it with
`run_repertoire_with` under `with_fallback(face)`: when `via_fallback` holds
every key, the plan needs nothing else (`route=engine`). Otherwise the face
must also take up to three of the run's own letters, the placeholders the
multi-operator commit route types and overwrites (`route=placeholders`); a
run with no letters to borrow then has its keys refused.

Trace: `text-edit-reface-planned page= run= characters= face= route=`.
