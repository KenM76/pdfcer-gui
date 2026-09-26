# `dialogs::print::spooler` — the one module that knows `pdfcer-print` exists

## Read this first: this module is the ADAPTER, and it is now live

Everything else in [`crate::dialogs::print`] — the three tabs, the range
parser, the zoom anchor, the preview raster cache, the clip disclosure,
the commit button's label — is written against the types *in here*.
Nothing else in the dialog names a printing type, which is what confined
the whole "make printing work" change to this one module.

## Two files, and where the seam is


| file | subject | changes when |
|---|---|---|
| this one | **the job** — which pages, at what size, in what order, placed where | the layout arithmetic changes |
| [`device`] | **the device** — which printers exist, what each can do, which sheets it offers, what its driver holds | the way a device is interrogated changes |

[`device`]'s types are re-exported here, so every caller still says
`spooler::Printer` and `spooler::device_features`. See the re-export's own
note for why the seam is not pushed out to the call sites.

## The defect this file carried for the whole of v0.1.0, recorded

This header used to open with the sentence *"`pdfcer-print` is NOT a
dependency of this crate"* and then set out, in full, the two edits that
would make the build print: add the manifest line, then fill the four
holes below. **The manifest line landed and the four holes were never
filled.** `pdfcer-print` sat in `Cargo.toml` and in `Cargo.lock`, was
compiled and linked into every shipped binary, and no source file in the
crate contained the identifier `pdfcer_print` outside a doc comment. So
[`list_printers`] kept returning a refusal, the dialog kept rendering
*"This build cannot reach a print device"*, and the commit button was
never drawn.

The operator's report was *"the print dialogue didn't work"*, and it was
exactly right. Two things are worth carrying forward from it:

1. **The whole test suite was green throughout.** It had to be: the tests
   asserted that every hole *refuses*, which is the correct assertion for
   an unlinked build and becomes a lock on the defect the moment the
   manifest line lands. A test that pins a refusal must name the condition
   the refusal is conditional on, or it outlives its own premise. The
   replacement — `every_call_reaches_the_engine_rather_than_refusing` —
   asserts the opposite property, and it is written so that it cannot pass
   on a machine with no printers.
2. **A doc comment that describes future work is a liability with a shelf
   life.** This one was precise, correct, and read by nobody at the moment
   it became actionable. Where a plan like that is written down again it
   belongs in `GUI_ROADMAP.md`, where something sweeps it, and not only in
   the header of the file it happens to be about.

## What this build does now

[`list_printers`] enumerates the system's printers, [`device_features`]
reads one device's duplex and copy support, [`plan`] turns the geometry
and places every page, and [`spool`] hands the rendered sheets to the
Windows spooler. The commit button is drawn whenever there is a device and
a non-empty plan, and pressing it consumes paper.

Three ways to have no printer are still said three ways — `pdfcer-print`
refuses to collapse them, because non-Windows `list_printers` returns
`Err(Unsupported)` rather than an empty `Vec`, since *"reporting the same
value for 'this platform cannot enumerate printers at all' would collapse
two different facts into one and send a caller looking for hardware"*
— `list_printers`'s `#[cfg(not(windows))]` stub, in its own words.
[`Unavailable`] carries that distinction across the
port; see [`crate::text::print`]'s header for the three sentences it
feeds.

## Why mirror the types rather than re-export them

Two reasons, and the second is the one that matters.

1. The dialog needs a handful of values (a placement, a sheet size, a
   resolution verdict), not the whole crate. Mirroring the values it
   reads makes the seam small enough to hold in the head.
2. **No arithmetic is mirrored.** There is no `place_page` here, no
   `sequence()`, no `job_resolution`, no `plan_job` — those are
   `pdfcer-print`'s, they are tested there, and a second copy would be the
   failure this project already names about range parsers: *"two range
   parsers would eventually disagree about something like `5,1-2` … and an
   operator moving between the GUI and a script would have no way to know
   which one they were talking to."* The same is true, with paper at
   stake, of two placement calculations. So [`plan`] is a **hole**, not an
   implementation: it either calls the engine or it refuses.

## What is deliberately NOT here

**Imposition — n-up, booklet, poster.** `FEATURES.md` records it as
`core — · cli [x] · gui [ ]`, and the roadmap names the prerequisite:
*"needs the sheet composition extracted into `pdfcer-print` so both shells
share one implementation."* Until that lands, an imposition control in
this dialog would be an affordance for something that cannot happen, which
is precisely what the no-placeholders rule forbids. When it does land it
is **one new tab**, not a change to the three that exist: n-up, booklet
and poster remap the *job* rather than scale a page, and
`docs/core-api/03` §6.4 records that the mutual-exclusion guard between
them is **CLI-local** — *"`pdfcer-print` will not stop you. A new GUI shell
must re-implement this guard."* That guard is the first thing that tab
owes.

## Item notes

### `fn fmt`

It reaches a `PDFCER_DIAG` trace line and
[`crate::text::print::failed`]'s `detail` argument — which is the same
passing-through of a structured engine error that
[`crate::text::canvas_render_failed`] does, and for the same reason:
the engine's own sentence is the specific half.

### `fn to_engine_paper`

# What pdfcer asserts over a driver configuration, and what it leaves

The engine amends a `DEVMODE` rather than replacing it, and the members
named in [`DeviceSettings`] win over whatever the configuration held. So
it matters which members those are, and the list is not symmetrical:

| member | asserted | consequence |
|---|---|---|
| orientation | **always** | an orientation the operator set in the driver's own dialog is overridden by this dialog's radios. `Auto` resolves per page, which is nearly always what a mixed CAD set wants — but it *is* an override, and the properties disclosure says so |
| paper | only when [`PaperChoice::Form`] | `DeviceDefault` leaves the configuration's own sheet standing |
| duplex | only when non-default | asserting `DMDUP_SIMPLEX` unconditionally would silently cancel a driver's own duplex default — a defect the engine names in `apply`'s notes |
| tray | only when the checkbox is on | same reasoning |

Everything else — media type, quality, colour handling, stapling, output
bin, the whole driver-private tail — is carried through untouched.

### `fn to_engine_spec`

`pages` is cloned rather than moved because [`plan`] takes `&JobSpec` — the
dialog rebuilds its spec every frame from the operator's current answers
and keeps ownership of it, and a signature that consumed the spec would
force a clone at every call site instead of the one here.

### `fn to_engine_bitmap`

The pixel buffer is **cloned**, and that is a deliberate cost rather than
an oversight. [`spool`] takes `&[PageBitmap]` because the dialog's commit
path builds the whole set and then hands it over; taking the vector by
value would let the copy be avoided, but it would also mean the commit
path could not re-attempt a spool without re-rendering every page. On a
job large enough for the copy to matter, re-rendering is the far larger
cost.

### `fn every_call_reaches_the_engine_rather_than_refusing_unconditionally`

# What it asserts, and why the obvious assertion is the wrong one

The test this replaces asserted that every one of these four functions
**refused**. That was correct while `pdfcer-print` was not a dependency
and became a lock on the defect the moment it was — the manifest line
landed, the refusals stayed, the suite stayed green, and the operator
found out by opening the dialog.

So this asserts the opposite property, and the wording matters: it
asserts that a call **reaches the engine**, not that it succeeds.
Success is not available to assert. This suite runs on machines with
printers and machines without, on Windows and (at fold-in) not, and a
test that needed a device would be a test that got `#[ignore]`d and
then stopped being run at all.

The distinguishing evidence is the **failure text**. Every refusal
this module can now produce carries `PrintError`'s own `Display`,
which is written as operator-facing prose; the string the deleted code
produced was `"pdfcer-print is not linked into this build"`. That
sentence can no longer be constructed — the variant that held it does
not exist — so this test is really asserting that the type has the
shape a linked build gives it, which no amount of environment can
fake.

### `fn no_refusal_claims_the_engine_is_unlinked`

The narrowest possible statement of the regression, and the one that
would fail if somebody restored a `NotLinked`-shaped shortcut — for
instance by wrapping the four calls in a `cfg` that compiled them out
on a machine where `pdfcer-print` was inconvenient.

### `fn the_conversions_map_every_variant_to_its_own`

Not a change-detector: these are the functions through which an
operator's answer becomes paper, and a mapping that sent
`ShrinkOversized` where `Fit` was meant would enlarge a business card
to A4 — the exact collapse `pdfcer-print` keeps two variants apart to
prevent. A wrong arm here is silent until the sheet comes out.

### `fn the_clip_count_covers_the_whole_job`

Pinned because three surfaces read it — the preview caption, the
commit button's label and the trace — and the entire point of
computing it once is that the button cannot promise a different number
from the caption above it.
