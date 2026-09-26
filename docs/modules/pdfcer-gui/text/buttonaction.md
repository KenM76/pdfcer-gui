# `text::buttonaction` — every word the *What this button does* chooser says

One module for one control, because the control is where this project's
rule-4 obligation is heaviest: two of the seven choices write an address
into the document that some other program may act on, and **the operator
cannot see that by looking at the page**. Everything they can learn about it
has to be said here.

## The disclosure rule these strings implement


⇒ *"Design it from `SubmitDisclosure`, not from Acrobat"* — a straight copy
of Acrobat's dialog would be a **regression** against what pdfcer can now
say. So these strings state the whole address, and the six facts about the
payload that ISO 32000-1 §12.7.5.2 makes true and nobody can guess.

## Where the disclosure is NOT

**Not on the canvas.** Rule 4's clause that is most often got backwards:
applied content renders exactly as saved content will. A button carrying a
submit gets no badge, no tint and no dashed outline on the page — it is
drawn as the file will draw it. The disclosure lives in this dialog, and
afterwards in the status line, which is off-canvas by construction.

## Not a warning, and not a refusal

None of these say *"are you sure?"*. No scheme, host or port is refused
anywhere — destination policy is open by operator ruling — and `https`
appears **zero times** in ISO 32000-1, so blocking `http://` would be pdfcer
inventing a conformance requirement. [`submit_unencrypted`] therefore
**states** it and lets the operator decide. Nothing here may be phrased as
*"the standard requires"*, because none of it is.

## Item notes

### `fn every_choice_is_named_and_explained`

The reach clause is the load-bearing half — see [`does_note`]'s comment
on why the four inert ones carry one too. A new variant added without a
note would fail to compile (the `match` is exhaustive); a new variant
added with an empty one would not, so this asserts non-emptiness.

### `fn the_submit_disclosure_names_every_fact_it_owes`

Asserted by keyword rather than by exact text so a rewording does not
break it — but a rewording that DROPS one of the four will, which is the
point. These are the facts an operator cannot learn any other way.
