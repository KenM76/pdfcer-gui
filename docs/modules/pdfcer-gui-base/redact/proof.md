# `redact::proof` — the absence proof, read through the engine's verb

The proof is `pdfcer_core::redact::verify_absence`. This module adapts it to
the two questions the redaction pipeline asks, and keeps the shell's own
tests of it as a contract test against the engine.

## What a proof is for, here

[`pdfcer_core::redact::apply_redactions`] returns a **[`RedactionReport`]** —
what the surgery believes it did. `verify_absence` is the independent
reader: it takes the finished bytes and the strings the surgery says it
removed, and goes and looks.

[`RedactionReport`]: pdfcer_core::redact::RedactionReport

## What this module adds

- [`prove`] returns a `Proof`: the engine's `AbsenceVerification` (the
  disclosure) and its drawn-content survivors (the refusal) as two fields,
  because the two answers have different consequences and must not collapse.
- [`survivors_in_content_streams`] re-asks the refusal question of the exact
  buffer `PreparedRedaction::write_to` / `to_verified_document` is about to
  hand on. A survivor that was not disclosed at preparation
  ([`disclosed_in_drawn_content`]) means the bytes changed between the proof
  and the write, and refuses.
- `contains` is the tests' own byte search, so the tests do not share their
  witness with the proof they audit.

## What stops *verified*

`AbsenceVerification::is_clean` is the only licence for the word. It is false
when any residual remains, when `streams_undecoded > 0` (a stream that would
not decode was never searched; its stored bytes still were), or when
`output_unparsed`. The dialog states each cause off-canvas:
`text::redact::verification_limit_line`, `proof_unsearched_line` and
`proof_unparsed_line` (`dialogs::redact::proof_limits`). Undecoded and
unparsed are disclosures, not acknowledgement gates: neither is a residual.

Trace: `redact-prove drawn_content_hits=… residuals=… short=… undecoded=…
unparsed=<0|1>`.

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
(see the test `a_short_string_is_counted_as_unverifiable_and_the_long_one_still_refuses`
for the per-glyph producer that forces it on the refusal half) and **counted separately** (see
[`AbsenceVerification::strings_too_short`]) rather than
silently skipped: a two-character redaction would match somewhere in any
real file, so a byte grep for it over raw bytes or over a compressed blob
carries no information, and pretending it does would turn the disclosure
into noise operators learn to click through. They are not searched for
anywhere, and [`crate::text::redact::verification_limit_line`] says so on
screen.

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
but each decoded blob is classified by its dictionary, and only a
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

## The tests

The module's tests predate the engine verb and run unchanged against it. A
red one is an engine contract change, reported through the request channel.

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
question every test below asks is what the engine makes of a stream's
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

A proof that only read
`/Contents` would report this file clean while the string sat in a form
XObject that every renderer draws.

This test holds the line the content/opaque classification
deliberately does not move: a form XObject — which is also the shape of
every annotation appearance stream — is drawn, so a survival in one is
a content hit and never an opaque one.

## `ResidualSite` is non-exhaustive

`text::redact::raw_residual_line` names every site the engine has today and
falls back to *somewhere this build has no name for* for one added later, so
a new site is disclosed rather than unrepresentable.
