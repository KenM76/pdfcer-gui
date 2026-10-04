---
name: a-release-build-does-not-compile-the-tests
description: a green `cargo build --release` plus a passing drive says nothing about test code — a new struct field broke a test literal and cost a full gate run; run clippy --all-targets before launching gates
metadata:
  type: feedback
---

**Before launching `run-all.sh`, run `cargo clippy --workspace --all-targets`
(or the lib tests). A release build and a passing ui-verify drive never compile
`#[cfg(test)]` code.**

**Why:** adding a field to `SignDialog` broke the struct literal in
`dialogs/sign/tests.rs`. The release build and the driven check were green;
the gate run (about an hour) then failed on both clippies, and because no
`.rs` edit is allowed while gates read the tree, the one-line fix had to wait
for the doomed run to finish.

**How to apply:** order is fmt → clippy all-targets (fast once warm) → stage
→ gates in the background → lib tests. Also grep `tests.rs` beside any struct
gaining a field.
