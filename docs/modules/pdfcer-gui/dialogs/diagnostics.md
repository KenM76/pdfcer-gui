# `dialogs::diagnostics` — the render report, off the status bar at last

The dispatch target for `tools.render_diagnostics`, on **Tools ▸
Diagnostics**.

## Why it is a dialog rather than a second status line

The argument for the command, recorded in `shell::manifest::tools`' header,
is about **placement** rather than capability: the renderer produces this
report on every raster and the status bar already shows a one-line summary
of it. The bar is the surface for controls an operator touches constantly,
and a diagnostic readout is neither a control nor constant — it is a thing
you go and look at when something is wrong, and it needs room to be more
than one line.

Three requirements fall straight out of that, and this file is them:

1. **A thing you go and look at.** Opened deliberately, holding one
   question's worth of answers, forgotten when closed — which is
   [`super::DialogsState`]'s own definition of a dialog as against a panel.
   A panel would keep state across documents and sit in the dock competing
   for width with the Objects list; nobody wants a render census permanently
   mounted.
2. **When something is wrong.** So it opens on demand and never on its own.
   `MODES_AND_PANELS.md` is explicit that application initiative is
   **never** — pdfcer opens no surface over the canvas unasked — and a
   diagnostic window that appeared because a page happened to substitute a
   glyph would be the clearest possible violation of it.
3. **Room to be more than one line.** The bar gets one elided line under
   **R128**; this gets the findings one per row, plus the three
   measurements of the render itself, plus the two counters the bar
   deliberately excludes.

## The status bar keeps its line, and that is not a duplicate

Both surfaces read the **same** derivation —
[`crate::app::status::notes::findings`] — so they cannot disagree about what
a raster compromised on. What differs is only the room: the bar answers *is
anything worth looking at?* at a glance, this answers *what, exactly, and
how expensive was it?*. `DEFECTS.md`'s "Not defects" table settles the
prominence question: the report is worth having and the status bar is the
wrong place to press it on an operator, so it is demoted rather than
deleted — which is exactly what makes a deliberate route to the full report
worth building.

## What it shows that the bar cannot

| | bar | here |
|---|---|---|
| the findings | one elided line, joined by `·` | one row each, in the same order |
| how long the render took | — | `RenderedPixels::elapsed`, measured around the rasterization alone |
| the raster scale and pixel size | — | from `RenderKey` and the uploaded texture |
| `tolerated` / `compat_skipped` | excluded on editorial grounds | shown, with a sentence saying they are not faults |

The duration and the scale are drawn **together**, deliberately. Render cost
on a dense CAD page is very largely **resolution-independent** —
`BENCHMARK.md` measures the floor as content-stream interpretation, nearly
all of the time at fit-page zoom and still the majority of it at 2× — so a
small raster is not a cheap raster. A duration shown on its own invites the
operator to zoom out and expect relief that will not come.

## Document-scoped, and it closes with the document

It describes *this page of this file*. A window left up over a closed
document would be reporting measurements of a raster that no longer exists,
which is the same reason print is document-scoped and About is not.

## Why it pushes no `Action`

[`super::DialogsState`]'s rule: the funnel exists for changes to **document**
state. This one reads a texture that has already been uploaded and renders
nothing new. It has nothing to undo, nothing to order against and nothing
that could alias.

## Item notes

### `fn footer`

Its own function because both bodies above need it and the early return
for "nothing drawn" must not be a window with no way out but the title
bar's cross.

### `struct DiagnosticsDialog`

It holds **no configuration**, exactly as [`super::about::AboutDialog`]
does, and for the same reason: everything it shows is read from the open
document's current texture on the frame it is drawn, so there is nothing for
the operator to change and nothing for closing it to forget.

Reading live rather than snapshotting on open is a decision. A snapshot
would freeze the report of whichever raster happened to be current when the
command was pressed, and the operator's very next act while diagnosing is to
change the zoom or the page — at which point a frozen window would be
describing a picture that is no longer on the canvas while looking exactly
like one that is. The title says *the picture currently on the canvas*, and
this is what makes that true.
