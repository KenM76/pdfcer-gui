# `ui-verify/checks/ui_scale`

`ui_scale_resizes_the_chrome` — the UI-scale preference reaches the window,
and nothing falls off the edge when it does.

# The defect this detects

Three of them, and they need one run between them because each is invisible
to the tests that cover the other two.

1. **The preference is read and never applied.** `app::prefs` parses
   `ui_scale`, `dialogs::settings::appearance` edits it, and `app::frame`
   hands it to `Context::set_zoom_factor`. Every one of those has unit
   tests and **not one of them can see the chain joined**: the parser's
   tests build strings, the control's tests build a `Prefs`, and the frame
   hook has no test at all because it needs a live `egui::Context` inside a
   real window. It is the same shape as a flag defaulting off in front
   of a correct decision function: every part right, the **join**
   unobserved, and no unit test positioned to see it.

2. **Something is clipped at a large scale.** This is the one that needs
   pixels and the reason this check exists at all. `MODES_AND_PANELS.md`
   states the rule twice over: *layout and clipping defects have exactly
   one oracle, a rendered screenshot.* A control laid out below the bottom
   of its own pane, and a two-row ribbon one gap short of fitting, are both
   states this shell can reach with **every unit test green**. Doubling
   every control in the window is the cheapest possible search for the
   next one.

3. **The scale is applied to the page as well as the chrome.** It must not
   be. `set_zoom_factor` moves `pixels_per_point`, which the canvas already
   reads for its raster scale — so the page re-rasterises at the new device
   density and stays **the same size relative to the window**. If the
   document instead grew with the UI, the setting would be a second page
   zoom, which is precisely what its own copy promises it is not: *"It
   never changes the page or the file — only the window around them."*

# Why it drives the FILE and not the slider

The obvious script is: open Settings, drag the slider, capture. This check
does not, for two reasons and the second is the important one.

The shallow reason is reach — the *pdfcer* group holding `file.settings` is
the last group on the File tab, and at the shipped 1100 px window width it
falls into the ribbon overflow, so driving it means opening a popup first.

The real reason is that **the file is the path an operator's setting
actually takes**. A slider drag exercises the draft's live preview, which
is one frame of one session. Writing `preferences.txt` and launching
exercises the whole chain the operator depends on every morning: parse,
normalise, adopt, apply, lay out. If that path is broken, a slider that
works is worthless — the operator sets their scale, closes pdfcer, and
opens it the next day at 100 %.

So this check writes the preference and launches twice, once at 1.0 and
once at [`LARGE`]. The slider's live preview is a separate property and is
deliberately not covered here; it wants its own check and its own dialog
step.

# The oracle: a control's SHARE of the window, not its size in points

This is the subtle part and the first version of this check got it wrong,
so the reasoning is written out rather than assumed.

The intuitive oracle is *"the ribbon tab is 28.3 pt at 1.0, so it must be
~51 pt at 1.8"*. **It is not, and a build that made it so would be
broken.** `Context::set_zoom_factor` does not enlarge point-sized things in
points — it changes how many *pixels* a point is worth:

```text
                      base (1.0)        large (1.8)
  pixels per point    1.0               1.8
  window, pixels      1100 x 800        1100 x 800   (unchanged — the OS
                                                      window did not move)
  window, POINTS      1100 x 800        611 x 444    (shrinks by 1.8)
  ribbon tab, points  28.3 x 24.0       28.3 x 24.0  (UNCHANGED)
  ribbon tab, pixels  28.3 x 24.0       51.0 x 43.2  (grows by 1.8)
  tab as % of window  2.6 %             4.6 %        (grows by 1.8)
```

A control specified as 24 pt tall stays 24 pt tall; that is what "specified
in points" means. What changes is the **canvas it is laid out on**, and
therefore its share of it. So the measurement that tracks what the operator
actually sees — *things got bigger* — is the region's **fraction of the
client area**, and that is what [`drive`] asserts.

The absolute point sizes are still printed beside the fractions, because
when this fails they are what says which of the two regimes the build is
in: unchanged points with an unchanged fraction means the zoom factor never
moved, and *changed* points would mean something is scaling the widgets
themselves, which is a different defect wearing the same symptom.

A capture is still taken at both scales, because assertion 2 is about
**where** things landed and the screenshot is the artefact a human reads
when this fails.

# What "clipped" means here, precisely

A declared region whose rect is not contained by the client area. That is a
narrow test and deliberately so: it catches the redaction-apply defect
exactly (a control declared at `y = 801.7` in a body ending at `y = 770.0`)
and it cannot produce a false positive from a control that is merely
*tight*, because a rect either fits or does not.

It does **not** catch a control clipped by a scroll area inside a panel,
because such a control is legitimately outside its viewport and the
application says so by declaring it there. Naming that limit rather than
widening the test: a check that flagged every scrolled-out row would fire
on every run and be switched off within a week.

## Item notes

### `const LARGE`

**1.8, not the 2.0 maximum.** Two reasons, and the first is about what the
check can conclude:

* At exactly the maximum, a build that silently clamped to *some* ceiling
  would produce the same measurement as one that honoured the request. 1.8
  is inside the range, so the assertion below distinguishes "applied" from
  "clamped to whatever it felt like".
* 1.8 is enough to break a layout that is going to break. The shipped
  window is 1100 × 800 and the ribbon band is ~103 pt; at 1.8 that is
  ~186 pt of an 800 pt window, which is where a two-row group with a
  caption starts competing for space with the canvas.

### `const RATIO_TOLERANCE`

Generous, and it has to be: a laid-out control's size is not a pure
multiple of the scale. Text is measured in whole pixels at the device
density, padding is rounded, and `egui` snaps some rects to the pixel grid
— so a 30.3 pt tab at 1.8 lands near 54.5 pt rather than at exactly
54.54 pt.

12 % is wide enough that no rounding regime trips it and narrow enough that
**it cannot be satisfied by the wrong answer**: the two failures worth
catching are "no scaling at all" (ratio 1.0, which is 44 % away) and
"scaled by the wrong factor" (the nearest plausible wrong factor is the
device pixel ratio, 1.0 or 2.0 on this hardware, both far outside).

### `const SCALED_REGIONS`

Ribbon chrome specifically, because it is drawn by `egui-shell` from the
theme's own metrics and is therefore the surface a scale change is
*supposed* to move. The canvas is deliberately excluded — assertion 3 is
that the page does **not** grow, and mixing the two into one list would
make a pass ambiguous.

### `struct Measured`

Returns the declared regions and the client area, which is everything the
assertions need. Kept separate from [`drive`] so the two launches cannot
drift apart — the failure that would produce is a comparison between two
runs configured differently, which would look exactly like a scaling defect.
