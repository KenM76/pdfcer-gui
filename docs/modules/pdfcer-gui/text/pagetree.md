# `text::pagetree` — what the operator is told when a save is refused
because the document no longer agrees with itself

One event, **three** sentences, and a wording rule that is stricter than
most of this catalog because the sentence has to do something unusual:
**explain a refusal whose cause is a defect in pdfcer itself, without either
blaming the operator or leaving him unsure whether his work still exists.**

## The event

[`crate::app::save::write_copy`] built the bytes, [`crate::pagetree::audit`]
walked their page tree, and some node's `/Count` does not match the number
of pages actually beneath it. Nothing was written. See
[`crate::pagetree`]'s header for what that state is and why it is not
repaired here.

## The four things the sentence has to carry, in this order

1. **That no file was written**, so the operator is not left looking for
   one. `crate::app::status::decline`'s `SaveFailed` line already says *"the
   copy was not written"* and this sentence is added **beside** it rather
   than instead of it —
   [`crate::app::save::redaction_refusal_note`]'s standing reason, which is
   that replacing the fact with an explanation leaves an operator unsure
   whether a file appeared.
2. **That his edit is still here.** This is the part a refusal most often
   gets wrong. The session is untouched — `to_incremental_bytes` takes
   `&self` — so the document on screen after the refusal is exactly the
   document that was there before it, undo stack and all. An operator who
   believes a failed save cost him his work will do something drastic to
   recover it.
3. **What is actually wrong, in his terms.** Not `/Count`, not `/Kids`, not
   "the page tree is inconsistent". The two numbers and the symptom: *this
   document says it has 36 pages and only 34 of them are really there;
   saving it would give you a file that opens in Acrobat with 2 blank pages
   at the end.* That is his own bug report read back to him, which is the
   strongest evidence a sentence can offer that it understands the problem.
4. **What he can do next.** When pdfcer caused the damage, exactly one thing
   works and it is named: **undo the page removal.** Nothing else does — not
   saving somewhere else (the fault is in the document, not the disk), not
   saving again (it is deterministic), not closing and reopening (the file on
   disk is the unedited one). When pdfcer did **not** cause it, undo is a
   circle and the sentence says so and names a different route — see
   [`save_refused_pre_existing`].

## The wording rule: name the symptom the OTHER reader will show

Every sentence here says what **Acrobat** will do, and that is deliberate
and is the only honest framing available. pdfcer's own reader walks `/Kids`
and sees a perfectly healthy 34-page document; if the copy described what
pdfcer sees, it would be describing a file that is fine. The whole defect is
that two readers disagree, so the operator is told about the reader whose
answer is wrong *and which he uses*. He named it himself in his report.

⇒ Corollary, binding on anyone editing these strings: **do not soften
"blank pages" into "may not open correctly".** He can verify "blank pages at
the end"; he cannot verify a hedge, and a hedge he cannot verify reads as
pdfcer refusing for reasons of its own.

## Why the number of blank pages is computed rather than described

`declared - reachable` is the count of pages Acrobat will list that are not
in the file, and on the delete path it is exactly the number of pages he
removed — which is the coincidence that let him diagnose it in one sentence
(*"equalling the number of pages I deleted"*). Printing the number rather
than saying "some" is what lets him recognise his own symptom.

## THREE sentences, not one, because three states are genuinely
different — and two of them would give bad advice in the third's place

| | when | what only it can say |
|---|---|---|
| [`save_refused_root`] | the **root** disagrees, and the file arrived sound | the exact symptom: *n* blank pages at the end |
| [`save_refused_interior`] | only an interior node disagrees | that readers will show the wrong pages, without promising which |
| [`save_refused_pre_existing`] | the file **already** disagreed when it was opened | that pdfcer did not cause it, that **undo will not help**, and the one route that repairs it |

The third is the one that must not be merged away. The first two both end
*"undo the page removal (Ctrl+Z)"*, which is right exactly when pdfcer caused
the damage — and a **circle** when the file came in broken. An operator who
empties his undo stack against a refusal his own tool told him undo would fix
has lost his work as well as his time, which is strictly worse than an
unexplained refusal. [`tests::only_the_sentences_pdfcer_can_undo_offer_undo`]
is what stops the three being consolidated back into one on the grounds that
they say nearly the same thing. Which one applies is decided by
`crate::pagetree::refusal_origin`, from structured data — a second audit
of the file on disk — and never by inspecting a message.


Both ordinary sentences used to end *"This is a fault in pdfcer, it has been
reported, and pdfcer will not write a file it knows is damaged."* Every
clause of that was true when written: `delete_pages` updated the immediate
parent's `/Count` and no ancestor, it had been reported that morning, and
the refusal was the only thing between the operator and a file Acrobat opens
with blank pages at the end.

**`Pass 251.1` fixed it the same day.** So on this engine the sentence
blames pdfcer for damage pdfcer did not do — and the operator meets that
sentence precisely when he is least able to judge it.

⇒ **The remedy was to delete the attribution, not to update it.** A
refusal owes him three things: that nothing was lost, what is wrong in his
terms, and what to press. **Whose fault it is is not one of them** — it is
the sentence's least useful clause and its most perishable, and this header
already carried a paragraph predicting exactly that it would go stale.


⚠ `save_refused_pre_existing` **keeps** its attribution, and must: its whole
job is to say *pdfcer did not do this and undo will not help*, which is a
statement about the file rather than about pdfcer's record, and getting it
wrong costs him his undo stack as well as his time.
