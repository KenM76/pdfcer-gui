# `ui-verify/checks/page_display_recentres`

`switching_the_page_display_recentres_and_a_facing_fit_fits_the_spread` —
**O177**, both halves, driven.

# The report


> *"when switching the view from scroll pages to show one page at a time or
> show two pages side by side the page or pages view should snap back to
> center of the canvas. also fit page when in 2 pages side by side views
> should fit the two side by side pages onto the canvas - right now it snaps
> to fitting one."*

Two sentences, two independent defects, and they are checked in one run
because they share a fixture, a window size and a display mode — not because
they share a cause. They do not:

| half | what is wrong |
|---|---|
| **recentre** | `Action::SetPageDisplay` deliberately suppresses the scroll-to-page that a page change would otherwise cause, so the continuous strip's offset survives the switch and the new arrangement is drawn wherever that offset happens to land |
| **fit the spread** | the fit's **scale** is computed from the ROW (`viewer::strip::fit_metrics`, which is facing-aware) and the fit's **placement** is converted back through the acting PAGE's rect (`canvas::offset`'s `strip_offset_for`), so a spread is placed as though the page were the thing being centred — one page lands in the middle and the other runs off the edge, which reads exactly as *"it snaps to fitting one"* |

That asymmetry is the finding worth carrying away: **when a feature's
scale rule is taught about a new layout unit and its placement rule is not,
the symptom presents as the scale being wrong.** The operator's sentence
says "fit", and the zoom was never the problem.

# Why `canvas-strip` exists and why neither half could be checked
without it

The canvas has published `page` (the acting page's drawn rect) since Phase 1
and `canvas-viewport` since O23. Under a facing mode `page` is **half of
what the operator is looking at**, so a check asserting "the thing on screen
is centred" from it would have to reconstruct the other half from a
PDF-scanned page size and a hard-coded spread gap — i.e. re-derive the
application's own layout arithmetic in the harness, and agree with it. That
is the defect class `crop=` and `rot=` were added to the canvas trace to
end.

`canvas-strip` is the union of the rects the canvas actually **drew** this
frame, folded from the same `drawn` vector the rasters went into. It cannot
disagree with what was painted, and it degenerates to the page rect under
`Single`, which is what lets one assertion cover both halves.

# ⚠ Why every assertion is made in a NON-continuous mode

`canvas-strip` is the union of the pages drawn *this frame*. Under a
continuous mode that is the visible run of the strip, which is by
construction about the size of the viewport and about centred in it —
**always**, on a correct build and on a broken one alike. Asserting
containment or centring there would be asserting a tautology.

So the run uses continuous only to *create* the displaced state, and every
claim is made after the switch out of it. The displacement itself is
measured from the acting page's `rect=` on the `canvas` trace line, which
does move when the strip scrolls.

# The fixture, and the cover rule

`fixtures/four-pages.pdf`, pinned rather than taken from `--pdf`, because
the facing half needs a document with a genuine two-page row and the
operator's own drawings are frequently single-sheet.

`viewer::display::PageDisplay::row_of` implements the **cover rule**:
row 0 holds page 0 **alone**, and rows 1.. hold `2r-1` and `2r`. A facing
fit measured at launch therefore measures **one page** and would pass on a
build that cannot fit two. The run presses **Page Down** once to reach page
index 1 before it measures anything, and then asserts that what it is
looking at really is wider than one page — a precondition that fails as a
SKIP rather than being assumed.

# What a passing run does NOT prove

That the zoom is the largest one that still fits (that is `fit_metrics`'
own subject and is unit-tested), that the spread's *gap* is right, or that
anything is correct under a continuous mode — see the warning above.

## Item notes

### `const VIEWPORT`

Fixed rather than maximised so the numbers below mean the same thing on
every machine, and wide enough that a two-page spread at a readable zoom is
a shape the fit can actually produce. `PDFCER_DIAG_VIEWPORT` switches
`with_active` off, so the window lays out fully without taking the desktop.

### `const CENTRE_TOLERANCE_PT`

Generous on purpose. The page rect is rounded to the pixel grid, the fit
divides in `f32`, and a scroll bar appearing or disappearing moves the
viewport by its own width. The defect this check is about moves the strip by
**half a page** — 300-plus points at the zoom this run reaches — so a
tolerance two orders of magnitude below that separates the two states with
room to spare, and a tighter one would report arithmetic rather than
behaviour.

### `const DISPLACED_PT`

A precondition that is ASSERTED, not assumed. A run whose scroll did
nothing would switch display modes from an already-centred start and pass
while measuring nothing — the exact shape `fit_places_the_view`'s own pan
precondition exists to prevent.

### `const SPREAD_RATIO`

1.5 rather than 2.0: the row is two pages **plus** a gap, so the true
ratio is a little over 2, and a floor at 1.5 is unambiguous against the
thing it has to exclude — a row of one page, ratio exactly 1.

### `fn strip_and_viewport`

Both are read from the **same** trace snapshot: reading them from two
snapshots is how a check comes to compare a strip from one frame against a
viewport from another, which on a frame where a scroll bar appeared is a
difference of fifteen points for no reason at all.
