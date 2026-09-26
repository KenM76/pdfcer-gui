# `text::sign` — every operator-facing string on the control that puts the
operator's own signature into a document

The write side of a subject whose read side already has two modules:
[`crate::text::security`] reports what a document says about its
protection, [`crate::text::trust`] reports what pdfcer could and could not
check about a signature that already exists, and this one describes
something pdfcer is **about to do with a private key**.

## THE STANDARD THIS MODULE IS HELD TO, AND IT IS NOT THE USUAL ONE

[`crate::text::trust`]'s subject is a **verdict**, and its failure mode is
claiming more than the engine checked. This module's subject is an **act**,
and its failure mode is different and worse: a sentence here can persuade an
operator to attach their legal identity to a document. So two rules bind
every string below, and both are narrower than "be accurate".

1. **Nothing here calls a signature valid, trusted, secure or verified.**
   Not once, not as a summary, not in a tooltip. Authoring a signature and
   the signature being *trusted by a recipient* are different facts settled
   by different parties, and this surface only ever performs the first.
   [`crate::panels::signatures`] is the only place in pdfcer that reports
   the second, it reports three facts that never collapse into one, and a
   cheerful word here would undo that whole design before the panel is
   opened. `sign.svg`'s own note carries the same constraint for the glyph.
2. **Every sentence about what will be written names what will be
   written.** `/Reason`, `/Location` and the signing time are the
   operator's words, copied verbatim into a legal artifact; the copy says
   so rather than describing them as "details".

## What is deliberately NOT offered, and it is a string's absence

**There is no *Name* field**, and its absence is a decision rather than an
omission — recorded here because an absence cannot be read out of the code
that does not contain it.

`SignRequest::name` writes `/Name`, and the engine's own note says `None`
*"omits the key and a verifier falls back to the certificate subject (Table
252 says it should anyway)"*. A free-text name beside a certificate is a
**second, unverifiable claim about who signed**: nothing stops it saying
something the certificate does not, and a reader that trusts `/Name` over
the subject would show a name nobody vouched for. The certificate is the
name. So the window shows the subject it read out of the operator's own
`.pfx`, and offers no way to write a different one.

## `/ContactInfo` is not offered either, for a smaller reason

It is legitimate and harmless — a phone number for a verifier who wants to
reach the signer. It is left out because three free-text boxes on a form
whose two important controls are the certificate and the destination is
three boxes an operator scrolls past, and because nobody has asked for it.
Adding it is one field and one string; that is the right size for a request,
and the wrong size for a guess.
