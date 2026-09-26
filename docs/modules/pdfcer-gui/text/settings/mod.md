# `text::settings` — every word the Settings window shows

The catalog area for [`crate::dialogs::settings`]. Ported from the old
shell's `ui_text.rs`, where these strings occupied roughly 700 lines in the
middle of a 7,912-line file.

## The one rule this module has that the rest of the catalog does not

Carried across verbatim from the source, because it is the reason the copy
is written the way it is:

> Every string here must be readable by someone who has never opened the
> PDF standard. These settings exist BECAUSE the standard is silent, so the
> operator is being asked to make a judgement — and a judgement cannot be
> made from a clause number. The clause is named for traceability; the
> SENTENCE has to stand on its own.

So `§8.6.4.4` appears in exactly one place per setting, inside a sentence
that would still make sense with the number deleted. An operator who has
never heard of ISO 32000-1 must be able to choose correctly.

## The three obligations, and how they are enforced by shape

`settings_panel.rs`'s header names three things a settings screen must
show that a conventional one omits. Two of them are enforced here by
**function naming**, not by review: every setting has a `*_title`, a
`*_silence` and a `*_radius`, and
[`crate::dialogs::settings::widgets::header`] takes all three as required
arguments. A setting cannot be added without answering all three.

| obligation | where it lives |
|---|---|
| 1. What the default rests on | inside the chosen option's `_note`, and only where it is true |
| 2. That a choice was made at all | `*_silence` — what the standard leaves open |
| 3. Which way costs what | `*_radius` — preview, extraction, or **saved bytes** |

### Obligation 1 is the one the source got wrong, and it is fixed here

The ambiguity register grades each recommended default: **(a)** observed
Acrobat behaviour, **(b)** corpus census, **(c)** other implementations,
**(d)** reasoned inference — *a guess*. Most are (d), and the source's own
header says a guess must say it is a guess.

It said so for five settings and not for five others that `pdfcer-core`
grades (d) just as explicitly: `image_minify`, `unmappable_code`,
`actual_text`, `missing_as` and `trailing_eol` all read as confident
recommendations. **Their notes now carry the disclosure**, in the same
words the settings that had it already use, so the contract the window
states about itself is true of all thirteen rather than of eight.

The one *positively* sourced default — CMYK JPEG polarity, tier (c) —
says so too, because "pdfcer matched every other engine" and "pdfcer
guessed" are different claims and must not read alike.

## Two disclosures the source documented in the engine and showed nowhere

Both are added here, and both are facts rather than directions:

- [`unmappable_omit_note`] now says that a run whose codes are **all**
  unmappable disappears entirely under *Leave it out* — not merely that
  characters go missing. The layout pass drops a run with no characters,
  so a page of `Identity-H` text with no `/ToUnicode` yields *zero runs*.
  That is the surprising half and the source's note omitted it.
- [`actual_text_bound`] is new. No length correspondence exists between
  `/ActualText` and the content it replaces, so character-level mapping
  back to glyph positions is **impossible across such a run whichever
  option is chosen** — which bounds search highlighting, selection and
  redaction-by-text to sequence granularity. `pdfcer-core` calls this *"a
  fact to disclose, not a direction to pick"* and the old window disclosed
  it nowhere.

Both settings' radius lines also now name **redaction**, because R35 is
explicit that a redaction built under one value is not equivalent under
another, and "affects copied and extracted text" does not tell an operator
that.
