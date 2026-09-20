---
name: update-engine-before-every-build
description: Always run cargo update on pdfcer-core/render/print before packaging a build — Ken's standing instruction, 2026-08-17
metadata:
  type: feedback
---

**Always `cargo update -p pdfcer-core -p pdfcer-render -p pdfcer-print` before
building a release.** Ken, 2026-08-17: *"always update core render and print
before building the latest."*

**Why:** the engine dependency is `git = "file:///D:/Dev/pdfcer", branch = "main"`,
so `Cargo.lock` pins a revision and only `cargo update` moves it. Ken works the
`D:\Dev\pdfcer` repo in a parallel session and it moves **fast** — it went 8, then
12, then 4, then 6 commits ahead of the locked revision within a single afternoon.
A build taken without updating silently ships an engine older than the repository
has.

This has already cost something concrete: a stale GitHub pin left the shell eight
commits behind `1e7a0be`, the fix that made `Separation`/`DeviceN`/`Lab`/`CalGray`/
`CalRGB` **images** decode instead of being dropped from the raster. Eighteen
pictures were missing from Ken's own file, and the *old* shell rendered it
correctly while the rebuild did not — the reverse of what anyone expects.

**How to apply:**

- `tools/package-portable.py` now does the update itself as a build step, so the
  normal path is automatic. Do not remove that step; `--no-update` exists for the
  rare case where an exact revision must be reproduced.
- If the update breaks the build or the tests, **report it and do not ship** —
  Ken's parallel session sometimes has uncommitted work in flight. A path
  dependency once failed to compile mid-rewrite of `redact.rs`; the `file://` +
  branch form takes committed history only, which is why it is that form.
- `BUILD-INFO.txt` records the revision actually linked, read from `Cargo.lock`
  rather than from the engine tree's HEAD. Those differ.

**★ The one exception, and the shape it actually takes (2026-08-26).** The
update can turn a test red *because it is working*: v0.14.0 added
`Settings::max_cmyk_buffer_bytes`, and this shell's
`every_setting_the_store_carries_has_a_control_in_this_window` immediately
failed because a setting the engine honours has no control in the Settings
window. That is the completeness gate doing its job on the first build after
the engine grew a setting — not a broken engine and not a broken shell.

When that happens in the middle of unrelated work: **hold the pin with
`cargo update --precise <old-sha>`, ship the unrelated work coherent and green
on the revision it was verified against, and write the bump up in `RESUME.md`
as the next session's first job** — with the ordered list of what taking it up
properly means. Do not appease the test to make the bump fit, and do not ship a
red tree. Then `package-portable.py --no-update`, or it re-resolves the lock
mid-package and undoes the hold.

Related: [[ui-verify-competes-for-the-machine]],
[[the-engine-session-runs-in-parallel-and-answers-within-the-hour]].
