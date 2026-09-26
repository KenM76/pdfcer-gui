# `pdfcer-gui/canvas/shapes/tests`

## Item notes

### `fn polyline`

`polyline-nodes.pdf` exists in this repository precisely because no engine
fixture carried a node-draggable polyline — see `open_local_fixture`'s
header on why there are two fixture roots.

### `fn the_walk_agrees_with_the_providers_anchor_numbering`

# Why this test is the important one in this file

[`super::with_nodes_moved`] displaces *anchor number N* by walking
`page_subpaths()` and counting `start` then one per segment. The provider
counts the same way, through `Subpath::anchors()`, and the engine's
`move_node` counts the same way again.

**Three implementations of one enumeration.** R74's rule is that a matching
rule must not be re-derived in the shell; index arithmetic that has to agree
with another module's is the same hazard in smaller clothes, and the failure
it produces is the worst kind — the preview bends the line at one end and the
commit bends it at the other, and *both look deliberate*.

⇒ So the agreement is asserted rather than reasoned. If the engine ever
changes what an anchor index counts, this goes red here rather than shipping
a preview that lies.

### `fn a_translation_moves_every_point_by_exactly_the_translation`

The failure this catches is a preview that builds, traces, paints, and shows
the shape exactly where it already was — which looks like "the preview is not
working" and is indistinguishable from the feature being absent.

### `fn an_identity_transform_lands_on_the_object`

Not a tautology: it is the assertion that `page_subpaths()` and the
provider's own numbers describe the same shape, so a preview built with no
gesture in progress would sit precisely on top of the rendered object rather
than a fraction away from it. A preview that is half a point out looks like a
rendering bug.

### `fn a_selection_past_the_cap_is_bounded_and_says_so`

Both halves matter. An unbounded preview turns the gesture this feature
exists to smooth into the slowest thing in the program; a preview that
silently returned nothing would read as the feature being broken on exactly
the drawings it was built for.

### `fn the_stroke_width_survives_a_rotation_and_follows_a_scale`

The rotation half is the one worth having. Reading `a` and `d` off the
matrix — the obvious implementation — reports a shape rotated by 90° as
having zero width, so the preview of a rotate gesture would fade out as it
turned. `average_scale` uses the axis lengths instead.
