# `ui-verify/checks/off_page_toggle`

`the_off_page_toggle_is_per_mode_and_remembered` — **the operator's switch
for off-sheet content, driven five times in one profile.**

# The report


> *"in our view ribbon area we need an option to show the stuff that is off
> page or not (and when not showing the stuff that is off page there
> shouldn't be a gap between pages where the stuff is, so it just goes back
> to looking before we added the view things that are off the page
> feature). by default, read doesn't show off page items, review and edit
> do show off page items. these settings can be changed by the user and
> their preference is remembered for each read review edit modes."*

Six requirements in one paragraph, and **four of them are about state that
survives a process**: the per-mode default, the operator's own answer, that
the answer is remembered, and that it is remembered *separately per mode*.
None of those can be observed in a single launch, and none of them can be
observed by a unit test at all — the store, the seed at document open, the
re-seed at mode change and the toggle's write are four different files, and
a test of any one of them is green while the chain is broken.

⇒ This check is a **ladder of five launches against one profile
directory**, which is the only arrangement in which "remembered" means
anything. `sandbox` gives each check a private copy of the binary and
therefore a private `userdata/`; the five launches below share it, in
order, and the later rungs read what the earlier ones wrote.

| # | mode | what is invoked | what must be true | which requirement |
|---|---|---|---|---|
| 1 | Read | nothing but the view | off-page content is **hidden** | the Read default |
| 2 | Read | `view.off_page` | it **appears** | the toggle works, from the command the ribbon item raises |
| 3 | Read | nothing | it is **still there** | the answer survived the process |
| 4 | Edit | `view.off_page` | it was **on before the click** and **off after** | the Edit default, and the toggle in a second mode |
| 5 | Read | nothing but the mode | it is **on** | Edit's answer did not touch Read's |

Rung 5 is the one that would be missing from a hand-written test of this
feature and is the one the operator asked for in as many words: *"remembered
for each read review edit modes"*. A single global flag passes rungs 1–4
and fails rung 5, and a single global flag is exactly what a first
implementation of this reaches for.

# Why each rung asserts BOTH a trace line and pixels

`off_page_visible`'s header carries the argument and it is unchanged here:
*layout and clipping defects have exactly one oracle, and it is a rendered
screenshot* (`D:/dev/rag/egui/`). The preference chain, though, is
invisible in a screenshot — a hidden object and a *missing* object look the
same. So each rung reads two independent things:

1. **The decision**, from the trace: `off-page-seed` (document open),
   `off-page-mode` (mode change) or `off-page-remembered` (the toggle),
   each naming the mode and the answer. This is what says the answer came
   from the *preference* rather than from luck.
2. **The consequence**, from the capture: ink, or no ink, at the centre of
   a square that lies entirely off the left edge of the sheet.

Neither alone is enough, and the failure they catch is different in each
direction. A trace-only check passes on a build that resolves the
preference correctly and then ignores it — which is the whole of the defect
for an operator. A pixel-only check cannot tell "Read hides it" from "the
renderer broke", and the report would send the next session into the
engine.

# The control that makes a negative rung mean anything

Rungs 1 and 4 assert an **absence**, and this suite's memory is explicit
that an absence assertion is worth exactly as much as its control: *"a
uniform failure at every rung of a sweep is about the probe."* A window
that never opened, a capture of the wrong monitor, a document that failed
to render — all of them produce a clean patch where the off-page square
should be, and all of them would read as a pass.

So **every** rung, positive and negative, also measures a patch at the
centre of the fixture's **on-page** square, and that patch must be ink. If
it is not, the check reports a harness finding and refuses to say anything
about the off-page patch — including that it was clean.

# The gap, which is the operator's OTHER sentence

> *"when not showing the stuff that is off page there shouldn't be a gap
> between pages where the stuff is, so it just goes back to looking before
> we added the view things that are off the page feature."*

That is a second consequence, not a restatement of the first, and it is
produced by a **different function**: `canvas::tier::decide` widens the
raster, `canvas::tier::overhang` widens the layout. A check that watched
only the ink would report green on a build that hid the off-sheet square
and left the band of grey standing exactly where he said it must not be.

So every rung also reads `canvas-pasteboard`, which carries the overhang
the layout was actually given, and asserts it: **exactly zero** with the
switch off, **strictly positive** with it on. Zero rather than *small*
because `overhang` returns early rather than multiplying a measured box by
nothing — so there is no rounding to tolerate, and a tolerance here would
be a place for a one-pixel band to hide.

The fixture is one page, so there is no *between-pages* gap to photograph;
what is measured is the reach that creates it, which is the same number for
one sheet as for fifty.

⚠ The overhang is in **screen** points, so it only compares across launches
while the zoom is held — which `view.zoom_actual` does on every rung. The
assertions below are against zero and against zero, which is why that is a
footnote rather than a hazard.

# Every way this reports SKIP

No binary, no diagnostic channel, no `canvas-viewport` region, not enough
grey on screen to reach x = −100 at 100 % zoom, or a capture that could not
be taken. **Not** any of the five rungs' own assertions: each of those is a
failure, and they are the reason this file exists.

## Item notes

### `const READ`

The same pair the sibling checks use and for the reason `off_page_visible`
measured: fit-page on a 200 × 200 fixture puts x = −100 outside the
viewport, the conversion refuses — correctly — and the check SKIPS, which
is not red.

`mode.read` is named EXPLICITLY on every Read rung rather than relied on
as the default, because the profile remembers the mode it was last in.
Rung 5 follows a rung that ended in Edit, and a rung that assumed Read
because the first launch was in Read would be reading Edit's answer while
reporting Read's.

### `fn the_ladder_covers_both_directions_and_all_three_writers`

The failure this catches is a later edit that trims the ladder into
something that still passes: five rungs that all expect `on=true`, or
five that all read `off-page-seed` and therefore never exercise the
mode change. Either would leave a green check over an untested half.
