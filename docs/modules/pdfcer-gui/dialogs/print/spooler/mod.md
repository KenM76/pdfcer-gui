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

### `enum Unavailable`

# Two variants, mapping onto two of the three sentences

[`crate::text::print`]'s header sets out three ways to have no printer,
deliberately said three ways. Two of them are failures and live here; the
third is not a failure at all and therefore has no variant:

| condition | represented by | sentence |
|---|---|---|
| pdfcer could not ask this system about printers **at all** | [`Unavailable::Spooler`] | [`crate::text::print::spooler_unavailable`] |
| this *particular* device would not describe itself | [`Unavailable::Device`] | [`crate::text::print::device_unavailable`] |
| the spooler answered and reported none installed | `Ok(vec![])` — **not an error** | [`crate::text::print::no_printers`] |

The third row is the one worth stating explicitly, because collapsing it
into the first is the exact defect `pdfcer-print` names: a machine with no
printers installed is a *normal machine*, and reporting that as a failure
sends an operator looking for a fault that does not exist. The engine
returns an empty `Vec` there and this type has nowhere to put one, which
is the type system holding the distinction rather than a convention.

# Why a `String` rather than the engine's `PrintError`

`PrintError` is `Debug + Clone` and neither `Copy` nor `Eq`, and this
value is stored in dialog state, compared in tests, and copied into trace
lines. Carrying the engine's own `Display` output — which is written as
operator-facing prose, complete with the remedy — keeps every one of those
cheap while losing nothing: nothing in the shell branches on *which*
`PrintError` it was, only on which of the two rows above applies, and that
is what the variant already encodes.

### `enum ScaleMode`

**Four modes, not three**, and the fourth is not a rounding error:
`pdfcer-print` keeps `Fit` and `ShrinkOversized` apart because collapsing
them — *"the natural simplification"* — *"silently blows a business card
up to A4"* — `ScaleMode`'s own doc. Fit scales in both directions; Shrink only
ever reduces.

Maps to `pdfcer_print::ScaleMode`, variant for variant.

### `enum Orientation`

`Auto` is resolved **per page** from the page's own aspect, which is what
keeps a document mixing portrait text with a landscape drawing upright
throughout.

### `enum Duplex`

**Driver-gated, never simulated.** pdfcer will not fake duplex by
reordering pages and asking the operator to reinsert the stack: *"that is
a workflow with a documented mis-assembly failure mode, and offering it as
though it were duplex would be claiming a capability the hardware does not
have."* [`DeviceFeatures::supports_duplex`] is what the dialog consults
before drawing the control at all.

### `struct JobSpec`

Maps to `pdfcer_print::JobSpec`, field for field. **Kept separate from
[`DeviceSettings`]** for the engine's own reason: everything here is
arithmetic pdfcer performs and can be exact about, and everything there is
a *request to the driver* which the driver may quietly decline. Presenting
both as though pdfcer controlled them is what makes a job silently come out
single-sided with nothing to say so.

### `enum PaperChoice`

# Why choosing paper is a REQUEST and not a setting

`pdfcer-print` reported, while building this: **two drivers were found
silently ignoring a paper request.** The `DEVMODE` is handed over with
`DM_PAPERSIZE` asserted, the driver is free to do as it likes with it, and
nothing comes back to say it declined. There is no acknowledgement in the
Win32 API to read and none to invent.

That is a fact pdfcer cannot verify and the operator cannot see, which puts
it squarely under rule 4 — *fuzzy, never sneaky*. The disclosure is
[`crate::text::print::paper_is_a_request`], off-canvas, in words, beside
the control that makes the choice. It is **not** a warning icon on the
preview and **not** a differently-styled sheet outline: the preview draws
the sheet the job was planned for, exactly as it would draw any other, and
pdfcer's uncertainty about the driver is reported in text next to it.

# Why there is no `Custom` variant here when the engine has one

Because there is no surface to type a size into. The engine's
`PaperSelection::Custom` takes a sheet in tenths of a millimetre and is
reachable through the driver's own properties dialog — an operator who
needs a 900 mm roll length sets it there, and
[`super::device::ConfigSummary::custom_paper_pt`] is read back so the
dialog can say what it holds. Mirroring a variant this shell cannot
construct would be a value with no producer; recorded in `NO_SURFACE.md`
rather than half-built here.

### `struct DeviceGeometry`

Maps to `pdfcer_print::DeviceGeometry`.

# Turned, and that word is the whole defect this type prevents

`printer_caps` reports the device's *default* `DEVMODE`. On a
portrait-default printer that is a portrait printable area — so a
landscape job planned against it under-scales every page to about 77 % of
correct size, leaves a wide empty margin, and **reports no clip**, so
nothing says it happened. The engine removed the `From` impl that made
that mistake reachable, *"because a wrong answer that is one `.into()`
away will be reached again"*, leaving `DeviceGeometry::from_caps` as the
only route — and it cannot be called without stating the orientation and
the first page.

The port honours that by not exposing raw capabilities at all: [`plan`]
takes the orientation and the page sizes and hands back a geometry that
has already been turned, so the picture the preview draws and the paper
the job lands on are the same claim.

### `struct JobResolution`

Maps to `pdfcer_print::JobResolution`, plus one value flattened: the engine
exposes `uncapped_page_mb()` as a method, and it is carried here as a
field so no formula of the engine's is restated in this crate.

### `struct Job`

# Why one struct rather than three calls

The three come from the same three engine calls, in a fixed order, against
the same inputs — and getting the order wrong is exactly the orientation
defect described on [`DeviceGeometry`]. Returning them together means the
dialog cannot plan against one geometry and preview against another.

### `fn clipped`

Counted over the **whole job**, not the sheet on screen, because a
multi-page job's clip is usually on a sheet the operator is not
looking at. This one number reaches three surfaces — the preview
caption, the commit button's label, and the trace — and it is computed
in one place so they cannot disagree.

### `struct PageBitmap`

Maps to `pdfcer_print::PageBitmap`. **RGBA8, row-major, top row first** —
i.e. `pixmap.data().to_vec()` handed over unchanged, premultiplied, with
no conversion in between. The engine is explicit that this is the
contract; re-encoding it here would be a second colour convention of
exactly the kind [`crate::render::raster`]'s header exists to prevent.

### `struct SpoolReport`

**Never constructed in this build**, because [`spool`] cannot succeed
here. The `allow` is scoped to this one type and names the condition that
removes it, following the precedent `crate::viewer` sets for salvaged
items whose first consumer arrives in a later stage. Deleting the type
instead would mean the footer had no shape to render a success into, and
the day the manifest line lands the success path would be written from
scratch rather than reviewed.

### `enum SettingsSource`

# Why a shell must report this, and why it cannot be inferred

pdfcer writes at most four members of a `DEVMODE`. Everything else a device
does — media type, print quality, colour handling, stapling, output bin,
the entire vendor-private half — lives in the driver's own configuration,
which pdfcer carries through untouched **when it has one**.

[`Self::Synthesised`] is the case where it did not. The driver refused to
report its settings, so the job went out carrying only what pdfcer sets
itself and everything the driver held was lost. **The job still prints**,
which is exactly what makes this dangerous: the operator gets paper, and
the paper is wrong in ways — plain instead of glossy, draft instead of
best — that look like a printer problem rather than a pdfcer one.

It is not visible from the printed page, not visible from the dialog, and
not derivable from anything the shell knows before the call. The engine
reports it because it is the only party that can, and the shell says it
out loud for the same reason.

### `fn spool`

Fill with
`pdfcer_print::spool(printer, &bitmaps, DryRun::No, None, settings, first_page_pt)`.

# This is the one call in the application that consumes paper

`pdfcer-print`'s own header: *"Printing consumes paper, occupies a device
other people may share, and cannot be undone. Nothing in this crate starts
a job as a side effect of anything else: `spool` is the only function that
reaches `StartDoc`, and it is reached only from a control an operator
deliberately clicked."* The shell's half of that contract is that this
function is reached from **one** place — the commit button — and from no
keyboard chord, no dispatch arm and no frame-loop condition.

`first_page_pt` must come from `bitmaps.first()`, never from the
document's page 0: a reversed or range-filtered job sends a different page
first, and the driver picks its paper from whichever one it is handed.

# Errors

[`Unavailable::Spooler`] carrying whatever the spooler reported — passed
through to the operator verbatim by [`crate::text::print::failed`],
because a structured spooler error is the specific half of that sentence.
