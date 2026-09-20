---
name: a-suite-that-shares-state-measures-the-order-it-ran-in
description: Driven checks sharing one profile pass by inheriting each other's state; isolating them makes long-green checks fail, and those failures look exactly like regressions
metadata:
  type: feedback
---

**A driven suite that shares persistent state measures the order it ran in, and
the contamination is invisible in the failing check's own report.**

**Why:** `pdfcer-gui.exe` is portable — it keeps `userdata/` **beside the exe** —
and `tools/ui-verify` launched every check from one copy. That was harmless only
while the application discarded the stored mode on every launch. The moment that
was fixed (2026-09-06), a check that clicked the Edit segment left
`mode: Some("edit")` on disk and **every later check started in Edit**.

It cost an investigation the same afternoon. `a_link_goes_to_the_page_it_names`
reported that clicking a link *"produced nothing"* — no `link-click`, no
`page-links`, the whole link layer absent — which reads exactly like a
regression in the press ladder. It was not: in Edit a click **selects** a link
rather than following it, deliberately, so the hit test is never called. On a
fresh profile the same binary reaches the link and fails for a completely
different, real reason.

⇒ **The shared profile was hiding a real defect behind an articulate wrong one.**

**How to apply:**

- `tools/ui-verify/src/sandbox.rs` gives every check its own hard-linked copy of
  the binary. `--shared-profile` restores the old behaviour with a warning.
  Keep it that way; a suite is not a suite if its checks can talk to each other.
- ⚠ **Expect isolation to turn long-green checks red, and expect each one to
  look like a regression.** `panels_float_close_and_dock` was the first:
  `view.panel_layers` is only in Edit's default dock (Read mounts five panels,
  Review seven, Edit thirteen), the check never said which mode it wanted, and
  it had been passing on a leaked one for months.
  ⇒ **A check that does not state its preconditions is not passing — it is
  agreeing with whatever ran before it.**
- ★ **The cheapest way to tell "we broke it" from "we can finally see it": run
  the same check against a binary from before the change it is blamed on.** Two
  runs, one comparison, no code read. It settled three separate accusations in
  one afternoon — all three were pre-existing.
- The same trap bites outside the harness: two `pdfcer-gui.exe` instances alive
  at once contend on one portable profile and silently invalidate a
  measurement. **A profile-backed measurement needs a profile nobody else
  holds.**

See [[never-drive-the-published-build]] and
[[a-check-that-cannot-fail-is-not-evidence]].
