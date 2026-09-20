---
name: a-symbol-citation-is-worse-than-a-line-when-the-symbol-is-long
description: Replacing local line citations with symbol names is only an improvement while the symbols are small — eleven table rows collapsing onto one 187-line function loses the distinction the table exists to draw
metadata:
  type: feedback
---

"Cite by symbol, never by line" is right for a **moving external** tree and is
not automatically right inside this repository. Before converting a batch of
local `file.rs:N` citations, measure how many DISTINCT symbols they collapse
to. Where several rows of a table cite different blocks of one long function,
the symbol name is the same for all of them and the conversion silently erases
what each row was pointing at.

**Why:** measured on `FORMS_PARITY.md` — 46 local line citations resolved to
**24 distinct symbols**, and **11 of them landed in one function**,
`panels::properties::fieldedit::section`, which is 187 lines where the next
longest function in its own file is 46. The table's rows are per-feature and
the line ranges are the only thing distinguishing them. A blanket rewrite would
have made the document *less* precise while passing every gate and looking like
compliance with R5.

The measurement that decides it is cheap and is the one to run: resolve every
citation to its enclosing symbol, then count distinct symbols against total
citations. A ratio near 1:1 means convert; 46:24 means the code has a seam
before the document has a defect.

**How to apply:** treat a many-to-one collapse as a **code** finding, not a
documentation one — the fix is splitting the function, and until that happens
the line numbers are carrying real information. Resolve by indentation, not by
brace counting: `cargo fmt` is a gate here, so every item sits at column 0 and
every associated item at exactly one indent, whereas a hand-rolled brace
counter desyncs silently on a raw string and is confidently wrong below that
point. Two shapes the resolver must handle or it reports the item ABOVE the
one meant: a citation landing inside a doc comment belongs to the declaration
BELOW it, and a multi-line `#[…]` attribute between them is not a stopping
point. Related: [[a-drifted-citation-lands-on-a-real-unrelated-symbol]],
[[a-detectors-scope-is-a-claim]].
