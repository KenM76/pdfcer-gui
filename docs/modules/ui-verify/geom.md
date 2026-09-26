# `ui-verify/geom`

## Item notes

### `fn contains_rect`

# Wholly, not partly, and the difference is what a check means by it

The question a driven check asks is *"can the operator click this?"*,
and a control half-outside its container is one whose visible half may
be the half without the label on it — or, in a scroll area, one whose
clipped remainder is what the click would land on. `intersects` would
answer yes for a button showing one pixel of its top edge.

### `fn is_substantial`

Used as a sanity check on trace-supplied rects. A zero- or
negative-area rect means the widget was never laid out, and converting
a document point against it would produce a plausible-looking screen
coordinate pointing at the window's top-left corner — a click that
lands on the wrong thing rather than nowhere, which is much harder to
diagnose than an outright failure.

### `struct FracRect`

**The only rectangle a check may write as a literal.** See the module docs.

The containing surface is named by whoever resolves it — a window's client
area in live mode, the whole image in `--image` mode — and a
[`crate::profile::RegionSet`] states which one it was calibrated against, so
a region set cannot be silently applied to a surface it does not describe.

### `fn resolve`

Clamps to the surface and guarantees a non-degenerate result whenever
the surface itself is non-degenerate: a region that rounds to zero
width would make [`crate::pixels::contrast_at`] sample nothing and
report a contrast of 1.0, which is indistinguishable from an invisible
caption. Reporting "the region is empty" as "the text is invisible"
would be a false FAIL, and false failures are how a gate gets ignored.
