# `render::raster` — the bridge from `pdfcer-render`'s pixmaps to egui textures

One job, kept in one place: take a [`tiny_skia::Pixmap`] out of
[`pdfcer_render::render_page`] and hand egui a
[`egui::TextureHandle`] it can draw, plus the [`Diagnostics`] that
came with it. Everything about GPU-texture lifetimes and pixel
formats is confined here so the canvas deals only in "do I have a
current texture for this page at this zoom."

## Premultiplied alpha — the one detail that silently corrupts output

`tiny-skia` stores pixels **premultiplied** (`Pixmap::data()` is
`[R·A, G·B… , B·A, A]`), and `epaint::ColorImage` offers constructors
for both conventions. Passing premultiplied bytes to
`from_rgba_unmultiplied` does not fail, error, or look obviously
wrong — it silently darkens every partially transparent pixel, which
on a page means every antialiased glyph edge. Text would render
slightly heavier than it should and nothing would ever say so. Hence
[`pixmap_to_color_image`] uses `from_rgba_premultiplied` and this
paragraph exists so nobody "cleans up" the choice later.

Page rasters are opaque anyway (`pdfcer-render` fills the pixmap white
before interpreting — PDF has no page background, paper is white), so
the practical blast radius is limited to antialiased edges. That is
precisely the kind of bug that survives review and shows up as "the
text looks a bit off compared to Acrobat."

## Texture filtering

Textures are uploaded with [`egui::TextureOptions::LINEAR`], and the
canvas draws them at whatever size the *current* zoom implies rather
than at their native pixel size. That combination is what makes the
debounced zoom work: between the operator spinning the wheel and the
re-render committing, the stale texture is smoothly scaled instead of
blocky or absent. Nearest-neighbour filtering would make the interim
state look broken rather than merely soft.

## This module is the threading seam

Rendering is off the UI thread — a dense CAD sheet takes on the order of
ten seconds at 1× and a minute at 2×, which drops frames if it runs inline.
[`crate::render::worker`] owns the channel, the cancellation and the
generation counter; nothing outside this module knows how a texture is
made, so the whole of that machinery reaches the rest of the crate through
[`texture_from_pixels`] and nothing else.
