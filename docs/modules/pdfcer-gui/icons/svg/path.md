# icons::svg::path — the SVG path-data grammar

One `d` attribute in, a stroked/fillable `tiny_skia::Path` out. Split out
of [`super`] because the two halves are genuinely different subjects: the
parent is an *element scanner* that walks tags and attributes, and this is
a *grammar* with its own lexer, its own state machine and its own
numerical geometry. They are also the two halves that grow independently
— a new element type touches only the parent, a new path command touches
only this file — and keeping them in one 1,400-line module was already
within a hundred lines of the project's 1,500-line limit.

## The grammar implemented here

The complete SVG path grammar: `M m L l H h V v C c S s Q q T t A a Z z`.
Implementing all of it rather than only the commands today's assets happen
to use costs a few dozen lines and removes a whole class of future failure
— an icon redrawn with a smooth-quadratic `T` two years from now must not
become a build break. Anything that is not one of those letters is
[`IconError::UnsupportedPathCommand`], never a skip.

## Two lexing rules that look like details and are not

* **Number extent is computed, not delegated.** `1.5.5` is two numbers,
  `1-2` is two numbers, and `M6 14h12l4 4` has no separators at all.
  Handing a slice to `f32::from_str` requires already knowing where the
  number ends, which is the hard part — and getting it wrong is exactly
  the "silently draws the wrong glyph" failure this whole module exists to
  avoid. See [`PathLexer::take_number`].
* **An arc flag is ONE character, never a number.** `link.svg` is written
  `a6 6 0 008 8`, where `008` is large-arc=0, sweep=0, x=8. A number lexer
  would swallow `008` as the single value 8 and draw a wildly wrong chain
  link. See [`PathLexer::take_flag`], and `parses_packed_arc_flags`.

## Arcs

`tiny-skia` has no arc primitive, so elliptical-arc segments are converted
to cubic Béziers by [`arc_to_cubics`], following the endpoint→centre
parameterization in the SVG 1.1 implementation notes (F.6.5) and the
≤90°-per-segment subdivision of F.6.6.
