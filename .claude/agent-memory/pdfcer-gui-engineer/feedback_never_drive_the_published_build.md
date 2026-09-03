---
name: never-drive-the-published-build
description: ui-verify's side effects persist into Ken's own OneDrive slot — drive target/release, or a scratch copy, never the published exe.
metadata:
  type: feedback
---

**Point `ui-verify` at `target/release/pdfcer-gui.exe`, never at
`C:\Users\Ken\OneDrive\pdfcer-gui1\pdfcer-gui.exe`.** To prove the packaged file
itself, copy it to `target/` first and drive it there.

**Why:** `package-portable.py` deliberately keeps the slot's `userdata/`,
because that state is Ken's. On 2026-08-24 the suite was pointed at the
published exe as a final check, and every side effect landed in his copy — it
was left with `wheel_paging = flip` switched on, which he had never asked for,
his page-display memory rewritten for `SW41177.pdf`, and his recent-files list
filled with test fixtures. It then reported a **failure** against a binary that
`cmp` proved byte-identical to the one that had just passed: the check was
measuring a layout the previous check had rewritten.

**How to apply:** after packaging, if you want to verify the artifact, `cp` it
to a scratch dir and run there — a portable build keeps state beside the exe,
so a copy is a clean slate, and that is also the only way to see what a *first*
launch looks like. If a published-build run ever reports a failure, run `cmp`
against `target/release` and check whether the `userdata/` timestamps fall
inside your run window before believing a word of it.

Related: [[feedback_always_publish_the_latest_build_to_onedrive]],
[[feedback_a_check_that_cannot_fail_is_not_evidence]].
