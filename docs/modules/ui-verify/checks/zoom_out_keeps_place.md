# `ui-verify/checks/zoom_out_keeps_place`

`zooming_back_out_keeps_the_view` — the **descent**, which nothing had ever
driven.

# The report


> *"Zoom out has a small bug where it sometimes seems to reposition the page
> so that it is off screen in the far bottom left corner. This happened when
> I zoomed back from around 2 million% but seems to happen at other
> junctions too."*

**2 million % is the same number as O24f's**, and it was not a number he
picked either time: on the day of both reports
`SUB_PIXEL_CONTENT_EXTENT / page_height` was 16,777,216 / 792 ≈
**2,118,000 %** on a US Letter sheet, which is where the position hands over
between the `f32` scroll offset and the `f64` [`DeepAnchor`]. O24f fixed
that hand-over **going up**. This check exists because nothing in the suite
had ever come back **down** through it.


[`DeepAnchor`]: pdfcer-gui `viewer::deep::DeepAnchor`

# Why the sibling check could not see this

[`super::zoom_keeps_place`] climbs. It climbs all the way to the ceiling,
one notch at a time, with a tolerance tight enough to catch a fraction of a
point — and then the run ends, at 10¹² %, having never once rolled the wheel
the other way. Its own header calls the hand-over *"half of what this check
is for"*, and it guards against a run that never crossed the boundary. Both
statements are true of the **upward** crossing only.

A hand-over is two functions, not one. Seeding the `f64` anchor from the
scroll offset on the way in and reconstructing the scroll offset from the
`f64` anchor on the way out are separate pieces of code, and only the first
one was ever written. **A check that only ever travels in one direction
tests one of them.**

# What is measured

The same quantity as the sibling, with the same instrument (its [`held`] is
`pub(crate)` for exactly this reason — two spellings of *"where is the
view"* would drift, and the one that drifted would be the one whose check
went green): **the page point under the centre of the canvas**, read from
the `f64` `canvas-pos` line, before and after every single wheel notch.

The pointer sits on that centre for every notch, so zoom-to-cursor should
hold that page point still to within rounding, at every zoom, in both
directions. A descent that throws the view away moves it by a large
fraction of the page — the defect is not subtle once it is looked for.

# The shape of the run

## Item notes

### `const MAX_CLIMB`

The threshold is about 132,000 % on a Letter sheet — `2^20` px of content
extent over 792 pt; see the module header for why the older 2,118,000 % no
longer applies — and a page-fit start is about 76 %, so the climb is a
factor of ~1,700. A wheel notch multiplies the zoom by roughly 1.22, so ~38
notches reach it. The cap is generous: **reaching it is now a FAIL**, and the
only build that reaches it is one whose wheel is not zooming — which the
descent guard would report anyway.

⚠ The cap is **not** re-tuned downward just because the threshold fell.
Spending a hundred notches this run would cost a second; arriving exactly at
the boundary and reporting a failure would cost a reader an investigation.

On [`FIXTURE`] the numbers are **measured, not derived**, and the first
two written here were neither.

That sheet is 2383.9 pt on its long side, not Letter's 792 — and the long
side is what the bound is taken against, because the threshold is
`SUB_PIXEL_CONTENT_EXTENT / longest_page_pt`. A paragraph written on
2026-09-13 said *"1683.8 pt tall"*, quoted a threshold of **623** and a
crossing at **33** notches; every one of those is wrong. 1683.8 is the
SHORT side, and using it inflated the threshold by the aspect ratio.

The driven run reports the real figures on its own progress lines:
threshold **~440**, crossed at notch **39**, at a zoom of **46,479 %**,
turning round at 154,316 %. Those are what [`MAX_CLIMB`]'s headroom is
measured against.

⚠ The Letter arithmetic above is kept anyway, because it is where the cap
came from — but it is arithmetic about a different sheet, and a reader who
takes it for this check's notch count will be out by six notches and two
orders of magnitude of zoom.

### `const FIXTURE`

The same A1 fixture `sweep-full.sh` hands the chunked checks, chosen for that
reason: it is the sheet on which this check has actually been observed
crossing the hand-over and coming back down, so every notch count in this
file is quoted against it. See the pinning block in `drive` for why `--pdf`
is ignored rather than preferred.

### `const PAST_THRESHOLD`

Not zero, deliberately. A descent that begins with the zoom balanced
exactly on the boundary could cross back on its first notch, and a check
whose first measurement *is* the hand-over cannot distinguish "the
hand-over is broken" from "the climb ended somewhere unlucky". Starting a
few notches inside gives the deep tier a chance to be measured holding
still before the interesting notch arrives.

### `const DESCENT_MARGIN`

The climb records how many notches it took; the descent is allowed that
many plus this margin, so the run is symmetric by construction rather than
by a hard-coded depth that would stop matching the day the ladder changes.

### `const SETTLE_NOTCHES`

Consecutive, not "the first one". The whole subject of this check is the
frames immediately after the hand-over — a descent that stopped the instant
the tier flipped would stop one notch before the defect had a chance to
show, which is the same mistake as measuring once per stage.
