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
