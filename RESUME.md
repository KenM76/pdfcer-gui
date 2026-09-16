# RESUME

The first file a session reads: what this program is, how to measure its state,
what to build next, and the traps that cost a session if they are rediscovered.

## What this is

A replacement GUI for the `pdfcer` PDF engine. `crates/pdfcer-gui` knows about
PDF; `crates/egui-shell` carries the ribbon, dock, modes and command registry
and never learns what a PDF is; two platform crates and the driving harness
`tools/ui-verify` complete the workspace. The engine is a **branch** git
dependency on `D:\Dev\pdfcer`, linked statically, so the binary already carries
it and there is no integration step. **That tree is read-only from here**:
engine work is filed into `D:\Dev\FeatureRequests\pdfce_FeatureRequests\open\`
and lands there as its own Pass; the channel answers within the hour.

`DEVELOPING.md` holds layout, build, packaging, the standing rules R1-R9, the
documentation standard and an index of every document here; `FEATURES.md` says
what an operator can reach in a real build; `D:\Dev\pdfcer\docs\core-api\index.md`
says what to call and in what order; `D:/dev/rag/egui/` holds this codebase's
egui findings, read before touching the dock or the canvas rect.

## State, as measured

Every figure has a command; run it rather than quote this file. Never carry a
count across a commit that touched a `.rs`, and never carry a NAME without
grepping it — a count only goes stale, a name can be born false.

| What | Command | What the command alone will not tell you |
|---|---|---|
| Engine pin | `grep -m1 -oE 'pdfcer\?branch=main#[0-9a-f]+' Cargo.lock` | `5d43d2ea` — a **branch** pin with no `rev`, so cargo re-resolves it opportunistically and it moves with no `cargo update` on our side. Re-read the lock in the same breath as quoting it; never carry a sha forward from a paragraph written an hour ago. `check-pin-citation.sh` reads this row's third cell and `FEATURES.md`'s first `**Updated:**` line, and fails if either disagrees with the lock |
| Engine HEAD | `git -C /d/Dev/pdfcer log --oneline -1 main` | The question is never whether the two shas MATCH — it is whether CODE has landed since the pin, because only that can falsify a sentence beginning *"the engine cannot"*. `git -C /d/Dev/pdfcer diff --stat <pin>..main -- '*.rs'` is the test; empty means such a sentence may be written. Read this log in the same breath as listing `open/`: a delivery has arrived here as a commit before it arrived as a reply three times |
| Engine version | `grep -A1 'name = "pdfcer-core"' Cargo.lock` | — |
| Last release | `git fetch --tags origin && gh api repos/KenM76/pdfcer-gui/releases/latest` | **Fetch first.** `gh release create` tags on the REMOTE, so `git describe` in an unfetched tree answers with an older tag and reports a commits-unreleased count wrong by a factor. Read every field back out of the API rather than inferring it from the flags passed in, and check the local zip's byte count against the asset's — agreement to the unit is the cheapest proof the upload is the file and not a truncation. The binary's own stamp and `published_at` sit twelve to twenty minutes apart on every release; label which clock |
| Driven checks | `ui-verify --list \| grep -cE '^  [a-z0-9_]+$'` | **Rebuild the harness first** — `--list` sits behind `refuse_if_self_is_stale`, and a stale harness prints nothing, which `grep -c` reports as zero and reads as a roster. The `$` is load-bearing: `--list` also prints a profiles table whose rows are two-space-indented lowercase words. It prints two lines per check, so a `wc -l` answers about double |
| Gates | `bash tools/gates/run-all.sh` | Three states, not two: `0` pass, `1` fail, `3` a gate was SKIPPED for an absent precondition. A skip is not a pass. **Tally on the `RESULT:` line, never on a pipeline's exit code** — `run-all.sh \| tail` reports `tail`'s zero. A stable pass-count across an engine pin bump proves nothing about whether the documents survived it; the tally moves only when a gate is added |
| Unit tests | `cargo test --workspace` | Summed over the `test result:` lines, then cross-counted by `cargo test --workspace -- --list \| grep -cE ': test$'`; passing plus ignored must equal the cross-count. Two methods, because a summed figure nobody cross-checks is how the last count drift got in. The cross-count is also how you know a new test RAN: three `#[test]` functions once compiled clean, reported nothing and executed zero times, because they were nested inside another function |
| Source files | `find crates -name '*.rs' \| wc -l` | **The command's scope is a claim.** `find crates tools` answers larger, because `tools/ui-verify` is a Rust crate and is not under `crates/`. Neither figure is wrong; quoting one under the other's command is |
| Backlog register | `python tools/walk-engine-backlog.py` | Rewrite the five headings from the walker's own printed figures, never by arithmetic on the old ones. A row is a verdict plus one paragraph, capped at 1,200 characters and checked by `--check`; file the row and set the headings in one edit with `--write YYYY-MM-DD` |
| Request channel | `ls /d/Dev/FeatureRequests/pdfce_FeatureRequests/open \| wc -l` | The invariant is *a session lists `open/` and nothing else, and empty means nothing is owed* — so a closed exchange left there mis-states the outstanding work by an order of magnitude. Closing is part of doing the work: archive both files AND write the `INDEX.md` row in the same sitting as the `done_*`. A move without a row is not a close, it is a deletion that leaves a file behind. **Nothing in that folder is in a git repository, so no gate can ever see this** |
| Registered commands | `grep -rn --include='*.rs' -E 'command\(' crates/pdfcer-gui/src/shell/commands/catalog/ \| grep -vE ':\s*(///\|//)' \| wc -l` | The obvious command is wrong, not merely its answer: a raw `grep -rhoE` over that directory counts one extra, a line of prose in `catalog/file.rs` that quotes the very pattern being searched for. The build's own `pdfcer-diag shell commands=` trace (`app/mod.rs:925`) is the tie-breaker |
| Dockable panels | `grep -n 'pub const ALL' crates/pdfcer-gui/src/panels/mod.rs` | — |

## Do next, in the operator's order

Ordered by what the operator reaches for, never by what arrived in the engine
channel: a reply is an input to *how* a thing is built, never to *which*. Each
row's argument is in `OPERATOR_REQUESTS.md`, which **only Ken closes**; the open
set is `grep '^## O' OPERATOR_REQUESTS.md`.

1. **Drive the four that just shipped — O201, O202, O203, O204.** All four are
   built, unit-falsified, gate-green and **not driven**, which is R1's exact
   failure shape: a green suite over a program nobody has used. Render-ahead is
   a band either side of the current page, ordered by distance ascending because
   that is the reverse of what `StripRasters::retain` evicts by; the form-field
   colours are two groups, the box's `/MK` and the text's `/DA`, and the second
   writes font, size and colour as one unit so a colour-only row has to re-send
   the other two; the placement ghost reads the same anchor function the drag
   does; the Tab ring takes the press off `RawInput` in `raw_input_hook` before
   egui can latch a focus direction. Each argument, and what was decided rather
   than derived, is in `OPERATOR_REQUESTS.md`. **Only Ken closes those rows.**
   One still-open sub-item: the forms panel's tab-order view numbers its rows
   from `/Annots` order while the ring uses the engine's derived sequence, so on
   a page carrying `/Tabs /R` or `/C` the two surfaces would disagree.
2. **O198's remainder, which is O188** — a title-block run the exporter wrote as
   one lump is still one lump: he can reach that text and drag a line, not take
   it apart. **The verb has landed and is in the pin**: `EditSession::split_text_object`
   with `text_object_split_plan` for the cost-before-committing half. Nothing in
   this shell calls either; the row in `ENGINE_BACKLOG.md` carries the five
   refusals that want operator sentences and the rule-4 disclosure `Line`
   granularity owes.
3. **O181** — installed fonts in Add Text, and the dead Format ribbon controls.
4. **O189** — bookmarks survive a cross-document page drag.
5. **O183** — the nine-part ce-dimension paragraph, part 7 first.
6. **O178** — multi-window tab dragging.
7. **O182** — white seams between the image tiles of a colour rendering.
8. **O194 steps 1, 4 and 5** — the ~30 surfaces in `UNIT_SURFACES.md` with no
   unit control, and `ui_text` abbreviations. Step 2 shipped as an invariant:
   `src/units.rs` is the only place a document length is converted or rounded for
   display, `whole()` the only function permitted to round one, and
   `check-unit-conversion.sh` fails the build on a fresh `25.4` under the GUI.
   Font and type sizes are excluded, and the exclusion is written into source.
9. **O195** — smart select in Review. `smart::enabled` defaults on and
   `clicking.rs` reads the scope every frame, but `textsel::takes_the_press`
   answers `tool.is_text() || (Select && !edit_content)`, so in Review the plain
   Select press is consumed as a text sweep and the smart rung never sees it.
   Flipping the manifest condition alone would ship a visible, inert control,
   which is what R9 exists to prevent.
10. **O176 — his verdict, not a repair.** At fit zoom a form field is about 28 px
    wide and its corner grips eat every point on it; the fix is a grip that yields
    the body below some size, not a harness zoom that hides it.
11. **Wire `SignReport::appearance_lines`** — the one engine delivery genuinely
    owed. It is what the engine drew into the signature box, and here it reaches
    only a trace the source marks as never displayed, so he signs without seeing
    what the stamp says while the engine records *a rectangle too small for them*
    as a real outcome. Off-canvas, in the signing dialog's result.
12. **Re-run a full driven sweep, and read the SKIP set before the tally.** The
    last one reported `passed=86 failed=3 skipped=141` and **128 of those skips
    were one stuck notification toast**, so it measured nothing for more than
    half its roster while printing a tally that reads like a result. Ninety-five
    minutes taking the real cursor and keyboard, so only while he is away, and
    start `target/scratch/toast-watchdog.ps1` alongside it. Diff the SKIP set by
    NAME against the previous baseline in `target/scratch/`, in both directions.
13. **The Set-scale window never appears, and the ordering story for it is
    disproved — D64.** `app::frame`'s `ui` is one function: ribbon at 724, the
    command drain at 954, `dialogs.show` at 995 — so a dispatch always precedes
    `dialogs.show` in the same frame, and `export_text` shows a dialog drawing
    in the frame of its own press. Do not cut a frame boundary at `canvas-pos`;
    the ribbon's line is the frame's first event, and cutting it wrongly
    invents a one-frame lag. What is actually measured: `scale-open` and
    `scale-seeded` fire, then no `viewport-inner` and no `dialog:set-scale`.
    Settle it by driving the binary with a trace line added immediately before
    the `scale` call in `dialogs::show`. Do not apply the repaint-at-dispatch
    fix this row used to prescribe.

## Traps

- **A SKIP is not a FAIL, and nearly every FAIL is a defect in the CHECK — a
  prior, never a verdict.** Load alone removes coverage with nothing turning
  red, so diff skips by NAME and run one; and a session that reads the first
  harness fault as proof of a harness bug stops where a program defect once sat.
- **A stuck notification toast voids a whole sweep and reports it as SKIP.**
  Windows refuses `SetForegroundWindow` to a background process while anything
  else owns the desktop, so a single `Windows.UI.Core.CoreWindow` belonging to
  `ShellExperienceHost` that takes the foreground and never yields turns every
  check that clicks or types into a skip. It cost 128 of 230 checks in one run,
  from about chunk 121 to the end, and `WM_CLOSE` does not dismiss it. The
  harness names the holder and its pid in **every** refusal — but a `grep` for
  that sentence finds a fraction of them, because the log’s own line-wrapper
  splits the phrase across two lines. Unwrap before counting
  (`' '.join(text.split())`). `target/scratch/toast-watchdog.ps1` dismisses such a
  toast during a run, matched on window class plus the host’s full executable
  path and never on image name.
- **While a sweep runs, no `.rs` and no `.toml` may change, `Cargo.lock`
  included.** `staleness_complaint` (`tools/ui-verify/src/launch.rs:788`) scans
  those two extensions and aborts every remaining chunk on a binary older than
  its source; a comment-only edit has cost sixty-one checks. Markdown, `.sh` and
  `.py` are not scanned. `tools/ui-verify/sweep-full.sh` drives a copy under
  `target/scratch/drive/`, so a harness-only rebuild may resume; re-derive chunk
  boundaries from `target/scratch/checks.txt`, never a fresh `--list`, which
  renumbers.
- **A commit does not invalidate the build stamp, and the inputs `build.rs`
  declares are why.** It declares `src`, `Cargo.toml`, `../../Cargo.lock`, the
  two icon assets, and `.git/HEAD`, `.git/refs/tags`, `.git/packed-refs`. A
  commit rewrites **`.git/refs/heads/<branch>`** and `.git/logs/HEAD`, and
  neither is declared; `.git/HEAD` itself holds `ref: refs/heads/main` and is
  not touched by a commit at all (measured: its mtime was twelve days older than
  HEAD's commit time). So the rebuild between the release commit and packaging
  is a no-op that ships the pre-commit, dirty stamp; `touch
  crates/pdfcer-gui/build.rs` forces it. A sweep therefore
  never drives the binary that ships — committing relinks it — so the honest
  claim is *against these sources, from this commit's tree*.
- **A symbol that stops being PRODUCED is invisible to every signature-based
  drift gate**: `check-engine-api-drift` and `check-ui-toolkit-drift` compare
  signatures, and a variant whose last constructor was deleted changes nothing
  public. `check-unreachable-refusals` is the cover — it diffs the engine at the
  locked revision against the symbols each `UNREACHABLE-FROM:` marker cites, in
  `tools/gates/unreachable-refusals.txt`. **Read that diff before an `--update`.**
- **Patch a document through a file, never a heredoc, and put no backslash in
  the payload.** The shell mangles non-ASCII prose and backslashes; an escaped
  newline in a non-raw Python triple-quoted string becomes a real newline in an
  f-string; an escaped NUL becomes a real NUL byte. Build the byte with
  `chr(10)`, `assert` the match count, `ast.parse` before running, and anchor on
  a line PREFIX — this prose uses em dashes, and a typed hyphen matches nothing.
- **Cite engine source by symbol name, or pair the line with the pin.** Line
  numbers rot monotonically because `edit.rs` grows at the head, and a rotted
  citation protects its claim: the reader who checks finds plausible code, stops.
- **A doc comment in this repo can be a GATE'S DATA, not narration.**
  `check-conventions.sh` reads `// conventions: <class>` blocks and scores each
  numbered row of the gesture class as answered or not; `check-unreachable-refusals`
  reads `UNREACHABLE-FROM:` markers; `check-verb-coverage` scores a verb
  "consumed" on prose. A comment sweep that looks purely cosmetic took the
  answer rows out of nine surfaces and turned one gate red with no code change.
  Before deleting comments in bulk, grep `tools/gates/` for what parses them.
  The same sweep also took **833 lines carrying the operator's own words**
  out of 351 files — his sentence is why a module is shaped the way it is,
  and no gate watches for it. The oracle is a count, not a diff: comment
  lines matching the `*"` quote marker, per file, before and after. There
  are 3,755 of them; if a comment pass lowers that number, it removed
  evidence, whatever else it removed.
  Those restored paragraphs carry 662 dated lines, 220 star markers and 170
  history phrasings, every one of them inside a block that holds one of his
  quotes. The global rule against dates, decoration and history is about
  narration a reader does not need; it is not a licence to take his sentence
  back out. Judge a comment by whether it carries the `*"` marker first.
- **Ten root documents were deleted on 2026-09-15** — `CONTINUE.md`,
  `DOC_DRIFT.md`, `GLYPH_ADOPTION.md`, `HANDOFF.md`, `HOW_IT_SHOULD_WORK.md`,
  `HOW_IT_WORKS_TODAY.md`, `INTERACTION_GAP.md`, `REVIEW_TRIAGE.md`,
  `SHELL_LAYOUT_PROPOSAL.md`, `SWEEP_REPAIRS.md`. Their conclusions live in the
  documents that remain; what is gone is the argument and the history. Three
  citations of the dead names survive **inside string literals** and were left
  alone because they are code, not comment: `crates/egui-shell/src/theme/contrast.rs:989`
  and `tools/ui-verify/src/checks/dimension_groups.rs:284,316`. Do not go
  looking for the files; `mockups/` cites three more in generated HTML and says
  so in its generator's header.
- **The shared sweep fixture is a compromise.** `fixtures/a1-titleblock.pdf` has
  one page, no AcroForm, no optional content, no transparency, no removable
  font, and its tallest text is about 2.4 px at fit zoom. Each absence is some
  check's unwritten precondition, so a text failure on it is that until measured
  otherwise; give a check its own `const FIXTURE`.

## Do not

- **Do not write to `D:\Dev\pdfcer`.** Read-only until fold-in.
- **Do not run `ui-verify` while the operator is at the machine.** It takes the
  real cursor and keyboard.
- **Do not kill `pdfcer-gui.exe` by image name** — it is his daily PDF reader.
  Kill by PID, verified against its path.
- **Do not drive the published build.** Copy the exe to `target/scratch/drive/`
  and his drawing to `target/scratch/docs/`; a run opens, types and saves.
- **Do not revert an experiment through git.** A reverting verb inside a chained
  command discards uncommitted work in the same file, and a hook refuses it:
  keep a copy, restore from the copy.
- **Do not append to the agent-memory index.** `check-memory-index.sh` caps
  `.claude/agent-memory/pdfcer-gui-engineer/MEMORY.md` at 24,000 bytes and the
  harness truncates from the **end**, dropping the newest rows — the ones a cold
  session most needs. Fold a new lesson into the entry of the same shape, and
  assert the byte count before writing.
- **Do not build S6 deep zoom or tiling.** Measured as a 9x regression.

## Release discipline

1. **Update the engine first**: `cargo update -p pdfcer-core -p pdfcer-render -p
   pdfcer-print`. `package-portable.py` does this as its own first step, so on a
   day the engine commits live every packaging run moves the pin and stamps the
   artefact `-dirty`: commit `Cargo.lock`, then package `--no-update`. A stale
   pin has cost eighteen missing images on his own file.
2. **The order:** commit, `touch crates/pdfcer-gui/build.rs`, rebuild, re-drive
   the checks that read the build stamp against the shipped exe, off-screen smoke
   launch, `package-portable --no-build --no-update`, push, `gh release create`.
   **Never pass `--prerelease`** — GitHub hides one from `releases/latest`.
3. **Smoke-launch off-screen before every release.** Ninety seconds, and it has
   beaten four thousand unit tests and the whole gate suite to a live defect: a
   geometry change moves where a document OPENS, not only where it can be dragged.
4. **Publish every build worth keeping.** `package-portable.py` rotates the two
   OneDrive slots itself, replacing the older so the previous build survives;
   pass `--slot` only to RETRACT. Read `BUILD-INFO.txt` back out of **both**
   slots afterwards — a tool's own report is not evidence about its own effect —
   and say which slot holds which build, because the slot name carries no
   version. His install at `C:\Users\Ken\OneDrive\pdfcer\` is never touched, and
   the packager's argument against a release with nothing visible is overruled.
5. **A test that proves WHICH mechanism ran must assert what the WRONG mechanism
   cannot produce.** Falsify it by putting the old call back, never by breaking
   the input.
