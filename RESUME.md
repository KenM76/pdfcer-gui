# RESUME — read this, then say "continue"

> ★★★ **LATE ADDENDUM, 2026-09-09 night: FOUR ENGINE CAPABILITIES WERE WIRED
> HERE AND NEVER RECORDED AS CONSUMED, WHICH LEFT THE GATE FOR EXACTLY THAT
> BLIND.** The librarian sweep's stale-limitation item is closed. Read this
> before the block below it; the block below is still accurate about the
> release and the numbers.
>
> **What was actually wrong.** Four markup asks filed 2026-09-06 were answered
> by the engine **the same day** and wired here within a day. `ENGINE_BACKLOG.md`
> went on saying **`BLOCKED`** about three of them until tonight, `FEATURES.md`
> carried the filed wording in two more places, and one section claimed all four
> were sitting in `open/` when all four are in `archive/` with a reply. Three
> days, four documents, **every gate green the whole time**.
>
> ★★★ **Two independent causes, either one sufficient — and the second is the
> one worth carrying forward.**
> 1. `check-stale-blockers.sh` scanned a hand-written `DOCS=(...)` that **did
>    not include `ENGINE_BACKLOG.md`** — the one file in the project whose
>    entire purpose is blocked rows. Fixed, with a comment at the list telling
>    the next author to add their document in the same commit.
> 2. **The gate's evidence is a `*CONSUMED*.md` note that WE write**, and no
>    note was ever written for those four. So even with the file in scope it
>    would still have passed. ⇒ **Filing the record is not paperwork; it is the
>    act that arms the check.** Both notes now exist
>    (`done_2026-09-06-markup-style-four-CONSUMED.md`,
>    `done_2026-09-06-text-import-CONSUMED.md`).
>
> ★★ **A third thing came out of falsifying the fix, and it is a trap in the
> other direction.** Re-run on the CORRECTED files, the gate reported two more
> hits — both correct ✅/Reachable rows *narrating their own history* (*"the ⛔
> on this row was stale and is corrected"*). Firing on those teaches the next
> engineer to delete the history sentence, which is the most valuable part of a
> corrected row. Fixed by reading the **cell-initial** verdict token rather than
> the whole line, and the excused count is now **printed on every run, green
> ones included**.
>
> ⚠ **What the program told Ken was never wrong.** The `/FreeText` stale-words
> sentence has always been gated on `!appearance_rebaked`, so it only ever fired
> on the narrow case that still survives (an appearance pdfcer would not have
> drawn — a shadow, a gradient, an image). **The staleness was in our prose, not
> in the software.** Do not "fix" the shell here.
>
> ✅ **Verified by falsification, not by reading:** stale documents restored
> from copies → gate RED on exactly the three real rows, the two narration lines
> excused; corrected documents restored → GREEN. All seven doc-reading gates
> re-run green afterwards.
>
> ⚠⚠ **And a self-inflicted one, because it will happen again.** Restoring the
> pre-edit `.bak` to reproduce the stale state **clobbered three finished
> corrections** in `FEATURES.md`, because only `ENGINE_BACKLOG.md` had a
> `.corrected` snapshot taken first. Before any falsification, snapshot the
> **post-edit** state of every file the experiment touches. The tell is
> `grep -c` for a phrase you know you wrote returning **0** — check it
> immediately after every restore.
>
> ⬜ **Still owed on all four asks: driven verification.** Wired and
> unit-tested is not driven (R1). None of the four has a `ui-verify` check that
> works the control in a running binary. `FEATURES.md` says **BUILT AND
> UNDRIVEN**, and that is accurate.
>
> ⇒ **The standing habit this replaces a warning sentence with: triage the reply
> channel every session.** The old block carried its own note saying *"none has
> a reply — re-grep the channel before repeating any of them as open"*. It named
> the exact test that would have caught this and **it was never run**. A note
> asking a future reader to re-verify something is not a check.

> ★★★ **LAST SESSION: 2026-09-09, evening. THE LOAD-ANOMALY DISCLOSURE IS
> DRIVEN, AND IT WAS FALSIFIED BOTH WAYS BEFORE IT WAS BELIEVED.** Fifth
> release of the day. OneDrive **`pdfcer-gui2` (22:51) is the new build,
> `pdfcer-gui1` (10:57) is the fallback**; GitHub `v0.5.0-dev.20260909.5` at
> `a98dad5`, verified from his side (`releases/latest` returns it,
> `prerelease: false`, one zip, 23,434,943 bytes). ⚠ He runs
> **`OneDrive\pdfcer\pdfcer-gui.exe`** — a THIRD location the rotation script
> does not touch, and three instances of it were live all session.
> **Measured 2026-09-09 evening: it is the 10:20 build — TWO releases behind.**
> So the redaction override ruling (10:57), the Find Zoom tick-box and the
> load-anomaly disclosure (22:51) are all absent from the program he actually
> runs. ★ Not copied over: the exe was live in three processes, and that folder
> is his install rather than a rotation slot — **tell him, do not overwrite it.**
> Always check its date before reading any report as "not fixed".
>
> **State, every number re-measured 2026-09-09 evening in the session that wrote this block:**
>
> | | | measured with |
> |---|---|---|
> | tests | **3,957 passing, 0 failing, 51 ignored** | `CARGO_BUILD_JOBS=2 cargo test --workspace`, summing every `test result` through `awk` |
> | gates | **32 of 32, 0 skipped** — and the run earned its keep, going red on 15 clippy `doc_lazy_continuation` sites in `status::anomalies` | `bash tools/gates/run-all.sh` |
> | engine | `pdfcer-core` **0.49.0 at `369d4de`** (`Pass 286.0`). ★★ The tree is **2 commits ahead and no engine bump was owed** — `git diff --stat 369d4de..HEAD` touches only `docs/` and `.claude/`, nothing under `crates/`. Measured, not assumed | `grep -A3 'name = "pdfcer-core"' Cargo.lock`; `git -C D:/Dev/pdfcer diff --stat` |
> | driven checks registered | **198** — up from 196 | `Box::new(` inside `checks/roster.rs`'s `all()`. ★ A raw `grep -c` over the file returns **199**; the 199th is `roster.rs:18`, a **doc-comment line**, not a check. Nothing sits outside `all()` — an earlier revision of this row said one did, which was a plausible reconciliation invented rather than measured, corrected 2026-09-09 |
> | request channel | **215 files open, 53 `reply_*`**. Triaged: the three newest — the per-mark `redacted_text` reply, the residual-sweep notice and the load-anomaly notice — are all **consumed**, and the newest of them IS the current pin. One request filed by us this evening | `ls .../open \| wc -l` |
> | smoke-launch | ✅ off-screen, `a1-titleblock.pdf` from the REPO ROOT, FINAL build: page drawn (`canvas-coverage covered=1.000`), no panic, no `overflow.*` region, **no `status-group:load-anomalies` on a clean file** (a third independent witness for the absence half), killed by PID+path | `PDFCER_DIAG=1 PDFCER_DIAG_VIEWPORT="-4200,-4200,1400,900"` |
> | source | **541,789 lines of Rust in 954 tracked files**; `pdfcer-gui` 387,909; **~54,000 lines of Markdown in 125 files** — ★ quoted to the nearest thousand **on purpose**: this figure changes with every documentation commit, including the one that states it, so an exact number here is stale before it is read (it was 53,658 when first written and 54,056 two commits later). Re-measure, never quote | `git ls-files '*.rs' \| xargs cat \| wc -l` — through `cat`, never `wc`'s per-batch totals |
>
> ## ★★★ THE FINDING: A GREEN CHECK THAT HAS NEVER BEEN RED IS NOT EVIDENCE
>
> `load_anomalies_reach_the_status_bar` passed the first time it ran. That is
> the moment to distrust it, not to file it. Two falsifications, one per clause:
>
> - point the **present**-assertion at the clean fixture — RED, and with the
>   right complaint ("the status bar declares no `status-group:load-anomalies`
>   region, so the census line never drew");
> - point the **control** at the contradicting fixture — RED, with the *other*
>   right complaint ("the census line is on for files that have nothing to
>   disclose").
>
> Each clause fails on its own; neither is carried by the other. Restore from a
> **copy**, never `git checkout` — the file was untracked and that verb would
> have deleted the session's work.
>
> ★★ **And the control fixture is asserted clean, not assumed clean.**
> `the_control_fixtures_a_driven_run_uses_are_genuinely_clean` opens both
> fixtures through the real engine on every `cargo test`. ⚠ It spells the two
> names **literally**, because `app::state::FOUR_PAGES` resolves against the
> ENGINE's read-only corpus (`pageops/four-pages.pdf`) and not this repository's
> `fixtures/` — the two instruments have to be naming the same bytes, and a
> constant that quietly names other bytes is how a control stops being one.
>
> ## ★★ THE SECOND FINDING: `Session::place` OVERRULED THE VIEWPORT
>
> `PDFCER_DIAG_VIEWPORT`'s **position** was being discarded: `launch::Session::place`
> moved every launched window to desktop `(780, 40)` unconditionally to dodge
> the on-screen keyboard. A no-input check whose doc comment said *"the window
> is placed off the desktop"* would have put it in the middle of his screen
> while claiming otherwise. `LaunchSpec::place` is the opt-out, documented at
> both ends as a promise that **no pointer or keystroke is sent by that launch**.
> Do not set it `false` on anything that clicks.
>
> ## WHAT TO DO NEXT, in his likely order
>
> 1. **Run `load_anomalies_are_listed_in_document_properties`.** Written,
>    registered, compiles, **never run** — it needs the pointer. It clicks a
>    mode segment, a ribbon tab and a ribbon item, then asserts
>    `properties.load-anomalies` is declared with **exactly one** row region on
>    the contradicting file, and absent (with `properties.info` present, proving
>    the panel is open) on the clean one.
>    Command: `target/debug/ui-verify.exe --check load_anomalies_are_listed_in_document_properties --out target/ui-verify/loadanom-panel`
> 2. **The 29 geometry/gesture checks, still unrun** since the solid scroll bars
>    landed 2026-09-08 — and now the dock fix, the engine pin, the Find zoom
>    box and the anomaly panel are all under them too.
> 3. **`double_clicking_a_text_box_edits_the_text`** — last known FAIL, still
>    uninvestigated.
> 4. **O161** — a redaction mark is selectable and deletable headlessly but he
>    could not select it at the keyboard. Drive it.
> 5. **The driven check for the Find Zoom box** — assert the zoom readout is
>    identical either side of a find jump with the box unticked. Owed since
>    `870fe72`.
>
> ## WHAT NOT TO DO
>
> - **Do not `taskkill /IM pdfcer-gui.exe`.** Kill by PID, verified by path.
> - **Do not run a driven sweep while he is at the PC.** Smoke-launch off-screen.
> - **Do not fuse the two load-anomaly checks.** The panel one SKIPs under
>   `--no-input`, and a SKIP is not red; fusing would silence the cheap half on
>   every unattended sweep.
> - **Do not bump the engine reflexively before a build without reading the
>   diff.** It is right when the engine moved code and it is a 10-minute engine
>   rebuild when the engine moved two librarian filings. Read
>   `git diff --stat <pin>..HEAD` first — `crates/` or not is the question.


> ★★★ **LAST SESSION: 2026-09-09, 04:30–10:30. READ `HANDOFF_20260909.md`
> FIRST** — it is the long form of everything below. FOUR releases today;
> **`pdfcer-gui1` (10:57) is the newest, `pdfcer-gui2` (10:20) the fallback**;
> GitHub `v0.5.0-dev.20260909.4` at `4bb449e`, verified from his side. ★ 10:45
> rulings: a redaction whose words appear ELSEWHERE is his call (disclosed +
> acknowledged, not refused — `ResidualSite::DrawnContent`); a mark is
> unselectable for him at the keyboard but selectable+deletable headlessly
> (O161, drive it); stamp re-bake re-asked of the engine (O162). He runs
> `OneDrive\pdfcer\pdfcer-gui.exe` (was the 08:27 build) — check its date
> before reading any report as "not fixed".
>
> **09:40, second report (O160), all three fixed and unit-tested, not driven:**
> the redaction proof refused on SINGLE CHARACTERS (Ghostscript draws one glyph
> per show operator) — floor now on both halves; **stamps resize** (matrix carry
> forced for `/Stamp`); **stamps/text boxes upright on `/Rotate` pages**
> (`set_annotation_rotation` composed at authoring). Tests **3,912 / 0 / 50**,
> gates 32/32. Engine is **8 commits past the pin** — `cargo update` first.
>
> **He came to the PC mid-sweep with four reports** (`OPERATOR_REQUESTS.md`
> **O159**): stamp resize, SW41177 text, the redaction box, freehand nodes.
> All four measured on HIS files headlessly; three were engine deliveries of
> the last twelve hours that this shell had not yet consumed or pinned, the
> fourth (stamp) is a real engine gap, filed. **Read the request channel
> before diagnosing any report — the engine answers within the hour now.**
> Freehand node editing is WIRED (subagent) and NOT driven. The sweep reached
> 64 of 196 (63 pass, 1 FAIL `double_clicking_a_text_box_edits_the_text`,
> uninvestigated). Tests **3,909 / 0 / 49**, gates **32/32**, pin **`5b8ec61`**
> (engine is 4 commits past it — `Pass 280.0` `run_repertoire` is the one we
> asked for; `cargo update` first).
>
> ⚠ **RAM:** run the suite `CARGO_BUILD_JOBS=2`; the harness killed the shell
> twice for low memory and left linkers as orphans. ⚠ **Sweeps:** launch
> detached (`Start-Process cmd /c bash target/scratch/sweep-full.sh`), never in
> the tool's 10-minute window; confirm `ui-verify: 1`. ⚠ **Releases:** `git
> push` BEFORE `gh release create` — `--target main` resolves server-side.
>
> ---
>
> ★★ **EARLIER THE SAME SESSION: THE CENTRAL-PANEL WOBBLE IS FOUND, REPRODUCED
> IN PURE EGUI, AND CLOSED AT THE DOCK.** `git log --format=%B 5ea6246` is the
> whole story.
>
> **State, every number re-measured 2026-09-09 in the session that wrote this block:**
>
> | | | measured with |
> |---|---|---|
> | tests | **3,900 passing, 0 failing, 47 ignored** | `cargo test --workspace`, summing every `test result` through `awk` |
> | gates | **32 of 32, 0 skipped** | `bash tools/gates/run-all.sh` |
> | engine | `pdfcer-core` **0.49.0 at `fccd6cd`** — bumped this session to the engine tree's HEAD (`Pass 277.0`) | `grep -A3 'name = "pdfcer-core"' Cargo.lock`; `git -C D:/Dev/pdfcer log -1` |
> | driven checks registered | **196** — unchanged | `Box::new(` inside `checks/roster.rs`'s `all()` |
> | request channel | **201 files open, 45 `reply_*`** — two replies overnight (both answers to ours, both consumed), one request filed by us | `ls …/open \| wc -l` |
> | smoke-launch | ✅ off-screen, `a1-titleblock.pdf` from the REPO ROOT, FINAL build (engine `fccd6cd`): page drawn, no panic, `dock.right.frame` constant through the fade that used to wobble, no `overflow.*` line, no Read-mode Delete controls, killed by PID+path | `PDFCER_DIAG_VIEWPORT="-4200,-4200,1400,900"` |
> | the launch-killer | **gone with the reboot** (uptime 1.9 h at session start) — five launches, zero `SetPropW` deaths | |
> | ⚠ RAM | `cargo test --workspace` at default jobs got the shell **killed for low memory** twice (8 linkers × ~1.3 GB on 16 GB) and left orphans; run it `CARGO_BUILD_JOBS=2`, and one cargo job at a time | memory `project_disk_is_tight…` |
>
> ## ★★★ THE FINDING: A SCROLL BAR FADING IN
>
> `dock/overflow_probe.rs` — every widget egui registered this pass whose
> rect crosses the window edge, by id, on a frame where the side's frame
> does — named it on its first read: a 10-pt strip on the Comments pane's
> right edge, decaying `1400.4 → 1400.1 → 1400.0`. Three parts, each from
> source: (1) `Panel::show` clamps a too-wide frame by moving the INNER
> edge, so `exact_size` fixes width, not position; (2) `scope_builder`
> merges a body's union into the frame — `set_clip_rect` truncates paint,
> not layout; (3) a SOLID scroll bar mid-fade has fractional bar use, the
> content rect is rounded up to pixels, and the sum overshoots the pane by
> the residue. `dock/scroll_fade_repro.rs` shows it with nothing but egui:
> `+0.25`, `+0.47`, then 0. Fix in `dock/stack.rs`: bodies draw in
> `new_child`, the compartment's own rect is allocated in their place.
> Guard falsified against the unfixed code (`[1080.4 .. 1400.4]`), green
> after. RAG: `D:/dev/rag/egui/a_solid_scrollbar_fading_in_overshoots_…`.
>
> ## ★★ THE ENGINE ANSWERED OVERNIGHT, AND ONE ANSWER MOVED A STICKY
>
> Two replies: Enter-in-a-text-box is fixed in the engine (`Pass 275.0`, in
> the pin now, NOT in `pdfcer-gui1`); and the sticky's refusal is a named
> variant (`ResizeFixedSizeMarker`, worded on our side) — **with a
> correction we acted on: a sticky's icon anchors at its rect's UPPER-LEFT,
> and our click built the rect upward, putting the icon 20 pt above the
> click in Acrobat.** `canvas::clicking` now hangs the square down from the
> click; four citations of the wrong corner corrected. Three backlog rows
> written (`/RC` drop: consumed; newline: consumed by the pin, driven check
> owed; font-coverage remedy: `wanted` — our `refusal_of` drops the face
> list). One request filed, corrected first by a grep that found
> `preview_font_resources_for` already shipping half of what I was about
> to ask for.
>
> ⚠ **Two harness inputs bit this session, both mine:** (a) launching from
> `target/scratch/drive/` with a RELATIVE fixture path opened no document —
> no `status` line in the trace is the tell; launch from the repo root.
> (b) `$!` after `(cmd & echo $!)` is the subshell's pid, not the exe's —
> two instances leaked. Find the exe by `Get-Process pdfcer-gui`, verify the
> path, kill by PID.
>
> ## WHAT TO DO NEXT, in his likely order
>
> 1. **Drive the 29 unverified geometry/gesture checks** — still the top
>    item; they have not run since the solid scroll bars landed (2026-09-08)
>    and now the dock fix AND the engine pin are under them too. Add the
>    `text_annot` family (Enter key now reaches a fixed engine) and
>    `comment_note` / `foreign_icon_name` (sticky click). Needs the desktop.
>    Command: `HANDOFF_20260908_RESIZE.md` §2 with `--check` repeated.
> 2. ~~Release~~ **RELEASED 2026-09-09 06:25** from `842f269` (FEATURES at
>    `4d9faaf`): OneDrive **`pdfcer-gui2` is the new build**, so
>    **`pdfcer-gui1` (2026-09-08 22:06) is the fallback**. GitHub
>    `v0.5.0-dev.20260909.1`, verified from HIS side — `releases/latest`
>    returns it, `prerelease: false`, one zip, 22,948,019 bytes. Published
>    undriven, as the standing rule says; the notes say so. ⚠ The engine was
>    already 1 commit past `fccd6cd` at packaging time — `cargo update` first
>    next build.
> 3. `canvas.selection-outline` published twice per frame — unchanged,
>    handoff §3.4.
> 4. `dock.<side>.body_min` is now always equal to `.frame` — it can go
>    (three lines in `draw_side`); left in for one release as a second
>    witness.
>
> ## WHAT NOT TO DO
>
> - **Do not `taskkill /IM pdfcer-gui.exe`.** Kill by PID, verified by path.
> - **Do not remove `RESIZE_FLOOR_PT`** because its source is gone — its doc
>   says why it stays.
> - **Do not run a driven sweep while he is at the PC.** Smoke-launch off-screen.
> - **Do not delete `overflow_probe`** — it is the tripwire for the guard.


> ★★★ **LAST SESSION: 2026-09-08, late evening. THE RESIZE RED IS FIXED, AND
> THE CAUSE WAS A CONTROL NOBODY COULD SEE.** Read `git log -3 --format=%B`
> and the RESOLVED box at the top of `HANDOFF_20260908_RESIZE.md` first.
>
> **State, every number re-measured 2026-09-08 in the session that wrote this block:**
>
> | | | measured with |
> |---|---|---|
> | tests | **3,896 passing, 0 failing, 47 ignored** (re-measured 22:55 after the spec-route deletion and the fit floor) | `cargo test --workspace`, summing every `test result` through `awk` |
> | gates | **32 of 32, 0 skipped** | `bash tools/gates/run-all.sh` |
> | engine | `pdfcer-core` **0.49.0 at `00ddbb1`** — the engine tree's HEAD, so the pin is current | `grep -A3 'name = "pdfcer-core"' Cargo.lock`; `git -C D:/Dev/pdfcer log -1` |
> | driven checks registered | **196** | `Box::new(` inside `checks/roster.rs`'s `all()` — the file holds **197** |
> | request channel | **198 files open, 43 of them `reply_*`** — nothing new since the handoff | `ls …/open \| wc -l` |
> | smoke-launch | ✅ off-screen, `a1-titleblock.pdf`, page drawn, no panic, killed by PID+path | `PDFCER_DIAG_VIEWPORT="-4200,-4200,1400,900"` |
>
> ⚠ **THE WINDOWS SESSION KILLS MOST LAUNCHES — 2026-09-09 01:30 — AND IT IS
> NOT HANDLES, NOT OUTLOOK.** `accesskit_windows` `SetPropW` dies with
> `HRESULT(0x80070008)` before a window exists: 1-in-3 in the evening, 4-of-5
> at 01:00, **10 of 11 checks × 3 attempts** at 01:30. Measured and ruled OUT:
> kernel handles (157 k, Outlook closed), USER/GDI objects (~2 k), both atom
> tables, commit (40 GB free), free RAM (3.8 GB either way). **Settled by a
> control: the 13:05 release build fails 2 of 4 the same way** — the subject
> is the 208-hour session, and the one monotone correlate is the ~60
> force-killed instances the harness produced tonight. **A logoff/reboot is the
> remedy; nothing else measured moves it.** The harness now retries exactly
> this stderr signature (3 attempts, each printed); on this session it still
> reports SKIP, honestly. ⇒ **First thing after the reboot: run the 32-check
> sweep** (`HANDOFF_20260908_RESIZE.md` §2, `--check` repeated) — the new
> scrollbars have ONE driven pass beyond the subject check
> (`a_fit_command_puts_the_page_on_screen`, green) plus the three from the
> evening. `D:/dev/rag/egui/an_intermittent_setpropw_…` has the table.
>
> ★ **The wobble hunt DID read its instrument on the one launch that survived:**
> `dock.right.body_min` ran to **1400.4** — 0.4 pt past the window's edge,
> left edge unchanged — on the wobble frame, so a widget inside the right
> dock overflows its pane by a 1/32-grid amount on isolated frames and egui's
> `Panel` re-derives its 320-wide rect from that right edge. The overflow
> bisect (publish every egui widget rect crossing the side's outer edge, by
> id) is drafted in this session's scratchpad as `overflow_probe.py` and NOT
> yet applied; apply it after the reboot, build, smoke-launch, read.
>
> ⚠ **Ken came to the PC mid-session** (`/loop` note: headless only). The
> 29-check regression sweep was killed at 14 of 29 by PID; its verdicts were
> never written (the harness prints them at the end). What IS driven and green
> on this build: `resize_scales_a_shape` (the subject), and from the first sweep
> `delete_key_after_canvas_click`, `dragging_a_markup_moves_it`,
> `rotating_a_markup_turns_it`. **The other 25 geometry/gesture checks are
> unverified against the new scroll bars** — run them the moment the PC is free:
> the command is in `HANDOFF_20260908_RESIZE.md` §2 with `--check` repeated.
>
> ## ★★★ THE FINDING: THE PRESS NEVER REACHED THE CANVAS
>
> egui's default **floating** scrollbar allocates no width, draws nothing until
> hovered, and `interact`s the outer **10 pt** of the scroll area *after* the
> content with `CLICK | DRAG`; a press anywhere on it centres the handle on the
> pointer — a scroll jump. At fit zoom the page edge sits 6 pt inside the
> viewport edge, so the sheet border's SE grip was drawn on an invisible control
> that took the press first. One new trace line (`canvas-gesture`,
> `gesture/mod.rs`) said it in one run: `started=0 … origin=1`.
>
> ⇒ **Explanation 2 in the handoff was RIGHT and was "disproved" by a number
> compared against the window instead of the viewport** (outline 428 px; the
> canvas viewport was 444 px wide). Memory:
> `feedback_a_disproof_is_a_measurement_too_and_the_dead_hypothesis_was_the_truth`.
>
> **Fix:** solid, always-visible bars beside the page (`present.rs::scroll_style`).
> **It exposed a second defect on its first run:** `fit::placement` measured the
> centre against the inner viewport and placed against the outer, creeping the
> page **7.4 px/frame** — half the 14 pt bar — because its exact-compare resize
> gate is held open by a **0.1–0.5 pt central-panel width jitter**. Every canvas
> viewport is now derived from one `inner_avail` measurement. Both in
> `D:/dev/rag/egui/` (`a_floating_scrollbar_is_an_invisible_control…`,
> `a_measure_and_place_pair_against_two_viewport_sizes…`).
>
> ## WHAT TO DO NEXT, in his likely order
>
> **Released 2026-09-08 22:07** from `b3f8738`: OneDrive **`pdfcer-gui1` is the
> new build**, so **`pdfcer-gui2` (13:06) is the fallback**. GitHub
> `v0.5.0-dev.20260908.6`, **verified from HIS side** —
> `gh api repos/KenM76/pdfcer-gui/releases/latest` returns that tag,
> `prerelease: false`, zip attached, 22,959,794 bytes. ★ The packager writes a
> FOLDER; the zip is `Compress-Archive` of it, then `gh release create` with no
> `--prerelease`. Recorded here because no earlier session wrote the recipe down.
>
> 1. **Drive the 25 unverified checks** when the PC is free (above).
> 2. **The central-panel width jitter** — measured, source STILL unknown, but
>    it no longer reaches the view: `fit::placement`'s resize gate has a
>    **0.5 pt floor** (`RESIZE_FLOOR_PT`, strictly greater, reference not
>    re-recorded under it) since 22:55, three unit tests, two of which went
>    red on their first draft. The fit ZOOM still flickers in its 4th decimal
>    from the same jitter (`ViewState::apply_fit` runs every frame); harmless,
>    unfloored. **Narrowed at 23:40, off-screen, three instruments:** the window
>    and `pixels_per_point` (1.0) are constant; the right dock's parent
>    `available_rect` is constant (`max.x` 1400.0); every child rect the shell
>    publishes is constant; and yet egui's ALLOCATED rect for the right dock —
>    now published as `dock.right.frame` — is translated **+0.3–0.4 pt** on
>    isolated frames (`[[1080.3 …] - [1400.3 …]]`, a 1/32-grid multiple, width
>    exactly 320, past the window edge). ⇒ It is inside egui's `Panel::show`:
>    the rect it takes back from the frame's response (`panel.rs`, the
>    `inner_response.response.rect` line), i.e. the union of something drawn in
>    the panel that the shell does not name. **Next instrument:** publish the
>    body ui's `min_rect()` at the END of the side's show closure and bisect
>    from there (the animated `show_bars_factor` of a body's `ScrollArea` is a
>    candidate — animated, fractional — but MEASURE it). Do not theorise.
> 3. ~~Delete the annotation spec route~~ **DONE 2026-09-08, 22:40** — 804
>    lines gone across 13 files (`Plan::spec_is_more_faithful`,
>    `carried_options`, `translated`, `copy_as_spec`, `Clipped::Markup`,
>    `Action::PasteMarkup`); `Plan::thin` and its disclosure kept by their own
>    argument. Tests **3,893** (one fewer: the tripwire test went with its
>    predicate), gates 32/32, clippy clean. Headless-only change; nothing
>    operator-visible, so NOT republished.
> 4. **`canvas.selection-outline` is published twice per frame** with two rects
>    — re-decide the shared name (handoff §3.4).
>
> ## WHAT NOT TO DO
>
> - **Do not `taskkill /IM pdfcer-gui.exe`.** Kill by PID, verified by path.
> - **Do not weaken `resize_scales_a_shape`'s aim.** A page-sized selection is
>   now the regression test for the scrollbar defect; its warning says so.
> - **Do not run a driven sweep while he is at the PC.** Smoke-launch off-screen.
> - **Do not disprove a hypothesis with a number whose reference you did not
>   write down.**

> ★★★ **START HERE: READ `HANDOFF_20260908_RESIZE.md` FIRST.** It is the
> troubleshooting brief for the one open problem — `resize_scales_a_shape` is
> red, it blocked a release, and **four explanations for it are already
> disproved**. Re-deriving any of them costs an hour; the handoff lists them so
> you do not.
>
> The rest of this block is the summary. Everything below it is history.
>
> **It is NOT a regression from that day's work** — it fails identically on a
> binary built from `50eebe6`, before any of it. It is older than that and
> nobody knows how much older; no previous session recorded running it.
>
> ## What is measured, and it is not much
>
> Driven, `a1-titleblock.pdf`, `--doc-point 0,300,500`:
>
> | | |
> |---|---|
> | the click selects | ✅ an object — `properties-panel` reports `2383.94 x 1683.78`, **exactly A1**, so it is the sheet's own border rectangle |
> | mode | `edit` |
> | selection outline on screen | **428 × 302 px** |
> | the corner drag | commits **nothing** and declines **nothing** — no `resize-commit`, no `resize-declined`, no resize event of any kind in the trace |
> | what DOES move | `canvas.selection-outline` drifts ~1 px across the drag, which reads as the gesture being taken as a **MOVE** |
>
> ## ⚠ TWO EXPLANATIONS WERE TRIED AND BOTH WERE WRONG
>
> 1. *"my `ghost_box` change broke it"* — no; it fails on the pre-change build.
> 2. *"the selection is the whole page, so the grip is at the page corner and
>    the drag aims off the canvas"* — **plausible, specific, and false.**
>    428 × 302 px is nowhere near the canvas, so the grip is reachable and the
>    drag stays on it. The check now prints its own aim and fires a
>    whole-page warning; the warning did **not** fire.
>
> ⇒ **Do not arrive at a third story from reading. That is what the whole of
> 2026-09-08 was, four times over.** The next step is to make the application
> say what it did with the press — there is no `canvas-grip` or
> `gesture-outcome` trace line for a press that lands on a grip, and its
> absence is why this is a mystery rather than a diagnosis.
>
> ## What was already fixed while investigating it
>
> - The check now reports **what it aimed at before what it found**. Its old
>   message named `canvas::interact` as the culprit with total confidence and
>   nothing else; a check that cannot say what it aimed at cannot be believed
>   about what it found.
> - `overlay::ghost_box`'s doc cited a region called `canvas-grip-box`. **There
>   is no such region** — the grip box is published under
>   `canvas.selection-outline`. Invented while writing the sentence, found the
>   same hour by a driven run that went looking for it.
>
> ## ★ And one real finding that is NOT the cause but is worth fixing
>
> `canvas.selection-outline` is published **twice per frame with two different
> rectangles** — once per selected entry in the loop, and once for the grip
> box at `overlay.rs`'s `if let Some(box_) = grip_box(…)`. They differ by a
> fraction of a pixel for a single selection and by a lot for a multi-select.
> A consumer asking for "the" selection outline gets whichever won the race.
>
> ⇒ The call site argues for the shared name deliberately (*"the name is the
> SELECTION's rather than the grips'"*), and that argument was made when only
> one of the two existed. It needs re-deciding, not just renaming.
>
> ---
>
> ## What NOT to do
>
> - **Do not release while this is red** without deciding deliberately that it
>   is harness-only. It is a Phase-1 capability — dragging a shape's corner on
>   the page — and "pre-existing" is not "fine".
> - **Do not weaken the check.** A page-sized selection is a legitimate thing
>   to fail on; the repair is a better aim or a better trace, not a lower bar.


> ★★★ **LAST SESSION: 2026-09-08, midday to afternoon. THE OPERATOR'S TEXT-EDIT
> REPORT WAS DIAGNOSED WRONG TWICE BEFORE IT WAS MEASURED, AND THE THIRD
> ATTEMPT WAS WRONG ON ITS FIRST RUN TOO.** Read `git log -6 --format=%B` in
> full first; the commit messages are the authoritative record.
>
> **State, every number re-measured 2026-09-08 in the session that wrote this block:**
>
> | | | measured with |
> |---|---|---|
> | tests | **3,888 passing, 0 failing** | `cargo test --workspace`, summing every `test result` through `awk` |
> | gates | **32 of 32, 0 skipped** | `bash tools/gates/run-all.sh` |
> | engine | `pdfcer-core` **at `b208dc6`** | `grep -A3 'name = "pdfcer-core"' Cargo.lock` |
> | shell | `3ce70a5`, on `origin/main` | `git log -1` |
> | driven checks registered | **195** | `Box::new(` inside `checks/roster.rs`'s `all()` — the file holds **196**; one is outside `all()` |
> | request channel | **`open/` holds 192 files, 42 of them `reply_*`** | `ls D:/Dev/FeatureRequests/pdfce_FeatureRequests/open \| wc -l` |
> | ui toolkit | egui/eframe **0.35.0**, one minor behind (0.36.x) | `bash tools/gates/check-ui-toolkit-drift.sh` |
>
> **Released 2026-09-08 13:05** from `3ce70a5`: OneDrive **`pdfcer-gui2` is the
> new build**, so **`pdfcer-gui1` (08:34) is the fallback**. GitHub
> `v0.5.0-dev.20260908.5`, **verified from HIS side** —
> `gh api repos/KenM76/pdfcer-gui/releases/latest` returns that tag,
> `prerelease: false`, zip attached, 23,352,138 bytes. Check `releases/latest`,
> never `gh release list`.
>
> ⚠ **He was at the machine all session**, so nothing was driven. The
> smoke-launch route (`PDFCER_DIAG_VIEWPORT="-4200,-4200,1400,900"`, kill by
> PID) was used instead and is what verified the new previews row. **Never
> `taskkill /IM pdfcer-gui.exe`** — it is his daily PDF reader.
>
> ---
>
> ## ★★★ THE FINDING OF THE DAY: **COUNT ONE HALF OF AN `AND` AND YOU WILL
> REASON ABOUT THE OTHER — THREE TIMES, ON THE SAME REPORT**
>
> He reported *"the BOM only sometimes works"*. It was explained twice and both
> were wrong in the same shape:
>
> 1. *"the planner throws the operator pin away"* — it keeps it, deliberately,
>    so a wrong BOM cell is never edited.
> 2. *"a repeated cell hits `AmbiguousOnThePage`"* — **that refusal's
>    population on his file is ZERO.** It needs a run that both repeats *and*
>    spans several show operators; his sheets carry 57–133 of the first and
>    4–11 of the second, and they never intersect.
>
> Both came from counting the cheap condition and inferring the expensive one.
>
> **The cause, found by performing 31 real edits and reading 31 real answers:
> his drawing's fonts are SUBSET-EMBEDDED, so each carries only the characters
> that font already draws.**
>
> ⚠ **And the first number for that was wrong too — a fourth sampling error in
> the same report.** *"46 of 95, no lowercase"* was one cell, `"BRACE"`, on one
> sheet. Re-measured per font per sheet through `edit_text`:
>
> | sheet | fonts | accepted, of 95 |
> |---|---|---|
> | 1 | 4 | 52, 40, 28, **6** |
> | 2 | 5 | 59, 57, 35, 21, **6** |
> | 3 (BOM) | 2 | 46, 31 |
> | 4 | 2 | 45, … |
>
> ★★★ **The floor is six** — the title-block logo font accepts `O P R S T` and
> a space, because `TOP ROPS` is the only string ever set in it. ⇒ There is no
> single alphabet for a drawing; there is one per font per sheet, which is why
> the same edit succeeds on one sheet and fails on the next.
>
> ★ The engine measured the same file independently and got *"four of six at
> 72/95"* — a different scope, and possibly a different question (font-program
> coverage vs what the edit path accepts). Filed as
> `note_our_two_font_coverage_numbers_disagree_and_the_difference_is_the_finding.md`.
> **A coverage percentage "for a document" is a category error — the unit is
> the font.**
>
> ★★ **And the third attempt was wrong on its first run**, in the way that
> mattered: the probe appended a `Z`, so 30 of 31 cells refused
> `UnsupportedFont`. That looked like a finding about the cells and was a
> statement about the letter `Z`. **The harness artefact WAS the answer** —
> but only because "why did *every* cell fail?" got asked instead of written up.
>
> ⇒ Rules, now in agent memory as
> `feedback_count_a_condition_and_you_will_reason_about_the_other_one`:
> measure the intersection; prefer performing to predicting; a fresh session
> per trial (a shared one measures the Nth edit on an (N−1)-times-edited
> document); **build test inputs from the subject's own alphabet**.
>
> ★ Everything the wrong diagnoses reached was amended, not just the next
> report: the engine request filed an hour earlier now leads with a retraction
> box and a LOW priority, and was renamed to stop claiming a BOM in its own
> filename.
>
> ---
>
> ## WHAT TO DO NEXT, in his likely order
>
> 1. **O150's remaining half needs the screen.** The alphabet is knowable
>    before the first keystroke, and the refusal currently arrives at *commit*
>    — so he types a whole word and loses it. Two questions only a driven run
>    answers: does the existing `FontLacksTheCharacter` sentence reach him at
>    all, and should the caret refuse the keystroke or accept it and disclose?
>    `panels/properties/refusedchar.rs` and `canvas/textedit/facewall.rs`
>    already hold the face-swap remedy.
> 2. **O151's drag is unverified.** The previews time-limit box smoke-launches
>    and both controls publish rects on one row, but a `DragValue` drag needs a
>    real pointer. `D:/dev/rag/egui/a_dragvalue_reseeded_...md` is the failure
>    mode; the fix is in place, unproven.
> 3. **Delete the annotation spec route (~600 lines).** `Pass 270.0` made
>    `Plan::spec_is_more_faithful` unable to return `true`. Filed in
>    `ENGINE_BACKLOG.md`; a `debug_assert` and
>    `the_spec_route_is_now_unreachable_and_says_so` hold it meanwhile.
> 4. **egui 0.36 when the machine is free.** Measured: **11 errors, 6 files, 2
>    causes**, everything else untouched. `UI_TOOLKIT_PINS.md` carries the
>    decision and the four verification steps. ⚠ Ten of the eleven are
>    `RawInput::modifiers` — the keyboard path this project has already shipped
>    two guard defects on. **Not a same-day bump.**
>
> ## WHAT NOT TO DO
>
> - **Do not `taskkill /IM pdfcer-gui.exe`.** Kill by PID, verified by path.
> - **Do not re-diagnose O150 from a count.** The probes are committed
>   (`tests/ken_sw41177_probe.rs`, `#[ignore]`d, six of them) so the
>   measurements re-run in one command rather than being re-derived.
> - **Do not `git checkout` to undo an experiment.** The egui 0.36 measurement
>   was taken by copying `Cargo.toml`/`Cargo.lock` aside and restoring from the
>   copies.
> - **Do not bump egui because it compiles.** See item 4.



> ★★★ **LAST SESSION: 2026-09-07 evening into 2026-09-08. FIVE DOCUMENTS AND
> THREE OPERATOR-FACING STRINGS WERE ASSERTING ABSENCES THE ENGINE HAD ALREADY
> CLOSED.** Read `git log -7 --format=%B` in full first; the commit messages are
> the authoritative record.
>
> **State, every number re-measured 2026-09-08 in the session that wrote this block:**
>
> | | | measured with |
> |---|---|---|
> | tests | **3,886 passing, 0 failing** | `cargo test --workspace`, summing every `test result: ok. N passed` through `awk` |
> | gates | **31 of 31, 0 skipped** | `bash tools/gates/run-all.sh` |
> | engine | `pdfcer-core` **v0.47.0 at `d206206`** | `grep -A3 'name = "pdfcer-core"' Cargo.lock` |
> | shell | `5769bfe`, on `origin/main` | `git log -1` |
> | driven checks registered | **195** | `Box::new(` inside `roster.rs`'s `all()` — the file holds **196**; one is outside `all()` |
> | request channel | **`open/` holds 190 files, 41 of them `reply_*`** | `ls D:/Dev/FeatureRequests/pdfce_FeatureRequests/open \| wc -l` |
>
> **Released 2026-09-08 08:35** from `5769bfe`: OneDrive **`pdfcer-gui1` is the
> new build**, so **`pdfcer-gui2` (04:39) is the fallback**. GitHub
> `v0.5.0-dev.20260908.4`, **verified from HIS side** —
> `gh api repos/KenM76/pdfcer-gui/releases/latest` returns that tag,
> `prerelease: false`, zip attached, 22,930,522 bytes. Check `releases/latest`,
> never `gh release list`.
>
> ⚠ **The engine rolled to v0.46.0 during the night** and its tip moved four
> times while this session ran — twice *mid-package*, once breaking the build
> (`Widget::border_color`). `cargo update` and a full build BEFORE invoking
> `package-portable.py`, which bumps the engine itself and will fail the run.
>
> ---
>
> ## ★★★ THE FINDING OF THE DAY: **AN EXCUSE IN THE VOICE OF A MEASUREMENT IS
> WORSE THAN SILENCE, BECAUSE SILENCE GETS INVESTIGATED**
>
> O142 (*"a click on empty paper cannot start new text"*) was found to have been
> **fixed for two days**. `8670523` bounded `hit_test` on 2026-09-05 19:13; this
> shell pinned an engine carrying it at `eafe88f` **three hours later**. The
> engine's reply even said *"your dead arm at `place.rs:129` should start firing
> with no shell edit."*
>
> What hid it: six lines at the tail of `the_text_tool_types_on_one_click`
> clicked blank paper every run and, on absence, printed *"the point named an
> existing run, which is a fact about this fixture rather than about the
> feature"* — **having measured nothing.** Neither branch could fail.
>
> ⇒ That triggered a hand triage of **all 41 `reply_*` files** in `open/`, by
> three agents. **It was not one stale row. It was a class**, and it reached the
> operator four ways — see the commit messages. `MANUAL.md` telling him page
> deletion would refuse to save was the worst of them.
>
> ## ⚠⚠⚠ THREE INSTRUMENTS IN TWO DAYS HAD STOPPED MEASURING AND STAYED GREEN
>
> Read this as one pattern, not three incidents:
>
> 1. `line_weights` — SKIPping for weeks; aimed at blank paper.
> 2. `the_text_tool_types_on_one_click`'s blank-paper half — an unevidenced
>    excuse on the failing branch.
> 3. `save::tests`' page-tree guard — printing *"close the engine request and
>    delete this test's engine half"* into a **silent SKIP** since the pin moved.
>
> **A SKIP is not red.** Nothing goes amber. Diff the SKIP set between runs.
>
> ## ⚠⚠ `#[non_exhaustive]` MEANS YOU HAVE NO COMPILE-TIME GUARANTEE, AND TWO
> ## COMMENTS CLAIMED ONE
>
> Bit twice in one evening, in unrelated code:
>
> * `reflow_refusal`'s `_` arm was about to swallow
>   `ReflowApplyError::PageEditedThisSession` — **the engine's reply warned
>   about it by name**, hours after shipping it. No compile error would have
>   fired. Fixed by routing through `ReflowDecline`, which the engine declares
>   **not** `#[non_exhaustive]` on purpose, so the match is compiler-proved.
> * `EngineRefusal`'s doc claimed it *"turns a new engine variant into a compile
>   error here"*. False the day it was written and unfixable —
>   `EncryptError::RedactionPending` fell through for three days and put
>   `save_applying_redaction` in a draughtsman's dialog.
>
> ⇒ Where the engine offers an exhaustive discriminant, switch on it. Where it
> does not, **say so plainly and let `check-engine-api-drift` be the alarm.**
>
> ## ★★ NO GATE CAN CATCH THIS CLASS ON THE CHANNEL AS IT STANDS
>
> `check-stale-blockers` detects *"we wired it and forgot the row"* and
> structurally cannot detect *"the engine fixed it and we never noticed"*. And
> none can be built: `open/` holds 41 replies whose names share **no key** with
> the 57 `done_*CONSUMED*` markers, while its README still declares *"open/ …
> EMPTY = NOTHING IS OWED"*.
>
> ⇒ The engine session has said, unprompted, that it has the identical problem,
> an owed unbuilt tool (`R242`'s `check-requests-scoped.py`), and that **a
> shared key between the two naming conventions is the obvious fix and neither
> side can adopt it unilaterally**. ★ It explicitly invited the note. **Send it.**
>
> ---
>
> ## ⬜ WHAT THE TRIAGE FOUND AND THIS SESSION DID NOT FIX
>
> All verified by grep, all with `file:line` in the commit messages:
>
> * ~~**`FillOutcome::applied_autosize_bound` / `AutoFitBound`**~~ ✅ **CONSUMED
>   2026-09-08, driven.** The `Floor` case now says the text will overflow and
>   names the remedy; the `Width` case got its own sentence because it points
>   at a different edit. O86 corrected.
> * ~~**`preview_font_resources_for` / `Std14Entry` / `standard_14`**~~ ✅
>   **CONSUMED 2026-09-08, driven.** Both halves: the chooser reads the engine's
>   survey, and the refused-character offer passes the refused character as the
>   candidate. `refused_char_untested` deleted; a dead-end sentence added for
>   the state the filter exposed (an EXACT list can be empty).
> * **Widget `/MK /BG` + `/BC`** — `MkColor` still unconsumed. ⚠ **But NOT for
>   the reason this list first gave.** *"The shaded field turns grey while it is
>   being edited"* is a **deliberate design position** argued in `canvas::forms`
>   §3 (*"the editor is deliberately not a facsimile"*), not a defect — and
>   repainting it would re-open D2. The honest consumer is the **Properties
>   panel**, showing the colours as readable properties. Row corrected
>   2026-09-08; see `ENGINE_BACKLOG.md` for the whole argument.
> * **`set_annotation_flags`** (new, `fad0d2d`) — verdict written, order fixed:
>   **honour `LockedContents` on the way IN before offering the writer that
>   clears it.**
> * ~~**`FieldRename::action_targets_retargeted` / `FieldDeletion::action_targets_orphaned`**~~
>   ✅ **CONSUMED 2026-09-08.** Both worded and conditional; the rename says the
>   buttons still work, the delete says they will *"do less than they say"*, and
>   both admit JavaScript was not handled. ⚠ **Chain NOT driven** — the blocker
>   is a fixture whose action names its target as a NAME STRING;
>   `submit-button.pdf` uses an object reference, which the traversal is blind
>   to. Named at the test site.
> * **`FontPreflight::real_bold` / `real_italic`**, and stale prose at
>   `textstyle.rs:75-87` (*"Bold is unreachable there through either route"*),
>   `editrefusal.rs:534` (*"this limit is on the list to fix"* — it shipped),
>   `FEATURES.md:1070`, `OPERATOR_REQUESTS.md:619-631`, and three
>   `ENGINE_BACKLOG.md` rows asserting a pin that has moved.
>
> ---
>
> ## The 2026-09-07 AFTERNOON block, kept below — its numbers are superseded by the table above
>
>
> ## ★★★ THE FINDING OF THE DAY: **A CONTRACT YOU WRITE FOR SOMEBODY ELSE'S
> FUNCTION IS A CLAIM TO MEASURE, NOT TO CARRY OVER**
>
> `canvas::annotquad` began the day as a declared workaround with its own
> `/Matrix` reader; `Pass 155.2` made `pdfcer_render::annot::appearance_placement`
> public and it became a thin adapter. The rewrite kept the old
> `OrientedBox::degrees` doc comment, which said *"normalised to `[0, 360)`"* —
> true of the deleted local implementation, which applied `rem_euclid`, and
> **false of the engine's `appearance_rotation_degrees`, which returns a signed
> `atan2`.**
>
> `is_upright` range-tests `(0.1..=359.9)`. So **every clockwise rotation
> reported itself upright**, `-89.15` fell outside the range, and the selection
> outline went straight back to being axis-aligned on the turn an operator makes
> most often.
>
> **Every one of the module's tests used a positive angle. All 31 gates were
> green.** It was found on the first driven run after the pin moved, by
> `rotating_a_markup_turns_it`, whose drag happens to be clockwise: `turned=0`
> on the same trace as a line saying the engine had turned the mark.
>
> ⇒ Two rules, and the second is the sharper one:
> **(a)** when a function is replaced by somebody else's, its doc comment
> describes the *old* one until it has been re-measured against the new one;
> **(b)** **a test suite that only ever exercises one sign of a value is not
> testing the value.**
> `a_clockwise_turn_is_normalised_into_the_same_range_as_an_anticlockwise_one`
> asserts both signs in one test, on purpose.
>
> ---
>
> ## ★★ THE TRIPWIRE FIRED FOUR HOURS AFTER IT WAS WRITTEN, AND THAT IS THE
> PATTERN TO COPY
>
> `annotquad`'s tripwire read the **pinned** engine's own `annot.rs` — located
> through `Cargo.lock`, so it follows the pin and cannot read `D:/Dev/pdfcer`'s
> working tree, which moves several times a day and is often ahead of what
> compiles here. On the first `cargo update` after `Pass 155.2` it failed with
> *"`Annotation` now has a `appearance_matrix` field"*, named what to delete, and
> pointed at the request to close.
>
> ★ **The version it replaced could not have fired**: `const
> ENGINE_HAS_TAKEN_OVER: bool = false` with a `debug_assert`. Nothing but a human
> who had already noticed could set it. Clippy flagged it, for a worse reason
> than it knew. It is kept **inverted** as
> `the_engine_owns_the_placement_and_this_module_only_projects`, because the live
> hazard is now the opposite one — somebody re-deriving the placement here *"just
> this once"*.
>
> ---
>
> ## ★★ TWO GATES CAUGHT WHAT NOBODY HAD BEEN TOLD
>
> **`check-engine-backlog` is how the rotation reply was noticed at all.** It
> reads the ENGINE's own `docs/FEATURES.md`, found a `[x] core / [ ] gui` row for
> *"Read an annotation's rotation, set it absolutely"*, and refused to pass.
> Nothing else in this session knew the reply existed.
>
> **`check-engine-api-drift` then named the two new public items this repository
> mentioned nowhere** — `AnnotationRotate::rect_derived_from` and
> `annot::rotation_degrees`. Note how it accounts for a name: **leaf AND owner
> must both appear in the workspace's sources or a root-level register**, so
> using `outcome.rect_derived_from` is not enough on its own; `AnnotationRotate`
> has to be named somewhere too.
>
> ---
>
> ## ⚠ THE ONE ROTATION CASE THAT STILL GROWS, AND IT IS NOT A DEFECT
>
> `RectDerivation::PreviousRect` — an annotation with **neither** an appearance
> stream **nor** rotatable geometry. Its artwork *is* its rectangle, §12.5.2
> requires that upright, so there is nowhere in the annotation an orientation
> could be recorded. **No rule can do better**, and the engine warned that a grip
> ignoring it *"re-introduces the operator's bug one level up, on exactly the
> annotations that cannot be fixed."*
>
> `text::rotating::rect_still_grows` fires on that rule **and only** that rule.
> `fixtures/square-no-appearance.pdf` is 479 hand-written bytes that reach it —
> hand-written because `add_markup` correctly never writes an annotation without
> artwork, so there is no route to the case through the engine's authoring API.
>
> ⬜ **Still owed**: baking an appearance for such a mark, which is the fix
> rather than the disclosure. The engine names it as one of the two honest
> options; this shell takes neither yet and says so.
>
> ---
>
> ## ⬜ WHAT IS BUILT AND UNDRIVEN
>
> * **Typing into the Angle field and pressing Apply.** The *read* is driven
>   (`rotating_a_markup_turns_it` asserts `angle=270.85` after a `-89.15` drag);
>   the write goes through `AnnotAction::SetRotation` →
>   `set_annotation_rotation` and no check presses Apply.
> * **The Left/Bottom/Width/Height fields for an ANNOTATION** — no driven check
>   anywhere touches `properties.annotgeometry.*`. Older than the Angle field.
> * **Everything from 2026-09-06's markup session** that O144 lists, unchanged.
>
> ## ⬜ THREE ENGINE CAPABILITIES WITH NO CALLER HERE
>
> Unchanged from this morning and still true: `Diagnostics::strokes_hairlined`,
> `place_text`, `blank_document`. ⚠ **And five shell files still assert that the
> text-import route does not exist**, which is false at our pin —
> `app/actions/exporttext.rs:36`, `dialogs/export_text.rs:12`,
> `dialogs/mod.rs:99`, `text/commands/file.rs:63`, `text/export_text.rs:13`.
>


> ★★★ **LAST SESSION: 2026-09-07. HIS THREE ROTATE SENTENCES — TWO SHIPPED AND
> DRIVEN, ONE FILED AT THE ENGINE WITH A PIXEL REPRODUCTION.** Read
> `git log -1 --format=%B` in full before anything else; it is the authoritative
> record and carries far more than this block.
>
> **State, every number re-measured 2026-09-07 in the session that wrote this
> block:**
>
> | | | measured with |
> |---|---|---|
> | tests | **3,856 passing, 0 failing** | `cargo test --workspace`, summing every `test result: ok. N passed` through `awk` |
> | gates | **31 of 31, 0 skipped** | `bash tools/gates/run-all.sh` |
> | engine | `pdfcer-core` **v0.44.0 at `e1bdb6c`** | `grep -A3 'name = "pdfcer-core"' Cargo.lock` |
> | shell | `b792cac`, on `origin/main` | `git log -1` |
> | driven checks registered | **190** | `Box::new(` inside `roster.rs`'s `all()` — the bare file count is 191, and the extra one is not in `all()` |
>
> **Released 2026-09-07 14:41** from `b792cac`: OneDrive **`pdfcer-gui2` is the
> new build**, so **`pdfcer-gui1` (2026-09-06 22:35) is the fallback**. GitHub
> `v0.5.0-dev.20260907.1`, **verified from HIS side** —
> `gh api repos/KenM76/pdfcer-gui/releases/latest` returns that tag,
> `prerelease: false`, zip attached. Check `releases/latest`, never
> `gh release list`: the list is the publisher's view and it is what hid a
> three-day-old front page for five builds.
>
> ---
>
> ## ⚠⚠ THE ENGINE PIN CANNOT MOVE WITHOUT WORK, AND THAT IS THE NEXT JOB
>
> `cargo update -p pdfcer-core -p pdfcer-render -p pdfcer-print` takes us to
> **v0.44.1 at `bb07cd0`** and **it does not compile here** — measured, then
> reverted with `git checkout -- Cargo.lock` before packaging:
>
> ```
> error[E0204]: the trait `Copy` cannot be implemented for this type   (StickyIcon)
> error[E0004]: non-exhaustive patterns: `StickyIcon::Other(_)` not covered
> ... 11 errors, all on the sticky-note surface
> ```
>
> `StickyIcon` gained an **`Other(String)`** variant and lost `Copy` — that is
> `Pass 253.5`, the engine's fix for
> `request_set_text_annot_style_rewrites_a_foreign_icon_name.md`, which **this
> shell filed**. Adopting it is a real piece of work with a UI question in it
> (what does the icon chooser show for a name it does not know?) and it is
> **the next job**. `reply_2026-09-07-both-set-text-annot-style-defects-FIXED.md`
> is the reply; read it first.
>
> ⇒ **Testing before packaging is what caught it.** `package-portable.py` runs
> `cargo update` unless you pass `--no-update`, so a green test run taken
> beforehand describes a different engine from the one that would ship. That is
> the second time this rule has paid.
>
> ---
>
> ## ★★★ THE FINDING OF THE DAY: **"IT IS CHEAP TO READ" AND "IT DOES NOT NEED
> RE-READING" ARE DIFFERENT CLAIMS, AND THE FIRST WAS USED TO JUSTIFY THE
> SECOND**
>
> `AnnotSelection::outline` — the box the selection outline and all nine handles
> are drawn from — was **stale after every edit to an annotation**: move,
> resize, rotate, a typed Apply in the properties panel, an undo of any of them.
> It only came back to the mark when the operator clicked away and clicked back.
> Measured from a driven trace: a quarter turn took `/Rect` from
> `473.7 × 249.6` to `256.6 × 477.4` and the painter went on stroking the first.
>
> ★★ **The field's own doc comment had argued that no refresh was needed**, in a
> section headed *"Why it needs no `resolved_for` twin"*:
>
> > *"[`outlines`] is cached against `(page, epoch)` because content bounds cost
> > a `decompose_page`… An annotation's outline is its `/Rect`, four numbers in a
> > dictionary, so it is re-read on the frame the selection is made and carried
> > on the selection itself."*
>
> Every clause is true. The conclusion does not follow. `SelectionState::
> resolve_annot` is the twin, keyed on `(page, epoch)` exactly as the content
> half always was, and it is **cheap in precisely the way the old note claimed**
> — one `/Annots` walk, no decomposition — which is why the fix is to run it
> rather than to invalidate harder.
>
> ⇒ **Wherever a comment explains why something is NOT refreshed, check whether
> the argument is about cost or about correctness.** This one was about cost and
> was answering a correctness question.
>
> ★ It also re-reads `/F` bit 8 now, so a lock applied to a selected mark takes
> the handles away on the next frame rather than the next click.
>
> ---
>
> ## ★★ A DISCLOSURE HAS A SUBJECT, AND WHEN THE SUBJECT GOES IT IS A LIE WITH A
> CITATION ATTACHED
>
> `text::rotating::rect_grew` said *"the dashed box around this mark is now
> larger… the mark itself is exactly the size it was."* Correct, well argued,
> and cited to §12.5.2 — **while the outline was drawn from `/Rect`**. The
> outline is now drawn at the mark's own angle, so no box swells and the
> sentence described something that does not happen.
>
> **Deleted**, with the whole account kept at its site — including the warning
> **not to restore it for O145**. A mark rotated twice really does get bigger,
> and that is a *defect*, not a consequence to disclose; disclosing a defect as
> though it were correct behaviour is how the old GUI accumulated its red flags.
> Its two unit tests went with it, and are **named in a comment where they
> stood**, because a test that vanishes looks like coverage nobody wrote.
>
> ---
>
> ## ★★ A TRIPWIRE KEYED ON YOUR OWN INTENTION IS NOT A TRIPWIRE
>
> `canvas::annotquad` is a **declared workaround** — this shell reads the
> appearance `/Matrix` itself and re-implements ISO 32000-1 §12.5.5, because
> `pdfcer_core::annot::Annotation` models no rotation and `pdfcer-render`'s
> placement is `pub(crate)`. Its first tripwire was
> `const ENGINE_HAS_TAKEN_OVER: bool = false` with a `debug_assert` beside it,
> which is **not a tripwire**: nothing can set it but somebody who has already
> noticed. Clippy flagged it as *"this assertion has a constant value"*, for a
> worse reason than it knew.
>
> What replaced it reads **the other side's API**:
> `tests::the_engine_still_has_no_rotation_field` locates the pinned engine
> checkout **through `Cargo.lock`** (so it cannot drift from the pin, and it
> reads the exact bytes `rustc` read — not `D:\Dev\pdfcer`, which moves several
> times a day) and fails when `pub struct Annotation` grows a `rotation`,
> `appearance_matrix` or `matrix` field. Falsified both ways.
>
> ⚠ It **fails rather than skips** when it cannot find the source. The crate
> could not have compiled without that checkout, so a red there means the
> locating is broken and needs fixing — never ignoring. A hard-coded external
> path turning a rename into a green check over an empty scan is a mistake this
> project has already made once.
>
> ---
>
> ## ⬜ OPEN AT THE ENGINE — re-grep before repeating this, it is dated
>
> Two filed **today**, both unanswered (the engine session paused for the day
> shortly after they landed — `D:/Dev/pdfcer`'s HEAD is `bb07cd0`,
> *"work paused"*):
>
> 1. `request_rotate_annotation_grows_the_artwork_when_applied_twice.md` —
>    **the operator's own report**. A 140 × 60 pt `/Square` turned 15° four
>    times is drawn **1.93× wider and 1.42× taller** than the same square turned
>    60° once. `rotate_annotation` bounds the *previous* `/Rect` while composing
>    only θ into the `/Matrix`; §12.5.5 step (c) then scales the artwork up to
>    fill the oversized rectangle. **No workaround exists** and the request says
>    so. `crates/pdfcer-gui/tests/annotation_rotation_grows.rs` asserts the
>    defect and **goes red the day it is fixed**, carrying its own instructions.
> 2. `request_an_annotations_rotation_angle_cannot_be_read.md` — the read model
>    has no rotation. Asks for the field **or**, preferred, a public
>    `pdfcer_render::appearance_placement` that would delete four copies of the
>    engine's own logic from this shell.
>
> Plus the three that were open before today — `/IRT` replies, review status and
> O137 are all **answered** (`reply_2026-09-06-all-six-review-and-import-requests-SHIPPED.md`);
> what is still unwired from that reply is listed below.
>
> ---
>
> ## ⬜ THREE THINGS THE ENGINE SHIPPED AND THIS SHELL DOES NOT CALL
>
> Measured 2026-09-07 by an agent grepping both trees at the pin, not read off a
> document:
>
> | symbol | in the engine at our pin | caller here |
> |---|---|---|
> | `Diagnostics::strokes_hairlined` | `pdfcer-render/src/interpret.rs` | **none** — zero hits in the whole shell |
> | `place_text` | `pdfcer-core/src/text_edit/placetext.rs` | **none** |
> | `blank_document` | same file | **none**; `app::blank::document()` still builds a page by hand |
>
> ⚠ **And four shell files still assert that the text-import route does not
> exist** — `app/actions/exporttext.rs:36`, `dialogs/export_text.rs:12`,
> `dialogs/mod.rs:99`, `text/commands/file.rs:63`, `text/export_text.rs:13` all
> say *"there is no route from a text file back into a PDF"*. **That is false at
> our pin.** A limitation sentence on this project has a shelf life measured in
> hours; these are two days old.
>
> ---
>
> ## ⬜ WHAT IS BUILT AND UNDRIVEN
>
> * **Typing into the Angle field and pressing Apply.** The *read* is driven
>   (`rotating_a_markup_turns_it` asserts 270.85° after a −89.15° drag); the
>   write is not.
> * **The Left/Bottom/Width/Height fields for an ANNOTATION**, which have been
>   there since 2026-09-06 — there is no driven check anywhere that touches
>   `properties.annotgeometry.*`. That gap is older than the Angle field.
> * **Everything from 2026-09-06's markup session** that O144 lists, unchanged.
>


> ★★★ **LAST SESSION: 2026-09-06 (late morning). A PLACED MARKUP CAN NOW BE
> FULLY EDITED — AND NOT ONE PRESS OF IT HAS BEEN DRIVEN.** His instruction:
> *"getting full editing working for the Markup tools. Also make sure you've
> used the same default colours and style look for these things as Adobe."*
> Seven parallel tracks in one tree, two commits — `99e4a01` and `f4aeee6` —
> **read both messages in full with `git log -2 --format=%B` before doing
> anything else.** They are the authoritative record and carry far more than
> this block.
>
> **State, every number re-measured 2026-09-06 in the session that wrote this
> block, not copied from the one that shipped it:**
>
> | | | measured with |
> |---|---|---|
> | tests | **3,770 passing, 0 failing**, 33 ignored, 18 suites | `cargo test --workspace`, summing every `test result:` line through `awk` |
> | gates | **31 of 31, 0 skipped** | `bash tools/gates/run-all.sh` |
> | engine | `pdfcer-core` **v0.42.0 at `821ab47`** | `grep -A4 'name = "pdfcer-core"' Cargo.lock` |
> | shell | `f4aeee6`, on `origin/main` | `git log -1` |
> | driven checks registered | **188** | `Box::new(` lines in `tools/ui-verify/src/checks/roster.rs`'s `all()` |
>
> **Released 2026-09-06 11:30** from `f4aeee6`: OneDrive **`pdfcer-gui2` is the
> new build** (its `BUILD-INFO.txt` reads `Built: 2026-09-06 11:30:22`, engine
> `821ab47`, shell `f4aeee6`), so **`pdfcer-gui1` (08:31) is the fallback**.
> GitHub `v0.5.0-dev.20260906.4`.
>
> ⚠ **`git tag` does not show that release, and a cold session will believe the
> last one was `.3`.** The remote carries `refs/tags/v0.5.0-dev.20260906.4` →
> `f4aeee6` (measured with `git ls-remote --tags origin`), and it is
> **lightweight** where `.1`–`.3` are annotated — so it has no `^{}` deref line
> and it was never fetched into this clone. `git fetch --tags` before reading the
> local tag list as a release history.
>
> ★ **The pin and the engine tree agree exactly today**, which is unusual and
> will not last: `D:\Dev\pdfcer`'s `HEAD` is also `821ab47`. Re-measure rather
> than assuming it still holds.
>
> ---
>
> ## ⬜⬜⬜ THE SINGLE MOST VALUABLE NEXT ACT: **DRIVE TODAY'S WORK**
>
> **NOTHING FROM 2026-09-06's MARKUP SESSION HAS BEEN THROUGH
> `tools/ui-verify`. Not one control.** That is not an inference from a commit
> message — it is a measurement of the tree:
>
> ```sh
> find evidence -newermt "2026-09-06 08:00"    # → nothing at all
> git log -2 --format='%h %ad' --date=iso      # → 11:03:58 and 11:23:18
> ```
>
> The newest artefact in `evidence/` is `evidence/sign-final`, written **07:56**
> — three hours *before* the first of today's two commits. Seven tracks shared
> one working tree all session and the harness **moves the real pointer**, one
> driven run at a time, so the whole of it is `⬜ BUILT AND UNDRIVEN` in
> `FEATURES.md` and every new row in `OPERATOR_REQUESTS.md` says so in those
> words.
>
> R1 is this project's founding rule: **a passing test is not a report of
> working software.** 3,770 green tests and 31 green gates say nothing about
> whether a single one of the five Format ▸ Markup controls draws, commits or
> greys correctly on screen. Drive them, in slices of six to eight, with a
> profile nobody else holds (see below). **That is the first job of the next
> session with the machine to itself.**
>
> ---
>
> ## ★★★ THE FINDING OF THE DAY: **A STAGE'S TRACE RECORDS WHAT THE STAGE
> DECIDED, NOT WHAT THE FRAME SETTLED ON**
>
> *"Open in the mode you were last in"* has been **dead since the day it
> shipped** — ten days — and the start-up trace reported it **working** the whole
> time. `f4aeee6`'s message carries the full account; the shape is this:
>
> `modes::assemble` emits
> `mode-restore stored=Some("review") using=Some("review")`, which is a **true
> statement about what that stage adopted**. Forty lines further down the *same*
> 230-line trace: `mode-changed from=Some("review") to=read remembered=true`.
> `PdfcerApp::new` builds `RibbonState` **before** it calls `modes::start` and
> seeds it with `modes().first()` — Read — and `PdfcerApp::docks` reconciles the
> pair once a frame treating the **ribbon** as authoritative. The restored mode
> lives exactly one frame and is then thrown away.
>
> ⇒ **Those two lines answer different questions, and a harness grepping only
> the first would report a working feature.** Same family as *"traces perfectly
> and does nothing"*, with the twist that **the trace is not lying** — it is
> answering a narrower question than its reader is asking. Wherever a later
> stage may overrule an earlier one, a stage-scoped trace line is **not evidence
> about the frame**.
>
> ★★ **It bit markup hardest, which is how it surfaced.** Markup is authored in
> **Review**, and the program has been reopening in **Read** every single launch
> — so the whole of this session's work was one mode-click away from being
> unreachable on every start-up.
>
> ★ **A test that cannot fail is not evidence.** A first attempt asserted
> `app.ribbon.mode() == app.modes.active()` after `PdfcerApp::new()`; it stayed
> **green with the fix deleted**, because `new` reads the real layout file and on
> a machine with no stored mode both sides are Read for a trivial reason. It was
> **removed** rather than left to imply coverage it did not have, and the
> off-screen launch is the oracle instead.
>
> ---
>
> ## ★★★ THE SIBLING TEST — how a registry-sourced value is proved factory
>
> **`ACROBAT_DEFAULTS.md` is new in the repo root** and is the source of truth
> for every per-kind default colour this shell now authors. Read it before
> changing any of them; it carries the floats, the hex, an adopted/declined
> verdict per row, and the command to re-run the measurement.
>
> ★★★ **The discriminator is a SIBLING, not a plausibility judgement.** A value
> repeated across registry keys **nobody would set together** — `cSound`
> carrying the highlighter's `1.0 0.384308 0.0` to six decimals — is a shipped
> factory table. A value that appears **once** is unproven in either direction,
> and the file says so per row rather than guessing.
>
> ★★ **The test did work in both directions on the same day, which is the whole
> reason to trust it.** It **overturned this session's own first verdict** — the
> file had argued at length for keeping yellow and dismissing Acrobat's orange as
> a personal setting; the operator answered *"change the highlighter colour to
> match adobe"* within the hour, and the corroboration (`cInk:InkHighlight`, a
> *different tool*, identical to six decimals) had been sitting unread in the
> same dump. It then correctly **rejected** a lone green
> `crichDefaults\ctextColor`, which appears in exactly one key. The
> struck-through paragraph is kept in the file because the shape of the mistake
> is the useful part.
>
> ⇒ And the framing lesson underneath it: *"is this Adobe's factory default or
> Ken's personal setting?"* was **the wrong question**. He asked to match the
> program on his desk, and the program on his desk highlights in orange.
>
> ⚠ **A recursive dump and a two-level dump disagree silently**, and the
> two-level one looks complete: `crichDefaults\ctextColor` is a third-level key
> and the first sweep of the hive never saw it.
>
> ---
>
> ## ⚠ A PROFILE-BACKED MEASUREMENT NEEDS A PROFILE NOBODY ELSE HOLDS
>
> The first off-screen measurement of the mode-restore defect was **invalid**:
> two `pdfcer-gui.exe` instances were alive at once, contending on one portable
> `userdata/` directory, and the second read a layout the first had rewritten.
> The rerun used a directory made for the purpose. ⇒ **Any check whose oracle is
> a stored preference — layout, mode, settings, recent files — must own its
> profile**, and on this machine that also means confirming nothing else is
> running before believing the reading.
>
> ---
>
> ## ⚠⚠ THIS FILE'S OWN PREVIOUS BLOCK WAS STALE, AND IT IS CORRECTED BELOW
>
> The 2026-09-05 block's closing line read:
>
> > ⬜ **OPEN AT THE ENGINE (4):** the page-tree ancestor count; markup vertices
> > unreadable (`/Vertices`, `/InkList` unmodelled — **nothing was faked**); no
> > verb authors a reply (`/IRT`); `/State` unmodelled; and **O137 — a
> > line-weights-off display mode**…
>
> **It was wrong in three separate ways at once**, every one of them measurable
> in under a minute. The corrected line now sits in that block with the
> superseded text beside it:
>
> 1. **Markup vertices had SHIPPED** — `Pass 255.0`, `35ca5be`,
>    `reply_2026-09-05-markup-vertices-SHIPPED.md` in the request channel — and
>    `canvas/annotnodes.rs` has consumed `/Vertices`, `/L` and `/InkList` since.
>    It is in the build: `git -C D:/Dev/pdfcer merge-base --is-ancestor 35ca5be
>    821ab47` → **yes**.
> 2. **The page-tree ancestor count had been FIXED** — `Pass 251.1`, `e4cefcd`,
>    `reply_2026-09-05-delete-pages-ancestor-count-FIXED.md`. Also an ancestor of
>    the pin, measured the same way.
> 3. **The header said four and the list held five.** The three genuinely open
>    items are `/IRT` (`request_a_reply_can_be_read_and_never_written.md`),
>    review status (`request_review_status_is_not_modelled_at_all.md`) and O137
>    (`request_a_line_weights_off_display_mode_for_dense_cad_drawings.md`) — no
>    `reply_*` file exists beside any of the three.
>
> ★★★ **The lesson is about WHICH document went stale, and it is not the
> obvious one.** `ENGINE_BACKLOG.md` was current. `FEATURES.md` was current — its
> own markup-nodes row names `Pass 255.0` by number. **Only `RESUME.md` was
> wrong**, and `RESUME.md` is the one document a cold session reads *first* and
> trusts *most*. ⇒ **The document with the widest readership had the weakest
> mechanism.** The backlog has `check-engine-backlog.sh`; `FEATURES.md` has a
> written bar and the `⬜ BUILT AND UNDRIVEN` convention; this file's top block
> is prose that nothing checks. Until it has a gate, **treat every "open at the
> engine" claim here as dated, and re-grep
> `D:\Dev\FeatureRequests\pdfce_FeatureRequests\open\` for a `reply_*` or
> `done_*` file before repeating it.** A `request_*` with no reply beside it is
> the only evidence that something is still open.
>
> ---
>
> ## ⚠ R2 HEADROOM — re-measured 2026-09-06
>
> `bash tools/gates/check-file-size.sh` is **clean** — 887 `.rs` files scanned,
> none over 1,500 — but it prints only the largest **three**, which is not enough
> to plan a split against. ⚠ **That 887 is the gate's own walk and `git ls-files
> '*.rs' | wc -l` says 893; the two disagree by six and both are correct for
> their method.** Do not quote one as the other — `FEATURES.md`'s Source row uses
> the git figure deliberately, because it is counting what is *tracked*, and the
> gate is counting what it *scanned*. The full list of files within twenty lines of the
> ceiling, from `git ls-files '*.rs' | xargs -d '\n' wc -l | sort -rn`:
>
> | lines | file | headroom |
> |---:|---|---:|
> | **1500** | `crates/pdfcer-gui/src/app/lifecycle.rs` | **0** |
> | **1500** | `crates/egui-shell/src/manifest/merge.rs` | **0** |
> | 1498 | `crates/pdfcer-gui/src/icons/catalog/mod.rs` | 2 |
> | 1492 | `crates/pdfcer-gui/src/app/dispatch.rs` | 8 |
> | 1491 | `crates/pdfcer-gui/src/app/actions/apply.rs` | 9 |
> | 1488 | `crates/pdfcer-gui/src/text/status/mod.rs` | 12 |
> | 1487 | `crates/pdfcer-gui/src/shell/manifest/mod.rs` | 13 |
> | 1486 | `crates/pdfcer-gui/src/app/save.rs` | 14 |
> | 1484 | `crates/pdfcer-gui/src/canvas/forms/boxes.rs` | 16 |
> | 1484 | `crates/pdfcer-gui/src/app/state.rs` | 16 |
> | 1484 | `crates/egui-shell/src/dock/mod.rs` | 16 |
> | 1483 | `crates/pdfcer-gui/src/canvas/notepopup/mod.rs` | 17 |
> | 1480 | `crates/pdfcer-gui/src/app/actions/forms.rs` | 20 |
>
> ★ **Two of the three files the 08:40 block named at 1500 have headroom
> again** — `app/dispatch.rs` was split twice this session and is at **1,492**;
> `app/actions/action.rs` is at **1,479** and is off the list entirely.
> `app/lifecycle.rs` and `egui-shell/src/manifest/merge.rs` are the two that did
> **not** move, and **one added line in either fails the build**. Split before
> adding, not after. **Markup remains the next R2 candidate** — that nomination
> is unchanged and is now backed by two new 1,100-line markup modules
> (`app/markupband.rs` 1,176; `panels/properties/markup.rs` 1,140).
>
> ---
>
> ## What is now reachable that was not this morning, in one list
>
> All ⬜ **BUILT AND UNDRIVEN**; `FEATURES.md` carries the full rows and
> `OPERATOR_REQUESTS.md` O144 carries the operator-facing account.
>
> * **Format ▸ Markup** — line colour, fill (with *No fill*), line width,
>   opacity, arrowhead position, as five `Item::Custom` controls in
>   `app/markupband.rs` on `fontband`'s architecture. `RIBBON_IA.md` §5.8 has
>   specified this row since 2026-08-12 and it was empty for twenty-five days.
> * **Fill (`/IC`) and arrowheads (`/LE`)** in `panels/properties/markup.rs` —
>   two `MarkupStyle` fields the engine has shipped for weeks with **zero GUI
>   callers**.
> * **A `canvas.markup` context menu** — *Add a point here* / *Remove this
>   point*, the discoverable form of node verbs that until now needed the Points
>   tool armed **plus** `Ctrl` / `Ctrl+Shift` and were announced nowhere. **Every
>   row asks `reshape_annotation_preview`** with the exact edit it would commit
>   and reads the **error variant**: greyed-with-reason at the vertex floor,
>   absent for a shape that will never have points. **No subtype list anywhere in
>   the shell.**
> * **Typed X / Y / W / H** for a selected annotation, and **Ctrl+D** duplicate.
> * **Arrow-key nudge** — 1 pt bare, 0.25 pt with `Ctrl` (Acrobat's direction:
>   the modifier makes the step *smaller*; Shift is untouched because it already
>   means *constrain to one axis* on this canvas) — and **z-order** front /
>   forward / backward / back, on an **Arrange** group that is **new on the
>   Markup tab** and is recorded as an amendment in `RIBBON_IA.md` §5.5.
> * **Acrobat's per-kind default colours**, measured from its registry. The
>   highlighter is **orange `#FF6200`, not yellow** — what everybody knows was
>   wrong, and this shell's `(1.0, 1.0, 0.0)` had been written from memory.
>
> **Five defects were found on the way, and they are worth more than the
> features** — the mode-restore death above, plus:
>
> * ⚠ **A sticky note / text box / stamp's style controls were drawn and could
>   not commit.** `spec_from_dict` matches ten `/Subtype`s and `/Text`,
>   `/FreeText` and `/Stamp` are not among them, so every press was refused
>   while the swatch and the opacity row sat there live. Reachability now comes
>   from that same reader **succeeding**, carried out of the call the panel
>   already made — never from a hand-written list, which is the thing that goes
>   stale the day the engine adds a subtype.
> * ⚠⚠ **`visible_when` was never read by the menu resolver, and never had
>   been.** `menu::plan::resolve` never called `Item::visible_condition()`; only
>   the ribbon did. So **every menu row meant to vanish was greyed instead — R9
>   inverted** — including two pre-existing `format.delete` rows and the dock
>   tab's Float/Dock pair, with prose at each site describing behaviour that was
>   not happening. **No test could see it: every one asked the model rather than
>   the resolution.** Three lines, calling the ribbon's own `sizing::visible` so
>   the two surfaces cannot disagree.
> * ⚠ **One edit draft was shared between a content object and an annotation
>   with the same number.** The geometry panel keyed its draft on
>   `(page, usize, epoch)`; content objects are numbered by paint order and
>   annotations by object id, both small integers, so selecting content object 7
>   and then annotation `7 0 R` on one page did not re-seed the draft and Apply
>   would have moved the annotation by the difference between two unrelated
>   boxes. Now a `Subject` enum.
> * ⚠ **`Alt+Arrow` would have nudged and paged at once.** egui's `consume_key`
>   matches with `matches_logically`, which **ignores extra Shift and Alt** — so
>   a bare-arrow consumer also fires on `Alt+Arrow`, which the keymap binds to
>   `pages.move_up`. The nudge reads `i.modifiers` itself. ⇒ **Assume nothing
>   about `consume_key` and modifier exclusivity.**
>
> ✅ **ALL FOUR OF TODAY'S ENGINE GAPS SHIPPED AND ARE WIRED — same day, and
> the turnaround is the finding.** They were filed one topic per file, rowed
> `blocked`, and answered in a single reply
> (`reply_2026-09-06-all-four-markup-requests-SHIPPED.md`) hours later. What was
> filed: a dashed border cannot be expressed *or preserved*; `set_markup_style`
> accepts a `width` for a text markup and **silently discards it**;
> `MarkupStyle::endings` is not a `StyleEdit`, so `/LE` can be set and never
> removed; and a `/FreeText`'s **painted words** cannot be rewritten.
>
> ★★★ **Two of the four were WORSE than reported, in directions this shell could
> not see.** The dash was being solidified by **four** verbs, not one — a
> resize or a vertex drag destroyed it exactly as a recolour did — so fixing the
> route named would have left three to be discovered one at a time. And
> `set_markup_note` now **re-bakes the appearance itself**, so this morning's
> twice-stated stale-words disclosure was deleted the same day it shipped;
> `MarkupNoteChange::appearance_rebaked` replaces it, and **false is not a
> failure** — it is correct and final on a sticky, a stamp, and a `/FreeText`
> pdfcer did not author.
>
> ⇒ **A limitation sentence on this project has a shelf life measured in hours.**
> Spell every engine-limit claim as a dated assertion and re-read it before
> quoting it; three documents still carried the filed wording after the reply
> landed and had to be corrected by hand.
>
> ★★ **`MarkupStyleSupport::for_subtype` deleted this shell's copy of the
> engine's subtype table** — six sites, one of them carrying a comment claiming
> the list had been *"checked against the engine source, not assumed"*. A
> comment recording a check ages; a call cannot.


> ★★★ **LAST SESSION: 2026-09-06. HE CAN SIGN A DOCUMENT.** His report of
> 2026-09-03 — *"a document cannot be signed"* — is closed: **File > Security >
> `Sign…`**, driven end to end, with the verdict taken by **a different
> subsystem in a different process**.
>
> **State: engine v0.42.0 at `d6b998f`** — read from `Cargo.lock`.
> ⚠ This line said *"v0.41.0, unchanged — `cargo update` was forbidden this
> session"*, which was true of the track that wrote it and false of the tree:
> a parallel track bumped the pin an hour earlier. **A state line written by
> one of several concurrent tracks describes that track, not the repository.**
> **MEASURE, do not quote**, as always.
>
> ⭐ **`HANDOFF_20260906.md`** is the record of this session — 84 commits, four
> releases, eight methodological findings. Read it after this block if you are
> starting cold.
>
> **Released 2026-09-06 08:31** from `f974ed9`: `OneDrive\pdfcer-gui1` is the
> new build, **`pdfcer-gui2` (05:44) is the fallback**. GitHub
> `v0.5.0-dev.20260906.3`. Tests **3,666**; gates **31 of 31** — the gate count
> rose from 30 on 2026-09-06 and several documents still say 29 or 30.
>
> ---
>
> ★★★ **THE FINDING OF THE DAY: THE CAPABILITY WAS NOT IN THE BINARY, AND THE
> WARNING ABOUT THAT WAS FORTY LINES ABOVE THE LINE THAT REPEATED IT.**
>
> `pdfcer-core` shipped `pdfcer_core::sign` on 2026-09-05 — 101 public items,
> written in answer to *this shell's own* request, its module header naming us.
> `crates/pdfcer-gui/Cargo.toml` took the crate with `default-features = false`
> and forwarded `jpx` and `ocrs`. The `signing` feature is default-on. **It was
> stripped.** Nothing failed to compile. No test went red. And
> `check-verb-coverage` scored `EditSession::sign` *consumed* for two days on
> the bare word `sign` in a doc table about the arithmetic sign of `/Count`.
>
> ★★ That is the JPX incident **verbatim**, and the comment recording the JPX
> incident is in the same `[features]` block, three days old: *"forgetting to
> forward does not fail to compile."*
>
> ⇒ **A warning does not protect a code path written after it** — fourth
> recorded instance. `tools/gates/check-forwarded-features.sh` now reads the
> ENGINE's own `default = [...]` and fails when a name in it is neither
> forwarded here nor refused in writing. **Falsified both ways.** Gate count is
> **31**.
>
> ★★★ **AND THE START-UP MERGE WAS NOT BEING CALLED.** `SHELL_FRAMEWORK.md`
> §5b's `capability:` field and `SkipReason::CapabilityAbsent` did not exist —
> §5b called them *"the gap that must be closed"*. Closing them found a second
> thing: `PdfcerApp::new`'s own comment claimed the merge ran, and the manifest
> went straight to the **strict** `validate_against`. In a lite build
> `file.sign` would have failed validation, `shell` would have been `None`, and
> `Capabilities::for_mode` returns **FULL** with no shell — **a lite build would
> have lost its ribbon and granted every authoring capability to every mode,
> including Read.** Now merged, skips traced, and held by
> `the_built_in_manifest_survives_a_build_without_an_optional_capability`.
>
> ★★★ **R8 PROVED FROM THE RUNNING PROGRAM**, two off-screen launches:
> the default build publishes `ribbon.item.file.sign` and skips nothing; the
> `--no-default-features` build publishes no such region and emits one line —
> *"`file.sign` … is provided by the `signing` capability, which this build does
> not include"*. **Gone, not greyed.** No `#[cfg]` in the ribbon, the manifest
> or any panel.
>
> ★★★ **THE VERDICT IS TAKEN IN ANOTHER PROCESS, AND THE FALSIFICATION IS THE
> WHOLE PROOF.** `a_document_can_be_signed_and_the_signature_is_in_the_file`
> signs, then opens the written file in a **fresh binary** and reads the
> Signatures panel (`Pass 10.5`'s verification side): `signature-row
> field="Signature1" integrity=verified`. Planted one flipped byte: phase A's
> trace stayed **character for character identical** — `sign-written
> field=Signature1 bytes=103508 self_verified=1` — and phase D read
> `digest-mismatch`. ⇒ The *"traces perfectly and does nothing"* class, caught
> only because the oracle is not the code under test.
>
> ⚠ **THE ENGINE HAS ALREADY MOVED PAST WHAT THIS BUILD PINS.** `Pass 10.14`
> (`187fa09`, **unreleased**, outside our pin — measured with
> `git merge-base --is-ancestor`, not read off a changelog) composes a REAL
> visible appearance: signer CN, date, reason, location, Helvetica, shrink-to-fit.
> `text::sign::placement_note` says the box is an empty frame, which is **true of
> what compiles here and false of engine `main`**. Filed in `ENGINE_BACKLOG.md`
> under *Signing hardening*; **re-read that string the moment the pin moves.**
> `check-engine-backlog` surfaced 10.14 on the day it shipped, which is the
> mechanism working.
>
> ★★ **Two harness findings.** (1) **A dialog is an OS window, so
> `session.frame()` is the WRONG frame** — the first driven run aimed every
> in-dialog click hundreds of points away and the symptom was *silence*. Use
> `driving::frame_of`, which is safe on main-window regions too. (2)
> **`Session::launch` latches the PASSWORD DIALOG as the target window** when a
> fixture needs a password at start-up; every click after it closes fails with
> *"GetClientRect failed"*. Not fixed; worked around with an empty-user-password
> fixture and written down at the constant.
>
> ⚠ **R2 fired three times.** Redaction's five `Action` variants became
> `RedactAction` (19 call sites) — the written plan nominated **markup**, and
> the reason for departing is recorded at the new enum: markup is 370 lines
> across 48 call sites in two modules, redaction 114 across 19 in the module
> that already held every body. **Markup remains the next candidate.**
> `manifest/mod.rs` → `+ item.rs`; `dispatch::protect` → `dispatch::security`
> (three commands, and signing is not protection). ⚠ `app/lifecycle.rs`,
> `app/dispatch.rs` and `egui-shell/src/manifest/merge.rs` are all at exactly
> **1500 — zero headroom**.
>
> ★ `text::security::cannot_author` was corrected for the **third** time and has
> had **zero call sites** throughout. A string nothing draws cannot be caught by
> looking at the screen, cannot be caught by a driven check, and is corrected
> only when somebody greps past it.
>
> ⬜ **Still the engine's:** certifying (`/DocMDP`) signatures, signing into a
> pre-placed field, and a reserve an operator can widen.



> ★★★ **LAST SESSION: 2026-09-05. THE OPERATOR IS AWAY AND THE MACHINE IS
> YOURS UNTIL 2026-09-08** — his words: *"I'm not at the keyboard unless I tell
> you I am there… the PC is free for you to use until Tuesday."*
>
> ⇒ **DRIVE.** `ui-verify` is the ordinary way to answer a question, not a thing
> to save up for. ⚠ **One driven run at a time** — the harness moves the real
> cursor; check `tasklist | grep -icE "^(pdfcer-gui|ui-verify)\.exe"` first.
>
> **State: engine v0.42.0 at `d6b998f`** — read from `Cargo.lock`, which is the
> only thing here worth trusting without re-running it. **Every count below and
> throughout this file is stale by construction: MEASURE, do not quote.** The
> commands are further down.
>
> ⭐ **`HANDOFF_20260906.md` is the record of the 2026-09-05/06 session** — 84
> commits, four releases, and the eight methodological findings that came out
> of it. Read it after this block if you are starting cold.
>
> **Released 2026-09-06 08:31** from shell `f974ed9`: `OneDrive\pdfcer-gui1` is
> the new build, **`pdfcer-gui2` (05:44) is the fallback**. GitHub
> `v0.5.0-dev.20260906.3`. Four releases in fourteen hours and **the slots
> alternated every time**, which is the whole point of them: whatever he is
> running, the previous build is still on disk beside it.
>
> ⚠ **Gates are 31, not 30** — `check-forwarded-features.sh` joined on
> 2026-09-06. Re-measure rather than quoting either number.
>
> ---
>
> ✅ **DONE 2026-09-06: the engine is at v0.41.0, this shell is on it, and HIS
> TYPO IS FIXED.** Struck in the same commit as the code, which is what the
> paragraph three blocks below asks for and what nobody had done twice before.
>
> **`Cargo.lock` is at `f9bc7c8` (v0.41.0).** `cargo test --workspace` = **3,596
> passing, 0 failing**. Gates **30 of 30, 0 skipped**. Both rewritten driven
> checks pass on the real binary and both were **falsified** — planted, the
> plant grepped out of the trace artifact, the check's own `[FAIL]` line
> required, restored from a byte copy. **Re-measure before quoting any of this.**
>
> ★★★ **HIS TYPO GOES IN NOW, driven on a copy of his own file**
> (`apartment work - signed.pdf`, page 2, doc-point `1,200.4,537.1`):
>
> ```text
> text-edit-caret kind=Edit page=1 run=12 len=36
> edit-text-pin page=1 run=12 one_operator=false find_len=36 occurrences=1 pinned=false
> edit-text page=1 n=1 epoch=1   ← "clien" -> "client", 36 operators spanned
> ```
>
> ★★★ **AND THE BRIEF'S DIAGNOSIS WAS WRONG, which is the most useful thing in
> this block.** The standing theory — repeated in three files — was that the
> shell sends the whole run as `find`, that `text_extract` synthesises the spaces
> inside it, and that no matcher could ever reach a string containing characters
> no operator wrote. **On his file that is false.** Measured, one `EditSession`
> per shape:
>
> | request | result |
> |---|---|
> | whole-run `find` **+ pin** — what the shell sent | `NotFound` |
> | whole-run `find`, **no pin** | **OK**, `operators_spanned=36` |
> | `"clien"` **+ pin** | `NotFound` |
> | `"clien"`, **no pin** | **OK**, `operators_spanned=5` |
>
> Thirty-six characters, thirty-six operators: **the spaces are in the
> operators.** The culprit was `Pass 256.0`'s clause *"a pinned request never
> spans"* — the shell sent a `find` **and** a `pinned_span`, so the engine looked
> for a 36-character string inside the one operator the pin named, which holds a
> single character. ⇒ **Drop the pin.** The synthesised-space case is real, is
> still documented, and belongs to his CAD drawings, not to this.
>
> ⚠ **The obvious remedy — "send only the part that changed" — would have been
> strictly worse.** The changed span alone (`"n"` → `"nt"`) occurs **33 times**
> on that page; the whole run occurs once. Narrowing the `find` narrows the
> match and *widens* the ambiguity.
>
> ★★★ **So the pin comes off ONLY when the text is unique on the page**, and
> that guard is the most important thing added today. `EditRequest` carries **no
> occurrence index** — `pinned_span` is its only disambiguator — so dropping it
> on a page holding the same words twice hands the choice to the engine's scan
> order. That document is a **signed quotation**. Where uniqueness cannot be
> established the shell refuses **by name** (`AmbiguousOnThePage`) and says how
> many it found. `canvas::textedit::page_occurrences` deliberately **over**-counts
> — every way it can be wrong refuses a safe edit rather than licensing an unsafe
> one — and `canvas::textedit::glyphwall` holds both directions on two authored
> fixtures, one where the edit must land and one where it must be refused.
>
> ★★ **`Pass 257.0` is real: the in-session font swap works end to end.**
> `facewall`'s tripwire went red on the first run after the bump, from its own
> `expect_err` message, and now asserts the success in both request shapes. O141
> is complete; the block walks `["offer", "swapped"]` and never reaches
> `"blocked"`.
>
> ★★★ **AND `facewall`'s OWN INSTRUCTION WAS WRONG — the finding to carry.** Its
> header told its reader to delete the shell's blocked-state sentence when the
> engine shipped, *"the state becomes unreachable"*. **It is not.** That arm is
> reached by **arithmetic** — the retype was raised and the edit epoch did not
> move — which is agnostic about the cause, and the falsification run proved it:
> with a refusing retype planted, the block reached `state=blocked` exactly as
> before. Deleting it would have converted every other cause into **silence**,
> which is this project's own standing cross-cutting defect. ⇒ **A state and its
> explanation have different lifetimes, and a plan written from one cause will
> happily delete the handling for all of them.** The sentence kept its state and
> lost its cause; its test now asserts it names **no** cause, naming the two old
> wordings so a future improvement cannot reinstate a falsehood from git history.
>
> ★★ **Two harness defects, both found by the first run of a branch.**
> `offer_must_retire` counted `ui-rect` lines to prove the application was alive
> — and `ui-rect` is published by a **control when it draws**, not once per
> frame. While the retype was refused the block stayed on screen publishing its
> region every frame, so the proxy always agreed with the thing it stood for. On
> a build where the retype **lands** the block retires, nothing publishes, and
> the guard reported *"the application drew no frame"* over a trace holding
> hundreds of lines proving it had. It counts `canvas-pos` now. ⇒ **Ask what the
> check SAMPLED**, fourth time this month, and **the first run of a branch is
> where a proxy gets to disagree.**
>
> ★ **`Pass 256.1` is consumed**: an ambiguous `/ToUnicode` inverse gets its own
> sentence. Same engine category, same remedy, **opposite fact** — the old
> wording said the font *"was built with only the letters your page already
> prints"*, and for an ambiguous character the letter is on his page twice.
>
> ⚠ **R2 fired twice and both splits were along seams already drawn in the
> source.** `canvas/textedit/mod.rs` → `+ plan.rs` (the section banner *"Planning
> the commit"* was the seam), and `text/textedit.rs` → `+ text/editrefusal.rs`
> (banner: *"Why an edit the operator committed did not happen"*). Everything is
> re-exported, so **no call site moved**. `app/actions/action.rs` and
> `app/lifecycle.rs` are still at exactly **1500 — zero headroom**.

> ✅ **The two "DO THIS FIRST" items that stood here are DONE, and this line
> replaces them because a completed instruction left standing is worse than no
> instruction.** A cold session read them, went to do the work, and found it
> already in the tree.
>
> - The engine bump: asked for v0.39.0, and the lock is at **v0.40.0**, two
>   releases past it.
> - `ReflowRefusal::PageAlreadyEdited`'s over-broad forecast: **deleted
>   2026-09-05.** The variant still exists and should — it is now raised *from
>   the engine's answer* rather than ahead of it, which is the whole point.
>   `app/actions/textstyle.rs:657` and `text/textedit.rs:265` carry the
>   reasoning.
>
> ⚠ **This is the second time this file has aged into a falsehood at the head,
> and the mechanism is worth naming.** A `⬜ DO THIS FIRST` block is written
> when it is the most urgent thing in the project, so it is placed where it
> cannot be missed — and then discharged by whoever does the work *in the code*,
> where this file cannot see it. **Nothing closes the loop but a person.** When
> you finish something this block names, strike it in the same commit as the
> code, not at session end.
>
> ---
>
> ★★★ **THE FINDING OF THE DAY: A DRIVEN FAIL IS A CLAIM ABOUT THE CHECK TOO.**
>
> The first full 175-check sweep produced **7 harness defects** and six apparent
> application defects. **Three of the six were the check.** One relaunched the
> binary and inherited the dock layout its own previous launch had saved (so
> `docked=0` was *honest*); one asserted a marker nothing in the codebase emits
> (**it could never have passed**); one pressed blank paper inside a polyline's
> bounding box, which is a marquee and correct behaviour. **Triage before
> fixing.** And one check had failed for two days because **CapsLock was on**.
>
> ★★ **And a regression I shipped that morning was the real cause of one of
> them:** the new sticky-note pop-up is placed right of its note, and
> `Area::constrain_to` **slides** a window that will not fit back over its own
> anchor — an `egui::Area` takes every press inside it, so the drag never
> reached the canvas. *An annotation could be rotated and not moved.* It flips
> sides now.
>
> ★★ **Mouse work does NOT die at high zoom** — 9 of 9 rungs to **2,298,019 %**,
> aim residual **0.0000 canvas pt**. The `f32` precision hypothesis is falsified
> for the second time.
>
> ---
>
> ★★★ **HIS PAGE-DELETE CORRUPTION — engine-side, guarded here.** Deleting pages
> from a **nested** page tree updates the immediate parent's `/Count` and no
> ancestor, so Acrobat shows the removed pages as blanks. **Only the removal
> direction** (`delete_pages`, `page-copy --cut` — byte-identical output);
> insertion already walks to the root correctly. Invisible on every flat
> fixture, present in every real CAD export. The save now **refuses** rather
> than writing a file it knows is damaged. ⚠ **Until the engine ships, deleting
> pages from one of his SolidWorks sets and saving WILL be refused.** [SUPERSEDED 2026-09-07 - the engine shipped e4cefcd on 2026-09-05, this shell pinned it on 2026-09-06, and the sentence above stayed here for two days. Deleting pages WORKS; the damaged-page-tree guard remains and is correct. The shell test that would have said so had turned into a silent SKIP. See O134.]
>
> ★★ **The mockup had been rendering NOTHING for a day** — one JS syntax error,
> an apostrophe in *"I didn't refuse that."* inside a single-quoted string. Its
> own smoke test passed on the broken file. **Oracle: `node --check` over the
> extracted `<script>`.** And the ribbon was **two rows where the mock is
> three** (`3×22 + 2×1 = 68`) — the mock's measurements were adopted on
> 2026-09-04 and its row count was not, which is the *"halfway done"* he
> reported.
>
> ★ **`compare-mockup-ribbon.py` now compares ITEMS and exits 1** — structural 0,
> item-level **16 groups**. Its docstring had said items *"are not
> commensurable"*; true of labels, **never true of icons**, which both sides
> spell in plain text. **That is the instrument starting to work.**
>
> ---
>
> **Landed 2026-09-05, all committed and pushed:** the note pop-up (readable in
> Read); Comments panel with delete/filter/sort/go-to; deferred redaction that
> keeps undo; signature trust; encryption and permissions; lossless annotation
> clipboard; colour on a clicked text object + multi-object recolour;
> object → layer; pop-out print preview; standard-14 embedding made an **opt-in**
> (it had been silently making a *licence* decision for him); paste restored in
> Review; Document properties given its own tab; ce-dimension corners
> add/remove; read mode says how to leave; ribbon + rail auto-hide; grips that
> work on a 0.85 pt object.
>
> ⚠⚠⚠ **CORRECTED 2026-09-06 — THIS LINE WAS WRONG IN THREE WAYS AND IS THE
> WORKED EXAMPLE THE TOP BLOCK IS BUILT ON. The superseded text is kept, struck
> through, because the shape of the mistake is the part worth having:**
>
> > ~~⬜ **OPEN AT THE ENGINE (4):** the page-tree ancestor count; markup
> > vertices unreadable (`/Vertices`, `/InkList` unmodelled — **nothing was
> > faked**); no verb authors a reply (`/IRT`); `/State` unmodelled; and
> > **O137 — a line-weights-off display mode**, which he asked for by name.~~
>
> Wrong (1) because **markup vertices had already shipped** when this was
> written — `Pass 255.0`, `35ca5be`, answered in
> `reply_2026-09-05-markup-vertices-SHIPPED.md` and consumed by
> `canvas/annotnodes.rs` since; wrong (2) because **the page-tree ancestor count
> had already been fixed** — `Pass 251.1`, `e4cefcd`,
> `reply_2026-09-05-delete-pages-ancestor-count-FIXED.md`; and wrong (3) because
> **the header said four and the list held five.** Both fixes are in the pin,
> measured with `git -C D:/Dev/pdfcer merge-base --is-ancestor <commit> 821ab47`,
> not read off a changelog.
>
> ⬜ **OPEN AT THE ENGINE (3), re-measured 2026-09-06** — and the measurement is
> *a `request_*` file in `D:\Dev\FeatureRequests\pdfce_FeatureRequests\open\`
> with no `reply_*` or `done_*` beside it*, which is the only evidence that
> something is still open:
>
> 1. **No verb authors a reply (`/IRT`)** — `request_a_reply_can_be_read_and_never_written.md`.
> 2. **Review status is not modelled at all** — `request_review_status_is_not_modelled_at_all.md`.
>    (This line previously said *"`/State` unmodelled"*, which named the key
>    rather than the filing.)
> 3. **O137 — a line-weights-off display mode**, which he asked for by name —
>    `request_a_line_weights_off_display_mode_for_dense_cad_drawings.md`. ⚠ It is
>    AutoCAD's `LWDISPLAY` off (**thick → thin**), *not* Acrobat's "enhance thin
>    lines" (**thin → thick**). They are opposites.
>
> ⚠ **Zero headroom:** `app/actions/action.rs` and `app/lifecycle.rs` at exactly
> **1500**. ⚠ **Re-measured 2026-09-06: `app/actions/action.rs` is at 1,479** —
> a track split it — **and `app/lifecycle.rs` is still at exactly 1,500**, joined
> there by `crates/egui-shell/src/manifest/merge.rs`. The full within-twenty-lines
> table is in the 2026-09-06 (late morning) block at the top of this file.


> ★★★ **LAST SESSION: 2026-09-05 (overnight). NINE TRACKS LANDED AND A BUILD
> SHIPPED.** GitHub `v0.5.0-dev.20260905`; OneDrive **`pdfcer-gui1` is the new
> build**, `pdfcer-gui2` holds 2026-09-04 12:33 as the fallback.
>
> **State:** engine **v0.38.0 at `b01964f`**. `cargo test --workspace` =
> **3,390 passing, 0 failing**. Gates **29 of 29, 0 skipped**.
> `compare-mockup-ribbon.py` exits **0**. Commit `a926423`, pushed.
> **Re-measure before quoting any of these.**
>
> ---
>
> ★★★ **THE FINDING TO CARRY: NINETY SECONDS OF DRIVING BEAT 2,677 TESTS, 29
> GREEN GATES AND A MATCHING RIBBON.**
>
> Eight tracks had just landed, everything was green, and the release was about
> to be packaged. Instead the binary was launched **off screen** —
> `PDFCER_DIAG_VIEWPORT="-4200,-4200,1400,900"`, which sets `with_active(false)`
> so it takes neither focus nor pointer — with the fixture as `argv[1]`, and its
> trace read:
>
> ```text
> mode-changed to=read panels=4
> comments-panel listed=3 with_note=3 authors=3 replies=1
> ui-rect name=comments.note_edit rect=[[1086.0 347.0] - [1146.9 365.0]]
> ui-rect name=comments.delete    rect=[[1133.7 368.0] - [1239.0 386.0]]
> ```
>
> **Three Delete buttons and a note editor, in Read.** Twelve controls that
> write to the document, in the mode whose stated posture is *the document is
> not yours to alter*. Cause: two questions where only one was asked —
> `annotation_deletion_refusal` answers *"would the ENGINE refuse this
> document?"* and says nothing about the operator's stance.
>
> ⇒ **Forty-six tests over that panel could not have caught it: none of them
> enters a mode**, and `canvas::tool::capabilities` falls back to `FULL` for an
> unset `Context`, so every one ran as though it were in Edit. A predicate test
> would have been *worse than none* — it would have passed on the build that
> never called the predicate.
>
> ★★ **The smoke launch is cheap and should be routine.** Copy the exe to
> scratch, `PDFCER_DIAG=1`, viewport off screen, fixture as argv1, read the
> trace. It does not take his desktop. ⚠ The watchdog Monitor kills it anyway —
> it cannot tell an off-screen unfocused launch from a driven run seizing the
> screen — but the trace survives, and the process lives long enough to prove
> startup, layout, document open and first render.
>
> ---
>
> ★★★ **HIS REVIEW REPORT WAS AN ABSENCE, NOT A DISCOVERABILITY PROBLEM.**
>
> *"I could add a yellow sticky note but even in read mode I don't think I could
> figure out how to read it."* Three barriers, each sufficient alone: nothing
> anywhere drew a note's words on the canvas; Read could not open the Comments
> panel (`markup.comments` is on the Markup tab, and Read is shown
> `["file","view"]`); and Read's dock did not mount it. Now: a canvas pop-up,
> the panel mounted in Read, and the toggle on the **rail**.
>
> ⚠ **The toggle is on the rail and NOT on View ▸ Panels, and the reason is a
> trap worth knowing.** `RIBBON_IA.md` P1 — *one command appears on at most one
> tab* — is enforced by `Shell::validate`. Adding `markup.comments` to the View
> tab is a **validation failure**, and `Capabilities::for_mode` returns `FULL`
> when the shell is absent, so an invalid manifest **silently grants every
> authoring capability to every mode**. Eight mode-gating tests went red at
> once. The tidier fix is a rename to `view.panel_comments`; its full cost is
> written at the rail site.
>
> ★★ **pdfcer had been writing `/Popup` and `/Open` into his files since sticky
> notes shipped and nothing ever drew one.**
>
> ---
>
> ★★ **"ASKED HIM INSTEAD OF BUILDING IT" IS A CLASS, AND THE SWEEP FOUND MORE.**
> O57's grips (*"the question for him…"* — a week lost on a defect he reported
> himself), O89's colour route (three candidates listed, none chosen), O47's
> font substitution. **The tell that a "question" is not one: the candidates are
> listed.** A session that could rank the options had already done the analysis
> and was handing back only the choosing.
>
> ★★★ **O47 was worse than unbuilt — it had shipped ON.** Standard-14
> substitution was unconditional since 2026-08-28, and the engine's own CLI has
> it **off** by default because the bundled faces are BSD-3-Clause: *"embedding
> one puts it inside a document you then distribute… that is your decision to
> make."* The GUI was making a **licence** decision for him on every press.
>
> ---
>
> ⬜ **NOT DRIVEN — the canvas pop-up itself, and twenty other checks.** Twenty
> driven check modules were written 2026-09-04 → 2026-09-05 and **not one has
> run**; there is no `evidence/` directory in this tree at all. `FEATURES.md`
> now marks those rows **⬜ BUILT AND UNDRIVEN** rather than ✅. **The single
> most valuable next act is to run them when the machine is free.**
>
> ⬜ **Only he can do:** turn the trust store on, press *Show what is in it*,
> read the counts back. No check has ever seen a signature read `trusted`, and
> his store's mtime is **2024-05-27 — sixteen months stale**.
>
> ⚠ **Open at the engine, 6 requests:** added content duplicated by the next
> content edit (his bug), no route from a text file back into a PDF, no verb
> authors a reply (`/IRT`), `/State`+`/StateModel` unmodelled, `/Open`
> unreadable, a sticky note's icon and colour unchangeable after creation, and
> **`set_encryption` ignores a pending redaction** — arm a removal, then
> encrypt, and you get an encrypted file holding the un-redacted content and the
> `/Redact` marks that say where it is.
>
> ⚠ **`FEATURES.md`'s state table was wrong in every field** and is rebuilt by
> measurement. Its line-count method (`git ls-files | xargs wc -l`) reads **one
> of two batches** on 825 files — 454,385 lines, not 60,699. **Seven
> operator-visible sentences were false** and are corrected; four more are named
> and unfixed.
>
> ⚠ **Zero headroom:** `app/actions/action.rs` and `app/lifecycle.rs` are both at
> **exactly 1500**.


> ★★★ **LAST SESSION: 2026-09-05 (overnight). SEVEN TRACKS IN FLIGHT AT THE
> TIME OF WRITING** — read the "what is running" table below BEFORE editing
> anything, because half the crate is owned by a background agent.
>
> ★★★ **THE ENGINE MOVED TO v0.38.0 AND TWO GATES CAUGHT IT WITHIN THE MINUTE.**
> `cargo update` took `pdfcer-core` from v0.37.0 (`8b24a0a`) to **v0.38.0
> (`b01964f`)** — Pass 10.5 (full signature trust validation) and Pass 250.2
> (undo-preserving deferred redaction). `check-verb-coverage` immediately went
> red naming four verbs this shell has never mentioned
> (`apply_redactions_deferred`, `cancel_pending_redaction`,
> `has_pending_redaction`, `save_applying_redaction`), and
> `check-engine-backlog` named two trust-store rows. **That is the case both
> gates were written for**, and it worked on the first arrival after they
> shipped. ⇒ **Run `cargo update` for the three engine crates before every
> build.** The engine session runs in parallel and answers within the hour.
>
> ★★★ **THE FINDING TO CARRY: "ASKED HIM INSTEAD OF BUILDING IT" IS A CLASS,
> NOT AN INCIDENT.** His directive of 2026-09-04 — ***"Always add new features.
> never ask. just do."*** — was applied to one row, and then the request log
> was grepped for the *shape* (`"the question for him"`, `"his call"`, `"not
> yet decided"`, `"for now"`). It found more rows in exactly that state, each
> with the analysis complete and nothing built:
>
> · **O57** ended *"The question for him: should a selection too small for its
>   grips draw them outside the box, as other editors do?"* — his answer already
>   existed twice over, and the asking cost a **week** on a defect **he reported
>   himself**. Now built; see below.
> · **O89** — *"I don't see where I am able to edit the color of text"* — listed
>   **three** candidate fixes and chose none, and deferred multi-object recolour
>   on *"there is no honest colour to open on"*, which every editor in the class
>   solves with an indeterminate "mixed" state.
> · **O47** withheld standard-14 font embedding because substitution is *"the
>   sneaky half of rule 4"* — which identifies the right constraint and draws
>   the wrong conclusion. Rule 4 forbids doing it **silently**, not doing it.
>
> ⇒ ★★ **The tell that a "question" is not one: the candidates are listed.** If
> a session could enumerate the options and note which is cheapest, it had
> already done the analysis and was handing back only the choosing. That is the
> part he does not want back.
>
> ★★★ **AN OBJECT 0.85 pt ACROSS CAN NOW BE MOVED — O57 closed.** His report:
> *"zoom in on the atoms of the banana pdf file and see what happens when you
> try to draw a box around a molecule and move it."* Every press gave
> `resize-declined reason=Degenerate`. Two constants, each right alone: a corner
> grip reaches **6 pt** into its box and there are two per axis, and
> `MIN_OUTLINE_EXTENT_PX` **floors the drawn box at 6 pt** — so the objects with
> least body to spare were floored to a size at which they had none.
> `handles::grip_bounds` now pushes the grip ANCHOR box outward by
> `max(0, (MIN_BODY_STRIP_PX - extent)/2)` per side. **Above the threshold the
> push is exactly zero and every grip lands byte for byte where it did**, which
> is what makes it safe unconditionally. Painter and hit test both follow,
> because both read `grip_rects`.
>
> ★★ **And yesterday's fix was DELETED rather than kept beside it.** The
> perpendicular-axis filter can never be false once the push exists, and **a
> condition that cannot fail is not a guard — it is decoration that reads like
> one.** A `debug_assert` naming the invariant took over.
>
> ★★ **R2 fired on `dialogs/mod.rs` (1,535) and the seam was already there.**
> The `impl DialogsState` block held two families sharing a receiver and nothing
> else: `open_*` **build** a dialog and decide whether it may exist;
> `ask_*`/`take_*_answer`/`show` carry a **question and its answer**. Openers
> moved to `dialogs/open.rs`, and the two guards every one of them applies are
> now stated once in that header instead of re-argued at twenty-one sites.
> **1,535 → 1,068 + 539.** ⚠ `app/actions/action.rs` and `app/lifecycle.rs` sit
> at **exactly 1500 — zero headroom.**
>
> ⚠ **Disk was at 98 % with seven compilers running.** Reclaimed ~8 GB from
> `D:\Dev\pdfceGUI\target` — the **pre-rename duplicate** of this project, dead
> since 2026-09-03. Its source tree was left alone. `target/debug` here is 29 GB
> and grows unbounded; clear `debug` and `doc`, **never** `release`.
>
> ### What was running when this was written (seven background tracks)
>
> | track | owns |
> |---|---|
> | signature trust (import store, evaluate, persist) | `dialogs/settings/**`, `panels/signatures*`, `trust/**` |
> | deferred redaction (replaces the collapsing route) | `redact/**`, `dialogs/redact*`, `app/actions/redact.rs`, `app/save*`, `EDITABLE_SURFACES.md` |
> | ribbon = mockup, structurally | `mockups/**`, `shell/manifest/**`, `shell/ron/built_in.ron`, `icons/**`, `RIBBON_IA.md` |
> | object → layer highlight (page objects) | `panels/layers/**`, `canvas/select*` |
> | lossless annotation clipboard | `canvas/clipboard.rs`, `clipboard/**`, `app/dispatch/clipboard.rs` |
> | O89 colour route + multi-object recolour | `panels/properties/**`, `app/actions/textstyle.rs`, `app/conditions/**` |
> | O112 pop-out preview + O47 standard-14 embedding | `dialogs/print*`, `dialogs/preview*`, `dialogs/embed*`, `dialogs/host.rs` |
>
> **State at the last clean measurement (commit `9f3e3d2`):** engine **v0.38.0
> at `b01964f`**. `cargo test -p pdfcer-gui --lib` = **2,546 passed, 3 failed**
> — all three owned by running tracks (`dialogs::settings::…acrobat_trust_store`
> and two icon-coverage counts mid-edit). Gates **26 of 29**, the three red ones
> being `check-file-size` (now fixed), `check-verb-coverage` and
> `check-engine-backlog`, both of which fired on the engine bump and are
> assigned. **Re-measure before quoting any of these.**
>
> ⬜ **NOT DRIVEN — the whole of 2026-09-04 and 2026-09-05.** No window has been
> launched since the operator said he was back at his keyboard. Roughly a dozen
> driven checks have been written and left unrun, each saying so in its own
> header. R1 still defines what "works" means and **a green test count is not a
> substitute for it.**
>
> ⬜ **STILL OPEN AT THE ENGINE, both filed 2026-09-04 and unanswered:** added
> content duplicated by the next content edit (his bug — `add_text` appends a
> stream, the extras-sweep runs only on the session's first content rewrite),
> and no route from a text file back into a PDF.


> ★★★ **LAST SESSION: 2026-09-03 (evening). HIS PRINT DIALOG — four defects,
> and the two scrollbars alone needed four separate fixes.** Long form at the
> top of [`CONTINUE.md`](CONTINUE.md); the ledger rows are O111 (closed), O112
> (half) and O113 (not started).
>
> ★★★ **The finding to carry: EACH WRONG ANSWER READ AS CORRECT IN THE SOURCE.**
> The scrollbar deadlock had four causes — content sized from the width
> *outside* the scroll area; `auto_shrink([false, false])`, which *defines*
> content to be at least the pre-bar viewport; two un-accounted `item_spacing`
> gaps; and a control strip laid out **379.9 pt inside a 340 pt column** since
> the day it was written. Causes (c) and (d) were found ONLY by tracing egui's
> own `content_size` and `inner_rect` from a running frame. ⇒ **For a layout
> defect, instrument the process before editing it.**
>
> ★★ **And the failure was INVERTED.** Bars at 1000×760 and 1300×900 where
> nothing needed scrolling; **no bar at all** at 700×520 where a whole section
> was clipped and unreachable. One screenshot would have confirmed the wrong
> story. **Walk the size series.**
>
> ★★★ **`check-theme-colors.sh` forbids invented values, NOT wrong roles.** The
> Print button was filled from `visuals.selection.bg_fill` — a **27 % wash**
> meant for canvas selection — so it rendered *paler than the Cancel beside it*
> and he pressed it a dozen times. Correctly sourced, gate green, defect D2 for
> the third time. `Theme::accent_pair` is now the one spelling.
>
> ★★ **A hand-written list inside a completeness sweep, for the third time.**
> `dialogs_open_in_their_own_window` did not include **Print** — the dialog
> whose report started that whole piece of work — and its header rationalised
> the omission in prose. Four defects shipped in the gap.
>
> ⬜ **NOT VERIFIED, named rather than implied:** the window visibly closing
> after a REAL print. The decision is unit tested; no driven check has spooled a
> job. Needs one that prints to a file device (`Microsoft Print to PDF` is
> installed).
>
> ⬜ **NEXT, in his likely order:** O113 (the clipping hatch — decide between an
> engine verb and sampling the raster; the second is a proxy), then O112's
> pop-out preview window.
>
> **State:** engine **v0.28.0 at `e27c3b4`** — it moved under the packager
> **twice** in one afternoon. **2,886 tests, 0 failing; 23 of 23 gates, 0
> skipped.** OneDrive: **`pdfcer-gui2` is the new build**, `pdfcer-gui1` holds
> the 14:13 one.



> ★★★ **LAST SESSION: 2026-09-03 (afternoon). v0.5.0 IS RELEASED.**
> https://github.com/KenM76/pdfcer-gui/releases/tag/v0.5.0 — the first release
> in the new repository; `KenM76/pdfceGUI` is archived and holds v0.1.0–v0.4.0.
> All five historical tags were pushed to the new remote. Engine v0.28.0 at
> `562ca7e`. **2,881 tests, 0 failing; 23 of 23 gates, 0 skipped.**
> OneDrive: **`pdfcer-gui1` = new build** (`09bb966`), **`pdfcer-gui2` = the
> 08:26 build** (`eed8d3e`). The long-form record is the top section of
> [`CONTINUE.md`](CONTINUE.md).
>
> ★★★ **THE ENGINE'S RENAME LANDED AND THE SHIM IS GONE.** Three dependency
> lines now name `pdfcer-*` against `file:///D:/Dev/pdfcer` directly, the call
> site is `is_pdfcer_choice`, and `check-engine-rename-shim.sh` **deleted
> itself** as its own header instructed. Do not go looking for it; the gate
> count is 23 and going *down* by one was the mechanism working.
>
> ★★★ **AND THE RENAME HAD BLINDED THE FALSIFICATION HARNESS.**
> `ui-verify`'s `PDFCER_LEGACY` profile — the OLD GUI, the build the checks must
> be seen to FAIL against — had all four of its external names swept to the new
> spelling. They belong to the frozen build at `D:\Dev\pdfce` and did not
> rename with us. **Three of the four fail silently**: an env var the old binary
> does not read leaves its tracing off, and a trace prefix it never prints
> parses to an EMPTY trace — indistinguishable from a build that said nothing.
> The suite would have reported *"the old build does not exhibit the defect"*.
> Repaired, and now held by two falsified tests rather than by a comment.
> `profile.rs` carries `old-name-exempt-file:` because it is the one file whose
> job is to spell the old names.
>
> ★★★ **The finding to carry: A PROXY CONDITION SURVIVES ONE CORRECTION.** The
> shim tripwire's own header proudly recorded catching itself testing
> `-d D:/Dev/pdfcer`. The fix tested the *crate* on disk — still a proxy. This
> shell builds from `git` + `branch`, which resolves **committed history only**,
> and for an hour the engine held 795 staged-but-uncommitted renames: the crate
> was on disk, in no commit, and doing what the gate instructed would have
> produced an unresolvable dependency. ⇒ **Ask what the mechanism READS, not
> what a human would look at.**
>
> ★★ **Two OneDrive housekeeping facts.** The rename moved the slots to
> `pdfcer-gui1`/`2`, so the first package of the day wrote into an **empty
> pair** while the tool printed its usual *"the other slot still holds the
> previous build"* — which was false. Repaired by hand. **`pdfceGUI1`,
> `pdfceGUI2` and two `.pdfceGUI*-outgoing` folders are now orphans in
> OneDrive** and are the operator's to delete.
>
> ⬜ **NOT DONE, and named rather than implied: the 152-check DRIVEN suite was
> not swept against this build.** The machine was in use. That is the first job
> of the next session if he is away from the keyboard.



> ★★★ **LAST SESSION: 2026-09-03.** The newest handoff is the top section of
> **[`CONTINUE.md`](CONTINUE.md)**. Two of his complaints closed and one
> harness defect that had been reporting the application as broken.
>
> ★★★ **His radius/diameter tool was picking OBJECTS, and one object is half his
> sheet.** `pdfcer object-list` on `SW41177.pdf` p1: three path objects carry
> **4,405**, **4,972** and **6,681** anchors, the largest holding 1,194 subpaths
> across 550 × 500 pt. Every one of them went into the circle fit on a single
> click. It picks POINTS now, the Tool panel lists and removes them, and the
> live radius is on screen so the number can be watched converging. Driven on a
> fixture built to carry the defect, falsified at `radius 299.78`.
>
> ✅ **Redaction works on his drawings.** The engine answered
> `request_redaction_refuses_any_region_that_touches_an_image` the same
> afternoon, twice — v0.26.0 then **v0.27.0** — with all three asks and one we
> never made: **vector lines are cut at the region boundary.** Our half is
> re-worded and the report carries the new counts and three new residuals.
>
> ★★★ **The finding to carry, because it is the sixth recurrence:** the same
> paragraph in our source was wrong twice in one morning, in opposite
> directions — first written from `D:\Dev\pdfcer`'s **dirty working tree**, then
> corrected to cite the pin, and the correction was false within the hour when
> the engine shipped. **A sentence about what the engine cannot do is a dated
> citation with a shelf life measured in HOURS.** The unit test asserting the
> same claim went red the moment the engine shipped, which is the behaviour a
> paragraph cannot have. ⇒ **Where the claim can be an assertion, make it one.**
>
> ★★ **And the harness's own ribbon search was still looking for a MENU** —
> `declared_or_in_overflow` clicked the overflow once, which was the whole search
> when it was a dropdown and moves the band by one group now that it is a scroll
> arrow. Three checks SKIPPED on it in one sweep, reporting lost commands. Its
> first fix asserted the wrong thing and SKIPPED against the build it was written
> for; **driving corrected the diagnosis, the reasoning had been plausible and
> wrong.**
>
> ⬜ **Open:** the driven half of O106 — a click on an actual raster, where the
> snap declines — needs a raster fixture. Named on the row rather than implied.
>
> ★★ **If the operator typed "continue" and nothing else, read**
> **[`CONTINUE.md`](CONTINUE.md) first.** It is the short path: what is next, in
> order, and the facts that are true and surprising. This file is the long state
> document it points into.

**Written 2026-08-18, last revised 2026-08-28 after the editable-surface
audit.** For a session starting cold on `D:\Dev\pdfcer-gui`.

This file is the **entry point**. `HANDOFF.md` is the long-form institutional
record and is still authoritative for the standing rules, the phase order and
the accumulated findings — read it after this one, or when this one points you
at a section of it.

---

## ★★★ THE PACKAGER RUNS `cargo update`, SO THE BUILD IS NOT WHAT YOU TESTED

Read this before the next release, not after it.

`tools/package-portable.py` runs `cargo update -p pdfcer-core -p pdfcer-render
-p pdfcer-print` **before it builds** — deliberately, and it is the right
default. The consequence is that a green `cargo test` taken half an hour before
packaging describes a **different engine** from the one that ships.

★★ On 2026-08-30 that gap shipped a regression. The tests were green on engine
`49caa88`; the packager pulled `71d13aa` (v0.17.0) and published; re-running the
tests afterwards found **two failures**, one of which was Bold silently faking
the weight on pages carrying a real bold face. The build had to be retracted
from its slot.

⇒ **Run `cargo test --workspace` and `tools/gates/run-all.sh` AFTER packaging,
before telling him it is published.** Or run `cargo update` on the three engine
crates first and test on that. Either works; doing neither does not.

---

## ★★★ BUTTON ACTIONS ARE CONSUMED — 2026-09-01, and the delay is the lesson

`set_button_action` (`Pass 182.0`/`183.0`/`183.1`) is wired. Placing a push
button asks what pressing it does, seven ways, with the submit's disclosure.
O60 and O61 are closed. Driven by
`a_placed_button_can_be_given_something_to_do`.

### ★★ Read this before you trust any "next session, do X" note anywhere

**The paragraph this replaces said, in bold: *"read that Pass's note before
anything else next session — it is the largest closed gap on the list."***

It sat there for two days. Two sessions read it. The engine's own reply had
said *"please check your own copy — your surface is now saying something
untrue."* The Button tool stayed greyed and the dialog kept telling the
operator a falsehood, while 2,181 tests passed.

⇒ **A note is not a mechanism.** What fixed it is
`tools/gates/check-verb-coverage.sh`, which fails the build when `pdfcer-core`
has a verb this shell neither calls nor has written a sentence about in
`EDITABLE_SURFACES.md`. It found five more the moment it worked. Filed to
`D:/dev/rag/rust/` as *deletion is loud, addition is silent*.

**So: after every `cargo update`, the gates ARE the changelog reader.** Run
`bash tools/gates/run-all.sh` before believing any statement in this file about
what the engine cannot do.

### The one gap left on this subject — CLOSED, and it was closed on the day it
### was filed

> `pdfcer-core` can WRITE a button's action and cannot READ one —
> `forms::Widget` models no `/A`. So the Forms panel has no row for a button
> **already in the document**; only the placement dialog. Filed as
> `request_a_buttons_action_can_be_written_and_not_read.md`, with a tripwire
> test in `canvas::formfield::action` that names its own deletion.

⇒ **Superseded 2026-09-07.** `Pass 212.0` shipped `EditSession::button_action`
with a fourth state (`Unmodelled`) on **2026-09-01**; the tripwire fired that
day and was retired with its headstone; `panels::forms::button::row` has read
an existing button's action ever since, at `button.rs:90`.

⚠ **The paragraph above stood for six days after it stopped being true**, in
this file, under a heading calling it *the one gap left* — while the code that
closed it was already merged. Two other copies of the same claim
(`FEATURES.md`, and a present-tense doc comment in `canvas::formfield::draft`)
were corrected in the same sweep. See the 2026-09-07 reply triage.

★★ And `note_widgets_rotate_now_and_three_verbs_you_could_not_reach.md` was
audited on 2026-08-30 against the shell. Most of it is already built. The
finding that is **not**, and it is cross-cutting:

> **Every engine refusal reaches the operator as SILENCE.** `funnel.rs` traces
> the `EditError` and says nothing. A worded-decline surface exists with fifteen
> recorders, and not one of them covers a ce-dimension verb or `rotate_widget`.

So `DimensionGroupIsDefault`, `DimensionGroupNotFound` — the stale-group id the
engine warns *"will surface as a new error dialog"*, which here surfaces as
nothing — all three `DimensionLabel*` refusals and `SidecarWrittenByNewerBuild`
land in silence. Precedent for the fix: `app/actions/annots.rs:327-334`.

---

## ✅ THE FOREGROUND CAME BACK — the blocked checks run, and they pass

The 2026-08-30 blockage (*"the window could not be brought to the front"*) was
the workstation being locked, not the code. **A Windows restart cleared it.**
`turning_a_field_right_turns_it_right` now passes, asserting **270** after one
right turn — the single number that catches a missing counterclockwise negation.

★★ It failed the first time it ran, and the cause was **the harness, not the
shell**: the rotation buttons published a plain `ui_rect` rather than the
visibility-gated form, so the trace carried a rect at the control's *content*
position — y = 1,253 in a 758-point window — and the check aimed the real
pointer outside the window, pressed nothing, and reported the feature as inert.
Gated now; the check scrolls to the row and says so if it cannot get there.

⇒ **That module's own header already stated the rule** — *"the per-control
regions below take the gated form, because a check clicks those"* — sixty lines
above the code that broke it. A rule written next to the code it governs is not
a mechanism either.

---

## ★★★ RUN `tools/verb-coverage.py` BEFORE BELIEVING ANY CAPABILITY CLAIM

**One command, two seconds, and on 2026-08-28 it found twelve gaps** — including
two operator settings that were honoured by nothing and three capabilities the
engine had shipped **in answer to this shell's own requests** and this shell had
then never consumed.

```
python tools/verb-coverage.py         # the EditSession verbs nothing here calls
```

The register it feeds is **[`EDITABLE_SURFACES.md`](EDITABLE_SURFACES.md)**,
which carries a hand-written reason per miss. Read it before starting any
feature work: *"is there a verb `pdfcer-core` implements that nothing here
calls?"* is a question `FEATURES.md`, `NO_SURFACE.md` and `GUI_ROADMAP.md` are
all structurally unable to answer, because none of the three is keyed on the
engine's verb list.

⇒ **A reply arriving is not a capability landing.** The engine session works in
parallel and answers within the hour; three separate times its answer sat
unconsumed for days while this project's own documents recorded the capability
as blocked.

★ A **miss** is strong evidence (the identifier appears nowhere). A **hit** is
weak (the name appears — a call site behind a condition nothing sets is a hit
here and dead in the running program). `tools/ui-verify` is the only instrument
for the second question.

---

## ★★★ THE SWEEP NEEDS THREE FIXTURES, NOT ONE — read before running it

**`--doc-point` is one coordinate and the suite is not one family.** Running the
whole suite with a single fixture and point produced **ten failures on
2026-08-28, of which seven were the harness aiming at the wrong thing.** The
articulate failure messages named real mechanisms and were about nothing.

★★★ **THE OPERATOR'S OWN TEST FILES LIVE AT `D:\Dev\pdfTests\`** — he said so
on 2026-08-28 after this file recorded them as lost. `D:\Dev	emp\pdfcer\` was
swept and is **not** where they live; that folder was scratch and this one is
his. It holds `SW41177\SW41177.pdf` (the drawing every text and selection check
was calibrated against), `ncored-benchmark-cad-drawing.pdf` (129,758 objects,
the benchmark), `banana-at-scale.pdf` (the deep-zoom subject) and the
licensed print-conformance suite — whose name stays out of this repository by
operator ruling, and which `tools/check-suite-name-absent.py` enforces. It
caught this line.

⇒ Use it. The repo fixtures below are the portable floor — they keep the sweep
runnable on a clean checkout — but a check calibrated against his drawing should
be aimed at his drawing.

★★★ **A FAMILY IS NOT FINE-GRAINED ENOUGH, measured 2026-09-02.** Two checks in
the *same* text family need *different* points on the same drawing, and each
fails at the other's:

* `text_edit_on_a_real_drawing` wants `0,1201,1185`, because it needs a run
  spanning **one show operator** — at `0,1140,62` the shell correctly refuses
  (whole-operator editing would corrupt a split run) and the check honestly
  declines to judge. It had been reported as a permanent SKIP and was **this
  table's fault, not the check's**.
* `double_clicking_a_text_box_edits_the_text` wants `0,1140,62` and finds no
  caret at all at `0,1201,1185`.

⇒ **Each check's own module header carries its calibrated point.** When one
SKIPs or fails on aim, read that header before touching this table — the header
is written by whoever calibrated it and this table is a summary that can drift
from it, which is exactly what had happened.

| family | fixture | point | why |
|---|---|---|---|
| geometry — resize, rotate, shift-constrain, node move | `fixtures/polyline-nodes.pdf` | `0,150,260` | needs a vector object with anchors under the pointer |
| scrolling, wheel-paging, pan | `fixtures/four-pages.pdf` | `0,300,500` | needs **more than one page**, and something to scroll |
| text — double-click to edit, sweep, the Format tab's font controls | `D:/Dev/pdfTests/SW41177/SW41177.pdf` | `0,1140,62` | needs a real run of text under the pointer; this point is on one |
| text — `text_edit_on_a_real_drawing` **only** | `D:/Dev/pdfTests/SW41177/SW41177.pdf` | `0,1201,1185` | ★★ needs a run that spans **one show operator**, which is a stricter requirement than "text is here" |
| everything else | `fixtures/a1-titleblock.pdf` | `0,300,500` | a real CAD sheet with missing fonts and a title block |

```bash
for f in "polyline-nodes 0,150,260" "four-pages 0,300,500" "a1-titleblock 0,300,500"; do
  set -- $f
  ./target/release/ui-verify.exe --exe target/release/pdfcer-gui.exe     --pdf fixtures/$1.pdf --doc-point $2     --second-pdf fixtures/four-pages.pdf >> evidence/ui-verify-run.txt 2>&1
done
```

### ★★ Checks that need a fixture of their own — proved 2026-09-03

Each of these SKIPPED the sweep on the default fixture and **passes** on the one
named. None was a defect; all were the harness aimed at a document that could
not exercise the case.

| check | fixture | point |
|---|---|---|
| `ctrl_c_copies_text_to_the_os_clipboard` | `D:/Dev/pdfTests/SW41177/SW41177.pdf` | `0,1140,62` |
| `restyling_selected_text_reaches_the_document` | same | `0,1140,62` |
| `the_face_chooser_offers_a_face_the_document_does_not_contain` | same | `0,1140,62` |
| `removing_embedded_fonts_reaches_the_document` | `fixtures/embedded-font.pdf` | `0,300,500` |
| `pages_stay_drawn_when_you_scroll_back` | `fixtures/four-pages.pdf` | `0,300,500` |
| `bezier_handle_drag_changes_a_curve` | `fixtures/polyline-nodes.pdf` | `0,150,260` |
| `exporting_form_data_writes_a_file` | `fixtures/text-field-with-appearance.pdf` | `0,300,500` |

| `the_format_tab_offers_font_controls_for_swept_text` | `fixtures/paragraph.pdf` | `0,90,703` |
| `deleting_a_label_leaves_the_other_labels_alone` | `D:/Dev/pdfTests/SW41177/SW41177.pdf` | `0,1140,62` |
| `deleting_a_line_leaves_the_rest_of_the_shape_alone` | ★★ `fixtures/hole-in-a-big-object.pdf` | `0,336,500` |
| `deleting_a_point_leaves_the_rest_of_the_line_alone` | `fixtures/polyline-nodes.pdf` | `0,150,260` |

★★ **The line rung is the second check to need `hole-in-a-big-object.pdf` and for the same reason `three_clicks_round_a_hole_measure_the_hole` does**: it is ONE path object holding a circle and forty unrelated segments, which is the shape of his own export. On `polyline-nodes.pdf` — which this table's geometry row would have sent it to — the page's single object holds ONE subpath, so the delete is correct, takes the whole object with it (also correct), and the check **SKIPs** because a right build and a wrong build produce the identical census. Regenerate with `python tools/gen-hole-in-a-big-object-fixture.py`.

★★★ **And two checks now PIN their own fixture and ignore `--pdf` entirely** —
`ocr_recognises_a_page_and_the_document_keeps_it` and, since 2026-09-03,
`three_clicks_round_a_hole_measure_the_hole`. Both say so in their notes when a
`--pdf` was supplied and thrown away, because a sweep that silently ignored a
flag is indistinguishable from one that honoured it.

★★ The second pins `fixtures/hole-in-a-big-object.pdf`, and the reason
generalises: it is **one path object holding a small circle and forty unrelated
segments**, which is the shape of the operator's own drawing. On any document
whose circles are their own objects the defect it detects **cannot occur**, so
an arbitrary fixture would make the check unable to fail. Regenerate it with
`python tools/gen-hole-in-a-big-object-fixture.py`.

★★★ **That last one was called a possible defect and was not one.** It traced
`selection-set page=0 object=0 via=press` with **no** `properties-panel` line —
something selected, the panel silent — which reads as the panel failing to
describe what the canvas selected. The dock listing said otherwise:
`dock.body.file.properties` was absent from the run while
`dock.tab.file.properties` — the tab **header** — was present. **The pane was
behind another tab**, and the dock draws only the active tab's body.

⇒ **A docked pane that is not in front publishes nothing**, which is
indistinguishable from a panel with nothing to say. Any check that reads a
panel's output must bring it to the front first; `dock.tab.<id>` is published
for exactly that. The point `0,90,703` was right all along — found with
`pdfcer extract-text --json`, which gives every run's first glyph, as the
check's own skip message advises.

★★★ **`--second-pdf` IS NOT OPTIONAL, and leaving it off costs FIVE checks
silently.** Measured 2026-09-02: `two_documents_get_two_tabs`,
`document_tabs_can_be_rearranged`, `a_page_dragged_between_documents_is_copied`,
`a_shift_drag_between_documents_moves_the_pages` and
`an_attachment_moves_between_two_open_documents` all SKIP without it — and a
skip is not red, so a sweep reports the same cheerful INCOMPLETE it uses for the
hundred checks that legitimately need a pointer. Two of the five pass the moment
it is supplied; it must be a file **different** from `--pdf`.

⇒ **Re-run a failing check alone with the fixture varied before believing it.**
A check that needs a particular kind of thing under the pointer should SKIP, not
FAIL, when it does not find one — that is unbuilt and is the standing fix.

★★★ **And one of the ten was the CHECK, not the program**:
`panning_past_the_overscan_renders_the_new_area` counted `render-async-done` and
the renders were completing **inline** — `render-inline … async=0`, nineteen of
them, 3 ms each. A region raster above the pixmap ceiling is small enough never
to take the thread. The check reported *"NO RENDER WAS REQUESTED"* and named
`RenderKey::same_region`, which was working perfectly. Fixed to count both paths;
it now reports 23.

⇒ **Ask what the check SAMPLED before asking what is broken.** Third instance in
one day.

---

## ★★★ 2026-08-29 LATE — form fields copy and paste, and the harness caught THREE things a green suite would not

**Nothing packaged.** Ask him whether he wants a build. `OPERATOR_REQUESTS.md`
**O58** is shipped-and-driven, awaiting his verdict.

`Ctrl+V` pastes a copied form field as a **new** field; `Ctrl+Shift+V` pastes it
as **another box for the same field**. His ruling, his chords.

### ★★★ The three findings, in the order they will bite a cold session

**1. The DUPLICATE is the faithful paste and the NEW FIELD is the lossy one.**
The opposite of what the names suggest, and the whole design rests on it.
`add_text_field` **merges** when the `/T` already names a field
(`edit.rs:13523`, `merged: true`), so a duplicate never rebuilds the field and
inherits `/DA`, `/Q`, `/V`, `/DV` and `/AA` exactly. A new-name paste has to
re-author through `New*Field`, which is geometry-plus-booleans, and drops the
font, alignment, default value, calculation script and border colour.

⇒ **Fifth stale blocker.** We came to the request channel to ask for a
widget-clone verb and found half of it shipped. *A backlog row is a record, not
evidence.*

**2. `Ctrl+V` did nothing, and the RAG entry that predicts it did not help.**
`egui-winit` raises `Event::Paste` only when the OS clipboard holds non-empty
text. `canvas::clipboard::copy_content` writes a marker for exactly this reason
— **one function away in the same file** — and the new field-copy path did not,
because the workaround lives at each *copy site*. The general form is now in
`D:/dev/rag/egui/`: **a documented platform trap does not protect a code path
written after it.** The durable fix is a funnel, not a paragraph.

**3. `Ctrl+Shift+V` is INDISTINGUISHABLE from `Ctrl+V` at the egui layer.**
`is_paste_command` (`egui-winit-0.35.0/src/lib.rs:1406`) is
`modifiers.command && keycode == Key::V` — **shift is not excluded** — so both
chords become `Event::Paste`, the raw key is swallowed, and `Event::Paste`
carries no modifiers. The shift now comes from `i.modifiers.shift`, read in the
same `ctx.input` borrow as the events. ★ The plausible opposite assumption would
have shipped a chord that silently pasted a new field every time he asked for a
duplicate.

### ★★ And a fourth, in the CHECK rather than the program

Its box oracle summed every frame's census, so it read **1 → 3 → 6** and passed
while measuring *repaints*. Distinct `(field, centre)` pairs now: **1 → 2 → 3
boxes, 1 → 2 → 2 names**. Ask what a check SAMPLED before believing it — the
fourth instance this month.

### Open on his desk

`request_form_fields_cannot_be_pasted_and_half_of_it_already_works.md` asks the
engine for `copy_field` / `paste_field` carrying a serialisable `FieldClip`,
which would make `Ctrl+V` lossless and would work between drawings. Nothing is
blocked on the reply.

---

## ★★★ Last session: 2026-08-28 LATE — fonts embed and unembed, markup drags, and the sweep's failure count is not a defect count

**Nothing packaged.** Ask him whether he wants a build. `FEATURES.md` is
re-measured; `OPERATOR_REQUESTS.md` has **three new questions for him (O47, O48,
O49)** and none of them blocks anything.

**Measured now (2026-09-02, re-run):** **2,860 passing tests across the
workspace**, **22/22 gates**, **143 registered driven checks**. Engine pinned at
`c7a774c` (re-measured 2026-09-02 from `Cargo.lock`). ★ The 22nd gate is
`check-stale-blockers`, new today and the first one aimed at **prose being
false** rather than at code being wrong — see `HANDOFF.md` §10. ★ **O59 is done**: cut refuses what it cannot carry, pages and bookmarks copy and paste, all three driven and falsified. Scaffold list **5**.

**What shipped:** Tools ▸ Embed fonts and Tools ▸ Remove embedded fonts, both
driven end to end; dragging an ordinary markup annotation; the Bold retry using
the engine's `selector` field.

### ★★★ The three findings, in the order they will bite a cold session

**1. The full sweep reported six failures and three of them were the aim
point.** The suite takes one `--doc-point`, and it was run with the *text*
checks' coordinate. Three geometry checks then failed with articulate, plausible
messages about the wrong subject — one naming a bug fixed two days earlier.
They pass at `0,300,500`, the point this file's own sweep command uses.
⇒ **Re-run a failing check alone with the parameter varied before believing
it.** The sweep command below is verbatim for a reason.

**2. Two checks are RED and staying red.** `zooming_does_not_throw_away_where_the_operator_panned`
and `zooming_back_out_keeps_the_view` fail past ~300,000 %, reproducibly, at the
same notch. The measuring instrument is fine — it reads an `f64` line the app
publishes for exactly this — so the drift is ours: the `f32` scroll offset stops
holding the position around 300,000 % and the `f64` tier does not engage until
~1,200,000 %. **Do not widen the tolerance.** Written up as **O49**.

**3. A resolver shipped for one commit that could not have worked**, and the
class generalises: `pdfcer-core` says *"the shell resolves it, we never go
looking"*, which is a statement about **that crate**. `pdfcer-render` had the
whole three-rung matcher with `Helvetica` → `Arial` in its doctest. Grep the
**sibling** crates before writing the missing half of a documented seam; a
mirrored enum across the boundary is the tell.

### ★★ Two things a cold session would otherwise rediscover

**A harness that leaves a tool armed measures a different program.** With a
markup or measure tool armed, a click on the page is a PICK, not a selection.
Press `V` first. `sys::vk::V`'s doc comment says so and the first run of the new
markup check ignored it, then reported *"the shape could not be selected"* about
a build whose selection is fine.

**An ask with no failing artifact behind it has no forcing function.** The
engine's `Pass 144.0` reply had two "ACT ON THIS" items. The one with a failing
test attached was consumed the same evening; the one asking to change a field
name in *working* code sat for a day, because `..` in the match arm meant the
new field broke and warned nothing.

---

## Previous session: 2026-08-28 OVERNIGHT — every form verb is wired, and six stale blockers fell

**Read `CONTINUE.md` first.** Seven commits, all driven, **nothing packaged** —
ask him whether he wants a build. `FEATURES.md` is re-measured and current.

**What shipped:** a form field's properties AND its box (position, size, border,
visibility, caption), Flatten on the ribbon, and form-data **export and import**
as FDF/XFDF/CSV. Nothing in the forms family is drawn-and-dead any more.

**★★★ The finding: a blocker's reason is prose, and no test can check prose.**
Six SCAFFOLDED entries went stale in twenty-four hours; **two were citations of
citations**. The rule now written on the allow-list's own assertion — *when you
touch that list, re-derive the reason of the entry beside the one you came for*
— found the fifth and sixth within two hours of being written. **An audit of
the remaining eleven was dispatched; read its result.**

**★★ The lesson that repeated within a day:** a module's summary line and
`vector_edit`'s label must not share a trace name, or a check reads the funnel's
line and reports *"the verb did nothing"* about a verb that worked. It happened
to `text-style`, was written up, and happened again to `import-form-data` — by
the session that wrote the note. ⇒ **An incident does not generalise itself.**
The fix is a naming convention at the point of use, not a third note.

**★ Before any driven run:** Windows toasts steal foreground. Killing
`ShellExperienceHost` buys one run; turn `ToastEnabled` off *reversibly*, prior
value read and restored. `D:/dev/rag/egui/`.

**★★ Known and his call:** the Properties pane is now ~450 pt of content in a
~180 pt dock slot, so the box controls take three scrolls to reach. Three
remedies and their costs are in `FEATURES.md`.

---

## Last session but one: 2026-08-27 EVENING — the Font group, and form fields became editable

**Read `CONTINUE.md` first.** Two features shipped and both were driven and
falsified: the Format tab's **Font group** (with the three surfaces that finally
tell an operator to press `T`), and **editing a placed form field's
properties** — required, read-only, tooltip, and the type-specific flags.

**What to do next:** the **widget half** — a box's `/Rect`, border, visibility
and caption, through `edit_widget`. Shell work only; the engine verb and a
96-line pane design brief both already exist. Three status strings are written
and have no caller yet, waiting for it.

**★★★ The finding, and it cost him a day.** The Properties pane was telling him
to *delete his field and place a new one* to change a flag. `edit_field` had
shipped the same day the sentence was written, three commits before the pin,
with a design brief in the request channel that nothing read. ⇒ **An absence
claim about a crate you do not build has a shelf life.** It was true when
written and false within hours. What catches that is reading the replies.

**★★ Four more stale claims found the same evening**, three of them in our own
files: `edit.form_flatten` "unbuilt" (it was a citation of a citation), the
command count out by nineteen, the group count disagreeing with itself across
five prose sites, and our own retracted claim about synthesised spaces — which
`pdfcer-core` refuted by measuring 256 fixtures. See `CONTINUE.md`; the last one
leaves a shipped function whose stated justification is void, and **do not
replace it with a second guess.**

---

## Last session but one: 2026-08-27 afternoon — the font tools

**Read `CONTINUE.md` first.** In one paragraph: `EditSession::format_text` was
consumed, so **existing text can be restyled** — font, size, Bold, Italic,
colour, in the Properties panel's *This text* section. Shipped, driven against
the operator's own drawing, falsified, published to `OneDrive\pdfcer-gui1`.

**Three things a cold session would otherwise rediscover the hard way:**

1. ★★★ **A text RUN is not a show OPERATOR.** `layout` closes a run on geometry;
   a producer closes an operator on whatever it felt like. A title-block cell can
   be one run made of three `Tj`s. Anything that edits text must work per
   operator — `canvas::textedit::pin::operators_in_run`, and read its header
   first.
2. ★★ **Never hand `TextRun::text` to `format_text` as a `find`** — the
   symptom is real, reproducible and still guarded against. ★★★ **The reason
   this file gave for it is RETRACTED**: it said the extraction synthesises
   spaces from `TJ` offsets, and `pdfcer-core` measured 256 fixtures and found
   **zero** glyph runs containing one (`layout` emits a derived space as its
   own glyph-less run). The real offender is `/ToUnicode` mapping one glyph to
   several characters. ⇒ `Reading::find` works and its stated justification is
   void; read its doc comment before touching it, and do not replace the
   retracted mechanism with a second guess.
3. ★ **In Edit mode you must press `T` before sweeping text**, or the drag is an
   object marquee. Nothing on screen says so; that is the next piece of work.

**Eight unit tests passed while the feature restyled one piece of fourteen and
stopped.** R1 earned its keep again — the whole of what went wrong is in
`CONTINUE.md`.

## ★★★ State, as measured on 2026-08-27 MORNING, after the form-XObject selection work

**This table is a reading, not a status.** Every row is what a command printed
at that commit; the tree has moved since, and the numbers move with it. It is
here so you know roughly where you are, not so you can quote it.

| | |
|---|---|
| **Tests** | 1,830 (`pdfcer-gui`) + 420 (`egui-shell`) + 144 (`ui-verify`), 0 failing |
| **Gates** | **18 of 18**, 0 skipped |
| **`ui-verify`** | **82 checks declared. The whole suite was driven on 2026-08-27 with the operator off the machine: 76 passed, 0 failed, 6 skipped** — `evidence/ui-verify-run-2026-08-27-SUMMARY.md` accounts for every one of the six. ★ Run it in **slices of six to eight**, not as one suite: three checks that skipped inside a twelve-member batch passed when re-run in a smaller one. Per-check runs are authoritative |
| **The four defects O44 found** | **Two were real and are fixed** — the status bar going off-window at `ui_scale 1.80`, and the Properties panel's Apply being unreachable because the panel had no scroll area. **Two were the tests** — `blend_space` red on any drawing without transparency, `dimension_groups` contradicting itself in consecutive sentences. Both test defects were permanent false reds on this project's usual fixture |
| **★ Two controls have no home but the status bar** | The **selection filter** and the **zoom stepper** are reachable nowhere else — no command, no menu, no chord. `status::fitting` refuses to shed either, and its reachability test is what discovered it. If either gains a ribbon home, add it to `SHED_ORDER` |
| **Panels** | **12.** Pages · Bookmarks · Layers · Signatures · Fonts · Objects · Properties · Forms · Comments · Redact · Dimension groups · Tool |
| **Engine** | `D:\Dev\pdfcer` local `main`, taken as a **git** dependency, pinned at `4c32afe` (**v0.14.0**) — one commit past the revision that shipped `hit_test_point_deep`, `PageObjects::leaves` and the deep `pick_line_in_page`. **Read `Cargo.lock`, not this row** |
| **Latest build** | `OneDrive\pdfcer-gui2`, published 2026-08-30 13:50 from shell `33e1879` on engine `cff102a` (v0.17.0+) — **O63**: the live shape preview, the erase, the hold, the delete preview and the catching-up line. `pdfcer-gui1` holds the 09:45 O62b build as the fallback. Tested and gated **after** packaging as well as before, per the `cargo update` trap below |

### ★★★ THE FORM-XOBJECT SELECTION IS SHIPPED — AND HAS NOT BEEN DRIVEN

The operator's headline complaint — *"when I click on one of the objects all I
get is the page selected"* — was consumed on 2026-08-27, in three commits, each
of which left the program working:

1. **`TargetId` became a two-variant enum.** `Object(u64)` indexes the page's
   own paint order; `Leaf(u64)` indexes `PageObjects::leaves`, an object painted
   from inside a form XObject whose token range belongs to a *different content
   stream*. `page_object_index()` — `None` for a leaf — is the only supported
   way to obtain an edit operand, so a form-relative index cannot reach a
   page-stream verb by construction.
2. **The pick went deep** — `hit_test_point_deep`, plus a marquee half we wrote
   ourselves because the engine has no deep rubber-band.
3. **The surfaces stopped lying** — the status line, Delete, and the drag
   refusal.

★ **This file predicted 96 call sites. The compiler found sixteen.** The 96
counted places that resolve a paint-order *index*, most of which never see a
`TargetId`. The prediction was not wrong about the danger, only about the size:
budget the *care*, not the hours.

#### ★★★ It is DRIVEN, and it is PUBLISHED

`a_click_inside_a_form_selects_what_is_drawn_there` passed on 2026-08-27, and
was **falsified in the same session**: with the shallow `hit_test_point_all`
put back and the binary rebuilt, it reports the operator's own sentence back at
us. Its two assertions, both through the OS:

```
after the click on the square: first=leaf:1
after the click in the gap:    first=none
```

The second is the one that forbids a *"fall back to the shallow hit test when
the deep one is empty"* repair — the commonest empty click is blank paper inside
a page-sized form, and a fallback would answer it with the form.

**What is left is the operator's verdict.** The `OPERATOR_REQUESTS.md` O46 row
does not close until he has clicked an object on the file he complained about
and said so.

#### ★★ Three things that still do not work, and he has been told

| | |
|---|---|
| **No edit verb can address a form-interior object** | `FormLeaf::is_editable()` is `false` for every leaf. Not our decision. The remedy offered is *"Select the form"*, which lands on an ordinary page object |
| **The measure tools cannot pick a line inside a form** | `linepick` does not see the leaf list. Filed. On the benchmark CAD sheet that is 10,256 invisible candidates — and it was equally true before, hidden behind the selection defect |
| **`pdfcer object-list --hit` still answers with the form** | and its own help calls itself authoritative for the GUI's behaviour, which is now false. Filed |

#### The numbers, measured 2026-08-27 with `pdfcer object-list`

| page | page objects | forms | leaves |
|---|---:|---:|---:|
| the conformance suite's composite page 1 | 28 | 4 | **242** |
| `ncored-benchmark-cad-drawing` p1 | 129,758 | 1 | **10,256** |
| `SW41177` p1 | 5,903 | 0 | 0 |

The release binary confirms it: an offscreen smoke launch on the first of those
traces `objects n=28 … forms=4 leaves=242 depth_overflow=0 cycles=0`.

#### ★★ Two new trace fields exist BECAUSE a check could not otherwise fail

- **`canvas-selection … first=object:N | leaf:N | none`.** Before it the line
  carried a count and a rung, and selecting the page-sized form and selecting
  the square inside it both produce `sel=1 level=Object` — so a driven check
  reading that line would have passed against the broken build.
- **`objects … leaves=N depth_overflow=N cycles=N`.** `n=` counts the page's own
  list only, which is a half-truth on exactly the documents he complained about.
  The two diagnostic counts come with it because a non-zero one means `leaves`
  is a floor rather than a total.

### ⚠ ON THIS PC, pdfcer FAILS TO START ABOUT ONE LAUNCH IN THREE

**It is the machine, not the program** — settled 2026-08-26 by the operator
testing the identical portable build on his laptop, where it is fine. Do not
diagnose it again.

The symptom is a panic before any window appears, from `accesskit_windows`,
reporting `HRESULT 0x80070008 "Not enough memory resources"` on a machine with
plenty of memory.

★ **What it costs you:** `ui-verify` launches a fresh process per check, so on
this PC roughly a third of the suite cannot start and reports SKIPPED. Those are
environmental, not product defects — read the skip reason before chasing one. A
run on this machine is therefore always partial, and reporting it as a pass would
be false.

### ★★ A HARNESS AIM THAT WAS WRONG AND HAPPENED TO HIT — 2026-08-27

`checks::ocr::click_region` converted a dialog's `ui-rect` against
`session.frame()` — the **application's** window — where that dialog has been
its own OS window since 2026-08-21. It was missed in the bulk conversion to
`driving::frame_of` and **did not fail for six days**, because the Recognise
button happened to sit where the stray click landed. It failed the moment the
page-scope group pushed the button further down.

★ **A wrong aim that happens to hit is a green result reporting nothing** —
this harness's own stated worst outcome. If you convert a check to drive a
dialog, use `driving::frame_of`; it is safe on a main-window region and costs
nothing, so converting pre-emptively is free.

★ And the same run found a check pinned to the wrong *fixture*: pointed at a
CAD sheet, the OCR check failed with `NothingRecognised` and the application
was right — every page already had text, so the doubling guard skipped all of
it. A check whose subject is *"did the recogniser read this page"* cannot take
an arbitrary document. It pins `fixtures/synthetic-image-only.pdf` and ignores
`--pdf`.

★★ A red herring on the way: OneDrive was found holding 404,000 handles, and
publishing builds was measurably feeding it (~27,000 per build, established with
a do-nothing control). Restarting OneDrive dropped it to 1,179 — **and the crash
rate did not change.** Real leak, real fix, wrong mechanism.


### ★★★ Colours no longer change with zoom — SHIPPED 2026-08-26

`pdfcer-render` composites transparency in a CMYK buffer whose *default* cap is
256 MiB = **13,421,772 px**; past it, blending falls back to sRGB and the
colours move (up to 16/255, measured). On **real A4 (595 x 842 pt)** that is
**zoom 518 %** — against **1946 %** for `MAX_PIXMAP_EDGE`, a factor of 3.76.
Every whole-page raster in that band came back with approximate colours.

**Both halves are now done.**

1. **`render::strategy::for_page` takes a third argument**, `Ink`, and ends the
   whole-page tier at the colour ceiling as well as the pixmap one. A region
   raster stays under the ceiling at any zoom because its buffer is sized to the
   region. Driven: at 801 % on the conformance composite page the trace reads
   `cmyk_buffer=true refused=0`, where it previously read `refused=1`.
2. **Settings > Colour > "Colours changing when you zoom"** carries
   `max_cmyk_buffer_bytes`, uncapped, parsed and formatted with the engine's own
   `parse_byte_size` / `format_byte_size` so the window and `settings.txt` speak
   the same strings.

### ★★★ …and the ONE thing not to undo about it: the tier switch is OBSERVED

The obvious implementation applies the colour ceiling to every page. **Do not.**
Measured, and the numbers are the whole argument:

| | |
|---|---|
| files declaring a subtractive page group | **15 of 4,012** in the engine's corpus — about 0.4 % |
| where the default ceiling falls on the operator's own D-size sheet | **263 % zoom** — inside his daily working range |
| transparency on that sheet | **none at all**, so nothing would have been gained |

So `OpenDoc::ink_pages` records which pages have been *seen* compositing in ink,
from the renderer's own `cmyk_buffer_engaged` / `cmyk_buffer_refused` on every
raster, written in exactly one place (`absorb_render`, traced as `ink-page`).
Only an observed page gets `Ink::Subtractive`. `interpret::page_blend_space` is
private so the engine cannot be asked directly; that is on the request channel.

★ `ui-verify blend_space` has **three outcomes** now, and the distinction is
load-bearing: SKIP when the fixture has no transparency (a CAD drawing — it used
to FAIL there, falsely, on every routine run), PASS when the ink survived, PASS
when the fallback engaged *and was disclosed*. The one assertion that can fail
either way is that `ink-page` was traced — falsified by disabling the
observation and watching it go red.

### ⚠ ON THIS PC, pdfcer FAILS TO START ABOUT ONE LAUNCH IN THREE

**It is the machine, not the program** — settled 2026-08-26 by the operator
testing the identical portable build on his laptop, where it is fine. Do not
diagnose it again.

The symptom is a panic before any window appears, from `accesskit_windows`,
reporting `HRESULT 0x80070008 "Not enough memory resources"` on a machine with
plenty of memory.

★ **What it costs you:** `ui-verify` launches a fresh process per check, so on
this PC roughly a third of the suite cannot start and reports SKIPPED. Those are
environmental, not product defects — read the skip reason before chasing one. A
run on this machine is therefore always partial, and reporting it as a pass would
be false.

★★ A red herring on the way: OneDrive was found holding 404,000 handles, and
publishing builds was measurably feeding it (~27,000 per build, established with
a do-nothing control). Restarting OneDrive dropped it to 1,179 — **and the crash
rate did not change.** Real leak, real fix, wrong mechanism.


### ★★★ Colours change with zoom — the ceiling is now READABLE, and two of our own numbers were wrong

`pdfcer-render` composites transparency in a CMYK buffer whose *default* cap is
256 MiB = **13,421,772 px**; past it, blending falls back to sRGB and the
colours move (up to 16/255, measured). On **real A4 (595 × 842 pt)** that is
**zoom 518 %**.

★ **`534 %` was ours and it was mislabelled**, in this file, in
`OPERATOR_REQUESTS.md` and in the request. The page we bisected on —
the industry print-conformance suite's composite page, `596 × 791 pt` — is neither A4 nor US
Letter, so every percentage derived from it is right for that file and wrong as
an "A4" figure. The engine repeated it for a day in a settings paragraph before
a doc sweep caught it, and its test now has a five-point band where the wide one
is precisely why it passed while the sentence was wrong. Corrected 2026-08-26.

★★ **And "about 5 GB is the maximum possible" understates it, in the unsafe
direction.** The ceiling bounds **one buffer**, and a page can hold several
page-sized ones at once — the page buffer, a transparency group's child, the
retained spare a sibling reuses, and a full backdrop copy for a knockout group.
Peak resident memory is up to about **4×** the ceiling on a page with nested
transparency. The honest sentence for the Settings control is *"up to about four
times this on a page with nested transparency"*.

**Our part, unchanged and now unblocked:** `render::strategy` switches to the
region tier at `MAX_PIXMAP_EDGE` — zoom **1946 %** on real A4 — so between
518 % and 1946 % we ask for a raster the engine cannot composite properly. A
region render below the ceiling composites in CMYK at any zoom (proved), so the
repair is to move our switch down.

**It needed a number the engine kept private. It is public now** — v0.14.0, see
the engine-pin note above. Use
`pdfcer_render::will_composite_in_cmyk(w, h, max_bytes)`, **never** a hardcoded
13,421,772: the predicate keeps the 20-B/px arithmetic on their side of the
boundary, which is the whole reason the request refused to hardcode it.

Until then the status bar discloses it (`status-group:blend-space`), and
`ui-verify blend_space` asserts both halves.

### ★★ Where the forms work got to, 2026-08-26

All three of the operator's asks are **shipped and driven**: the five ribbon
buttons place a field by click or drag, a dialog collects the details and
remembers them for the next one, and clicking an existing field in Edit mode
opens its properties in the side pane. `ui-verify form_field` proves the whole
sequence against the release binary and was falsified in both directions.

**Two things a cold session should not rediscover:**

1. **`edit.form_create_field`'s "structural certification gate" never existed.**
   It was recorded as a blocker for nine days. What the engine actually refuses
   is `TooltipChoice::Undecided`, which is an accessibility requirement and is a
   field of the very dialog the feature needed. **Fourth stale blocker in this
   project** — a backlog row is a record, not evidence.
2. **`enabled_when` greys a ribbon item and enforces nothing.** Ninety-nine
   commands carry one, and every non-ribbon route reaches the dispatcher without
   consulting it. Do **not** "fix" this with a blanket guard at the top of
   `dispatch_command`: it was written, and two tests refused it because it makes
   `Ctrl+Z` on an empty stack do nothing *and say nothing*. Greying is a hint;
   the worded decline is the answer. Only arms that act unconditionally need the
   check, and they must say why.

**What is filed and waiting:** `request_field_property_edit.md` in the request
channel — the engine has no verb to change an existing field's required,
read-only, tooltip or border. The properties pane discloses the limit and names
the remedy; do not build around it quietly.

### ★★ Two things that will otherwise cost you an hour each

**1. Always publish with `--verify`.** The 2026-08-24 23:04 publish omitted it,
so its tests and gates never ran, and the engine's new `page_blend_space_source`
setting arrived as a surprise instead of as a packaging failure. Pass
`--no-update --verify` when the build should be the one you already verified —
plain `--verify` re-resolves the engine first and can move it under you.

**2. When a driven check says it could not raise the window, read WHICH window
is holding the foreground** — the harness now prints its class, title and pid.
On 2026-08-25 nine checks skipped with the "no foreground rights" message and
the cause was a stray `OpenWith.exe` dialog on the desktop: not the harness and
not the application. `D:/dev/rag/egui/` carries the finding.

### ★★★ READ THIS FIRST: R1's debt was paid on 2026-08-19, and re-incurred the same day

The morning of 2026-08-19 paid it. The operator handed over the machine, all 38
checks then declared were driven, and **four application defects came out that
1,417 passing tests could not see** — the table further down keeps them, because
their shapes recur.

**Then he took the machine back, and six more features shipped without being
driven at all.**

| shipped 2026-08-19 afternoon | driven? |
|---|---|
| `panels::dimension_groups` — the window became a **dock panel**, six folds | ✗ |
| `MarkupKind::Cloud` — the revision cloud, its glyph, its ribbon row | ✗ |
| `panels::tool` — the Tool panel, in its own dock stack in two modes | ✗ |
| `dialogs::unsaved` — the close/open/new confirmation | ✗ |
| the settings-coverage gate, and `quad_point_order`'s control | ✗ |
| the two-tone I-beam and the pre-first-click measure hover *(morning, before handover)* | ✗ |

That is the largest verification debt this project has carried, and it is on
exactly the class of change R1 exists for: **two new panels, a new dialog on the
close path, a new markup kind, and a new ribbon row.** `CONTINUE.md` §3.0 is the
queue and it outranks everything else in this file.

★ **Three checks were rewritten and never run**, and the Tool panel took the top
stack in the right dock of Review and Edit — so **every other panel's coordinates
moved**. A red from any right-dock check is more likely to be that than a defect.

★★ **And the check the day most needs does not exist.** The Tool panel was built
to make `edit.text` and `edit.add_text` findable, and *"it renders"* is not that
claim — asserting it renders would repeat the original failure exactly, because
the commands rendered on the ribbon all along. The honest check is a **first
frame with zero clicks**: launch, open the fixture, enter Edit, screenshot, and
assert both labels are on screen inside the panel's rect.

### The four defects driving found on 2026-08-19, kept because their shapes recur

| defect | how it presented |
|---|---|
| **A `Window` with a `default_width` and no HEIGHT around a `ScrollArea` grows ~38 pt every frame** | two dialogs walked off the screen; the Manage-groups Add button was laid out at y=1114 in a body ending at y=676 — drawn, positioned, unclickable. ★ **The panel move retired the condition rather than tuning it**: a dock panel's height is the dock's, decided before the body draws |
| **The Bookmarks authoring row sat after an unbounded scroll** | `add_outline_item`, wired that morning, was **unreachable on any document with a real outline** — the 122-bookmark fixture pushed the row 129 pt below the panel |
| **A region published at the TOP of a `ScrollArea` closure over `ui.min_rect()`** | reported `0.0 pt high` for ever — an instrument that can only return one answer cannot detect what it was added to detect |
| **The Manage-groups Add button was below its own settings block** | it acts on the LIST and was positioned under a different group's settings, so it made the wrong claim about what it acts on even when visible. ★ The panel fixed the **claim**, not just the reach: Add now sits directly under the list |

★ And **three harness defects that produced confident, wrong failure reports
about working code** — see `D:/dev/rag/egui/a_ui_rect_change_log_produces_confident_wrong_failures_in_BOTH_directions.md`.
The drop caret was reported *"never published"* over a trace containing it.
**Read the trace before believing the check.**

⚠ **The suite is not deterministic.** The final full run had 35 passed · 1
failed · 4 skipped; all three of the non-passes then passed in isolation, with
messages pointing at pointer injection and window activation rather than at the
application. Per-check runs are authoritative; a full-suite red needs the member
re-run before it is believed.

Two skips are legitimate and not defects: `ocr` (no models in this build) and
`page_ops` on `SW41177.pdf` (the fixture carries 36 `/Rotate` entries, so the
evidence would be indistinguishable from its own furniture — it PASSES against
`D:\Dev\pdfcer\fixtures\synthetic\pageops\four-pages.pdf`).

### ★★ Two defects found by AUDIT rather than by driving, and both were worse

Answering `pdfcer`'s capability-register questions found two things no test and
no driven run would have reached, because neither has a symptom on screen.

| | |
|---|---|
| **Close destroyed unsaved edits, silently** | `file.close`'s tooltip promised *"You are asked what to do about unsaved edits first"* since the day it shipped, and nothing asked. Open and New too. ★ Why it survived: **the guard that should have caught it existed, was well argued, was correct, and was answering a different question** — `save_pending` asks *is a save in flight*, which is permanently `false` here by design. Fixed |
| **`Document::recovery()` is never called** | a document whose cross-reference table pdfcer **rebuilt by scanning** opens with no indication at all. `last_wins_collisions` means two definitions of one object existed and pdfcer chose between them: the operator is looking at one of two possible documents and has not been told there was a choice. Blocked on nothing. **Still open** — `NO_SURFACE.md` §3b |

The transferable half: **driving finds what an operator can see. It cannot find
a promise nobody kept, or a report nobody rendered.** Both of these were sitting
in plain source, and what surfaced them was another project reading our
documentation and asking a question about it.

### ★★★ And four recorded claims turned out to be false, in one day

| the claim | the truth | cost |
|---|---|---|
| `markup.cloud` — *"the ONLY markup kind still absent for an ENGINE reason"* | `MarkupSpec::Cloud` had shipped | the operator asked **three times** over ~3 weeks |
| `NO_SURFACE.md` — *"Opacity: blocked on the engine, `/CA` is never written"* | `set_markup_style` writes it, tests both ways | a capability inventory reporting a false blocker |
| `FEATURES.md` — *"the theme preset is not yet choosable"* | choosable since 2026-08-17 | found by the **other project** reading our file |
| the shared `gui` column | read two incompatible ways for weeks, neither side seeing a contradiction | seven rows re-based down |

> **A blocker that names a repository this project does not build cannot fail a
> test.** Nothing compiles differently, nothing lints, and CI stays green
> *precisely because the feature is still absent*.

Every external blocker here is now a **dated citation** rather than a verdict.
`NO_SURFACE.md` §1c and `D:\dev\rag\rust\` carry the argument. This is the
single most expensive pattern this project has found, measured in weeks of the
operator asking for something that was not blocked.

## ★★ The harness — last run 2026-08-18, and what it found

`ui-verify` drives the real cursor and keyboard, so it may not run while the
operator is at the machine. Last run 2026-08-18 with the operator off the PC:
**28 passed, 1 failed, 3 skipped.**

★★ **It CAN type, and for months this project believed it could not.** Three of
the four new checks press keys: `add_text_takes_real_keystrokes`,
`text_annot_takes_the_keyboard_unclicked`, `every_declared_chord_dispatches`.
The fourth is `about_reports_the_build`, the first driven check of the About
window — which had no declared region, so nothing could find it, while it
carries the third-party attributions and their legal obligation.

See the founding-rule section near the end of this file. The false belief is
the single most expensive thing recorded in this repository so far, and the
shape of how it survived matters more than the fix.

### ★ The two skips, and why neither is a gap in the application

- `page_ops_round_trip` — the fixture already carries 36 `/Rotate` entries, so
  the check's evidence (find `/Rotate 90` in the saved copy and not in the
  source) would be ambiguous on this document. Wants a fixture with none.
- `ocr_recognises_a_page_and_the_document_keeps_it` — needs a model present.

### ★★ Two checks were reporting FALSE failures, and both were believed

`print_dialog_reaches_the_spooler` stood at FAIL and `print_paper_changes_the_plan`
at SKIP, both saying the File tab declares no `ribbon.item.file.print`. True,
and false: at the harness's 1100 pt window the ribbon had correctly folded
Print into the **overflow**. It was written up as a harness gap and left.

`driving::declared_or_in_overflow` looks in both places, and both checks pass.

**That is the second false-failure-believed of the same day** — the other being
"this machine cannot send synthetic keys", which cost the keyboard. A harness
that cries wolf gets believed the way any worn-out alarm does, and the cost is
not the noise: it is that the next real failure reads as more noise. **When a
check reports something absent, ask what else could make it absent before
writing the limitation down.**

```bash
cargo run --release -q -p ui-verify -- --exe target/release/pdfcer-gui.exe \
  --pdf D:/Dev/temp/pdfcer/SW41177.pdf --doc-point 0,300,500 > evidence/ui-verify-run.txt 2>&1
```

★ **Redirect to a file.** The first attempt piped through `tail`, which threw
away the failure detail and cost a second run of the operator's window. The
second run was skipped wholesale as STALE because a source file was edited
while it was in flight — **finish the edits, rebuild, then run.**

### The one failure, and it is the harness rather than the application

`print_dialog_reaches_the_spooler` reported that the File tab declares no
`ribbon.item.file.print`. It does not — at **1100 pt**, the harness's window
width, the ribbon has correctly folded the **Print, Document and pdfcer groups
into the overflow**, and `ribbon.overflow` is declared in the same frame.

Two things are true and both matter:

1. **The check has a gap.** `settings_headings_legible` meets the same
   condition in the same run and handles it — *"the Settings control is not on
   the ribbon band at this window width … Opening the overflow to reach it."*
   The print check needs the same step. `print_paper_changes_the_plan` skips
   for the same reason and defers to it, correctly.
2. **`file.new_from_template` made the File band one item wider** on
   2026-08-18, which is what tipped it over at that width. On a wide window
   Print is still on the band; it was not verified at the operator's own window
   size.

**Fix the check first**, and only then decide whether the size chooser has
earned a place on the band — that is an IA question for the operator, not a
build session's.

### The three skips are honest and named

| check | why |
|---|---|
| `page_ops_round_trip` | the fixture already carries 36 `/Rotate` entries, so the evidence would be indistinguishable from the document's own furniture. Point `--pdf` at `D:\Dev\pdfcer\fixtures\pageops\four-pages.pdf` |
| `ocr_recognises_a_page_and_the_document_keeps_it` | needs the `models/ocrs` weights beside the exe, i.e. a **packaged** build |
| `print_paper_changes_the_plan` | ★ FIXED — both now look in the ribbon overflow |

### Still not written

★★ **A first-frame discoverability check, and it is the most important missing
one in the project.** Launch, open the fixture, enter Edit, screenshot with
**zero clicks**, assert the strings `Add text` and `Edit text` are on screen and
inside the Tool panel's rect. The panel was built because the operator could not
find two commands that were on the ribbon all along, so a check asserting the
panel *renders* would repeat the original failure exactly — the commands
rendered too. Pair it with: arm `edit.text`, click blank paper, assert
`Refusal::NoRun`'s sentence is on screen in the panel. That second one **fails
today** for want of anywhere to put the sentence, which is the point.

An **annotation-selection** check — click a stamp, assert `annot-select`, press
Delete, assert one fewer annotation. Every trace line it needs already exists,
and Delete can now be pressed, because the keyboard works.

An **unsaved-edits** check — make an edit, press Close, assert the confirmation
appeared **and that the document is still open**; then press *Close without
saving* and assert it is not. The second half is the one that matters: a
confirmation that appears and is ignored is worse than none, and until
2026-08-19 that path destroyed the operator's work silently while its own
tooltip promised otherwise.

★ **A check that types for real is DONE** — three of them, in fact
(`add_text_takes_real_keystrokes`, `text_annot_takes_the_keyboard_unclicked`,
`every_declared_chord_dispatches`). The text-EDITING check still seeds its
draft through `PDFCER_DIAG_TYPE`, which bypasses the event loop; that seam is
now a convenience rather than a workaround, and the link it skips is covered
by `add_text`.

**Re-measure before you rely on any of it.** Prose drifting from a number is a
defect this project has spent seven corrections on — the gate runner's own
header spent months saying "Three gates carry one" while four ran, and the
README claimed 1,530 tests against an actual 1,839. Both were fixed by
deleting the count, not by updating it. Do the same here if you find yourself
tempted to edit a number rather than re-run the command.

```bash
git log --oneline -1
cargo test --workspace
bash tools/gates/run-all.sh
cargo run --release -q -p ui-verify -- --exe target/release/pdfcer-gui.exe \
  --pdf D:/Dev/temp/pdfcer/SW41177.pdf --doc-point 0,300,500
python tools/package-portable.py --verify --note "what this milestone added"
```

The `ui-verify` skips are honest and named: `page_ops` wants a fixture with no
`/Rotate` furniture, and `ocr` wants the model weights beside the binary.
`markup_style` is intermittent — it skipped in the run before last and passed
in the last one, so treat a third skip as scheduling, not as a regression.

---

## What to do next, in the operator's likely order

> ### ★★★ Read this before picking anything: the order came from the wrong place
>
> On 2026-08-18 the operator said, of a day and a half of work:
>
> > *"I'm still flabbergasted by how the GUI is still not user friendly. …It
> > feels like nothing is moving forward on these things. …Hours and hours,
> > and I click and can't figure out how to enable some of the basic stuff."*
>
> He was right, and the cause was **scheduling, not difficulty**. This list
> used to be ordered by *what the engine most recently unblocked* — which is
> how print paper sizes and a page-size chooser got built while
> **clicking a stamp did nothing at all**. Both were things he had asked for;
> neither was on the path to what he was actually hitting.
>
> **Order this list by what the operator reaches for, not by what just
> arrived in the channel.** The engine's replies are an input to *how* a thing
> gets built, never to *which* thing.

### ★★★ First: DRIVE. `CONTINUE.md` §3.0, and it outranks everything below.

**Six features are shipped and undriven** — two panels, a dialog on the close
path, a markup kind, a ribbon row, and two cursor fixes. Three checks were
rewritten and never run. The Tool panel took the top stack in the right dock of
two modes, so every other right-dock check's coordinates moved.

It needs nothing but the desktop, and the operator off it.

### 1. The Format tab's first slice — restyle a selected annotation

**Selection landed 2026-08-18** (`00ff4c7`): a stamp, note, shape or ce
dimension can be clicked, is outlined, and can be **deleted**. What is still
missing is everything that made the operator ask *"how do I **edit** a stamp"*:
colour, width, opacity.

`EditSession::set_markup_style` is the verb and it is shipped. The routing is
already done for you — `AnnotKind` on the selected target is `Markup` or
`CeDimension`, and the second **must** go to `set_dimension_style` instead.

> ★★ A ce dimension is a `/Line` with `/IT /LineDimension`. It passes every
> "markup pdfcer can author" test, and restyling one through `set_markup_style`
> regenerates it as a **bare line — label and witness lines gone** — from an
> operator who asked only to recolour it. The engine refuses by name; the kind
> on the target is what stops the refusal being reached.

Deliberately **not** in that verb, so the tab must not offer them: note text,
**move and resize**, and `/LE` on PolyLine. Move/resize is the next ask after
this one and needs its own engine request.

### 2. Dimension select-and-drag — a REGRESSION, not an unbuilt feature

The **old** GUI does this: `run_dimension_drag` at
`D:\Dev\pdfce\crates\pdfce-gui\src\main.rs:22782`, with
`doc.selected_dimension`, `doc.dimension_drag`, and `dimension_rects`
hit-tested per page. The new shell never called `dimension_rects` at all.

Selection now covers *clicking* one. **Dragging it is still gone**, and R6 says
nothing regresses. The old code exists and is salvageable; 18 references to
`selected_dimension` in that file are the whole feature.

### 3. One open operator report — the other was found and fixed

**★ FIXED, but NOT DRIVEN: "add text types nothing."** The dialog latched on
having **asked** for focus rather than on holding it, so a request that lost
its opening frame was never retried and the field swallowed every keystroke
while looking exactly like a focused one. Losing that frame is the normal
case, not an edge case: the dialog's first draw is the frame *after* the
gesture that opened it, so the pointer release is still being resolved around
the request, and egui keeps the earlier of two requests in one pass. Fixed at
`1b4949f` with a bounded retry and a regression test that was run **both**
ways — it fails on the old implementation and passes on the new.

It has not been confirmed against the operator's own report. **That is the
one thing outstanding on this item**, and it needs the machine.

Two things this cost, both worth knowing before writing the next test of a
window: `RawInput::default()` has no `screen_rect`, so a dialog that sizes
itself from the screen lays out unlike the application; and it has no `time`,
which egui then fills from the **wall clock**, so a multi-frame test flakes
under load. That flake read as "test interference" and sent me looking for a
polluting sibling test that does not exist. Both are in
`D:/dev/rag/egui/rawinput_default_has_no_screen_rect_and_no_time_...md`.

**No context-sensitive panel, and no tool indicator.** *"When I click to use a
tool I have no indicator to tell me what to do next or what tool I am even
using."* Verified: the status bar carries page, find, fit, zoom and
disclosures, and **nothing names the armed tool**. Read `MODES_AND_PANELS.md`
before designing — the flexible panel system is specified there and this is not
a thing to improvise.

### 4. "Highlight fillable fields" — the smallest real win on this list

Form filling **works**, in every mode including Read: `canvas::forms` never
consults the mode. What is missing is that **nothing shows where the fields
are** — Acrobat tints them blue, pdfcer paints nothing and only changes the
cursor to an I-beam. That is the whole of *"How do I click on a form to edit
it in the Canvas?"*

`canvas/forms.rs`'s own header names it and declines to build it:

> *"…a weaker one than Acrobat's blue field tint. The honest remedy is a
> **"highlight fillable fields" toggle**, which is a ribbon command; this
> module deliberately adds none and the entry point is reported rather than
> wired."*

It is a view overlay like rulers, the grid and find hits — **not** content
marking, so rule 4 permits it. One command, one condition, one overlay pass.

### 5. Resize an EXISTING page

`set_media_boxes(indices, rect)` shipped with `set_media_box` and only the
second is used. Belongs in Document ▸ Properties, and is a **design** question
before a coding one: does content move, does `/CropBox` follow, is shrinking
below the content a refusal.

> Read `archive/2026-08-18-mediabox-and-markup-reply.md` first. `/MediaBox` is
> inheritable (§7.7.3.4), so the write is three-way, and *"a target equal to
> the inherited value REMOVES the page's own entry"* is load-bearing and
> invisible to a one-page fixture — **writing to the ancestor that supplies
> the value resizes every sibling.**

### ~~Not ours: revision clouds~~ — ★★ this heading cost three weeks

**Struck 2026-08-19. It shipped that day, in about an hour.**

It read: *"Confirmed moving upstream on 2026-08-18 — `EditError::TooFewVertices`
and a `Cloud` subtype are in `D:\Dev\pdfcer`'s working tree. The operator:
'don't worry about item 5. It's aware of that one now.'"*

Every word of that was true, and the **heading** was wrong. He meant the
*engine* was aware. This file turned that into *"not ours"*, filed it under a
heading a reader takes as a scheduling decision, and the operator went on asking
for the revision cloud tool while the only thing blocking it — a `MarkupSpec`
variant — sat shipped in a repository one `grep` away.

Kept, struck, because the mis-reading is the finding:

> **"The engine is aware" is not "this is not ours."** An upstream repository
> acknowledging a gap says nothing about whose work the *surface* is, and a
> heading that says otherwise stops anyone re-checking.

And the deeper one, which now governs every blocker in this project: a claim
about a repository you do not build **cannot fail a test**, so it goes on being
read as current until somebody happens to look. `NO_SURFACE.md` §1c.

## What NOT to do

- **Do not start Phase 5 (text editing) early.** It is deliberately last —
  `HANDOFF.md` §8. It is the defect that began this project, and every earlier
  version of that file treated it as the obvious next move. It is not.
- **Do not build S6 deep zoom or tiling.** Measured as a 9× regression.
- **Do not write to `D:\Dev\pdfcer`.** Read-only to this project. Engine work
  goes through `D:\Dev\FeatureRequests\pdfce_FeatureRequests\` and lands there
  as its own Pass. That channel answered seven requests in a day — it works.
- **Do not run `ui-verify` without the operator's go-ahead** if they are using
  the machine. It drives the real cursor and keyboard.

---

## Standing operator instructions set in this session

1. **Always `cargo update -p pdfcer-core -p pdfcer-render -p pdfcer-print` before
   building.** Automated as a build step in `package-portable.py`; `--no-update`
   exists for reproducing an exact revision. The engine repo moved 8, then 12,
   then 4, then 6 commits ahead inside one afternoon, and a stale pin already
   cost eighteen missing images on the operator's own file.
2. ★★ **Publish EVERY build worth keeping to OneDrive**, alternating
   `pdfcer-gui1` / `pdfcer-gui2`, newest replacing the older slot. Restated as a
   standing rule by the operator on 2026-08-19.

   ```bash
   python tools/package-portable.py     # updates the engine, builds, mirrors, rotates
   ```

   The alternation is a property of the tool, not something to track: it picks
   the older slot itself and preserves that slot's `userdata/`, because the
   operator runs the exe straight out of OneDrive on this machine and others.

   The obligation is the part that is not automated. **A build that exists only
   in `target/release/` or `D:uilds\` has not reached the operator.** Run it
   at the end of any session that landed working changes, and immediately after
   any fix he might want to try — without asking, since it writes only to
   `D:uilds\` and the OneDrive slot and never to a repository.

   And **say which slot in the report**, together with which one holds the
   previous build. The slot name carries no version information, so "packaged"
   on its own leaves him opening folders to find out which is which. The reason
   there are two slots at all is the project's own fallback property applied to
   the day-to-day: the previous build stays intact beside the new one, to fall
   back to and to compare against.
3. **Put engine work through the channel**, and the other session picks it up in
   parallel.

### ★ When the engine repo is busy, the packager races itself

`package-portable.py` updates the engine as its **first** step, so on a day the
other session is committing live, every packaging run moves the pin, dirties
the tree, and stamps the artefact `-dirty`. Two consecutive runs on 2026-08-18
produced two dirty builds for a reason that had nothing to do with either of
them — the engine moved 11 commits, then 1, then 1 again, inside twenty
minutes.

The sequence that works, and what `--no-update` is actually for:

```bash
cargo update -p pdfcer-core -p pdfcer-render -p pdfcer-print
git commit Cargo.lock -m "Take the engine to <rev>"
python tools/package-portable.py --no-update --verify --note "…"
```

Standing instruction 1 is still honoured — the engine IS updated immediately
before the build. What is skipped is a **second** update racing the build it is
meant to precede. On a quiet day the default path is fine and this never comes
up.

---

## ★★ What the last two sessions found — the part worth carrying

### ★★★ From 2026-08-18 (latest): fourteen shortcuts had never worked

Found while investigating "add text types nothing", which it does **not**
explain. `app::keyboard::commands` matched the frame's keypress against
`DERIVED`, a hand-written table of eight chord spellings, and refused outright
any chord holding Shift or Alt. The manifest binds twenty-one. So fourteen
bindings were declared in `built_in.ron`, printed in menus and tooltips as
shortcuts, and delivered nothing:

```
Ctrl+Z  undo          Ctrl+S        save a copy      F11   fullscreen
Ctrl+Y  redo          Ctrl+E        edit text        [ ]   rotate
Ctrl+Shift+Z redo     Ctrl+Shift+E  add text         Alt+Up/Down  move page
Ctrl+H  read mode     Ctrl+Shift+C  copy page text   Ctrl+Alt+N   from template
```

**Undo had a keyboard shortcut everywhere except the keyboard.** This is very
likely a large part of *"I click and can't figure out how to enable some of the
basic stuff"* — and it is why the operator has been reaching for the ribbon for
everything.

Three things to carry:

1. **A table kept in step with a manifest by hand falls out of step with it**,
   and the failure is silent from both ends: the entry looks bound, the hint
   looks true, the key does nothing. The table is gone; `parse_chord` reads the
   manifest through `egui::Key::from_name`.
2. **`Modifiers::matches_logically` is permissive** — it asks whether the
   pattern's modifiers are *present*, not whether the extras are *absent*, so
   `Ctrl+Shift+Z` satisfies `Ctrl+Z`. Bound to redo and undo, that makes one
   keypress mean two opposite things with iteration order deciding. Compare the
   three flags exactly. Refusing the extra modifiers outright was the old
   code's answer to the same hazard, and it is what killed `Ctrl+Shift+E`.
3. **★ The meta-lesson, and the one to re-read before writing any gate.** A
   gate *did* exist. Its doc comment stated the general rule — *"a chord this
   module cannot see would then be a keymap entry, a menu hint and a tooltip
   promising something no keypress delivers"* — and its body then said
   `if !is_digit_chord { continue; }`, sweeping seven of twenty-one. `Ctrl+O`
   had already been found dead once and was fixed by adding one row: the
   instance closed, the class left open. **When a gate's prose is general and
   its body has a `continue`, the `continue` is the bug.**

Its replacement presses every chord and asserts the command comes back, which
is a stronger claim than spellability — a spelling test passes on a dispatcher
that spells a chord correctly and then filters it out for holding Shift.

**Not driven.** All four new tests are headless. Pressing Ctrl+Z on a real
document is a thirty-second confirmation and has not been done.

### From 2026-08-18 (earlier the same day): two features, and three drifted claims

**What shipped.** The print dialog grew a **paper list**, the driver's own
**Properties…** button and a restored **tray** checkbox; `file.new_from_template`
grew a **page-size chooser**. Both were engine gaps that had been filed and
answered, and in both cases the shell half was smaller than the reasoning
around it.

**The finding worth carrying is about DISCLOSURES THAT EXPIRE.** Three separate
true-when-written sentences were false by the time they were read:

| the claim | why it expired |
|---|---|
| *"Paper comes from this printer's settings. **pdfcer cannot change it.**"* | shipped copy, correct for months, falsified by the control added three lines above it |
| `app::blank` §3a *"the size picker is BLOCKED on the engine"* | correct on 2026-08-17, unblocked on 2026-08-18 |
| `catalog.rs` *"86 of 101 named, 15 refused"* | the registry held 94, of which 85 named and 9 refused. `86 + 15 = 101` is internally consistent, which is why nobody looked twice |

The third is the instructive one: a test **had** been added after the fourth
drift of that pair, and it did not catch the fifth, because it pins the split
against its own literals and the *sentence* was never one of them. Its failure
message said *"update that sentence together"*. **A test that asks a human to do
the thing they just failed to do is a note, not a gate.** The heading no longer
carries numbers.

The repair is now this project's standing move and it has been taken four
times — the gate runner's header, `README.md`'s test count, this heading, and
the paper sentence. **When prose and a measurement disagree, delete the prose's
copy of the measurement rather than correcting it.** Where the prose must state
a limitation instead, there is no gate that can help; the only defence is
noticing at the site of the change that invalidates it.

**Two design decisions worth not re-deriving.**

- **`NotListed` is not `no`.** `pdfcer-print` declined our proposal to gate the
  tray control on a `bool`, with a measurement: `DC_BINS` on Microsoft Print to
  PDF returns nothing at all, while that same device's `dmDefaultSource` is
  already `DMBIN_FORMSOURCE`. A bool would have hidden a control from a device
  that was doing the thing by default. R83 forbids offering what the hardware
  *cannot* honour; it does not forbid offering what the driver merely declined
  to advertise.
- **A new document is not an edited document.** `file.new_from_template`
  serializes and re-parses rather than handing over the `EditSession` that
  resized the page. Otherwise a brand-new A1 sheet arrives already modified,
  with `Ctrl+Z` waiting to take it back to A4.

**One of my own tests failed on its first run, correctly.** It asserted *"every
paper size's name differs from its uppercased identifier"* — false for A0
through A6, and rightly so. Restated as *"no name contains a hyphen"*, which is
what actually distinguishes a wrong fallback (`ANSI-D`) from a right one (`A0`).
Same family as a test that pins a refusal: an assertion that is *checkable* is
not the same as one that is *true*.

### From the session before: predicates too coarse, and a harness that lied

| reported as | actually |
|---|---|
| *"synthetic keyboard input does not reach the window"* | only **chords** failed. `keybd_event` posts asynchronously and egui drains once per frame, so modifier-down and key-down in the same microsecond deliver an **unmodified** key. Three 12 ms sleeps fixed it |
| *"18 controls laid out outside the window"* | the `ui-rect` trace is a **change log** and could not report that a control stopped being drawn. The ribbon overflow had correctly swallowed them. Fixed at source with `ui-rect-gone` |
| *"selection is not taking the hit test's result"* | six doc-points across a dense sheet all reported `hit 0 objects`. **A hit test that misses everywhere is a gate, not a hit test** — the check had never left Read mode |
| *"three headings illegible"* | three headings **not on screen**. A `ScrollArea` lays out below-the-fold children before clipping them |

Two mistakes kept in the docs because they looked reasonable: seven invented
stamp label strings (`TextAnnotSpec::Stamp` takes ISO Table 181's `StampName`,
so every stamp would have carried `/Name /Draft` whatever it read), and leaving
the UI-scale check's injected preference at 1.8 "on purpose" — next full run,
**20/0/4 → 3/1/21**. The distinction missed was **who owns the state**:
application side-effects stay, harness-injected inputs get restored.

**`tools/gates/check-string-gaps.sh` came from that session and is worth
knowing about before you write operator copy.** Rust's line continuation eats
the newline *and the next line's indentation*; lose the trailing backslash and
the indentation ships. The literal still compiles and still passes every test
that does not compare it to a hand-written expectation. The same grep found
**36 across 22 files, eight of them in copy the operator reads on screen**. It
is invisible in a diff — you see a wrapped sentence and the spaces read as
indentation, which is what your eye is trained to skip. Run the gate; do not
look.

**`--verify` had never worked, for a reason nobody had diagnosed.**
`subprocess.run(["bash", …])` resolves `System32\bash.exe` — **the WSL
launcher** — before Git Bash, which also explains a CRLF symptom filed
separately. One root cause, two unrecognisable symptoms. **A workaround written
against a wrong diagnosis outlives the problem and hides it.**

---

## ★★★ The founding rule, and the day it paid for itself

> **Verify by driving the binary, not by a passing test.**

**2026-08-18, second half: everything below was driven.** The operator handed
over the machine and the harness ran. What it settled is worth reading before
anything else in this file, because two of the three findings **contradict what
a green test suite said an hour earlier**.

### It falsified my own fix

The morning's `dialogs::textannot` focus fix — latch on `has_focus()` rather
than on having asked — has a headless regression test that fails on the old
implementation and passes on the new. It looked like the operator's bug.

`text_annot_takes_the_keyboard_unclicked` was then run against a binary built
with the **old** latch. **It passed.** The dialog took the keyboard all along;
the race the test constructs does not happen in the real frame. The fix stays,
because asking for focus and holding it really are different facts and the
bounded retry costs nothing — but **it is not the explanation**, and anyone
reading the commit for that story should read this paragraph instead.

### It found a defect no test could have

`app::keyboard::commands` compared a per-frame modifier snapshot
(`i.modifiers`) against a per-event fact (`Event::Key`). On a long frame — the
application rasterizing a dense CAD sheet — a quick `Ctrl+Z` arrives with Ctrl
already up and is silently dropped. It presented as *harness flakiness*: a
different pair of chords dead on each run, and reordering the list moved which.

### And it named what nobody was looking at

Nine module headers in `tools/ui-verify` recorded, as a fact about the machine,
that synthetic keyboard input does not reach the target window. It was inferred
from `Ctrl+E` arming nothing — `Ctrl+E` being one of the fourteen chords the
dispatcher never dispatched. Eight of those headers cited `checks::find_bar` as
the source; `find_bar` **passes**, and its own report says *"control chord
Ctrl+2 arrived, so the input channel works"*. The record contradicted itself in
the same run report for months.

**A constraint inferred about the environment is a reading, not a fact** — the
operator's own standing rule, and this is the second time it has cost this
project real work. A reading that stops people testing something is the
expensive kind: while it stood, no check drove a chord; because none did,
nothing contradicted it; and undo had no keyboard for months.

---

## Where a session can still fall short of it

The morning half of 2026-08-18 shipped two features with checks written and
**not run**, because the operator's desktop was in use. That is the normal
state of this project between hand-overs, and it is stated plainly rather than
softened: this project was founded on a commit that said *"analysis-confirmed,
NOT empirically verified"* and was treated as done anyway.

What driving buys, in four trace lines:

```
Markup > Text box armed the text-annotation tool
the page carries 0 annotation(s) before the drag
the release authored nothing — still 0 — and opened the dialog instead
Accept authored: the page went from 0 to 1
```

That middle line is the whole feature. A build where the release authored
directly passes **every** unit test in `canvas::textannot` — the spec builder is
pure and correct either way — and puts an empty box on the operator's drawing
every time they let go of the mouse.

---

## Where to read next

| file | for |
|---|---|
| `HANDOFF.md` | the standing rules, the phase order, the accumulated findings, and §5's six obligations of registering a command |
| `FEATURES.md` | what works today, row by row. The acceptance contract |
| `NO_SURFACE.md` | every hard-coded value with no control — **and the standing warning that a row here is not automatically a build-the-surface task** |
| `DEFECTS.md` | the defects this project exists to fix, with `file:line` |
| `D:\Dev\FeatureRequests\pdfce_FeatureRequests\` | the channel. `open/` empty means nothing is owed; `INDEX.md` is the memory |
| `D:\dev\rag\egui\`, `D:/dev/rag/rust/` | ecosystem findings — read before non-obvious work, write findings back |
