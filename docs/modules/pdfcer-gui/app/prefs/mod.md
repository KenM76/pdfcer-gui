# `app::prefs` — the shell's own preferences, as distinct from the engine's settings

## Why this is not `pdfcer_core::settings`

That store has a stated purpose and this is not it. Its own window says so
in its first paragraph: *"The PDF standard leaves some things genuinely
undefined … Where that happens, pdfcer asks you rather than deciding
quietly."* Every one of its entries exists because a **standard declines to
have an opinion**, and each one states what clause is silent.

How sharp a page is rasterised is not that. Nothing in ISO 32000-1 is silent
about it; it is a **preference**, a trade of sharpness against time that
depends on the operator's machine and on how big their drawings are. Filing
it beside the CMYK conversion intent would make the settings file's own
framing dishonest — and would put a value with no clause number in a file
whose every entry cites one.

Persisting it belongs with the ribbon layout and the keymap, under the same
`userdata/` roof and in their own file.

## Same roof, same shape, same fail-soft contract

It sits in the directory `pdfcer_core::settings::resolve_store` resolves,
beside `settings.txt` and `layout.ron`, so the update instructions —
*"replace the program files, keep your `userdata` folder"* — cover it
without being reworded.

The format is the engine's: flat `key = value`, `#` comments, and **per-key
recovery**. That last point is the one worth copying rather than a
convenience: an unknown key is left in the file and reported, a bad value
falls back for that key alone, and one bad line never discards the rest.
The file is meant to be hand-editable, and a parser that fails a whole
document over one typo punishes the operator for using it.

## Why there are two render preferences and not seven

`RIBBON_IA.md` §5.2 commissions a View ▸ Render group of five, plus two
behaviour settings. Only two of the seven name something this shell and
this engine can honour:

| commissioned | verdict |
|---|---|
| **Render quality** | ✅ [`RenderQuality`] — a raster-scale multiplier over `viewer::raster_scale`, which is otherwise `zoom × pixels_per_point` exactly |
| **Zoom settle delay** | ✅ [`Prefs::zoom_settle_ms`] — the operator's number in place of `render::settle::ZOOM_SETTLE`'s compiled-in one |
| Render strategy (whole page · tiled progressive) | ❌ there is no tiled-progressive path in this shell. `pdfcer_render::render_page_region` exists, so it is buildable — but it is a rendering **architecture**, not a setting, and a radio offering it would be an affordance for a code path that does not exist |
| Thin lines | ⚠ Not a preference. `RenderOptions::stroke_display` is the engine's knob, and this shell drives it as a **View toggle** — `view.line_weights`, per document, on `crate::viewer::ViewState` — because it is flipped several times while reading one sheet, which is what a ribbon tab is for. Where a persisted default would go, if one is ever asked for, is argued at `crate::text::commands::view_line_weights` |
| Antialiasing | ❌ `RenderOptions` exposes no knob; the rasteriser sets `anti_alias: true` as a literal. (`shading.rs`'s `anti_alias` is the *document's* `/AntiAlias` key — a property of the shading pattern, not a viewer preference, and honouring it is correct.) |
| Floating panels (Off · Allowed) | ❌ nothing to gate: `egui_shell::dock::float` implements tear-out and no code path forbids it, so *Allowed* is the only state that exists |
| App initiative (Never · Ask · Allowed) | ❌ **the setting has nothing to gate.** Nothing in this build opens a surface unasked — which is the specified default, *Never*, already true by construction. A control whose only value is the one already in force is a control that does nothing |

The last two rows are the interesting ones, and they are why this table is
here: an absent preference is not always a gap. A setting that exists only
to switch off a behaviour pdfcer does not have would mean building the
behaviour first.

## …and two more, from the opposite direction

[`opening`]'s two preferences — how the first page is fitted, and which
overlays are already on — are **not** commissioned by `RIBBON_IA.md`. They
come from the `NO_SURFACE.md` sweep, which is the inventory of *every
tunable an operator would plausibly want to change and cannot*, and they are
the two rows in it that cost an operator something on **every document they
ever open** rather than once.

That contrast is worth carrying, because it says where the next preference
will come from. A commissioned list is written from the outside, before the
shell exists, and most of its rows name nothing. The sweep is written from
the inside, by reading the constants the code actually holds, and both of
its candidates are real. **An inventory of what the program does beats a
wishlist of what it might.**

## The two stores are two files, and the operator never finds out

`dialogs::settings::Draft` edits both and the window has one Save and one
Cancel. See its `working_prefs` field, which states the rule: *"one Cancel
discards both, one Save writes both, and `is_dirty` is true if either
moved."*

## Item notes

### `fn format_percent`

`1e12` is what `f32::to_string` produces for a trillion, and a file the
operator opens in a text editor should say `1000000000000`. The file is
his to read and edit; a machine-shaped number there is a small rudeness
with a real cost, because he cannot tell at a glance what he set.
