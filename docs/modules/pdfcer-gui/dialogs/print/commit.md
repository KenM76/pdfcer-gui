# `pdfcer-gui/dialogs/print/commit`

## Item notes

### `fn render_tile`

# ONE builder, called from both, and that is the point

Two independently-written builders eventually disagree about something, and
neither side can tell which one they are looking at. For a print preview
that failure is the whole feature — a preview exists to say what will come
out of the printer, so a preview built from its own options is a preview
that can be confidently wrong.

The choices it encodes, carried across with their reasoning:

- **`view_magnification` stays `None`** — the PRINT answer under §8.11.4.5,
  which says a printing application *"shall not apply the changes based on
  usage application dictionaries"*. Inheriting the canvas's options would
  apply the zoom-driven optional-content states the operator happens to be
  looking at.
- **The operator's layer overrides are NOT applied**, for the same clause:
  they are a viewing choice, and §8.11.4.5 puts printing on the document's
  own default configuration. `RenderOptions::layers` left at `None` is what
  expresses that — and `None` is *not* an empty set, which would reveal
  every layer the document turned off.
- **The annotation scope IS the operator's**, because it is a statement
  about the job rather than about the view.

## The settings surface landed, and this paragraph is what it changed


> One choice the old shell encoded is missing here and its absence is not an
> omission: **the CMYK conversion intent**. `pdfcer-core`'s settings surface
> does not exist in this crate yet, so there is no operator choice to carry.
> When it lands, it belongs here *and* in [`preview::PreviewKey`] in the
> same commit — otherwise the preview keeps showing a page rendered under
> the previous intent, which is the exact staleness class that key exists to
> close.


[`preview::PreviewKey`] gained the same five, for the reason that note gave.
One poster sheet: the tile's part of the page, rendered at the plan's
density, then — when the sheet has a band — placed on a sheet-sized
bitmap with its cut marks and label.

### `fn report`

Built by hand rather than by printing, which is the whole reason
[`PrintDialog::commit_notes`] was extracted: proving that the window
closes must not require putting a job on the operator's printer.

### `fn a_successful_print_asks_the_dialog_to_close`

> *"it doesn't close after I hit the print button [...] it looks greyed
> out as though it doesn't do anything even when I hit print - but it is
> working, so after many clicks I checked the printer and of course
> there was a dozen jobs there."*

`Some` is the signal to record and close; `None` is the signal to stay
open. Asserting on the discriminant rather than on the wording, because
the wording belongs to `crate::text::print` and a test that pinned it
here would be a second copy of it.

### `fn a_failed_print_leaves_the_dialog_open`

On failure the operator's next act is to choose a different printer or a
different range — which is what this window is for — and the driver's
own words in the footer are the only thing telling them which. Closing
would destroy the reason and the settings together.

### `fn a_synthesised_settings_source_adds_a_second_sentence_to_the_same_receipt`

Two sentences, not two `record_note` calls. `record_notes`' doc comment
records why that matters: the slot holds one disclosure, so a second
call REPLACES the first and which one survived would be decided by
statement order rather than by importance.

The receipt is first because it is the sentence an operator reads if
they read only one.

### `fn commit_notes`

Returns `Some(notes)` when the job went to the spooler: the sentences to
put on the application's disclosure row, in reading order. The caller
records them and closes the dialog. Returns `None` on failure, which
means *"say nothing here and leave the window open"* — the footer draws
the driver's own words and the operator picks another printer.

# Why this is extracted rather than left inline

Because the behaviour it decides is the operator's 2026-09-03 report —
*"it doesn't close after I hit the print button [...] there was a dozen
jobs there"* — and the only way to drive the inline version is to
actually print. Spooling a real job to his printer to prove a window
closes is not a test, it is the defect.

So the decision is separated from the act. `ui-verify` cannot reach it
(no headless route ends in a real spool), and this project's rule is
that a unit test is the floor rather than the ceiling — so what is
asserted here is deliberately the part that is **pure logic**: which
outcome closes, and which sentences travel. The act of printing is
`Self::commit`'s, and is covered by `print_dialog_reaches_the_spooler`.

Stated plainly because it is a real gap: *"the window closes after a
successful print"* is asserted as a decision, not as an observed
window disappearing. Closing that gap needs a driven check that prints
to a file device — `Microsoft Print to PDF` is on this machine — and it
is worth building; it is not built.

### `fn commit`

# The one place in the GUI that starts a print job

Reached only from the commit button, via [`Self::commit_requested`].
Nothing here runs as a side effect of opening, previewing, saving or
rendering — which is the shell's half of `pdfcer-print`'s own contract
that *"`spool` is the only function that reaches `StartDoc`, and it is
reached only from a control an operator deliberately clicked."*

# Why the whole job is rasterised inline

It blocks the UI thread for as long as the job takes. That is the
honest behaviour for now and it is not an oversight: a print that
proceeds in the background needs a cancel affordance, a progress
surface and an answer to "what happens if the document is edited
mid-job", and shipping the render off-thread without those three would
replace a visible wait with an invisible race. The single-slot render
worker next door is for *display*, where a cancelled render costs
nothing; a cancelled print costs paper.

### `fn trace_plan`

`scale=` is on this line beside `orientation=` because they are the
pair that exposes the orientation defect: a radio that changes
`orientation=` and not `scale=` on a landscape page is that regression,
restated. A harness can assert the relationship; a screenshot cannot.

`clipped=` and `claim=` are on this line TOGETHER, and the pairing is
the assertion — operator request O113. `clipped=` is the unchanged
geometric count; `claim=` is what the button says, as `<state>:<count>`.
A driven check asserts the *correction* between them, which no capture
can supply: a button reading "Print" and a button reading "Print"
because the cache silently never matched are the same photograph.
