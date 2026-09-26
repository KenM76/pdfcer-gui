# `ui-verify/checks/settings_headings`

`settings_headings_legible` — the regression test for **D2**.

# The defect

Every collapsible section heading in the Settings dialog — *Appearance,
Theme, Colour, Images and transparency, Copying and extracting text, Pages
and printing, Saving files* — renders near-white on light grey. So do the
dock tab labels. At 1× they are simply not readable.

## Cause

the old GUI's `theme.rs` loops over all five widget states setting
`corner_radius`, `bg_stroke` and `fg_stroke`, and then:

```text
v.widgets.inactive.weak_bg_fill = p.panel;
v.widgets.hovered.weak_bg_fill  = p.surface;
v.widgets.active.weak_bg_fill   = p.accent;
v.widgets.active.fg_stroke      = Stroke::new(1.0, p.label_backdrop);
```

`label_backdrop` is `rgba(250,250,250,220)`. Pairing it with the accent is
*correct* — light text on an accent fill. But only `weak_bg_fill` is
assigned the accent. **`widgets.active.bg_fill` is never set at all.** So
widgets that paint their background with `bg_fill` rather than
`weak_bg_fill` — `egui_tiles` tab buttons, `CollapsingHeader` headers — get
the near-white foreground on a light background.

# Why CI did not catch it

Two tests sit directly adjacent and neither covers it:

* `text_contrasts_with_its_background_in_every_preset` checks `text`
  against `surface`/`panel` and `text_muted` against `surface`. It never
  tests `label_backdrop`.
* `label_plates_stay_page_facing_not_chrome_facing` **asserts that
  `label_backdrop` stays light** — correct for its stated purpose, because
  labels also sit over the white page — without checking what is actually
  behind it in chrome.

Both test the *palette*. The defect is in the *pairing*, and the pairing
only exists once something is drawn. There is one oracle for that, and it
is the rendered screenshot.

# How this check detects it

It measures the WCAG contrast ratio of each heading's rendered region
against its own background, using the population algorithm in
[`crate::pixels`] rather than a min/max that a single stray pixel could
fake. The threshold is WCAG 2.1 AA for large text, 3:1 — a published
standard rather than a matter of taste, which is what stops a failing check
becoming an argument about whether the grey is nice.

The defect measures around **1.1:1**.

# Two modes, and why the offline one exists

* **Live** — drive the application to its Settings dialog and capture. This
  is the mode the new application will use. Half of what it needs now
  exists: `diag::ui_rect` is in the new binary and the dialog's headings
  would be located by declaring themselves, with no fractions for the
  harness to hard-code and no calibration to go stale when the dialog is
  resized. The other half does not: **there is no Settings dialog yet**,
  and no scripted step that would open one.
* **Offline (`--image`)** — assert against a screenshot somebody already
  captured. This exists for falsification: `evidence/crop_settings.png` is
  the dated artefact `DEFECTS.md` D2 cites, and running this check against
  it is how the harness demonstrates it detects the real defect rather than
  merely claiming to.

So the live mode SKIPs against both binaries, for a reason that is about
**modal state** rather than about the trace: the new application has no
Settings dialog, and the old one has a dialog with no scripted way in.
Pointing this check at the old GUI's own captured evidence reports FAIL.
Both are honest, and the second is the acceptance criterion.

Note that this is exactly why this check does not launch anything, while
[`super::ribbon_captions`] does. A ribbon is chrome — it is on screen as
soon as the window is, so its trace answers the question. A dialog is not,
so launching would confirm only that a dialog nobody opened declared no
regions, and would put a window on the operator's desktop to do it.
