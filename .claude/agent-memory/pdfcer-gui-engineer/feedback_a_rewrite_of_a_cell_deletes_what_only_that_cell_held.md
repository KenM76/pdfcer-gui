---
name: a-rewrite-of-a-cell-deletes-what-only-that-cell-held
description: Re-stamping RESUME.md's Request-channel cell deleted the one clause recording a finding that exists in exactly one uncontrolled copy — the cell said so itself, which is why the clause was there
metadata:
  type: feedback
---

**Rewriting a container is deleting everything in it that you did not
deliberately carry forward — and the cells most worth re-stamping are exactly
the ones holding notes that have no other home.**

**Why:** on 2026-09-14, re-measuring `RESUME.md`'s state table for a release,
the *Request channel* cell was rewritten with a fresh file count and a fresh
warning. The old cell had carried a clause recording `done_G013`'s §2a finding
— *a guard's coverage may be borrowed from a neighbouring function's current
shape, and a borrowed guard has no owner, so nobody is told when it is
returned*. That clause was in a state-table cell, of all places, **because the
finding lives in exactly one file, in
`D:\Dev\FeatureRequests\pdfce_FeatureRequests\`, a folder under no version
control**. The cell said so in its own words. The rewrite deleted it, and the
only reason it came back was reading the diff rather than the exit code.

Nothing could have caught it. `git diff` showed a cell replaced by a cell,
which is what a re-stamp looks like. No gate reads that folder. The clause's
own justification for existing — *there is no other copy* — is also the reason
its loss is silent.

**How to apply:**

- Before replacing any cell, block or section wholesale, read the OLD text for
  sentences that are not about the current measurement. A state cell that has
  acquired a ⚠ or a ★★★ clause has acquired it because something had nowhere
  else to go.
- `git diff` the re-stamp and read the **minus** lines specifically. The plus
  lines are what you meant to write; the minus lines are the whole risk.
- Better: when a finding's only copy is outside version control, the re-stamp
  is the prompt to **mirror it into the repository now** rather than to
  preserve the clause again next time. A note that survives by being copied
  forward by hand each release survives until the first tired session.

Sibling of [[never-git-checkout-to-undo-an-experiment]] (the same loss by a
different mechanism) and of
[[a-disclosure-has-a-subject-delete-it-when-the-subject-goes]], which is its
mirror image: that one is about failing to delete prose when its subject goes,
this one about deleting prose whose subject is still there.
