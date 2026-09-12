---
name: a-document-that-quotes-its-measurement-command-can-be-broken-by-it
description: FEATURES.md's Source row quoted `xargs -d` with its delimiter inline; the newline went into the file literally and had been splitting that table row into two malformed rows for six days, invisibly in an editor.
metadata:
  type: feedback
---

**Never write a command's delimiter verbatim into a Markdown table cell.**
Describe it in words.

**Why:** 2026-09-12. `FEATURES.md`'s **Source** row proudly quoted the method
that produced its own number — `git ls-files '*.rs' | xargs -d '<newline>' cat
| wc -l` — and the newline was written as a **real newline**. That ends the
table row mid-sentence; Markdown renders the remainder as a fresh malformed
row starting with a stray quote. It had been like that since 2026-09-06, in
the ship-to-operator document, through four releases.

⇒ Two compounding reasons nobody saw it: **it is invisible in a text editor**
(the cell just looks wrapped), and **every gate in this repo reads the file as
text, not as rendered Markdown**. Same oracle rule this project already holds
for layout defects — the only instrument that can see it is a renderer, the
way the only instrument for a clipping defect is a screenshot.

**How to apply:** when a document cites the command that measured it — which
this project does deliberately, because *re-measure, never quote* — spell any
whitespace delimiter in prose ("piped through `xargs` with a newline
delimiter"). More generally, after editing a long single-line table cell,
check the row count of the table, not just that the text is present. Related:
[[a-verbatim-quotation-of-another-files-count-goes-stale-invisibly]].
