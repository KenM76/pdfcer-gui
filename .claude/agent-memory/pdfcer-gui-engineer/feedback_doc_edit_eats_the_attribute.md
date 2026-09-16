---
name: doc-edit-eats-the-attribute
description: Replacing lines from a doc comment up to the `fn` line deletes the #[test] / #[must_use] / #[cfg] sitting between them — the test then compiles, reads as present, and runs zero times
metadata:
  type: feedback
---

**A Rust item's attributes live BETWEEN its doc comment and its signature. So
a patch whose range is `[first /// line, fn line)` deletes them — silently,
and the file still compiles.**

**Why:** on **2026-09-16**, rewriting the doc of
`the_panels_sort_satisfies_the_engines_own_sorted_test` to say it now pins a
delegation rather than a copy, I anchored the replacement on the `fn` line and
substituted doc lines only. `#[test]` was inside the range. The function
remained, fully written, fully documented, perfectly compiled — and
**untestable**, because nothing marked it as a test. `cargo test` reported
`6 passed` before and `5 passed` after, and only the *count* said so.

★ **The same shape hits `#[must_use]`, `#[cfg(…)]`, `#[allow(…)]` and
`#[derive(…)]`.** Losing `#[must_use]` on a `pub const fn` returning a
sentence is invisible for ever. Losing `#[cfg(test)]` on a module changes what
ships. Losing a `#[derive]` fails loudly, which makes it the *least* dangerous
of the set — the dangerous ones are the attributes whose absence is legal.

**How to apply:** when a patch replaces a doc comment, end the range at the
**last `///` line**, never at the item. Concretely — assert the boundary
instead of assuming it:

    assert lines[b - 1] == "#[must_use]"   # or "    #[test]"
    lines[a:b-1] = new_doc.split(chr(10))

and then **re-run the test count**, because for a `#[test]` the count is the
only witness. An identical count after adding or editing a test is the finding,
not a coincidence.

Related: [[a-new-test-that-does-not-raise-the-count-did-not-run]] — the count
rule this is the mechanism for; that entry blamed nesting, and this is a second,
different way to produce the same zero. [[verify-the-result-not-the-diff]],
[[a-rewrite-of-a-cell-deletes-what-only-that-cell-held]].


---

## ★ Same edge, one item later, 2026-09-16 — **an INSERTION anchored in the
middle of a multi-section doc block splits that block and adopts its tail**

The entry above is about a replacement whose range was too long. This is an
insertion whose anchor was in the wrong place, and the damage is the mirror
image: nothing was deleted, and a heading moved owners.

`choice_row`'s doc comment is several `#` sections long. I seated a new helper
above the paragraph beginning *"`/Opt` entries may be `[export display]`
pairs"*, believing it was the start of a doc block. It was the **middle** of
one. The result compiled and read fine in a diff: the helper's own doc ended
where I put it, and `choice_row`'s `# /V stores the EXPORT value` heading —
the very section explaining the bug being fixed — became the *helper's*
documentation, with `choice_row` losing it.

⇒ **A `///` line is not evidence that a doc block starts there.** Anchor an
insertion on the **item keyword** of the thing above it (`fn`, `struct`, the
closing `}` of the previous body) or on a blank line at column 0, never on
prose. After inserting near a doc comment, print the ten lines above and below
the seam and read whose doc each heading now belongs to — the compiler has
no opinion about which item a doc comment describes.
