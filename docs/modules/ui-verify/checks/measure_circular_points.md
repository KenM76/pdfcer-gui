# `ui-verify/checks/measure_circular_points`

`three_clicks_round_a_hole_measure_the_hole` — the operator's own report,
driven, on a fixture built to reproduce it.

# The defect, in his words and then in numbers


> *"can you check our radius/diameter dimensioning tool? selecting a point
> sometimes makes a big circle, and selecting more points around a hole
> doesn't always get it to narrow down to the size of the hole."*

The cause was measured, not guessed. The tool hit-tested for a **PDF path
object** and fed *every anchor of every subpath of that object* to the
circle fit. On his own drawing — `SW41177.pdf` page 1, read with
`pdfcer object-list` — three objects carry **4,405**, **4,972** and
**6,681** anchors, the largest holding 1,194 subpaths across a 550 × 500 pt
region. One click anywhere on it handed the fit six thousand points
scattered over half the sheet, and the circle through them is enormous.

# Why this check pins its own fixture and ignores `--pdf`

Because on a document with one tidy circle in its own object, **the defect
cannot occur**: a click contributes that circle's anchors, the fit is the
circle, and the broken build passes. A check that can only pass is not
evidence.

So `fixtures/hole-in-a-big-object.pdf` is built by
`tools/gen-hole-in-a-big-object-fixture.py` to have exactly the shape that
produced the report: **one** path object holding a 30 pt-radius circle *and*
forty unrelated segments spread across the page. Under the old build a
single click on the rim fits a circle through 85 scattered anchors —
hundreds of points of radius. Under the new one, three clicks on the rim fit
the rim. The two builds are told apart by a number.

⇒ This is the same discipline `ocr` follows for the same reason, and it is
why the check reports the fixture it used in its own notes: a sweep that
passed `--pdf` and had it ignored must be able to see that happen.

# What it asserts, and why each one is separately necessary

| Phase | Asserts | The build it fails |
|---|---|---|
| A | three clicks on the rim produce three `measure-circular-point action=add` lines, `n=1,2,3` | one where a click is still an object toggle — the second click on the same object would REMOVE, and `n` would read 1, 0, 1 |
| B | after the third, `r` is the hole's radius | one where a click contributes the whole object's anchors. This is the operator's sentence, as a number |
| C | the Tool panel lists one row per point | O107's panel half, which nothing else can substitute — a pick set on a dense sheet is invisible |
| D | clicking a row removes that point | O107's literal ask |
| E | clicking the canvas near a picked point removes it too | O107's other route — *"we should be able to unselect points/clicked locations"* |

**B is the load-bearing one and A alone would be worthless.** A build
that accepted three clicks and fitted them to the wrong geometry satisfies A
completely. The count says the clicks registered; only the radius says the
tool measured the thing under them.

# What it deliberately does NOT assert

That a **free** position works — O106. The fixture is vector geometry and
every rim click snaps, which is the correct behaviour on a drawing that has
geometry; asserting the free path here would mean aiming somewhere the snap
declines, which is a different fixture (a raster) and a different check.
`canvas::measure::circpick`'s unit tests cover the composition; the driven
half of O106 is unbuilt and is named here rather than left implied.

## Item notes

### `const PROPERTIES_BODY`

**The pick list moved on 2026-09-04** — `OPERATOR_REQUESTS.md` O123
dissolved the Tool panel and sent its live controls to Properties, on the
operator's own argument: *"I never understood why there is a tool dock when
everything can be in object and properties."*

The dock's own name is used rather than a panel-published body region
because Properties has never published one — and the dock's is the better
oracle anyway, since it goes through `crate::diag::ui_rect_visible` and
therefore says *the compartment is reachable* rather than *a function ran*.

### `const HOLE`

Hard-coded here **and** in the generator, which is a duplication with a
reason: the generator is the source and this is the *expectation*, and a
check that read its expectation out of the thing it is checking would pass
on any fixture at all. If the two ever disagree, the failure message below
names both numbers.

### `const RADIUS_TOLERANCE_PT`

Two points, not two per cent, and the difference matters. The failure this
separates from a pass is an order of magnitude — the broken build fits a
radius in the **hundreds** — so any threshold between "a few points" and
"half the page" tells the two apart. Two points is the loosest value that
still fails a build fitting the rim of anything other than this hole, and it
absorbs the two or three points of aim a real pointer costs at fit-page
zoom.

### `fn raise_properties_panel`

Three states, and the middle one is the reason this is a function:

1. **Already the active tab** — the dock publishes its body; nothing to do.
2. **Mounted, behind a sibling** — the dock publishes the tab header while
   its body does not. Clicking the header raises it. A ribbon toggle must
   NOT be used here: it would *unmount* a panel that is already there, and
   the check would then report an absent list about a panel it closed
   itself.
3. **Not mounted at all** — the check SKIPs. Deliberately a skip rather
   than a ribbon hunt: `file.properties` is mounted by every mode's default
   arrangement, so its absence means the operator's persisted layout removed
   it, and a check that re-mounted somebody's closed panel would be
   measuring a dock it had just rearranged.

# Errors

SKIPs the check when the panel cannot be reached, because a check that could
not open the surface it reads has learned nothing about it.
