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

| What | Command | Value, 2026-09-13 00:40 (the release cut) |
|---|---|---|
| Engine pin | `grep -m1 'pdfcer?branch=main#' Cargo.lock` | `d86cb19` |
| Engine HEAD | `cd /d/Dev/pdfcer && git log --oneline -1 main` | `d86cb19` — ★ **level with the pin.** Level is the only state in which a sentence of the form *the engine cannot do X* is safe to write. ⚠ The pin is a BRANCH pin and it has moved mid-session without a `cargo update`; re-read the lock in the same breath as quoting it. |
| Engine version | `grep -A1 'name = "pdfcer-core"' Cargo.lock` | `0.53.0` |
| Last release | `git fetch --tags origin && gh release list --limit 3` | `v0.5.0-dev.20260913.1`, cut 2026-09-13 05:52 UTC, `prerelease=false`, and it IS what `releases/latest` advertises. ⚠⚠ **Fetch first.** `gh release create` tags on the REMOTE, so `git describe` in a tree that has not fetched answers with an older tag — that mistake put *"51 commits unreleased"* in a report where the answer was 21. ⇒ **And never pass `--prerelease`**: GitHub hides a pre-release from `releases/latest`, so the front page went on advertising a three-day-old zip while a new one sat in the list. |
| Driven checks | `ui-verify --list \| grep -cE '^  [a-z0-9_]+$'` | **220** (the sweep chunks 219 and runs 1 from the ALONE table separately — its own first line says `sweeping 219 checks in chunks, plus the ALONE table`) |
| Gates | `bash tools/gates/run-all.sh` | **43 passed, 0 failed, 0 skipped** — 43 registered, up two on 2026-09-13: `check-gate-input-scope` and its self-test. ★ It audits the other gates' INPUT SETS, after the same defect was written **four** times: a checker that asks git which files exist is asking about the INDEX, so every file the current session wrote is invisible to it, and the tell is the TIMING — green before the commit, red after it, with nothing edited. ⇒ **Falsify anything in this class by planting the violation in an untracked file**; a gate with the hole cannot see one at all. ★ `check-pin-citation` went **red** on the run before this table was rewritten, naming both stale pin citations in `FEATURES.md` and here. It reads the row three above. |
| Unit tests | `cargo test --workspace` | **4,239 passing, 0 failed, 51 ignored, 4,290 defined** — summed over 24 `test result:` lines, then cross-counted by `cargo test --workspace -- --list \| grep -cE ': test$'` = 4,290. Two methods, because a summed figure nobody cross-checks is how the last count drift got in. |
| Source files | `find crates -name '*.rs' \| wc -l` | 785 |
| Backlog register | `python tools/walk-engine-backlog.py` | 173 rows — wanted 35 / blocked 8 / unknown 0 / declined 14 / **shipped 116** |
| Request channel | `ls /d/Dev/FeatureRequests/pdfce_FeatureRequests/open \| wc -l` | 33 — newest mtime **Sep 12 03:55**, and the pin was committed Sep 12 17:59, so **nothing on the channel is newer than the build** and there is no reply to triage. ★ `reply_G007` stays unconsumed on purpose: it accepted the topic-key convention itself, asked nothing and delivered nothing, so a `done_G007` would acknowledge an acknowledgement. |
| Registered commands | `grep -rhoE '\bcommand\(' crates/pdfcer-gui/src/shell/commands/catalog/ \| wc -l` | 162 |
| Dockable panels | `grep -n 'pub const ALL' crates/pdfcer-gui/src/panels/mod.rs` | 13 |

⚠ **The check-count command was wrong in this very table, and the warning
that replaced it then went stale too.** `--list` prints **two lines per check
plus an eight-line header**, so a `wc -l` answers `2n + 8` — roughly double
the real figure, which is why it looked plausible. Measured against
`target/release/ui-verify.exe` on 2026-09-13: **220** names, **448** lines.

★★★ **Write the relationship, not the pair of literals.** The first repair of
this note said *"`wc -l` answered 434 where the answer is 213"*. Both numbers were
correct the day they were written and both moved the next time a check was
added, so the note that exists to stop count drift had drifted — sitting three
lines beneath a row that said 220. A lesson spelled as two constants expires;
one spelled as `2n + 8` cannot. ⇒ **If a count here is about double what you
expect, you counted lines.**

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

★★★ **`v0.5.0-dev.20260913.1` shipped 2026-09-13 05:52 UTC** — five of the
thirteen `FEATURE.txt` rows, the deep-zoom raster refusal (O186 part three), and a
front page written for a person. Verified as what `releases/latest` advertises,
`prerelease=false`, with the packager's own zip as the only asset. **17 commits**
since `.20260912.1`, counted after a `git fetch --tags origin`.

★ **Both destinations, and the fallback survives.** Mirror in
**`OneDrive\pdfcer-gui1`** (2026-09-13 01:50); **`pdfcer-gui2` still holds
`v0.5.0-dev.20260912.1`** (2026-09-12 07:14). **The next package rotates into slot
2** — and the rotation works this out for itself from the two slots'
timestamps, so pass `--slot` only to RETRACT, never to steer a normal build.
⚠ His own install at `C:\Users\Ken\OneDrive\pdfcer\` is **not**
touched by a publish and was not touched by this one.

✓ **Clean tree, and `--verify` this time**, so `BUILD-INFO.txt` carries
`tests: PASS` and `gates: PASS 41/41` rather than *not run*:
`pdfcergui-20260913-0139-d86cb19-b4e3137`, no `-dirty` suffix, source digest
`707f02b6c188`.

⚠ **The stamp names `b4e3137` and the tag is on `0f51565`.** The difference is
one commit of agent-memory markdown, which is outside both `BUILD_AFFECTING` and
`SOURCE_GLOBS`, so the shipped bytes are the bytes of the tagged tree. Stated here
because a reader comparing the two will otherwise go looking for a program
difference that does not exist — and because the honest fix next time is to
commit everything, memory included, before launching the packager.

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
nothing edited. That audit is not done and is the cheapest item on the register.

⚠ **The number in a release title is the release count for that DAY, not the
session's.** An earlier edition of this block called `.3` *"the fifth release of
the day"* and it was the fourth. If you need the count, `gh release list`.

---

## ★★★ The sweep ran again — 220 checks, and NOTHING went green to red

**2026-09-12 22:42 — 2026-09-13 00:22, against the binary that ships,
taking the real cursor and keyboard for one hour forty.** The second full
driven sweep in a day, and the first run after a tranche of harness repairs
landed.

```
=== TALLY passed=185 failed=4 skipped=31 codes: rc=1 rc=3   (220 checks)
```

Against the morning's `passed=162 failed=11 skipped=40` over 213 checks:
**failures 11 → 4, skips 41 → 31, and not one check went from green
to red.** That last clause is the one worth having. A release is allowed to
ship with known failures standing; it is not allowed to break something that
was measured working, and the only way to tell those apart is to run the whole
suite rather than the part that changed.

★★★ **All four remaining failures are defects in the CHECK, not the
program, and each is written up with the line of its own evidence that refutes
it — `SWEEP_REPAIRS.md`, rows R4, R6, R8 and R9.** Read that file before
investigating any FAIL in this suite. The clearest of the four: a check
reported that *the shell built no plan* on a click into real text, and three
lines above its own assertion the trace shows the shell detect no run under
the click, convert the gesture to an Add, place the text and disclose the font
substitution off-canvas exactly as rule 4 requires. Four correct behaviours
read as silence, because the oracle greps three event names and `add-text` is
not one of them.

### The SKIP diff, in both directions, because a SKIP is not red

```
comm -23 now morning  ->  pages_stay_drawn_when_you_scroll_back     (1 new)
comm -13 now morning  ->  11 names the repairs turned back ON
```

Baselines: `target/scratch/sweep-skips-2026-09-12.txt` (41 names) and
`sweep-fails-2026-09-12.txt` (11); tonight's are `sweep-skips-now.txt` (31)
and `sweep-fails-now.txt` (4). **Do this diff before quoting any tally, in
both directions.** A previously-passing check falling into a SKIP moves the
`failed=` figure not at all.

★ **Those four files are DATED RECORDS and are not edited when a name is
closed** — a baseline that gets corrected is no longer a baseline. Struck since
it was taken, by name: `the_print_window_opens_on_the_settings_you_last_used`
(2026-09-13, and it **PASSES**, it did not merely stop skipping). ⇒ So the next
sweep's diff against this baseline should show that one name leaving the SKIP
set and **arriving in the pass set**; it landing in neither is a finding about
the sweep, not about the check.

★★ **All eleven reactivated checks PASSED**, which is what discharges
both items that used to sit in the section below. And the one new SKIP is not
a regression — it is `SWEEP_REPAIRS.md` R10, a check finally given the
eight-page document it had been asking for that still could not reach its
subject, because the page cache **fills itself by the very act of scrolling
through it**. Its skip message blames the document and the view mode; its own
trace falsifies both (`pages=8`, `display=continuous`).

⇒ **The harness and doc-drift registers are now the biggest owed body of
work in this repository**, all of it `.rs` or markdown and none of it blocked
on the desktop: `SWEEP_REPAIRS.md` (10 rows plus two appendices) and
`DOC_DRIFT.md` (12 rows, S11 the highest-leverage). That is item 2 under
*Do next*.

---

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

## Do next, in the operator's order

> ★★★ **Order this list by what the operator reaches for, not by what just
> arrived in the engine channel.** On 2026-08-18 print paper sizes got built
> while clicking a stamp did nothing at all. The engine's replies are an input
> to *how* a thing gets built, never to *which* thing.

1. ★★★ **His own list, and `O186` Stage 1 is the top of it.** The deep-zoom
   raster refusal that shipped tonight is the **symptom**; the cursor jumping
   away from the thing he was zooming into is the **cause**, and he said so
   himself in `FEATURE.txt` item 6. It is fully designed and not written: a
   `visible_origin_range(strip, viewport, overhang)` in `canvas/geometry.rs`
   returning `(-pb, (strip + pb - viewport).max(-pb))`, a confinement of
   `doc.deep_anchor` at the end of `canvas/deep.rs`'s `if deep` block, and an
   escape hatch in `present.rs`'s empty-`drawn` branch **before** its `return`
   so page-flip and Ctrl+wheel still work on a blank canvas. See `DESIGNS.md`
   §O186 for the measured anchor and the driven check it owes.

   Then the seven `FEATURE.txt` rows still open, in this order — cheapest
   first, which is how the five that shipped got shipped: **O185** (print
   dialogue memory and Cancel) → **O181** (installed fonts in Add text, and
   the dead Format ribbon) → **O188** (moving and deleting one text block
   inside a group) → **O189** (bookmarks on a cross-document page drag)
   → **O183** (the nine-part ce-dimension paragraph, part 7 first)
   → **O178** (multi-window tab dragging) → **O182** (white seams in the
   KUBOTA render). Designs for O181, O183, O185, O188 and O189 are in
   `DESIGNS.md`. ⚠ **Only Ken closes a row.**
2. ★★★ **Work the two registers. All of it is `.rs` or markdown and none of
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
     `run-all.sh` with its self-test, which is why the suite is **43** and not 41.
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
| `SWEEP_REPAIRS.md` | The repairs a driven sweep earned — each one a defect in the CHECK, not the app. Read before investigating any FAIL. |
| `DOC_DRIFT.md` | Stale claims in this project's own doc comments, dated against the commit that falsified each one. |
| `DESIGNS.md` | The designs that are argued and not yet built — O186 Stage 1, O181, O183, O185, O188, O189. Read before designing any of those; they are ★ **not** operator rulings. |
| `D:\Dev\pdfcer\docs\core-api\index.md` | *"I want to do X — what do I call, in what order, and what will bite me?"* |
| `D:/dev/rag/egui/` | Empirical egui findings from this codebase. Read before touching the dock or the canvas rect. |

**The founding rule, in one line:** a phase is not done until its behaviour is
asserted by driving the real binary. Two of the worst defects in the old GUI
were invisible to a green test suite and obvious within thirty seconds of using
the app. *"The tests pass"* is not a report of working software.
