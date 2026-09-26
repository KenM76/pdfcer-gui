# `dialogs::print::spooler` — the one module that knows `pdfcer-print` exists

## ★ Read this first: this module is the ADAPTER, and it is now live

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

## ★ The defect this file carried for the whole of v0.1.0, recorded

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
