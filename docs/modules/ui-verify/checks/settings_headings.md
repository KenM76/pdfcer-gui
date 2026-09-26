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

## Item notes

### `const HEADING_PREFIX`

Matched **literally**, so it is part of the contract with
`crate::dialogs::settings::REGION_HEADING_PREFIX`. That constant's own doc
comment states the other half of the bargain: the key is deliberately not
derived from the caption, because a caption is operator copy that may be
reworded or translated, and a check aimed at a region named after it would
silently stop finding its subject and report *a heading that is not there*
rather than *a heading that is illegible*. Those are different verdicts and
only one of them is true.

### `const CONVENTION`

Completes the sentence "the application declared no …", so it names this
check's own convention rather than a generic one — a reader who gets this
SKIP should know exactly which string to grep the application for.

### `fn open_settings`

`file.settings` is the first item of the *pdfcer* group, which is the LAST
group on the File tab. At the shipped 1100 pt window width that group does
not fit, so the item has no rect of its own — and there are **two different
things** the ribbon may have done with it:

| | what the trace shows | how to reach the item |
|---|---|---|
| on the band | `ribbon.item.file.settings` | click it |
| folded into the overflow | `ribbon.overflow` | open the overflow, then click |
| **collapsed as a group** | `ribbon.group.file.pdfcer.collapsed` | click the group button, then click |

The third arrived with the O31 ribbon work, which gave the band a middle
rung: when it runs short of width a whole group folds into one captioned
button whose items live in its popup. This function hand-rolled the first
two and therefore **could not run at all** afterwards — every invocation
SKIPped with *"neither `ribbon.item.file.settings` nor `ribbon.overflow` was
declared"*, which was true, and not the whole truth: the trace it printed
contained `ribbon.group.file.pdfcer.collapsed` five lines down.

**It SKIPped rather than FAILed, and that is the only reason this was
cheap.** A check that had claimed the Settings control was missing would
have sent somebody looking for a defect in a ribbon that was behaving
exactly as designed — the false-failure-believed pattern this suite has paid
for twice. The honest SKIP cost nothing but the coverage.

# The fix is to stop hand-rolling it

`driving::declared_or_in_overflow` already knows all three places and tries
them in the right order — direct, then each collapsed group (non-destructive:
a popup can be opened and closed without moving the band), then the overflow
(which scrolls, and so must be last). It was written for exactly this and
`export_dxf` already uses it.

**A rule stated twice is a rule that drifts**, and this is what the drift
looks like: the shared helper gained a third case, this copy did not, and
nothing failed — the check simply stopped being able to begin. There is now
one statement of "where can a ribbon command be".
