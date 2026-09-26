# `dialogs::export_image` — a picture of the page, in a format that can
actually hold what is on it

## The gap this closes — `OPERATOR_REQUESTS.md` **O120**

The operator, 2026-09-03, verbatim:

> *"can you add the ability to export page(es) to png, jpg, svg. note that
> there had better be full support (including transparency where
> supported!). Also I'd like to be able to copy and paste anything to other
> software - like copy and paste vector graphics into word or inkscape for
> example if possible."*

He said it to the **engine** side, which shipped all of it the same day and
sent a note — *"informational, no reply needed; consume when convenient"* —
that nothing here was required to read. `RIBBON_IA.md` §5.1 had carried the
row (`Export image… (PNG/JPEG/TIFF, DPI picker)`, marked **C**) since the
ribbon was specified, and `shell::manifest::registers`' `PLANNED` entry said
*"needs a DPI picker and a save dialog; no engine work"*. Both were right and
neither was a gate, so it stayed unbuilt for a day after it had been asked
for out loud.

★ **Two departures from that IA row, stated rather than made quietly:**

* **TIFF is not offered.** `pdfcer-render::export` encodes PNG and JPEG and
  nothing else. The IA row predates the export module; offering a format
  with no encoder behind it would be a control that fails on press.
* **SVG is offered, and the IA row does not mention it.** The row predates
  `pdfcer_render::svg` entirely. It belongs in this window rather than in a
  second one because the question the operator is answering — *what kind of
  picture of this page do I want* — is one question, and the answer that
  makes his own stated example work (*"copy and paste vector graphics into
  word or inkscape"*) is the vector one. A separate `Export SVG…` command
  would put the right answer behind a control he would have to know existed.
* **EMF is offered, and the IA row does not mention that either**, for the
  same reason at one remove: it postdates the row by longer still. It is
  here rather than in a second window because it answers the same single
  question with a *different program* in mind — the operator who has been
  handed an SVG their copy of LibreOffice 24 will not open needs the answer
  in the list they are already looking at, not behind a command they would
  have to know existed. Its hint names the programs; see
  `crate::app::actions::imageexport::ImageFormat::Emf`.

## ⚠ The second half of the operator's sentence is NOT in this window

*"Also I'd like to be able to copy and paste anything to other software"* —
copy-out to the OS clipboard — is deliberately absent, and the reason is
recorded in `crate::clipboard`'s header rather than here. In one line: it
needs `clipboard-win` and `windows` as direct dependencies (both already in
the lockfile, neither in this crate's manifest) and one `unsafe` block that
`#![forbid(unsafe_code)]` will not host, and a copy-out that places only the
raster formats makes Word's paste a flat picture — so half of it is worse
than none of it.

★ **The EMF work here is not a consolation prize for that.** It is the same
bytes the clipboard's second entry will carry, produced by the same call,
disclosed by the same sentences. When the placement half is built it reuses
`app::actions::export::emf_bytes`'s options and `text::export_image::
emf_fidelity`'s wording without re-deriving either.

## ★★★ The sentence the whole window is arranged around

*"full support (including transparency where supported!)"* — **the
parenthesis is the instruction.** It concedes that one of the four cannot
do it and asks pdfcer to be the thing that says which.

So the Background group is not a checkbox with a hint. It is a checkbox that
**goes dead when JPEG is selected, with the reason under it in words**, and
the reason names the format, names what would otherwise happen, and names
the two formats that can. `crate::app::actions::imageexport::ImagePlan::
impossible` then refuses the same combination a second time at the writer,
because a guard that lives only in a window is a guard that a keymap, a
restored plan or a later window can walk past.

## ★★ Everything the window can be wrong about, it says BEFORE the picker

`export_dxf`'s ordering rule — *"the operator is never asked where to put a
file that turns out to be empty"* — generalises here into four live
disclosures, each drawn beside the control that causes it:

| shown | because |
|---|---|
| the pixel size of the largest page | a resolution is an abstraction; a pixel count is the thing that lands on disk |
| a resolution past `MAX_PIXMAP_EDGE` | the engine refuses it, and a refusal after a save dialog is a wasted answer |
| a typed range naming no page | fixable in the box in front of them |
| the multi-file naming pattern | a save dialog cannot say *"the name you type is a stem"* |

## ★ Why the render is not previewed

`dialogs::print::preview` draws the page because a print job **places**
it — margins, scaling, a clip that will happen — and the preview is the only
way to see the placement. An image export places nothing. What comes out is
what the canvas is already showing, at a different number of pixels, so a
preview here would be a second, smaller copy of the canvas: cost without a
claim. What an operator cannot see and is therefore owed is **the pixel
count** and **what could not be expressed exactly**, and both are given in
words.

## Rule 15

This window offers a **resolution** and never a scale. The distinction is
not pedantry here: `dialogs::export_dxf` is an entire window built to
establish a scale from the **ce dimensions** the operator has drawn, because
a DXF at the wrong scale opens cleanly and is wrong. A picture has no scale
to get wrong — it is a picture of a page at a stated size — so nothing here
reads the dimensioning model, and nothing here consults **pdf dimensions**
either.
