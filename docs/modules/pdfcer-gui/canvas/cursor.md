# `canvas::cursor` — pdfcer's own crosshair, because the platform's is invisible


> *"The crosshairs when over the canvas are white making it hard to see
> them."*

## What was actually happening

Nothing in this crate draws that crosshair. [`crate::canvas::tool`] asks for
`egui::CursorIcon::Crosshair`, egui-winit maps it to the platform's stock
crosshair, and on Windows that is `IDC_CROSS` — a **monochrome** cursor
whose colour is decided by the operator's mouse-pointer scheme and by the
accessibility pointer-colour setting, neither of which this application can
read or influence. On a white or inverted scheme it is a white cross, and a
white cross on white paper is not a cursor.

So the fix is not a colour change. It is to **stop asking the platform for a
crosshair and supply one**, which egui 0.35 supports directly:
`Context::set_cursor_image` hands an RGBA bitmap to
`winit::window::CustomCursor`, and it is a real OS cursor — composited by
the window manager, so it does **not** lag the pointer and is **not**
clipped by our window, which are the two failures of drawing a cursor with
`egui::Painter` instead.

## Why two tones rather than the inversion the operator expected

The report guessed at the mechanism — *"I assume they change based on if
they are over a black or white or grey object"* — and that guess describes
how this used to work and no longer does anywhere.

XOR/inverting cursors were a real facility: a monochrome cursor with an AND
mask and an XOR mask, where the XOR bits inverted whatever was underneath.
Windows still *accepts* such cursors and X11 had the same idea. What killed
them is compositing: a desktop compositor draws the cursor into a separate
layer and blends it, and there is no blend mode that means "invert the
contents of the layer beneath me". A `CustomCursor::from_rgba` bitmap is
straight RGBA and has no way to express inversion at all.

What every application that needs a precise cursor does instead is a
**two-tone glyph**: a dark core with a light outline, or the reverse.
Photoshop's *Precise* cursor, Illustrator, GIMP, Inkscape and AutoCAD's
crosshair are all this. It is strictly better than inversion for the case
that motivates both — mid-grey, where an inverted cursor becomes *another
mid-grey* and disappears, while a black-cored white-haloed cross stays
legible.

So: **black core, white halo, on every background.**

## Why these two colours are not theme colours, and the gate agrees

Every other colour in this application comes from `egui_shell::theme` and
`tools/gates/check-theme-colors.sh` enforces it. This one must not, and the
reason is specific rather than an exemption of convenience:

**A theme colour is chosen to contrast with the application's own surfaces.
This cursor has to contrast with the operator's document**, which is
whatever a CAD exporter drew — including, on any given drawing, a region of
exactly the accent colour. A themed cursor would be invisible on the one
page that happened to match it, and there is no palette entry that can be
right about content pdfcer does not control.

Black and white are the only pair with that property, which is why every
reference application converged on them. Nothing here constructs a
`Color32`; the bitmap is bytes, so the gate has nothing to say either way,
and this paragraph is the argument it would want if it did.

## The centre gap is not decoration

The arms stop short of the centre, leaving the target pixel and its
neighbours unobscured. A crosshair whose arms meet hides the very point it
is pointing at, which matters on a dimension pick or a snap — the operator
is aiming at a line one pixel wide. Same reasoning, same solution, as every
application listed above.

## Scaling, and why the bitmap is cached per size

The bitmap is device pixels; the operator's UI scale and display DPI decide
how many of them a cursor should be. So it is generated at
`32 * pixels_per_point` and **cached by that pixel size**, because
egui-winit dedupes the upload to the OS by `Arc::as_ptr` — returning the
same `Arc` across frames means the cursor is converted to a platform handle
**once**, and returning a fresh one every frame would re-upload a bitmap at
sixty hertz.

## The trap: `cursor_image` is STICKY between frames

`egui::PlatformOutput::take` explicitly keeps both `cursor_icon` and
`cursor_image` across frames — *"sticky between frames"*, in its own
comment. And `egui-winit`'s `apply_cursor` prefers the **image** whenever
one is present, so a bitmap set once outlives every later `set_cursor_icon`
from anywhere in the application.

Set it and never clear it and the crosshair follows the pointer onto the
ribbon, into the panels, over the scrollbars, and stays there after the
document is closed. That is why [`crate::app::frame`] clears it once per
frame **before** anything draws, and the canvas re-asserts it if it wants
it: one place resets, one place asks, and a frame in which the canvas does
not run cannot leave a stale cursor behind.
