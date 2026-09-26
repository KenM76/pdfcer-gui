# `pdfcer-gui/render/pressure`

**Graphics-memory pressure, made observable** — the blank page that
nothing reports.

# The defect

O219: *"sometimes before this happens the view goes blank and when I zoom
in a little more I get the error."* A blank page with no message is the
signature of a texture upload that failed for want of graphics memory:
`GL_OUT_OF_MEMORY` is raised on a flag, nothing traps, the texture object
stays bound with no storage, and the canvas draws an empty rectangle at
full frame rate. `egui_glow` reads that flag only under
`debug_assertions`, so in the build the operator runs, **nothing in the
render stack looks at it at all**.

[`native_gl::drain`] is the instrument. This module is the bookkeeping that
makes a reading of it mean something.

# The frame boundary this module is built around

`eframe` runs the whole of [`eframe::App::ui`] and *then* calls
`paint_and_update_textures`, which is where `ctx.load_texture`'s queued
delta actually becomes a `glTexImage2D`. So:

* an upload **ordered** during frame *N* is **performed** at the end of
  frame *N*, and
* the error it raised is first readable at the **top of frame *N+1***.

⇒ [`poll`] runs at the top of the frame and is told about the *previous*
frame's uploads. Draining anywhere inside the frame's own work would read
a flag the frame's own uploads have not reached yet, and attribute every
failure to whatever was on screen one frame too early.

# A code carries no provenance, so attribution is the hard part

GL's error flag records *that* something failed, never *what*. [`attribute`]
is therefore deliberately unwilling: it blames the canvas's whole-page
raster only when that raster was the frame's **only** upload. Everything
else is traced with the reason it could not be pinned, which is the
measurement that says whether the rule is too strict to ever fire.

That refusal is only worth anything if the census is COMPLETE. Every
`ctx.load_texture` in this crate records here — the canvas raster, the page
thumbnails, the print preview and the icon sheet — because a route that
uploads without recording does not merely go unseen: it makes a frame that
uploaded two things look like a frame that uploaded one, and the one left
standing gets blamed for the other's failure.

⚠ [`Unattributed::NoUploads`] therefore means *"nothing this crate knows
about uploaded"*, not *"nothing uploaded"*. `egui` grows its own font atlas
and `egui_tiles` its own decorations, and neither passes through here.

# ⚠ A debug run is not evidence

In a debug build `egui_glow`'s own `check_for_gl_error!` runs immediately
after each GL call and **clears** the flag. By the time [`poll`] looks,
there is nothing on it. That is not a defect in either party — it is why
any measurement taken with this module must be taken from a release build.

## Item notes

### `struct Ordered`

`ctx.data` rather than a field on `PdfcerApp`, for the reason every other
cross-cutting per-frame value in this crate travels that way (the theme,
the selected tool, the text draft, the dialog owner): the producers are an
`OpenDoc` method, a panel, a dialog and an icon cache, and threading a
field to all four would put a graphics-memory concern into each of them.

### `fn a_thumbnail_is_never_blamed_though_it_is_a_whole_page_raster`

The two share [`crate::render::raster::texture_from_pixels`] and are
built from the same key type, so a rule keyed on the pixels alone
cannot tell them apart — and would blame a 40 kB thumbnail, at whatever
page happened to scroll into the Pages panel, for a failure raised by
the font atlas. This test is what makes that a build failure.

### `fn a_lone_region_upload_is_reported_but_not_blamed`

Region rasters are a fixed multiple of the viewport at every zoom, so
one failing is evidence about the machine — O221 — and lowering a zoom
ceiling in response would take zoom away for a reason that had nothing
to do with zoom.

### `fn an_unrelated_error_still_produces_a_verdict`

`attribute` keys on `is_clean`, deliberately: a frame that raised an
`INVALID_OPERATION` had something happen, and a trace reporting nothing
would hide it. Acting on it is `out_of_memory`'s job, at the call site.

### `fn a_recorded_upload_comes_back_out_once`

Pins the two halves together: a `record_*` that stashed under one id and
a `take` that read another would silently report `NoUploads` forever,
which is indistinguishable from a healthy session.
