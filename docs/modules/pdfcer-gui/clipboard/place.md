# `clipboard::place` — **the half that was missing, and the transaction it
# makes**

[`super`] builds the bytes and states the order. This module produces the
payload from a real document and hands the ordered set to
`native_clipboard::place`, which is the crate that owns the `unsafe`.

## ★★★ The rule that governs every function here

**The whole transaction lands or nothing is placed.**

[`super::ORDER`]'s documentation carries the measurement: a copy-out that
places only the raster formats degrades a Microsoft Word paste to a flat
picture — silently, with no error and no warning, producing something that
looks correct at 100% and cannot be scaled, recoloured or ungrouped. An
operator would report that as *"pdfcer's copy doesn't paste as vectors"*,
which is indistinguishable from the feature not existing except that it
costs them the time to find out.

⇒ So [`staged`] asks [`super::CopyPayload::degrades_word_to_a_picture`]
**before** producing a single entry, and refuses the whole payload when it
answers `true`. And `native-clipboard` stages every handle before the
clipboard is opened, so a refusal at any point leaves the operator's
clipboard exactly as it was. Two halves of one property, one in each crate,
each where it can be checked.

## The two operands, and why the selection route is worth having

| route | what is copied | how the bytes are produced |
|---|---|---|
| [`selection_payload`] | the selected page objects | `EditSession::copy_objects` → `ObjectClip::to_pdf` → a standalone one-page PDF whose `/MediaBox` is the selection's bounds → the engine's file writers |
| [`page_payload`] | the whole current page | the live edit session's `DocumentView` → the same writers |

★★ The selection route **fell out cleanly** and is therefore taken. Both
ends of it already existed: `canvas::clipimage::publish` has produced a
standalone PDF from an `ObjectClip` since the object clipboard shipped, and
`pdfcer_render::svg::export_svg` / `emf::export_emf` take a plain
`&Document` — the `_view` suffixed forms this shell normally uses are the
*session* variants, and a freshly parsed clip has no session to view. So the
selection route is four lines of plumbing between two things that were
already there, rather than a second implementation of anything.

⚠ It copies **page content only**. `doc.selection.object_indices_on` names
content objects; an annotation selection yields none, so a copy with only a
markup selected falls back to the whole page rather than refusing. That is
the honest answer — a markup's vector form *is* on the page — and it is
stated here because the alternative reading ("selection copy is broken for
comments") is the one a reader would otherwise reach.

## ★ Why the render is transparent by default

`pdfcer copy-page`'s own default: `--background` is `None` unless asked for,
so the SVG carries no backdrop, the EMF is in *"its natural state (nothing
is drawn where nothing was painted)"*, and the raster keeps its alpha. That
is what makes a pasted drawing sit on a Word page's own colour rather than
on a white rectangle the operator then has to crop.

The `CF_DIBV5` entry is the one that needs care about alpha, and
[`super::dib_v5`] handles it — premultiplied, which is the convention
Chromium writes and Mozilla reads. The `"PNG"` entry placed **before** it
carries straight alpha unambiguously, so only readers old enough to need the
DIB are exposed to a convention that is not written down anywhere normative.

## ⚠ No test in this module touches the real clipboard

Same rule as [`super`], for the same reason, and it is the reason [`staged`]
is a separate function from [`place`]: the ordering and the bytes — which is
everything a test could usefully assert — are decided by `staged`, which
crosses no syscall. `place` is `staged` plus one call into
`native-clipboard`, and *that* call is verified by construction and by
review rather than by a unit test. Said plainly rather than dressed up: the
`unsafe` placement has no automated coverage, and a test that gave it some
would silently destroy whatever the operator had copied.
