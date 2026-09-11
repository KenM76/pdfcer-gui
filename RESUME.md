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

| What | Command | Value, 2026-09-11 |
|---|---|---|
| Engine pin | `grep -m1 'pdfcer?branch=main#' Cargo.lock` | `1eb1c7c` |
| Engine HEAD | `cd /d/Dev/pdfcer && git log --oneline -1 main` | `1eb1c7c` — **current, v0.52.0** |
| Last release | `git fetch --tags origin && git describe --tags --abbrev=0` | `v0.5.0-dev.20260911.1` — **0 commits behind** |
| Driven checks | `cargo run -q -p ui-verify -- --list \| wc -l` | 212 |
| Gates | `bash tools/gates/run-all.sh` | **32 passed, 0 failed, 0 skipped** |
| Unit tests | `cargo test --workspace` | 4,161 passing |
| Source files | `find crates -name '*.rs' \| wc -l` | 771 |
| Backlog register | `python tools/walk-engine-backlog.py` | 170 rows |

⚠ **The pin is a claim with a shelf life of hours.** `Cargo.lock` pins the
engine by git revision, not by path, so every sentence in this repository of the
form *"the engine cannot do X"* is a statement about **a pin**, not about the
engine. Under O165 the first act of a session is to compare the two rows above
and, if they differ, read the intervening log **before** reading any backlog row.

⚠ **`git describe` LIES in a fresh clone or a repo that has not fetched.**
`gh release create` makes the tag **on GitHub**, not here — nothing pulls it
back. On 2026-09-11 the operator asked *"new release soon?"* and the answer
began **"51 commits have stacked up since Tuesday's build"**, computed from a
`describe` that had never seen the three releases cut since. After
`git fetch --tags origin` the true figures were **21 commits** and
**yesterday afternoon**. Both commands were right; the instrument was reading
local state about a fact that lives remotely. ⇒ **Any sentence containing
*"commits unreleased"* or a release date gets a `git fetch --tags origin`
immediately in front of it**, and a cross-check against `gh release list`.

⚠ **The last three releases shipped without the driven sweep**, this one
included. Unit tests, 32 gates and an off-screen smoke launch were green each
time; none of the three is evidence that a control is *reachable*. The
blocker today is not time — an Outlook *Internet Email* password prompt sits
at desktop 1134,407–1514,625, over the middle of the only display, and every
pointer-driven check SKIPs on it. **It is the operator's window; ask, never
close it.** Confirm it is gone before scheduling a sweep:
`Get-Process OUTLOOK | Select MainWindowTitle`.

---

## Do next, in the operator's order

> ★★★ **Order this list by what the operator reaches for, not by what just
> arrived in the engine channel.** On 2026-08-18 print paper sizes got built
> while clicking a stamp did nothing at all. The engine's replies are an input
> to *how* a thing gets built, never to *which* thing.

1. **Read the sweep result, then triage every FAIL and every SKIP.** The full
   driven sweep ran on 2026-09-11 for the first time in three releases; its log
   is `target/scratch/sweep-full.log`. ★★ **`=== SWEEP-DONE` alone means
   nothing** — the runner now prints `=== TALLY passed=… failed=… skipped=…
   codes: …` beside it, and a sweep whose `passed` is zero ran nothing at all.
   That is not hypothetical: the first attempt on 2026-09-11 reported
   SWEEP-DONE having launched no window, because the harness binary was stale.
   And per standing memory, **the first full sweep after a long gap yields more
   harness defects than application ones** — audit the check before filing an
   app defect against it.
2. **The owed driven checks — and MEASURE the list before working it.**
   `target/scratch/driven-audit.md` (2026-09-11) is the measured register: every
   `OPERATOR_REQUESTS.md` row whose heading says *"not yet driven"*, checked
   against `roster.rs`. ★★ **The headings are stale in both directions.** Three
   checks named as owed on 2026-09-11 —
   `load_anomalies_are_listed_in_document_properties`,
   `rereading_under_the_other_value_is_offered` and
   `double_clicking_a_text_box_edits_the_text` — had been registered all along;
   the naming document was the only thing that said otherwise. A registered
   `Box::new(…)` line in `roster.rs` is the evidence, not a row that claims a
   gap. `CONTINUE.md` is history, not a backlog.
3. **★ Wire the eight engine deliveries this shell does not call.** The request
   channel was audited end to end on 2026-09-11 and is **done**: `open/` went
   **229 → 13**, `archive/` 84 → 301, `INDEX.md` 79 → 228 rows, every citation
   re-measured as resolving. Do not re-triage it; read `open/` and believe it.
   **What came out of it is the work.** Eight capabilities the engine shipped —
   in several cases in answer to this shell's own request — have **zero call
   sites here**, each now a `wanted` row in `ENGINE_BACKLOG.md` opening
   `**★ 2026-09-11 channel audit.**` (grep that marker to get the set):
   the automatic bold ladder, `SignReport::appearance_lines`,
   `hit_test_rect_deep`, `EditSession::page_objects`, `MkColor`,
   `add_named_destination`, `FieldPathCrossesTerminal`, and the narrowed
   hybrid-rewrite refusal our prose still states too widely.
   ★★ **Start with `hit_test_rect_deep`**: it is not an absence but a
   **diverged duplicate** — our `provider::hit_test_rect` grew a `MarqueeMode`
   and a container filter (O88) the engine's version knows nothing about, so two
   implementations of one question now disagree. Its `INDEX.md` row said so on
   2026-09-05 and nobody read it for six days.
   ⚠ **An absence claim has to name the receiver.** `page_objects` returns 164
   hits in this repo and every one is *ours* (`OpenDoc::page_objects`), not the
   engine's (`EditSession::page_objects`). A bare-name grep reads as thoroughly
   consumed and is the exact opposite.
   The 13 still open: four asks the engine owes (two of them **the same
   tiny-skia region-render defect filed twice by us**), eight deliveries above,
   and the 2026-09-07 protocol note.
4. **Under O165, release when the engine moves.** Compare the two pin rows
   above at the start of every session; if they differ, read the intervening log
   before reading any backlog row.

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
   beaten four thousand unit tests and twenty-two green gates to a live defect.
4. **A register row is a verdict plus one paragraph**, capped at 1,200
   characters and checked by `python tools/walk-engine-backlog.py --check`. File
   a row and set the five headings in the same edit with
   `python tools/walk-engine-backlog.py --write YYYY-MM-DD` — never by adding to
   the figure that was there. A rule described in prose is a rule that will be
   approximated.

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
