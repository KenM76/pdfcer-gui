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

# ★★★ Why the sibling check could not see this

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
