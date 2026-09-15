---
name: a-tool-that-mutates-the-tree-before-stamping-it-reports-its-own-dirt
description: package-portable.py runs cargo update then reads git describe --dirty, so a build packaged from a provably clean tree is named -dirty by construction; and the re-package then diffs its changelog against the RETRACTED build, reporting 1 commit where 21 belonged
metadata:
  type: feedback
---

**When a guard reports a bad state, ask whether the tool that reads the guard
is the thing that created the state.** Two defects in one release, both in
`tools/package-portable.py`, both invisible because every individual fact each
one printed was true.

**Why:** measured 2026-09-11 cutting `v0.5.0-dev.20260911.1`.

**Defect 1 — the `-dirty` stamp the script earns for itself.** The standing
rule is *package from a clean tree, the last zip says `dirty`*. The tree WAS
clean at the commit. The script's **first act** is `cargo update -p
pdfcer-core -p pdfcer-render -p pdfcer-print`; the engine pin had moved, so
`Cargo.lock` changed; thirty seconds later the same script read `git describe
--dirty` and stamped:

    pdfcergui-20260911-0631-1eb1c7c-5e1ba78-dirty-d95f2bedd9c9

The only modification in the tree was one the script had written. **A tool
that mutates its subject before measuring it will report the mutation as the
subject's fault**, and the report is correct in every particular. The fix is
one of: stamp the revision *before* updating; commit the lock; or refuse with
*"the engine pin moved, commit Cargo.lock and re-run"*. The workaround —
commit the lock, re-package with `--no-update --no-build` — is not the fix.

**Defect 2, which the workaround then caused.** The script builds its
changelog by diffing against **the previous build it finds in the
destination**. Re-packaging over a retracted build makes that the retracted
build. The corrected package reported:

    Shell commits since the previous build (5e1ba78):
      7e439eb Engine pin ... (documentation only)

One commit, a pin bump — for a release carrying **21** shell commits and five
operator-visible landings. Nothing threw, nothing was corrupt, and the number
that should have been 21 was 1. ★ **A changelog that diffs against "the last
artefact" instead of "the last artefact the operator actually has" is
silently wrong for exactly as long as a retraction is in play.** Corrected by
hand against the build still in the *other* OneDrive slot, with the reason
written into the shipped `BUILD-INFO.txt` rather than left in a commit.

**How to apply:** when `package-portable.py` prints `-dirty`, run `git status`
**before** believing it — if the only change is `Cargo.lock`, the script did
it. Commit the lock, then re-package with `--no-update --no-build` **and
`--slot <the slot the retracted build went to>`**: the rotation replaces the
*older* slot, and the retracted build has just made that slot the newer one,
so the default would overwrite the last known-good fallback. Then check the
changelog block and correct its span by hand. Related:
[[feedback_publish_the_portable_zip_to_github_every_release]],
[[feedback_gh_release_create_tags_remotely_so_git_describe_goes_stale]].

## ★★★ 2026-09-15 — a function lifted OUT of a runner brings its side effects with it

To falsify a new `tally()` in `sweep-full.sh` I did the right thing — ran it
against the real log of the sweep it was written to catch, plus a clean control
and an empty file — by `sed`-ing the function out of the script and calling it
with `LOG=` pointed at each input. It reported correctly every time.

It also **appended its findings to every log it read.** `tally()` ends each
line with `| tee -a "$LOG"`, which is exactly right inside the runner and
destructive inside a harness: six stray lines went into the 2026-09-15 sweep
log — the evidence behind a defect entry and two `RESUME` claims — and two more
into a live re-run that was still being written by a background process.

⇒ **A falsification harness must never read the artefact in place.** `cp` to
scratch and point the harness at the copy, then prove it: measure the input’s
byte count before and after a run and require them equal. That check takes one
line and would have caught this on the first of four runs instead of the fourth.

**The general shape, which is not about `tee`:** code extracted from its
context keeps every side effect the context made safe. A logger, a cache write,
a lock file, a `tee` — all invisible when you are reading the function for what
it *computes*. Before lifting anything out to test it, read it once more asking
only *what does this WRITE?*

Related: [[a-checks-own-gesture-can-satisfy-the-condition-it-was-written-to-catch]],
[[never-drive-the-published-build]] — the same rule one level up.
