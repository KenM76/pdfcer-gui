# `pdfcer-gui/text/markup/edits`

## Item notes

### `fn a_dropped_rich_copy_is_disclosed_and_an_absent_one_is_not`

Both directions, because either alone is satisfiable by a broken build:
a function returning `Some` unconditionally passes the first, and one
returning `None` unconditionally passes the second. This project has
shipped the second shape — a disclosure wired to a field nobody set —
and the only symptom was silence.

The silent case is the *common* one and that is why it is asserted at
all: pdfcer's own annotations never carry `/RC`, so every comment this
operator writes and then edits takes the empty path. A sentence that
fired on all of them would be read once and skipped thereafter,
including on the one edit where it mattered.

### `fn the_rich_text_sentence_is_in_his_words_not_the_specs`

The wording rule this catalog follows everywhere: name the thing in the
operator's vocabulary. A drawing-office reviewer has no idea what `/RC`
or `/DS` are, and the only fact they can act on is that a comment they
had styled elsewhere is now plain text.

It must also **not open with an apology or a loss**, because the net
effect of this change is that their document stopped contradicting
itself. Before it, `/Contents` held the new words while `/RC` held the
old ones and some readers showed the old ones — on a `/FreeText`, on the
page itself.

### `fn a_delete_with_no_collateral_produces_no_sentence`

The overwhelmingly common annotation has no pop-up, no replies and no
group. A sentence that appeared on every selection would be read the
first three times and skipped for ever after, which is exactly what makes
the interesting case invisible when it finally arrives.

### `fn the_two_tenses_name_the_same_four_consequences`

Pinned by counting: for one set of counts each function names every
consequence the other names. What is deliberately NOT asserted is that
the strings are equal or mechanically derived — they are not, because
*"1 reply is left"* and *"1 reply will be left"* are different English —
and a test that demanded a shared template would forbid the difference
that makes both of them readable.

### `fn neither_tense_promises_redaction`

Deleting an annotation takes an entry out of `/Annots`; it does not touch
page content, and an incremental save leaves the previous revision in the
file. `docs/core-api/03-capabilities.md` §3.4 — *"delete is not
redaction"* — and a preview that promised removal would be the exact
wording `crate::text::redact`'s header forbids, stated one gesture
earlier than the disclosure that already observes the rule.

### `fn the_locked_sentence_names_the_file_as_the_author_of_the_rule`

§12.5.3's `Locked` is a statement the producer wrote into the annotation.
Wording it as pdfcer's own decision would send an operator looking for a
pdfcer setting to turn off, and there is not one.
