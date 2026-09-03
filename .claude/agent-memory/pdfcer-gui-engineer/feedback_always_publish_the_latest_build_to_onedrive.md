---
name: always-publish-the-latest-build-to-onedrive
description: The latest build always goes to OneDrive — verified or not. Ken's slots are the safety net, so "not driven yet" is never a reason to withhold it.
metadata:
  type: feedback
---

**Every time you produce a build worth keeping, package it and mirror it to
Ken's OneDrive, alternating between the `pdfcer-gui1` and `pdfcer-gui2` slots.**

## ★★★ AND "WHEN YOU ARE DONE WORK" IS ITSELF THE TRIGGER — 2026-08-29

Ken, in his own words: ***"always release the latest version when you are done
work."***

He said it after a session in which I **held a build back** — the work was
committed and green, and I reasoned that the previous slot already carried the
headline fix and that each OneDrive mirror costs the machine ~27,000 kernel
handles, so a second publish an hour later was not worth it. I wrote that
reasoning into `CONTINUE.md` as a considered decision.

**It is not what he wants.** The rule is not *publish when there is something
notable* — that is `package-portable.py`'s own header talking about handle cost,
and it is a real cost that he has now weighed and overruled. The rule is:

> **finishing work is the trigger.** If there is a commit that is not in a slot,
> package it.

⇒ Do not perform the cost-benefit. He has done it. The fallback slot makes a
bad build cheap; a build sitting only in `target/release` is worth nothing to
him at all.

★ It composes with the two rules already here rather than replacing them: still
`cargo update` the engine first, still read both slots back by date afterwards,
and still say in the report what has **not** been verified.

## ★★★ AND "IT HAS NOT BEEN VERIFIED" IS NOT A REASON TO WITHHOLD IT

Ken, 2026-08-21, correcting exactly that:

> *"no it doesn't matter if it has been checked or not. I always want the
> latest build there."*

He said it after a session in which a release was deliberately held back —
the driven suite could not run because he was at the keyboard, and the build
carried an engine bump touching the compositing path. The caution was
defensible and it was **not what he wants**, and the reason it is not is
already built into the tool: **the other slot holds the previous build.** He
has a fallback by construction, so the cost of a bad build is a folder swap,
while the cost of withholding is that he does not have the work at all.

So: **package and publish, and say in the report what has not been checked.**
The disclosure belongs in the report and in `BUILD-INFO.txt` (`--note`), never
in a decision to hold the build back.

★ This does not relax R1. Driven verification is still what "done" means and
still gets run — it is a gate on *claiming a feature works*, not a gate on
*putting the binary where he can reach it*. Those were being conflated.

```bash
python tools/package-portable.py
```

That one command already does all of it — it builds the portable folder under
`D:\builds\`, mirrors it to `C:\Users\Ken\OneDrive\pdfcer-gui<n>`, and **picks
the older of the two slots automatically**, so the alternation is a property of
the tool rather than something to track by hand. It also preserves the
`userdata/` folder already in the target slot, so settings survive a swap.

**Why:** stated by Ken on 2026-08-19. OneDrive is how he actually gets the
build — he runs it from there, on this machine and others. A build that exists
only in `target/release/` or `D:\builds\` has not reached him. The alternation
is the point: the previous build stays intact in the other slot, so if the new
one misbehaves there is always a working one beside it to fall back to and to
compare against. That is the same fallback property the whole project rests on
(`D:\Dev\pdfcer\` keeps shipping while the rebuild happens), applied to the
day-to-day.

**How to apply:** run it at the end of any session that landed working changes,
and after any fix Ken might want to try immediately. Do not ask first — it
writes only to `D:\builds\` and the OneDrive slot, never to a repository. Say
in the report which slot it went to and which one holds the previous build, so
he knows which is which without opening either.

Two things that make the report useful rather than noise:

- **name the slot**, not just "packaged" — `pdfcer-gui2`, and note that
  `pdfcer-gui1` holds the previous one.
- if the build is one he asked for specifically, say what changed in it, since
  the slot name carries no version information.

**★★ ALWAYS read the build stamp out of BOTH slots after packaging, and do not
trust the tool's own report.**

```bash
for d in pdfcer-gui1 pdfcer-gui2; do
  printf "%s: " "$d"; grep -m1 "^Built:" "C:/Users/Ken/OneDrive/$d/BUILD-INFO.txt"
done
```

**Why:** the mirror destroyed Ken's fallback build **twice on 2026-08-20**, and
the second time was after a fix. First it cleared the slot before copying;
then, repaired to stage-then-clear-then-swap, it failed identically — because
`shutil.rmtree` is itself non-atomic and a lock on one file leaves everything
already removed removed. Both times the tool printed a message asserting
nothing had been replaced, and both times that was false. `pdfcer-gui1` was
restored by hand from `D:\builds\` on both occasions.

The lock is **OneDrive's own sync client**, which is permanent on a synced
folder. The tool now never deletes in place — copy to `.slot-incoming`,
`os.rename` the slot aside, `os.rename` the staging in, then delete — because a
failed directory rename moves nothing. Full finding in
`D:\dev\rag\rust\`.

The two-date check is the only reason either failure was noticed. It costs two
lines and it is not optional.

**★★★ AND ON 2026-09-03 IT CAUGHT A THIRD FAILURE, OF A NEW KIND: THE SLOTS
THEMSELVES HAD BEEN RENAMED OUT FROM UNDER THE FALLBACK.**

The project rename moved the packager's targets to `pdfcer-gui1` / `pdfcer-gui2`.
The previous builds were in `pdfceGUI1` / `pdfceGUI2`. So the first package of
the day wrote into an **empty pair of slots**, and the tool printed:

```
  pdfcer-gui1  no readable build  <- replacing this one
  pdfcer-gui2  no readable build
  ...
  (replaced the older slot; pdfcer-gui2 still holds the previous build)
```

The last line is **unconditional prose** and it was false. The `no readable
build` two lines above is the honest half, and it is easy to read as "first run
after a rename, fine" rather than as "your fallback does not exist".

Repaired by copying `pdfceGUI2` into `pdfcer-gui2` before re-packaging, so the
two-slot property holds again. `pdfceGUI1`, `pdfceGUI2` and two
`.pdfceGUI*-outgoing` staging folders are now orphans in OneDrive.

⇒ Same class as the two above and worth stating as the general rule:
**the tool's own report is not evidence about the tool's own effect.** The
two-date read-back has now caught three distinct failures of one script.

Related: [[feedback_update_engine_before_every_build]] — `cargo update -p
pdfcer-core -p pdfcer-render -p pdfcer-print` comes first, or the package carries
a stale engine.

**★★★ AND THE PACKAGED BINARY IS NOT THE BINARY THE TESTS RAN AGAINST.**

`package-portable.py` runs `cargo update` on the three engine crates **itself**,
before it builds. On 2026-08-29 the engine moved `6624e18` → `97d445f` between
the last green test run and the packaged exe, and the script said so plainly:

> *the engine MOVED and `--verify` was not passed, so nothing has been tested
> against the revision this build will link.*

It is a well-written warning in the middle of a wall of output and it is easy to
read past. **After every publish: re-run the suite and the gates against the new
lock**, or pass `--verify` and let the script do it. On that occasion everything
was green and the revision was docs-only, but that was luck rather than method —
and the `Cargo.lock` change has to be committed either way, or the next session
starts on a tree that says "dirty" for no visible reason.

⇒ Related to the two-slot date check above, and the same class: **the tool's own
report is not evidence about the tool's own effect.**
