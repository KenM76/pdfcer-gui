# `redact::proof` — the absence proof, and the only thing entitled to the
word *verified*

★ This module is the ONLY place the absence proof exists. Nothing else in
this repository, and nothing in the engine, answers the question it asks.

## What a proof is for, here

[`pdfcer_core::redact::apply_redactions`] returns a **[`RedactionReport`]** —
a description of what the surgery believes it did. It does not return a
verdict: there is no `RedactionVerdict` and no `verify_redaction` anywhere
in `pdfcer-core`. So a caller that writes the report's bytes has been told
what happened by
the code that did it, which — as `tools/ui-verify`'s `save_copy` check
records from the other side — is *"a trace line written by the code under
test, about itself."*

This module is the independent reader. It takes the finished bytes and the
list of strings the surgery says it removed, and it goes and looks.

[`RedactionReport`]: pdfcer_core::redact::RedactionReport

## The three verdicts, and why the middle one is not a refusal

[`pdfcer_core::redact::RedactionReport::redacted_text`] carries the distinct
strings the surgery decoded while removing them — kept, in core's own words,
*"for the absence-proof gate to grep"*. Each one is looked for in three
places, and **where** it turns up decides everything:

| Where the string still occurs | Verdict | Why |
|---|---|---|
| in a decoded **content-bearing** stream of the output — page content, a form XObject, a tiling pattern, a Type 3 glyph procedure | **DISCLOSE as [`ResidualSite::DrawnContent`], acknowledgement-gated** | These are the streams a renderer draws and a text extractor reads. The proof greps the WHOLE output, so a hit here is more often another occurrence of a word that was never marked than the removed glyphs still being drawn, and pdfcer cannot tell those apart. A hard refusal survives for one case: a content survivor found at write time that was NOT in the disclosed list, meaning the bytes changed between the proof and the write. |
| anywhere else in the output — in a decoded **opaque** stream (a font program, image samples, an ICC profile, an object-stream container, an attachment) or in the **raw bytes** | **DISCLOSE** as a residual requiring the operator's explicit acknowledgement, naming *where* it was found | pdfcer cannot tell a genuine un-recognised carrier from an unrelated coincidence. Refusing would be a trap the operator cannot act on; claiming removal would be a lie. Naming it, and naming the place, is the only honest option. |
| nowhere | **verified** | This is what licenses [`crate::text::redact`]'s wording contract to use the word "verified" at all. |

Strings shorter than [`MIN_VERIFIABLE_LEN`] are excluded from BOTH halves
(see [`leaked_in_content_streams`] for the per-glyph producer that forces
it on the refusal half) and **counted separately** (see
[`AbsenceVerification::strings_too_short_for_raw_check`]) rather than
silently skipped: a two-character redaction would match somewhere in any
real file, so a byte grep for it over raw bytes or over a compressed blob
carries no information, and pretending it does would turn the disclosure
into noise operators learn to click through. They are still checked against
**content-bearing** streams, where a survival *is* meaningful — which is
exactly what [`crate::text::redact::verification_limit_line`] says on
screen (*"those were checked against the decoded page content only"*).

## ★★★ Why only a CONTENT-bearing stream can refuse

Handing the first row of that table **every** stream in the file — on the
argument that a decoded stream is content a renderer or a text extractor
will read back — is true of a content stream and false of most of the
streams in a real document.

The cost is measured, not theorised. On this repository's own
`fixtures/a1-titleblock.pdf` — a drawing sheet of exactly the kind this
program is for — marking the word *construction* and applying produces:

```text
REFUSED: VerificationFailed { survivors: [" construction"] }
```

Nothing is written, and the removal has in fact **succeeded**: 13
characters deleted from 1 content stream, 1 mark applied, 0 retained. The
byte run the proof finds is inside object 9 — a stream with `/Length1 19092`
and no `/Type`, i.e. an **embedded TrueType font program**, whose `name`
table carries the OpenType stylistic-set descriptions *"Classic
construction"* and *"Closed construction"*. A font's description of its own
letterforms vetoes the redaction.

That is not a rare shape. **Every** PDF with an embedded font carries an
English-language `name` table, so under that rule every redaction of an
ordinary English word on an ordinary document is liable to be refused
outright — a redaction tool that refuses every time to do any work.

★ **The classification would be inverted.** The raw-byte half has the right
instinct — a byte run in a place nothing draws is a coincidence pdfcer
cannot rule out, so *disclose* — and [`MIN_VERIFIABLE_LEN`] exists entirely
because of it. Applying the **opposite** rule to the **same** kind of
evidence gives the identical coincidence, merely because it sits inside a
Flate stream rather than beside one, the harshest verdict in the module
instead of the mildest.

So the sweep decodes every stream — narrowing *that* would hide evidence —
but each decoded blob is classified by [`role_of`], and only a
**content-bearing** blob can produce a refusal. Everything else is promoted
into the disclosure list with the place named, so nothing passes silently:
it is reported, in the operator's face, and gated behind the residual
acknowledgement.

★★ What this deliberately does **not** relax: a survival in page content, a
form XObject (which is what an annotation appearance stream is), a tiling
pattern or a Type 3 glyph procedure is the one hit that can still stop a
write. It is disclosed at preparation and, once acknowledged, allowed; what
refuses at write time is a content survivor that was **not** in the
disclosed list. The test
`the_sweep_reaches_a_stream_that_is_not_page_content` holds the line that
such a stream is reached by the sweep at all.

## ★ One decode, two halves

[`prove`] decodes every stream **once** and hands the same blobs to both
halves. Calling the disclosure half and the content half one after the
other, each decoding the document independently, costs two full inflate
passes over the finished file for one question. The two halves stay
separate functions because they answer two separate questions and each has
its own test; what is shared is the evidence, not the reasoning.
