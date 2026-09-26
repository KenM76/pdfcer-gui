# `text::unembed` — what the Remove-fonts window says before it takes
something out

The copy for [`crate::dialogs::unembed`], and the destructive twin of
[`crate::text::embed`].

## This is the window `tools.unembed_fonts` was blocked on, and the
blocker was TRUE

Nine of the project's scaffolded commands turned out to be sitting behind
reasons that had expired. **This one was not.** Its recorded reason said:

> `panels/fonts.rs` records that the old shell's confirmation window exists
> because *"three of unembedding's four consequences are invisible on the
> canvas (a broken PDF/A claim, an invalidated signature, a renamed font)"*.
> That disclosure surface is rule 4 work and is not built.

It is built here. Every sentence below exists because of one of those four,
and the module is worth reading as the argument for why the window could not
have been skipped.

## The FOURTH consequence, which nobody had written down

**Unembedding does not make the file smaller when pdfcer saves it.**

`UnembedPlan::bytes_reclaimable` is the number an operator is chasing, and
the engine is explicit about the trap: §7.5.6's update section is *appended*,
so the deleted objects get free cross-reference entries in a new section and
**their bytes remain in the prior revision, which is still in the file.** An
incremental save after an unembed produces a *larger* file. Only a full
rewrite drops the bytes.

`crate::app::save` writes **incrementally, always**, by design and by a
promise in a tooltip that has been on an operator-visible surface since the
command was registered. So this shell cannot deliver the reclaimed bytes at
all today.

⇒ The engine's own rule is *"this number must never be reported without the
save mode that delivers it"*, and the honest reading of that here is not to
soften the number — it is to **state the number and then state that pdfcer's
Save will not deliver it**. Filed as an operator question in
`OPERATOR_REQUESTS.md`; hiding it would make the window a sales pitch.

## Item notes

### `fn the_size_sentence_never_promises_a_smaller_file`

The one assertion in this module that guards a real trap rather than a
wording preference. `bytes_reclaimable` is the number an operator opens
this window for, `app::save` writes incrementally, and a sentence that
reported the first without the second would promise a smaller file that
pdfcer cannot produce.

### `fn intro`

It states what changes and what does not, in that order, because the
second half is the part an operator will not predict: **no text moves.**
`/Widths` is untouched and no content stream is rewritten, so every glyph
keeps its advance — what changes is which face draws inside those advances.
An operator who expected reflow and got none would think it had not worked.

### `fn remove_row`

The **shared-program** case is disclosed on the row and is the one an
operator cannot possibly infer: two fonts may point at the same stream, and
when the other one is not part of this operation the key comes out of this
descriptor while the **bytes stay in the file**. So the font is unembedded
and nothing is recovered, which looks exactly like a bug from outside.

### `fn blocked_row`

It delegates to `UnembedBlocker::reason`, which is a deliberate
exception to this crate's *"every user-visible string lives in `ui_text`"*
rule and the reason is stated in the engine's own doc: those are *"the same
words the Fonts panel and `list-fonts` already show, because a font that
refused in the report and refuses here must refuse for the **same stated
reason**."*

⇒ Two catalogs for one classifier is how the report and the refusal come to
disagree about the same font — and an operator reading two different reasons
for one font learns that neither is trustworthy. One classifier, one
sentence.

### `fn size_note`

**The fourth consequence, and the one that was in no register.** See the
module header: `bytes_reclaimable` is the number the operator wants and
`crate::app::save` writes incrementally, so pdfcer's own Save leaves every
one of those bytes in the file.

Both halves are said, in this order — the number first, because it is
real and is what the operation achieved, then the reason it does not reach
the disk. Reporting only the second would look like the feature failing;
reporting only the first would be the sales pitch.

### `fn pdfa_line`

**The first of the four invisible consequences.** Every part of ISO 19005
requires embedded fonts, so unembedding genuinely breaks a conformance claim
— and `pdfcer-core` deliberately does **not** refuse on it, saying in as many
words that it is *"a consequence the operator may knowingly accept, not a
structural impossibility"*, and that **the shells gate on it**.

This is that gate. It is a sentence rather than a refusal, because the
engine's position is that the choice is the operator's and the disclosure is
the shell's.

### `fn signature_line`

**The second of the four invisible consequences**, and the only one that is
irreversible outside this session. A signature covers a byte range; an
incremental save appends and therefore leaves the earlier signature's range
intact, but the *document* it certifies no longer matches what a reader
renders.

Reported only when the document actually carries one, for
[`pdfa_line`]'s reason: a warning about signatures on every unsigned drawing
is noise that teaches an operator to stop reading the window.

### `fn removed_disclosure`

Conditional clauses, like [`crate::text::embed::embedded_disclosure`]'s,
and for the same reason — but the **size** clause is unconditional here and
that is deliberate. It is the operator's motive for the whole operation, and
a disclosure that omitted it whenever the number was inconvenient would be
omitting exactly the case they care about.
