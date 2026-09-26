# `ui-verify/checks/qat_icons`

`qat_controls_are_icon_only` — the regression test for the icon painter
that existed, was tested, and was never handed to the ribbon.

# The defect


And the QAT rendered **text buttons**, because nobody wrote
`.with_icon_painter(&mut icons)` at the one call site that matters.

The trace said so plainly, and in exactly the terms this check asserts on:

```text
ui-rect name=ribbon.qat.file.open rect=[[8.0 5.0] - [81.1 23.0]]     <- 73 pt: a label
ui-rect name=ribbon.qat.file.open rect=[[8.0 5.0] - [32.0 23.0]]     <- 24 pt: a glyph
```

# Why no unit test could have caught it, which is why this file exists

`egui_shell::ribbon::qat`'s `shows_label` decides icon-only from **three**
conditions: the command names an icon, it has a tooltip to serve as that
icon's accessible name, and *the application supplied a painter*. The third
clause was itself added after an earlier build produced a row of blank grey
boxes.

Both outcomes are legal, and **both render correctly**. A ribbon of text
buttons is not an error state — it is what a consumer with no icon set is
supposed to get. The difference is not a condition a type can forbid or an
assertion can trip: it is *whether a caller remembered to pass an
argument*.

So the crate's own tests were all true and all silent. `icons`' 52 tests
proved the painter draws. `egui-shell`'s proved the seam accepts it. The
one fact nobody could state in either crate is that `pdfcer-gui` hands the
former to the latter — because that is a property of a call site, and the
only place a call site's *effect* is observable is a running window.

This is the project's founding rule with a fourth instance behind it, and
the first three are recorded in `DEFECTS.md` and `README.md`: verification
means driving the binary, because a green suite is evidence about the code
that was written and not about the code that was not.

# What this asserts, and why it is a shape rather than a pixel

**Every `ribbon.qat.*` region is approximately square.**

An icon-only control is its glyph plus symmetric padding, so its width and
height are within a small factor of each other. A control that fell back to
a label is its glyph *or* its text plus padding, and the text is a word —
`Save a copy…` measured 107 pt against an 18 pt height, a ratio of nearly
six.

Deliberately **not** an assertion on an exact width. A width is a function
of the theme's icon metric, the padding, and `pixels_per_point`, none of
which this check owns; pinning one would produce a check that fails when a
designer changes a spacing constant, which is how a check stops being run.
The ratio is invariant under all three.

Deliberately **not** a pixel assertion either. Reading the glyph's pixels
would test that the *icon set* draws, which `icons`' own tests already do
properly and offline. What was missing was never "does a glyph render" — it
was "did anything ask for one", and the reserved rectangle answers that
without a screenshot, without raising a window, and therefore without
taking the operator's focus.

# Why the QAT and not the whole ribbon

Band controls legitimately show a label *beside* an icon — that is what a
ribbon button is — so a wide band control says nothing. The QAT is the one
surface in the shell whose controls are icon-**only** by design, which
makes it the only place the missing painter is expressible as a shape.

One consequence worth stating: this check passes on a build with **no icon
keys at all**, because `shows_label`'s first condition then fails for a
different reason and the control is honestly a label. That is correct. This
check's subject is the painter, and a build with no icons has nothing to
paint. `crate::checks::ribbon_captions` covers the ribbon's legibility
either way.
