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

| What | Command | Value, 2026-09-12 07:16 (the release cut) |
|---|---|---|
| Engine pin | `grep -m1 'pdfcer?branch=main#' Cargo.lock` | `ad73160` |
| Engine HEAD | `cd /d/Dev/pdfcer && git log --oneline -1 main` | `ad73160` — ★ **level with the pin**, for the first time in four releases. Level is the only state in which a sentence of the form *the engine cannot do X* is safe to write. |
| Last release | `git fetch --tags origin && git describe --tags --abbrev=0` | `v0.5.0-dev.20260912.1` - cut 2026-09-12 07:16, 15 commits, verified as what `releases/latest` advertises |
| Driven checks | `ui-verify --list \| grep -cE '^  [a-z0-9_]+$'` | 213 (the sweep chunks 211 and runs 2 from the ALONE table separately) |
| Gates | `bash tools/gates/run-all.sh` | **41 passed, 0 failed, 0 skipped** — ⚠ **41 gates are registered, not 39**; the runner prints its own tally and that is the only figure to quote. A hand-written 39 stood here and in `FEATURES.md` while `cargo fmt` and `cargo clippy` were red. — ★★ up four, and `check-pin-citation` went **red** on the run before this table was patched, naming both stale pin citations. It reads the row above. |
| Unit tests | `cargo test --workspace` | **4,199 passing**, 51 ignored — summed over 24 `test result:` lines, which is the only method that counts this workspace correctly. |
| Source files | `find crates -name '*.rs' \| wc -l` | 777 |
| Backlog register | `python tools/walk-engine-backlog.py` | 172 rows — wanted 35 / blocked 8 / unknown 0 / declined 13 / **shipped 116** |
| Request channel | `ls /d/Dev/FeatureRequests/pdfce_FeatureRequests/open \| wc -l` | 33 — ★ **one reply is genuinely unconsumed and it is meant to be**: `reply_G007` accepted the topic-key convention itself, asked nothing and delivered nothing, so a `done_G007` would acknowledge an acknowledgement. Leaving it red is what keeps the *reply with no done* check falsifiable. |

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

★★★ **`v0.5.0-dev.20260912.1` shipped 2026-09-12 07:16** — the canvas
context menu, which had been deleting itself under the cursor since 2026-09-10,
plus the shaded-field editor and the dotted-name refusal. Verified as what
`releases/latest` advertises, with the packager's own zip as the only asset.
Mirror in **`OneDrive\pdfcer-gui2`** (07:14); **`pdfcer-gui1` holds `.4`**
(2026-09-11 20:05), so the fallback survives. **The next package rotates into
slot 1** — and the rotation works this out for itself from the two slots'
timestamps, so pass `--slot` only to RETRACT, never to steer a normal build.

⚠ **It packaged from a clean tree and the name proves it**:
`pdfcergui-20260912-0714-ad73160-2ec99f5`, no `-dirty` suffix. That is the
first release in three where it is true, and the reason is `--no-update`:
the engine had nothing to pick up, so the packager was not asked to rewrite
`Cargo.lock` thirty seconds before stamping the tree it had just modified.

⚠ **`BUILD-INFO.txt` in that zip says verification was *not run*.** It means
the packager did not re-run it, not that it was not done — 41 gates and 4,199
tests were measured against this exact source before the commit, and the
release notes say so. ⇒ If a future release wants that line to read green,
the flag is `--verify` and it costs a test run plus a gate sweep.

⚠ **The number in a release title is the release count for that DAY, not the
session's.** The previous edition of this block called `.3` *"the fifth release
of the day"* and it was the fourth. If you need the count, `gh release list`.

---

## ★★★ The sweep RAN — 213 checks, and what it changes

**2026-09-12, 04:34, against this exact binary.** The first full driven sweep
in three weeks, and the first one any release has ever had.

```
=== TALLY passed=162 failed=11 skipped=40   (213 checks)
```

**1 application defect. 30 harness defects.** Read that ratio before you read
any single check's failure message — 21 of the 30 come from one shared fixture
or one shared aim point, 5 checks accused a named application module wrongly,
and 4 were refuted by a line their own trace already contained.

★★★ **The one application defect was real and two days old: every canvas
context menu deleted itself as the cursor moved onto it.** `bfc8dea`
(2026-09-10) changed the menu's anchor from a fixed response to a per-frame
choice, and since egui derives a popup's id from its anchor — while
`contains_pointer` is layer-aware, so an open menu makes its own anchor stop
containing the pointer — the menu re-attached under an id nobody opened and
`Memory::end_pass` dropped it. **Silent: no close call, no event, no trace
line.** Fixed, falsified, unit-tested, and written up in `HANDOFF.md` §10 and
`D:/dev/rag/egui/`. ⚠ It survived 4,193 unit tests and 41 gates because **no
driven check had ever activated a `menu.item.*` row** — publishing a rect and
being clickable are different claims.

⇒ **The harness repair list is now the biggest single body of owed work in
this repository**, and it is all `.rs`, none of it blocked on the desktop, so
it can be done at any time. It is item 2 under *Do next*.

---

## ⚠ Two things still owed against the shipped build

The sweep is no longer one of them — it ran — and **the three off-page
pointer checks are discharged: all three PASSED in it**, on their own
`off-page-object.pdf` fixture. Two items are left, and only the first needs
the desktop.

1. **The bold ladder is WIRED, NOT DRIVEN.** Shipped in `.3` with that caveat
   printed in his release notes. The unit evidence is strong (the assertion
   moved from *"an edit happened"* to *"the run's `/Font` resource key changed"*
   and was falsified by putting the old call back), but nobody has pressed Bold
   on a title block in the running program. **If he reports bold misbehaving,
   this is the first place to look.**
   ★★ **And the sweep did not discharge it, which is a sharper fact than
   “not driven yet”:** the three checks that would have
   — `restyling_selected_text_reaches_the_document`,
   `the_face_chooser_offers_a_face_the_document_does_not_contain`,
   `the_format_tab_offers_font_controls_for_swept_text` — all ran and all
   **skipped**, each saying so honestly: the shared `--doc-point` selected a
   `Path`, not text. ⇒ The repair is an aim point on text, not a rerun.
2. **The deep-marquee adoption has no driven check at all.** Our duplicate
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
2. **★★★ Work the harness repair list the sweep produced — 30 defects, all
   `.rs`, none of them needing the desktop.** This is the largest owed body of
   work in the repository and the cheapest, and until it is done **the next
   sweep will report the same thirty things in thirty different words.**

   Three repairs cover most of it and each is one idea, not a list:

   * **Give every check its own fixture, as a `const FIXTURE` beside the
     check**, the way `off_page_census.rs` already does. `sweep-full.sh` hands
     all 211 chunked checks `--pdf fixtures/a1-titleblock.pdf`, a file that is
     one page, `/Rotate 0`, no AcroForm, no optional content, no transparency
     and no removable font — and every one of those is some check's unwritten
     precondition. Ten checks reported ten unrelated-sounding problems from
     that one decision. Named fixtures: `four-pages.pdf`,
     `four-pages-unrotated.pdf`, `layered-drawing.pdf`, `transparency-cmyk.pdf`,
     `text-field-with-appearance.pdf`, `embedded-font.pdf`, `paragraph.pdf`,
     `polyline-nodes.pdf`, `autosize-field.pdf`.
   * **Derive the aim point from the crop box in the trace, never type it.**
     `measure_calibrates_by_picking_two_points` missed the page by **16 points
     out of 2,384** — a typed 2400 against a crop box ending at 2383.94.
   * **Drive the text checks on a letter-size fixture, or at a higher zoom.**
     ★★★ `a1-titleblock.pdf` carries 123 characters on a 2383.9 × 1683.8 pt
     sheet and at the sweep's fit zoom **the tallest text on it is 2.4 screen
     pixels**. Sixteen checks reported sixteen reasons for that one fact. ⚠ Any
     future text failure on this fixture is this, until measured otherwise.

   Then the singletons, each already diagnosed:
   `redaction.rs::click_region` must use `frame_for`/`frame_of` (a dialog click
   that has not landed since 2026-08-21); the two zoom checks expect a thread
   panic and should say so with `expect_thread_panic()` on `raster-limit`;
   `double_click_text` needs the application to emit `canvas-pick-class`;
   `signature_trust_is_reported_as_its_own_fact` never opens the Signatures
   panel; `rotate_handle_turns_a_selection` and `shift_constrains_a_resize`
   need aim `0,300,500`; `print_remembered` must reach the shipped defaults by
   `sandbox::write_prefs(&dir, "")` rather than by DELETING the file — ★ the
   write-path fix for the first-run offer was correct and simply did not apply
   to a check that deletes instead of writing.

   ⇒ **Delete** `a_save_that_would_produce_blank_pages_is_refused` and the
   engine half of `app::save::tests::deleting_a_page_from_a_nested_document_is
   _caught_at_the_save`: the engine is fixed, the guard walked a real four-level
   tree and found nothing, and a check that cannot fail is not evidence. **The
   guard itself stays** — one tree walk at save time is all that stands between
   a regression and a file that opens in Acrobat with blank pages on the end.
   Close `request_delete_pages_leaves_ancestor_count_stale_on_a_nested_page
   _tree.md` with it.

   ⚠ **And two of the 'harness' results are findings about the PRODUCT.** At
   fit zoom a form field is 27.8 px wide and its corner grips eat every point
   on it, and a 50 × 35 px check box has eight 8 px grips with 2 px of slack
   between them. **The harness cannot start those drags and neither can the
   operator.** Same shape as the standing finding that a slack in screen units
   shrinks in the units he cares about. Raising the harness zoom hides it;
   the product fix is that a grip must yield the body below some size.
3. **The owed driven checks — and MEASURE the list before working it.**
   `target/scratch/driven-audit.md` (2026-09-11) is the measured register: every
   `OPERATOR_REQUESTS.md` row whose heading says *"not yet driven"*, checked
   against `roster.rs`. ★★ **The headings are stale in both directions** — three
   checks named as owed had been registered all along. A registered `Box::new(…)`
   line in `roster.rs` is the evidence, not a row that claims a gap.
   `CONTINUE.md` is history, not a backlog.
4. **★★ Wire the ONE engine delivery that is genuinely owed —
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
