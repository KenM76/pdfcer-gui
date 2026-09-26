# `text::redact::removed` — the words themselves, not the count of them

Consumed by [`crate::dialogs::redact::disclosures::removed_text`], drawn
under [`super::will_remove_heading`] beneath the counts.

## What this block exists to prevent

`OPERATOR_REQUESTS.md` **O217**, fourth bullet: *"what will be removed is
visible before it is committed."* Redaction is the one verb in this program
where a selection that takes too much cannot be undone, and until this block
the apply report answered *how much* — regions, pages, characters, streams —
and never *what*. A number cannot be checked against an intention. The words
can.

The case that makes it load-bearing rather than pleasant: the shell's unit
of text selection is the visual line, and `G032` records that the engine
groups a line without any horizontal-gap criterion, so a bill-of-materials
row welds its item number, part number, description and quantity into one
selectable thing. An operator who marks the quantity is marking the row. He
cannot see that on the canvas — R8b forbids marking the canvas, and rightly
— so the only place he can be told is here, and the only wording that tells
him is the text.

## The claim these sentences are allowed to make

[`pdfcer_core::redact::RedactionReport::redacted_text`] is not a prediction.
[`crate::redact::prepare_redaction_apply`] performs the whole removal into
memory before this dialog draws, and the engine keeps the strings *because
the interpreter decoded the codes while removing them* — its own field doc
says they are kept "for the operator's review and for the absence-proof gate
to grep". This shell had built the grep and not the review.

Two properties of that vector shape every sentence here, and both are
disclosed rather than smoothed over:

- **One entry per marked region, concatenated.** A region that removed
  `"1"`, `"BRACKET"` and `"4"` is one entry reading `1BRACKET4`, not three.
- **Distinct.** Two regions that removed identical text collapse to one
  entry, so the entry count is a floor on the region count and never equals
  it by construction. An operator counting entries to check his marks would
  be counting the wrong thing, so [`removed_text_lead`] says so.
