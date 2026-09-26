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

## Item notes

### `struct PathLexer`

Byte-oriented because the grammar is pure ASCII; any non-ASCII byte can
only be a stray character and will fail the command/number lex rather
than being mis-sliced.

### `fn skip_separators`

`\r` is in the set alongside `\n`, and that is not decoration: with
`core.autocrlf=true` a fresh clone gets CRLF assets, and a scanner
that treated `\r` as a stray byte would ship blank icons on exactly
the machines that had never seen the repository before.

### `fn take_number`

Written by hand rather than by handing a slice to `f32::from_str`
because the *extent* of the number is the hard part: `1.5.5` is two
numbers, `1-2` is two numbers, and `M6 14h12l4 4` has no separators at
all. Getting the extent wrong is precisely the "silently draws the
wrong glyph" failure this module exists to avoid, so the extent is
computed explicitly and only then handed to `from_str`.

### `fn take_flag`

This is not a number lex, and the difference is load-bearing.
`link.svg` is written `a6 6 0 008 8`, where `008` is large-arc=0,
sweep=0, x=8. A number lexer would swallow `008` as the single value
8 and draw a wildly wrong chain link.

### `fn arc_to_cubics`

Implements the endpoint→centre parameterization of the SVG 1.1
implementation notes F.6.5, then subdivides the swept angle into segments
of at most 90° (F.6.6) because a single cubic cannot approximate a larger
arc acceptably — the 270° arcs in `undo.svg` and `rotate-ccw.svg` become
three cubics each.

Returns `None` for the degenerate cases the spec says to treat as a
straight line: either radius zero, or coincident endpoints.

Parameters mirror the SVG grammar exactly: `from`/`to` are endpoints,
`rx`/`ry` radii, `rot_deg` the x-axis rotation, and the two booleans the
large-arc and sweep flags. Radii are enlarged (never shrunk) when they are
too small to span the endpoints, per F.6.6 step 3 — otherwise `sqrt` of a
negative number would silently produce NaN geometry.

The arithmetic is `f64` throughout even though the geometry is `f32`: the
centre parameterization takes a difference of squared radii, which is
where `f32` loses the digits that matter, and the result is rounded back
to `f32` only at the end.

### `fn parse_path_data`

Implements the full path grammar (module header). Three pieces of state
make the whole thing work and are worth naming explicitly:

* `cur` — the current point. Relative commands are offsets from it, and a
  command that needs it before any `M` is [`IconError::NoCurrentPoint`]
  rather than an implicit origin, because an implicit origin silently
  draws a glyph anchored at the viewBox corner.
* `start` — the current subpath's first point, which `Z` returns to and
  which a command *after* a `Z` continues from (SVG's rule, and the one
  most often got wrong).
* `cubic_reflect` / `quad_reflect` — the previous cubic/quadratic control
  point, mirrored on demand by the smooth forms `S`/`T`. Reset to `None`
  after any non-curve command, per the spec: `S` after an `L` is a plain
  curve, not a reflection of something three commands ago.
