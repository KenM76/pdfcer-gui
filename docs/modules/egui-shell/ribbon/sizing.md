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

### `const LARGE_STACK_GAP`

`4`, from the mockup's `.rb.big { gap: 4px }`. Two points is enough when
the glyph is 16 pt — vertically the two parts are already separated by the
icon's own bottom edge and the label's ascent — but the glyph here is 24 pt
([`crate::theme::Metrics::ribbon_icon_large_pts`]), and a two-point gap
under a half-again-larger picture reads as the label touching it.

### `const LARGE_SIDE_PADDING`

A Large button's content is centred rather than left-aligned, so it needs
symmetric breathing room; the ordinary button padding is tuned for a row of
text and looks tight around a centred icon.

`8` — `.rb.big { padding: 5px 8px 2px }`, the horizontal figure. Narrow
on purpose: a Large control pays for its presence with **height and glyph
size**, not with width, so a row of them sits closer together than the side
padding alone would suggest.

### `const LARGE_MIN_WIDTH`

Why a floor is needed at all, when the control is already as wide as its
widest part plus padding: because its widest part can be *tiny*. A Large
control whose label is `Save` and whose glyph is 24 pt measures
`24 + 16 = 40` pt, and a run of Large controls that changed width with
every word would read as a ragged fence rather than as a row of equal
buttons. Word, Acrobat and the mockup all pin a floor; this is the
mockup's.

It is **not** applied to the label's wrap width — see
[`LARGE_LABEL_WRAP`]. A floor that also widened the text would make the
floor unreachable, because every label would grow to meet it.

### `const LARGE_LABEL_WRAP`

**The label WRAPS.**

This is the half of the operator's *"text label location"* complaint that
is not a manifest change. A Large
control is *icon above label, centred*, and the labels this ribbon
carries are sentences by button standards — `Recognise text…`,
`Save a compacted copy…`, `New from template…`. Laid out on one line,
such a control is 120 pt wide and 56 pt tall: a letterbox, which is not
what a Large control looks like anywhere, and which pushes the groups
beside it off the band.

So the label is laid out into a galley of at most this width and may take
two lines (or more — nothing here caps the line count, because a cap would
mean silently dropping a word, and `RIBBON_SCALING.md`'s ladder is this
project's answer to "it does not fit"). The height that galley reports is
then part of the control's own content height, which is what stops a
two-line label from being clipped by
[`crate::theme::Metrics::ribbon_large_pts`].

### `fn visible`

The `visible_when` filter, applied **before measurement**, which is the
whole point: a hidden item must not merely be skipped when drawing, or the
group reserves space for a control that never appears and the band's plan
is wrong by exactly the width of every hidden item.

This is **visibility**, not enablement, and R9 draws the line: *an
unavailable capability renders nothing; greying is reserved for
**temporarily** unavailable and is always explained on hover.*
[`Command::enable`] is the greying — no document open, empty undo stack.
This is the disappearing — the command does not apply on this surface, in
this mode, in this build.

An item with no condition is always visible, which is nearly all of them.

### `fn resolved`

See the module header: `Small` is earned. `can_paint` is whether the
application installed an icon painter at all — a manifest asking for
icon-only controls in a build with no icons would otherwise draw a band of
empty rectangles.

`Large` is **not** conditional on an icon. A large button with no icon is
a large label, which is odd-looking but legible and unambiguous; a large
button with an icon and no label would be the mystery, and `Large` always
draws its label.

### `fn width`

This and [`render`] are one decision written twice, and they must not
diverge: a control measured at one width and drawn at another is how a band
that "claims to fit" clips its last group. `band`'s own comment makes the
same point about the icon slot. Every branch here has a matching branch
there, in the same order, and
[`crate::ribbon::width_tests`]'s `a_band_that_claims_to_fit_really_does_fit`
is what would catch a drift between them.

### `fn render_large`

# Why this is built by hand rather than from `egui::Button`

`egui::Atoms` lays out with `push_right`; there is no vertical form, so a
`Button` cannot stack an icon over a label. The alternatives were a nested
`Ui` inside a frame that *looks* like a button — which does not respond
like one, and gets the hover and pressed visuals wrong on the day the theme
changes — or this: allocate the rect, take a real `Response`, and paint the
button's own `WidgetVisuals` into it.

Painting from `ui.style().interact(&response)` rather than from theme
colours directly is what keeps a Large control identical to every other
button under hover, focus, disabled and selected. A hand-drawn control that
picked its own colours is the shape of the defect this project's
`check-theme-colors` gate exists to refuse.
