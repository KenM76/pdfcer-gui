# `pdfcer-gui/app/status/tests`

## Item notes

### `fn an_edit_disclosure_does_not_change_the_bar_height`

# Why this needs its own test beside the fill one

Same rule, different arrival. A fill disclosure follows the operator
typing into a field; this one follows a **drag on the canvas**, which
is the gesture during which a re-fit is most damaging — R128's measured
symptom is *"the page jumped when I clicked an object"*, and a drag is
a click that has not finished. If this line grew the bar, the page
would re-fit on the frame the operator released the mouse, the object
would land somewhere other than where they dropped it, and the
investigation would start in the move code, where nothing is wrong.

# The three assertions, and why none of them is the obvious one

1. **A measurement happened at all** (`Some(_)`, never `None`) — see
   [`bar_frame`] for why a bare `f32` would let a vacuous run pass.
2. **The sentence reached the painter** — more shapes with the
   disclosure live than without it. Without this, assertion 3 is
   satisfied just as well by a `disclosure_line` that returned early
   and drew nothing, which is true and proves nothing.
3. **The height did not move.** Asserted as `Some(true)` rather than
   with a bare `assert!`, so a run in which either frame failed to
   measure reads as `None` and fails, rather than reading as agreement.

The planted notes are the **worst case that can actually occur**: two
of `pdfcer-core`'s real sentences at once, which is what a node drag
that both expands a rectangle and materialises an implicit start
returns. They are long, and long is the point — the defence against
them is eliding inside a bounded sub-region with the whole text on
hover, not wrapping, because wrapping is how a one-row bar becomes a
two-row bar.

### `fn the_panel_is_tall_enough_for_the_controls_the_theme_actually_draws`

`the_bar_is_exactly_as_tall_open_as_closed` asserts the same property
and passes against a build where it is false, because it measures in an
`egui::Context::default()` — egui's own spacing, not this application's
theme. In that world the controls are under 24 points and the assertion
is true. In the real window `Metrics::control_height` is 28, a button
adds 2 points of padding on each side, and the bar's content is 30
against a panel whose content box was 26.

Measured on a real window at both scales: `status-bar 972.0 .. 1002.0`
in a 1000-point client at 1.00, and `416.4 .. 446.4` in a 444.4-point
client at 1.80. Two points of two controls clipped off the bottom of the
window, at every scale.

So this one **applies the theme** and asserts against every preset the
application ships, not just the current one — because what introduces
this defect is a preset raising its control height.

### `fn the_bar_is_exactly_as_tall_open_as_closed`

Rule R128, asserted rather than argued. A status panel whose height
varies feeds the fit-to-viewport recompute, and the measured result
on pdfcer was a page that shrank 230 % → 224 % → 215 % across three
frames with no zoom input, plus click coordinates that went stale
between the frame they were captured on and the next render. The
symptom reads as a selection bug and gets investigated in the
selection code, where nothing is wrong.

This is the property that forbids an [`egui::CollapsingHeader`] here:
changing its own height is the entire behaviour of that widget. It is
also what `ui.set_min_height` in [`show`] is for — without it the row
would shrink to whatever the content happened to need.

### `fn every_glyph_the_status_bar_draws_has_a_glyph`

`⏴`, `⏵`, `⏷`, `−` and `·` are not decoration: three of them are the
entire visible text of a control. A codepoint the font set cannot
draw renders as a tofu box, which is defect D2's shape — an invisible
label — with the operator's page position behind it.

The obvious choices for this catalog — `◀` `▶` for the page steps and
`▸` `▾` for the disclosure — are all four missing from egui's bundled
fonts (Ubuntu-Light + NotoEmoji + emoji-icon-font), and would ship as
four tofu boxes on the two controls an operator touches most.

Checked against `FontFamily::Proportional`, which is what every label
and button on this bar resolves to, and run inside a real pass because
egui has no fonts before one.

## Why this asks `GlyphProbe` and never [`epaint::Fonts::has_glyph`]

`has_glyph` returns `resolve_face(c) != replacement_face_key`, so it
says "no" to every codepoint whose first supporting face happens to be
the face that also supplies `epaint`'s substitution mark `◻` — which
for the proportional family is `NotoEmoji-Regular`, the supplier of
`⚠`. It reports `⚠` (U+26A0) as undrawable when it draws, and
`DEFECTS.md` D12 records what believing that cost: thirteen shipped
sentences recorded as rendering tofu when they render correctly.

[`crate::text::glyphs::GlyphProbe`] instead lays the character out and
looks at what was drawn. The full mechanism, the measurements and the
three-sentinel fingerprint are in that module's header.

**The mark on the edit-disclosure line is `⚑` and stays `⚑`.** It
draws, it is in the bar, and re-opening a settled copy decision on the
strength of a corrected diagnosis is churn, not a fix.

This gate is the narrow, hand-listed one; the broad one is
[`crate::text::glyphs::tests::every_glyph_the_catalog_draws_has_a_glyph`],
which reads the whole catalog from source and needs no list.

### `mod disclosure_independence`

Asserted as a truth table because the obvious mistake, when a third line is
added beside two existing ones, is to make them alternatives — an `else if`
chain that shows whichever fires first. They answer different questions and
can all be true at once:

| line | answers |
|---|---|
| fill | what a form fill had to INFER |
| edit | what a move or delete had to change about an object's FORM |
| recovered | how this FILE was assembled before anything was drawn |

A document opened from a damaged index, edited, and with a form filled owes
the operator all three.

### `fn the_overflow_case_says_the_text_will_not_fit_and_the_others_do_not`

Calling `forms_fill_autosize_note` for **every** auto-size outcome tells an
operator whose field is too small for its text *"pdfcer chose 6.0 pt.
Another program filling this field may choose differently."* — a sentence
about interoperability, when the fact is that **the text is going to
overflow the box**.

The engine reports the difference. `AutoFitBound::Floor` exists for exactly
this and its own branch comment reads *"the one case where the returned
size does NOT fit the constraint that produced it"*, so a shell that reads
`applied_autosize` and drops `applied_autosize_bound` has thrown the answer
away.

`OPERATOR_REQUESTS.md` **O86** tells the operator that *"pdfcer now tells
you which way it decided … held at pdfcer's legibility floor; the box is
too small for this text, which will overflow"*. That is true of the engine
and the CLI, and a shell that drops the bound makes it false at the one
surface the operator reads — the worst of the three states a claim can be
in.

### `fn no_bound_reported_takes_the_general_sentence_and_never_claims_overflow`

`None` is a real state, not a missing one. The engine declines to name
a bound for multiline because that route derives from the whole box height
and naming a constraint *"would report a constraint that was never
evaluated"*. Reading `None` as `Height` would be the same error as reading a
missing texture as zero thinned strokes (`O137`) — **a measurement that did
not happen is not a measurement of zero.**
