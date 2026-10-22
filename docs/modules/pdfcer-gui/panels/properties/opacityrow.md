# `panels::properties::opacityrow` — one opacity field, or why there is none

Shared by Line and opacity (`strokestyle`) and the clicked-text section
(`textobject`).

## Contract

- `Alpha::of(values)` takes each member's opacity as `Option<f64>` (`None` =
  the engine could not read it) and answers `None` for no members,
  `Unknown` if any member is unreadable, else `Read { first, mixed }`.
- `row(ui, label, alpha, region)` draws the label and either a 0–100 % drag
  field (published as `region`) or, for `Unknown`, a weak sentence (published
  as `{region}.unknown`). It answers `Some(0.0..=1.0)` once, when a drag stops
  or the field loses focus on a changed value, so a drag is one rewrite and one
  undo entry.
- A mixed reading shows the first member's value with a note that a new value
  sets them all.
- `Alpha::token` is the trace token: the value to three places, `mixed` or
  `unknown`.

## Why no field over an unreadable opacity

A field has to open on some number. Opened on 100 % over a run whose real
opacity is unreadable, a click and Enter would overwrite a value nobody saw.
The engine refuses the write in that state anyway (`OpacityUnrestorable`), so
a field would be a control that can only fail.
