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

| What | Command | Value, 2026-09-10 |
|---|---|---|
| Engine pin | `grep -m1 'pdfcer?branch=main#' Cargo.lock` | `15a1272` |
| Engine HEAD | `cd /d/Dev/pdfcer && git log --oneline -1 main` | `ca15151` — **the pin is stale** |
| Driven checks | `grep -c 'Box::new(' tools/ui-verify/src/checks/roster.rs` | 206 |
| Gates | `bash tools/gates/run-all.sh` | 30 gates plus self-tests |
| Unit tests | `cargo test --workspace` | re-run it |
| Source files | `find crates -name '*.rs' \| wc -l` | 757 |
| Backlog register | `python tools/walk-engine-backlog.py` | 162 rows |

⚠ **The pin is a claim with a shelf life of hours.** `Cargo.lock` pins the
engine by git revision, not by path, so every sentence in this repository of the
form *"the engine cannot do X"* is a statement about **a pin**, not about the
engine. Under O165 the first act of a session is to compare the two rows above
and, if they differ, read the intervening log **before** reading any backlog row.

⚠ **The last two releases shipped without the driven sweep.** Unit tests and
gates were green both times; neither is evidence that a control is reachable.

---

## Do next, in the operator's order

> ★★★ **Order this list by what the operator reaches for, not by what just
> arrived in the engine channel.** On 2026-08-18 print paper sizes got built
> while clicking a stamp did nothing at all. The engine's replies are an input
> to *how* a thing gets built, never to *which* thing.

1. **`bash tools/ui-verify/sweep-full.sh`** — first act of the next session that
   has the desktop. It takes the real cursor and keyboard, so it needs the
   operator off the machine. `stamp_dialog_reopen` (O171) and
   `default_app_offer` (O173) are both written and have **never executed**.
2. **Bump the engine pin** — `cargo update -p pdfcer-core -p pdfcer-render -p pdfcer-print`.
   The pin above is stale, and the intervening passes carry a fix for a live
   data-destroying defect in a verb this shell already calls: restyling a
   stamp's colour rewrote its label to the stamp name's default text.
3. **Wire the stamp size surface** — a size field in the stamp's properties
   panel. He asked twice, most recently 2026-09-09: *"still can't adjust the
   size of a stamp on the canvas, or by entering a different size in the
   properties box."* The engine has now shipped both halves; the backlog row
   names the calls.
4. **Place a custom stamp (O172)** — `place_page_artwork` landed in the engine
   on 2026-09-10 and is the half of O169 he will reach for first. It was
   blocked; it is not blocked now.
5. **The owed driven checks** — the four 2026-09-06 markup asks,
   `load_anomalies_are_listed_in_document_properties`,
   `rereading_under_the_other_value_is_offered`,
   `double_clicking_a_text_box_edits_the_text` (last known FAIL), O161, and a
   Find zoom-box readout check.

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
   beaten four thousand unit tests and thirty green gates to a live defect.
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
