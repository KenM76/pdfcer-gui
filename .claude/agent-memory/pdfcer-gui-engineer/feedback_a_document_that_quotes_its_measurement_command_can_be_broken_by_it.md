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

---

## ★★ Second instance, 2026-09-14 — **a literal `|` inside an INLINE CODE SPAN
inside a table cell still invents a column**

Same family, one step subtler. The first instance was a literal newline. This
one is a pipe, and the trap is that **the backticks do not protect it**:

    | Unit tests | ... cross-counted by `cargo test --workspace -- --list | grep -cE ': test$'` = 4,361 ... |

A Markdown renderer splits a table row on `|` **before** it parses inline code,
so that cell becomes two and everything past the header's last boundary is
**discarded**. The row still looks complete in an editor and in `git diff`; the
only thing that sees it is a renderer or a checker that counts boundaries.

`tools/gates/check-doc-markup.py` is the sole witness here and it said so
exactly: *“this row has 4 cell boundaries where its header has 3 — a renderer
DISCARDS everything past boundary 3.”*

**How to apply:**

- **In a table cell, DESCRIBE the pipeline instead of quoting it** — *“list the
  tests and count the lines ending `: test`”* — and put the verbatim command
  in a fenced block outside the table, where nothing is escaped.
- If the command must be inline, escape it as `\|`. Prefer describing it: an
  escaped pipe inside code renders as `\|`, which is then a command that does
  not work if anyone copies it.
- ⇒ **The general rule both instances share: a table cell is the one place in
  Markdown where a command's own syntax is also the document's syntax.** Any
  cell containing `|`, a newline, or a leading `-` is a hazard, and the cells
  most likely to contain one are exactly the cells that quote how the number
  beside them was measured.
