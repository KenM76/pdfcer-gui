# `pdfcer-gui/dialogs/print/preview/preview_tests`

## Item notes

### `fn on_screen`

Mirrors [`paint`]'s own `origin` computation so the anchor tests below
assert the property that matters — "this point did not move" — rather
than re-stating the formula they are meant to be checking.

### `fn ctrl_wheel_zoom_holds_the_point_under_the_pointer_still`

This is the whole reason the anchor term exists, and it is the one
property a reader can check without re-deriving the algebra. Asserted
on an OFF-CENTRE point, because every wrong version of this formula —
including simply omitting the term — is correct at the centre.

### `fn a_refused_zoom_leaves_the_pan_exactly_where_it_was`

The bug this pins is subtle and would look like a hardware fault: at
maximum zoom the wheel stops magnifying but keeps sliding the sheet
sideways, so the preview appears to drift on its own. It comes from
using the REQUESTED step for the anchor term instead of the effective,
post-clamp ratio.

### `fn a_large_format_sheet_is_capped_by_pixels`

The bound that matters. An ANSI E sheet at the target DPI would be
5100 x 6600 px and about 134 MB of RGBA for a picture drawn 300 pt
wide — and CAD sheets are exactly the population this project's
operator prints, so this is the common case, not the exotic one.

### `fn the_pixel_ceiling_binds_above_the_office_sizes`

The regression the ceiling's own doc comment records is a value chosen
too low: 1600 px silently downgraded Letter, Legal and A4 — the common
case — in order to bound the rare one. Asserting only that those three
are uncapped would let the constant drift *upward* unnoticed instead,
so the boundary is pinned from both directions.

**A3 is on the capped side, and that is correct rather than a
near-miss.** Its long edge is 1191 pt, which at the target DPI is
2481 px — past the 2200 ceiling. A3 is a drafting sheet, not an office
page, so it belongs with the large-format population this bound exists
for; US Legal at 2100 px is the largest size that clears it. If either
constant moves, this test says which side of the line each size landed
on rather than merely that something changed.

### `fn the_overhang_band_is_the_same_fraction_of_the_page_at_every_zoom_and_pan`

[`super::ink::InkMask`] speaks 0..1 page space and knows nothing about
the canvas. Everything the preview does — the fit, the operator's zoom,
their pan, and the placement's own scale — reaches the mask only through
[`normalised_in`], whose `whole` is the page's on-screen rectangle. If
that mapping is wrong the mask is asked about **the wrong part of the
page**, and the failure is quiet: a hatch appears somewhere plausible
and covers the wrong ink.

So the property asserted is the one that matters — the same band comes
back as the same fraction of the page no matter where or how big the
page is drawn — rather than a re-statement of the arithmetic.

### `fn a_blank_overhang_hatches_nothing_and_says_so`

The page overhangs on both axes — the placement reports a clip and the
job-wide count will still name this sheet — but the overhanging bands
are empty paper. Nothing may be drawn, and the verdict must be
`BlankBand` so [`column`] can print the sentence that stops the count
and the picture contradicting each other.

The two halves are asserted **together**, in one call, because the whole
point of [`lost_regions`] returning both is that they cannot disagree.

### `fn an_inked_overhang_hatches_only_the_ink_and_says_so`

The distinction the old code could not make. Before O113 this case and
the one above drew the identical full-height red band; the assertion on
the hatched area is what separates them.

### `fn no_raster_falls_back_to_the_whole_band_rather_than_to_silence`

The degraded state [`texture_for`] documents. "We could not look" must
not present as "nothing is lost" — a missing raster is not allowed to
switch a warning off, so the fallback is the pre-O113 behaviour.

### `fn the_far_edge_overhang_bands_do_not_overlap_or_reach_into_the_printable_area`

This fixture's page sits inside the printable area on its left and top, so
two of the four bands are empty and dropped — which is also the
evidence that O208's widening left an unmoved page alone. The four-edge
case is
[`a_page_dragged_off_all_four_edges_hatches_four_disjoint_bands`].

The old code took `right.union(bottom)`, and `Rect::union` is a
**bounding box**, not a set union: the union of a tall strip on the
right and a wide strip along the bottom also covers the region that is
neither right of nor below the printable area — paper that prints
perfectly. This pins that the two bands now meet without overlapping,
and that neither reaches back into the printable rectangle.

### `fn a_page_dragged_off_all_four_edges_hatches_four_disjoint_bands`

This state was unreachable before the operator could move the page:
`place_page` clamps its offsets at zero, so the near edges could not
overhang and the hatch marked two edges because only two were possible.

The assertion is not "there are four rectangles" — that would pass on
four wrong rectangles. It is the pair of properties a set union has and a
hand-written band list does not: **pairwise disjoint**, so no corner is
hatched twice and reads as a darker patch, and **total area equal to
`placed` minus `printable`**, so nothing is missed and nothing is drawn over
paper that will print. Those two together pin the geometry without naming a
single band, which is what stops this test from being a restatement of the
code it is checking.

### `fn a_page_dragged_off_the_near_edge_hatches_the_near_edge_only`

The half of O208 a two-band hatch was silent about: an operator who drags a
drawing left to bring its right-hand side onto the paper is choosing to lose
the left-hand side, and a hatch that marked only the far edges would show
nothing at all for the crop they had just chosen.

Also pins the converse, which is the regression the widening could cause: a
page that overhangs only the left must NOT grow a band on the right.

### `fn the_preview_upload_reads_pixels_as_premultiplied`

The same fixture `render::raster`'s own test uses — a half-transparent
red pixel stored the way `tiny-skia` stores it (`R·A, G·A, B·A, A`).
Read as *unmultiplied*, epaint would take the red channel at face value
and re-multiply it, yielding `r = 64`; read as premultiplied it
round-trips.

This test exists because [`upload`] is a **second** call site for a
convention that module says must have exactly one. Until that is fixed
there, this is what stops the two drifting silently — and the failure
mode being defended against is not a crash but every antialiased glyph
edge in the preview quietly darkening.
