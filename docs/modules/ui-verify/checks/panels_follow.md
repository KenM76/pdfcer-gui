# `ui-verify/checks/panels_follow`

Two checks on the side panels following the operator (design: `DESIGNS.md`
§ O414 and § O416). Both open `fixtures/two-photos.pdf` in Edit mode with the
default layout, where Properties and Objects share one stack on the right,
and first bring the Objects tab to the front.

## `properties_comes_forward_for_an_armed_tool_and_gives_the_front_back`

1. Arm `ribbon.item.measure.linear` on the Measure tab; require
   `properties-forward tool=true`.
2. Press Escape; require `properties-back to=view.panel_objects`.

## `objects_tree_follows_an_object_picked_on_the_page`

1. Click the second picture's centre on the page; require an
   `objects-follow` line reading `shows=true` (two rows always fit).
2. Require no `properties-forward` after it: a pick with the tree in front
   leaves the tree there.

## What a failure means

| arm | failure |
|---|---|
| arming | the tool's options stayed behind the Objects tab |
| Escape | Properties kept covering the tab the operator had in front |
| page pick | the tree did not look for the picked row |
| no forward | Properties took the tree's place although the tree shows the pick |
