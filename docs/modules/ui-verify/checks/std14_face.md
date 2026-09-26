# `ui-verify/checks/std14_face`

`the_face_chooser_offers_a_face_the_document_does_not_contain` — **open the
font list, pick a face that is not in the file, and the file changes.**

# What this is for

`pdfcer-core` v0.15.0 (`Pass 162.0`) closed the last of the four things the
operator named as not fully editable:

> **FONTS** — text can be restyled to a face the document **DOES NOT
> CONTAIN**, for the fourteen faces every PDF reader is required to have.
> pdfcer authors the font resource on demand, with widths, embedding nothing.

The engine shipped it and **the shell could not reach it**: the chooser built
its list from `preview_font_resources`, which by construction enumerates the
*page's own* `/Font` resources, so the one thing the release note is about
was absent from every surface in the program. That is the specific defect
this check exists to detect — a capability that is present, tested, released
and unreachable.

## ★★ The way this feature can pass every unit test and be dead

`panels::properties::face::choices` is unit-tested: give it a page carrying
`ArialMT` and it answers fourteen addable faces. **Every one of those tests
would still pass on a build where the popup never draws them** — where the
two group headings are never reached, where the disclosure is drawn off the
bottom of a 78-point-wide popup, where the row is drawn and the click parks
nothing, or where the selector sent is the label of the row above.

The chain this drives:

| # | link | its own test |
|---|---|---|
| 1 | a sweep produces a `TextSelection` and the panel draws a Text section | `restyle_text` |
| 2 | the chooser's combo is on screen and opens | ★ **nothing** |
| 3 | the popup separates the two kinds of row and draws the standard-14 heading | ★ **nothing** |
| 4 | ★★★ **the disclosure is on screen where the choice is made** | ★ **nothing** — and no unit test can see it, because what is under test is that a string reached a rectangle |
| 5 | a standard-14 row is clickable | ★ **nothing** |
| 6 | the click reaches `format_text` and the document changes | unit-tested |

Link 4 is the one this check would be worth writing for on its own. pdfcer
*"authors the font resource on demand, with widths, embedding nothing"*, so
the restyled text is drawn with **the reader's own copy** of that face —
invisible on this screen and visible on somebody else's machine. Rule 4's
surviving half is that an inference the operator cannot see still owes an
off-canvas report, and a disclosure that is written, catalogued, unit-tested
for its three clauses and then never painted has discharged nothing.

## ★ The control point, and why it is not optional

`properties.text.face.new` must be **absent** before the combo is clicked.
Without that, a build whose popup was somehow always open would pass this
check on regions that were never opened by the gesture — the defect
`max_zoom` names in its own header, wearing the same green tick.

## ★★ It has been seen to FAIL, which is the acceptance criterion

[`crate::checks`]' founding rule: *"it must fail against a build where the
wiring is absent, and the wiring must be something no unit test in the
workspace can observe."* A check that has only ever been seen to pass is
indistinguishable from one that cannot fail.


| build | verdict |
|---|---|
| page faces only (the planted build, and the shell as it shipped) | **FAIL** — *"the chooser opened and declared no `properties.text.face.addable` region"*, with `properties.text.face` as the only region under that prefix and a screenshot beside it |
| page faces + the standard fourteen | **PASS** — `text-style-applied page=0 change=face applied=19 runs=14`, followed by `format-text` |

★ Note what the planted build's failure text shows: the popup **opened**,
the page's own faces were **listed**, and every unit test in the workspace
was green — including `choices`' own, which asserts the filter and not the
painting. The whole defect was one absent group in one popup, and this is
the only instrument in the project that can see it.

## What this deliberately does not assert

**Which** of the fourteen was applied. The trace's `text-style-applied` line
carries `change=face` and not the selector, and adding the face name to it to
satisfy a check would be the harness dictating a diagnostic's contents. The
row that is clicked is the *first* addable row on a fixture whose page fonts
are known, which is deterministic enough; and `format-text` landing is the
claim that matters, because it is the one that says a `/Font` object was
written into the operator's document.
