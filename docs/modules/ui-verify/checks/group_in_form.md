# `ui-verify/checks/group_in_form`

`parts_of_a_placed_drawing_group_inside_it_and_ungroup`, on
`fixtures/form-parts.pdf`: one placed drawing holding a two-bar path, a
polyline and a text object.

PASS needs, in order:

1. A click and a double-click on the upper bar select it, a part of the
   drawing; a Shift-click on the polyline traces `canvas-selection sel=2
   first=leaf:`.
2. Ctrl+G traces `group-in-form-applied members=2` and `leaves=` naming two
   parts: the members found again inside the new group after the engine
   renumbered the parts.
3. A click on the bar selects one part; Ctrl+Shift+G traces
   `ungroup-in-form-applied` whose `form=` is the `group=` of step 2 and whose
   `container=` is step 2's `container=`, not `0`: the group came apart into
   the drawing that held it, not onto the page.

## Falsified

- The build before Group inside a drawing existed → FAIL at step 2: the trace
  reads `command-declined id=markup.group reason=nothing-to-group`.

## What it does not prove

- The status-line sentences or the shared-drawing remedy; the fixture places
  its drawing once (`invocations=1`).
- The refusals: parts of two drawings, a drawing without its own resources,
  nesting past the engine's limit. They reach the status line through
  `actions::group::refusal_for`, unit-tested there.
- That the page still paints the same; the engine's.
