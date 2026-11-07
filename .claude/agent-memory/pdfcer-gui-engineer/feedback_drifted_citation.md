---
name: a-drifted-citation-lands-on-a-real-unrelated-symbol
description: A stale line citation into a branch-pinned tree does not dangle — it lands on a real function in the right file, so "does the line exist" is not a test of it
metadata:
  type: feedback
---

A line citation into a dependency pinned by **branch** rots with no local
event, and it rots into something worse than a dangling reference: upstream
*insertion* shifts a whole file uniformly rather than scrambling it, so the
drifted number still lands inside readable prose about a real function in the
right file. Verifying a citation by opening the line and checking that
something plausible is there **confirms every wrong one**.

**Why:** measured on one sweep of four living documents plus the forms parity
table — **66 of 92** engine citations had drifted and **not one dangled**. Two
pairs had come to name each *other's* type: two live line numbers, each
attached to the wrong symbol, each reading perfectly. `edit.rs` line 48704, written
against the `/AA` removal, now lands inside `EditSession::add_image`; line 36381, written against the `/RV` removal, lands in
`add_text_annotation_inner`; `lib.rs` line 804, written against the cmyk
diagnostics flag, lands on `panic_text`. The same applies to the frozen
archive at `D:\Dev\pdfce\crates\pdfce-gui` — a frozen tree is not
frozen-correct, because the
citations kept drifting until the freeze, so what froze was the error.

**How to apply:** the only test is *does this line name the symbol the
sentence is about* — resolve the enclosing symbol, do not eyeball the line.
Then replace the citation with that symbol and delete the number; R5 already
requires it. `tools/gates/check-engine-citation.sh` enforces the shape, but
see [[a-detectors-scope-is-a-claim]] — it is blind to a bare citation into a
small engine file, which is exactly the shape a reader will not re-check.
Related: [[a-quotation-i-wrote-myself-can-carry-a-line-number]],
[[a-verbatim-quotation-of-another-files-count-goes-stale-invisibly]].

**Same shape:** A tree that no longer changes does **not** make line citations into it safe.
The citations were written against an *in-flight* tree and kept drifting until
the freeze — so what froze was the error. Measure before trusting one.

**Why:** a commit in this repo recorded the opposite as settled — citations
into the archived `D:\Dev\pdfce\crates\pdfce-gui` "cannot rot at all… frozen
evidence" — and
the next sitting measured eleven of them against the archive. Seven were wrong
by +2 to +29 lines. Only two landed exactly. Every wrong one still landed
inside readable prose, because upstream *insertion* shifts a whole file in one
direction rather than scrambling it, so the number keeps pointing at real text
about a real function that has nothing to do with the claim.

**How to apply:** "the tree is archived" is an argument about the *future*, not
the past, and it never discharges a citation. Anchor by symbol regardless of
whether the target still moves. When you inherit a doc or commit asserting a
class of citation is safe, that assertion is a hypothesis with a cheap
measurement attached — run it. See [[a-measured-limit-belongs-to-a-revision-not-a-design]]
and [[an-injected-file-is-a-dated-snapshot]].
