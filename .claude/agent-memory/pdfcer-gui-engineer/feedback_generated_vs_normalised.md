---
name: a-gate-comparing-generated-bytes-to-a-committed-file-measures-who-wrote-it-last
description: When a checked-in file is produced by a generator and normalised by git on commit, a byte-identical comparison passes only on the machine where the generator wrote last — green locally, red on every clone, and invisible to git diff
metadata:
  type: feedback
---

A gate of the shape *"regenerate it and require the committed copy to be
byte-identical"* is only honest if **nothing transforms the bytes between the
generator and the commit.** If `.gitattributes` normalises the file — `text`,
`eol=lf`, a clean filter — then the blob can never equal the generator's output,
and what the gate reports is **which of the two last wrote the working copy**,
not whether the file is stale.

**Why:** `cargo-about` embeds each crate's LICENSE verbatim and one dependency's
Apache-2.0 text is stored with CRLF. `*.md text eol=lf` strips exactly those
carriage returns at `git add`, so `THIRD_PARTY_LICENSES.md` in git has never
held them. The gate passed while the working copy still carried the generator's
own output, and went red the moment anything rewrote that file from the
committed bytes — which is also the state of **every fresh clone and every
checkout**. It failed, was regenerated, went green, and failed again
identically a day later with the file untouched in git throughout.

**The part that makes it hard to see:** git shows nothing in either state. The
same `text` attribute normalises the worktree side before comparing, so
`git diff` is empty whichever form is on disk. This gate was the only
instrument in the repository that could observe the condition — and it was
reporting it as a finding about staleness.

**How to apply:** before writing or trusting a regenerate-and-compare check,
run `git check-attr text eol -- <file>`. If anything is set, normalise the same
thing on both sides of the comparison and say so in the header, or the check is
not clone-stable. Do **not** exempt the file from `text` to make the bytes
survive — that puts mixed line endings in a shipped artefact to satisfy a diff.
The general form: **a comparison is only as stable as the least stable
transform between its two sides**, and a green result whose cause is "this
machine ran the tool recently" is not evidence about the repository at all.

⇒ Corollary for the fix's own headline: when a gate prescribes a remedy,
check that the remedy survives a commit. This one printed
`cargo about generate …`, which restored green locally and could not change
anything git could see, so it would have been run for ever.

Related: [[git-status-is-not-a-content-oracle]],
[[a-gate-whose-input-set-comes-from-git-measures-the-index]],
[[a-sweep-keyed-on-the-symptom-collects-the-healthy-too]],
[[a-check-that-cannot-fail-is-not-evidence]].
