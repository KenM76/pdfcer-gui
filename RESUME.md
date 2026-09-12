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

| What | Command | Value, 2026-09-11 22:15 |
|---|---|---|
| Engine pin | `grep -m1 'pdfcer?branch=main#' Cargo.lock` | `01c4a10` |
| Engine HEAD | `cd /d/Dev/pdfcer && git log --oneline -1 main` | `21403ff` — **one ahead, and it is docs-only** |
| Last release | `git fetch --tags origin && git describe --tags --abbrev=0` | `v0.5.0-dev.20260911.4` |
| Driven checks | `ui-verify --list \| grep -cE '^  [a-z0-9_]+$'` | 213 (the sweep chunks 211 and runs 2 from the ALONE table separately) |
| Gates | `bash tools/gates/run-all.sh` | **35 passed, 0 failed, 0 skipped** |
| Unit tests | `cargo test --workspace` | **4,189 passing**, 51 ignored |
| Source files | `find crates -name '*.rs' \| wc -l` | 775 |
| Backlog register | `python tools/walk-engine-backlog.py` | 171 rows — wanted 37 / blocked 8 / unknown 0 / declined 13 / **shipped 113** |
| Request channel | `ls /d/Dev/FeatureRequests/pdfce_FeatureRequests/open \| wc -l` | 30 — **many are closed exchanges awaiting archival** |

⚠ **The check-count command was wrong in this very table.** `--list` prints
**two** lines per check plus a header, so `wc -l` answered 434 where the
answer is 213. It had been quoted as 212 for long enough that the drift went
unnoticed — in the table whose own heading says *re-measure, never quote*.
The row now carries a command that counts check NAMES. If you see 434
anywhere, it is this defect.

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

★★★ **`v0.5.0-dev.20260911.4` shipped 2026-09-11 20:09** — the render report
naming the blend space and who chose it, plus the redaction-override ruling.
Verified as what `releases/latest` advertises. Mirror in
**`OneDrive\pdfcer-gui1`** (20:05); **`pdfcer-gui2` holds `.3`** (19:50), so the
fallback survives. **The next package rotates into slot 2.**

⚠ **The number in a release title is the release count for that DAY, not the
session's.** The previous edition of this block called `.3` *"the fifth release
of the day"* and it was the fourth. If you need the count, `gh release list`.

---

## ⚠ The four things owed against the shipped build, all blocked on the desktop

**None of the last five releases had the driven sweep run against it.** Unit
tests, 33 gates and an off-screen smoke launch were green every time; not one of
the three is evidence that a control is *reachable*. The blocker is not time —
the operator is at the machine, and `ui-verify` drives the real cursor and
keyboard. Confirm the desktop is free before scheduling any of this.

1. **The bold ladder is WIRED, NOT DRIVEN.** Shipped in `.3` with that caveat
   printed in his release notes. The unit evidence is strong (the assertion
   moved from *"an edit happened"* to *"the run's `/Font` resource key changed"*
   and was falsified by putting the old call back), but nobody has pressed Bold
   on a title block in the running program. **If he reports bold misbehaving,
   this is the first place to look.**
2. **Three off-page pointer checks**, each of whose `PDFCER_DIAG_INVOKE` string
   was changed to open in **Edit** because their subject is off the sheet and
   Read now hides it: `an_object_off_the_page_survives_being_zoomed_in_on`,
   `a_band_dragged_into_the_margin_reaches_an_object_off_the_page`,
   `a_band_that_starts_in_the_margin_reaches_an_object_off_the_page`. **An
   instrument edited but not run is the failure mode this harness exists to
   remove.** `an_object_off_the_page_is_actually_drawn` needs no pointer and
   **was** run, green.
3. **The deep-marquee adoption has no driven check at all.** Our duplicate
   `hit_test_rect` is deleted and the engine's `hit_test_rect_deep` answers
   instead; a band and a click can no longer disagree, and nothing has confirmed
   that by dragging one.

---

## Do next, in the operator's order

> ★★★ **Order this list by what the operator reaches for, not by what just
> arrived in the engine channel.** On 2026-08-18 print paper sizes got built
> while clicking a stamp did nothing at all. The engine's replies are an input
> to *how* a thing gets built, never to *which* thing.

1. **When the desktop frees up, drive the four owed items above** — the ladder
   first, because it is what he was given today and what he will notice.
2. **Read the sweep result, then triage every FAIL and every SKIP.** The full
   driven sweep ran on 2026-09-11 for the first time in three releases; its log
   is `target/scratch/sweep-full.log`. ★★ **`=== SWEEP-DONE` alone means
   nothing** — the runner prints `=== TALLY passed=… failed=… skipped=…` beside
   it, and a sweep whose `passed` is zero ran nothing at all. That is not
   hypothetical: the first attempt that day reported SWEEP-DONE having launched
   no window, because the harness binary was stale. And per standing memory,
   **the first full sweep after a long gap yields more harness defects than
   application ones** — audit the check before filing an app defect against it.
3. **The owed driven checks — and MEASURE the list before working it.**
   `target/scratch/driven-audit.md` (2026-09-11) is the measured register: every
   `OPERATOR_REQUESTS.md` row whose heading says *"not yet driven"*, checked
   against `roster.rs`. ★★ **The headings are stale in both directions** — three
   checks named as owed had been registered all along. A registered `Box::new(…)`
   line in `roster.rs` is the evidence, not a row that claims a gap.
   `CONTINUE.md` is history, not a backlog.
4. **★ Wire the remaining SIX engine deliveries this shell does not call.** The
   2026-09-11 channel audit found eight with zero call sites; **two are now
   done** — the automatic bold ladder and `hit_test_rect_deep`. Grep
   `ENGINE_BACKLOG.md` for `**★ 2026-09-11 channel audit.**` to get the set. The
   six left: `SignReport::appearance_lines`, `EditSession::page_objects`,
   `MkColor` (**a rule-4 violation live in the build** — a widget's `/MK /BG` is
   painted over with theme grey, so the canvas shows something the saved file
   will not), `add_named_destination`, `FieldPathCrossesTerminal`, and the
   narrowed `HybridFullRewrite` refusal whose prose is still too wide at three
   sites (`app/save.rs`, `dialogs/compact.rs`, `app/dispatch.rs`) plus a stale
   `FormLeaf::is_editable` sentence in `canvas/moving/mod.rs`.
   ⚠ **An absence claim has to name the receiver.** `page_objects` returns 164
   hits here and every one is *ours* (`OpenDoc::page_objects`), not the engine's
   (`EditSession::page_objects`). A bare-name grep reads as thoroughly consumed
   and is the exact opposite.
5. **Under O165, release when the engine moves.** Compare the two pin rows at
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
| `D:\Dev\pdfcer\docs\core-api\index.md` | *"I want to do X — what do I call, in what order, and what will bite me?"* |
| `D:/dev/rag/egui/` | Empirical egui findings from this codebase. Read before touching the dock or the canvas rect. |

**The founding rule, in one line:** a phase is not done until its behaviour is
asserted by driving the real binary. Two of the worst defects in the old GUI
were invisible to a green test suite and obvious within thirty seconds of using
the app. *"The tests pass"* is not a report of working software.
