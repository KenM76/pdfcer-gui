# icons::paint — the seam `egui-shell` asks the application to fill

`egui-shell` renders a ribbon it is **forbidden to understand**. An icon
set is a licensing decision, a rasterization decision and a look; none of
those are a shell's business, so the shell carries an opaque icon **key**
(`Command::icon`, a `String`) and calls back into the application to draw
it. [`paint_ribbon_icon`] is pdfcer's answer to that callback.

## The shape of the seam, and why it is that shape

```text
egui_shell::ribbon::IconPainter<'a> = dyn FnMut(&egui::Painter, &IconRequest<'_>) + 'a
```

The painter receives an [`egui::Painter`] and **not** a `&mut egui::Ui`.
That is deliberate on the shell's side and it constrains everything here:
the icon is painted into a slot *inside a button that is already being
laid out*, whose rectangle the button's own layout computed. A `&mut Ui`
would let the application allocate layout space inside a widget that has
already decided its size, which corrupts the button's geometry in ways
that surface as text overlapping its own frame. A `Painter` can draw and
cannot allocate, so the seam is safe *by type* rather than by
instruction.

The consequence for this module is that it must be able to rasterize,
upload and draw with nothing but a `Painter` in hand — which is why the
texture cache is a thread-local (see [`super::cache`]) rather than
something threaded through a call chain that has no room for it.

## Supplying a painter is what turns the ribbon into a ribbon

`egui_shell::ribbon::qat`'s `shows_label` draws a QAT control icon-only
only when **all three** hold: the command names an icon, it has a tooltip
to serve as that icon's accessible name, and *the application actually
supplied a painter*. Its doc comment records why the third clause exists:


Until a painter is supplied the whole ribbon falls back to text buttons.
Supplying one is the difference between a toolbar and a ribbon.

## An unknown key draws a VISIBLE MARK, never nothing

This is the decision this module most needs a reader to understand,
because the obvious alternative is wrong in a way that is easy to miss.

"Draw nothing and let the caller fall back to a label" **cannot work**.
The fallback is not downstream of the painter — it is *upstream* of it.
`shows_label` is evaluated from `ctx.icons.is_some()` when the control's
atoms are assembled, before any key is resolved and before the painter is
ever called. By the time this function discovers it does not recognise a
key, the label has already been dropped from the button and a square slot
has already been reserved. Drawing nothing into that slot produces
exactly the row of blank grey boxes that `shows_label`'s third clause was
added to prevent, and it produces it *silently*.

So an unrecognised key is drawn as [`paint_missing_mark`]: a rounded
square with a diagonal slash through it, in the same theme foreground
tint as a real glyph. Three properties make that the right answer:

* **It is visible.** The control has an identity, a hit target and a
  tooltip; it is legibly *wrong* rather than invisibly absent.
* **It cannot be mistaken for a real icon.** Nothing in the set is a
  plain slashed square — the mark reads as "no glyph for this", which is
  the truth, and reads that way at 16 px.
* **It is not a placeholder.** The no-placeholders rule forbids drawing a
  *guess* at what belongs there. This draws a disclosure that nothing
  does. Those are opposites: a placeholder invites the reader to believe
  the interface is finished, and this one states that it is not.

The alternative of guessing at a near-match key (`"fit_page"` →
`Icon::FitPage`) is refused for the same reason
[`super::Icon::from_key`] is an exact lookup: a fuzzy resolver draws the
*wrong* glyph for a typo, and a wrong glyph is undetectable where a
missing one is obvious.

The key is additionally reported through [`crate::diag`] so the
offending string can be read out of a trace, rather than guessed at from
a screenshot of a slashed box.

## The selected cue, and the seam that had to widen to carry it

[`super::IconWeight::Bold`] — the "selected state is never colour alone"
cue that replaces emboldening a text label on a control that has no text
— **is** applied here, for a selected control.

It could not be when this module first landed. `IconRequest` carried
`enabled` but not `selected`: the shell knew the state (it passes
`selected` to `egui::Button::selected`) and did not forward it, so
across this seam the only selected cues were the button frame egui
paints and the tint the shell derives — both of which a theme can make
subtle, and neither of which is the glyph.

That was recorded here as a limitation of the seam rather than of the
pipeline, with a note that closing it was a one-field addition on the
`egui-shell` side and that the consumer was already implemented and
waiting. The field was then added for exactly this consumer.

The general point is worth more than the fix. A reusable shell can
reserve the slot, derive the tint and track the interaction state, but
it **cannot** honour "never colour alone" on the application's behalf,
because the second cue lives in the glyph and only the application can
draw one. Every rule of that shape ends as a field on the request.

## Item notes

### `const ASSET_STROKE_UNITS`

Only [`paint_missing_mark`] needs this — real glyphs carry their own
`stroke-width` — but it is the set's weight, and the missing mark has to
look like it belongs to the same family or it reads as a rendering
artefact rather than as a deliberate report.

### `const FULL_UV`

Named rather than rebuilt at each call site so that "draw the entire
glyph" is stated once; a subtly wrong UV rect would crop a glyph in a way
that looks like bad artwork.

### `fn theme_tint`

Tests must not invent a colour: the whole point of the theming story
is that no colour is chosen outside the theme module, and a test that
reached for a literal would be modelling something the application
never does.

### `fn shapes_from`

Shape count is a coarse instrument, and deliberately so: asserting on
`epaint`'s internal shape *variants* would pin this module to an
implementation detail of a dependency, and the property that actually
matters here is "did anything get drawn at all", because the failure
being guarded against is *nothing* getting drawn.

### `fn painted`

A bare egui frame is not guaranteed to emit zero shapes — plugins and
the root `Ui` may contribute — so every assertion below is a
*difference* against a frame that painted no icon. Asserting on the
raw total would make these tests depend on egui's internals rather
than on this module's behaviour.

### `fn an_unknown_key_draws_a_visible_mark_rather_than_nothing`

The failure this guards against is precise: by the time the painter
is called, `shows_label` has already dropped the control's text label
on the strength of a painter existing. A painter that silently draws
nothing therefore produces a control with no label, no glyph and no
explanation — the exact row of blank boxes the shell's third
`shows_label` clause was added to prevent, reintroduced from the
application's side.

### `fn a_near_miss_key_gets_the_mark_rather_than_the_nearest_glyph`

`fit_page` is one character away from a real key. Resolving it fuzzily
would draw a plausible glyph and hide the typo forever; the mark makes
it visible on the first frame.

### `fn the_painter_satisfies_the_shell_seam`

The wiring is one line in another module, and if the bound did not
hold it would fail there rather than here — in a file this module's
author does not own, during a build somebody else is running. This
coerces `paint_ribbon_icon` to the shell's own
`IconPainter` type alias, which is the same check the ribbon
performs, done here where it is this module's problem.
