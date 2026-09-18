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

1. **O209 — the form fields did not look or act like Acrobat's; six of seven
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
   header now says so.
2. **O208 is built, driven and falsified — what is left is his word.** Both
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
3. **The form tools, a unit typed beside a number, and the rule behind both —
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
4. **Drive the four that just shipped — O201, O202, O203, O204.** All four are
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
5. **O212 — panel docking, tear-out and cross-compartment drops, the rest of
   steps 3 to 5 of six.** Steps 0 to 2 are built, falsified plant by plant and
   gate-green: the dock retains its own geometry; a dock tab drags along its own
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
   out. **No operator gesture reaches any of it yet**, which is why no register
   or `FEATURES.md` row moved. `DESIGNS.md` carries the staging table and the
   traps. **What is left of step 3 needs the screen** — the painted overlay:
   fill the five quads, highlight the rect the replay returns, wire the release.
   Two things to re-measure first: `panels_float_close_and_dock` has a
   live undriven defect from O126 A5 — a floated panel opens an empty window and
   *Dock all* recovers nothing — and steps 4 and 5 build directly on that
   machinery; and `FEATURES.md` carries no float or tear-out row at all, though
   the register marks the capability built. Also owed from step 1:
   `panel_tabs_can_be_rearranged` drove green *before* the caret dimming landed,
   so it wants one re-run on a build carrying it.
6. **O198's remainder, which is O188** — a title-block run the exporter wrote as
   one lump is still one lump: he can reach that text and drag a line, not take
   it apart. **The verb has landed and is in the pin**: `EditSession::split_text_object`
   with `text_object_split_plan` for the cost-before-committing half. Nothing in
   this shell calls either; the row in `ENGINE_BACKLOG.md` carries the five
   refusals that want operator sentences and the rule-4 disclosure `Line`
   granularity owes.
7. **O181** — installed fonts in Add Text, and the dead Format ribbon controls.
8. **O189** — bookmarks survive a cross-document page drag.
9. **O183** — the nine-part ce-dimension paragraph, part 7 first.
10. **O178** — multi-window tab dragging.
11. **O182** — white seams between the image tiles of a colour rendering.
12. **O194 steps 1, 4 and 5** — the ~30 surfaces in `UNIT_SURFACES.md` with no
   unit control, and `ui_text` abbreviations. Step 2 shipped as an invariant:
   `src/units.rs` is the only place a document length is converted or rounded for
   display, `whole()` the only function permitted to round one, and
   `check-unit-conversion.sh` fails the build on a fresh `25.4` under the GUI.
   Font and type sizes are excluded, and the exclusion is written into source.
13. **O195** — smart select in Review. `smart::enabled` defaults on and
    `clicking.rs` reads the scope every frame, but `textsel::takes_the_press`
    answers `tool.is_text() || (Select && !edit_content)`, so in Review the plain
    Select press is consumed as a text sweep and the smart rung never sees it.
    Flipping the manifest condition alone would ship a visible, inert control,
    which is what R9 exists to prevent.
14. **O176 — his verdict, not a repair.** At fit zoom a form field is about 28 px
    wide and its corner grips eat every point on it; the fix is a grip that yields
    the body below some size, not a harness zoom that hides it.
15. **Wire `SignReport::appearance_lines`** — the one engine delivery genuinely
    owed. It is what the engine drew into the signature box, and here it reaches
    only a trace the source marks as never displayed, so he signs without seeing
    what the stamp says while the engine records *a rectangle too small for them*
    as a real outcome. Off-canvas, in the signing dialog's result.
16. **Re-run a full driven sweep, and read the SKIP set before the tally.** The
    last one reported `passed=86 failed=3 skipped=141` and **128 of those skips
    were one stuck notification toast**, so it measured nothing for more than
    half its roster while printing a tally that reads like a result. Ninety-five
    minutes taking the real cursor and keyboard, so only while he is away, and
    start `target/scratch/toast-watchdog.ps1` alongside it. Diff the SKIP set by
    NAME against the previous baseline in `target/scratch/`, in both directions.
17. **The Set-scale window never appears, and the ordering story for it is
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
