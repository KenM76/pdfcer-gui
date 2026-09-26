# `pdfcer-gui-base/hairline`

## Item notes

### `const INK`

Generous on purpose. The question is *how much of the page is covered by
stroke*, and a threshold tuned tight to full black would count only the
cores of the strokes and would move with the renderer's antialiasing. 128 is
"more than half way to black", which every pixel inside a 4 px stroke
satisfies and no page background does.

### `fn hairlined_at`

A sibling of [`ink_at`] rather than a second return value from it,
deliberately: that function is about pixels and this one is about the
engine's own count, and a caller reading `(ink, total, thinned)` would have
to remember which two are pixels. Both build their options the same way —
`RenderOptions::default()` plus one assignment — so neither is measuring a
third code path.

### `fn line_weights_off_puts_less_ink_on_a_real_drawing`

# What a failure means, in each direction

* **Equal** — `stroke_display` reached the renderer and changed nothing.
  Either the engine dropped the field, or `SCALE` is low enough that the
  §8.4.3.2 floor had already put every stroke at one pixel. The second is
  excluded by [`the_two_modes_are_identical_where_there_is_nothing_to_cap`],
  which pins that boundary from the other side.
* **MORE ink** — the wrong convention shipped. That is Acrobat's *enhance
  thin lines*, which thickens sub-pixel strokes, and it is the opposite of
  what he asked for. This is the failure worth having a test for: it looks
  like a working feature from every other angle.

The threshold is a **ratio**, not a pixel count, so it survives a change
of `SCALE` or of the fixture's page size. It asks only that the drawing lose
a fifth of its ink, where the arithmetic predicts about three quarters (a
4 px stroke becoming a 1 px stroke) — deliberately far below the expected
effect, because what is being pinned is the *direction and reality* of the
change, not a rendering constant that would make this test a tripwire on
every antialiasing tweak the engine ever makes.

### `fn the_two_modes_are_identical_where_there_is_nothing_to_cap`

At scale 1.0 the fixture's default-width strokes are already one device
pixel, held there by the engine's pre-existing §8.4.3.2 floor. `Hairline`'s
ceiling is `min(floored, one pixel)` — a **ceiling, not a set** — so it has
nothing to do, and the two renders must come out identical.

# Why this is worth a test of its own

Two reasons, and the second is the one that would otherwise cost a
afternoon.

1. It pins the **contract**: this mode is a ceiling. A future engine that
   implemented it as *"set every stroke to one pixel"* would pass the test
   above and fail here, and the difference is visible on a drawing whose
   producer already emitted hairlines — it would make them THICKER, which is
   the opposite convention arriving through the back door.
2. It makes `SCALE` a **measured boundary rather than a lucky constant**. If
   somebody lowers `SCALE` to 1.0 to make the suite faster, the test above
   starts failing against a perfectly good build; with this one beside it,
   the pair says plainly that the scale is load-bearing and why.

### `fn the_hairline_counter_counts_what_was_thinned_and_not_what_was_drawn`

`app::status::disclosure::line_weights_disclosure` picks between
`text::status::line_weights_off` and `line_weights_no_effect` on
`Diagnostics::strokes_hairlined == 0`. A disclosure whose trigger never
occurs is indistinguishable from one that is broken, so this pins the
**condition** rather than the sentence — the sentence is a `const fn` and
needs no test.

| render | `strokes_hairlined` | which sentence |
|---|---|---|
| a real CAD sheet, mode ON | **> 0** | *line weights are off* |
| the same sheet, mode OFF | **0** | — (no disclosure at all; the toggle is on) |

**The second row is the control and it is the load-bearing one.** With
the mode off nothing is thinned, so the counter must be zero — and if it
were not, it would be counting *strokes drawn* rather than *strokes
thinned*, and the zero sentence would then never appear on any drawing with
linework. The engine states that distinction explicitly (*"it counts strokes
thinned, not strokes drawn"*) and this is what holds them to it.

⚠ It does **not** assert a zero count with the mode ON, because this fixture
cannot produce one: `a1-titleblock.pdf` is a real drawing with real
linework. The zero-with-mode-on case is reached by scrolling to blank paper,
which is a region question and belongs to a driven check, not here.
