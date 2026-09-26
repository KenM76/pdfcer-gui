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
