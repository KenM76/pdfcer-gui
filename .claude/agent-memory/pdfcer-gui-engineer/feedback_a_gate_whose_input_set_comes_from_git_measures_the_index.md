---
name: a-gate-whose-input-set-comes-from-git-measures-the-index
description: A gate built on git grep / git ls-files is green before the commit and red after it with nothing changed; pass --untracked or scan the tree
metadata:
  type: feedback
---

A check whose **input set** is derived from git state is measuring the index, not
the working tree. It will report clean on a violation that is sitting in a file on
disk, and go red on the identical content the moment `git add` runs.

**Why:** on 2026-09-13 the release pre-flight ran every gate and printed
**41 passed / 0 failed**. The commit added `DESIGNS.md` and `DOC_DRIFT.md`. Thirty
minutes later `package-portable.py`'s own pre-flight failed the **same tree** on two
lines that had been in those two files the whole time. Nothing about the content
changed between the two runs.

That was the **second** occurrence. The gate's own header already described the
disease — it had been written after an untracked evidence file did exactly this —
and the fix made at the time was to exclude one directory. **Treating the instance
and leaving the mechanism bought one day.**

**How to apply:**

- `git grep --untracked` and `git ls-files --others --exclude-standard` close it;
  plain `git grep` and plain `git ls-files` do not. Prefer `find` when the question
  is genuinely *what is on disk*.
- The tell is the **timing**, not the content: a gate that flips at a commit
  boundary without an edit is this, every time. Do not go looking for what the
  commit changed in the flagged file — it changed nothing.
- A gate run as a **release pre-flight** is being asked what the commit WILL
  contain. The index structurally cannot answer that question, so a pre-flight
  built on it is reassurance rather than evidence — see
  [[feedback_a_check_that_cannot_fail_is_not_evidence]].
- Falsify the repair by planting the violation in an **untracked** file. The old
  version cannot see it at all, which is the whole point and is invisible to any
  amount of reading.
- Audit the siblings rather than one gate. Every checker in `tools/gates/` that
  reaches for git has the same hole.
