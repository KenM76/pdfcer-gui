---
name: doc-edit-eats-the-attribute
description: Replacing lines from a doc comment up to the fn line deletes the #[test] / #[must_use] / #[cfg] sitting between them — the test compiles, reads as present, runs zero times, and the test COUNT is the only witness
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

Related: [[doc-edit-eats-the-attribute]] — the count
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

## The detection rule — a new test that does not raise the count did not run

**After adding tests, check the total went up by the number you added.**

**Why:** 2026-09-08. Three `#[test]` functions were appended to the end of a
file by a script that inserted before the file's final `}` — a brace belonging
to a *function*, not the module. All three landed as **nested `fn`s inside
another `fn`**, where `#[test]` is not collected. They compiled. They produced
no failure. They ran **zero times**. The only signal was three
`function is never used` warnings inside a build that reported success, and I
nearly committed them on the grounds that the file compiled and the suite was
green.

**How to apply:**
- The number is in `cargo test`'s own output. `3,881 → 3,884` for three added
  tests is the whole check.
- ⚠ The same run reported `test result: ok` for the module they were *supposed*
  to be in — a green result for a module is not evidence that a specific test in
  it ran. Grep the run for the test's **name** if the count is ambiguous.
- Same family: a source-scanning test that reads the file it lives in matches
  its own assertion string, and a script that cuts Rust by counting `{`/`}` will
  swallow whatever follows a `{:?}` in a format string. Cut by column-0 anchors.

Related: [[a-check-that-cannot-fail-is-not-evidence]],
[[a-skip-is-not-red-so-a-check-can-stop-running-unnoticed]],
[[unit-tests-that-call-the-verb-cannot-see-the-chain-in-front-of-it]].
