# `text::panels::comments` — every string the Comments panel shows

The copy for [`crate::panels::comments`], which lists **every annotation
in the document** — the comment list a reviewer works through. One module
per panel surface, as [`super`]'s header lays out; `crate::panels::comments`
is the sole consumer.

## Most of this is salvaged verbatim, and the doc comments came with it

Nine of the entries below came across from the old shell's `ui_text.rs`
**with their doc comments**, because in this project a doc comment on a
string is usually the record of the defect the wording was changed to fix.
⚠ **Fresh words re-derive a decision already paid for, without access to
the evidence that bought it.**

The two that carry the most reasoning:

- [`comments_all_without_notes`] exists because pdfcer's own markup
  authoring cannot write `/Contents` on a geometric shape —
  `pdfcer_core::annot_author::MarkupSpec` has no text-bearing variant on
  purpose — so a document whose annotations pdfcer drew shows a column of
  identical "no note" captions. `docs/core-api/03-capabilities.md` §3.4
  makes saying so **mandatory copy**: *"A bare 'No note text' column reads
  as data loss."*
- [`comments_none`] names what is **excluded**, because a document that is
  nothing but form fields would otherwise show an empty comment list and
  look broken.

## What is new here, and why each one exists

Six entries have no ancestor in the old shell. Every one of them is a
**disclosure** that `docs/core-api/03-capabilities.md` §3.4 ("★ what the UI
must disclose") or §3.5 ("Traps") asks for by name, and each says which:

| Entry | Commissioned by |
|---|---|
| [`comments_excluded`] | this panel's own decision to state its filter in numbers rather than in the abstract — see [`crate::panels::comments`]' header |
| [`comment_row_hidden`] | §3.4.5 — *"A Comments panel that silently omits it is hiding document content; list it and mark it hidden."* |
| [`comment_row_appearance_unresolved`] | §3.4.4 — pdfcer *"displays nothing and does not guess"*, and the governing default is explicitly a reasoned guess, i.e. an inference, i.e. rule 4 applies |
| [`comment_row_description_caption`] | §3.5 — *"`/Contents` is dual-purpose … a UI labelling this 'comment' is right for markup and wrong for a Link"* |
| [`comment_row_is_group_member`] | §3.5 — the §12.5.6.2 group-attribute rule is **deliberately not applied** by core, so what this panel shows is the raw dictionary value and a conforming reader shows something else |
| [`comment_row_ce_dimension_heading`] / [`comment_row_ce_dimension_no_note`] | project rule 15 — a **ce dimension** is a `/Line` annotation, and a row that called it "Line" would be true about the file and useless to the operator |

## ★ Rule 15 is enforced by a test in this module

*"Never write a bare 'dimension".* **ce dimensions** are the ones pdfcer
authors (`/Line` + `/IT /LineDimension` + a `/PieceInfo` sidecar); **pdf
dimensions** are CAD-exported page content. They have opposite properties
and the ambiguity has already sent one investigation down the wrong path,
so [`tests::no_string_here_says_a_bare_dimension`] sweeps every entry
rather than trusting review — this is a *catalog*, which is exactly the
kind of file where a bare noun slips in during a late reword.

## Conventions, restated from [`crate::text`] because they bind here

- **Sentence case, no trailing period on labels; full sentences with
  punctuation for prose.**
- **Never state a capability the build does not have.**

### ★★★ A SENTENCE ABOUT WHAT THE BUILD CANNOT DO HAS A SHELF LIFE

This panel once carried, as a reasoned decision, *"this build's panel has
no Delete, because `Action` carries no variant that could delete an
annotation."* ⚠ **The reason was correct, was written down, and stopped
being true without anything in this file changing.**
`AnnotAction::Delete { page, id }` exists, `crate::app::actions::annots::delete`
reaches `EditSession::delete_annotation` through it, and the canvas Delete
key and the Format tab both use it — leaving this panel, the reviewer's own
work list, as the one surface that could not do the plainest thing on it.

The operator's standard for this panel is *"the review features should
look and act the same as they do in Acrobat Reader"*, and Acrobat lets a
reviewer delete their own comment.

⇒ [`comment_row_delete`] and [`comment_row_delete_tooltip`] are below, and
the rule this cost is: **where a claim about a capability can be an
ASSERTION, it must be one.** Here it is
`crate::panels::comments::tests::the_delete_control_reaches_the_engine`,
which goes red the day the wiring stops being real — which prose cannot.
