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

# ★★★ The frame boundary this module is built around

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

# ★★ A code carries no provenance, so attribution is the hard part

GL's error flag records *that* something failed, never *what*. [`attribute`]
is therefore deliberately unwilling: it blames the canvas's whole-page
raster only when that raster was the frame's **only** upload. Everything
else is traced with the reason it could not be pinned, which is the
measurement that says whether the rule is too strict to ever fire.

★ That refusal is only worth anything if the census is COMPLETE. Every
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
