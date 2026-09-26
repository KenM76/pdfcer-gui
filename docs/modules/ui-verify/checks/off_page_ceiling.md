# `ui-verify/checks/off_page_ceiling`

`the_off_page_halo_never_costs_the_operator_his_zoom` — **O218/O221's
mechanism, driven.**

# The report

> *"Sometimes when I zoom in I still get the error 'This page could not be
> drawn. This zoom is further in than pdfcer can rasterize…' instead of the
> rasterizer just stopping at the last zoom level that it accomplished. …
> Zooming capability seems to be affected by the number of pdfs I have open
> … I think, but I could be wrong and it could be a bit random."*

It was not the document count and it was not random. It was **off-page
content**, which is common in CAD exports and rare elsewhere, and which the
shell displays by default in Review and Edit and not in Read.

# A region is not automatically small

Two different rectangles reach the render path wearing one type, and they
behave **oppositely** as the operator zooms:

| region | device size as zoom rises |
|---|---|
| the visible-rect tier's box | **constant** — it is a multiple of the WINDOW |
| the off-page halo box | **grows with the zoom**, exactly as the sheet does |

`OpenDoc::raster_order_fillable` treated *any* present region as fillable,
which is true of the first row and false of the second. The halo box is by
construction **larger** than the sheet, so it crosses the rasterizer's
`MAX_PIXMAP_EDGE` **first** — on this fixture at about half the zoom the
sheet does. Add the one-frame skew (the canvas picks the region at frame
N's scale; the settle step orders it at frame N's *new* scale) and a halo
validated at one rung is ordered at the next, comes back `BadRasterSize`,
and `absorb_render` turns that refusal into the document's **permanent**
zoom ceiling.

Measured on `fixtures/off-page-object.pdf`, same recipe, two builds:

| build | refusal | ceiling learned | where the zoom stopped |
|---|---|---|---|
| before | `px=23040x12800 scale=64.0 region=1` | `scale=48.0` | **4,800 %** |
| after | none | none | **1,677,721,600 %** |

# Why this check is headless, and why it is in Edit mode

The window is placed off the desktop and no OS input is ever sent to it, so
it can run while the operator is working — the zoom verbs arrive through
the scripted-keystroke seam instead. `PDFCER_DIAG_INVOKE=mode.edit` is
**load-bearing, not decoration**: `OffPagePrefs::default_for_mode` turns
off-page display OFF in Read, and a Read-mode run never enters the halo
tier at all. A whole eight-rung ladder was once measured in Read mode and
recorded a real, precise zero — of the wrong variable.

# The controls, and why they carry no engine constant

`tools/ui-verify` has one dependency and cannot import
`pdfcer_render::MAX_PIXMAP_EDGE`; a harness constant *naming* an engine
constant would be a copy that decays the day the engine's moves. So both
controls are **relational**, read from the application's own trace:

1. **The halo box is bigger than the sheet.** Its `box=` against the
   `crop=` the same run publishes. That is the whole premise — if the halo
   were the smaller rectangle there would be no defect to check.
2. **The application abandoned the halo tier while off-page display was
   still on.** A `tier=halo` line followed by a `tier=whole` one is the
   application *saying* it crossed the halo's own wall. That is the point
   the pre-fix build died at, so a run that never reaches it has not
   entered the window where the defect lives and this check would be inert.

Both are reported as harness findings rather than as failures: a run that
never got into the defect's window has learned nothing about the defect.

# The three failures, in causal order

1. **An order the shell could not fill** — any `bad-raster-size`. The root
   cause; the message names whether it was a region order or a whole-page
   one, because those have different owners.
2. **A permanent ceiling learned** — any `raster-ceiling-learned`. The
   effect: the document is pinned for as long as it stays open.
3. **The zoom stopped climbing the moment the halo stopped fitting** — the
   operator's own symptom. The last zoom of the run must be strictly above
   the zoom the application was at when it abandoned the halo tier. Before
   the fix those two figures were the *same number*: fourteen further
   chords changed nothing.

# Every way this reports SKIP

No binary, no diagnostic channel, the seam not delivering its last chord,
no document opened, off-page display not coming up, no halo tier entered,
the halo box not exceeding the sheet, or the climb never reaching the wall.
**Not** a refusal, **not** a learned ceiling and **not** a stalled zoom —
those three are the failures this check exists for.

# ⚠ What has been falsified, and the one thing that has not

A red run proves the check; it does not prove the check's *parts*, because
the first arm to fire is the only one that ran. Each was therefore driven on
its own, against real evidence rather than a plant:

| arm | how it was made to fire | what it said |
|---|---|---|
| refusal | the pre-fix binary | `px=23040x12800 scale=64.0 region=1` |
| ceiling | the pre-fix binary, this arm moved first | `scale=48.0 zoom=64.00 to=48.00` |
| stalled zoom | the pre-fix binary, this arm moved first | abandoned at 4,800 %, ended at 4,800 %, 0 further figures |
| off-page display off | `mode.read` instead of `mode.edit` | no `offpage=on` on any frame |
| no halo tier | `four-pages.pdf`, which has nothing off its sheet | `[tier=whole offpage=on, tier=whole offpage=on]` |
| wall never crossed | four rungs instead of twenty-four | scales ordered `[3.32 3.31 4 6 8 16]` |

**Not falsified: the halo-bigger-than-the-sheet control.** No run reaches
it, because `render::halo::region` only returns a union once the content's
reach passes `OVERHANG_TOLERANCE_PTS` beyond the crop box, so a halo that is
not the bigger rectangle is not a state the application can be driven into.
It is a guard against that invariant changing, not an instrument arm, and it
is recorded here as one so a later reader does not quote it as evidence.

## Item notes

### `const FIXTURE`

⚠ Pinned, and `--pdf` is ignored. A document with no off-page content
never enters the halo tier, so this check's entire subject would be absent
and its controls would turn every run into a SKIP.

### `const CLIMB`

The fixture's halo wall sits about four rungs above its fit zoom and its
sheet's wall about one rung above that, so a dozen would prove the fix.
Twice that is deliberate: the rungs past the wall are the ones that were
silently doing nothing, and a check that turned round just after the
boundary would pass on a build that recovered for one rung and stalled
again.

### `const SEED`

⚠ **A one-shot statement about start-up, not about the run.** It is emitted
while the document is being opened, which is BEFORE a command from
[`INVOKE_ENV`] has fired — so on a fresh profile it says `mode=read on=false`
on a run that then enters Edit and displays off-page content perfectly well.
Read as the current state it makes a working run look inert, which is
exactly what it did here once. The authoritative reading is the `offpage=`
field on the canvas' own per-frame [`HALO`] line, because that is the one
the canvas acted on. This constant survives only to explain a genuinely
off run.

### `const RUNG_DEADLINE`

Generous on purpose. The seam paces its chords twenty frames apart and the
last rungs of this ladder are drawn at magnifications where a frame is not
free; the whole run measured about forty seconds on an idle machine.
Exceeding this is reported as a finding rather than as a hang.

### `fn preconditions`

Both are reported as harness findings. This project has twice produced
confident, detailed and entirely wrong defect reports out of a reading
taken on a run where the subject was never present.

### `fn wait_for_rung`

Not [`Session::settle`] as the primary wait: that polls the frame counter,
and this application stops drawing the moment the seam stops asking it to.
The thing being waited for is a trace line, so the trace line is what is
waited for.
