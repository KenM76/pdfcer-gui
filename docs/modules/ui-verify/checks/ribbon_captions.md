# `ui-verify/checks/ribbon_captions`

`ribbon_group_captions_legible` — every ribbon group has a caption you can
read.

# The defect this detects

Two of them, sharing one region and one measurement, because they are
indistinguishable until you look:


[`crate::checks::legibility`] reports them separately, because they have
different fixes: a uniform region means the caption is not being drawn and
the theme is a red herring. It reports a third case separately too — a
caption the ribbon declares at a rect outside the window — because that is
a layout defect and neither of the first two.

# Why the ribbon in particular

`RIBBON_IA.md` specifies seven tabs, each with several captioned groups.
The caption is what makes the ribbon navigable — it is the only text that
says what the row of icons beneath it is *for*. A ribbon whose captions
are invisible degrades to an undifferentiated field of icons, which is the
state the audit found parts of it in.

It is also the surface most likely to regress silently, because captions
are drawn by shared chrome code: one widget-style change alters every
caption on all seven tabs at once, and nothing about the diff says so.

# How it works, as of S2

1. Launch the binary with its diagnostic switch set, opening the fixture if
   one was given.
2. Read the trace and collect every region the application **declared**
   with a `ui-rect` event.
3. Keep the ones whose names follow the ribbon-caption convention
   ([`is_ribbon_caption`]).
4. Capture the window and measure each of them against the WCAG 2.1 AA
   large-text floor of 3:1.

There is no tab iteration and no calibration step, and both absences are
the point. The application reports where its captions are; the harness
measures what it reports. A ribbon that gains an eighth tab, collapses to
an icon rail (`MODES_AND_PANELS.md` puts that on the roadmap, and it moves
every caption in the window) or reflows at a narrower width changes what it
declares, and this check follows it with no edit here.

# What it reports today, and why that is not a fudge

**The ribbon does not exist yet.** `crates/egui-shell/src/ribbon/` is being
written as this is; nothing declares a caption region. So step 3 finds
nothing and the check reports SKIPPED with a reason that says so *in those
terms* — "the application declared no ribbon group caption regions, and
here are the three it did declare".

That specific wording is load-bearing. The three verdicts available here
are:

* **PASS** would be a lie: nothing was measured.
* **FAIL** would be worse than a lie: it would file a defect against
  captions nobody has written, and this codebase has already paid for one
  filed-then-retracted defect (see [`crate::coords`]). A check that fails
  on unwritten code teaches its readers to ignore it, after which its true
  reports get ignored too.
* **SKIP naming the missing subsystem** is the honest report, and it is
  also *actionable*: it tells the ribbon's author exactly which names to
  publish.

# The one thing that must remain true for this to start working by itself

[`is_ribbon_caption`] must recognise the names the ribbon publishes. See
its documentation for what it matches and why the rule is a pair of words
rather than an exact spelling.

## Item notes

### `fn ribbon_regions`

Split out of [`assess`] so that the claim *"this check starts asserting on
its own the day the ribbon declares its captions"* is **testable without a
ribbon**. A test can hand this function a trace containing the lines the
ribbon will emit and observe that the whole chain — parse, match, convert
to capture pixels, resolve — produces a trace-sourced plan. If that claim
were only exercised by running the real ribbon, it would be untested for
exactly as long as it matters.

`frame` is the live window's measured geometry; it supplies the DPI scale
that turns the application's logical rects into pixels of the capture. See
[`WindowFrame::logical_to_capture_pixels`] for why no origin term appears.
