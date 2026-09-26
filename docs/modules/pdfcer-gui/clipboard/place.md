# `clipboard::place` — **the half that was missing, and the transaction it
# makes**

[`super`] builds the bytes and states the order. This module produces the
payload from a real document and hands the ordered set to
`native_clipboard::place`, which is the crate that owns the `unsafe`.

## The rule that governs every function here

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

The selection route **fell out cleanly** and is therefore taken. Both
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

## Why the render is transparent by default

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

## Item notes

### `const COPY_DPI`

150 DPI, which is `pdfcer copy-page`'s own default in the engine's CLI
(`--dpi`, `default_value_t = 150.0`) — so the shell and the command line
produce the same clipboard for the same page rather than two answers that
happen to look alike.

It is a compromise and worth naming as one. The number affects only the
two **raster** entries and the rasters *embedded inside* the two vector
ones: geometry is resolution-independent and is recorded at whatever scale
the writer chose. So this is the resolution of a scanned page's picture, not
of a CAD sheet's line-work. 300 would double the byte count of every copy —
on a clipboard, which is memory the whole desktop shares — for a difference
visible only when a raster page is enlarged past its natural size.

### `fn render_options`

Through the settings funnel, never `RenderOptions::default()`. That is
the rule `app::actions::export` states and a `syn` check in `app::settings`
enforces from the other side — it fails the build on any call site that
constructs its own — so that one place turns the operator's configuration
into render options, and an exported picture, a printed page and a copied
one cannot disagree about CMYK intent or mask resampling.

The annotation stance and the layer overrides come from the DOCUMENT,
which is what makes this *a copy of what you can see*. An operator who has
hidden a layer is looking at a drawing; the thing that reaches Word is a copy
of that drawing, not of the one underneath it.

**Both routes take these, including the selection one**, and that is a
decision rather than reuse. `canvas::clipimage` renders a clip through the
three-argument `render_page`, which takes no options at all, and argues that
a freshly parsed standalone document has *"no session, no annotations and no
layers — so there is nothing for those parameters to say"*. True of the
annotation and layer fields; **not** true of the colour-management ones. A
clip's paths and images came out of the operator's document and must be
interpreted the way the operator configured, or a copied selection is a
different colour from the copied page it was cut out of.

### `fn page_payload`

# Errors

[`Refusal::NoPage`] for a document with no page at the view's index,
[`Refusal::Render`] when a writer refused.

### `fn selection_payload`

`None` — rather than an error — when nothing on the page is selected, so
the caller falls through to the whole page. That is the behaviour every
program in the class has: `Ctrl+C` with nothing selected copies the page in
a viewer, and refusing would make the command useless in the commonest case.

### `fn raster_into`

Shared by both routes so that the **DPI recorded in the PNG** — and the
pixels-per-metre in the DIB derived from the same number — cannot be right
on one route and wrong on the other.

`Some(COPY_DPI)`, never `None`. The engine's note on the exporter is
unambiguous about what leaving it out costs: *"without `pHYs` Word places a
300 DPI page four times too large."* On a clipboard that is worse than on a
file, because there is no dialog in between where a size could be corrected.

### `struct Staged`

Owned bytes rather than borrowed, because two of the four are **built at
this boundary and exist nowhere else**: the SVG's trailing NUL is added by
[`svg_payload`] and the DIB is assembled by [`dib_v5`]. A borrowing form
would need the caller to hold four temporaries alive in the right order,
which is the kind of arrangement that survives exactly until somebody adds
a fifth format.

### `fn staged`

The pure half of the placement, and everything a test can usefully assert:
which formats, in which order, with which bytes. It crosses no syscall and
touches no clipboard.

# Errors

[`Refusal::WouldDegrade`] when the payload has a raster and no vector — see
the module header. [`Refusal::NoPage`] when there is nothing at all to
place, which is refused rather than treated as a success for the reason
`native_clipboard::PlaceError::Nothing` gives: a caller that cannot tell
*placed nothing* from *placed everything* will report the second.

### `fn place`

# Errors

[`Refusal`] — see [`staged`] for the two it raises itself, and
[`Refusal::Clipboard`] for everything the operating system refuses.

### `fn slot_for`

The mapping from this shell's vocabulary to `native-clipboard`'s, and it is
the ONLY place the two meet. `Slot` says *how a byte block becomes a
handle*; [`ClipFormat`] says *what the bytes are for*. Keeping them separate
types is what lets that crate know nothing about documents — see its header
— and this function is the whole cost of the separation.

**Its own function rather than an inline `match` inside [`place`]**, and
the reason is that a test could not otherwise see it. The first version of
this code inlined the `match`, and the falsification pass then found that
swapping `Png` from `Registered` to `Predefined` **broke nothing**: the test
was asserting `ClipFormat::is_registered` against a hand-written `matches!`
in the test itself, which is a tautology, while the code that actually
chooses the slot went unread. Extracting it is what turns that test into
evidence rather than decoration.

Getting a slot wrong is silent in the worst way. `Registered` on a
predefined format registers a private name nothing reads; `Predefined` on a
registered one places the bytes under a numeric id that means something else
entirely. Both look like a successful copy from inside this program, and the
operator finds out when the paste offers nothing.

### `fn the_entries_are_framed_in_the_measured_order`

The single most important assertion in this module. A pasting
application takes the first format it recognises, so this ordering *is*
what decides whether Word receives an editable graphic or a picture —
and there is no second chance to influence it at paste time.

### `fn a_raster_only_payload_is_refused_rather_than_half_placed`

*Half is worse than none.* This is the assertion that says so: the
engine measured a raster-only paste into Word as a plain picture, and a
plain picture arriving where the operator asked for vectors is
indistinguishable from the feature not existing.

### `fn the_framing_is_applied_where_the_bytes_are_staged`

Both are Chromium's exact byte shapes, which is what Microsoft validated
Office against. Asserted on the framed entry rather than on
`svg_payload` alone, because the mistake this guards against is not
*"the function is wrong"* — [`super::super`] already tests that — it is
*"the placement forgot to call it"*.

### `fn a_missing_format_is_skipped_and_the_rest_keep_their_order`

The EMF is the one most likely to be absent in practice: it is the
youngest writer and the one with the most it cannot express. Losing it
costs LibreOffice 24.x its only vector route and costs nobody else
anything, so the copy goes ahead.

### `fn the_registered_slots_match_the_registered_formats`

The failure this guards against is silent in the worst way: registering
`CF_DIBV5`'s *name* would create a private format nothing reads, and
treating `"PNG"` as predefined would place it under format id 0. Both
look like a successful copy from inside this program.
