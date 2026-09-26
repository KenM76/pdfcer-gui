# `text::redact::carriers` — the places a copy of the removed text can hide


[cc]: pdfcer_core::redact::CarrierAction::CheckedClean

## What a *carrier* is, in one paragraph, because the word is doing work

ISO 32000-1 §12.5.6.23 obliges a redaction to remove the marked content from
**all** of the document, not from the page. A PDF can hold the same run of
text in a dozen places that are not the page: the document-information
dictionary, an XMP packet, a form's XML description, the tagged-reading
tree, an attached file, an earlier saved revision. The engine calls each of
those a **carrier**, sweeps every one of them, and records a verdict per
carrier in [`RedactionReport::carriers`][cs]. This module is where those
verdicts become English.

[cs]: pdfcer_core::redact::RedactionReport::carriers

## The defect this module was written to close

Before it, this shell read the carrier list at exactly two sites and both
were the same `==` filter against `DisclosedNotScrubbed`. That has three
consequences, and each is worse than the last:

1. **The carrier's raw engine key was printed to the operator.** The
   sentence he saw read *"⚠ struct_tree: present in this document…"*. The
   engine documents those keys as *"a short stable identifier"* — an
   identifier for a program, in a report written for a person.
2. **`residual_sweep` got the generic sentence**, which is about a carrier
   that *holds* content. The sweep holds nothing; it is the engine's search
   of every other object in the file. The generic sentence was therefore not
   merely jargon but false, in the residual list, on the one surface where
   rule 1 forbids a comfortable sentence.
3. **`CheckedClean` was invisible.** Because both readers are `==` filters
   and neither is a `match`, the new variant was not swallowed by a
   catch-all — it was never read at all, and nothing on screen changed by
   one character. The engine's own doc comment says what that costs:

   > *"a shell that tells an operator 'nothing to do' when the truth is
   > 'checked, clean' has taken away the one thing that distinguishes a
   > diligence sweep from a no-op."*

   And this is not a theoretical variant. `pdfcer_core::redact`'s
   `carrier_info` reports `CheckedClean` from its own final `else`, and so
   does `carrier_residual_sweep` — the two commonest carriers on the
   operator's own files.

## The wording rule this group adds to the three it inherits

[`crate::text::redact`]'s rules 1–3 bind here unchanged. This group adds a
fourth, and every string below obeys it:

> **Say where, in his vocabulary, never in the engine's.** A carrier name is
> the whole actionable content of a residual line: *"the tagged-reading
> structure a screen reader follows"* is something an operator can decide
> about in a second, and *"struct_tree"* is something he can only click
> past. This is the same finding `raw_residual_line` was corrected for on
> 2026-09-04, one sentence over, after his report that the warning *"always
> finds text that wasn't redacted"*.

**The fallback is the raw key, deliberately.** `CarrierStatus::carrier` is
an open vocabulary — a future engine build may add one — and the choice at
that moment is between printing an unknown identifier and printing nothing.
Printing nothing would drop a disclosure the engine went to the trouble of
making. So an unknown carrier reads awkwardly and is still *there*, which is
the correct trade on this surface. The engine-API drift gate is what will
tell us a new key exists; it is how `CheckedClean` was found in the first
place.
