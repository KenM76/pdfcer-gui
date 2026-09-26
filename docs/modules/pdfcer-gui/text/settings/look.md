# `text::settings::look` — what changing it makes you see in the DOCUMENT

## ★ The split is by BLAST RADIUS, which is the window's own taxonomy

Not by dialog group, and not alphabetically. Every setting in this window
carries a `*_radius` line stating *which way costs what*, and that line is
one of exactly four things:

| module | radius | settings |
|---|---|---|
| [`super::look`] | changes what the DOCUMENT looks like; the file is untouched | CMYK intent, CMYK JPEG polarity, CMYK ceiling, blending space, comment author, mask resampling, minification |
| [`super::extract`] | changes what you GET OUT — copy, search, redaction-by-pattern, new ce dimensions | word gap, unmappable codes, replacement text, parallel tolerance |
| [`super::bytes`] | changes what pdfcer WRITES | separations, missing appearance state, index line endings, trailing newline |
| [`super::shell`] | changes pdfcer's OWN window and touches no document at all | theme, UI scale, render quality, zoom settle, opening fit, wheel paging, paste chords, chrome, page cache, mesh padding, preset, auto-hide, Tab order |

That taxonomy is load-bearing rather than a filing convenience: it is the
distinction the window exists to make legible, and a test in [`super`]
asserts that exactly the byte-changing settings say they change the file —
in both directions, so a preview setting cannot quietly claim a consequence
it does not have.

The first three rows are answers to a silent standard and the fourth is
not, which is why [`super::shell`] is a separate module rather than a
section of this one: an operator reading *"the standard leaves this
undefined"* over a choice about how big pdfcer draws its own buttons is
being told something untrue.

One setting is filed by its radius rather than by its group and it is worth
naming: **CMYK JPEG polarity** is here under *look*, and its radius line
also says *"and the saved file if pdfcer re-compresses the image"*. It is
the only setting whose radius spans two of the four. It sits with the
others in its dialog group, where an operator looks for it.
