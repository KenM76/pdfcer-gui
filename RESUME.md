# RESUME

The first file a session reads: what this program is, how to measure its state,
what to build next, and the traps that cost a session if they are rediscovered.

## What this is

A replacement GUI for the `pdfcer` PDF engine. `crates/pdfcer-gui` knows about
PDF; `crates/pdfcer-gui-base` holds the modules beneath it that reference
nothing above them, so cargo's no-cycles rule is what keeps them down there;
`crates/egui-shell` carries the ribbon, dock, modes and command registry and
never learns what a PDF is; two platform crates and the driving harness
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
| Engine pin | `grep -m1 -oE 'pdfcer\?branch=main#[0-9a-f]+' Cargo.lock` | `c5a80c3b` — a **branch** pin with no `rev`, so cargo re-resolves it opportunistically and it moves with no `cargo update` on our side. Re-read the lock in the same breath as quoting it; never carry a sha forward from a paragraph written an hour ago. `check-pin-citation.sh` reads this row's third cell and `FEATURES.md`'s first `**Updated:**` line, and fails if either disagrees with the lock |
| Engine HEAD | `git -C /d/Dev/pdfcer log --oneline -1 main` | The question is never whether the two shas MATCH — it is whether CODE has landed since the pin, because only that can falsify a sentence beginning *"the engine cannot"*. `git -C /d/Dev/pdfcer diff --stat <pin>..main -- '*.rs'` is the test; empty means such a sentence may be written. Read this log in the same breath as listing `open/`: a delivery has arrived here as a commit before it arrived as a reply three times |
| Engine version | `grep -A1 'name = "pdfcer-core"' Cargo.lock` | — |
| Last release | `git fetch --tags origin && gh api repos/KenM76/pdfcer-gui/releases/latest` | **Fetch first.** `gh release create` tags on the REMOTE, so `git describe` in an unfetched tree answers with an older tag and reports a commits-unreleased count wrong by a factor. Read every field back out of the API rather than inferring it from the flags passed in, and check the local zip's byte count against the asset's — agreement to the unit is the cheapest proof the upload is the file and not a truncation. The binary's own stamp and `published_at` sit twelve to twenty minutes apart on every release; label which clock |
| Which slot he runs | `grep -H '^Shell:' /c/Users/Ken/OneDrive/pdfcer-gui{1,2}/BUILD-INFO.txt` | **The slot name carries no version and the rotation alternates**, so the newer slot is 1 on one release and 2 on the next. Never carry "the slot he runs" forward in prose — a paragraph about what a past release contained must say *first published in* and stop, because a sentence naming the live slot is true for one release and silently wrong for every one after. The older slot is the fallback by construction: `package-portable.py` replaces the older, so the previous build always survives |
| Driven checks | `ui-verify --list \| grep -cE '^  [a-z0-9_]+$'` | **Rebuild the harness first** — `--list` sits behind `refuse_if_self_is_stale`, and a stale harness prints nothing, which `grep -c` reports as zero and reads as a roster. The `$` is load-bearing: `--list` also prints a profiles table whose rows are two-space-indented lowercase words. It prints two lines per check, so a `wc -l` answers about double |
| Gates | `bash tools/gates/run-all.sh` | Three states, not two: `0` pass, `1` fail, `3` a gate was SKIPPED for an absent precondition. A skip is not a pass. **Tally on the `RESULT:` line, never on a pipeline's exit code** — `run-all.sh \| tail` reports `tail`'s zero. **A background task is the worst host for that**, because the completion notification restates the pipe's status in the harness's own voice, beside the word *completed*, and a bare control (`exit 1` in the background) reports `1` correctly — so there is no wrapper bug to find and the only remaining explanation looks like a defect in `run-all.sh`, which exits `1`/`3`/`0` correctly. The tell is the output FILE: a 95-minute run that left 47 lines had a filter eat the detail and the status with it. Launch it `> "$OUT" 2>&1` and the notification reports the runner's own status. A stable pass-count across an engine pin bump proves nothing about whether the documents survived it; the tally moves only when a gate is added. **Run `cargo fmt --all` before starting one** — `cargo fmt` is the LAST gate, so a formatting slip in hand-written Rust is found forty minutes in and costs the entire run. **And start it `CARGO_BUILD_JOBS=2`**: clippy runs after fmt, `--all-targets` spawns a `clippy-driver` per TARGET, and by then the sweep has spent the session's handles on one `sed` per source file — several drivers then die together with `exit code: 0xc0000142, STATUS_DLL_INIT_FAILED` and no diagnostic at all, which reads as four crates breaking at once. It is process count, not memory, whatever a supervisor's kill reason says; the same command alone finishes green in seconds. A sweep killed before its SUMMARY has no verdict, and hand-summing the PASS lines cannot see a gate that never ran |
| Unit tests | `cargo test --workspace` | Summed over the `test result:` lines, then cross-counted by `cargo test --workspace -- --list \| grep -cE ': test$'`; passing plus ignored must equal the cross-count. Two methods, because a summed figure nobody cross-checks is how the last count drift got in. The cross-count is also how you know a new test RAN: three `#[test]` functions once compiled clean, reported nothing and executed zero times, because they were nested inside another function |
| Source files | `find crates -name '*.rs' \| wc -l` | **The command's scope is a claim.** `find crates tools` answers larger, because `tools/ui-verify` is a Rust crate and is not under `crates/`. Neither figure is wrong; quoting one under the other's command is |
| Backlog register | `python tools/walk-engine-backlog.py` | Rewrite the five headings from the walker's own printed figures, never by arithmetic on the old ones. A row is a verdict plus one paragraph, capped at 1,200 characters and checked by `--check`; file the row and set the headings in one edit with `--write YYYY-MM-DD`. **`--check` prints "the headings do not agree with the walk" on ANY non-zero exit, including a pure over-cap failure** — read the section above the banner before re-writing headings that are already right |
| Backlog verdicts | `python tools/check-backlog-verdict-drift.py` | The walker counts a row by the section it SITS IN; this one asks whether an absence row is contradicted by `crates/pdfcer-gui/src`. Registered in `run-all.sh`, so it is normally green already — run it by hand after moving rows. It reads only the FIRST cell, so a symbol that shipped must leave that cell even when the row's narrowed remainder still cites it in the body. An exemption is per SYMBOL, `<!--namesake:IDENT-->`, never per row |
| Request channel | `ls /d/Dev/FeatureRequests/pdfce_FeatureRequests/open \| wc -l` | The invariant is *a session lists `open/` and nothing else, and empty means nothing is owed* — so a closed exchange left there mis-states the outstanding work by an order of magnitude. Closing is part of doing the work: archive both files AND write the `INDEX.md` row in the same sitting as the `done_*`. A move without a row is not a close, it is a deletion that leaves a file behind. **Nothing in that folder is in a git repository, so no gate can ever see this** — which is also why a sentence asserting an identifier's state (*can be closed*, *still open*, *we filed*) is grepped out of `INDEX.md` and `archive/` before it is written, never taken off the engine row that cites it: a row citing `E001` as filed outlived our own `done_` for it by three days |
| Registered commands | `grep -rn --include='*.rs' -E 'command\(' crates/pdfcer-gui/src/shell/commands/catalog/ \| grep -vE ':\s*(///\|//)' \| wc -l` | The obvious command is wrong, not merely its answer: a raw `grep -rhoE` over that directory counts one extra, a line of prose in `catalog/file.rs` that quotes the very pattern being searched for. The build's own `pdfcer-diag shell commands=` trace (`app/mod.rs:925`) is the tie-breaker |
| Dockable panels | `grep -n 'pub const ALL' crates/pdfcer-gui/src/panels/mod.rs` | — |

## Do next, in the operator's order

Ordered by what the operator reaches for, never by what arrived in the engine
channel: a reply is an input to *how* a thing is built, never to *which*. Each
row's argument is in `OPERATOR_REQUESTS.md`, which **only Ken closes**; the open
set is `grep '^## O' OPERATOR_REQUESTS.md`.

1. **O218 – O225, his eight-row zoom and text report. Five are built and not
   yet driven, two are observability only, one is unstarted and its first
   question belongs to the engine.**
   **Built, driven checks owed.** O218 — every ceiling the shell derived was
   `RenderQuality::multiplier` too high, so it offered a zoom the engine then
   refused; `viewer::raster_density` is now the single factor and
   `zoom_for_raster_scale` its named inverse. O220 — the refusal placeholder
   swallowed Ctrl+wheel, so the only way out was the bottom bar. O222/O223 — a
   text draft settles when the armed tool is no longer its caret, stated once
   in the frame order rather than at each of eight exits; Escape commits on the
   canvas caret, on the note window, and on the Comments panel's note and reply
   editors, and a dialog's **first** Escape leaves the field so that one press
   cannot reach Cancel. ⚠ `Context::text_edit_focused()` cannot guard Escape at
   all — egui clears focus in `Focus::begin_pass`, before any widget runs and
   without touching the buffer — so a guard asking it about Escape is INERT,
   not weak, and the dialog carries a previous-pass flag instead. O225 — page
   previews default to no limit.
   **O219 / O221 are OBSERVABILITY ONLY, and the distinction is the whole
   status.** `crates/native-gl` drains the GL error flag behind a safe
   signature; `render::pressure` decides what a reading can be pinned on, by
   elimination, and traces `gl-pressure` when the flag is dirty and
   `gl-max-texture-side` on change. **Nothing lowers a ceiling and nothing
   drops a raster.** The drain is the first statement of the frame because
   `load_texture` only queues — an upload ordered in a frame is performed after
   `ui` returns, so its error is first readable at the top of the next one.
   **What is owed before either row may be called repaired** is a zoom
   **series** — not two endpoints — on a fixed page with one, then three, then
   six documents open, driven through the real binary. A blank canvas has
   exactly one oracle, a captured screenshot. `ui-verify` competes for his
   machine: ask in one line with the cost.
   ⚠ **Two things must never be reported as the fix for his symptom.** The
   per-axis edge guard is **inert on his card** — measured, release build,
   off-screen: `gl-max-texture-side side=16384` against a whole-page edge
   budget of 16383. And anything measured in a **debug** build reads clean,
   because `egui_glow`'s own check runs first and clears the flag; the
   instrument is live only in release, which is the build he runs.
   **A second window is a second PROCESS.** Nothing opens a document in another
   OS window and nothing launches a second copy of the program, so the two
   share one GPU and a reclaim inside one process cannot see the other's
   textures. The clamp covers the case he described; the reclaim narrows it.
   **O224 is the engine's, filed as `G033`, and nothing here is owed until it
   lands.** The SVG and EMF writers consume `display_list::Op`, whose variants
   are `Fill`, `Stroke` and `Layer` — `interpret`'s `paint_glyph` has outlined
   the glyph before any writer sees the page, and the enum is `pub(crate)`, so
   the gap is unreachable from here. DXF is the exception because it walks
   `vector::PageObjects` instead, which is why his *"formats that support it"*
   is a fair expectation rather than a wish. The option is not drawn at all
   until the engine can honour it (R9), and both export reports already lead
   with the outline fact.
   All eight rows stay **FILED**. Only the operator closes one.

2. **O215 / O216 / O217 — the text chunk as a thing the operator can aim at.
   Five of O215's six asks are built and driven; the sixth is the engine's and
   is filed as `G032`. O216's two are built and driven. All four of O217's
   asks are built and driven; its row stays FILED because its closing
   criterion is a driven check on a table row of `SW41177.pdf`, which is
   blocked behind `G032`.**
   **What shipped.** Clicking a block of text draws a hairline box round every
   chunk inside it, off a toggle — View ▸ Navigate ▸ *Text chunks*, and the
   same row on the rail, live in Content Edit, persisted (ask 3). With the
   boxes up, a second plain left-click inside one of them stands on that chunk,
   every click after it re-picks the chunk under the pointer, and a **press**
   on a neighbouring chunk moves to that chunk instead of resetting to the
   whole block (asks 1 and 6). The press is the half the report was about:
   `canvas::presspick::covers` asks, on press, whether the pointer is inside
   the *current* selection's outline, so at the chunk rung a press a few points
   off the held chunk fell through to `select_only` and the drag carried the
   block. *"Sometimes it moves the chunk and sometimes it moves the entire
   block"* is what aiming at an invisible target feels like, and both halves
   are load-bearing: the click path narrows, the press path stays narrowed.
   Narrowing is gated on `canvas::chunks::boxed`, so the rung is never offered
   where no box says the unit exists.
   **Driven, and falsified.** `chunk_boxes_show_what_a_text_block_is_made_of`
   clicks `paragraph.pdf`, asserts six boxes for six chunks, presses the toggle
   and asserts `canvas-chunks-declined reason=switched-off`, then presses it
   back. Its oracle is three subsystems deep on purpose: a missing ribbon item
   writes none of the three lines, a missing dispatch arm writes the first
   only, and a build reading the persisted preference instead of the live one
   writes the first two — a defect otherwise invisible until the next restart.
   `clicking_a_chunk_selects_that_chunk` clicks one point twice —
   `level=Object`, then `level=Part` — then a neighbouring line and back,
   asserting the chunk index changes and then returns; the indices are
   compared, never pinned, so a change in provider granularity does not read as
   a selection defect. Plants: a loop that draws nothing turns the first red at
   `drawn=0`, and disabling the press path's entered-object branch turns the
   second red on the return click.
   **Ask 4's modifier half shipped.** Shift — or Ctrl, one `||` upstream in
   `canvas::interact` — adds the chunk under the pointer and takes a held one
   back out. `canvas::moving::eligible` forks on `selected_parts_on(...).len()`
   into `MoveSubject::TextLines`: one `EditSession` call, one undo entry,
   however many chunks. The press half is the one with teeth —
   `Grabbable::bounds` at the chunk rung is the UNION of the held outlines, so
   a set of lines 0 and 2 spans line 1, and a `covers` asking only about the
   object claims a press in the gap: he aims at a label, drags, and two OTHER
   labels move. `pressing::body_under` narrows to the chunk under the point.
   `shift_click_builds_a_chunk_set_the_whole_program_honours` drives it and is
   falsified by three plants, one per half.
   **Ask 4's band half shipped too.** `marquee::take_chunks` forks ahead of
   the object band, so a band dragged while a chunk is entered sweeps lines
   rather than ascending to the block; `chunks::within` measures each line with
   the same call that draws its box, the direction rule is unchanged — right to
   left crosses — and the three arms are the click's own `Combine`. Four
   preconditions and an empty replacing band all fall THROUGH to the object
   band rather than doing nothing, so sweeping blank paper still means what it
   means everywhere else. `a_rubber_band_inside_a_note_takes_its_lines` drives
   three bands, the status row, one plural move and one undo, and is falsified
   four ways — one per arm it can lose.
   Ask 5 is met: dragging a line of a note moves one outline per held chunk
   AND a translucent copy of the line's own pixels, both following the pointer.
   The gate that withheld the outline read *is this an inner rung* where it
   meant *is the real geometry already travelling*; the copy is withheld on
   that second condition alone, and it asks whether the shape preview CONTAINS
   anything rather than whether it exists — `for_move_subject` answers `None`
   for every text-line subject, so `is_none()` and the right spelling agree on
   this fixture and a driven row cannot tell them apart.
   `dragging_a_chunk_shows_where_it_is_going` drives the singular and the
   band-built plural arm, reading `boxes=3` and `drawn=3 clipped=0 reason=none`
   on one gesture. Falsified three ways: the restored rung-keyed gate
   (`boxes=0 rung=part suppressed=o63`), the deleted `draw_raster_ghost` call
   site, and a `.take(1)` that blits only the first held chunk.
   ⚠ That third plant reddens through the ABSENCE arm, not the count arm:
   `diag::trace_changed` emits only on a change, so a plural arm that draws the
   same 1 the singular arm drew is SILENT. The absence message names both
   causes for that reason.
   First published in `v0.5.0-dev.20260920.1`.
   **The preview is asserted in pixels as well as in the trace.**
   `Driver::drag_observed` holds a drag open: it presses, walks to the
   destination, rests there with one-pixel nudges so a frame runs with the
   pointer where it will be photographed, and hands control to an observer
   with the button still down. The release is in a guard’s `Drop`, so an
   observer that fails — or panics — cannot leave the physical button pressed
   with the operator’s desktop left rubber-banding.
   `the_travelling_copy_is_on_the_glass` photographs the gesture and asks five
   questions in the order they have to be asked: the destination is blank
   before the press, it carries ink during the hold, the SOURCE reads the same
   in both frames, the copy reads lighter than the line it copies, and its ink
   coverage is comparable. `pixels::mean_luminance` is the new oracle;
   `contrast_at` and `ink_run_into` both quantise, so a tint small enough to
   leave every pixel inside its own bucket moves neither of them.
   ⚠ The third question is the R8b one and it must come before the fourth.
   A build that washes the un-moved line while a move is in flight fails the
   fourth as well, and the fourth names the opposite defect — *the copy looks
   like content*, where the truth is *the content was made to look like a copy*.
   It measures at zoom 2 for arithmetic reasons: one 8.4 pt line is six device
   pixels tall at fit zoom, and nothing survives a 3 px inset. Every region is
   clipped to `canvas-viewport` first, because a 267 pt line at that zoom runs
   off the canvas into the dock beside it and `logical_to_capture_pixels`
   clamps to the WINDOW, not the canvas — dock grey then reads as content.
   **Falsified three ways, and one of the three is a measured no-op.**
   `RASTER_GHOST_ALPHA` at `255` is the plant that justifies the row existing:
   `dragging_a_chunk_shows_where_it_is_going` stays GREEN on that build — the
   painter still decides to blit, and still says so — and this one goes red.
   Deleting the call site reddens both, so it separates nothing. Taking the UV
   against the page rect rather than `paint_rect` changes nothing at all:
   `paint_rect` equals the page rect at every zoom below the pixmap ceiling,
   so the ink-coverage question is **unfalsified**, not merely weak, and
   exercising it needs this gesture driven at the region tier.
   **What is next, in order.** The size cap: eight files sit within fifty lines
   of it, so the seam-finding pass is a batch rather than a one-off.
   The seam for each is already argued in `DESIGNS.md` under *Where the seam
   is in each file now crowding the size limit* — including the two that have
   none — so that pass is a patch, not a re-derivation. `canvas/keys/tests.rs`
   and `ribbon/width_tests.rs` are the same split twice and belong in one
   commit.
   **O215's ask 2 is the engine's and is filed as `G032`**: `runs_share_a_line` never
   reads the horizontal translation, so a table row welds into one line — 565
   welded lines on his own drawing, the widest spanning 709.4 pt of blank paper.
   **Not worked around**, deliberately: a post-split on bounds would make the
   shell's unit differ from `delete_text_run`'s with neither side able to
   detect it.
   **O216 ask 1 is BUILT and DRIVEN.** Emptying a chunk commits the emptying
   and undo is the recovery, which is what Acrobat does. It was cause 1 of
   the four: `canvas::textedit::commit_into`'s `Anchor::Run` arm carried
   `&& !draft.text.is_empty()`, so an emptied chunk raised no action at all —
   no plan, no engine call, no refusal, no sentence. The guard now sits on
   `Origin` and `Box` alone, where an Add caret with nothing typed really is
   a caret. `emptying_a_chunk_commits_the_emptying` drives his own gesture on
   his own drawing; ★ every keystroke in it is calibrated from the draft's
   own `text-select` line, because an absent commit has five causes and four
   of them are the harness.
   **O216 ask 2 is BUILT and DRIVEN, and it was the ROUTE that needed
   proving.** `DeleteTextLine` was already wired and already driven — through
   the **Points** tool, which he has never been told about. A verb and the
   door to it fail separately, so the verb's row can be green while the ask
   stays unmet. `deleting_a_clicked_chunk_leaves_the_rest_of_the_block_alone`
   is the fourth rung of `deeper_rung_delete` and drives the door he has:
   boxes on, click the block, click the box, Delete. Same fixture, same line,
   same verb as the Points-tool rung, so any difference between them is the
   route and nothing else. ★ A ladder still at `Object` is a **FAILURE** in
   that rung and a SKIP in the other three — its point is pinned and its
   boxes were switched on, so a second click that does not reach a chunk is
   the unmet ask, not an arrangement the harness could not make. Falsified
   twice: `ClickHit::chunk` shut reddens the door, and a Part rung reaching
   no verb reddens on the application's own decline line.
   **O217's fourth bullet is BUILT and DRIVEN.**
   The apply report now lists the text the marks will destroy, one quoted
   line per region in the order it was drawn, deduped so identical removals
   appear once. That is the disclosure half of the row: redaction on a welded
   table row still takes the part number and the description with the
   quantity, but he now reads those words above the commit button instead of
   finding out after the file is written. Every other figure in that report
   is a count, and a count cannot be checked against an intention.
   `the_apply_report_lists_the_text_it_will_destroy` generates a page whose
   only content is a string the harness wrote itself, marks the whole page,
   opens the report, and fails four ways: no block, the wrong branch, fewer
   characters listed than the page carries, or a block that never published a
   rectangle. Falsified twice on a release build — the deleted call site, and
   a zeroed character count, which reddens the content arm alone.
   ★ The trace carries `state`, `entries`, `chars` and `lines` and never a
   character of the document: `PDFCER_DIAG` is redirected into files, and a
   redaction surface that copied confidential strings into a second file
   would have undone its own job. That is also the harness's limit — it
   proves the block exists, was laid out, found text and listed at least as
   many characters as the page carries; it cannot prove the strings read
   correctly.
   **O217's first two bullets are BUILT and DRIVEN, and the gap was the
   ROUTE.** The unit was already right everywhere redaction reads it:
   `redactsel::mark_selection` builds its quads from the selection's own
   outlines, and `SelectionState::outline_rect` answers `part_bounds` for any
   entry carrying a subpath — so a selection standing at the Part rung has
   always marked that chunk's box. What did not exist was a way to reach the
   verb from the chunk. `edit.redact_selection` lived on the Edit ribbon tab
   alone, which is the defect **O53** names, and the operator's hand is on the
   line he just clicked. It is now a row in the canvas object menu, between
   *Unshare form* and *Delete* — a marking verb, not an editing one, so
   `DESIGNS.md` §6.2's context-menu ban does not reach it, and
   `canvas.object` is structurally Edit-only (`reading = !caps.edit_content`)
   so the row needs no mode predicate of its own.
   `redacting_a_clicked_chunk_marks_only_that_chunk` marks the whole block and
   then one line of it **in one launch** and compares the two bounds: 91.1 pt
   against 11.1 pt on `paragraph.pdf`, 12%, with an undo between them so both
   gestures aim at the same document. Both marks go through the context menu,
   so the route is proved by use. Falsified by deleting the `part_bounds` arm
   from `outline_rect` — both bounds become 91.1 pt and the row reddens at
   100%.
   **O217's third bullet is BUILT and DRIVEN — a gesture over a SET of lines
   is ONE mark holding one region per line.**
   `marking_two_chunks_makes_one_mark_and_two_regions` marks chunks 0 and 2 of
   the same six-line block — two apart, so chunk 1 is left unselected between
   them and a union cannot hide behind adjacency — after taking a singular
   mark as the control in the same launch. Measured: one line 11.1 pt, the
   pair's union 43.1 pt, 3.9×, and one `Ctrl+Z` takes the whole mark back.
   ★★ **Its oracle is two fields pulling opposite ways, and neither is an
   oracle alone.** The panel's census must rise by exactly one — a build
   writing one annotation per line reports the right region count and costs an
   undo per line — and the verb must report two regions — a build that
   unioned the pair reports the right census and destroys the line between
   them. Falsified twice, separately: folding the outlines into an enclosing
   rectangle reddens the region half while the census half stays green, and
   issuing one `add_redaction` per outline reddens the census half while the
   region half stays green. Each plant leaving the other assertion satisfied
   is the proof the second field is a second oracle and not a restatement.
   ⚠ The trace publishes the **union**, not each quad, so the check proves
   there are two regions and what they span together, never that each is its
   own line's box. That proof is the apply report, region by region.
   The menu gesture both redaction checks press lives in
   `checks/redact_menu.rs` — one copy, because a second would be a second
   place for the route to drift with both copies still passing.
   ★ The check exists in that shape because `quads=` alone is **an assertion
   both outcomes satisfy**: `quads=1` is written for a chunk-sized mark and a
   block-sized one alike. The trace carries `bbox=llx,lly,urx,ury` for that
   reason, which is geometry and never text — the same R8b rule the apply
   report holds. No number is pinned; a fixture whose leading moved would move
   both bounds together.
   **Bullet three is built and not driven.** A shift-click or band set marks
   one `/Redact` with one quad per held chunk, because the verb consumes every
   outline in the selection. No check drives the plural case.
   **The row still does not close.** Its closing criterion is a chunk of a
   table row on `SW41177.pdf`, and there is no separate chunk to aim at there
   until `G032` lands.
   First published in `v0.5.0-dev.20260920.2`.


3. **O213 / O214 — the shell addresses the visual line everywhere, driven and
   green, and it has not yet been put to him.**
   **What shipped.** The Part rung, the object tree row (`Line #49`), the
   status sentence (*1 line of 144*), the context menu, the selection outline,
   the delete verb and the move verb all count and address lines. Run indices
   survive only inside the `MoveTextLine` and `DeleteTextLine` action arms,
   which translate at the call site; nothing else in the program holds one.
   **What it is worth on his sheet.** `SW41177.pdf` page 0, text object 5871 is
   **237 show operators forming 144 lines**, and `#2 USE SPACERS 8 9 10 11 IF
   REQUIRED.` is nine of them — runs 50..59, baseline 927.23, every one
   `Explicit`. Before the re-key a click landed on run 53, a 19 pt fragment
   mid-phrase; now it lands on the line and a drag carries all nine.
   **Driven, all three checks, 2026-09-19 at `348e1e1`.** `move_line_of_text`
   presses once per visual line and got four *different* answers from four aims
   in one launch; `deeper_rung_delete` took a label out and left the other five
   (`text-lines` 6 → 5, page objects unchanged); `run_menu_route` right-clicked
   the third of six lines and the menu offered `line:2/6`, the line under the
   pointer. **What is owed is Ken's word, not another measurement** — O214 stays
   open until he drags a line of his own title block. O188's history is why the
   grep-the-exe pre-flight was never allowed to stand in for this: every token
   was in the binary while the capability addressed the wrong unit.
   **O213's own oracle is driven on his own sheet and it holds.**
   `edit-text-left-edge page= run= committed= runs=n/m before= after= moved=`
   is raised by `app::actions::textcommit` either side of the funnel, and step 9
   of `text_edit_on_a_real_drawing` bounds `moved` at 0.5 pt — the same bound
   `canvas::textedit::glyphwall` holds a synthesised document to. At
   `--doc-point 0,549.0,935.5` on a copy of `SW41177.pdf` the caret lands on run
   100, his `#2 USE SPACERS` line, `narrowed=1` fires, and the left edge reads
   474.160 before and after. Falsified by planting a 12 pt shift in the second
   reading and driving the rebuilt binary: red, naming O213.
   ★ **The follower count is NOT this number.** Step 8's
   `followers_repositioned` counts operators *after* the edited one; it reported
   0 on the commit that moved his line +240.16 pt, which is how the defect
   reached him through a check already watching reflow scope.
   **The narrowing landed after the release he had**, which is why he saw the
   defect and this tree did not. First published in `v0.5.0-dev.20260919.1`;
   both rows now wait on his word, not on a measurement.
   **The grouping is the engine's own**, not a second opinion:
   `pdfcer_core::vector::edit::text_object_split_points(obj, SplitGranularity::Line)`
   is called rather than re-derived from baselines, so the box that is drawn and
   the thing that moves cannot disagree.
   **The move needs no split and adds nothing to the file.** `plan_move_text_run`
   compensates the run after the one it moves, and nothing renumbers, so N
   sequential `move_text_run` calls over a line's run range compose — measured on
   his file as dx +0.00 dy -30.00 on all nine with runs 49 and 59 untouched —
   folded into one undo entry by `coalesce_last`, disclosing off-canvas when that
   fold is declined. Delete is the same loop run **descending**, because
   `plan_delete_text_run` removes only the show operator's own bytes.
   **One case is declined**: a line holding an interior `Inherited` fragment,
   where the per-run guard cannot tell *"the next fragment is also moving"* from
   *"the next fragment is staying"*. Filed as **G030**. Declining the whole line
   and naming the reason off-canvas is the honest answer until a set-taking
   `move_text_runs` lands; the split-and-transform route is deliberately NOT
   built, because on a Word or Chrome export every line after the first inherits,
   so it would cut his document's paragraphs into pieces on each nudge. On CAD
   output every fragment is `Explicit`, so he never reaches the decline.
   **What a cold session should not re-derive.** An `Inherited` run starts where
   its predecessor's pen stopped, so for horizontal text it shares that baseline
   and is always in its predecessor's line group — which makes
   `WouldMoveNextRun` and `NoPositionOfItsOwn` **unreachable at line level**
   except for rotated text, where the advance changes `f`. That is why
   `fixtures/inherited-runs.pdf` carries a rotated pair: five show operators in
   four lines, giving all four answers in one document. Its two rotated lines
   meet at y = 428.008 with no gap, so the two aims that press at them are
   pinned by `provider::line::tests::the_aims_driven_at_this_fixture_land_one_per_line`,
   which asserts each point is inside **one** box — containment alone would be
   satisfied by a point in both.
   **The Object rung is a separate cost and is not this.** `move_objects`
   refuses every text object while `transform_objects` accepts them, so every
   whole-text-object nudge pays a `q`/`cm`/`Q` wrapper. Filed as **G029**; it
   does not block him.
4. **O209 — the form fields did not look or act like Acrobat's; six of seven
   are driven, the seventh is built and awaiting the screen, and a sweep is
   owed.** Seven complaints in one paragraph and they were seven different
   defects: a pick the popup forgot between press and release, a list frame
   sized as a maximum and dressed as an application menu, `/Ff` bit 18 never
   reaching the canvas so every `/Ch` drew as a drop-down, Edit mode returning
   before the form shading so widgets were invisible in the one mode that has
   to find them, a resize ghost that knew annotations and outlines but not
   widgets, and resize arithmetic that assumed a grip's pivot was a corner so
   the four edge grips collapsed the rectangle. All six fixed and driven; the
   arguments and causes are in `OPERATOR_REQUESTS.md` O209.
   **The seventh is the editable combo (`/Ff` bit 19) and it is BUILT, NOT
   DRIVEN** — `canvas/forms/choosing/typing.rs`, a live text box with a chevron
   drop button, unit-tested, both new assertions falsified, gates green, and
   R1-incomplete because driving needs the pointer and the screen. That drive
   is the first thing to do on a day he is away.
   **Then the sweep he reserved**: *"I haven't checked the other form options
   for completeness"* — TX, CB, RB, PB and SG compared against Acrobat the way
   `/Ch` was. Use `tools/acrobat-form-study.ps1`, and read its header first:
   a synthesized click on a popup photographs a **white slab**, the keyboard
   opens the same popup photographably, and `SetForegroundWindow` must be
   called before **every** click or the page half-paints. The general finding
   is in `C:\personal_rag\pdf\`.
   **What a cold session should not re-derive.** The `/Tx` editor moved out of
   `forms.rs` into `canvas/forms/textbox.rs` so the editable combo and the text
   field state `/Q` quadding and `/MK /BG` tinting once rather than twice — a
   second caller for it now exists, so do not fold it back. And
   `check-ui-strings.sh` is line-based: a whitespace-bearing literal is flagged
   only when its opening and closing quotes are on ONE physical line, so an
   `#[expect(..., reason = "...")]` fails un-continued and passes
   backslash-continued. Exempt it with a `// ui-text-exempt:` block inside the
   attribute rather than re-wrapping to dodge the scanner; the gate's own
   header now says so. Two more of its edges, both easy to misread: a
   `// ui-text-exempt:` comment covers exactly **one** following non-comment
   line, so placing it above `out.push_str(if on {` exempts that line and not
   the literals two lines below it — put the comment inside the call; and a
   `#[cfg(test)] mod x;` declaration causes `x.rs` / `x/mod.rs` to be dropped
   from the scan **whole**, which is why a fixture living in a sibling file
   needs no exemptions of its own.
5. **O208 is built, driven and falsified — what is left is his word.** Both
   clauses are built: the page drags on the
   sheet, the four shortcuts and reset-all are there, the offset is typed or
   nudged, the frame and sign are stated, the per-edge overhang is disclosed
   off-canvas, and the crop hatch is four bands instead of two.
   **Both its checks are now driven GREEN and both were falsified.**
   `the_printed_page_can_be_moved_on_the_paper`: drag `-303.84,-303.84` against
   `303.87` predicted at scale 0.3949, hatch `.r.b` to `lrtb`, `centre-h` alone
   to `-804.37,0.00 edges=lr.b`, `centre-v` composing to Centre, and both reset
   routes returning to the engine's own placement. Three planted defects each
   gave FAIL with the right sentence: a no-op `centre-h`, a `centre-h` wired to
   the other axis, and a doubled drag delta.
   `print_clip_claim_follows_the_preview`: `overhang=blank-band claim=none:0`
   against a geometric count of 1, and a planted verdict cache that never
   matches gave `claim=geometric:1` and FAIL.

   **What it took, and what a cold session should not re-derive.** Two harness
   defects with no symptom: the print dialog is its own **child OS viewport**,
   so every rectangle it publishes carries `viewport=` and must be converted
   through `frame_of` rather than `session.frame()` — converting through the
   main window put the clicks hundreds of points off the control, silently,
   because the numbers stayed plausible. And the drag was sized at 240 pt
   against a preview canvas that measures **340 x 438 logical points**, so its
   own room guard could never pass. Then two defects in the assertions
   themselves: a `THRESHOLD_PT = 45.0` that was invented prose — egui's real
   click-versus-drag distance is `Options::max_click_dist = 6.0` logical points,
   and since the driver steps 15 pt the whole travel arrives, so the prediction
   is `DRAG_PT / scale` and nothing is swallowed — and `centre-h` / `centre-v`
   pressed only from an already-centred page, where a no-op button and a
   wrong-axis button both pass.

   **The clip-claim check needed a FIXTURE, not a flag** —
   `fixtures/blank-overhang.pdf`, 1,000 x 800 pt with every mark 95 pt clear of
   the crop line on either orientation of Letter paper and no page-wide
   background rectangle, which would be ink under `INK_MAX_LEVEL` and would
   make every band report `losing`. On the shared `a1-titleblock.pdf` it
   SKIPPED in every sweep since it was written, so `sweep-full.sh` now names it
   in the `ALONE` table.

   **The controls live on a Position tab** of the print dialog, not at the foot
   of the scale tab, so the check presses `print.tab.position` after the scale
   radio and scrolls over that button rather than over `print.scale.fit` — the
   scale radios are on a different tab and are not drawn at all once Position
   is open.

   **What remains on O208 is Ken's word.** The row stays FILED until he closes
   it.
6. **The form tools, a unit typed beside a number, and the rule behind both —
   O205, O206, O207.** The table he asked for is written: `FORMS_PARITY.md`,
   Acrobat against the engine at the pin against this shell, per kind x
   capability, every row cited. **Work from its section 8.1 in its own order.**
   **Row 1 is built** — `panels/properties/choiceopts.rs` draws all seven
   `/Opt` operations at the end of the `FieldType::Choice` branch, plus the
   default choice and the three `/Ff` bits Acrobat groups with them. Unit-tested
   and gate-green. **Row 1 is now also DRIVEN, and is done** —
   `the_option_arrows_are_greyed_only_at_the_ends_of_the_list` asserts all six
   arrow states across three rows of `ComboOne` in `fixtures/all-field-kinds.pdf`,
   and was falsified twice: a planted live top-row arrow gave FAIL, a broken
   selection seam gave SKIP rather than green. Two general things landed to make
   it possible. `diag::ui_control` publishes a named control's `enabled=` beside
   its rect, because **a correctly greyed control and a broken one are one
   observation to a driven check** — it takes the `&Response` so the pair
   cannot be half-emitted, and it emits the state even when the rect is clipped.
   And `PDFCER_DIAG_SELECT_FIELD` selects a form field by name, which is the
   first headless route to the Properties pane at all: before it, the largest
   editing surface in the shell was R1-unreachable on any day he is at his
   machine, which is most days. **Every future properties check is unblocked by
   that seam, not only this one.**
   **Row 2 is built and DRIVEN, and is done** — `canvas/forms/choosing.rs`
   answers a combo or list box on the sheet, and
   `a_drop_down_can_be_answered_on_the_page` drives it. Two things a cold
   session should not re-derive from it. A page-anchored popup **chooses its
   side before its constraint rectangle**, because one constrained to the
   viewport slides back over its own anchor and then takes that anchor's
   clicks; the general finding is in `D:/dev/rag/egui/`. And egui refuses to
   store a focus-lock filter until the widget has held focus for a **full
   frame**, so `canvas::forms::keyboard_box` calls `set_focus_lock_filter`
   unconditionally every frame rather than on the frame that requests focus -
   the failure is SILENT, the field simply stops answering keys, and it hit
   the arrow keys and Escape on every on-page field editor, not only this one.
   Building it also placed **more fields on the page**: `boxes::place` now
   finds the sheet by walking each page's `/Annots` rather than by reading the
   widget's own `/P`, which `pdfcer-core` reports absent when a producer wrote
   it directly, so two `NotOnCanvas` reasons collapsed into one and fields that
   were panel-only appear where they are drawn.
   **Row 27 is a defect measuring row 2 found, and it is fixed** — a
   multi-select stack copied `/V` and edited it, so a stored value matching no
   option rode along into `set_choice_value` and the operator's first tick came
   back refused, naming a value they never touched. The selection is rebuilt
   from `/Opt` now, and the drop is disclosed in words. **Not driven** — it
   needs a fixture whose `/V` names no option, which
   `fixtures/all-field-kinds.pdf` does not carry, and that fixture is the
   cheapest next thing on this row.
   **Start at row 3**: `classify()` refuses with `NotOffered` and says nothing
   at all (`canvas/forms/boxes/mod.rs`), which is why he had to discover row 2
   by clicking; row 19 is O207, and it is adoption rather than engineering —
   `parse_length` already implements the CAD length grammar at the pin
   (`pdfcer_core::dimension::length_parse`), so the work is one helper in
   `pdfcer-gui-base`'s `units` reached from the entry sites, the same shape O194 step 2 used for
   the display side. Section 8.2’s rows are the channel’s work, not ours — count
   them, do not quote a number — and five have moved: **G022 and G023 are both shipped upstream and the pin
   carries them** — read out of the engine checkout, which is the only thing
   that settles such a claim, since a commit message can describe work that
   never landed — so a `/Btn` rotation now turns, and G022’s reply carries a fact the request did not, that `/Q` is
   inheritable, so the clear-alignment control it unblocks is labelled *inherit*
   and never *Left*. That control is still unbuilt and is now GUI work with no
   blocker in front of it. E3 went out as **`request_G024`**, which asks for an *emitter* over
   the three script enums `classify` already parses — Acrobat’s Format, Validate
   and Calculate tabs, plus `/CO` upkeep and the `/F`+`/K` pairing, as **one
   set**. **It has landed and the pin carries it** - `d2fa7352` is *Pass 308.6
   (G024)*, now two commits below the pinned revision, measured in the checkout
   rather than believed from a commit message, and there is no
   reply file in `open/` announcing it: the delivery arrived as a commit before
   it arrived as a note, which is now the third time. So the Format, Validate
   and Calculate surface is GUI work with nothing in front of it. That last
   clause is O206, which is doctrine rather than a feature:
   clause 7 of this file’s own contract, instrumented as section 7’s ten
   sibling sets. Building row 1 also produced **`request_G025`** and
   **`request_G026`**, both about `/Opt`, and **both have landed** - engine
   `Pass 308.7` + `308.8` in one commit, which the pin carries. `edit_field`
   now makes the same duplicate-export refusal `add_choice_field` always made,
   and `sort` means the same thing on both choice builders: the engine sorts
   when one edit supplies the list and sets the flag, and `choice_option_order`
   / `sort_choice_options` are exported so a shell cannot hold a second opinion
   about the ordering. **One workaround was deleted, one was kept, and the
   difference is worth a sentence.** The copied comparator went: the panel now
   calls the engine's sorter. The panel's own duplicate check STAYED, because
   an engine refusal reaching `vector_edit` is shown as `Declined::EditRefused`
   - *"That change was refused"* - which names neither the rule nor the value,
   and the operator is looking at a long list. That leaves one rule spelled in
   two places, which is the shape `G026` was filed against, so it went back out
   as **`request_G027`**: export the predicate as well as the ordering, for the
   engine's own stated reason. **It landed as `duplicate_choice_export` and is
   consumed** — `choiceopts::refuse_duplicate` asks the engine's own predicate
   and only the SENTENCE is the shell's half now, which is the part that has to
   be in the operator's language. The drift gate found the delivery, not the
   channel: there was no reply file, which is the fourth time. A new outcome
   field, `options_sorted`, is wired as the tripwire for the drift the export
   was meant to prevent. A request for one of a set is a request for the row, and the
   row ships whole or not at all. Row 26's combo and list halves are
   done — `the_option_arrows_are_greyed_only_at_the_ends_of_the_list` and
   `a_drop_down_can_be_answered_on_the_page`, both pinning
   `fixtures/all-field-kinds.pdf`. **Radio is the half left**, and it shares
   the focus-lock defect row 2 found, so it is a drive rather than a build.
7. **Drive the four that just shipped — O201, O202, O203, O204.** All four are
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
8. **O212 — panel docking, tear-out and cross-compartment drops: built, driven,
   and CLOSED BY HIM** — *"Their position is easy to manage and clear!"* Only
   the residue below is open.
   What is still a reading rather than a measurement is a non-unit ui scale:
   every window-origin conversion in the tear-out and float-drag paths was
   measured at `ppp = 1.0` only. The float window covering the compass it is
   being aimed with is an open question in `GUI_ROADMAP.md`.
   Steps 0 to 2 are built, falsified plant by plant and gate-green: the dock retains its own geometry; a dock tab drags along its own
   strip behind a dimmed-at-a-no-op caret; and `dock::drop` is the drop grammar
   as a pure value — `DockLayout::move_panel` and `accepts_drop`, with a
   12,000-case invariant fuzz that asserts what it swept. Step 3's two headless
   parts are built too. `dock::compass` resolves a pointer position over the
   retained geometry into a `DropTarget` — five zones over a compartment's body,
   its tab strip resolved first as a boundary between tabs — and returns the
   quadrilateral each zone is hit as, so the overlay cannot draw a shape it does
   not resolve. `dock::preview` answers what the drop would *do*:
   `DockLayout::preview_drop` clones the layout, applies the candidate with the
   same `move_panel` the release calls, and re-walks the rects with the very
   function `Dock::show` lays its own columns and stacks out with — so the
   highlight names the outcome, including the column a drag empties on its way
   out. **Step 3 is now complete**: `dock::overlay` fills the five quads over the
   hovered compartment's body, washes the armed one, outlines the rect the replay
   returns, dims the outline at a release that would permute nothing, and
   publishes a `DropPreview` the release reads. Eleven driven unit tests in
   `dock/overlay_tests.rs` carry it, over a shared `dock/drive.rs` harness, and
   eight falsification plants were all caught. **The gesture found a real defect**:
   every tab strip sits at the same y, so a drag carried sideways onto another
   strip stayed inside the reorder's y band and the origin strip went on drawing a
   caret hundreds of points away while the drop was never offered — `drag::preview`
   now bounds x by the strip exactly. `DESIGNS.md` carries the staging table and
   the traps; `D:/dev/rag/egui/` carries both findings.
   **Step 4 is complete too.** `dock::tear` draws a window outline around the
   pointer whenever a tab drag is carried outside every side's rect grown by a
   splitter's thickness, and `Intent::Float` carries an optional desktop position
   so the window opens where the operator let go rather than at the cascade. Eight
   driven tests in `dock/tear_tests.rs`.
   **★ The falsification is the finding, and it is written into the three files it
   concerns.** Three of seven plants left the suite green: the stand-downs in
   `tear::draw` and the stated ordering in `drag::settle` are all implied by the
   one geometric predicate `outside_the_dock`, so no input reaches them and no
   test can falsify them. They are kept — a release build where the implication
   breaks should draw one affordance rather than two — and what now measures the
   implication is a `debug_assert!` in `drag::settle` that names the two
   affordances that answered one drag. Falsified directly: widen the predicate and
   the guard absorbs it (green, and that is the input under which it is live);
   widen it and remove the guard and the assertion fires by name.
   **Step 5's dock half is built and falsified.** `DockState::set_float_drag`
   takes the panel, a pointer in the application window's own screen points, and
   whether this is the release frame; `dock::floatdrag` resolves that point with
   the same compass a tab drag uses, draws the same overlay, and applies the same
   move on release. Seven driven tests in `dock/floatdrag_tests.rs`, which send no
   pointer events at all, because in the real gesture the pointer is not
   `egui`'s. Two of six falsification plants only bit once the plant was moved to
   the real mechanism: the take inside `floatdrag` is implied by the take in
   `Dock::show`, and a pointer shifted by 40 pt still resolves to the same zone
   because the edge band is a quarter of the body capped at 64 pt. That tolerance
   is the file's stated blind spot, and it is why the conversion has to be driven
   rather than unit-tested.
   **The caller is built, so the gesture is reachable.** `dock::floatwin` senses
   a press on a float window's header strip, converts the pointer with
   `child_inner.min + local - app_inner.min`, and renews `set_float_drag` every
   frame until the release — the float window's own viewport is the only channel
   that reports the pointer during the gesture, because the application's context
   reports the button up throughout. The conversion is measured at `ppp = 1.0`
   only; whether the two window origins still agree at a non-unit ui scale is a
   reading. `DESIGNS.md` item 3 carries the numbers.
   **`panel_carried_home_lands_where_it_was_aimed` drives it.** `Driver::carry`
   is the harness verb: it raises the window the gesture *begins* in rather than
   the application's, and it rests 400 ms on the destination before letting go,
   because the pointer crosses two viewports before it becomes a drop offer and a
   release on the arrival frame lands nothing. The oracle is **which**
   compartment — a left-dock panel carried to the right dock's Objects stack, both
   tabs asserted into one tab bar by containment — so a landing that went home
   reads differently from a landing that went where the pointer was. Falsified
   twice, and the two plants separate the assertions: a fixed left-dock drop point
   reddens only *the panel is in no stack*; the right dock's **other** stack
   reddens only the aimed-at assertion.
   **The aim point is published by the application, not re-derived.** The header
   strip is the shell's geometry and the shell has no diagnostic channel and must
   not grow one, so `float.header.<panel>` is published from the tab-menu handler
   the application already supplies for that strip — inside the float's own
   `ViewportScope`, because that handler runs *before* the body closure that
   normally enters it and an untagged region converts against the wrong window's
   origin.
   **The window covers the compass** it is being aimed with, since it follows the
   pointer. The drop resolves and lands regardless; the remedy is an open question
   in `GUI_ROADMAP.md`.
   **The flag saying a float window has been opened is swept from
   `DockState::floats_seen`**, not from the set of floats about to be drawn: a
   panel docked back by *Dock all*, or by a drop applied earlier in the same
   frame, has already left that set, and a panel that kept its flag reopens
   wherever the platform chooses rather than where the operator left it.
   **`panel_tabs_can_be_rearranged` and `panels_float_close_and_dock` both drive
   PASS.** The latter's O126 A5 findings were the check's own, not the
   application's: the sections were not hermetic and fed each other through the
   saved layout, and the empty-window oracle asked for a viewport-tagged region
   that nothing in the program published. Both are repaired — every section fires
   `view.reset_layout` and asserts it landed, and the float body publishes
   `float.body.<panel>` and `float.content.<panel>` against the dock's own
   `empty=` count. **The drop compass and the tear outline are driven too**, by
   `a_drag_over_the_dock_offers_the_compartment_under_the_pointer` and
   `a_drag_carried_off_the_dock_offers_a_window`. Neither names a compartment:
   the arrangement is discovered from the regions the run itself published,
   because the left dock draws no tab strip at all while its rail is showing and
   a check that named a strip would assert about a surface the operator cannot
   see. **The subject of a landing assertion is the panel's body, not its tab** —
   a tab is a property of the compartment a panel is in, a body is a property of
   its being docked at all. **The tear check also proves the window opens where
   the outline promised** — same size, same desktop corner. The expected corner
   is computed by the harness from the application window's own client origin,
   read from the OS, and never from the `at` the application published: those
   two agree by construction, so a build that converts the outline to desktop
   points wrongly passes a check written that way. That was measured — deleting
   the conversion term opened a real window 788 pt left and 71 pt up of the
   outline and the tautological check still said PASS. Eighteen falsification
   plants, all caught.
9. **O198's remainder, which is O188 — the MOVE half is the O213/O214
   item above; what is left here is taking the lump APART.**
   `EditSession::split_text_object` and
   `text_object_split_plan` are in the pin and nothing in this shell calls
   either. Splitting is a separate capability from moving a line and buys
   nothing for it — a split-out line is still a text object, and `move_objects`
   refuses those (**G029**). The row in `ENGINE_BACKLOG.md` carries the five
   refusals that want operator sentences and the rule-4 disclosure `Line`
   granularity owes.
10. **O181** — installed fonts in Add Text, and the dead Format ribbon controls.
11. **O189** — bookmarks survive a cross-document page drag.
12. **O183** — the nine-part ce-dimension paragraph, part 7 first.
13. **O178** — multi-window tab dragging.
14. **O182** — white seams between the image tiles of a colour rendering.
15. **O194 steps 1, 4 and 5** — the ~30 surfaces in `UNIT_SURFACES.md` with no
   unit control, and `ui_text` abbreviations. Step 2 shipped as an invariant:
   `src/units.rs` is the only place a document length is converted or rounded for
   display, `whole()` the only function permitted to round one, and
   `check-unit-conversion.sh` fails the build on a fresh `25.4` under the GUI.
   Font and type sizes are excluded, and the exclusion is written into source.
16. **O195** — smart select in Review. `smart::enabled` defaults on and
    `clicking.rs` reads the scope every frame, but `textsel::takes_the_press`
    answers `tool.is_text() || (Select && !edit_content)`, so in Review the plain
    Select press is consumed as a text sweep and the smart rung never sees it.
    Flipping the manifest condition alone would ship a visible, inert control,
    which is what R9 exists to prevent.
17. **O176 — his verdict, not a repair.** At fit zoom a form field is about 28 px
    wide and its corner grips eat every point on it; the fix is a grip that yields
    the body below some size, not a harness zoom that hides it.
18. **Wire `EditSession::page_objects`, and decide cache invalidation first** —
    the one engine delivery of substance still unconsumed. Every edit pays two
    decompositions of the same page: the engine memoises its own against the
    session revision, and this shell runs a second parse in
    `panels::objects::provider` cached on `OpenDoc` by `app::cache` against a
    key of our own. The two are not interchangeable by name — ours is
    `OpenDoc::page_objects(&self)`, the engine's
    `EditSession::page_objects(&mut self, page_index)` — so measure the cost
    before assuming it, `tools/render-profile` in the engine tree being the
    instrument. The blocker is not the call: it is that dropping our cache means
    trusting the session's revision key for every invalidation our panels
    currently drive themselves.
19. **Re-run a full driven sweep, and read the SKIP set before the tally.** The
    last one reported `passed=86 failed=3 skipped=141` and **128 of those skips
    were one stuck notification toast**, so it measured nothing for more than
    half its roster while printing a tally that reads like a result. Ninety-five
    minutes taking the real cursor and keyboard, so only while he is away, and
    start `target/scratch/toast-watchdog.ps1` alongside it. Diff the SKIP set by
    NAME against the previous baseline in `target/scratch/`, in both directions.
20. **The Set-scale window never appears, and the ordering story for it is
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

- **A census of what the program EMITS cannot be completed, so no check may be
  built on one.** Trace names reach the diagnostic channel by at least four
  routes: a `format!` first token, a bare `"...".to_owned()` literal, an
  `eprintln!` inside `diag`, and helpers like `diag::ui_rect(name)` whose
  argument is a **runtime string no static scan can enumerate at all**. The
  first cut of `check-trace-names`'s mechanism 5 compared documented names
  against a `format!`-only census and immediately accused `ui-rect`, which is
  live. An incomplete census does not merely miss cases — **it blames correct
  code**, and a gate that fails on correct code is a gate somebody switches
  off. The sound shape is the negative one: a plain substring search asking
  only whether a name appears anywhere outside a comment, so the errors are
  missed defects rather than accusations. Loose in the safe direction beats
  precise in the unsafe one.

- **A rung that publishes nothing is a rung no driven check can see, and the
  Escape ladder had one.** Every claimant on `canvas::keys`'s Escape ladder
  emits `canvas-escape outcome=…` — except, until now, the text draft, which
  is the one rung that WRITES what it retires. `text-edit-abandon` is not a
  substitute: its own contract says it is not evidence that anything was lost,
  and it fires whatever ended the draft. The structural cause was a shadowed
  binding (`let vertex_abandoned = vertex_abandoned || …`) that left no named
  result in scope where the outcome arms are written; the rung-scoped name
  `rung_3a_spent` replaces it. ⚠ Renaming that binding is **not** a rename —
  five downstream reads bound to the *un-shadowed* original and the compiler
  says nothing. Grep every use first, and assert the count in the patch.

- **A panel body owns its own scrolling; the dock only styles the bar.** The
  dock's helper sets `ScrollStyle::solid()` and nothing else — it creates no
  `ScrollArea`, so a body taller than its slot lays its remainder out past the
  bottom of the pane and, in a side dock, past the bottom of the **window**.
  Nothing clips and nothing errors. The redaction marking panel shipped with
  its search field, its find-and-mark button, its match-mode pair and its hint
  off screen at 1,100x800, leaving *Mark whole page* — the widest redaction
  offered — as the only reachable way to mark anything. The oracle is one
  comparison, `child.max.y > window_inner.max.y`; no unit test and no gate can
  see it. **A control a driven check will CLICK is published with
  `ui_rect_visible`, never plain `ui_rect`** — then an absent rect means *not
  reachable*, which is the truth and is actionable. And treat any comment
  saying *the host already wraps this* as a claim to grep, not a fact: that
  sentence is what kept the missing area alive through every later read.

- **A harness helper that takes a bare `Rect` cannot be made viewport-aware —
  the signature is the defect.** A rect is in the coordinate space of the
  viewport that drew it, and the apply dialogs are real OS child viewports with
  their own origins. Converting one against `session.frame()` yields a
  plausible desktop point several hundred points away: no error, no
  missed-click report, just an acknowledgement that never takes. Because the
  dialog declares its confirm region only when armed, it presents as *"the
  application declared no `redact-apply-confirm` region"* — a **SKIP**. Both
  redaction checks, one of them the only cover on the most irreversible
  operation the program has, had exercised nothing since the dialogs became
  viewports. `click_region` now takes the region **name**, re-reads the trace
  (it is a change log, so a rect fetched before an intervening click may name
  where the control *was*) and converts through `driving::frame_of`. The
  correct twin was already sitting in `checks/ocr.rs` and was never swept: a
  fix that lives in one copy of a duplicated helper is not a fix.

- **`PDFCER_DIAG_VIEWPORT`'s height is silently clamped to the monitor's work
  area, so a control below the fold CANNOT be revealed by asking for a taller
  window.** Measured 2026-09-16: heights of 2000, 2200 and 2600 all produced the
  **byte-identical** clip rect `[[846.0 821.8] - [1200.0 1406.0]]`, because both
  monitors here are 3440x1440 with a 3440x1392 work area. Nothing is reported
  back — the request simply does not happen. The tell was available at the
  second rung and was nearly read past: **a result uniform at every rung of a
  sweep is a measurement of the probe, not the subject.** A check that asserts
  on a rect it cannot bring into view fails identically on a correct build and a
  broken one, which manufactures an investigation that ends at the harness. So:
  assert the traced **state** for every row (`diag::ui_control` publishes
  `enabled=` whether or not the rect was published), and assert **reachability**
  for at least one row, never for all — *"some row is in the viewport"* catches
  the shipped-unreachable defect, *"all three are"* is a claim about the monitor.
  Full workings in `D:/dev/rag/egui/`. Width was never swept, and a figure quoted
  for the wrong axis is wrong by the aspect ratio.

- **A SKIP is not a FAIL, and nearly every FAIL is a defect in the CHECK — a
  prior, never a verdict.** Load alone removes coverage with nothing turning
  red, so diff skips by NAME and run one; and a session that reads the first
  harness fault as proof of a harness bug stops where a program defect once sat.
- **Anything appended to the print dialog is a SIZE claim, on both axes, and
  there is now a gate for half of it.** A plain `ui.horizontal` lays out past
  the end of its column and reports a `min_rect` that wide, which becomes the
  body's content size and a scrollbar the operator cannot dismiss — he has
  reported that in the same seven words twice. `check-scroll-row-wrapping.sh`
  now fails any non-wrapped row under `dialogs/print/` that does not carry
  `// scroll-row-exempt: <reason>`; two rows legitimately do. The vertical half
  has no gate and cannot easily have one: `allocate_ui_with_layout` sizes the
  options column at a **floor**, so a group or a wrapped tab-strip row that
  grows past it overflows silently, and the fitting case reads back the
  allocation rather than the children's real height — there is no trace value
  for the remaining slack. **Smoke-launch off-screen and read `print-body`'s
  `egui_content` against `egui_view` after any edit there, including a
  reworded tab label**: a label's width is the strip's height.

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
  lines carrying the `*"` quote marker, before and after. Measure it with
  `grep -rn '^\s*//.*\*"' --include='*.rs' crates/ tools/ | wc -l`, which reads
  **7,375** over the tree of 2026-09-20. If a comment pass lowers that
  number, it removed evidence, whatever else it removed.
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
- **A line citation into the engine rots into something that still READS
  correct.** The lock takes pdfcer by branch with no `rev`, so the source moves
  with no command run here. Upstream insertion shifts a file uniformly rather
  than scrambling it, so a drifted number still lands inside readable prose
  about a real function in the right file — `edit.rs` line 48704, written against
  the `/AA` removal, now lands inside `add_image`. **A repair keyed on *does
  this line exist* passes every one of them**; 66 of 92 measured had drifted
  and none dangled. `check-engine-citation.sh` is registered and green, but it
  is a shape checker with a stated blind spot: a bare citation into a SMALL
  engine file — `forms.rs`, `form_script/mod.rs`, `dimension/length_parse.rs`
  — carries a line number a local file could plausibly hold, so nothing sees
  it. Cite by symbol; that is what R5 says and it is the only form that
  survives. The same argument covers `D:\Dev\pdfce\crates\pdfce-gui`: a frozen
  tree is not a frozen-correct one, because its citations kept drifting right
  up to the freeze, so what froze was the error.

- **Seven source files sit within fifty lines of the hard limit**, led by
  `crates/pdfcer-gui/src/render/settle.rs` at 1,486 against
  `check-file-size.sh`'s 1,500, the rest running down to the 1,450 mark. Adding
  one ordinary function to any of them turns a green gate red mid-task, and the remedy R2 requires is
  *find the seam*, never *raise the limit* — which is a refactor, not an edit,
  and it will arrive at the worst moment unless it is done first. The seam for
  each is argued in `DESIGNS.md`; one of the seven has none, and knowing which
  one is what stops a cut being chosen to hit a line count.
  ⚠ `tools/ui-verify/src/checks/mod.rs` is off this list and will return: it
  grows by about a dozen lines **per driven check registered**, because R1 work
  adds a documented `pub mod` there every time. It is the one file whose
  pressure is a function of doing the project's own standing work, so it reaches
  the cap on a schedule rather than by accident. Its one deferral — the
  `Check` trait and `CheckContext` to `checks/harness.rs` — is spent, and the
  remedy left is the subdirectory fold, which costs a rename of every
  `crate::checks::<name>` reference. Take it before the next two checks, not
  after.
  Re-measure before quoting these:
  `find crates tools -name '*.rs' -not -path '*/target/*' -print0 | xargs -0 wc -l | grep -v ' total$' | sort -rn | head`
  — the `grep -v` is load-bearing, because `xargs` batches and emits one
  `total` line per batch.

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

1. **Re-measure `FEATURES.md` against the build before packaging.** It is the
   file he reads to know what he has, so a stale row is not an inconvenience,
   it is a report he cannot act on. Every row carries the command that produces
   its figure rather than the figure itself; run those commands, and read the
   three that still carry a bare number — the engine version, the panel count
   and its line citation, and the ribbon's captioned-group count. A row that
   turns out to have been a *claim* rather than a stale figure is corrected in
   place and keeps what it got wrong.
2. **Update the engine first**: `cargo update -p pdfcer-core -p pdfcer-render -p
   pdfcer-print`. `package-portable.py` does this as its own first step, so on a
   day the engine commits live every packaging run moves the pin and stamps the
   artefact `-dirty`: commit `Cargo.lock`, then package `--no-update`. A stale
   pin has cost eighteen missing images on his own file.
3. **The order:** commit, `touch crates/pdfcer-gui/build.rs`, rebuild, re-drive
   the checks that read the build stamp against the shipped exe, off-screen smoke
   launch, `package-portable --no-build --no-update`, push, `gh release create`.
   **Never pass `--prerelease`** — GitHub hides one from `releases/latest`.
4. **Smoke-launch off-screen before every release.** Ninety seconds, and it has
   beaten four thousand unit tests and the whole gate suite to a live defect: a
   geometry change moves where a document OPENS, not only where it can be dragged.
5. **Publish every build worth keeping.** `package-portable.py` rotates the two
   OneDrive slots itself, replacing the older so the previous build survives;
   pass `--slot` only to RETRACT. Read `BUILD-INFO.txt` back out of **both**
   slots afterwards — a tool's own report is not evidence about its own effect —
   and say which slot holds which build, because the slot name carries no
   version. His install at `C:\Users\Ken\OneDrive\pdfcer\` is never touched, and
   the packager's argument against a release with nothing visible is overruled.
6. **A test that proves WHICH mechanism ran must assert what the WRONG mechanism
   cannot produce.** Falsify it by putting the old call back, never by breaking
   the input.
