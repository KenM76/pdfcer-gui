# `ui-verify/checks/form_text_reshape`

Three checks on a text object inside a placed drawing (a form XObject):

- `a_text_line_inside_a_placed_drawing_can_be_merged`
- `a_text_inside_a_placed_drawing_can_be_split_into_lines`
- `a_text_line_inside_a_placed_drawing_takes_a_typed_width`

## What it drives

`fixtures/form-text-runs.pdf` (built by `form-text-runs.PROVENANCE.py`): one
form at (40, 40) whose leaf 0 is a text object of two lines. Line 0 is two
show operators, "Two" and " runs" (baseline page y = 100); line 1 is one,
"Second line" (baseline page y = 76). Edit mode; each check enters the leaf
through `form_node_move::enter_leaf_at`, then:

- merge: a click on "Two" picks line 0 at the Part rung; the canvas menu's
  `format.merge_text_runs` row is picked;
- split: with the whole text selected (Object rung), the canvas menu's
  `format.split_text_lines` row is picked;
- width: launched with `PDFCER_DIAG_INVOKE=file.properties`; a click on
  "Second line" picks line 1, and `150` is typed into
  `properties.text.run-width` and committed with Enter.

## Verdict

FAIL when the menu row is absent (an error naming the rows offered), when no
`merge-text-runs-applied` / `split-text-lines-applied` / `text-run-width`
line follows (quoting any decline line), or when that line lacks
`in_form=true`. Then merge needs `merged=2`, split `pieces=2`, width
`detail=applied`. PASS otherwise.

## Falsified

With each leaf arm routed to the page-object verb (the leaf index read as a
page-object index), all three FAIL: merge with no applied line, split with
`split-text-lines-declined reason=other`, width with the engine's refusal of
object 0 (an image: the form itself). With the source restored, all three
PASS.

The `in_form` arm, the absent-row arm and the count arms have not been made
to fire.

## What it does not prove

That the saved form stream holds the merged, split or widened text: the
checks read the funnel's trace, written only after the engine returned `Ok`.
The shared-content remedy is not exercised: the fixture draws its form once.
