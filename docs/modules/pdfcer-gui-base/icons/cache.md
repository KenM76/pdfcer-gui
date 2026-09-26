# icons::cache — one raster per (icon, physical size, weight), memoized

The ribbon re-runs every frame (60/s), and rasterizing thirty icons per
frame would be absurd. [`IconCache`] memoizes the uploaded
[`egui::TextureHandle`] per [`CacheKey`], so nothing is rasterized twice
unless the display scale changes or a control becomes selected.

## Why the tint is not in the key

Because it cannot be: the raster is a white coverage mask (see
[`super::svg`], "Theming") and the colour arrives at draw time. Putting
the tint in the key would multiply the cache by the number of theme
presets times the number of widget states for *exactly zero* benefit,
and — worse — would mean a hover produced a texture upload. The cache is
keyed on the three things that genuinely change the pixels: which icon,
how many physical pixels a side, and how heavy the stroke is.

The one exception is a coloured glyph ([`super::accent`]): two colours do
not fit in one mask, so both are baked in and keyed, and a hover on a
coloured icon does upload a texture — once per tint, then cached.

`super::tests::cache_serves_repeat_requests_without_re_rasterizing`
asserts the hit path, and
`super::tests::cache_re_rasterizes_for_a_different_size_or_weight`
asserts the miss path. Both matter: a cache that never misses is a cache
that shows a stale, wrongly-sized glyph after the window moves to a
150% monitor, which is the exact blur this pipeline exists to prevent.

## Why it is a thread-local rather than application state

The seam this module ultimately serves is `egui_shell`'s
`IconPainter` — a `dyn FnMut(&egui::Painter, &IconRequest)`. The painter
is handed a `Painter` **precisely so that it cannot allocate layout**,
and it is invoked from deep inside a button that is already being laid
out. There is nowhere in that call chain to thread a `&mut` cache from,
short of making every application that supplies a painter own one and
close over it — which would push a pure-memoization detail into the
shell's public contract.

An interior-mutable per-thread cache sidesteps it with no behavioural
cost, because the cache is pure memoization: evicting it changes
performance, never pixels. eframe runs the UI on a single thread, so
per-thread is per-app in practice, and a second thread would simply get
its own (unused) cache rather than a data race.

[`IconCache`] itself is public and independently constructible so the
caching contract is unit-testable without going through the
thread-local.

## Item notes

### `const CACHE_CAPACITY`

The cache grows only along three axes — one entry per icon, per weight,
per distinct physical size the display scale has taken this session —
so in normal use it settles well under this and never reaches it. The cap
exists solely so that a session that repeatedly changes display scale
(dragging a window between a 100% and a 150% monitor) cannot accumulate
stale textures without bound.

# The arithmetic this number has to satisfy

One entry per icon per weight per distinct physical size, plus, with
coloured icons on, one per accented icon per distinct control tint (a
ribbon has about four: rest, hover, pressed, selected ink). `Icon::ALL`
holds **143** icons and [`IconWeight`] has two variants, so one display
scale is 286 plain entries; the 91 accented icons at four tints add up to
364 more, 650 in all. **Re-derive this number whenever the icon set
grows**; it is that sum times the number of display scales a session
should hold without churning.

Exceeding it is not a crash and not a wrong pixel, which is why it is worth
writing down: the cache clears wholesale and re-rasterizes the entire
visible ribbon, repeatedly, on any machine whose window is dragged between
two monitors of different scale. A hitch, blamed on the renderer, caused by
a constant nobody re-derived when the set changed size.

Clearing wholesale rather than evicting least-recently-used is
deliberate: it is one line, it happens approximately never, and the
recovery cost is one frame of re-rasterization.

### `fn cache_serves_repeat_requests_without_re_rasterizing`

Tint is not part of the key by design (mask + tint), so re-asking
while the theme, the hover state or the enabled state has changed is
still a cache hit — which is why this loop does not vary anything.
There is nothing to vary: none of it reaches this layer.

### `fn the_capacity_guard_clears_rather_than_growing_without_bound`

Reached by asking for more distinct *sizes* than the cap, which is
the only axis an operator can actually drive without bound (drag a
window between monitors of different scale, repeatedly).

### `struct CacheKey`

The tint is absent for a plain glyph — see this module's header. A
coloured glyph has two colours and a mask can carry only one, so its
colours are baked into the pixels and are part of the key.

### `fn texture`

# What happens when the asset is broken

It degrades to a 1×1 transparent texture and a one-line stderr
complaint rather than panicking. The assets are compiled-in
constants, so a failure here means the *build* shipped a broken one —
a condition `super::tests::every_icon_parses` is designed to catch
first — and taking down an editor holding the operator's unsaved
edits over a missing 16 px glyph would be a far worse outcome than a
blank slot with an intact tooltip and accessible name.

Note the asymmetry with an **unknown key**, which is a different
failure with a different answer: see [`super::paint_ribbon_icon`].
A broken asset is a build defect that the test gate catches before an
operator ever sees it, and the stderr line is addressed to the
developer who broke it. An unknown key can reach a real operator
(a command naming an icon the set does not have), so it is drawn
visibly instead of silently.

### `fn with_cache`

Kept private so no caller can hold the `RefMut` across a re-entrant call
(which would panic); every entry point in [`super`] borrows, does one
lookup, and releases before it draws anything.
