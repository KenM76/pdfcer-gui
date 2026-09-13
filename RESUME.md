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
| Engine pin | `grep -m1 'pdfcer?branch=main#' Cargo.lock` | `3e73a02` — moved from `d86cb19` at 09:40 today to take `reply_G013` (km, yd, mi; `Unit::all()` as a slice) |
| Engine HEAD | `cd /d/Dev/pdfcer && git log --oneline -1 main` | `3e73a02` — ★ **level with the pin.** Level is the only state in which a sentence of the form *the engine cannot do X* is safe to write. ⚠ The pin is a BRANCH pin and it has moved mid-session without a `cargo update`; re-read the lock in the same breath as quoting it. ★★★ **And "level with HEAD" is not the same as "nothing is in flight":** at 09:00 today this row was level, and a reply that changed three of this project's source files was already written and waiting in `open/`. Level tells you the LOCK is current. It tells you nothing about the request channel, which is not a git repository and which no command in this table reads. |
| Engine version | `grep -A1 'name = "pdfcer-core"' Cargo.lock` | `0.53.0` |
| Last release | `git fetch --tags origin && gh release list --limit 3` | `v0.5.0-dev.20260913.1`, cut 2026-09-13 05:52 UTC, `prerelease=false`, and it IS what `releases/latest` advertises. ⚠⚠ **Fetch first.** `gh release create` tags on the REMOTE, so `git describe` in a tree that has not fetched answers with an older tag — that mistake put *"51 commits unreleased"* in a report where the answer was 21. ⇒ **And never pass `--prerelease`**: GitHub hides a pre-release from `releases/latest`, so the front page went on advertising a three-day-old zip while a new one sat in the list. |
| Driven checks | `ui-verify --list \| grep -cE '^  [a-z0-9_]+$'` | **221** (the sweep chunks 220 and runs 1 from the ALONE table separately — its own first line says `sweeping 220 checks in chunks, plus the ALONE table`) |
| Gates | `bash tools/gates/run-all.sh` | **47 passed, 0 failed, 0 skipped** — 47 registered, up **four** on 2026-09-13: `check-completeness-tests` and `check-memory-index`, each with its own self-test. Both were built out of defects this same morning produced, which is the only reason to trust either: a gate written from an imagined failure tends to assert the imagined thing. ★ `check-completeness-tests` walks every test whose name promises exhaustiveness and asks whether it iterates a real `all()` or a hand-written list — **29 rows outstanding, nine of them against an engine enum**, which is the severe pairing. ★ `check-memory-index` keeps this project's agent memory indexed and its links resolving. ★ `check-pin-citation` itself was **repaired** this morning: its extractor ended in `tail -1` and read the sha a row says it moved AWAY from, which fails a correct row and, far worse, **passes a stale one**. Now `head -1`, with a self-test case in each direction. |
| Unit tests | `cargo test --workspace` | **4,247 passing, 0 failed, 51 ignored, 4,298 defined** — summed over 24 `test result:` lines, then cross-counted by `cargo test --workspace -- --list \| grep -cE ': test$'` = 4,298. Two methods, because a summed figure nobody cross-checks is how the last count drift got in. |
| Source files | `find crates -name '*.rs' \| wc -l` | 786 |
| Backlog register | `python tools/walk-engine-backlog.py` | 173 rows — wanted 35 / blocked 8 / unknown 0 / declined 14 / **shipped 116** |
| Request channel | `ls /d/Dev/FeatureRequests/pdfce_FeatureRequests/open \| wc -l` | **37** — and the newest entry is the one to read first: **`notice_2026-09-13-three-more-all-accessors-are-slices-and-one-deliberately-is-not.md`, Sep 13 10:31**, a NOTICE rather than a reply, so nothing is owed back. It is **deliberately not in this build**: it breaks two `all()` accessors, the pin was taken at 09:40, and the ninety-minute sweep below measures the binary built from that pin. ★★★ **The port is planned rather than improvised** — `HANDOFF.md` carries the call-site inventory, including the one question that had to be measured before it could be scheduled (the new accessor is `const`, so `panels/docprops/mod.rs`'s `const FIELDS` line survives unchanged; had it not been, that module's whole fixed-size-array design would have had to go). It also closes one of the nine FOREIGN completeness rows, because `SnapKind::all()` now exists. ★ Earlier today: `reply_G013` answered, adopted and consumed (`done_G013_CONSUMED.md`, 10:12). ★ `reply_G007` stays unconsumed on purpose: it accepted the topic-key convention itself, asked nothing and delivered nothing, so a `done_G007` would acknowledge an acknowledgement. ⚠ **Re-`ls` this folder before quoting it** — it changes faster than any other input to this project, it is in no git repository, and nothing warns you. |
| Registered commands | `grep -rn --include='*.rs' -E 'command\(' crates/pdfcer-gui/src/shell/commands/catalog/ \| grep -vE ':\s*(///\|//)' \| wc -l` | **161** — ★★★ **the obvious command is wrong, not merely its answer.** A raw `grep -rhoE 'command\('` over that directory returns **162**, and the extra is a line of prose inside a comment in `catalog/file.rs:556` that quotes the very pattern being searched for. The build's own `pdfcer-diag shell commands=` trace is the tie-breaker and says 161. ★ This row was carrying 162 an hour before this release, taken from the raw grep. |
| Dockable panels | `grep -n 'pub const ALL' crates/pdfcer-gui/src/panels/mod.rs` | 13 |

⚠ **The check-count command was wrong in this very table, and the warning
that replaced it then went stale too.** `--list` prints **two lines per check
plus an eight-line header**, so a `wc -l` answers `2n + 8` — roughly double
the real figure, which is why it looked plausible. Measured against
`target/release/ui-verify.exe` on 2026-09-13, after the O186 work added a check: **221** names, **450** lines.

★★★ **Write the relationship, not the pair of literals.** The first repair of
this note said *"`wc -l` answered 434 where the answer is 213"*. Both numbers were
correct the day they were written and both moved the next time a check was
added, so the note that exists to stop count drift had drifted — sitting three
lines beneath a row that said 220. It then went stale a THIRD time the same day, at 221, which is the whole argument. A lesson spelled as two constants expires;
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

## ★★★ The sweep ran again — 221 checks, same four failures for the third time, and the news is in the SKIP column

**2026-09-13 10:16 to 11:51, against the binary that ships, taking the real
cursor and keyboard for about ninety-five minutes.** The fourth full driven
sweep in two days. It did not abort.

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
2. ★★★ **Five rows he filed on 2026-09-13, AFTER `FEATURE.txt` — and they
   are measured, so none of them starts from zero.** O192, O193, O194 and O195
   are his words; O196 is one we found and he has not reported yet. Full
   `file:line` findings are in `OPERATOR_REQUESTS.md` under a
   `### MEASURED 2026-09-13` block on each row. ⚠ **Only Ken closes a row.**

   ★★★ **O192 + O193 are ONE structural fix and should be built together.**
   The Set Scale window neither reads the scale it is about to overwrite nor
   says which dimension group it is aimed at. The cause of both is that
   `ScaleDialog::open` takes a bare `GroupId` and `show` takes only a context and
   an action queue, so the dialogue physically cannot see the document —
   `dialogs/open.rs` destructures `status` and discards it. Pass `&OpenDoc` in;
   `dialogs/page_size.rs` is the exemplar and `export_dxf.rs` already makes the
   one call needed. ★ Groups are **already named**, so the naming sub-task
   O193 anticipated is discharged before it starts.

   ★★★ **And there is a third defect in O192 he has not reported: pressing
   “Measure it on the drawing” throws away everything already typed.**
   `app/frame.rs` closes the dialogue and rebuilds it with default fields.

   ★ **A correction to O192's own text, kept because the row was wrong:** it
   claimed this was *“the third surface to have it”* and proposed a sweep of
   every dialogue that writes a value without reading one. **The sweep was run.
   All 28 dialogues were checked and Set Scale is the class's only member.** The
   generalisation was wrong; the defect is real.

   ◑ **O194 — units everywhere — is a sweep and its enumeration is DONE.**
   `UNIT_SURFACES.md` lists all 55 surfaces with `file:line`: 3 already follow
   the operator's unit, 4 offer a menu, ~18 are hard points, ~12 hard
   millimetres. ★★★ **Step 2 does not wait on the engine and is the highest
   return in the row: the conversions already disagree.** Six private constants
   under two names, seven re-declared closures, two of them in `f32`,
   inconsistent rounding ⇒ *a sheet of exactly 210.5 mm renders 211 in the page
   thumbnail's tooltip and 210 in the print dialogue, today.* ⚠ Font and type
   sizes must be **excluded, and the exclusion written into the source**, or
   somebody eventually offers him a font size in kilometres. ⚠
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

   ⚠ What is still ours in O194 is the **whole** of steps 1 and 2 — the ~30
   hard-`pt` and hard-`mm` surfaces, and the one conversion table. Nothing in
   that waits on anything.

   ★★ **O195 — smart select in Review — is NOT a condition change, and the
   row's own ★ clause saying it was is struck as FALSIFIED.** The preference is
   **already on** in Review. What Review lacks is not the mechanism and not the
   setting: it is a **consumer**. `canvas/clicking.rs` hands the press to text
   selection whenever the gate matches, and in Review the gate matches `Select`
   unconditionally — so the smart-select rung never runs. Flipping the manifest
   condition would ship a visible, inert control, which is the exact failure R9
   exists to prevent. ⇒ **The deliverable is that a Review click resolves the
   way he means, not that a control appears.** It is a capability-model question.

   ◑ **O196 — found, not reported: three export windows forget every
   setting.** Image, text and DXF export all open on hard-coded defaults every
   time. ★★ The precedent is already built — `PrintDialog::open(doc, remembered)`
   and its driven check, from his own O166. ⚠ **Do not fold this into O192.**
   Reading no current value and remembering no previous value are different
   defects and a fix for either does nothing for the other.

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
