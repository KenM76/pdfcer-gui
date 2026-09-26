# `egui-shell/ribbon/overflow`

## Item notes

### `fn region`

The RIGHT arrow's name is `ribbon.overflow`, which names the question
rather than the glyph. Region names are a **cross-repo stability
contract** with `tools/ui-verify`, and what those checks ask of this
control is: is the affordance on screen at every width, is it
hit-testable under real metrics, does any visible group overlap it.
None of that is a claim about the mechanism, so naming the region after
the mechanism would put an implementation detail into a cross-repo
contract.

The left arrow answers a question nothing asked before, so it carries a
name of its own.
