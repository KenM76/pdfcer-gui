# `dialogs::print::spooler::device` — what a printer IS, and how it is configured

## The seam this file is on the other side of

[`super`] is the adapter for **the job**: which pages, at what size, in
what order, placed where on a sheet. This file is the adapter for **the
device**: which printers exist, what each one can do, which sheets it
offers, and what its driver currently holds.


## What is still true of both halves

**This module and its parent are the only files in the crate that name
`pdfcer_print`.** Everything else in [`crate::dialogs::print`] — the three
tabs, the preview, the footer — is written against the mirrored types
here. That is what confined "make printing work" to one module in
August 2026, and it is worth keeping: see [`super`]'s header for the full
reasoning, including why no arithmetic is ever mirrored.

## The rule that governs every capability query here

**A query that answers "I do not know" is not a query that answered
"no".** `pdfcer-print` was explicit about this when it declined this
project's proposal to gate the tray control on a `bool`:

> *"`DC_BINS` on Microsoft Print to PDF returns nothing at all, while
> that same device's `dmDefaultSource` is already `DMBIN_FORMSOURCE` — it
> picks by form by default. A bool would have collapsed 'the driver said
> nothing' into 'no', and told the operator a device cannot do the thing
> it was already doing."*

So [`FormSourceSupport`] has three states and not two, and the shell's
reading of them is the inverse of R83's usual direction: **`NotListed`
and `Unknown` still get the control**, with the disclosure. R83 forbids
offering an affordance the hardware *cannot* honour; it does not forbid
offering one the driver merely declined to advertise.

Contrast [`DeviceFeatures::supports_duplex`], which is a genuine
capability answer and *is* gated: `DC_DUPLEX` returning zero means the
device is simplex, and no setting in the dialog will change that.
