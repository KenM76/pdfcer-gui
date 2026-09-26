# `ui-verify/checks/smart_select`

`a_click_selects_the_whole_drawing_and_a_double_click_goes_inside` —
**Smart-Selector, driven on a real wrapped CAD sheet.**

# The request


> *"if a click selects an object that is made of multiple objects (group,
> form, etc) a double click should bring me further down the chain … If I
> recall this is similar to how Inscape does things and we should follow
> that convention."*

## ★★★ The trace field this whole check turns on

`canvas-selection … first=` says **which of two index spaces** the selection
landed in — `object:N` for a page object, `leaf:N` for something painted
inside a form XObject. It exists because `sel=` and `level=` cannot tell
them apart:

```text
canvas-selection via=click mod=false sel=1 level=Object first=object:12   ← the container
canvas-selection via=enter-form mod=true sel=1 level=Object first=leaf:1180 ← inside it
```

Both are `sel=1 level=Object`. A check reading only those would pass on the
build this feature replaces **and** on the one it introduces, which is this
project's own definition of measuring nothing.

## What the two clicks must produce, and why that order is the feature

| # | gesture | oracle |
|---|---|---|
| A | one click on drawing content | `first=object:N` — the **container** |
| B | double-click at the same point | `smart-enter`, then `first=leaf:M` |
| C | Escape, Escape | `canvas-escape outcome=LeftContainer` |

★★ Step A is the half that sounds backwards and is the actual change. Before
this feature a click selected the **leaf** — the engine excludes forms from a
deep hit test, so the interior was all a click could reach and the wrapped
drawing itself was unselectable except through a Format-tab command. So a
build with the feature missing fails step A, not step B: it goes straight to
`first=leaf:…` on the first click.

★ Step C is two presses, not one, and the count is the assertion. `canvas::keys`
puts the container **below** the selection on the Escape ladder — one press
clears what is selected, a second steps out — because the selection is the
more transient of the two. A build that leaves on the first press would strand
an operator who pressed Escape to drop a selection outside the container they
were working in.

## ★★★ Why it opens its OWN fixture and ignores `--pdf`


| document | forms | what a driven click selected |
|---|---|---|
| `SW41177.pdf` | **none** — `/Subtype /Form` appears zero times | a page object, correctly; nothing to enter, ever |
| `ncored-benchmark-cad-drawing.pdf` | one, over **10,256 leaves** and 129,758 page objects | a page object at all three points tried: 119703, 1528, 64850 |

The second is the instructive one. Points were chosen by asking
`hit_test_point_deep` directly, with a 3 pt tolerance, and every one of them
still selected a page object when driven — because the shell asks with
`SELECT_SCREEN_TOLERANCE_PX` converted **at the current zoom**, and that
sheet opens at about 0.39×, making six screen pixels roughly fifteen points
of page. At that radius the big page objects win everywhere. The leaves that
do survive a tight probe are 4 × 6 pt glyph strokes — a pixel and a half.

⇒ So the feature is reachable by an operator who has zoomed in and
unreachable by a harness aiming at a page opened to fit. That makes the
DOCUMENT the wrong instrument, not the feature wrong, and the answer is
`tools/gen-form-xobject-fixture.py`: a 400 × 300 pt page whose entire
content is one `Do` on a form holding three fat crossing strokes. Its header
carries the measurements above and why each dimension is what it is.

★ The real drawings keep their checks — this suite drives them for
everything whose subject IS a real drawing. This one's subject is a
containment relationship, and a fixture states it exactly.
