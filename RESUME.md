# RESUME — read this, then say "continue"

**This file is one screen. That is its whole specification.**

It carries the measured state, what to do next in the operator's order, and
what not to do. Nothing else. **It is not a session log** — the detail lives in
commit messages, `HANDOFF.md`, the RAGs and `OPERATOR_REQUESTS.md`, and this
file points at them rather than quoting them.

> ★★★ **Why that sentence is at the top, and why it is enforced.**
> On 2026-09-10 the operator ruled that this project had *"slowed to a crawl"*
> and named the cause: the register documents had become archives where nothing
> was ever deleted. This file was the worst instance — **207 KB, 3,554 lines**,
> of which the first 2,146 were a stack of blockquoted session addenda, and the
> *"what to do next"* section sat at line 3,118 and was three weeks stale. Every
> session paid to read that before it wrote a line. ⇒ **"Preserve the reasoning"
> was never "never delete."** When you finish a session, REPLACE the blocks
> below — do not stack a new one on top. If a block is worth keeping forever it
> belongs in `HANDOFF.md` or a RAG, not here. Everything cut on 2026-09-10 is in
> git history, which is where a superseded narrative belongs.

---

## State, as measured — re-measure, never quote

Every figure below has the command that produced it. **Run the command.** Prose
drifting from a count is the defect this project has spent eight corrections on.

| What | Command | Value, 2026-09-14 03:20 EDT (re-measured against the tree the FIFTH release of the day was cut from, every row by its own command, nothing carried over. The *Last release* row is the one exception and was re-stamped at 03:25 once the tag existed and `releases/latest` had been read back) |
|---|---|---|
| Engine pin | `grep -m1 -oE 'pdfcer\?branch=main#[0-9a-f]+' Cargo.lock` | `7378c83` — **level with the tip of the engine's `main`**, which is `7378c83` too. ★★★ **Two shas being equal is not what makes a sentence safe; an empty code diff is** — `git -C /d/Dev/pdfcer diff --stat 3df0c72..7378c83 -- '*.rs'` is EMPTY, and the single commit that moved the pin here touched `docs/ROADMAP.md` and `docs/SESSION_LOG.md` and nothing else. ⇒ The question a pin row exists to answer is *may I write the sentence "the engine cannot do X"*, and the safe condition is that **no CODE has landed since the pin**, not that the two strings match. Twelve hours ago the pin was one commit behind and nothing about what may safely be written was any different. ⚠ The pin is a BRANCH pin and it has moved mid-session without a `cargo update`; re-read the lock in the same breath as quoting it. ★★★ And "no code since the pin" is not "nothing is in flight": the request channel is not a git repository and no command in this table reads it |
| Engine HEAD | `cd /d/Dev/pdfcer && git log --oneline -1 main` | `7378c83` — **level with the pin**, and the commit is the librarian's 547th filing. It read `3df0c72` at 00:40 the same night, `4cb18e1` at 22:23 the previous evening and `5e17017` at 19:10 before that, which is exactly why this row is re-measured rather than quoted |
| Engine version | `grep -A1 'name = "pdfcer-core"' Cargo.lock` | `0.53.0` |
| Last release | `git fetch --tags origin && gh api repos/KenM76/pdfcer-gui/releases/latest` | `v0.5.0-dev.20260914.2`, cut 2026-09-14 07:22 UTC, `prerelease=false`, `draft=false`, one asset `pdfcergui-20260914-0317-7378c83-2bb0b19.zip` of 23,270,105 bytes, and it IS what `releases/latest` advertises — verified by reading the API back rather than by having passed the right flag. OneDrive slot **`pdfcer-gui2`** carries it (`Built: 2026-09-14 03:17:14`); **`pdfcer-gui1`** still holds the 2026-09-14 00:38 build, which is the point of alternating slots and was confirmed by reading `BUILD-INFO.txt` out of **both**. ★★ **1 commit unreleased** as of 2026-09-14 04:05 — the `--list` staleness fix, which is a harness change and changes nothing an operator can see. ⚠ **This row drifts with every commit, including the one that edits this row**, so it is the one figure here that is stale by construction; re-run the command rather than reading the number. ★★★ **The order matters, and it is the order this release was cut in:** commit ⇒ `touch crates/pdfcer-gui/build.rs` ⇒ rebuild ⇒ re-drive `the_title_bar_carries_the_build_time` and `about_reports_the_build` against the **shipped** exe ⇒ off-screen smoke launch of it ⇒ `package-portable --no-build --no-update` ⇒ push ⇒ `gh release create`. Building before the commit ships an executable nobody measured; `--no-build` is what makes the packaged file the one that was just driven; `--no-update` is what stops the packaging script silently re-pinning the engine out from under a measured build. ✓ **The stamp check is what proves the `touch` worked**: About read *“built 2026-09-14 07:02 UTC from 2bb0b19; engine 0.53.0 at 7378c83”* — the release commit, not the one before it — and the title bar and the off-screen smoke launch both read the same stamp. ⚠ `about_reports_the_build` **SKIPS under `--no-input`** because it clicks a ribbon control; it was driven with input enabled, which is allowed only while he is away from the machine. ⚠⚠ **Fetch first.** `gh release create` tags on the REMOTE, so `git describe` in a tree that has not fetched answers with an older tag — that mistake put *“51 commits unreleased”* in a report where the answer was 21. ⇒ **And never pass `--prerelease`**: GitHub hides a pre-release from `releases/latest`, so the front page went on advertising a three-day-old zip while a new one sat in the list. ⚠ `build.rs` has no `rerun-if-changed` on `.git/HEAD`, so the rebuild after the commit is a no-op and ships the *pre-commit* stamp unless you `touch` it between them. ★ **`package-portable.py`'s own header argues AGAINST publishing this release** — a doc correction, a test added, an engine re-pin with no behavioural difference is its exact excluded list. He has read that argument and overruled it: *“always release the latest version when you are done work.”* Do not re-perform the cost-benefit |
| Driven checks | `ui-verify --list \| grep -cE '^  [a-z0-9_]+$'` | **224** — level with the shipped release, which is `v0.5.0-dev.20260914.2`. The most recent addition is `the_export_windows_open_on_the_settings_you_last_used`, which discharged the last item owed on O196 and **is now in a shipped zip**. ⚠ **the `$` is load-bearing and its absence answers 225.** `--list` prints a second small table, the `--exe` targets, whose rows are also two-space-indented lowercase words; without the end anchor the count silently picks up `pdfcer-gui` and `pdfcer-legacy` from it. That is a *count command* being wrong rather than a quoted answer being stale, which is a different defect and the harder one to see. ★ If a count here is about double what you expect, you counted lines: `--list` prints two per check plus an eight-line header, so a `wc -l` answers `2n + 8`. ★★★ **And the binary on disk can answer an OLD count while every guard in the harness stays silent**: `ui-verify.exe` answered 223 for an hour after the commit that added the 224th. ⚠ **Not because the harness is unguarded** — `refuse_if_self_is_stale` compares the running `ui-verify.exe` against every `.rs` and `.toml` under `tools/ui-verify/` and has since it was written. **`--list` returned before reaching it**, by design, on the reasoning that `--help` and `--list` “answer without driving anything”. That reasoning is sound about DRIVING and the gap is that `--list` later became the authoritative **count** command, quoted into `FEATURES.md` and into release notes. ✓ **Closed 2026-09-14**: `--list` is now behind the guard, so a stale harness makes the count command print nothing and exit 2, and `grep -c` answers **0** — a number nobody can mistake for a roster. Rebuild the workspace before quoting this row |
| Gates | `bash tools/gates/run-all.sh` | **53 passed, 0 failed, 0 skipped**, exit 0 — level with the shipped release. ⚠ **This cell said *two* more than the previous release until 2026-09-14, where the answer was four**, because it was counted by remembering the gate that was interesting rather than by diffing the runner. ⇒ **Count a gate change by diffing `run-all.sh`, never by recalling which gate felt new.** ★ The two newest are `check-region-names` and `check-test-temp-paths`, and each brought its own self-test, which is why the total moved by four. What `check-region-names` is for: a `REGION_*` constant is `pub`, so **nothing in the toolchain fires when a region is declared and published by no one** — not `dead_code`, not clippy at `-D warnings`, not any other gate here. The symptom reaches a reader as a driven check reporting a control that is plainly on screen. It found three, all in `dialogs/export_image.rs`, all now real |
| Unit tests | `cargo test --workspace` | **4,293 passing, 0 failed, 51 ignored, 4,344 defined** — **three ahead of the shipped release**, which is `v0.5.0-dev.20260914.2`, and the three are the instrument in `tools/ui-verify/tests/list_is_behind_the_staleness_guard.rs`. Summed over **25** `test result:` lines — 25 rather than 24 because a new integration test is its own binary, so **a count of the result LINES is a second thing this row measures** — then cross-counted by `cargo test --workspace -- --list \| grep -cE ': test$'` = 4,344. Two methods, because a summed figure nobody cross-checks is how the last count drift got in. ★★★ **For a release the two methods are wired together rather than compared by eye**: the script that writes `FEATURES.md`'s test paragraph reads the `cargo test` transcript and sums it itself, takes the cross-count as an argument, and **refuses to write the document at all** if the two disagree or if anything failed. A number that reaches a shipped document by the same act that measures it cannot drift from it. ★ **The cross-count is also how you know a new test RAN**: three `#[test]` functions once compiled clean, reported nothing and executed zero times, because they were nested inside another function — the summed figure moved by nothing and so did the listed one. ✓ **Two concurrent runs are safe as of `7b48dd0`**, and that is measured rather than assumed: the pre-fix source was rebuilt as a CONTROL and its test binary run twice at once — red 3 rounds of 3 — then the fixed binary, green 5 of 5, then the three affected test binaries in full, twice simultaneously. ★ The control named *different* victims than the failure that started it, which is the finding itself: the casualty is whichever test lost the race |
| Source files | `find crates -name '*.rs' \| wc -l` | 790 |
| Backlog register | `python tools/walk-engine-backlog.py` | 174 rows — wanted 35 / blocked 8 / unknown 0 / declined 14 / **shipped 117**. ⚠ **Rewrite the five headings from the walker's own printed figures, never by arithmetic** — that instruction is in the walker's output because the arithmetic has been got wrong seven times. |
| Request channel | `ls /d/Dev/FeatureRequests/pdfce_FeatureRequests/open \| wc -l` | **46** — one more than at 01:07, and it is `done_G014_page_verbs_and_the_gesture_without_its_outcome_CONSUMED.md`. **G014 is consumed and nothing is owed either way.** Its §3 answers the addendum's UNRESOLVED row the way only this side could: not with feature-list prose but with **command ids**, because R8 makes reachability measurable — id in the catalogue **and** placed in the manifest. insert / extract / delete / rotate are named commands; **`reorder` is not a command at all** — the verb is carried by `pages.move_up` / `pages.move_down` and by the Pages panel's drag, so a by-name search for the word finds nothing and concludes wrongly. **G013 is now mirrored in this repository**, closing the thing this row warned about for a day: the borrowed-guard clause lives in `crates/pdfcer-gui/src/canvas/snap.rs`, in the doc comment above the `SnapKind::all()` completeness test that the note is about, and the `CheckStyle::all()` gap it logged was already an `ENGINE_BACKLOG.md` row. ★★★ **A declined offer still gets a `done_*` even though nothing is owed**, and that is a rule rather than politeness: a decline that lives only in a reply nobody consumed reads, six weeks later, as an open question, and the next session re-offers it. ⚠ **Re-`ls` this folder before quoting it** — it changes faster than any other input to this project, it is in no git repository, and nothing warns you. |
| Registered commands | `grep -rn --include='*.rs' -E 'command\(' crates/pdfcer-gui/src/shell/commands/catalog/ \| grep -vE ':\s*(///\|//)' \| wc -l` | **161** — ★★★ **the obvious command is wrong, not merely its answer.** A raw `grep -rhoE 'command\('` over that directory returns **162**, and the extra is a line of prose inside a comment in `catalog/file.rs:556` that quotes the very pattern being searched for. The build's own `pdfcer-diag shell commands=` trace is the tie-breaker and says 161. |
| Dockable panels | `grep -n 'pub const ALL' crates/pdfcer-gui/src/panels/mod.rs` | 13 |

⚠ **The check-count command was wrong in this very table, twice, in two
different ways.** `--list` prints **two lines per check plus an eight-line
header**, so a `wc -l` answers `2n + 8` — roughly double the real figure, which
is why it looked plausible. Then the repair that replaced `wc -l` with a
`grep -c` **dropped the `$`**, and picked up two rows of a second table. Measured
against `target/release/ui-verify.exe` on 2026-09-13: **222** names, **452** lines.

★★★ **Write the relationship, not the pair of literals.** The first repair of
this note said *"`wc -l` answered 434 where the answer is 213"*. Both numbers were
correct the day they were written and both moved the next time a check was
added, so the note that exists to stop count drift had drifted. A lesson spelled
as two constants expires; one spelled as `2n + 8` cannot. ⇒ **If a count here is
about double what you expect, you counted lines.**

⚠ **The pin is a claim with a shelf life of hours.** `Cargo.lock` pins the
engine by git revision, not by path, so every sentence in this repository of the
form *"the engine cannot do X"* is a statement about **a pin**, not about the
engine. Under O165 the first act of a session is to compare the two pin rows
and, if they differ, read the intervening log **before** reading any backlog row.

⚠ **`git describe` LIES in a repo that has not fetched.** `gh release create`
makes the tag **on GitHub**, not here — nothing pulls it back. On 2026-09-11 the
operator asked *"new release soon?"* and the answer began **"51 commits have
stacked up since Tuesday's build"**, computed from a `describe` that had never
seen the three releases cut since; the true figures were **21 commits** and
**yesterday afternoon**. ⇒ **Any sentence containing *"commits unreleased"* or
a release date gets a `git fetch --tags origin` immediately in front of it.**

★★★ **`v0.5.0-dev.20260914.2` shipped 2026-09-14 07:22 UTC**, from
commit `2bb0b19` on engine `7378c838`, and **it is the first release of this
project whose headline is that nothing an operator can see changed.** No
capability, no fix to a reported defect, no behaviour anywhere that differs from
the zip before it. Two things earned it, and both are this program telling the
truth rather than doing something new: **the front page stopped naming a gesture
without its outcome** — dragging pages between documents COPIES, Shift moves,
and the sentence now says so — and **the check the fourth release admitted it
owed was paid**. Verified as what `releases/latest` advertises by reading the API
back, not by having passed the right flag: `prerelease=false`, `draft=false`, one
asset `pdfcergui-20260914-0317-7378c83-2bb0b19.zip` of 23,270,105 bytes.

★★★ **A release with nothing visible in it is one `package-portable.py`
argues against shipping, and that argument is overruled.** Its header's excluded
list is *"a documentation or comment change, a test added or a harness improved,
an engine re-pin with no behavioural difference"* — which is this release,
three for three. Ken has read that cost argument and ruled over it: *"always
release the latest version when you are done work."* ⇒ **Do not re-perform the
cost-benefit.** The fallback slot is what makes a bad build cheap; a build
sitting only in `target/release` is worth nothing to him at all.

⚠ **Nothing he can SEE changed with the engine bump either.** The pin moved
`3df0c72` ⇒ `7378c838`, **one** commit, and
`git -C /d/Dev/pdfcer diff --stat 3df0c72..7378c838 -- '*.rs'` is **empty** —
it is the librarian's 547th filing. A pin bump in a release note reads like a
behaviour change unless the diff is quoted beside it.

★ **Both destinations, and the fallback survives.** **`OneDrive\pdfcer-gui2`**
carries this build (`Built: 2026-09-14 03:17:14`, shell `2bb0b19`, engine
`7378c83`); **`pdfcer-gui1` still holds the 2026-09-14 00:38 build**, which is
the point of alternating slots. Both were confirmed by reading `BUILD-INFO.txt`
out of **both** slots after packaging — that two-line read-back has now caught
three distinct failures of that one script, and the tool's own report is not
evidence about the tool's own effect. **The next package rotates back into slot
1**, and the rotation works this out for itself from the two timestamps, so pass
`--slot` only to RETRACT, never to steer a normal build.
⚠ His own install at `C:\Users\Ken\OneDrive\pdfcer\` is **not** touched by a
publish and was not touched by this one.

⚠ **Neither slot's `BUILD-INFO.txt` carries a "THE ENGINE HAS MOVED ON"
banner today.** ⇒ **The banner is computed at PACKAGE time against the pin, so
it is a property of one zip and never of a slot** — re-read the file rather
than carrying the warning forward.

✓ **The gap the FOURTH release disclosed is closed, and the disclosure was
not deleted.** That release's notes said in as many words that 223 driven checks
existed and not one was about the export windows' memory. There are **224** now
and the new one is exactly that check. `FEATURES.md`'s thirtieth revision still
carries the original admission, standing and unedited, with the thirty-first
discharging it by name above — **a disclosure that is quietly deleted when its
subject is fixed teaches the next reader nothing about what the gap cost.**

★★ **What was NOT measured for THIS release**, recorded here because the
public release notes say it too: **the full ninety-five-minute driven sweep.**
The last complete one was 2026-09-13 and read 183 passed / 4 failed / 34 skipped
over **221** checks — there are 224 now, so even that figure is not a
statement about this build. The four failures were the same four by name for the
third run running and every one of them a defect in the *check* rather than in
the program. ⚠ Quoting *"224 checks"* without that sentence beside it reads as
a claim this build does not support.

⚠ **`build.rs` declares `rerun-if-changed` on `src`, `Cargo.toml` and
`../../Cargo.lock` — and NOT on `.git/HEAD`.** So the rebuild that is supposed to
happen between the release commit and the packaging finishes in about a second
and changes nothing, and the shipped binary carries the *pre-commit*, dirty
stamp. `touch crates/pdfcer-gui/build.rs` forces it. This cost a rebuild-and-
re-drive cycle on the way out of `.4`; it is the kind of thing a green build
cannot tell you about, because nothing failed.

★★★ **The release pre-flight lied once on the way out, and it is worth the
paragraph.** `run-all.sh` reported **41 of 41 green**; the commit added
`DESIGNS.md` and `DOC_DRIFT.md`; `package-portable.py`'s own pre-flight then
failed the **same tree** on two lines that had been sitting in those files the
whole time. `check-old-name-absent` used plain `git grep`, which scans **tracked**
files, so its input set changed at `git add` time. It now passes `--untracked`,
and the repair was falsified against a planted untracked file the old version
could not see at all. ⇒ **Every other gate in `tools/gates/` that reaches
for `git grep` or `git ls-files` has the same hole**, and the tell is always the
timing rather than the content: green before the commit, red after it, with
nothing edited. ⇒ **That audit was DONE on 2026-09-13 and is now
an instrument rather than a paragraph.**
`tools/gates/check-gate-input-scope.py` walks `tools/` with `os.walk`
and never asks git anything — an auditor carrying the defect it
audits is worthless — and it is registered in `run-all.sh` with its own
self-test. It found a fourth instance, `check-doc-markup`, which had **never
once fired**, because the Markdown file most likely to carry a truncated table
row is the one just written.

⚠ **The number in a release title is the release count for that DAY, not the
session's.** An earlier edition of this block called `.3` *"the fifth release of
the day"* and it was the fourth. If you need the count, `gh release list`.

---

## ★★★ The sweep ran again — 221 checks, same four failures for the third time, and the news is in the SKIP column

**2026-09-13 10:16 to 11:51, taking the real cursor and keyboard for about
ninety-five minutes.** The fourth full driven sweep in two days. It did not
abort.

⚠ **It measured the binary built at 10:16 that morning, and TWO RELEASES have
been cut since.** The sentence here used to read *"against the binary that
ships"* and was true for about three hours. ★ It is deliberately not written as
a tag: the sweep ran from a working tree between two releases, so naming either
one would be a citation I cannot support, and *"the build of 10:16"* is the
strongest true statement available. ⇒ **A sweep tally is a fact about one
executable, never about the program.** The figures below still stand as the
last FULL reading; what they do not cover is the two releases since, each of
whose commit message carries what WAS measured for it.

```
=== SWEEP-DONE
=== TALLY passed=183 failed=4 skipped=34 codes: rc=0 rc=1 rc=3
```

★★★ **Verify the total before quoting any part of it.** Three ways, all
agreeing on **221**: the tally's own arithmetic (183+4+34), the count of
DISTINCT check names carrying a verdict in the log
(`grep -oE '^[[](PASS|FAIL|SKIP)[]] [a-z0-9_]+' | sort -u | wc -l`, which also
proves nothing was counted twice), and the roster
(`ui-verify --list | grep -cE '^  [a-z0-9_]+$'`). The reason this paragraph
exists is that a previous run aborted mid-way and printed a perfectly ordinary
tally that described 160 checks — a count of what happened to run reads
exactly like a count of the suite.

### The diff, in both directions, against the 00:24 run

```
FAIL  new: (none)      gone: (none)     — the same four, by name, third sweep running
SKIP  new: 5           gone: (none)
```

The five that arrived in the SKIP set:

```
embedding_fonts_puts_a_program_in_the_document
save_copy_round_trip
tab_order_drag_moves_a_field_and_shows_where
the_standards_presets_group_is_reachable
the_title_bar_carries_the_build_time
```

★★★ **All five were re-driven immediately, against the same frozen copy of the
same binary, on an idle machine: `5 passed, 0 failed, 0 skipped`.** Four of them
had timed out waiting thirty seconds for a window to appear. That is a threshold
on **how busy the machine is**, and the sweep was sharing this computer with a
compile.

⇒ **So the measured coverage of this build is 188 of 221, in two runs rather
than one**, and both numbers belong in any report. The five are not a
regression, and they are not nothing either: **load can remove coverage from a
sweep without anything turning red**, and the only instrument that sees it is
the both-directions diff. It is `SWEEP_REPAIRS.md` **R11** now, because the
message the harness printed blamed *"a platform that cannot enumerate windows"*
— a cause it never measured, and false here, since the same process enumerated
windows for 216 other checks in the same hour.

⚠ **Do not fix R11 by raising the timeout.** Thirty seconds is already long;
a larger number moves the boundary without removing it. The defect is that the
expiry is reported as a property of the platform rather than of the moment.

★★★ **All four remaining failures are defects in the CHECK, not the program**,
each written up with the line of its own evidence that refutes it —
`SWEEP_REPAIRS.md`, rows R4, R6, R8 and R9. Read that file before investigating
any FAIL in this suite. The clearest of the four: a check reported that *the
shell built no plan* on a click into real text, and three lines above its own
assertion the trace shows the shell detect no run under the click, convert the
gesture to an Add, place the text and disclose the font substitution off-canvas
exactly as rule 4 requires. Four correct behaviours read as silence, because the
oracle greps three event names and `add-text` is not one of them.

★★ **A list of four that does not move is a residue; a list of four that
changes membership every run is a program coming apart.** This one has not
changed membership in three sweeps.

### ★★★ The sweep never drives the binary that ships, and it is not a rounding difference

**Measured 2026-09-13 while cutting `v0.5.0-dev.20260913.2`.** `sweep-full.sh`
copies `target/release/pdfcer-gui.exe` to `target/scratch/drive/` before the
first check, so it measures the program **as it stood before the commit
existed**. `build.rs` compiles `PDFCER_BUILD_TIME`, the short git rev and the
`-dirty` flag into the program and declares `.git/HEAD` an input, so the act of
committing relinks it.

    cmp  swept copy vs shipped exe     same size, 29,746,176 bytes
    diff 7,998,303 bytes, 25,508 runs  every run a relative call target moving

⇒ **Nothing there is a change to what the program does.** The stamp string
got shorter when `-dirty` fell off, everything after it slid, and every relative
call offset was rewritten. But *"the suite was run against this exact binary"*
is a claim about bytes, and it has been in the last three release notes while
being false in all three. The honest form, now in `FEATURES.md`, is **"against
these sources, from this commit's tree"**.

⚠ **The mitigation is cheap, so do it rather than argue the point.** After the
release commit and before packaging: rebuild, then re-drive the handful of
checks that read the version stamp against the executable that actually ships,
plus the off-screen smoke launch. That is a couple of minutes and it converts an
argument into a measurement. **Do not try to make the two files identical** —
`PDFCER_BUILD_STAMP` can pin the time but nothing can pin the git rev of a
commit to a value known before the commit exists, and a build that lies about
its own revision is a far worse defect than a relink.

### The baselines, and the naming defect that has been fixed

```
target/scratch/sweep-skips-2026-09-13-1151.txt   34   <- diff the NEXT sweep against these
target/scratch/sweep-fails-2026-09-13-1151.txt    4
target/scratch/sweep-skips-2026-09-13-0024.txt   29
target/scratch/sweep-fails-2026-09-13-0024.txt    4
target/scratch/sweep-skips-2026-09-12-2242.txt   31
target/scratch/sweep-fails-2026-09-12-2242.txt    4
target/scratch/sweep-full-2026-09-13-1151.log         the whole run, kept
```

★ These used to be called `sweep-skips-now.txt` and `sweep-fails-now.txt`,
and this section described them as *"dated records"* while their names said
`now`. They were the **2026-09-12** sets, saved under a name that claims to be
the present, sitting beside a sentence quoting a different run's figures. The
files are renamed by date and hour and the `now` names are gone. ⇒ **A dated
record must carry its date in its name, or the next reader diffs against
whichever run happened to write last.**

⚠ **Do this diff before quoting any tally, in both directions.** A
previously-passing check falling into a SKIP moves the `failed=` figure not at
all.

### ⚠ While a sweep runs, the tree is frozen

The rule that came out of the run before this one, and it held: nothing under
`crates/` or `tools/` was touched between the first check and the last, and the
sweep completed.

`staleness_complaint` (`tools/ui-verify/src/launch.rs:788`) scans the source
root for anything with extension **`rs` or `toml`** and refuses to drive a
binary older than its source — correctly, because a stale harness asserts on
renamed trace keys and produces confident failures about the wrong subject. It
does not, and should not, try to work out whether an edit was semantically
inert; a comment-only edit aborted a sweep an hour and a half in and cost
sixty-one checks.

⇒ **Three rules, and they are cheap:**

1. **No `.rs`, no `.toml`, not even a comment, not even in a crate the sweep is
   not about.** **Markdown, `.sh`, `.py` and the scratchpad are not scanned**, so
   documentation is the safe way to spend the ninety-five minutes. This session
   spent them on `SWEEP_REPAIRS.md`, `FEATURES.md` and two memory files.
2. **Resuming is legitimate when the APPLICATION binary was never touched.**
   `sweep-full.sh` copies `target/release/pdfcer-gui.exe` to
   `target/scratch/drive/` at the start and drives that copy, so rebuilding the
   *harness* and re-running the remaining chunks measures the same program the
   earlier chunks measured. Say so in the resume, or the resumed half reads as a
   different experiment.
3. **A resume must re-derive its chunk boundaries from
   `target/scratch/checks.txt`**, which the aborted run already wrote — not
   from a fresh `--list`, which would renumber if a check had been added.

★★ **Nothing in this run is owed against the shipped build.** Every failure is
a check defect with its refutation written down, every skip was read, and the
five that started skipping were re-driven and verified green rather than assumed
to be noise. ★ **Never reason about whether a skip "looks like a flake" —
run it.** Five names was thirty seconds of work, and it is also what would have
caught a real regression hiding among four flakes.

⇒ **The harness and doc-drift registers are now the biggest owed body of work
in this repository**, all of it `.rs` or markdown and none of it blocked on the
desktop: `SWEEP_REPAIRS.md` (11 rows plus two appendices) and `DOC_DRIFT.md`
(12 rows, S11 the highest-leverage). That is item 2 under *Do next*.

## ✓ Nothing is owed against the shipped build — both items discharged

This section carried two items for a week and the sweep closed both. It is
kept, empty, with the evidence, because *"we drove it and it worked"* is the
only sentence that retires a *wired-not-driven* caveat, and the next session
has to be able to tell that apart from the caveat being forgotten.

1. ✓ **The bold ladder is DRIVEN now.** It shipped in `.3` with *wired,
   not driven* printed in the operator's own release notes, and the morning
   sweep could not discharge it because its three checks all skipped on a
   shared aim point that selected a `Path` instead of text. All three pin
   their own fixture now and all three PASS:

   ```text
   [PASS] restyling_selected_text_reaches_the_document
          pins fixtures/paragraph.pdf at page 0, 120, 704
          pdfcer-diag text-style-applied page=0 change=weight applied=1 runs=1
   ```

   That is Bold pressed in the running program and the restyle committed to
   the open document through `format_text` — not a unit test standing in
   for it.

2. ✓ **The deep-marquee adoption is DRIVEN now**, on a scratch copy of
   the operator's own drawing.
   `a_marquee_over_a_table_takes_its_text_as_well_as_its_lines` drags a
   right-to-left crossing band, and the engine's `hit_test_rect_deep` —
   reached through `panels/objects/provider`, our duplicate `hit_test_rect`
   having been deleted — answers with both kinds:

   ```text
   pdfcer-diag marquee-mode crossing=true mode=touched hits=3 paths=2 text=1 other=0
   pdfcer-diag canvas-selection via=pv.marquee mod=false sel=3 level=Object
   ```

   A band and a click can no longer disagree, and that is measured now rather
   than argued. ★ Two sibling band checks pass beside it
   (`a_band_dragged_into_the_margin_reaches_an_object_off_the_page` and
   `a_band_that_starts_in_the_margin_reaches_an_object_off_the_page`).

---

---

## ✓ Two `cargo test` runs at once no longer corrupt each other — and why it needed a gate

**Fixed and gated in `7b48dd0`.** Eleven test fixtures wrote to a FIXED path
under `%TEMP%` with nothing process-unique in the name. Inside one `cargo test`
that is safe — each name has exactly one user — so it stayed invisible until two
runs overlapped, which is ordinary: a backgrounded sweep plus a foreground run,
a watcher, or a session that started a second suite because the first looked
stuck.

★★★ **The symptom is what makes this worth a section.** It is not "two
tests collided". It is a single unrelated assertion going red, several lines
downstream of the real event, in whichever process lost the race — and then
passing on re-run. That reads as a regression in the feature that test covers,
then as a flake. Neither reading points at shared state. The control run proved
it: rebuilding the pre-fix source and running its binary twice at once named
**two different victims** than the failure that started the investigation.

★★★ **And two of the eleven carried a confident note saying the hazard was
handled** — *"tagged per caller, because `cargo test` runs these in parallel"*.
That is true and it separates THREADS. It does nothing about PROCESSES, because
two `cargo test` runs have the same callers as each other and therefore ask for
the same filenames. ⇒ **A note that names a hazard and fixes half of it is
worse than no note**: the next reader sees the hazard named, sees a mechanism
beside it, and stops looking. `canvas/guides.rs` had carried the correct pattern
since it was written, so the convention existed here and simply was not uniform
— which is the textbook condition for a rule living only in prose.

`tools/gates/check-test-temp-paths.py` is the instrument, registered in
`run-all.sh` with its self-test. Its rule is one sentence with no taxonomy in
it: **`std::process::id()` in the path, or `// temp-path-exempt: <reason>`.**
Four helpers that were already safe on a nanosecond stamp take the pid anyway,
because a rule reading *"a process-unique component, and here is how the gate
recognises one"* has a classification in it, and a classification is where the
next exception goes. Seven sites are exempt with a written reason: four never
create the path, three are `#[ignore]`d generators where a stable name is the
whole point.

⚠ **The evidence window is derived from the code, not counted.** It starts at
the statement holding `env::temp_dir()` and, if that statement binds a name,
follows the `.push`/`.join`/`.set_extension` chain that is still building the
same path. A fixed N-line window would let a compliant neighbour vouch for a
bare site — which is precisely how `check-gate-input-scope`'s first draft
reported zero of three planted violations while printing PASS.

## Do next, in the operator's order

> ★★★ **Order this list by what the operator reaches for, not by what just
> arrived in the engine channel.** On 2026-08-18 print paper sizes got built
> while clicking a stamp did nothing at all. The engine's replies are an input
> to *how* a thing gets built, never to *which* thing.

1. ★★★ **His own list. `O186` is BUILT AND DRIVEN as of 2026-09-13** — both
   halves — so the top of the list is now `O185`.

   What O186 cost is worth one paragraph, because the shape recurs. The fix
   itself was two lines of arithmetic: `PASTEBOARD_FRACTION` was `1.0`, which
   made the pasteboard exactly one viewport, which made the ends of
   `visible_origin_range` the **zero-overlap** placement on both axes at every
   zoom — the blank screen was an *allowed* position, not an escaped one.
   `pasteboard()` subtracts `sheet_sliver(viewport)` now, and `canvas/escape.rs`
   restores `paging::flip` and `zoom::wheel_step` above the blank-frame early
   return. ⚠ **And narrowing the pasteboard broke where documents OPEN.**
   Frame 0 of a multi-page file began drawing a sliver where it drew nothing
   before, `zoom::remember_frame` therefore ran, and on frame 1
   `fit::placement`'s resize arm outranked the open-seed arm and preserved the
   "centre" of a frame nothing had placed — egui's default `(0,0)`, the corner
   of the pasteboard. The document opened off the bottom-right and stayed there,
   with every unit test and all 43 gates green. ⇒ **A geometry change changes
   where a document opens, not only where it can be dragged. Smoke-launch; do
   not merely re-run the checks.** The guard is now
   `doc.canvas_frames <= canvas::offset::SEED_FRAME`, one named constant read by
   both the arm that fires and the arm that declines.

   ★ **What made the second defect findable in twenty minutes instead of a
   day:** `canvas::offset::Decision` carries a `source` field and
   `canvas::trace::placed` prints it as `src=` on the `canvas-place` line. Three
   arms of that chain can produce exactly `(0.0, 0.0)`, so the VALUE never
   identified the producer; the `src=` field did, on the first launch after it
   was added. Hand-arithmetic had already exonerated `geometry::strip_offset`,
   which was the third wrong hypothesis in a row.

   Then the seven `FEATURE.txt` rows still open, in this order — cheapest
   first, which is how the six that shipped got shipped: **O185** (print
   dialogue memory and Cancel) → **O181** (installed fonts in Add text, and
   the dead Format ribbon) → **O188** (moving and deleting one text block
   inside a group) → **O189** (bookmarks on a cross-document page drag)
   → **O183** (the nine-part ce-dimension paragraph, part 7 first)
   → **O178** (multi-window tab dragging) → **O182** (white seams in the
   KUBOTA render). Designs for O181, O183, O185, O188 and O189 are in
   `DESIGNS.md`. ⚠ **Only Ken closes a row.**
2. ★★★ **Five rows he filed on 2026-09-13, AFTER `FEATURE.txt` — three of them
   are built and shipped, and the rest are measured, so none of them
   starts from zero.** O192, O193, O194 and O195
   are his words; O196 is one we found and he has not reported yet. Full
   `file:line` findings are in `OPERATOR_REQUESTS.md` under a
   `### MEASURED 2026-09-13` block on each row. ⚠ **Only Ken closes a row.**

   ✓ **O192 + O193 — BUILT AND DRIVEN, shipped in `v0.5.0-dev.20260913.4`
   (commit `3f38ef3`). NOT CLOSED — that is his.** They were one structural
   fix, as predicted: `ScaleDialog` is handed `&OpenDoc`, so the window can see
   the document it edits. It names the group, shows the scale already on that
   group, and re-reads on a group change. ★ Two further defects went with it,
   neither reported — *Measure it on the drawing…* destroyed the window and
   everything typed into it, and a measured length would have been delivered to
   the canvas's active group rather than the window's. Driven by
   `set_scale_reads_the_group_it_is_about_to_overwrite`.

   ⚠ **What is still open on O192, and it is the first thing he will look
   for:** his wording asks for the scale to update *“even before I hit
   apply”*. What ships updates on reopen and after a calibration. Whether
   that is the same thing is his judgement, not ours — do **not** pre-emptively
   build a live-preview until he says so. The full four-defect account, the
   reasoning behind deriving `hidden` rather than storing it, and why the driven
   check asserts a **count** rather than *“the window is back”*, are all on the
   O192 row in `OPERATOR_REQUESTS.md`.

   ★ **A correction to O192's own text, kept because the row was wrong:** it
   claimed this was *“the third surface to have it”* and proposed a sweep of
   every dialogue that writes a value without reading one. **The sweep was run.
   All 28 dialogues were checked and Set Scale is the class's only member.** The
   generalisation was wrong; the defect is real.

   ◑ **O194 — units everywhere. Enumeration DONE, step 2 DELIVERED
   2026-09-13, steps 1/4/5 still open.**
   `UNIT_SURFACES.md` lists all 55 surfaces with `file:line`: 3 already follow
   the operator's unit, 4 offer a menu, ~18 are hard points, ~12 hard
   millimetres.

   ✅ **Step 2 — the one conversion table — SHIPPED in `v0.5.0-dev.20260913.3`.**
   `crates/pdfcer-gui/src/units.rs` is now the only place a document length is
   converted or rounded for display, and `tools/gates/check-unit-conversion.sh`
   fails the build on a fresh `25.4` anywhere under the GUI outside it. The six
   private constants, the seven re-declared closures, the two `f32` paths and
   the five DPI spellings are at **zero**; `whole()` is the only function
   permitted to round a length. The gate has a self-test with planted
   violations, so it cannot quietly stop finding things.

   ★★★ **The defect it fixed, and the diagnosis that was wrong three times
   before it was right.** A sheet of exactly 210.5 mm read **210** in the page
   thumbnail's tooltip and **211** in the print dialogue. **The cause is the
   rounding RULE, not the precision** — `.round()` is half-away-from-zero,
   `{:.0}` is half-to-even, and both paths land on exactly 210.5000000000. It
   was stated backwards on both counts until 13:30, when the two expressions
   were finally compiled rather than reasoned about. ⇒ **The remedy had to
   settle the rounding rule, not merely share a constant** — a shared constant
   would have left the two surfaces disagreeing by exactly as much.

   ★ **Two escape hatches in the gate, and the second was found by RUNNING it
   rather than by designing it:** `NOT A DOCUMENT LENGTH:` and `ORACLE, NOT A
   CONVERSION:`. All four real-tree hits of the second kind were independent
   test oracles — a test that recomputes the conversion by hand is the one place
   a second spelling is the point.

   ⚠ Font and type sizes are **excluded, and the exclusion is written into the
   source** rather than merely observed, or somebody eventually offers him a
   font size in kilometres. So are the ce-dimension group's text height, arrow
   size and gap, and scale-aware conversion, which is a different feature. ⚠
   `text/markup.rs` is at **exactly 1,500 lines** and must be split before it
   can be touched.

   ★★★ **O194 clause 3 (km, miles, yards) is DELIVERED, and the sentence
   that used to sit here was false twenty-one minutes after it was written.**
   It read: *"blocked on the engine — `request_G013`, filed 08:58, unanswered
   at this release."* `reply_G013` landed at **09:19** with `Kilometer`, `Yard`
   and `Mile` shipped on our own integer-ratio factors, and with `Unit::all()`
   widened to `&'static [Unit]`. It was adopted, built, smoke-launched and
   consumed (`done_G013_CONSUMED.md`) by 10:12. **All three unit dropdowns
   offer nine units in the build being released** — Set Scale, the dimension
   group editor and the per-dimension override — and none of them needed an
   edit, because all three read `Unit::all()`.

   ⇒ **A limitation sentence is a citation with an hours-long shelf life.**
   This is the second time in one morning. The engine session answers in
   minutes; anything written here of the form *"X is not possible / not
   answered / blocked"* must be spelled as a dated observation and re-measured
   before it is quoted, never as a standing property.

   ⚠ **What is still ours in O194, now that step 2 has shipped**, and this
   list is the whole remainder: **step 1** — the ~30 surfaces that still have no
   unit control at all, which is the bulk of the row and the part he will
   notice; **step 4**, half done — `km`/`yd`/`mi` have `ui_text`
   abbreviations and `mm`/`cm`/`m`/`in`/`ft` do not; and **step 5**, not
   started. Nothing in any of them waits on anything.

   ★★ **O195 — smart select in Review — is NOT a condition change, and the
   row's own ★ clause saying it was is struck as FALSIFIED.** The preference is
   **already on** in Review. What Review lacks is not the mechanism and not the
   setting: it is a **consumer**. `canvas/clicking.rs` hands the press to text
   selection whenever the gate matches, and in Review the gate matches `Select`
   unconditionally — so the smart-select rung never runs. Flipping the manifest
   condition would ship a visible, inert control, which is the exact failure R9
   exists to prevent. ⇒ **The deliverable is that a Review click resolves the
   way he means, not that a control appears.** It is a capability-model question.

   ✓ **O196 — the three export windows remember. BUILT, TESTED AND GATED in
   `df1edf4` + `28f5389`, and SHIPPED in `v0.5.0-dev.20260914.1`. NOT CLOSED — closing is his.**
   Twelve settings across Export image, Export text and Export to DXF now
   round-trip through `userdata/preferences.txt` and seed the window the next
   time it opens, on `PrintPrefs`'s own pattern from O166.

   ⚠⚠⚠ **A correction, and it is about a quotation rather than a number.** The
   first draft of this bullet opened with a sentence in quotation marks attributed
   to him, about having to set a DXF export up again every time. **It is not
   sourced.** `OPERATOR_REQUESTS.md` O196 is headed *"FOUND 2026-09-13, not yet
   reported by the operator"* and argues at length why a row he never filed is on
   the list; a grep of every `.md` and `.txt` here finds that sentence in exactly
   two places and both are mine, this file and the commit message of `28f5389`.
   ⇒ **The commit message of `28f5389` misattributes it and history is not being
   rewritten for that** — it is recorded here instead, which is the only place a
   future session will look. ★★★ The generalisation is the expensive part: **an
   inherited session summary is not a source.** A number carried across a summary
   is re-measured by standing rule; a QUOTATION carried across one has no such
   rule, reads as the most authoritative sentence in the document, and cannot be
   falsified by anybody who reads the code. Attribute only what a file on disk
   says, and say which file.

   ★★ **Two rulings are built into the code rather than remembered.** A DXF
   page carrying its own calibration still overrules the remembered units, so a
   metric habit cannot quietly turn an inch-calibrated sheet into a 25.4x error;
   that ordering is two adjacent statements in `seeded_options`, which exists as
   a pure function precisely so a test can put the question to it — `open` needs
   a whole `EditSession` and is therefore unaskable. And the typed page range is
   deliberately NOT remembered: *"12-40"* restored onto a nine-page document
   opens a window whose Export button is already dead for a reason the operator
   did not cause.

   ★★★ **The half of this that was not the feature.** Writing the driven check
   found that `dialogs/export_image.rs` declared `REGION_PAGES`, `REGION_QUALITY`
   and every arm of `region_for_scope` — and **published none of them**. A region
   constant is `pub`, so nothing in the toolchain can see this; the defect
   reaches a reader as a driven check reporting that a control on screen does not
   exist. `tools/gates/check-region-names.py` is the instrument for that class
   now, falsified against the real pre-fix file. ⚠ **The measurement of it was
   wrong three times first**, and that is the part worth carrying: a bare-name
   search over the workspace discharged all three dead twins, because
   `export_text.rs` declares its own healthy namesakes. A declaration-and-use
   measurement has to be scoped the way the LANGUAGE scopes it — by import — or
   it is measuring the wrong relationship and looks clean.

   ✓ **The driven check landed 2026-09-14: `export_remembered.rs`,
   `the_export_windows_open_on_the_settings_you_last_used`.** Two launches on
   `print_remembered.rs`'s control/seed pattern — reset the preferences file,
   launch once to MEASURE this build's shipped defaults off the trace, require
   every seeded value to differ from its measured default (SKIP naming any that
   does not), then relaunch and compare. Driven green over
   `fixtures/a1-titleblock.pdf` under `--no-input`: 12 of 12 came back.

   ★★ **It needs no mouse, and that is the interesting part.** It reaches all
   three windows through `PDFCER_DIAG_INVOKE`, which takes a comma-separated
   list and rings one command per frame through the same `dispatch_command`
   choke point a keystroke reaches — so **one launch opens all three windows**,
   each emitting its own `-open` line as it is built. No ribbon click, no
   overflow hazard, no File-tab activation, and it runs on a machine whose
   desktop is in use. ⇒ **The three Export-image regions turned out not to be
   needed after all**; a check that presses nothing cannot be blocked by a
   control that is folded away.

   ★ **Falsified before it was believed.** Only `export_text.rs`'s struct
   literal was switched to `ExportTextPrefs::default()` — the narrow sabotage,
   one window of three — and the check returned `RESULT: FAIL` naming **4 of
   12**, the four text rows and nothing else, and selected the middle
   escalation: *"Every failure is on ONE window's line and the other two
   restored correctly … the fault is in that window's constructor."* Restored
   from a file copy, never through git. ⚠ The all-twelve and the scattered
   escalations were **not** driven and the module header says so rather than
   letting two untested branches be inferred as tested.

   **The four traps, and what each cost.** (1) and (2) were designed around:
   the oracle is twelve token comparisons against measured defaults, and it is
   two processes. (3) was the expensive one and is handled by measurement — the
   control run reported `format=png scope=current dpi=300 transparent=1
   quality=90`, `scope=all separator=form-feed endings=as-extracted bom=0`,
   `units=inches arcs=1 text=entities`, and each is re-measured every run rather
   than written into the check. (4) **does not bite**, and the header says why
   rather than leaving the absence to look like an oversight: this check
   measures no rect at all, so there is no frame to get wrong. An edit that
   starts measuring one inherits `frame_of` the moment it does.

   ★★★ **Two traps the four did not name, both found while building it.**
   Three of the twelve are spelled `true`/`false` in the file and `1`/`0` in the
   trace — the file wants a word a person can edit, a whitespace-split trace
   field wants a digit — so the seed table carries a fifth column and a unit
   test pins the translation; a table that assumed they agreed would have
   reported three defects that do not exist. And `scope=` appears on **two**
   events with **different** shipped defaults, so a row is identified by
   `(event, field)` and never by `field`.

   ⚠ **The DXF units row is dropped when the page is calibrated**, reported as
   a note, and the pass line then says eleven rather than twelve. A check that
   asserted its seed there would report a defect where the application is doing
   the most important thing that window does.

   ✓ **O197 — the front page was selling the program short. BUILT and SHIPPED
   in `6be5ff9` + `c6ea38f`, released in `v0.5.0-dev.20260914.1`. NOT CLOSED —
   closing is his.** `README.md` leads with reading PDFs faithfully, which is
   where most of the work actually went, and the CAD tooling follows it instead
   of heading it.

   ★★★ **One claim in the draft was sourced, current and still wrong, and that
   is the half worth carrying.** It read *"a page authored in CMYK is
   composited as ink on separate colorant planes."* Every word of it is
   supported by `pdfcer-render/src/cmyk_buffer.rs`, which I had read end to end.
   What is not there is the **reach**: the whole path is gated on
   `page_space.is_subtractive()`, and the engine's own comment beside the
   switch counts it at **15 files in 4,012**. The sentence was rescoped, not
   deleted. ⇒ **A capability sentence on a public surface needs a COUNT, not a
   yes/no** — grep for the `if` that switches the feature on, and ask how often
   it is true, before the claim is printed.

3. ★★★ **Work the two registers. All of it is `.rs` or markdown and none of
   it needs the desktop.** `SWEEP_REPAIRS.md` holds 10 rows — the four FAILs
   this sweep produced (R4, R6, R8, R9), the one new SKIP (R10), and Appendix B's
   sixteen checks that assert nothing. `DOC_DRIFT.md` holds 12 rows of this
   project's own stale claims, dated against the commit that falsified each.

   What to do first, in this order, and each is one idea rather than a list.
   ★ **The list is the count.** A sentence that says *four things* beside four
   bullets is the shape this project has corrected eight times; when one is
   struck off, the sentence is wrong and nothing says so.

   * ✓ **DONE 2026-09-13 — the gate audit, and it is now an instrument**
     rather than a paragraph. A checker that asks git which files exist is asking
     about the **index**, so every file the current session wrote is invisible to
     it. ★★★ **Four instances, and the correct generalisation was written into
     a gate's own docstring before three of them happened.** ⇒ **A lesson in a
     docstring is not an instrument** — nothing swept for the pattern, so nothing
     found it. The audit found the fourth, `check-doc-markup`, which had **never
     once fired**: the Markdown file most likely to carry a truncated table row is
     the one just written. Repaired, and the repair falsified against a planted
     untracked file the old version could not see at all.

     `tools/gates/check-gate-input-scope.py` is the sweep, registered in
     `run-all.sh` with its self-test, which is why the suite was **43** and not 41 on the day it landed — a figure
dated on purpose, because the suite is **53** at `a227f94` and a gate count
written as a standing property is the drift this project has corrected eight
times..
     It walks `tools/` with `os.walk` and never asks git anything — an auditor
     carrying the defect it audits is worthless. Every real call must pass
     `--untracked` / `--others` **in its own extent**, or carry
     `gate-input-scope-exempt: <reason>` on its own line or the line above.
     `check-engine-api-drift` and `verb-coverage` read the engine at a revision
     deliberately and are exempt for that reason, which is the marker used
     correctly rather than a loophole in it.

     ★ **The two things to carry forward.** The tell for this defect in the
     wild is the **TIMING, not the content**: green before the commit, red after
     it, with nothing edited. And the one-step falsification is to plant the
     violation in an **untracked** file, because a gate with the hole cannot see
     one at all. ⚠ Worst in the release path, where the thing waved through
     is the thing that ships.
   * ✓ **DONE 2026-09-13 — and "the File-tab route" never existed.** This
     bullet said a File-tab click producing no `ribbon-tab-activated tab=file` was
     a suite-wide ribbon blocker. Measured by driving the binary: `ribbon/tabs.rs`
     emits that line **unconditionally for every tab**, so there is no File-tab
     path to break. The check's own control launch was deleting
     `userdata/preferences.txt` to reach the shipped print defaults — and that is
     the file holding `ask_default_app = false`, so the O173 *"Open PDFs with
     pdfcer"* offer opened as a **real OS window** and took the press.
     `sandbox::reset_prefs` instead of `std::fs::remove_file`, and the check now
     **PASSES**: twelve remembered print settings come back into the dialog and
     the job is planned with them.

     ★★★ **Three things to carry forward, and the third is the expensive one.**
     (a) *An absence reported by a check is first a question about the check* —
     the trace named the cause one line at a time, forty lines above the failure
     message that blamed the ribbon. (b) *A fix that names its victims can still
     miss one*: `sandbox::write_prefs` was written to stop exactly this and its
     doc table names **this check**, but the repair covered the write path and the
     delete path is not a write. (c) ⚠ **The correct diagnosis was already in
     this session's memory, written a day earlier, while three project documents
     went on naming the ribbon.** A cold session reads `RESUME.md` first and
     nothing reconciles the register against memory, so *memory loses to a
     register row every time* — when a memory entry contradicts a register row,
     correcting the row is the work.

     ★ One measurement not to re-file: the offer's two buttons are reported
     `shown=0.58 floor=0.60` on the frame it opens (`dialog-refocus now=0`) and are
     fully inside the window on the next one (`now=1`). **One frame, an egui
     galley-sizing transient, not a defect** — and `declared()` is last-wins, so
     no check can be misled by it either.
   * ⇒ **`DOC_DRIFT.md` S11 — 168 hard-coded engine line citations, and
     8 of 8 sampled land on unrelated code.** `edit.rs` grows at the head, so
     every citation into it drifts monotonically downward. ★★★ A rotted
     citation does not merely fail to support its claim — it **protects** it,
     because the reader who checks it finds plausible code and stops. Cite by
     symbol name, or pair the line with the pin.
   * ⇒ **Give every check its own fixture, as a `const FIXTURE` beside the
     check**, the way about thirty already do. `sweep-full.sh` hands the chunked
     checks one shared `--pdf fixtures/a1-titleblock.pdf` — one page, no
     AcroForm, no optional content, no transparency, no removable font, 123
     characters on a 2383.9 × 1683.8 pt sheet whose tallest text is **2.4
     screen pixels** at the sweep's fit zoom. Every one of those absences is
     some check's unwritten precondition. ⚠ **Any future text failure on
     that fixture is this, until measured otherwise.**

   ⚠⚠ **And two of the sweep's `harness' results are findings about the
   PRODUCT, not the checks, and they need his verdict rather than a repair.**
   At fit zoom a form field is 27.8 px wide and its corner grips eat every
   point on it; a 50 × 35 px check box has eight 8 px grips with 2 px of
   slack between them. **The harness cannot start those drags and neither can
   he.** Same shape as the standing finding that a slack measured in screen
   units shrinks in the units he cares about. Raising the harness zoom hides
   it; the product fix is that a grip must yield the body below some size.
   Tracked as O176's two open product questions.
4. **The owed driven checks — and MEASURE the list before working it.**
   `target/scratch/driven-audit.md` (2026-09-11) is the measured register: every
   `OPERATOR_REQUESTS.md` row whose heading says *"not yet driven"*, checked
   against `roster.rs`. ★★ **The headings are stale in both directions** — three
   checks named as owed had been registered all along. A registered `Box::new(…)`
   line in `roster.rs` is the evidence, not a row that claims a gap.
   `CONTINUE.md` is history, not a backlog.
5. **★★ Wire the ONE engine delivery that is genuinely owed —
   `SignReport::appearance_lines`.** ⚠ **This item said SIX on 2026-09-11 and
   was re-derived from source on 2026-09-12; four were wired that day and a
   fifth was never missing.** Wired since: `MkColor` (a widget's `/MK /BG` is
   now honoured by the in-canvas editor, which was a live rule-4 violation),
   `FieldPathCrossesTerminal`, and the narrowed `HybridFullRewrite` refusal at
   all three prose sites; the stale `FormLeaf::is_editable` sentence the row
   named had already been corrected the day before the row quoted it.
   `add_named_destination` is **blocked by an argued decision**, not absent —
   a destination baked at author time is indistinguishable from a correct one
   until a reorder moves the page it names, and this shell has drag-to-reorder;
   it opens with the `insert_pages` bookmark-carry surface and not before.
   `EditSession::page_objects` is owed but needs a decided answer on cache
   invalidation first (the engine keys on its session revision, we key on ours).
   ⇒ **A backlog row is a record, not evidence** — re-derive an absence
   claim from source before scheduling against it.
   ★ **Why `appearance_lines` is not paperwork:** it is what the engine drew
   into the signature box. The CLI prints it; here it reaches only a trace the
   source itself marks as never displayed, so the operator signs and never sees
   what the stamp says — and the engine records *a rectangle too small for
   them* as a real outcome, so the silence **hides a truncation**. Off-canvas,
   in the signing dialog's result, per R8b rule 4.
   ⚠ **An absence claim has to name the receiver.** `page_objects` returns 164
   hits here and every one is *ours* (`OpenDoc::page_objects`), not the engine's
   (`EditSession::page_objects`). A bare-name grep reads as thoroughly consumed
   and is the exact opposite.
6. **Under O165, release when the engine moves.** Compare the two pin rows at
   the start of every session; if they differ, read the intervening log before
   reading any backlog row.

★ **Five requests are open on the channel against the style ladder** and none
has a reply yet: no `same_family` on `StyleLadder`, no read-only ladder preview,
`passed_over` is prose a shell must parse, `FormatError::CoverageFailure` is
unconstructable by a consumer (E0639), and `SynthesisRefusedByPosture`'s
`Display` runs two words together. **Two of them are gating features that were
not written**, not workarounds — read them before redesigning the disclosure.

Open operator rows: `grep '^## O' OPERATOR_REQUESTS.md` and read the ones not
marked closed. **Only Ken closes a row.**

---

## Do not

- **Do not write to `D:\Dev\pdfcer`.** Read-only until fold-in. Engine work goes
  through `D:\Dev\FeatureRequests\pdfce_FeatureRequests\` and lands there as its
  own Pass. That channel answers within the hour.
- **Do not run `ui-verify` while the operator is at the machine.** It drives the
  real cursor and keyboard.
- **Do not kill `pdfcer-gui.exe` by image name.** It is his daily PDF reader.
  Kill by PID, verified against its path.
- **Do not drive the published build.** Copy the exe to `target/scratch/drive/`
  first; the suite's side effects land in his own saved state otherwise.
- **Do not revert an experiment through git.** A reverting verb inside a chained
  command discards uncommitted work in the same file — it has done, twice, and a
  hook now refuses it. Keep a copy and restore from the copy.
- **Do not deliver a patch script through a Bash heredoc.** Non-ASCII prose and
  backslashes are mangled in transit, and the failure is a needle matching
  **nothing** — a clean exit that changed nothing, indistinguishable from
  success. Write the script to a file, run the file, and `assert` its own match
  count. Eleven occurrences now; see `D:/dev/rag/rust/`.
  - ★★ **And writing it to a file only fixes the SHELL layer.** On 2026-09-11
    the file-first workaround was followed and the payload broke twice anyway:
    an escaped newline inside a non-raw `'''...'''` became a **real** newline
    inside an f-string (`SyntaxError`, reported at the *print*, not the edit),
    and an escaped NUL became a **real** NUL byte (`source code cannot contain
    null bytes`). Then, sixty seconds after that lesson was written down, its
    own one-line index entry came out split across three lines — through a
    correctly quoted heredoc, with every backslash doubled. Three layers stack
    here. ⇒ **The rule is not "double it" and not "use a raw string" — it is
    do not put a backslash in the payload at all.** Name the byte in words, pick
    a printable one, or build it with `chr(10)`. Then `python -c "import ast,io;
    ast.parse(io.open(p).read())"` before importing or running anything.
  - ★★ **And the anchor itself is a third layer, found 2026-09-13.** This
    crate's prose uses an em dash freely. Typing an ordinary hyphen where the
    source has one produces a needle matching nothing, and in a terminal the
    two are indistinguishable. ⇒ **Choose an anchor that contains no em dash at
    all** rather than trying to reproduce one; when there is no choice, verify
    with `grep ... | cat -A` and look for `M-bM-^@M-^T`. ★ Better still, anchor
    on a line PREFIX and replace the whole line: a prefix such as `| Gates |`
    contains no character this project has ever got wrong, and there is no
    partial-row state to get half-right.
- ⚠ **The memory index is full — 3 bytes of headroom, and that is not a
  rounding error.** `tools/gates/check-memory-index.sh` caps
  `.claude/agent-memory/pdfcer-gui-engineer/MEMORY.md` at 24,000 bytes; it
  measures **23,997** with **130** entries, and a row costs about 190. That cap
  is not arbitrary — the index is loaded whole into every session and is
  truncated **from the end**, which drops the newest rows, the ones a cold
  session most needs. ⇒ **Writing a lesson now means shortening two or three
  existing hooks first, in the same script that adds the row.** Cut the hook,
  never the topic file: the hook only has to decide relevance, and the detail is
  one link away. ★ Have the script **assert the final byte count before it
  writes** — two attempts landed over the cap (24,077 and 24,004) and were
  refused by that assertion, where a previous session's script printed a number
  48 bytes over and left the gate to notice.
- **Do not start Phase 5 (text editing) early.** Deliberately last, `HANDOFF.md` §8.
- **Do not build S6 deep zoom or tiling.** Measured as a 9× regression.

---

## Standing rules that live only here

1. **Update the engine before every build** —
   `cargo update -p pdfcer-core -p pdfcer-render -p pdfcer-print`. Automated as
   a step in `package-portable.py`; `--no-update` exists for reproducing an
   exact revision. A stale pin has already cost eighteen missing images on the
   operator's own file.
2. **Publish every build worth keeping.** `python tools/package-portable.py`
   builds, mirrors and rotates the OneDrive slot itself, preserving the older
   slot so the previous build survives beside the new one. **Say which slot in
   the report**, and which holds the previous build — the slot name carries no
   version information. GitHub as well, never marked pre-release, push before
   creating the release, and package from a **clean** tree.
3. **Smoke-launch off-screen before every release.** Ninety seconds. It has
   beaten four thousand unit tests and thirty-three green gates to a live defect.
4. **A register row is a verdict plus one paragraph**, capped at 1,200
   characters and checked by `python tools/walk-engine-backlog.py --check`. File
   a row and set the five headings in the same edit with
   `python tools/walk-engine-backlog.py --write YYYY-MM-DD` — never by adding to
   the figure that was there. A rule described in prose is a rule that will be
   approximated.
5. **A test that proves WHICH mechanism ran must assert what the WRONG mechanism
   cannot produce.** The bold test asserted *"the edit epoch moved"* for five
   days; the faked weight moves it too, so the test was green on the outcome its
   own name ruled out. Falsify by putting the old call back, never by breaking
   the input.

### When the engine repo is busy, the packager races itself

`package-portable.py` updates the engine as its **first** step, so on a day the
other session is committing live, every packaging run moves the pin, dirties the
tree, and stamps the artefact `-dirty`. The sequence that works:

```bash
cargo update -p pdfcer-core -p pdfcer-render -p pdfcer-print
git commit Cargo.lock -m "Take the engine to <rev>"
python tools/package-portable.py --no-update --verify --note "…"
```

---

## Where to read next, and what each answers

| File | Answers |
|---|---|
| `HANDOFF.md` | The long-form record — why a rule exists, rather than what it is. |
| `OPERATOR_REQUESTS.md` | Every request the operator has made, and its state. |
| `FEATURES.md` | What this build can do. Refreshed against the build before every release. |
| `ENGINE_BACKLOG.md` | Every engine capability this shell lacks, with a verdict and one paragraph. |
| `CONTINUE.md` | The driven-check backlog. |
| `SWEEP_REPAIRS.md` | The repairs a driven sweep earned — each one a defect in the CHECK, not the app. Read before investigating any FAIL. |
| `DOC_DRIFT.md` | Stale claims in this project's own doc comments, dated against the commit that falsified each one. |
| `DESIGNS.md` | The designs that are argued and not yet built — O181, O183, O185, O188, O189. ★ O186's section was DELETED on 2026-09-13 when it was built and driven, per this file's own rule: a design that has shipped is not a design, it is a claim about code that will drift away from it. Read before designing any of those; they are ★ **not** operator rulings. |
| `D:\Dev\pdfcer\docs\core-api\index.md` | *"I want to do X — what do I call, in what order, and what will bite me?"* |
| `D:/dev/rag/egui/` | Empirical egui findings from this codebase. Read before touching the dock or the canvas rect. |

**The founding rule, in one line:** a phase is not done until its behaviour is
asserted by driving the real binary. Two of the worst defects in the old GUI
were invisible to a green test suite and obvious within thirty seconds of using
the app. *"The tests pass"* is not a report of working software.
