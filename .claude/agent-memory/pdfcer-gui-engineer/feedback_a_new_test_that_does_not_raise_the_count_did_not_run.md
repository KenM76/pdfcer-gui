---
name: a-new-test-that-does-not-raise-the-count-did-not-run
description: Compare the test total before and after adding tests — it costs one number and catches tests that compiled but were never collected
metadata:
  type: feedback
---

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
