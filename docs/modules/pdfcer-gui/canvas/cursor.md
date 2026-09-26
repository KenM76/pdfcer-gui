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

## Item notes

### `const LOGICAL_SIZE_PTS`

32 is the size of a standard Windows cursor and of the stock crosshair this
replaces, so an operator who has used the application before this change
sees the same-sized pointer with a different treatment rather than a
different pointer.

### `static CACHE`

A `Mutex<Vec<…>>` rather than a map: there are at most a handful of distinct
UI scales in a session and usually exactly one, so a linear scan of a
two-element vector is cheaper than hashing and much easier to read.
Contention is nil — this is touched once per frame from the UI thread.

### `static IBEAM_CACHE`

Two caches rather than one keyed by shape: each holds at most a handful of
entries, and a shared one would need a compound key for no saving. The cost
of getting a compound key wrong is handing back the wrong glyph.


Unbounded, and that is safe rather than lucky: [`Tilt`] quantises to five
degrees and folds into a half turn, so there are at most 36 angles, and in
practice a document has one or two.

### `fn render_ibeam`

Halo first then core, for the reason [`render`] gives — drawing them the
other way round leaves a light glyph with a dark outline, which is thinner
in its dark part than its light one and reads as blurry.

# Drawn by inverse rotation, not by rotating a drawn bitmap

Each destination pixel is mapped **back** into the beam's own upright frame
and tested for membership there. Rotating an already-drawn bitmap forward
would leave unpainted pixels wherever two source pixels landed on the same
destination — a beam full of holes at every angle that is not a multiple of
90° — and closing them would mean resampling, which on a two-tone glyph
whose entire value is a crisp one-pixel core is precisely the wrong tool.

The membership test is the same shape the upright version drew directly: a
bar `width` across and `2 × half_height` along, plus a serif slab `width`
deep at each end. Substituting `tilt = 0` gives back the original glyph
pixel for pixel, which is the check that this generalises rather than
replaces it.

### `fn render`

# The order is load-bearing: halo first, then core

The halo is drawn as a *wider* arm and the core is drawn over the middle of
it. Drawing them the other way round would put the halo's own pixels over
the core and leave a white cross with a black outline — legible, but the
opposite of the convention every reference application uses, and thinner in
its dark part than in its light one, which reads as blurry.

### `fn the_ibeam_core_is_dark_and_its_halo_is_light`

*"The I cursor turns white for text selection so I cant see it on a
white background."* Same defect as the crosshair's, three weeks apart,
same cause: `IDC_IBEAM` is a monochrome stock cursor coloured by the
operator's pointer scheme.

So the assertion is not "it renders" — it is that the **centre pixel is
black**, because a light-cored glyph would satisfy every other test here
and reproduce the bug exactly.

### `fn the_ibeam_is_a_bar_and_not_a_blob`

A square two-tone blob would pass the colour test and be a worse
crosshair. The shape carries the meaning: text flows this way, and the
caret lands between two glyphs.

### `fn only_the_two_cursors_drawn_over_paper_are_replaced`

The negative half matters: every other `CursorIcon` this application
asks for is over CHROME, where the platform's stock cursor is correct
and a custom one would be wrong. Only the two drawn over the operator's
document need replacing.

### `fn the_bitmap_matches_the_size_it_declares`

The length invariant is the one that matters: `CustomCursor::from_rgba`
**rejects** a buffer whose length is not `w * h * 4`, and egui-winit's
response to a rejection is to log a warning and fall back to the
platform cursor — i.e. silently back to the defect this module exists to
fix. A wrong length would therefore look exactly like the module not
being wired up.

### `fn the_hotspot_is_the_centre_and_the_centre_is_clear`

Two properties in one test because they are the same claim from two
sides: the hotspot pixel is the geometric centre, and the centre gap
means the operator can see what they are aiming at. A regression in
either — an off-by-one hotspot, or a gap of zero — is invisible on
screen and shows up as dimensions that are consistently one pixel out.

### `fn an_arm_is_a_dark_core_inside_a_light_halo`

This is the whole feature: a cursor of one tone is exactly the defect
reported. Sampling the arm rather than counting pixels, because what
matters is the *arrangement* — a bitmap that happened to contain both
colours somewhere would satisfy a count and could still be illegible.

### `fn the_same_scale_returns_the_same_allocation`

`egui-winit` dedupes its upload to the OS by `Arc::as_ptr`. A fresh
`Arc` per frame would convert a bitmap to a platform cursor handle sixty
times a second — and it would still *work*, which is why this is worth a
test: the symptom is a performance cost nobody would attribute to the
cursor.

### `fn the_tilt_quantises_and_folds`

Each row is a case that would produce a visible defect on its own: an
unfolded angle doubles the cache and uploads a duplicate cursor to the
OS; a 180° that survived rounding would be a distinct entry drawing
identical pixels; and a non-finite angle — reachable from a degenerate
page transform — must not panic in the middle of a pointer move.

### `fn a_quarter_turned_ibeam_is_a_horizontal_bar`

The upright test above this one asserts `tall > wide * 2`; this asserts
the exact reverse on the same glyph at 90°. Asserting both is what makes
the pair evidence: a renderer that ignored the tilt would pass the first
and fail this, and one that rotated everything unconditionally would do
the opposite.

### `fn the_core_stays_dark_at_every_angle`

The operator's *other* cursor report — *"the I cursor turns white for
text selection so I cant see it on a white background"* — is a property
of the glyph, not of its orientation, and a rotation implemented by
resampling would soften exactly this pixel into grey. Checked at four
angles including one that is not a multiple of 90°, because that is
where a resampling implementation would first show.

### `fn the_ibeam_cache_is_keyed_by_angle_as_well_as_size`

This is the test for the failure the cache's own header names: keyed by
size alone, the first angle asked for would be stored and every later
angle would silently receive it — so the cursor would appear to reorient
**once** and then never again. That reads as the feature half-working
rather than as a cache bug, which is exactly the kind of defect that
survives a manual look.

The positive half is the same claim `the_same_scale_returns_the_same_
allocation` makes for the crosshair: `egui-winit` dedupes its upload to
the OS by `Arc::as_ptr`, so a fresh `Arc` per frame would convert a
bitmap to a platform cursor handle sixty times a second.

### `fn a_turned_bitmap_still_matches_the_size_it_declares`

The length invariant again, at an angle: `CustomCursor::from_rgba`
rejects a buffer whose length is not `w * h * 4` and egui-winit's
response to a rejection is to fall back to the platform cursor — i.e.
silently back to the white-on-white defect this module exists to fix. A
rotation that resized the buffer would look exactly like the tilt not
being wired up.

### `fn a_nonsense_scale_is_clamped_rather_than_allocated`

Not defensive programming for its own sake: `pixels_per_point` is
derived from a preference the operator can edit, and this crate has
already shipped one preference that reached a layout pass unvalidated.

### `fn ibeam_ascii`

This exists because of the constraint `apply`'s docs set out: **a cursor
cannot be verified by screenshot.** Windows composites the pointer
separately from window contents, so `BitBlt` and `PrintWindow` — the two
ways `ui-verify` captures a window — return an image with no cursor in
it at any price. R1's usual answer, *drive it and look at the picture*,
has no picture to look at here.

The unit tests assert the properties that can be stated as numbers — the
core is dark, a 90° beam is wider than tall, the cache tells angles
apart. What they cannot assert is whether the glyph *looks like an
I-beam* at 30°, and this is how that is checked: by eye, deliberately,
on demand.

`cargo test -p pdfcer-gui --lib canvas::cursor::preview::ibeam_ascii -- \
 --ignored --nocapture`
