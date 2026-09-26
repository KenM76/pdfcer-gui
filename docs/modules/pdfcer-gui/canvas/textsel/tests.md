# `pdfcer-gui/canvas/textsel/tests`

## Item notes

### `fn on_page`

Everything below drives a **real** `PageText`: `PageText`, `TextRun` and
`ExtractedGlyph` are all `#[non_exhaustive]`, so this crate cannot build
one — which is a constraint worth naming rather than working around,
because it means every assertion here is about the engine's actual output
on an actual file.

### `fn on_rotated_page`

[`on_page`]'s twin, and separate from it rather than parameterised,
because the two fixtures come from different trees for a reason
`app::state::open_local_fixture` sets out: the engine's corpus is
read-only and contains no page of rotated strings, so this shell had to
author one.

### `fn on_string`

Derived from the string's own glyphs rather than from the generator's
matrix, so the helper keeps working if the fixture moves — and derived
at all, rather than guessed, for the reason [`first_glyph_centre`]
gives: a coordinate that misses is symptom-identical to a hit test that
is broken.

### `fn sweeping_a_vertical_string_copies_it_on_one_line`

> *"when I copy and paste into notepad, I get the text on one line as
> expected […] as it is now […] it pastes each letter onto its own
> line."*

Sweep the whole 90° string and the clipboard must hold `UPWARD` — six
characters, no newline. Before §8 it held `U\nP\nW\nA\nR\nD`.

The `\n` assertion is separate from the equality on purpose: an equality
that failed would report the whole string and leave a reader to spot the
escapes, and this is the exact character the defect is about.

### `fn a_vertical_selection_is_one_tall_band`

> *"when I select the text it shades each letter as part of the same
> block."*

One quad, not six. And the quad must be **taller than it is wide**,
because the string runs up the page — a box of the right area in the
wrong orientation is exactly what the pre-§8 code produced and would
satisfy a count-only assertion.

### `fn an_upside_down_string_copies_on_one_line_too`

90° and 270° break the extraction's *baseline* clause; 180° breaks its
*backward-jump* clause, because `advance` is published as a positive
magnitude and text advancing in −x therefore looks like a jump of twice
the advance at every glyph. Same symptom, different line of `classify`,
and a fix that handled only the vertical case would have looked complete
on the operator's own file.

### `fn horizontal_text_on_a_rotated_page_is_unchanged`

This is the regression guard with the hardest possible input: a page
whose census is *not* empty, so the rotated path is live, containing
ordinary text that must come out exactly as it always did — one line,
one box, wider than tall.

### `fn the_operators_own_vertical_stamp_comes_back_whole`

`#[ignore]`d, and the reason is a rule rather than a convenience:
`SW41177.pdf` is a customer drawing exported from SOLIDWORKS, and the
standing instruction is that SolidWorks-derived work product does not enter
a repository that could be published. It cannot be committed as a fixture,
so it cannot be a test that runs on a clean checkout — it is a test that
runs **here**, on the operator's machine, against the file the report named.

Run it deliberately:
`cargo test -p pdfcer-gui --lib the_operators_own_vertical_stamp -- --ignored --nocapture`

## What it checks that the synthetic fixture cannot

`fixtures/rotated-text.pdf` reproduces the *mechanism* — see
[`super::fixture`] — but it is a page this project wrote, and a page this
project wrote is a page this project already understood. The real stamp is
82 glyphs of a Windows path in 8 pt Arial, laid down by SOLIDWORKS' own PDF
exporter, on page 36 of a 36-page drawing set, in a title block full of
other text. Three things about it were surprises:

* only **10 of its 72 runs** hold more than one glyph, which is what killed
  the first design (see [`super::writing`] §2.1);
* its worst inter-glyph gap is **0.010 pt** against a 1.600 pt threshold, so
  the strict `Break::None` rule costs it nothing;
* it sits at `x = 1205.8` on a landscape sheet — the far right in the
  *file*, the bottom left **on screen**, because the page carries a
  `/Rotate`. That is the difference `tilt_at` exists to handle and is
  invisible in any fixture authored without one.

### `fn the_cursor_is_told_which_way_the_text_under_it_runs`

> *"In Adobe when I hover over it the I cursor re-orients itself to match
> the text orientation […] as it is now the I cursor doesn't reorient."*

[`tilt_at`] is what turns the I-beam, and the assertion that matters is the
**sign**. `writing` measures directions in PDF user space, which is Y-up;
the cursor lives in canvas space, which is Y-down. A string running **up**
the page is `+90°` in the file and must come back as `−90°` on screen, and
an implementation that forgot the flip would pass every test that only
checked the magnitude — and would then draw a beam that leaned the wrong way
at every angle that is not a multiple of 90°, where nobody would notice
until they met a skewed stamp.

The negative half is asserted too: horizontal text and blank paper must both
answer `None`, so the beam stays upright rather than being told to turn by
zero degrees. Those are the same picture and different costs — `None` skips
a bitmap lookup — but more importantly they are different *claims*, and this
module's job is to make the claim rather than the picture.

### `fn a_skewed_band_is_marked_as_a_parallelogram`

The canvas wash is a `Rect` and therefore over-covers a 30° band at the
corners — that is stated in §8 and accepted. What must NOT happen is the
same approximation reaching [`TextSelection::page_quads`], because those
are written into the file as a text markup's `/QuadPoints` and an
over-covering highlight in a saved document is permanent.

Asserted by the one property that separates the two: a bounding box has
its corners on the page's axes, so `ul.1 == ur.1`. A real 30° band does
not.

### `fn first_glyph_centre`

Derived from the extraction rather than guessed, for the reason
`ui-verify`'s `coords` module gives about guessed points: a coordinate
that misses is symptom-identical to a hit test that is broken, and this
project has already filed one retracted defect on exactly that.

### `fn a_selection_carries_its_text_and_its_boxes_from_one_pass`

The brief's own requirement, asserted the only way it can be asserted
from outside: select every character on the page, and check that the
value carries *both* halves and that they describe the same thing —
non-empty text, at least one box, and a box count that cannot exceed the
number of lines the engine derived.

The last clause is what makes this more than "something was produced": a
build whose grouping key was wrong would emit one box per **glyph**, and
a page of text has far more glyphs than lines.

### `fn every_painted_box_has_the_page_space_quad_a_markup_would_use`

The property the text-markup kinds rest on: the wash the operator sees
and the `/QuadPoints` written into the file are the same boxes, so a
build where one vector was filtered and the other was not would mark
glyphs it never highlighted. Asserted as an equal length **and** as a
per-entry correspondence of *width* — a length check alone would pass on
a build that pushed the right number of wrong quads.

The width comparison is deliberately loose about units: canvas space is
scaled by nothing here (it is page points, Y-down) so on an upright page
the two widths are equal, and the assertion is written as "both
non-degenerate and within a point" rather than as equality, because a
rotated fixture would legitimately swap the axes.

### `fn an_edit_makes_a_selection_stale_and_stops_the_highlight`

Both halves, because the second is the one rule 4 turns on: a stored quad
after an edit may be over different glyphs, and drawing it anyway is the
thing `crate::find`'s staleness section calls out as forbidden outright.

### `fn a_double_click_takes_a_word_and_a_triple_click_takes_at_least_the_line`

The two emphatic gestures, asserted *against each other* rather than
separately: a build where triple-click fell through to the word case
would pass two independent "selects something" tests and fail this one.
A word is also asserted to contain no whitespace, which is what
distinguishes it from a line on any page whose lines have more than one
word — and the test says so rather than assuming it.

### `fn a_plain_click_clears_the_selection`

Expressed as `None` rather than as an empty selection, which is the
invariant `TextSelection`'s own docs rest on: the field on the document
is a two-state question.

### `fn a_drag_selects_the_same_range_in_both_directions`

Dragging right-to-left must select exactly what dragging left-to-right
selected — the case a naive implementation gets wrong by assuming the
press is the earlier position, and the same class of error
`GestureOutcome::Markup`'s docs record for a normalised rect.

### `fn select_all_takes_every_character_on_the_page`

That is exactly the distinction a copy has to get right in the other
direction: [`resolve`] walks the **runs**, derived-whitespace runs
included, which is what makes a copied paragraph paste as a paragraph.
Asserting against `plain_text()` is asserting against the same
segmentation the operator can see on the page.

### `fn a_degenerate_drag_selects_nothing`

Asserted at a point far outside the page box, because
`EditableTextModel::hit_test` deliberately falls back to the *nearest*
line rather than answering `None` — so the clearing has to come from the
range covering no glyphs, and a build that had "nearest line" leak into a
selection would fail here rather than in front of an operator.

### `fn overshooting_the_end_of_a_line_keeps_the_selection`

Measured on this fixture before the clamp existed: a drag from the start
of `HORIZONTAL` to any point past `x = 161.34` — its box plus one
line-height — returned `None`, so the wash vanished mid-gesture and the
operator was left holding nothing. On a drawing sheet that is most
sweeps: `fixtures/layered-drawing.pdf` carries a 396 pt note on a 2,384 pt
page, so a sweep across the sheet leaves reach after one sixth of it.

Both target points sit in the **gap** between `HORIZONTAL`'s reach and
`DOWNWARD`'s, which starts at `x = 222.35` — the vertical line's reach is
its own 74.65 pt height, not its 12 pt size. A target past that would
select through to it, which is the next test.

### `fn the_clamp_finds_the_furthest_text_the_drag_passed_not_the_nearest`

This is what a bisection seeded from the anchor could not do: it would
converge on the near edge of the gap at `x = 161.34` and silently
under-select, which looks like a working clamp until the day two columns
are swept at once.
