# `prefs::pictures` — how the page view draws an enlarged picture

`PicturePrefs::smooth_enlarged`, file key `smooth_enlarged_pictures`, default
`true`. `magnify()` turns it into the engine's `MagnifyFilter`: `Smooth`
(bilinear between a picture's pixels) when on, `PointSample` (square blocks)
when off.

**Default on** because Acrobat smooths an enlarged scan on screen, and a
zoomed-in scan drawn as blocks was what the operator asked to be rid of
(O322). The engine's own default stays `PointSample` — the print-correct and
parity-measured answer — so the option is carried only by the canvas
`RenderRequest::image_magnify`; print, print preview and every export build
their options through `SettingsExt::render_options` and never see it.

Changing it needs no cache key: `adopt_settings` drops every cached raster
when the Settings window is saved.
