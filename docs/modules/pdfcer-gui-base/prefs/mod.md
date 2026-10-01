# `prefs` — the shell's own preferences, as distinct from the engine's settings

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
| **Zoom settle delay** | ✅ [`Prefs::zoom_settle_ms`] — the operator's number in place of `app::settle::ZOOM_SETTLE`'s compiled-in one |
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

### `mod cache`

Its header carries the part a reader would otherwise carry out wrongly: the
cache prunes itself to the VISIBLE SET on every frame, so raising the budget
on its own changes nothing at all.

### `const DEFAULT_MAX_ZOOM_PERCENT`

**The maximum, on the operator's instruction of 2026-08-22** — *"Also
set the default to be able to hit the maximum zoom."*

It was 800 % for one build, chosen so a fresh install behaved exactly as
the shell had before the setting existed. That was the cautious call and he
overruled it, consistently with his earlier one: *"it is up to the user to
determine how much of a performance hit they want to take."* A capability
he has to find a preferences file to switch on is a capability most of its
users never have.

What this does NOT change is the behaviour he cares about. The ceiling is
permission, not policy: `viewer::zoom_ceiling` still lets the whole-page
raster bind wherever it can, so **panning stays instant at every zoom that
could render whole-page before** — the region path engages only above it,
where the alternative is not a slower zoom but no zoom at all.

### `const MAX_MAX_ZOOM_PERCENT`

⚠ **It is past the range the page has been confirmed to draw in, and that
is `DEFECTS.md` D27.** Read the units carefully, because this constant is a
*percentage* and the measurement below is a *factor*: a trillion percent is
a zoom factor of 1×10^10. Driving the real binary found the page **drawn**
at 8.6×10^9× (859 billion percent) and **not drawn** at 1×10^10× — so this
value is the first rung measured to fail, not a margin inside the working
range.

What fails is not the raster: at a trillion percent the strip renders
cleanly with no failed tiles and simply shows no page. The limit is no
longer the scroll offset — tier 3's `f64` anchor fixed that — it is the
**strip's own extent**, still computed as `page × zoom` in `f32`, which
reaches 6×10^12 points at that zoom on US Letter.

Two ways out, and they are a choice for the operator rather than a cleanup:
lower this to the confirmed range, or stop building the strip in
`page × zoom` space at deep zoom — the move tier 3 made for the offset, one
layer out. Until one of them happens the shell offers a rung that accepts a
number and then misbehaves, which is the defect this feature has otherwise
refused throughout.

None of this is a judgement about what is sensible. The operator was
explicit that the performance trade is his to make (*"it is up to the user
to determine how much of a performance hit they want to take"*); the
question here is only what the shell can put on the screen.

### `fn auto_hide`

One conversion, named, rather than an `if … { OnHover } else { Off }` at
each of the two call sites. The two settings are stored as `bool` because
`settings.txt` is a `key = true | false` file an operator edits by hand and
a third spelling of the same fact would be a third thing to get wrong; the
shell's [`egui_shell::peek::AutoHide`] is an enum because it has room to
grow a third position (Office has three). This function is the seam where
that difference is absorbed, and it is the place a third position would be
mapped.

### `struct Prefs`

## `PartialEq` but not `Eq` — and it was `Eq` until [`Self::ui_scale`] landed

A scale is a continuous quantity and `f32` has no total equality, so the
derive cannot be kept. Nothing is lost: the only thing that compares two
`Prefs` is `dialogs::settings::Draft::is_dirty`, which asks *"has the
operator changed anything?"* — and `PartialEq` answers that exactly. `Eq`
would additionally promise reflexivity, which the one field that could
break it (a `NaN` scale) cannot reach, because [`chrome::normalise_ui_scale`]
clamps every value that enters the struct.

### `enum PrefNote`

The same shape as `pdfcer_core::settings::SettingNote` and for the same
reason: the file is hand-editable, so a mistake in it must be findable, and
**at its line number**. A message saying only "something was wrong" sends
the operator to read the whole file.

### `fn path`

Derived from the same `pdfcer_core::settings::resolve_store()` the
settings and the layout use, so the three cannot drift apart — which is
the failure this project already found once, when two callers in one
process disagreed about which home was live and put two files that
belong together in two places.

### `fn load`

A missing file, an unreadable one, a broken line or a value out of range
all yield usable preferences with a reason in the returned notes. **A
missing file produces no note**, deliberately: a first run is the
expected state, not a fault, and reporting it would train the operator
to ignore the channel that carries the real problems.

### `fn save`

Unlike loading, saving fails **loudly**: the operator asked for
something to be remembered and is owed the truth if it was not. Same
asymmetry the engine's store holds itself to.

# Errors

The path could not be resolved, its directory could not be created, or
the write was refused. Carried as a `String` because the caller's only
use for it is a trace line — the operator-facing half is a fixed
sentence, for the reason `text::status::settings_not_saved` documents.

### `fn seed_view`

Called once per document, from `PdfcerApp::adopt`, and from nowhere else.

# Why this is a method here rather than a field read in `ViewState::default`

Because `ViewState::default()` cannot see the application. `OpenDoc::assemble`
builds a document without a `PdfcerApp` in reach — its own comment says
so — which is the same constraint that put `adopt_settings` in the open
path rather than in the constructor. Seeding here keeps `ViewState`'s
`Default` the **conservative** answer, which is what every test that
builds one without a configuration relies on.

# The remembered-guides override still wins, and that is not a
coincidence of ordering

`OpenDoc::assemble` may already have set `view.guides = true`, because
`canvas::guides::opening` turns the layer on for a document that has
guides saved against it — *"the presence of the work is the
preference"*. This function therefore **ORs** rather than assigns for
that one field:

| remembered guides | preference | result |
|---|---|---|
| yes | on | shown |
| yes | off | **shown** — the work outranks the default |
| no | on | shown, and empty until the first is placed |
| no | off | hidden |

Row two is the one that matters and it is the reason this is not three
plain assignments. A preference is a statement about documents in
general; a document that carries guides is a statement about *that*
document, and the specific beats the general. Assigning would hide work
the operator did, on the document they did it on, because of a switch
they set weeks earlier about something else.

Rulers and grid have no per-document memory at all, so they assign.

# What it deliberately does not touch

[`crate::viewer::ViewState::display`] — the single/continuous/facing
arrangement. That has its own per-document store and its own operator
requirement; see [`opening`]'s header for why a global default for it
would be a second axis colliding with the one that was asked for.
