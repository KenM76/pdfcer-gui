---
name: a-rename-can-blind-an-instrument-silently
description: A hard-coded external path turns a rename into a green check over an empty scan; derive the path from the manifest instead
metadata:
  type: feedback
---

**An instrument that hard-codes an external path goes blind when anything
renames, and it goes blind reporting success.** Derive the path from the
manifest the compiler actually uses.

**Why:** the 2026-09-03 rename to `pdfcer` broke four things at once and three
said nothing:

* `check-verb-coverage` printed **"PASS: all 0 uncalled verb(s)"** having read
  nothing — a *gate*, green, over an empty scan;
* `check-shipped-assets` SKIPPED, leaving 111 redistributed files unchecked for
  licensing;
* `package-portable.py` would have shipped a portable build with **no OCR
  models**, silently;
* `build.rs` reported no engine version — the only one whose own test caught it.

Every one was a literal like `ENGINE = Path("D:/Dev/pdfcer")` that the rename
dutifully updated to a directory nobody had created yet.

**How to apply:** `tools/engine_path.py` reads the `git = "file:///…"` URL out of
`crates/pdfcer-gui/Cargo.toml` — the same answer the compiler used — so there is
one claimant for *"where is the engine"*. Any new tool reaching outside the repo
uses it. And make the failure loud: `require()` exits rather than returning a
default, because the whole lesson is that a missing input must not read as an
empty result.

★ Two corollaries from the same day. **`pdfcer` contains `pdfce`**, so the
substitution is not idempotent and "did I get them all?" cannot be answered by
grepping the old name — only by the stem *not followed by* `r`. And a tripwire
must test the **real** condition: the shim gate first fired on the engine's
*directory* appearing, but the rename is two Passes and the clone comes first.
Related: [[feedback_a_check_that_cannot_fail_is_not_evidence]],
[[the-project-is-pdfcer-gui-since-2026-09-03]].
