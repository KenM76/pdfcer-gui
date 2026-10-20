---
name: filtered-test-run-hides-registry-tests
description: run-all.sh runs no unit tests; a filtered cargo test skips registry-wide ones. Pin moves and command additions need the unfiltered workspace run before commit
metadata:
  type: feedback
---

Adding a command breaks four tests that live far from the feature: the
ledger's registry count, the icon-coverage count, handler-token uniqueness,
and `built_in.ron` staleness. A filtered run (`cargo test -p pdfcer-gui --lib
export`) never reaches them, and `run-all.sh` skips tests, so the Crop commit
shipped with all four red and was found one commit later.

**Why:** the gates are green and the filtered run is green, so two green
signals cover a red suite.

**How to apply:** before committing any change that registers a command,
run the unfiltered `cargo test -p pdfcer-gui --lib` (about 15 s once built).
After a new command, regenerate the RON with
`cargo test -p pdfcer-gui rewrite_built_in_ron -- --ignored` and bump both
ledger counts. See [[feedback_skip_red_check_stop]].

**Pin moves too.** The v0.88.0 pin commit went in on 86/86 gates without
`cargo test --workspace`, and the engine had moved a calibration test
(`reflow::tests::the_index_is_one_the_engine_accepts_on_a_right_aligned_fixture`).
A pin move changes engine behaviour under every test, so run
`CARGO_BUILD_JOBS=2 cargo test --workspace` before that commit.

**Do not isolate a cause with `git stash -u`.** It fails on the locked
`.claude/agent-memory` folders, leaves the tree unreset, and then cannot pop.
Reason from what the failing assertion reads instead, or use a worktree.
