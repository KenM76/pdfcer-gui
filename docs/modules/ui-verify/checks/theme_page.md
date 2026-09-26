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
