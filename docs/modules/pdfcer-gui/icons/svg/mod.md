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
