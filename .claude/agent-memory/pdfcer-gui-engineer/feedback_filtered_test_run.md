---
name: filtered-test-run-hides-registry-tests
description: A filtered `cargo test <name>` run skips the registry-wide tests (ledger counts, handler-token uniqueness, built_in.ron) that every new command breaks
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
