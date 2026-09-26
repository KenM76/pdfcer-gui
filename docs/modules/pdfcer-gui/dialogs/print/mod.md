# `dialogs::print` — the print flow: the dialog, its preview, and the spool call

## Printing is the one action in pdfcer with no undo

Carried across from the old shell verbatim, because it is the sentence
every other decision in this directory follows from:

> Everything else this application does can be reverted, closed without
> saving, or corrected before a save. A print marks paper, occupies a
> device somebody else may share, and cannot be taken back. That single
> fact decides most of what is in this file: why the dialog is its own
> stationary surface rather than a dock pane, why the preview shows the
> printable RECTANGLE and not just the sheet, and why no keyboard chord
> spools. (Enter **does** commit, from the affirmative button — see below.)

## The dialog IS the confirmation. There is no second gate.

> The CLI defaults to a dry run and requires `--send`. That is right for a
> scriptable tool whose operator is not watching, and wrong here: a GUI
> whose premise is that the operator is looking at the settings does not
> also need them to confirm the settings.
>
> What replaces a second gate is disclosure with teeth — the clip count is
> in the BUTTON'S OWN LABEL, so the uncertainty is stated in the
> disclosure rather than implied by a confirm step existing (rule 4).

Two things follow from it, and the first is not what a reader expects:

- **Enter presses Print**, through [`crate::dialogs::host::Host::footer`],
  which treats the affirmative button as the Enter target unless a text
  field has focus. That is `ui-conventions/dialogs.md` G4 and it is
  deliberate — a print dialog that ignored Enter would be the only one on
  the machine that did. What carries the weight instead of a second gate is
  **disclosure on the button itself**: the affirmative control is painted in
  the theme `accent` (never the translucent selection fill — see
  `Host::buttons`) so it cannot read as disabled, and when the job clips it
  says so *in its own label*. An operator pressing Enter out of habit has
  already been shown, in the control that key will press, what the job will
  do.
- **No keyboard chord commits.** A chord may *open* this dialog; nothing
  spools a job. Reversible actions get chords; the irreversible one does
  not.

## Where this module splits, and why there

The salvaged source was one 2,022-line file, over the project's 1,500-line
ceiling (`R2`). The seam is not a line count — it is the three genuinely
separable questions the file answers:

| file | question | can be wrong how |
|---|---|---|
| `mod.rs` (this) | *what is the job, and how does the surface hold together?* | wiring, layout, the commit path |
| [`preview`] | *what will the sheet look like?* | **arithmetic** — fit, anchor, raster scale, indexing |
| [`tabs`] | *what are the operator's answers?* | **arithmetic** — range parsing |
| [`spooler`] | *what does `pdfcer-print` look like from here?* | the port, and nothing else |

The split follows testability, exactly as the crate root's own table does:
everything that could be *silently* wrong — an anchor term that drifts, a
range that recovers from a typo, a plan list indexed by the wrong number —
is in a module with unit tests around it, and what is left in this file is
wiring that can be reviewed by reading.

## Reaching a device

`pdfcer-print` is a dependency of this crate and [`spooler`] is the only
module that names it. Printers are enumerated, the device's own properties
sheet opens over this window, and the commit button spools a real job
through `StartDoc`. When there is no device to reach, the dialog says which
of the two things happened — the spooler could not be reached at all, or it
answered and this machine has no printers installed — and returns before the
footer is drawn, so there is no commit button rather than a greyed one (R9).
`text::print`'s header owns the distinction between those two sentences.

## What is deliberately absent: imposition

N-up, booklet and poster are `cli [x] · gui [ ]` in `FEATURES.md`, blocked
on sheet composition being lifted into `pdfcer-print` so both shells share
one implementation. An imposition control here would be an affordance for
something that cannot happen. See [`spooler`]'s header for what the tab
owes when it lands — starting with the mutual-exclusion guard, which is
**CLI-local today** and which a GUI must re-implement rather than inherit.

## Item notes

### `mod autopaper`

Split out of [`preview`] rather than added to it, at the seam between
*"what do these pixels say"* and *"how is the preview painted"*. The first
is pure arithmetic over a byte slice and is fully testable with no GUI at
all; the second needs an `egui::Ui`. Keeping them in one file would have
put a page of pixel-threshold reasoning in the middle of a painting
routine and pushed `preview.rs` toward R2's 1500-line ceiling.
**Which sheet the pages want** — operator request O167, 2026-09-10. Pure
arithmetic over the driver's form list and the job's rotated page extents,
separated from the dialog because it is the half a unit test can drive.

### `mod remembered`

Two functions, and they are here rather than beside [`commit`] (which is
what calls them) because the judgement they encode is a different
subject: *which* of this window's twenty controls describe the operator
rather than the document. See [`crate::app::prefs::printing`] for the
rule and the argument for every inclusion and every omission.

### `fn host`

Everything the job depends on is recomputed here, every frame, from
the operator's current answers — there is no cached plan that could
describe a different job from the one the preview is showing. That is
affordable because planning is arithmetic over a page-size list; the
two things that are *not* affordable per frame (enumerating printers,
asking a driver about duplex) are the two that are not done here.
This dialog's window: what it is called, how big it opens, and the
floor it may not be dragged below.

Built fresh each frame and owning nothing — the position the operator
drags it to lives in `egui::Memory`, keyed on the id string. See
[`crate::dialogs::host`]'s header for why that is what let the other
thirteen dialogs be converted in one line each.

The size argument is unchanged and carried verbatim from the
`egui::Window` this replaced. The floor is not a preference:
`resizable` with no minimum lets the operator drag the window down to a
title bar and a scrollbar, which is a state with no way back except
closing it — and closing this dialog discards the job they were
configuring. 520 x 380 is the smallest size at which one column and
both scrollbars are still usable.

### `fn refresh_device`

See [`Self::features_for`] for the defect this closes: the old shell
read capabilities only for the *initially* selected device and never
again, while letting the operator change printer, so a duplex control
could survive onto a simplex device and produce a job that came out
single-sided with nothing to say why.

Called once per **change of selection**, never per frame: three of the
four things it does open a device context, and doing that sixty times
a second while a dialog sits open would be rude to a service other
applications share.

# Two things are DROPPED here, for two different reasons

**The configuration**, because a `DEVMODE`'s private tail is one
driver's private format and handing it to another device is undefined
rather than degraded. The engine refuses it by name; dropping it here
means the refusal is never reached.

**The paper choice**, and this one is subtler and worth stating in
full. [`PaperChoice::Form`] holds a `dmPaperSize` integer, and those
are only standard up to a point: the low ids are Win32 constants
(`DMPAPER_LETTER` is 1, `DMPAPER_A3` is 8), but everything a vendor
defines lives above `DMPAPER_USER` and means whatever that one driver
says. Carrying `Form(257)` from an EPSON to a plotter would silently
request a different sheet under the same number — no error, no
mismatch, just the wrong paper. It resets to
[`PaperChoice::DeviceDefault`], which is the only value that means the
same thing on every device.

A failed read of either falls back to the safe direction: no features
(so no duplex control) and no forms (so no paper list).

### `fn effective_device`

Identical to [`Self::device`] in every respect but one: a `paper` of
[`PaperChoice::AutoFromPages`] is replaced by whatever
[`Self::auto_paper`] resolved it to this frame — a concrete
[`PaperChoice::Form`] when a sheet was chosen, or
[`PaperChoice::DeviceDefault`] when there was no basis for one.

# Why the resolution is a function and not an assignment

Because [`Self::device`] is what the **operator** chose and it must
survive. Collapsing `AutoFromPages` into `Form(9)` in place would mean
the combo stopped reading *"Match the pages in this document"* the
instant it was chosen: the operator would pick auto, watch the control
jump to "A4", and have no way to tell whether pdfcer had matched the
pages or simply ignored them. Worse, opening a second document in the
same session would then print on the first document's sheet under a
label that named no policy at all.

So the choice is stored once and resolved on every read. The cost is a
struct copy per frame; the property bought is that *the control always
says what the operator asked for and the job always uses what pdfcer
worked out*, and neither can drift into the other.

# Callers, and why there must be no others

Three: [`Self::show`] (which plans with it), [`Self::commit`] (which
spools with it), and [`Self::trace_plan`] (which reports it). Any
fourth site reading `self.device` for a paper value is a site that can
hand `AutoFromPages` to something that has no meaning for it.

### `fn open_properties`

Runs **after** the window's closure has returned — see
[`Self::properties_requested`] for why a nested modal message loop
cannot be started from inside an egui layout pass.

# What happens to the three outcomes

| outcome | effect |
|---|---|
| accepted | the configuration is stored, and the paper combo adopts whatever sheet it names |
| cancelled | **nothing at all** — no message, no state change. The operator declined |
| refused | [`Self::properties_error`] is set and shown; whatever configuration was already held survives |

# Why the paper combo follows the driver's dialog

Because otherwise two surfaces describe the same job differently. An
operator who picks A3 in the driver's dialog and returns to a combo
still reading *"From the printer's own settings"* has been told
something false by a control they can see, about a setting they just
changed. Adopting the id makes the combo a report of the truth rather
than a competing claim — and because the engine amends rather than
replaces, asserting the same value changes nothing about the job.

A configuration naming a **custom** sheet has no id to adopt, so the
combo stays on `DeviceDefault` — which is correct: `DeviceDefault`
asserts no paper, so the configuration's own custom sheet stands. The
disclosure line reports it rather than the combo.

### `fn job_spec`

The custom scale is materialised here rather than stored live, so the
percentage spinner can be edited while some other sizing mode is
selected without the mode changing under the operator's hand.

### `fn options_column`

# The printer selector is OUTSIDE the tabs, always visible

It is not a setting like the others — it is the thing that decides
which of the others exist. [`Self::features`] is read from the selected
device and gates the duplex radios (R83), so a tab that could hide the
printer name would let the operator change device, watch controls
appear and disappear, and have no way to see what they had changed it
to without going looking.

# The tab strip reuses the ribbon's widget, deliberately

`egui::Button::selectable` plus a bold weight on the active one is what
the ribbon already draws for its own tabs. Inventing a different tab
affordance for the second tabbed surface in the application would teach
the operator that "tab" looks like two different things. The bold
weight is not decoration: R84 forbids state carried by colour alone.

### `const US_LETTER_PORTRAIT_PT`

Mirrors `pdfcer_print::US_LETTER_PORTRAIT_PT`. Such a job spools nothing, so
the value never reaches paper; it exists so the commit path carries no
`Option` for a case that cannot print.

### `mod spooler`

The alternative was a second copy of five enums on the other side of the
boundary, and this project's standing lesson about mirrored enums is that
they drift: a variant added here would round-trip through the preferences
file as somebody else's default, silently, with every test still green.
Widening this module is the cheaper of the two mistakes and the only one
the compiler can police.

### `struct PrintDialog`

# Why a dialog struct rather than a dock panel

Printing is a single transaction with a start and an end, not something an
operator dips in and out of while working — which is what a dock pane is
for. It is also *modal in spirit and not in mechanism*: nothing blocks the
rest of the shell, but the surface is screen-anchored and stationary
rather than positioned relative to the page, because controls whose
position is derived from the page move on every zoom and scroll.

### `fn open`

# Two things happen here and nowhere else

1. **The spooler is enumerated, once, on a deliberate click.**
   Enumerating printers can block briefly on a network spooler, so it
   must not happen inside the frame loop.
2. **The preview opens on the page the operator is looking at.** Not
   page 1: the commonest print is "this sheet", and opening the preview
   somewhere else makes the operator step back to where they already
   were.

The guard against re-opening over a half-configured job is
[`crate::dialogs::DialogsState::open_print`]'s, because it is the one
place that can see whether a dialog already exists.


*"the printer dialogue box needs to remember our last settings."* Every
field below that reads `remembered` was a literal until that day, so an
operator who prints every drawing landscape, two-sided, on the plotter,
at 600 dpi re-answered all four questions on every single print.

What is in that value and what is deliberately not is
[`crate::app::prefs::PrintPrefs`]'s subject, argued at length in its own
header. The rule, in one line: **a setting is remembered only if it
would still be the right answer for a different document.** Which is
why the range, the preview's sheet, its zoom and the active tab are
still literals here and always will be.
