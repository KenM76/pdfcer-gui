# icons::svg — the SVG-subset parser and the tiny-skia rasterizer

Turns one asset's text ([`super::assets`]) into geometry, and geometry
into a white-on-transparent coverage mask at a caller-chosen **physical
pixel** size. Nothing here knows what any icon means; it knows how to
read a path `d` attribute and how to stroke it.

A hand-rolled parser is safe to rely on here only because of the rule the
doc comments below keep restating: it **refuses** what it does not
understand rather than guessing at it.

This module is the **element scanner**: it walks tags, reads attributes,
decides how a shape is painted, and rasterizes the result. The **path
`d` grammar** — the lexer, the command state machine and the arc
conversion — is [`path`], split out because it is a different subject that
grows for different reasons: a new element touches only this file, a new
path command touches only that one.

## What the parser supports, and what it refuses

This is deliberately NOT a general SVG implementation. It reads exactly
the shape of file the [`super::assets`] §3 style contract describes:

* **Elements:** `<svg>` (opened/closed, attributes ignored), XML
  comments, `<path>`, `<rect>`, `<circle>`. **Any other element is an
  error** — `<g>`, `<use>`, `<text>`, `<defs>`, gradients, transforms and
  CSS are all out of subset and rejected loudly, never skipped, because
  silently skipping a `<g transform=…>` would draw a correctly-shaped
  glyph in the wrong place.
* **Attributes:** `d`, `x`, `y`, `width`, `height`, `rx`, `cx`, `cy`,
  `r`, `stroke`, `fill`, `stroke-width`, `stroke-linecap`,
  `stroke-linejoin`, `stroke-dasharray`. Unknown attributes are ignored
  (they are cosmetic metadata like `aria-hidden`/`xmlns`).

  `stroke-dasharray` is the exception and is **parsed**, because it is
  geometry: ignoring it does not lose decoration, it draws a **different,
  existing icon** — `new-from-template` becomes `new-document`,
  `unembed-fonts` becomes `embed-fonts`, `redact-selection` becomes
  `redact`, `select-all` becomes a plain rectangle — while passing every
  icon test this crate has. **Before adding an attribute to the ignored
  list, establish that it cannot change what shape is drawn**; "cosmetic"
  is a claim about a list somebody wrote, and the list grows.
* **Paint:** `stroke="currentColor"` strokes; `stroke="none"`/absent does
  not. `fill="currentColor"` fills; `fill="none"`/absent does not. The
  colour VALUE is discarded — see "Theming" below — but its presence or
  absence decides whether the shape is drawn at all, so
  `stroke="currentColor"` and `stroke="none"` are not interchangeable.
  `stroke-linecap` accepts `butt`/`round`/`square`; `stroke-linejoin`
  accepts `miter`/`round`/`bevel`. Any other value is an error rather
  than a silent fallback to the default, because a wrong cap on a
  2.5-unit stroke at 16 px is a visible defect that would otherwise ship
  unnoticed.
* **Path commands and numbers:** the complete SVG path grammar
  (`M m L l H h V v C c S s Q q T t A a Z z`), the SVG number grammar
  with implicit separators, and packed arc flags. All of it lives in
  [`path`], whose header explains the two lexing rules that look like
  details and are not. Anything outside the grammar is
  [`IconError::UnsupportedPathCommand`], never a skip.

Every failure mode returns an [`IconError`] carrying enough context to
find the offending byte; nothing falls back to "draw something".

## Theming: one raster per icon, tinted at draw time

Every asset is `stroke="currentColor"` — a single-colour outline with no
palette. So each icon is rasterized ONCE as a **white-on-transparent
coverage mask**, and the colour is applied at draw time by the caller.
Consequences, all of them deliberate:

* Light theme, dark theme, hovered and disabled all share ONE raster.
  There are no light/dark asset pairs to keep in sync, and structurally
  no way for an icon to end up hardcoded-black on a dark background —
  which is exactly the drift `tools/gates/check-theme-colors.sh` exists
  to catch, removed here by construction rather than by policing.
* The tint is therefore **not** part of the cache key
  (`super::cache::CacheKey`) — that is the entire point of the mask.
  Re-tinting is free; re-rastering is not.

Because the mask is white `(255,255,255,a)` premultiplied to `(a,a,a,a)`,
a multiplicative tint yields `(a·Tr, a·Tg, a·Tb, a)` — a correctly
premultiplied, correctly antialiased tinted glyph, for any tint.
`mask_is_white_so_tinting_is_exact` pins that property, because it is an
*arithmetic* precondition of the theming story rather than a convention
anyone would notice breaking.

## Item notes

### `const BOLD_STROKE_FACTOR`

1.35 was chosen as the smallest factor that is unambiguously visible at
16 pt (2.5 → 3.375 viewBox units, ~1.1 physical px heavier at 100% scale)
without the glyph starting to blob shut at its tightest interior features
(`keyboard.svg`'s 3-unit key gaps, `shape-highlight.svg`'s hatch). It is
a *cue*, not a redesign.

### `struct Shape`

Kept separate per element rather than merged into one path because the
set mixes paint styles within a single icon — `redact.svg` has stroked
outlines *and* one filled bar, and `shape-highlight.svg` deliberately
mixes a 2.5-unit contour with a 1-unit hatch.

### `fn attr`

Matches on `name="` with a preceding delimiter check so that looking up
`x` does not match `rx`, and `stroke` does not match `stroke-width` — the
single most likely silent-wrongness bug in a scanner this simple, and the
reason this is one shared helper rather than an inline `find` at each call
site. `attribute_lookup_is_not_a_substring_match` pins it.

### `type PaintAttrs`

Absent `stroke` means "not stroked" and absent `fill` means "not filled",
matching the root `<svg fill="none">` the style contract mandates.
`stroke-width` defaults to the contract's 2.5 so an asset that omits it
still draws at the set's weight rather than at tiny-skia's 1.0.

### `fn push_round_rect`

tiny-skia's `push_rect` has no corner radius, and the set uses `rx` on
most rects, so the corners are drawn as four 90° cubic arcs. Radius is
clamped to half the shorter side, which is what SVG requires and what
stops a hand-edited `rx="99"` from turning into a self-intersecting mess
instead of a stadium.

### `fn a_dashed_stroke_draws_less_ink_than_a_solid_one`

# Why this test is a pixel count and not a parse assertion

Because "the attribute parsed" is not the question. A build that parses
the file perfectly and ignores the dash draws a **solid** line, which is
a valid icon of a different thing: the glyphs that use a dash silently
become ones the set already has — `new-from-template` →
`new-document`, `unembed-fonts` → `embed-fonts`, `redact-selection` →
`redact`, `select-all` → a plain rectangle.

A test that asserted `shape.dash == Some(vec![4.0, 4.0])` would pass on
a build that parsed the attribute and then dropped it before
`Stroke` — which is one line's slip away and is the whole failure
mode. **The only assertion that cannot be satisfied by a build which
forgets to USE it is one about the pixels.**

Counted rather than compared to a reference image: a count is stable
against antialiasing, against tiny-skia's version, and against the
weight factor, where a golden image is not. The relationship — dashed
covers strictly less ink than solid, and both cover some — is what is
actually being claimed.

### `fn an_odd_dash_array_is_doubled`

`stroke-dasharray="4"` means *4 on, 4 off*, not *4 on, nothing off*.
Getting it wrong draws a plausible dash at the wrong duty cycle — the
confidently-wrong outcome this module refuses everywhere else — so it
is pinned rather than left to the reader of the spec.

### `fn a_malformed_dash_array_is_refused`

The module's rule is *reject loudly, never skip*, and it applies to a
value as much as to an element: silently dropping `stroke-dasharray`
draws a solid line, and a solid line is a different icon.

### `fn mask_is_white_so_tinting_is_exact`

Every non-transparent pixel must be premultiplied WHITE, i.e.
`r == g == b == a`. That property is what makes a multiplicative tint
produce a correctly premultiplied tinted glyph for **any** tint
colour, which is in turn what lets one raster serve every theme
preset and every widget state (module header, "Theming").

### `const VIEWBOX`

All geometry in the assets is in these units; [`IconArt::rasterize`]
scales by `px / VIEWBOX` and lets tiny-skia scale the stroke width with
it, which is why a 2.5-unit stroke stays optically identical at every
output size.

### `enum IconError`

Every variant means "this asset is wrong", never "this input was
untrusted" — the assets are compiled-in constants, so any of these is an
authoring bug that `super::tests::every_icon_parses` is there to catch
before it ships. They carry position/context because the alternative (a
bare "parse failed") turns a two-minute fix into an afternoon.

Hand-written `Display`/`Error` impls rather than `thiserror`: this crate
does not depend on `thiserror`, and adding a dependency to spell four
lines of `match` would be a poor trade against the workspace's
"no dependency pdfcer's lockfile does not already carry" rule.

### `fn parse`

See this module's header for the exact supported subset. This is a
scanner, not an XML parser: it walks the byte stream looking for
`<`, dispatches on the tag name, and reads `name="value"` attribute
pairs out of the tag body. That is sufficient (and safe) because the
only inputs are the crate's own compiled-in constants, which are
mechanically uniform by the style contract — and any input that is
*not* uniform hits [`IconError::UnsupportedElement`] rather than
being interpreted loosely.

# Errors

Returns the [`IconError`] describing the first thing it refused to
guess at. It never returns partial geometry: a half-read asset would
draw a half-glyph, which looks like a rendering fault rather than an
authoring one and is therefore attributed to the wrong subsystem.

### `fn shape_count`

Exists so a test can prove a multi-element asset was fully read
rather than truncated at the first element. The renderer never needs
to count shapes.

### `fn rasterize`

The colour written is always opaque white; only the alpha channel
carries information, and the caller's tint supplies the hue at draw
time. Antialiasing is on — at 16 pt these glyphs are ~2 px strokes
and aliased diagonals would be immediately obvious.

Returns a 1×1 transparent image (never panics, never `None`) if the
pixmap allocation fails for an absurd `px`; a missing icon is a
cosmetic defect, a crashed editor holding unsaved edits is not.

### `fn rasterize_with`

`|_| Some(WHITE)` is [`Self::rasterize`] exactly. The two-colour icon
option draws the same art twice through this — once without the accent
shapes, once with only them — so each layer is still a white mask the
caller tints, or bakes a pair of colours into one image where the
drawing surface takes a single texture. Colours are drawn opaque.
