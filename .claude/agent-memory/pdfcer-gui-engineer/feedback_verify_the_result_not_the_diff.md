---
name: verify-the-result-not-the-diff
description: A mass edit verified over its own diff — "every non-comment line byte-identical, brace balance unchanged" — passed on a source file that would not compile; two independent checks shared the blind spot because both were defined over removed lines rather than over the resulting file
metadata:
  type: feedback
---

**A check defined over *what an edit removed* cannot see a property of *what
the edit left behind*. Verify the artefact, not the diff.**

**Why:** 2026-09-15, a regex pass deleted 15,255 comment lines across 687 `.rs`
files. It shipped with a verification that sounded airtight, and every clause of
it was true:

- every non-comment line byte-identical to before;
- brace and paren balance unchanged;
- zero unterminated string literals;
- a Rust-aware lexer tracking string and raw-string state so nothing inside a
  literal could be mistaken for a comment.

`tools/ui-verify/src/checks/forms_spotlight.rs` satisfied all four **and did not
parse.** A doc comment held a Windows path whose `\f` escapes had been eaten by
some earlier patch script, and the pass split that one `///` line into three.
Lines two and three lost their `///` prefix, so they became code — but they are
*new* lines, so no existing non-comment line changed, and braces balanced
because the fragments happened to contain none.

★★★ **I then ran my own independent check and reproduced the identical blind
spot.** I filtered the removed lines for any that did not begin with `//` and
got zero, and reported "provably comment-only" to the operator. Same error,
arrived at separately: I measured the **removed set**, and the defect was in the
**resulting file**. Two verifications do not become one good verification when
they share a frame.

The compiler found it in ninety seconds. It was the only instrument in the room
whose input was the artefact.

**How to apply:**

- ★★★ For any mechanical edit at scale, the acceptance test is **build the
  result**, not describe the change. `cargo check --workspace --all-targets` is
  cheap, total, and has no blind spot that argument can talk it out of.
- When a verification is phrased as a list of properties of the *diff* —
  "N lines removed", "no code lines touched", "balance unchanged" — ask what
  property of the **output** each one actually implies. Usually: none.
- Deleting comments is not inert on Rust source. It can end a doc comment, merge
  `pub mod` runs so `reorder_modules` alphabetises them, and remove the line
  breaks that were forcing a literal to stay multi-line. All three happened in
  this pass. Snapshot module order and a comment-stripped hash per file, run
  `cargo fmt --all`, then diff the snapshots — that is how the three real
  movements were separated from 687 files of noise.
- The same run also re-taught [[a-command-judged-through-a-pipe-reports-the-pipes-exit-code]]:
  `cargo fmt … | tail -20; echo rc=$?` reported `fmt rc=0` over a hard parse
  error, and the background task's summary said "exit code 0". Read the log.

Siblings: [[a-detectors-scope-is-a-claim]] (the input set was wrong),
[[a-check-that-cannot-fail-is-not-evidence]] (the assertion could not go red),
and [[an-assertion-both-outcomes-satisfy-is-not-a-measurement-of-which-one-shipped]].
This one is the third axis: right assertion, real reach, measured on the
**wrong object**.
