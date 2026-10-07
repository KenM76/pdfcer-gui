# `ui-verify/checks/form_part_delete`

Three checks for O288 item 2, on parts of a placed drawing (a form XObject):

- `a_subpath_inside_a_placed_drawing_can_be_deleted`
- `an_anchor_inside_a_placed_drawing_can_be_deleted`
- `a_text_line_inside_a_placed_drawing_can_be_deleted`

## What it drives

`fixtures/form-parts.pdf` (built by `form-parts.PROVENANCE.py`): one 400x300
page, one form at (40, 40) holding two bars (leaf 0), a four-anchor polyline
(leaf 1) and a two-line text block (leaf 2). Edit mode. Each check enters its
leaf through `form_node_move::enter_leaf_at` (click the form, double-click to
the leaf), then:

- subpath: double-click the upper bar, which enters the Part rung;
- anchor: double-click the polyline, then double-click `canvas.anchor.1` and
  require `canvas.selected-anchor`;
- text line: a single click on "Second line", which picks that line as a
  chunk of the selected block (a double-click opens the text for editing).

Then the Delete key.

## Verdict

FAIL on a `canvas-delete-declined` line after the key, naming the refusal.
FAIL when no line for the expected verb follows (`delete-subpath-in-form`,
`delete-node-in-form`, `delete-text-line-in-form`), quoting the
`{verb}-refused` line if the engine refused. FAIL when that line has no
operand count or `n=0`. PASS otherwise; the funnel writes the verb's line
only after the engine returned `Ok`.

## Falsified

With `canvas::deleting::Address::of` returning `Refusal::InsideForm` for a
leaf (the shell's earlier rule), all three FAIL with `DELETE WAS REFUSED`
(`level=Part` for the subpath and line, `level=Node` for the anchor). With
the source restored, all three PASS with `n=1`.

Only the refusal arm was planted; the no-verb and no-count arms have not been
made to fire.

## What it does not prove

That the right line was removed from the saved file: the check reads the
funnel's count, not the content stream. Copying a leaf (G145) and resizing or
rotating one (G144) are not engine verbs yet.
