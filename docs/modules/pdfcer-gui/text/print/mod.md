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

## Three ways to have no printer, said three ways

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

## Item notes

### `fn the_three_no_printer_sentences_read_differently`

Not a tautology test — the same argument as
`crate::text::tests::the_three_open_failures_read_differently`. The
value of the distinction is that an operator can tell from the words
alone which of "this build cannot print", "you have no printers" and
"this printer would not answer" is true, because the three have
different remedies. Three functions producing near-identical prose
would satisfy the type system and defeat the design.

### `fn the_commit_label_states_the_clip_count`

This is the whole disclosure mechanism: if the number ever stopped
appearing in the string, the button would silently become an ordinary
Print button on a job that loses content.

### `fn the_counted_sentences_are_grammatical_at_one`

Cheap to get wrong ("1 sheets will be clipped"), and prose that reads
as machine output is prose an operator trusts less — which matters
most on exactly the sentences that are trying to warn them.

### `fn the_three_commit_labels_are_distinguishable_claims`

| label | what it claims | when |
|---|---|---|
| [`commit_with_clipping`] | N page boxes exceed the printable area | nothing examined |
| [`commit_losing_content`] | N sheets really do lose ink | every clipped sheet examined |
| [`commit_may_lose_content`] | **at most** N sheets lose ink | some examined, some not |

The hedge is the load-bearing distinction: it must be present on the
bounded claim and absent from the two measured ones. A wording change
that put "may" on all three, or took it off the ceiling, would collapse
three states into one sentence and hide exactly the difference the
count was made better to expose.

### `fn only_the_bounded_summary_hedges`

[`clip_summary`] serves both the geometric and the measured state — in
the first it is the unchanged shipped wording, in the second it is
verified — so the only sentence that may hedge is the ceiling's.

### `fn the_dpi_disclosure_names_what_it_costs`

An operator deciding whether to raise the cap needs the cost of doing
so, not merely the fact that a cap exists. Dropping any one of the
three turns a decision aid back into a notification.
