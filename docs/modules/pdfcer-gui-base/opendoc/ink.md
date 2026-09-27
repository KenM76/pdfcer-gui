# `pdfcer-gui-base/opendoc/ink`

## Item notes

### `fn ink_at`

Two facts in one value, because they are only ever wanted together and
separating them would let a call site pair the observation with the
wrong ceiling: *has this page been seen compositing in ink*, and *what
ceiling has the operator set*. See [`crate::render::strategy::Ink`].

The ceiling is `Settings::max_cmyk_buffer_bytes` read **through the
document's own settings**, which is where the settings window's Apply
lands — so raising it in the window moves the tier on the next frame,
with no reopen and no second copy of the value anywhere.

It is NOT taken from `SettingsExt::render_options`, which would be the
tempting spelling: that builder produces the options a *render* is run
with, and this question is asked before there is a render to run.

### `fn learn_ink`

`pdfcer_render::page_composites_in_ink` is the **same computation** the
renderer performs, not a second one — the engine's own test asks each
fixture and then renders it and requires the two to agree. A pre-flight
that can disagree with the render is worse than no pre-flight, because a
caller acts on it.

# Why this takes the render options when [`Self::ink_at`] does not

`ink_at` reads `Settings::max_cmyk_buffer_bytes` directly: that is a
**budget**, and its question is asked before there is a render to run.
This function needs the options themselves, because
`page_blend_space_source` is the setting that **changes the answer** —
the same page can composite in ink under one policy and not under
another, and the annotation scope can remove page content from the
question entirely. Passing anything but the options a render would
actually use reintroduces exactly the disagreement the engine's
agreement test forbids.

# It is the SPACE question, not the BUDGET question

`composites_in_ink` is *the page asked for ink*, and it survives a
change of zoom. Whether the colorant buffer is actually engaged is the
second half, `will_composite_in_cmyk(w, h, budget)`, and that lives in
[`crate::render::strategy::for_page`] where the pixel dimensions are.
This function deliberately takes no dimensions.

# It records BOTH halves of the answer

`composites_in_ink` goes to [`Self::ink_pages`] and is what the render
tier reads. `source` goes to [`Self::ink_source`] and is what the
*operator* reads, in the render report - it is the difference between
*this sheet declares a subtractive page group* and *this sheet declares
nothing and the file's output intent decided for it*, and the second is
the case Settings - Colour's `page_blend_space_source` exists to govern.
A shell that stored only the flag would be throwing away the fact that
makes that control legible.

# Cost

One page dictionary read, once per page, for the life of the open
document — [`Self::ink_asked`] is why "no" is remembered as firmly as
"yes". Called from the tier decision, which runs every frame, so the
set is load-bearing rather than an optimisation.
