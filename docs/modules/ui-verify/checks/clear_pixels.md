# `a_click_on_a_clear_pixel_picks_the_ink_beneath`

**Defect it guards.** An image whose soft mask leaves part of it fully clear
still takes every click inside its placed parallelogram, so a line or shape
that shows through the clear part cannot be picked. The engine skips fully
clear samples in `pdfcer_core::vector::hit_test_point_deep_with` given an
`ImageAlpha`; the canvas's point query is
`objectprovider::ObjectModelProvider::hit_test_all`, which reads the masks
decoded into an `ImageAlphaCache` when the provider is built.

**Fixture.** `fixtures/clear-image-over-ink.pdf`, built by
`clear-image-over-ink.PROVENANCE.py`: a 400 x 300 page. Object 0 is a black
square from (220,100) to (300,200). Object 1 is a 2 x 1 image placed over
(100,100)-(300,200), painted after the square; its soft mask is opaque on the
left half and clear on the right half.

**Steps.** Launch on the fixture in Edit. Click at (260,150), where the image
is clear over the square: the new `selection-set` line must say `object=0`.
Click at (150,150), on the image's opaque half: the line must say `object=1`.

**Falsified** by passing `NoImageAlpha` in place of the filled cache in
`objectprovider::alpha::PageAlpha::hits`: the first click selects object 1.

**What it does not prove.** Partly transparent samples (only a fully clear
sample passes a click through), images inside a placed drawing, and a click
on a rotated image's corner outside its parallelogram, which the engine's
parallelogram test covers without a mask.
