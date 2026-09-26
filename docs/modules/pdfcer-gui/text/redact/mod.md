# `text::redact` — every word the redaction surface says

Consumed by [`crate::panels::redact`] (mark and review) and
[`crate::dialogs::redact`] (the apply transaction and its report).


> The wording rules here are stricter than anywhere else in this catalog,
> because **this is the one feature where a comfortable sentence is a
> security defect.** Three of them, and binding on anyone editing these
> strings:
>
> 1. **Never say "removed" without qualification when anything was left.**
>    A residual is named in the SAME sentence as the success, never in a
>    footnote the operator can miss.
> 2. **Never say "verified" unless a verification step actually ran.** One
>    does — [`crate::redact::prepare_redaction_apply`] greps the finished
>    bytes — so the word is earned; but [`verified_line`] is the only place
>    it may appear, and only from a clean
>    [`crate::redact::AbsenceVerification`].
> 3. **Never put the word "Undo" near a post-apply state.** Every OTHER edit
>    in pdfcer teaches the operator that undo is available until save; this
>    is the one moment that learned expectation is wrong, so the copy
>    corrects it on screen instead of leaving it to be assumed.


Rule 3 was written for a world in which an apply was a write, and it survived
`Pass 250.1`'s collapsing verb because that verb destroyed the undo log
outright. `Pass 250.2` makes it false as stated: the deferred route
**preserves undo completely**, and a staged removal can be undone — by
stepping back over the marks, or by calling it off with
[`cancel_button_staged`]. A rule forbidding the word "Undo" near that state
would forbid the true and useful sentence.

What the rule was actually protecting is the state where undo genuinely does
not help, and that state is now precisely nameable:

> **3′. Never suggest that Undo can recover content that has reached a
> file.** Before the save, undo works and the copy may say so. After it, no
> sentence may offer undo as a way back — not on the write-now
> destinations, not on the deferred one after
> [`saved_applying_redaction`], not anywhere.

The distinction is the whole of it: undo reaches the **arming**, and never
reaches the **removal**. `tests::no_post_apply_sentence_mentions_undo_as_a_way_back`
enforces 3′ over the sentences the removal has happened in, and the sweep's
membership list is now the load-bearing half — a sentence about a
*pre*-save state belongs out of it, and one about a post-save state belongs
in it.

## ★ The one distinction every string here has to keep alive

`crate::text::commands::edit_redact`'s shipped tooltip states it in four
words — ***"Marking is reversible; applying is not"*** — and
`crate::shell::manifest::edit` explains why the two commands sit together in
that order: *"the asymmetry between them is the dangerous part."*

The single most-cited real-world redaction failure is an operator who
believes marking **is** redacting and ships the marked file. So the marking
copy never says "removed", the review count says the content is *still
there*, and the apply copy leads with permanence rather than burying it.

## ★ A departure from the source, and it is about this shell rather than
about copy

The old shell's permanence statement already deviated from its own ui-spec,
because apply there wrote a **new file** and left the open document alone.
That was true here too until 2026-09-04, when the operator asked for the
choice every other edit in this shell gives him — *"why can't it just wait on
saving until I choose to save over the existing file or save as a new
file?"* — so the permanence statement now has **three** forms, one per
destination: [`permanence_statement`]`(false)` for a new file,
[`permanence_statement`]`(true)` for replacing the open file, and
[`permanence_statement_deferred`] for the destination that lands in the open
document and writes nothing, which is the default and the one he actually
asked for. The clause that does **not** change between the three is the full
rewrite and the impossibility of getting the content back; what changes is
what happens to the file he opened. What is new is that this shell's *ordinary*
save is incremental
and promises so on `file.save_copy`'s tooltip — which makes
[`single_revision_note`] carry more weight here than it did there: it is the
one place an operator is told that this write does **not** behave like the
save they already know.

## Conventions

[`crate::text`]'s, unchanged: sentence case and no trailing period on a
label, full sentences with punctuation for prose, an ellipsis on a control
that asks a question before acting.

One addition of this module's own: **`⚠` is the residual mark and the only
non-ASCII character used here.** It is measured drawable — `DEFECTS.md` D12
records the correction that established it — and
`crate::icons::glyphs::tests::every_glyph_the_catalog_draws_has_a_glyph`
sweeps this file with the rest of the catalog, so a decorative glyph added
later that the bundled fonts cannot draw fails a test rather than shipping
as a box. That matters more on this surface than on any other: a residual
line whose first character is a broken box reads as a rendering failure, and
an operator who has decided a surface is broken stops reading it.
