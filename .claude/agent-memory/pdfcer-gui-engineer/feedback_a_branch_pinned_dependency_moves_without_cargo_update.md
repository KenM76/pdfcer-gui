---
name: a-branch-pinned-dependency-moves-without-cargo-update
description: The engine pin in Cargo.lock can advance with no `cargo update`, because the manifest says `branch = "main"` and not `rev` — so "the pin is X" is a claim with a shelf life of minutes
metadata:
  type: project
---

**`Cargo.lock`'s engine revision is not stable between two commands in the same
session.** `crates/pdfcer-gui/Cargo.toml` takes all three engine crates as
`{ git = "file:///D:/Dev/pdfcer", branch = "main" }` — **a branch, with no
`rev`** — and the engine session commits to that branch several times an hour.
Cargo will re-resolve such a dependency opportunistically; it does not need
`cargo update` to do it.

**Why this matters more here than in a normal repo:** every number this project
quotes about a build is keyed to the pin. `BUILD-INFO.txt` names it, the release
asset's file name contains it, `FEATURES.md`'s revision header states it,
`tools/gates/engine-api-snapshot.txt` carries it as a baseline label, and every
"33 gates green" claim is implicitly *against that pin*.

**Measured, 2026-09-11.** Between packaging `d2465f5` at 19:53 and starting a
gate sweep at 20:06, `Cargo.lock` changed to `f8f9a26` at 20:04 with no
`cargo update` in the session's history. The release was unaffected only because
the binary was compiled and the folder stamped **before** the move — which is
luck, not a mechanism. Had it moved between the build and the stamp,
`BUILD-INFO.txt` would have named an engine the exe does not contain, and nothing
would have said so.

**How to apply:**
- **Re-read the lock immediately before quoting a pin**, in the same command that
  prints the claim. A pin read a few tool calls ago is hearsay.
- **A sweep is a measurement of a pin, not of a tree.** Record which one. If the
  lock moved mid-sweep, the sweep is void — clippy and the drift gate can
  straddle two different engines and both report green. This already produced one
  void sweep on 2026-09-11 (`bs3f5dbva`).
- If a drift gate fails right after an unexplained lock move, suspect the move,
  not the API. Check `git -C /d/Dev/pdfcer log --oneline -3` first.
- Related: [[a-still-broken-report-is-first-a-question-about-which-build-and-which-pin]],
  [[a-measured-limit-belongs-to-a-revision-not-a-design]],
  [[a-tool-that-mutates-the-tree-before-stamping-it-reports-its-own-dirt]].
