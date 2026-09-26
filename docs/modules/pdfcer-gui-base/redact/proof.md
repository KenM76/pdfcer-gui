# `redact::proof` — the absence proof, and the only thing entitled to the
word *verified*

This module is the ONLY place the absence proof exists. Nothing else in
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

## Why only a CONTENT-bearing stream can refuse

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

**The classification would be inverted.** The raw-byte half has the right
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

What this deliberately does **not** relax: a survival in page content, a
form XObject (which is what an annotation appearance stream is), a tiling
pattern or a Type 3 glyph procedure is the one hit that can still stop a
write. It is disclosed at preparation and, once acknowledged, allowed; what
refuses at write time is a content survivor that was **not** in the
disclosed list. The test
`the_sweep_reaches_a_stream_that_is_not_page_content` holds the line that
such a stream is reached by the sweep at all.

## One decode, two halves

[`prove`] decodes every stream **once** and hands the same blobs to both
halves. Calling the disclosure half and the content half one after the
other, each decoding the document independently, costs two full inflate
passes over the finished file for one question. The two halves stay
separate functions because they answer two separate questions and each has
its own test; what is shared is the evidence, not the reasoning.

## Item notes

### `enum StreamRole`

See the module docs on why only a content-bearing stream can refuse. The
distinction is not cosmetic: it is the difference between a refusal that
writes nothing and a disclosure the operator can act on, applied to
byte-for-byte identical evidence.

### `fn leaked_in_content_streams`

Returns `Some(survivors)` when any redacted string of at least
[`MIN_VERIFIABLE_LEN`] characters is still present in a **content-bearing**
decoded stream — bytes a renderer draws and an extractor reads — and `None`
when the output is clean by that measure.

# The floor applies HERE too

Checking strings of any length on this half assumes the needle is a WORD —
*inside a content stream even a two-character survival is the redacted
glyphs still being drawn.* The needle is not always a word:
**Ghostscript 8.15 draws every glyph with its own show operator**, so on a
24-page drawing the surgery's `redacted_text` for the run `3.5 TYP` is
seven single characters — `["3", ".", "5", " ", "T", "Y", "P"]` — and this
half finds `"3"` on all 24 pages (as it should: every drawing has a 3 on
it) and refuses a redaction that succeeded. Selecting more text raises the
count, so the tool reports dozens of pieces of supposedly-removed text and
stops being trusted.

A needle under the floor carries no information in ANY real file, in a
content stream as much as in the raw bytes — that argument is wrong about
the stream and right about the word. So short needles are
not refused on; they are **counted and disclosed** as unverifiable by
[`verify_absence`] (`strings_too_short_for_raw_check`), and the operator
reads that on the window.

What is NOT relaxed: a needle of four or more characters surviving in drawn
content is still a hard refusal, and the test
`a_short_string_is_counted_as_unverifiable_and_the_long_one_still_refuses`
holds both halves of that line.

# The engine joins per mark — keep the floor anyway

`pdfcer_core::redact::RedactionReport::redacted_text` is **one entry per
`/Redact` mark**, carrying the concatenation of what that mark removed, so
`3.5 TYP` arrives as one seven-character needle rather than seven
one-character ones and this half refuses on it as it should.

**[`MIN_VERIFIABLE_LEN`] is not thereby obsolete, and must not be
removed as a workaround whose cause is gone.** It is not only about
per-glyph producers: a mark covering a genuinely short string still yields
a genuinely short needle, and `"3"` on a drawing is `"3"` whoever wrote the
file. The joining changes how OFTEN the floor is reached on such producers,
not whether it is right when it is. Belt and braces is the right posture on
the one operation where a false *clean* is an incident.

A test written on an ordinary producer cannot tell those two worlds
apart — a file that draws the run in a single `Tj` reports `["3.5 TYP"]`
whether the engine joins per mark or not. A plausible-looking test that is
incapable of failing measures nothing.

The filter on [`StreamRole::Content`] is the whole of what keeps this
refusal honest. Without it this function sees every stream in the file, and
a font program's own description of its ligatures vetoes a completed
redaction. See the module docs.

### `fn verify_absence`

`decoded` is passed in rather than computed because a residual is *"in no
content-bearing stream AND somewhere else"*, so both halves of the sweep are
needed to classify a single hit — and because [`prove`] already has them.

# The order of the four questions, which is the whole of the logic

1. **Is it shorter than [`MIN_VERIFIABLE_LEN`]?** Then count it as
   unverifiable and stop — no half of the proof can say anything about it
   (see [`leaked_in_content_streams`] for why this question comes ahead of
   the content check). `strings_checked` excludes
   it, so *"verified N pieces"* on the window counts only what was.
2. **Is it in a content-bearing stream?** Then it is a residual at
   [`ResidualSite::DrawnContent`] — disclosed, acknowledgement-gated, with
   the sentence that says *outside the area you marked*. (See the site's
   own docs for why this is a disclosure and not the refusal.
   [`leaked_in_content_streams`] computes the same set, for the write-time
   check that nothing UNDISCLOSED survived.)
3. **Is it in an opaque decoded stream?** Disclose it, naming that stream's
   kind. Checked before the raw bytes because the answer is more specific:
   an uncompressed font program would satisfy both, and *"inside an embedded
   font program"* tells the operator more than *"somewhere in the file"*.
4. **Is it in the raw bytes?** Disclose it as [`ResidualSite::RawBytes`].

### `fn decoded_streams_of`

A document that cannot be re-parsed yields an **empty** list rather than a
panic or an error. That looks like a false clean bill and is not, for a
reason worth stating plainly: these are bytes pdfcer itself just wrote, so an
unparsable output means a **writer** bug, and the raw-byte arm of the
disclosure still covers the whole buffer either way. A skip narrows the
evidence rather than fabricating it — and [`super::prepare_redaction_apply`]
is separately unable to produce such a buffer, because it re-parses the
output itself.

### `fn decode_every_stream`

The wide sweep is still the point, and it is unchanged: a redaction that only
*looked at* page content streams would say nothing about a form XObject, a
metadata stream, an embedded file, or an **object-stream container**, whose
compressed payload can carry a stale copy of a dictionary that was promoted
out of it (engine rule R38). Decoding the container like any other stream is
what lets a grep see that copy at all.

The narrowing is in the **verdict**, never in the sweep. Every stream is
decoded and searched; only a blob [`role_of`] calls [`StreamRole::Content`]
can refuse a write. The rest can only disclose. See the module docs for the
measurement behind that.

Streams whose filters this build cannot decode are skipped rather than
failed: their *raw* bytes are still covered by the raw-byte arm of the
disclosure, so a skip narrows the evidence rather than fabricating it.

### `fn content_stream_ids`

These two cannot be recognised from the stream's own dictionary — a page
content stream carries no `/Type` and no `/Subtype` at all (it is the
*emptiest* dictionary in the file, typically just `/Length`), and a Type 3
glyph procedure is the same shape. They have to be found from the other end,
by walking what refers to them.

That asymmetry is why the classification is a whitelist reached two ways
rather than a blacklist of known-opaque kinds. A blacklist gets the default
wrong in the safe-looking direction and then has to be complete forever; this
gets the default wrong in the *disclosing* direction, where being wrong costs
the operator a sentence to read rather than a refused document.

### `fn role_of`

The single decision the whole refusal turns on. Getting it wrong in
one direction (calling a drawn stream opaque) would let a real leak be
disclosed instead of refused; getting it wrong in the other (calling an
opaque stream content) is the defect being fixed — a coincidence refusing a
completed redaction.

The three ways a stream is recognised as content:

1. **by reference** — it is in some page's `/Contents`, or it is a Type 3
   glyph procedure. See [`content_stream_ids`] for why these cannot be
   recognised any other way.
2. **`/Subtype /Form`** — a form XObject. This is also the shape of an
   **annotation appearance stream**, which is why appearances need no case of
   their own: a `/Widget`'s or a `/FreeText`'s `/AP` `/N` is a form XObject
   and is caught here.
3. **`/PatternType 1`** — a tiling pattern, whose stream is the content of
   one cell, painted repeatedly.

Everything else is opaque, and the `site` it is given is what the operator
will be shown. The font-program test comes first among those because it is
the common case and because its markers are unambiguous: `/Length1` is
defined by ISO 32000-1 Table 127 as the length of an uncompressed **font
program**, and `/Subtype /Type1C`, `/CIDFontType0C` and `/OpenType` are the
three `/FontFile3` subtypes.

### `fn pdf_drawing`

Synthetic rather than a fixture file: the point of every test here is a
*known* byte layout, and a real producer's output would make "the string
is in a decoded stream" an accident of that producer's filter choices.

### `fn a_string_still_in_a_decoded_stream_is_reported_as_a_survivor`

The first thing to establish about any absence proof: a check that only
ever reports "clean" is satisfied by any build at all. So this asserts
the *positive*
— a string that genuinely is in a decoded stream is found — before
anything below asserts an absence.

### `fn a_raw_byte_run_outside_every_stream_is_disclosed_rather_than_refused`

The middle row of the module's table, which is the row a simpler design
would collapse. The fixture puts the run in a place no content stream
reaches — a `/BaseFont` name — which is exactly the "unrelated
coincidence" case the wording is careful not to call a leak.

### `fn a_short_string_is_counted_as_unverifiable_and_the_long_one_still_refuses`

Refusing on a two-character needle surviving in a content stream — on
the argument that inside a stream even two characters are glyphs still
being drawn — refuses a correct redaction on every page of a
Ghostscript 8.15 drawing: that producer draws one glyph per show
operator, so `redacted_text` is `["3", ".", "5", " ", "T", "Y", "P"]`
and `"3"` is on every sheet. A needle under the floor carries no
information anywhere, so both halves of the floor's argument point the
same way: count it, disclose it, refuse on nothing shorter than the
floor.

### `fn an_empty_needle_matches_nothing`

`contains` returns `false` for an empty needle deliberately: the
mathematically-correct answer (`true`, every haystack contains the empty
string) would make every proof report a leak.

### `fn bytes_that_do_not_parse_still_get_the_raw_byte_half`

No stream can be decoded, so the decoded half reports nothing — and the
**raw** half still finds the run, which is what stops this from reading
as a clean bill.

### `fn pdf_with_extra_stream`

The extra stream is deliberately **not** referenced from the page: the
question every test below asks is what [`role_of`] makes of a stream's
own dictionary, and a reference would answer a different question.

### `fn every_opaque_site_is_recognised_by_its_own_dictionary`

One fixture per site rather than one assertion over a table: a table
that got the *same* wrong answer for every row would still be internally
consistent and would pass.

### `fn a_survivor_in_drawn_content_is_listed_as_a_residual_at_its_own_site`

One finding, two consumers. A whole-file grep finds the same word on
pages that were never marked, so the content hit is a
[`ResidualSite::DrawnContent`] residual behind the acknowledgement gate
— the operator can override and redact what the tool can — AND is still
reported as a survivor for the write-time check that nothing
undisclosed slipped in between preparing and writing.

### `fn a_tiling_pattern_and_a_type3_glyph_procedure_are_drawn_content`

Neither can be recognised the way a form XObject can. A tiling pattern's
stream carries `/PatternType 1` and no `/Subtype`; a Type 3 glyph
procedure carries **nothing at all** and is reachable only through its
font's `/CharProcs`. Both paint glyphs, so a survival in either is the
redacted content still on the page — and a whitelist that missed them
would silently downgrade a real leak to a tick-box.

### `fn the_sweep_reaches_a_stream_that_is_not_page_content`

The case that motivated `decode_every_stream`: a proof that only read
`/Contents` would report this file clean while the string sat in a form
XObject that every renderer draws.

This test holds the line the content/opaque classification
deliberately does not move: a form XObject — which is also the shape of
every annotation appearance stream — is drawn, so a survival in one is
a content hit and never an opaque one.

### `const MIN_VERIFIABLE_LEN`

Below this length a byte-run match tells you nothing: `"Dr"` occurs inside
`/Widths`-adjacent binary, font names, dates and half the words in any
document, so such a hit would fire on a perfectly good redaction. Four
characters is the point at which a coincidental match stops being the
expected outcome — chosen deliberately conservatively, and paired with the
fact that short strings are still verified against **content-bearing**
streams (where the same match *is* meaningful, because it is being drawn).

The floor governs the whole disclosure half — raw bytes **and** opaque
decoded streams — not the raw bytes alone. A two-character run inside a
compressed font program is the same coincidence this constant exists to
refuse to draw conclusions from, merely wearing a `/FlateDecode`.

The count of strings this excludes is reported, never hidden — see
[`AbsenceVerification::strings_too_short_for_raw_check`], and
[`crate::text::redact::verification_limit_line`], which is the sentence that
puts the number in front of the operator.

The floor governs BOTH halves of the proof — the refusal as well as the
disclosure. See [`leaked_in_content_streams`] for the per-glyph producer
that makes a refusal-half floor necessary.

### `enum ResidualSite`

This exists because the disclosure it feeds has to be **actionable**. A
residual an operator cannot place is a warning they can only ignore, and a
warning that is always ignored is worse than none, because it also trains
them to ignore the real one.

Naming the site converts *"the text is still in the file somewhere"* into
*"the text also spells a word inside an embedded font program"*, which the
operator can weigh in a second. It is still a disclosure and never a verdict:
pdfcer states where the bytes are, not what they mean.

The variants are **carriers**, deliberately in the engine's vocabulary
(`pdfcer_core::redact::CarrierStatus::carrier`), so the two disclosure
vocabularies on one screen do not diverge.

### `struct Residual`

A struct rather than a bare `String`, so the site travels with the text
instead of being re-derived (or, more likely, lost) by whichever surface
renders it. Rule 15's spirit: a value that means *"the text `X` occurs in a
font program"* must not be able to degrade into a value that means *"`X`"*.

### `struct AbsenceVerification`

This is the structure the wording contract reads: *"never say **verified**
unless a real verification step ran"*. [`Self::is_clean`] is the predicate
that licenses the stronger word, and
[`crate::text::redact::verified_line`] is the only sentence in the catalog
permitted to use it.

### `struct Proof`

Two fields because the two answers have **different consequences** and must
not be collapsed: [`Self::survivors`] is a refusal and
[`Self::verification`] is a disclosure. A single "how did it go" value would
invite a caller to treat the worse one as the milder one, which is precisely
the reading this whole module exists to prevent.

### `fn survivors_in_content_streams`

The write path's last gate: [`super::PreparedRedaction::write_to`] re-asks
this question about the exact buffer it is a statement away from handing to
the file system. See that method's docs for why a second run of a check that
has already passed is not redundancy but the thing that makes the proof
**structural** rather than procedural.

### `fn contains`

The same naive scan `pdfcer-core`'s own absence tests use, kept local rather
than exported from core: it is three lines, and **an absence proof that
shared its search routine with the code it is auditing would be a weaker
proof.**
