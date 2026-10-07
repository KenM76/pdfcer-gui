# `ui-verify/checks/form_part_copy`

`copying_a_part_of_a_placed_drawing_says_why_nothing_was_copied`, for O288
item 2.

## What it drives

`fixtures/form-parts.pdf`, Edit mode. `form_node_move::enter_leaf_at` selects
the polyline inside the placed drawing (leaf 1), then the scripted Copy
command (`pointer.copy`).

## Verdict

The engine copies page objects only (G145), so a selection made of parts of a
placed drawing cannot be copied. FAIL when a `clipboard-copy` line follows
(something claims to have been copied). FAIL when no
`clipboard-copy-refused reason=inside-form` line follows, or it carries no
`n`. FAIL when the status bar draws no `status-group:edit-disclosure`. PASS
otherwise.

## Falsified

With `canvas::clipboard::copy`'s inside-form branch disabled (the selection
falls through to `Refusal::NothingSelected`, which is untraced and says
"nothing is selected"), the check FAILs with `after Copy: no refusal`. With
the source restored it PASSes with `reason=inside-form n=1`.

## What it does not prove

The sentence's wording: the trace declares the disclosure region, not its
text; `text::clipboard`'s tests cover the wording. A mixed selection (page
objects plus parts of a placed drawing) copies the page objects and names the
parts left behind through `partial_copy`; that path has unit tests only.
