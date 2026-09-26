# `text::status::diagnostics` — **the narrator's whole vocabulary**


## The seam is a consumer, not an alphabet

A catalog area in this crate is keyed by **the surface it serves**, and
every function here is read by exactly one: `app::status::notes`, whose
`findings` table pairs a renderer counter with one of these sentences and
whose `notes_line` joins the results. Nothing else in the shell calls any
of them, and the Render-diagnostics dialog reaches them only *through*
`findings`, deliberately — two tables would agree on the day they were
written and disagree the first time an eleventh counter was added to one.

⇒ So the split cost no call site a character: every name is re-exported
from [`super`], and `t::diagnostics_images_skipped` still resolves exactly
where it always did. That is the difference between a structural fix and a
churn commit, and it is the same argument [`super::selection`] carries.

## What a sentence in here has to do

**Name the consequence on the page, never the mechanism in pdfcer.** The
catalog's own worked example is [`diagnostics_fonts_skipped`], which says
*"text from 2 fonts not drawn"* rather than *"2 unsupported fonts"*: a
count of unsupported fonts is a fact about pdfcer, and missing text is a
fact about the picture in front of the operator. He can act on the second.

**Singular and plural are written out.** *"1 images not drawn"* is the sort
of thing an operator screenshots, and every entry here pays six lines to
avoid it.

**And it must not accuse the document.** These are things pdfcer had to
substitute or leave out; [`diagnostics_tooltip`] says so in as many words,
because the difference between *"pdfcer approximated something"* and
*"your document is damaged"* is the single most valuable thing this
surface can teach.
