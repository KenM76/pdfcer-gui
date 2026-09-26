# `text::print` — every word the print dialog shows

The catalog area for [`crate::dialogs::print`]. One module per surface is
the rule this directory's `mod.rs` states; the print dialog is a surface,
and it is a large one — three tabs, a preview, a device selector and a
commit button whose label is itself a disclosure.

## The copy in here is doing three different jobs

Distinguishing them is what keeps the voice consistent, so they are named:

1. **Names.** A radio's label, a heading, a tab. Sentence case, no
   trailing period, and they name the *thing*, not the act — "Actual
   size", not "Print at actual size".
2. **Disclosures.** Sentences pdfcer owes the operator because pdfcer
   inferred something, capped something, or is about to lose something:
   [`clip_summary`], [`dpi_capped`], [`raster_note`],
   [`commit_with_clipping`]. These are full sentences with punctuation,
   they name the number, and they never apologise. `docs/core-api/03`
   §6.3 enumerates exactly which values are inferences; every one of them
   has a function here.
3. **Refusals.** Why a control is absent or a job cannot go
   ([`spooler_unavailable`], [`no_printers`], [`no_pages_selected`],
   [`range_unparsable`]). These say what is true and what the operator
   can do, and they are deliberately *different sentences* for different
   causes — see the next section, which is the single most important
   convention in this file.

## ★ Three ways to have no printer, said three ways

This mirrors [`crate::text`]'s own three-way open-failure distinction, and
for the same reason: an operator must be able to tell from the words alone
which of these is true, because the three have completely different
remedies.

| function | what is actually true | what the operator does |
|---|---|---|
| [`spooler_unavailable`] | pdfcer could not ask this system about printers **at all** | nothing, in this build |
| [`no_printers`] | the spooler answered, and reported none installed | install a printer |
| [`device_unavailable`] | this *particular* printer's driver would not describe itself | pick another printer |

`pdfcer-print` is explicit that collapsing the first two is a defect:
non-Windows `list_printers` returns `Err(Unsupported)` rather than an
empty `Vec`, because *"reporting the same value for 'this platform cannot
enumerate printers at all' would collapse two different facts into one and
send a caller looking for hardware"* — `list_printers`'s
`#[cfg(not(windows))]` stub. The error type
carries that distinction across the port in
[`crate::dialogs::print::spooler`]; these three sentences are what it is
carried *for*.

## Why the commit button's label is a format string

[`commit_with_clipping`] exists because the print dialog **is** the
confirmation — there is no second gate — so the uncertainty has to be
stated *in the disclosure itself* rather than implied by a confirm step
existing. That is rule 4 applied to a button. A separate warning label
beside the button would be the version an operator can look past.
