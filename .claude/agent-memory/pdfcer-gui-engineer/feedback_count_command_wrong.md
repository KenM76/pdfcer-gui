---
name: a-count-command-can-be-wrong-not-just-its-quoted-answer
description: RESUME.md's own state table carried a check-count command that measured the wrong thing (434 vs 213) — re-measuring faithfully reproduced the wrong answer for months
metadata:
  type: feedback
---

**Re-running the command in a state table is not enough — audit what the
command counts.** `ui-verify --list | wc -l` had sat in `RESUME.md`'s "State,
as measured — re-measure, never quote" table for weeks. `--list` prints **two**
lines per check plus a header, so the command answers **434** where the answer
is **213**. The quoted value had drifted to 212 and nobody noticed, because
every re-measurement faithfully reproduced a number nobody compared against
reality.

**Why:** this project has been corrected on count drift eight times, and the
discipline that grew out of it — *never quote, always re-measure* — assumes the
command is a correct instrument. It defends against a stale number and is
completely blind to a wrong instrument. The header warning about drift sat four
lines above the drifting row. Fixed by counting check NAMES:
`ui-verify --list | grep -cE '^  [a-z0-9_]+$'`.

**How to apply:** the first time you run any command in a measured table, spend
one line checking its OUTPUT SHAPE — `head -6` before `wc -l`. Ask what one
unit of the thing being counted looks like in that output. A `wc -l` over
anything with a header, a blank line, or a multi-line record is a claim about
formatting, not about the subject. Same family as
[[a-proxy-condition-survives-one-correction]] and
[[a-check-that-cannot-fail-is-not-evidence]]: ask what the mechanism READS,
not whether it ran.

---

**Second instance, 2026-09-13, and it is the REPAIR that was mis-transcribed.**
The corrected command is `grep -cE '^  [a-z0-9_]+$'`. Written without the
trailing `$` it answers **224** where the answer is **222**, because `--list`
prints a *second* small table -- the `--exe` targets -- whose rows are also
two-space-indented lowercase words, so `pdfcer-gui` and `pdfcer-legacy` are
counted as checks. Three different numbers were in circulation (221 carried in
the table, 222 true, 224 from the loose pattern) and the instinct was to pick
one. **Diagnose instead: `--list | sed -n '1,12p'` and look for a second
table.**

* The general form: **a repair to a measurement command has an anchor, and the
anchor is the whole repair.** An end-anchor, a `--include`, a `-w`, a
`':!docs'` pathspec -- each is the part that makes the command mean what the
prose beside it says, and each is the part a re-typing drops. If a count comes
out slightly high, look for a second section of the same output rather than for
a new item.

---

**Third instance, 2026-09-14, and the mechanism is neither a header nor a
second table — it is the SHELL.** Counting this repository's Rust with
`git ls-files '*.rs' | xargs wc -l | tail -1` answers **239,849**. The answer
is **594,445**. `xargs` splits a 1,032-path argument list into several `wc`
invocations to stay under the Windows command-line limit, each one prints its
own `total`, and `tail -1` reads **the last batch only** — about 40% of the
truth, in exactly the shape an answer comes in. There is no warning, no partial
output, nothing on stderr; `wc` did its job perfectly, several times.

It was caught only because the figure contradicted a stale row in the same
document that said 576,229, and a *smaller* number where growth was expected is
a contradiction. Had the drift gone the other way it would have been quoted.

* **The general form, and it is a third distinct mechanism:** the first two
  instances were about what the command's INPUT looks like (two lines per
  record; a second table). This one is about what the shell does to the
  command BETWEEN the pipe stages. ⇒ **Any pipeline containing `xargs` and an
  aggregate is suspect** — `wc`, `sort -u | head`, `grep -c`, `du -c` — because
  `xargs` is licensed to run the command more than once and every aggregate
  then aggregates a fragment.
* **The fix is not a bigger `-n`**; it is to stop aggregating in the shell.
  A Python loop that opens each path and counts `b'\n'` cannot be batched, and
  it is four lines.

---

**Fourth instance, 2026-09-15, and the mechanism is the FILE, not the command:
no trailing newline.** A 128-name work list written with
`open(p,'w').write(NL.join(names))` has 127 newlines, so `wc -l` answers **127**.
The list then fed a chunked re-run whose `N` came from that count — one check
would have been dropped, silently, from a run whose entire purpose was to close a
coverage gap.

* `wc -l` counts newline CHARACTERS, not lines, and every generator that joins
  rather than terminates produces a file it under-counts by exactly one. So does
  every text editor that trims the final newline.
* **The cheap check is `tail -c 1 file | od -c`** — one command, unambiguous — or
  cross-count with `grep -c ""`, which counts the unterminated last line.
* ⇒ The family rule now covers four distinct mechanisms: the input's shape (two
  lines per record), a second section in the same output, the shell batching
  between pipe stages, and the file's final byte. **An off-by-one in a count is
  the least alarming-looking wrong answer there is**, which is why it is the one
  that reaches a runner.

**Fifth, same day, and this one had a CORRECT instrument standing beside it.**
To check a generated work list for stray CR bytes I ran
`od -c f | grep -c '\r'` and got **187** — alarming, on a 128-line file. The
truth was **0**: `tr -cd '\r' < f | wc -c` said zero, and a `grep -qx`
round-trip selected from the file correctly. I never did establish what the
backslash pattern matched after two layers of shell quoting, and that is the
point — **I did not need to.**

⇒ When a count is surprising, do not debug the count: **reach for a second,
simpler instrument that cannot have the same failure mode.** `tr -cd` counts
bytes and has no pattern language to get wrong. Ten seconds, and it settles the
question the escaping argument never would.

**And the corollary that actually costs time:** I had *already* been bitten by
this exact thing an hour earlier — a grep over wrapped log text answering 15
where the truth was 129. A pattern-matching count over text you did not format
is a claim about the formatter and the quoting, never only about the subject.

## ★ SIXTH, 2026-09-15 — a zero count that MEANT "no defect", from a pattern that matched nothing

Investigating why one dialog never drew, I counted frame boundaries between the
ribbon press and the dialog's first rect with `/canvas-place frames=/`. It
returned **0 for every trace**, including the three where the dialog demonstrably
drew. I read that as *"dialogs draw in the same frame as the press"* and wrote it
down as a measurement that killed the frame-timing hypothesis.

The real line is `canvas-place src=none want=none frames=34`. The pattern matched
nothing, anywhere. Matching on the line name alone gave 1, 1, 1 and 1 — the
latency is a constant one frame, which was the defect.

⇒ **The trap is that the wrong answer was the reassuring one.** A zero here does
not look like a broken instrument; it looks like *"no frame boundary, therefore
no timing problem"*. A count that arrives as evidence AGAINST a hypothesis
deserves the same falsification as one that arrives for it — more, because
nobody re-checks a number that closed a question.

**The tell, free and immediate:** the same command run with no filter should have
returned hundreds of per-frame lines. **Run the pattern against a case you know
the answer for before believing it against one you don't** — here, any trace at
all would have shown a non-zero total. Related:
[[a-check-that-cannot-fail-is-not-evidence]],
[[a-disproof-is-a-measurement-too-and-the-dead-hypothesis-was-the-truth]].

## ★ SEVENTH, 2026-09-21 — the output has NO total, so the eye supplies one

A commit here reported *"Tests: 31 ok, 0 failed"*. The workspace runs **4,628**
tests. **31 is the number of test BINARIES.** `cargo test --workspace` prints
one `test result:` line per target and **never prints a grand total**, so a
`| tail` shows the last few targets' tallies -- each a small, plausible,
correctly-formatted number -- and a reader assembles a total that was never
offered.

* This is the family's sixth distinct mechanism and the nastiest, because
  **nothing is wrong with the command**. There is no header to skip, no second
  table, no batching, no missing newline. The output is complete and correct;
  what is missing is the aggregate, and the mind fills it in.
* The tell is that the number is *too small to be interesting*. 31 tests for a
  600k-line workspace should have read as a filtered run, and did not, because
  it sat in a summary line nobody re-derives.
* **The instrument:** sum the lines rather than reading one.
  `cargo test --workspace 2>&1 | grep -E '^test result' | sed 's/.*ok\. \([0-9]*\) passed; \([0-9]*\) failed; \([0-9]*\) ignored.*/  /' | awk '{p+=$1;f+=$2;i+=$3} END{print p,f,i}'`

⇒ **Before quoting a tally from a multi-target tool, ask whether the tool prints
a total at all.** `cargo test`, `cargo clippy` per-crate, a chunked gate runner
and a per-file linter all answer no. A `tail` on any of them reports the last
chunk, and a `head` reports the first; neither reports the subject.

**And the sharpest part: the correct instrument was already written down.**
`RESUME.md`'s measured-state table prescribes exactly this -- sum the
`test result:` lines, then cross-check with
`cargo test --workspace -- --list | grep -cE ': test$'`, and *"passing plus
ignored must equal the cross-count"*. Done properly it gives 4,628 + 62 =
**4,690**, matching. The slip was not ignorance of the method; it was quoting a
figure in a commit message without going to the table that owns it. ⇒ **A
number in a commit message is subject to the same table as a number in a
document** -- a commit is read far more often than a roadmap row.

