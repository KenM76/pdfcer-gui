# `canvas::present` — **drawing the canvas**: the scroll area, the pages in
it, and the geometry the frame hands back

[`show`] and its body [`show_in`], plus the constants and helpers that only
they use. Everything else about the canvas — what a click means, what a drag
does, what is selected — lives in the sibling modules `canvas` indexes.

## Why this is its own file


⇒ It was. R2's own instruction is *"when a file approaches the limit, that is
the signal to find the seam, not to raise the limit"* — and a module index
that cannot accept a new entry without something else being deleted is a
file that has stopped being an index.

## The seam, stated

`canvas/mod.rs` is now **only** a module index and the canvas header: 53
`pub mod` lines, each with the paragraph that says why that module exists.
Adding a 54th costs nothing and takes nothing away.

This file is the one thing that file also happened to contain — a thousand
lines of *drawing*, which is a different subject from *what the canvas is
made of*. They change for different reasons, which is the test this project
applies to every split it makes.

## Nothing moved except its address

The move is textual: the same items, in the same order, with the same
documentation. `show` and [`Sampled`] are re-exported from `canvas`, so
every call site still says `canvas::show(...)` and no caller learned that
this file exists.

## Item notes

### `fn show_in`

Returns the context-menu tokens *and* what the frame learned about where
its pages ended up — see [`CanvasGeometry`] on why that has to travel
outwards rather than be read again.

### `const CANVAS_MARGIN`

Subtracted from the viewport *before* the fit scale is derived rather
than added as a layout margin afterwards, so "fit page" really does fit
with the gap visible instead of fitting exactly and then being clipped
by the gap.

`pub` because [`zoom::zoom_to_rect`] fits a *region* by the identical rule
and must leave the identical gap — a framing command that pressed the
region flush against the panel edges while "Fit page" left 16 points would
read as two different ideas of what fitting means.

### `struct Sampled`

> `tool` is *what* the operator armed, `caps` is *whether* the mode
> permits it, and `pen` is *what it will look like*. All three are
> sampled once per frame and for the same reason — a gesture means what
> it meant when it started.

`tool` is not here because the canvas reads it from `egui::Memory`
itself; these three are the ones the application owns and must hand over.

# Why sampling matters more than tidiness

Every field is `Copy` and every one is read at the top of the frame, so
the canvas sees a **consistent** snapshot for the whole frame. A canvas
that re-read any of them mid-frame could start a drag under one mode and
finish it under another, which is the class of defect
`app::gating::on_mode_capabilities_changed` exists to prevent from the
other end.

### `fn show`

Operator intent leaves by two routes, and the split is not arbitrary:

* **`actions`** carries what the canvas itself decides — a zoom step, a
  fit, a Delete raised by the Delete key. These are already `Action`s
  because the canvas knows what they mean.
* **the return value** carries `egui_shell::HandlerToken`s: the commands
  the operator chose from a context menu. The canvas must *not* translate
  those, because translating them is what `PdfcerApp::dispatch_token`
  does for the ribbon, and a second translation is how the two surfaces
  start disagreeing about what `format.delete` means. Handing the token
  on unchanged is what makes `RIBBON_IA.md` §5.8's *"carries the same
  commands again"* literally true.

`host` is `None` when the application has no validated shell — see
[`MenuHost`] — in which case no menu is attached and a right-click does
nothing, which is the correct behaviour for a build with no menu
document rather than a disabled feature.

Beyond those two, everything this function decides lands in the three
documented bookkeeping fields (see the module docs). The document itself
is never touched.

# The rulers wrap this, and the wrapping is three statements

[`rulers::reserve`] takes a **constant** bite out of `ui` before anything
measures the viewport (rule R128 — see that module's header §3), the whole
of the canvas is then drawn into a child `Ui` covering what is left, and
the gutters are painted afterwards from the geometry [`show_in`] hands
back. Painting them *after* is what puts a guide preview over the page
rather than under it, and what lets the ruler mark the page's own edges —
neither of which is knowable until the scroll area has settled.
