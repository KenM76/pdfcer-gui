# `app::actions::vector::leaftransform` — resize or rotate parts of a placed drawing

Commits `VectorAction::TransformLeavesInForm`: one
`EditSession::transform_objects_in_form(page, leaves, matrix, TransformOptions::default())`
call through `vector_edit_on_page`, so one undo entry and a page-scoped
re-render.

## Where it comes from

The canvas resize grips (`canvas::resizing`) and the rotate handle
(`canvas::rotating`). Each reads the selection's page objects first; when
there are none it reads its form leaves (`SelectionState::leaf_indices_on`)
and builds the same PAGE-space matrix it would for a page object. The engine
maps that matrix into the form's own space, so the shell never composes the
placement CTM itself.

A selection holding both page objects and leaves transforms the page objects
only; the engine's verb takes leaves of one form and nothing else.

## Refusals

The engine refuses a leaf list spanning two forms
(`FormLeafSelectionSpansForms`), an index the form does not have
(`FormLeafOutOfRange`), a placement it cannot invert (`DegenerateCtm`) and a
matrix with no inverse (`SingularTransform`). Each reaches the operator as the
funnel's status sentence and the `transform-leaves-in-form-refused` trace line.

## Shared drawings

The edit rewrites the form's content, so every placement of that form changes
— other invocations on this page and on other pages. The engine reports the
reach as `invocations` and `pages` on its outcome; the shell traces both and
passes the engine's disclosures to the status notes.

## Trace

`transform-leaves-in-form-applied page= leaves= invocations= pages= m=[a b c d e f]`,
written only when the engine applied the edit. `m` is the page-space matrix as
handed over, so a build that sent the identity, or a matrix in form space,
shows it on the line.

The page-object twin's `transform-objects-applied` line follows the same
rule: written only on success, so a check reading it cannot pass on a refused
edit.
