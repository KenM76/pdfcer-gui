# `egui-shell/ribbon/sizing`

## Item notes

### `fn large_label`

[`width`] and [`render_large`] are one decision written twice and must
not diverge — the module's own standing warning, and the reason
`crate::ribbon::width_tests`'s `a_band_that_claims_to_fit_really_does_fit`
exists. A wrapped label makes that warning sharper, because the wrap point
is a *font-dependent* fact rather than an arithmetic one: two call sites
that each asked `egui` to wrap "the same" text could disagree over a font
change, a size change, or a stray trailing space. They now cannot, because
there is one call.

[`egui::Color32::PLACEHOLDER`] rather than a real colour so the galley is
the **same cache entry** the painter will later ask for — `egui` memoizes
layout jobs under a placeholder colour and substitutes the real one at
paint time, so measuring costs a hash lookup rather than a second layout.
It is also what lets one galley serve the enabled, disabled and selected
paints, which differ only in ink.

### `fn only_a_disabled_scope_produces_a_disabled_response`

This is a claim about **egui**, so it is asserted against egui rather
than reasoned about. A control painted greyed by choosing
`visuals.widgets.inactive` by hand, but allocated from the ordinary
`Ui`, keeps `response.enabled() == true`: its `on_disabled_hover_text`
never opens, and `ribbon::control`'s
`if response.clicked() { ctx.invoke(…) }` invokes the command anyway.
The band says no and the shell does it regardless.

The second case is what this crate relies on: allocating inside
`ui.disable()`'s scope produces a response that reports itself
disabled, which is what both the tooltip and the click gate read.

Written as a table over both cases rather than asserting only the
second, because a build in which BOTH were disabled would satisfy a
one-sided assertion and would grey every Large control permanently.

### `fn frame_when_inactive_removes_the_resting_ink_and_not_the_rectangle`

Asserted **against egui**, in the shape of
[`only_a_disabled_scope_produces_a_disabled_response`] above, because
it is a claim about a library rather than about this crate. The
operator's complaint was *"every ribbon item in the real build is drawn
with a visible button FRAME … the mockup draws them frameless"*, and
`crate::ribbon::control::command_button`'s answer is one builder call.
That call is only safe if two things are true of egui, and neither is
obvious from the method's one-line doc:

1. **The button does not move.** `egui-0.35.0/src/widgets/button.rs:364-368` keeps
   `frame.inner_margin` in *both* branches — the framed one and the
   margin-only one — so a control that loses its resting ink keeps its
   rectangle. Everything the band plans is built on
   `measure::button_padding`, and a frameless button that measured
   differently from a framed one would make every planned group width
   a lie at rest and true under the pointer.

2. **The ink really goes.** A frameless resting button emits strictly
   fewer paint shapes than a framed one. Without this half the test
   would pass against an implementation where the flag did nothing at
   all — which is precisely the vacuity this test exists to avoid.

The second assertion counts `Shape::Rect`s recursively, because
`egui` nests shapes (`Shape::Vec`) and a top-level count would miss a
frame painted inside a group. Counting *fewer* rather than an exact
number is deliberate: the button also paints its text and, in the band,
its icon, and pinning a total would make this test fail every time
something unrelated changed how a label is emitted.

### `fn inked`

Counting bare `Shape::Rect`s does not work, and finding that out
is the useful half of this test. `egui`'s margin-only branch still
emits a `RectShape` — `Frame::paint` always does — just one whose
fill is `TRANSPARENT` and whose stroke is `Stroke::NONE`. A count
of rectangles is therefore 1 in both cases and would have passed
against an implementation where the flag did nothing at all, which
is exactly the vacuity this test exists to avoid.

### `fn small_is_earned_and_falls_back_when_it_is_not`

Asserted as a table over all eight combinations, because the rule is a
conjunction and a test of one clause at a time would pass against an
implementation that had dropped a different one.

### `fn only_small_is_ever_downgraded`

`Large` deliberately does not require an icon: a large button with no
icon is a large label, which is legible. The mystery this rule guards
against is an icon with no name, and `Large` always draws its label.
