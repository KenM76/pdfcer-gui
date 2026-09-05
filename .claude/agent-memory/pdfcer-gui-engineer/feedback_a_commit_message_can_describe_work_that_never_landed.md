---
name: a-commit-message-can-describe-work-that-never-landed
description: A `;`-separated shell chain lets a failed edit be followed by a successful commit, producing a truthful-looking message about a change that was never applied. Verify the artifact, not the exit code of the pipeline.
metadata:
  type: feedback
---

**Lead the rule:** before committing, **grep the artifact for the change you are
about to claim**. Not the script's exit code, not "the command ran" — the file.

## What happened, 2026-09-05

One Bash call did four things separated by `;`:

```sh
python -c "...prepend a block to RESUME.md..." ; head -5 RESUME.md ; wc -l RESUME.md ; git add RESUME.md && git commit -m "RESUME was two days stale…"
```

The Python raised `FileNotFoundError` and wrote nothing. `head` and `wc`
succeeded on the **unmodified** file. `git add` succeeded (adding an unchanged
path is not an error). `git commit` succeeded, sweeping in an unrelated staged
file — and landed a commit whose message described, in detail, a change that
**did not exist in the tree**.

Nothing in the output looked wrong except one traceback four lines above a
`git log --oneline -1` that printed the new commit's subject. The subject read
as confirmation.

**Why:** `;` is not `&&`. And `git add <unchanged-path>` is a no-op that
returns 0, so even an `&&` chain would not have caught it — only inspecting the
file would.

## The Windows detail that caused the failure, worth its own line

**Python's `open()` resolves `/d/temp/x` as DRIVE-RELATIVE** — current drive +
`\d\temp\x` — so with the cwd on `D:` it looked for `D:\d\temp\x`. Bash converts
`/d/temp/x` in *argv* (which is why `python /d/temp/script.py` works), but it
cannot convert a path that is a **string literal inside** the program. Inside
Python on Windows, always write `D:/temp/…`.

## How to apply

- **After any scripted edit, assert it landed** before doing anything that
  depends on it. One `grep -c '<the new text>' <file>` is the whole cost.
- Prefer `&&` over `;` when a later step is only meaningful if an earlier one
  worked — but do not treat `&&` as sufficient, because `git add` on an
  unchanged path succeeds.
- `git show --stat HEAD` after committing: if the file you wrote about is not in
  the stat, the message is a lie. That check caught this one.
- If it has already landed and is unpushed and seconds old, **amend** — the
  message then describes what is actually in the commit. A follow-up commit
  leaves a false statement permanently in history.

Related: [[feedback_a_check_that_cannot_fail_is_not_evidence]] and the
falsification rule *"assert the plant actually landed"* — this is that same
rule applied to ordinary edits rather than to planted defects. The project had
already learned it for plants and had not generalised it.
