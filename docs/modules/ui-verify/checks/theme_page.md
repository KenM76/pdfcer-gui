# `ui-verify/checks/theme_page`

`every_theme_preset_keeps_the_page_white` — the harness's two theme blind
spots, closed in one check.

# The two gaps it closes

A suite that drives one preset never drives **Airy**, and a suite that
samples the window body never samples the **page canvas** under a theme.

Both are about the same three radio buttons that
[`super::settings_theme`] already drives, which is why this began life
inside that file and moved out of it: the two checks together crossed rule
R2's 1,500-line ceiling, and the seam R2 forced is a real one. That module
asks *does choosing a theme change the program*; this one asks *and what did
it change that it had no business changing*. They share three clicks —
[`super::settings_theme::open_the_theme_picker`] — and nothing else.

## The Airy preset was never driven — and it is the worst one

[`super::settings_theme::SettingsThemeTakesEffect`] clicks exactly one radio, `Dark`, because Dark
is the preset whose effect is unmistakable. `Quiet` is the default and gets
measured by being the *before* picture. **`Airy` was clicked by nothing in
this repository.**

That is not a tidy gap. Airy is the preset most likely to fail a contrast
assertion, measured: on the two defects found by the same review — the
selected dock tab and the document tab's close ✕ — the luminance gaps under
Airy were **28.2 and 5.0**, against 45 and 18 under the presets that *were*
driven. Airy's panel is pure white (`#FFFFFF`) and the 27 % selection wash
barely darkens it, so white-on-white is five levels of luminance away. ⇒
**The preset most likely to fail is the one nothing drove.**

[`EveryThemePresetKeepsThePageWhite`] drives all three.

## Nothing sampled the PAGE under a theme — only the window body

Every theme oracle in this project, this file's original check included,
measures **chrome**: a dialog body, a rendered widget pair, a palette.
Nothing had ever asked what the theme did to the **sheet**.

**That is the single invariant a dark theme in this product must hold.**
pdfcer draws CAD drawings. A dark chrome is a comfort; a *tinted sheet* is an
unreadable drawing, because the linework's contrast is the whole content and
the paper is the reference the eye reads it against. `egui_shell::theme`
knows this and says so — `Preset::Dark`'s own doc comment is *"Dark chrome
against light content, as CAD tools do it"*, and its `label_backdrop` and
`label_text` deliberately stay dark-on-light *"because they sit over
CONTENT, whose colour the document decides and the theme does not."*

A stated intention held by nothing but a comment is exactly the shape of
defect this suite exists for. [`EveryThemePresetKeepsThePageWhite`] measures
the page raster itself, under each of the three presets, and asserts it does
not move.

# Everything else about how this measures

is on [`EveryThemePresetKeepsThePageWhite`] itself, which carries the oracle
argument, the vacuity table and the two witnesses. The constants each carry
their own derivation, including the one that was **wrong on the first live
run and corrected against the pixels** — see [`MIN_PRESET_DISTINCTION`],
which is also where a finding about `Palette::content_backdrop` is recorded
for somebody else to act on.

## Item notes

### `const PRESETS`

This is `egui_shell::theme::Preset::ALL` restated as strings, and the
restatement is the point rather than a duplication to be apologised for:
`ui-verify` does not compile against `egui-shell`, so the *only* way this
crate can know the set is to write it down — and writing it down is what
makes an omission visible.

A preset added to the shell and not added here ships undriven, which is the
condition `Preset::ALL`'s own comment warns about in the other direction:
*"A preset missing here ships unverified."*

### `const PAGE`

**The sheet itself, not the area around it.** Its sibling
[`CANVAS_VIEWPORT`] is the scroll area the sheet sits in, and that module's
own comment says why confusing the two is a real error rather than a
pedantic one: at fit-page the two rects differ by the centring margin, and a
check that measured one while meaning the other *"would sample the grey
surround"* — which is the backdrop this file deliberately samples on
purpose, elsewhere, for the opposite reason.

### `const CANVAS_VIEWPORT`

Needed for the **backdrop band**: the strip of canvas surround between the
viewport's edge and the sheet's. It is the one surface that proves a preset
took effect *in the same capture that measures the page*, and it is measured
rather than assumed — see [`MIN_PRESET_DISTINCTION`] for what it turned out
to be painted with, which is not the role named for the job. See
[`backdrop_band`].

### `const MIN_PRESET_DISTINCTION`

# The derivation, and the wrong one it replaced

This constant was first written as 10, derived from `Palette::content_backdrop`
— the role whose own doc comment says it exists for exactly this surface:
*"the area behind the application's main content — deliberately its own role
rather than reusing `surface`, because the content must read as an object ON
something, and a backdrop equal to the panel makes its edge disappear."*
Quiet `#6E7074`, Airy `#8A8D93`, Dark `#16171A`: a closest pair of 28.

**The first live run measured 242, 249 and 36.** Those are `Palette::surface`
— `#F2F2F3`, `#FAFAFB`, `#24262A` — so the canvas surround is *not* painted
with `content_backdrop`, and a grep confirms nothing in either crate reads
that role at all. The number was corrected to what the application actually
paints; the finding was reported rather than absorbed, because it is the
condition that role was written to prevent and it is not this crate's to fix.

| preset | measured surround | palette role | distance from `quiet` |
|---|---|---|---|
| `quiet` | 242, 242, 243 | `surface` `#F2F2F3` | — |
| `airy`  | 249, 249, 250 | `surface` `#FAFAFB` | **7** |
| `dark`  | 36, 38, 42    | `surface` `#24262A` | **206** |

**4** sits below the closest real pair and above the reading a run in which
nothing happened produces — which is **0**, not a small number, because two
identically painted regions in a lossless capture are identical.

Seven is a thinner margin than this file would choose, which is why colour
is not the only witness: [`MIN_METRIC_SHIFT_PTS`] gives the `quiet`↔`airy`
pair a second, independent one, and the two are combined with an `or`.

### `const MIN_METRIC_SHIFT_PTS`

The second witness, and it is the one that makes the Airy assertion solid.

Airy is the only preset that changes `Metrics` as well as `Palette` —
`control_height` 24 → 28, `gutter` 4 → 8, `panel_padding` 6 → 12,
`corner_radius` 3 → 6 — so the ribbon above the canvas gets taller and the
canvas viewport's top edge moves down with it. Measured on the first live
run: **143.3 → 175.0 logical points**, a shift of 31.7.

Dark inherits `quiet.metrics` verbatim (`metrics: quiet.metrics` in
`Theme::dark`), so that pair shifts by exactly 0.0 and is distinguished by
colour instead. Between them the two witnesses cover all three pairs with a
wide margin each, which no single one of them does.

And the first live run gave this constant a second job nobody planned.
Under Airy the canvas surround measured **249, 249, 250** and the sheet
measured **249, 249, 249** — the paper and the surface it sits on are the
same colour to within one level, which is the very outcome
`Palette::content_backdrop` was declared to prevent (*"a backdrop equal to
the panel makes its edge disappear"*). A colour witness cannot tell a
correctly-aimed backdrop sample from a mis-aimed one that landed on the
sheet, under that preset, at all. The layout witness can, and does.

**2.0** points: a layout that did not change reports a difference of exactly
zero — these are floats straight from the trace, not measurements — so the
floor only has to sit above float formatting, and it sits an order of
magnitude below the 31.7 it is looking for.

### `const PAGE_MIN_CHANNEL`

A page raster with white paper measures 255 on every channel; a JPEG-ish
off-white scan or a page whose producer filled it with `0.98 g` measures in
the high 240s. **235** admits both and excludes anything a *theme* could
plausibly do — the palette's lightest chrome surface (`airy`'s `#FFFFFF`
panel) is white, and its darkest (`dark`'s `#16171A`) is 22, so there is no
near-miss to worry about.

This is the ABSOLUTE half of the page assertion and it is the weaker half.
The one that carries the argument is [`MAX_PAGE_DRIFT`]: *the paper is the
document's colour and the theme has no vote on it*, which is a claim about
**movement** and needs no opinion about what colour the fixture's paper is.

### `const PAGE_MAX_SPREAD`

A neutral white has equal channels. A theme that tinted the sheet would
almost certainly tint it toward its own hue rather than merely darken it —
every preset in this shell is built around a blue accent — so an unequal
R/G/B on the paper is the specific fingerprint of the defect. 8 is one
quantisation bucket, i.e. the sampler's own noise floor.

### `const MAX_PAGE_DRIFT`

**Zero is the expected reading**, for the reason [`MAX_REVERT_DRIFT`] gives:
the page raster is rendered by `pdfcer-core` from the document's own content
and the preset is not an input to it, so the same pixels are painted. 6 is
one quantisation bucket, and it is far below any tint worth the name — the
mock-up that prompted this check keeps its page at `#FFFFFF` while its
chrome goes to `#16171A`, a distance of 233.

### `const MIN_BAND_PTS`

Below this the band is mostly the sheet's own drop shadow and antialiased
edge rather than the backdrop, and the dominant colour it reports would be a
blend of the two — a measurement that moves for reasons that have nothing to
do with the preset. 16 points is comfortably more than any edge treatment in
this shell and comfortably less than the ≈ 140-point margin a fit-page view
leaves above an A-size sheet in a maximised window.

A run with no band this thick **SKIPS**: see [`backdrop_band`].

### `fn backdrop_band`

# Why a band rather than the viewport

Because the viewport CONTAINS the page, so its dominant colour at any
ordinary zoom is the paper — the very thing this reading must be independent
of. The band is the part of the viewport the sheet is not on, which is the
surface the shell paints its canvas surround with.

# The geometry: all FOUR sides, and the thickest wins


So all four candidates are considered and the **thickest** is taken —
thickness being the dimension across the strip, which is the one that decides
whether the sample is backdrop or edge treatment.

Each candidate is clipped along its long axis to the sheet's own extent
rather than the viewport's: that keeps the sample clear of the viewport's
corners, where two strips meet, and it means a strip never contains the
ruler, the corner box or a scrollbar track.

# The inset, on both axes

The middle 60 % in each direction. The outer fifths hold the sheet's drop
shadow at one end and the viewport's own boundary at the other; a dominant
colour taken across either is a blend that drifts for reasons unrelated to
the preset. On the 585-point strips above, a fifth is 117 points — far more
than any scrollbar or edge treatment this shell draws.

# `None`, and why it is a SKIP rather than a fallback

When no strip reaches [`MIN_BAND_PTS`] there is genuinely no backdrop on
screen: the sheet fills its viewport, which happens at any zoom past fit.
The alternative to skipping is sampling the sheet's own shadow and calling it
the backdrop, which would produce a *number* — and a number is what a caller
cannot tell from a measurement.

### `fn is_white_paper`

Both halves are needed and they catch different failures: the channel floor
catches a sheet that was **darkened**, the spread catches one that was
**tinted**. A theme built around a blue accent would do the second, and a
check that looked only at brightness would let a faintly blue sheet through
at full luminance.

### `fn measure_preset`

The outer `Result` is the SKIP channel; the inner one is FAIL (`Err`) versus
a measurement (`Ok`).

# Every rect is re-read after the click, and none is carried in

`airy` changes the shell's METRICS as well as its colours — `control_height`
24 → 28, `panel_padding` 6 → 12, `gutter` 4 → 8 — so the ribbon is taller,
the docks are wider and the canvas has moved by the time the capture is
taken. A page rect read before the click is a rectangle the sheet has slid
out of, and sampling it would report a tinted page about a build that is
fine. This is the single most likely way for this check to produce a false
defect report, which is why the re-read is not an optimisation to fold away.

### `struct EveryThemePresetKeepsThePageWhite`

# The invariant, and why it is the one that matters

pdfcer draws CAD drawings. A dark chrome is a preference; a **tinted sheet
is an unreadable drawing**, because the linework carries all of the content
and the paper is the reference the eye measures it against. Grey a drawing
sheet by fifteen levels and every hairline on it loses the contrast it was
drawn with — and unlike a chrome regression nobody files it as a bug,
because a drawing that is merely *hard* to read still looks like a drawing.

The shell already believes this. `Preset::Dark`'s own doc comment is *"Dark
chrome against light content, as CAD tools do it"*, and it keeps
`label_backdrop` and `label_text` light-plated with a stated reason —
*"because they sit over CONTENT, whose colour the document decides and the
theme does not."* A dark board that keeps the page white is the single
invariant a dark theme in this product must hold.

Stated in three places and enforced in none is the exact shape of every
defect this suite exists for: **a rule cited in a comment near the code, and
not enforced by a mechanism inside it.**

# What it measures, and why two things rather than one

Per preset, from **one capture of the application's own window**:

| sample | region | the claim |
|---|---|---|
| the sheet | `page` | it did not move, and it is white |
| the surround | a band of [`CANVAS_VIEWPORT`] outside `page` | it DID move |

The second is not decoration; it is what stops the first being vacuous.
*"The page stayed white"* is trivially true of a build in which the click
never landed, the radio does nothing, the theme is not installed, or the
window never opened. A check asserting only the page would pass on all four
and report a property it had never exercised. So each preset must be shown
to have **changed the surround** before its page reading is admitted as
evidence, and a run where it did not is a SKIP naming
[`SettingsThemeTakesEffect`] as the place that diagnosis lives.

And the surround is the right witness rather than a convenient one: it is
the pixel **immediately adjacent to the sheet**, in the same capture. A theme
that reached the page would have had to reach it through there. Sampling the
dialog instead would prove the theme changed *somewhere*; this proves it
changed at the page's own edge and the page did not follow.

Measuring it also found something nobody had looked for.
`Palette::content_backdrop` exists precisely for this surface and says so —
*"deliberately its own role rather than reusing `surface`, because the
content must read as an object ON something, and a backdrop equal to the
panel makes its edge disappear"* — and the surround measures `surface` under
all three presets. **Nothing in either crate reads `content_backdrop` at
all.** So the sheet sits on the same colour as the panels and its edge is
exactly as invisible as that comment predicts. This crate reports that and
does not assert on it: it is a finding about the theme, not about the
invariant under test, and the fix is a call site in another crate.

# It drives all three presets, and Airy is the point

[`SettingsThemeTakesEffect`] clicks Dark and nothing else, so without this
check **nothing in this repository clicks Airy** — and Airy is the preset
most likely to be wrong, measured: two known contrast defects have luminance
gaps of 28.2 and 5.0 under Airy against 45 and 18 under the presets that are
driven, because Airy's panel is pure white and a 27 % wash barely darkens
it. The preset nothing drives is the preset most likely to fail.

Each preset must also measure **distinct from the others**
([`MIN_PRESET_DISTINCTION`]), which is a real assertion in its own right: a
radio that is drawn, publishes a rect, accepts a click and selects a preset
whose palette is never installed is `DEFECTS.md` D10 confined to one preset,
and Dark-only coverage structurally cannot see it.

# What would make this vacuous, and what is done about each

| vacuity | guard |
|---|---|
| the fixture's page is not white paper | measured under the light preset FIRST and SKIPPED, naming the file |
| the clicks never landed | the surround must move per preset, else SKIP |
| the sheet fills the viewport, so there is no surround | [`backdrop_band`] returns `None` and the check SKIPS |
| a capture of the wrong window | the page is read from the APPLICATION's frame, re-raised per preset |
| a stale rect after Airy re-flows the layout | every rect is re-read from the trace after every click |

That last row is not hypothetical. Airy is the one preset that changes
**metrics** as well as colours — `control_height` 24 → 28, `panel_padding`
6 → 12, `gutter` 4 → 8 — so the ribbon grows, the docks resize and the canvas
moves. A check that computed the page rect once and reused it would, under
Airy and only under Airy, sample a rectangle the sheet had since slid out
of. It would report a tinted page, confidently, about a build that is fine —
the exact failure this project's own rule warns of: **ask what a failing
pixel check SAMPLED before asking what is broken.** Six false defect reports
were filed here from one wrong page index.
