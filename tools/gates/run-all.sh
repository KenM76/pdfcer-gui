#!/usr/bin/env bash
# run-all.sh — every gate in this directory, plus fmt and clippy, in one run.
#
# ===========================================================================
# WHAT THIS IS FOR
# ===========================================================================
#
# One command a developer runs before pushing, and the one CI runs. The reason
# it exists as a script rather than a list of steps in a CI YAML is the lesson
# recorded at the top of `check-ui-strings.sh`: pdfcer's string rule lived as an
# inline CI grep, was red at baseline for months, and therefore enforced
# nothing. A gate has to be runnable locally, in one command, or it becomes
# scenery.
#
# ===========================================================================
# THE THREE-STATE MODEL, AND WHY "SKIPPED" IS NOT "PASSED"
# ===========================================================================
#
# Every gate here returns one of three things:
#
#   0  PASS     — it ran, and found nothing wrong.
#   1  FAIL     — it ran, and found something wrong.
#   2  SKIPPED  — its PRECONDITION was absent. The crate does not exist yet,
#                 the tree has no source files, the binary was never built.
#
# Most gate runners have two states and fold the third into the first. That is
# the single defect this project is most determined not to repeat:
# PROJECT_PLAN.md §4.1 documents a gate that "would print `ui-strings: clean`
# while checking a handful of files", because finding nothing looks exactly
# like finding no violations.
#
# So SKIPPED is tracked separately, printed in its own block, and — critically
# — a run containing any skip exits **3**, not 0. It is not a failure, and it
# is not a pass either. CI must not go green on a gate set that did not fully
# run. If a skip is expected (another crate is mid-write, this is a partial
# checkout), the human reads the reason and decides; the machine does not get
# to decide it for them.
#
# ===========================================================================
# ORDER
# ===========================================================================
#
# The self-tests run FIRST, before any gate is trusted. If a gate cannot detect
# its own planted violation, its verdict on the real crate is worth nothing,
# and finding that out after a green run is finding it out too late.
#
# EVERY GATE THAT IS A GREP OVER SOURCE CARRIES ONE. That is the rule; the
# list is whatever is dispatched below, and this paragraph deliberately no
# longer names a count.
#
# It used to say "Three gates carry one" and it was wrong by the time anyone
# read it — `check-strong-text.sh` was added with a self-test and the sentence
# was not, so a header describing the file it sits in was off by one, then by
# two. **A number written in prose beside the thing it counts is a claim that
# decays**, and this project has now spent six corrections on that exact shape.
# The dispatch block is the list. Read it instead.
#
# The reason the rule is "every grep": a grep over source is the category that
# fails SILENTLY. A pattern that stops matching, a path that stops resolving,
# and a find that walks an empty tree all print exactly what a clean run
# prints.
#
# `check-shipped-assets.sh` fails silently for a different and worse reason. It
# checks that every redistributed third-party asset's licence reaches the
# operator, and a repository with no asset directories, or a scan that finds
# none, prints "clean" just as loudly as one where every obligation is
# discharged. That gate ALSO has no natural failure in daily use: assets are
# added rarely, so it could sit green for months while quietly checking
# nothing. Its self-test plants four separate violations, exempts a fifth, and
# passes a sixth.
#
# `check-string-gaps.sh` is the newest and the same argument applies twice
# over: the defect it hunts is invisible in a diff, so nobody would notice the
# gate had gone blind either.
#
# fmt and clippy run LAST, because they are the slow ones and because a
# formatting complaint is the least interesting thing this script can tell you.
#
# ===========================================================================
# USAGE / EXIT CODES
# ===========================================================================
#   tools/gates/run-all.sh              everything
#   tools/gates/run-all.sh --no-cargo   gates only, no fmt/clippy (fast)
#
#   0  everything ran and everything passed
#   1  at least one gate FAILED
#   3  nothing failed, but at least one gate was SKIPPED — an incomplete run

set -uo pipefail          # NOT -e: a failing gate must be recorded, not fatal

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
cd "$ROOT" || exit 1

RUN_CARGO=1
[ "${1:-}" = "--no-cargo" ] && RUN_CARGO=0

PASSED=(); FAILED=(); SKIPPED=()

rule() { printf '%s\n' "------------------------------------------------------------------------"; }

# run <label> <command...> — run one gate, classify by exit code, record it.
run() {
    local label="$1"; shift
    rule
    echo ">> $label"
    rule
    "$@"
    local rc=$?
    case "$rc" in
        0) PASSED+=("$label") ;;
        2) SKIPPED+=("$label") ;;
        *) FAILED+=("$label") ;;
    esac
    echo ""
    return 0
}

echo ""
echo "pdfcer-gui gates — $(date '+%Y-%m-%d %H:%M:%S') — $ROOT"
echo ""

# --- 0. the gates prove they can fail, before their verdicts are believed ---
run "check-ui-strings --self-test"  bash "$HERE/check-ui-strings.sh" --self-test
run "check-theme-colors --self-test" bash "$HERE/check-theme-colors.sh" --self-test
run "check-strong-text --self-test" bash "$HERE/check-strong-text.sh" --self-test
run "check-shipped-assets --self-test" bash "$HERE/check-shipped-assets.sh" --self-test
run "check-string-gaps --self-test" bash "$HERE/check-string-gaps.sh" --self-test
run "check-trace-names --self-test" python "$HERE/check-trace-names.py" --self-test
run "check-orphan-docs --self-test" python "$HERE/check-orphan-docs.py" --self-test
run "check-region-names --self-test" python "$HERE/check-region-names.py" --self-test
run "check-doc-markup --self-test" python "$HERE/check-doc-markup.py" --self-test
run "check-gate-input-scope --self-test" python "$HERE/check-gate-input-scope.py" --self-test
run "check-memory-index --self-test" bash "$HERE/check-memory-index.sh" --self-test
run "check-completeness-tests --self-test" python "$HERE/check-completeness-tests.py" --self-test
run "check-unit-conversion --self-test" bash "$HERE/check-unit-conversion.sh" --self-test
run "check-test-temp-paths --self-test" python "$HERE/check-test-temp-paths.py" --self-test
run "check-patch-residue --self-test" python "$HERE/check-patch-residue.py" --self-test
run "check-settings-funnel --self-test" python "$HERE/check-settings-funnel.py" --self-test
run "check-scroll-row-wrapping --self-test" bash "$HERE/check-scroll-row-wrapping.sh" --self-test
run "check-texture-census --self-test" python "$HERE/check-texture-census.py" --self-test
# ★★ A gate can report OK over an evidence set of size ZERO and look exactly
# like a gate that passed. Its `archived` case is the guard: it returns 1 only
# if the consumption notes in `archive/` are read, so narrowing the evidence
# set back to `open/` turns this suite red instead of turning that gate
# silently blind.
run "check-stale-blockers --self-test" bash "$HERE/check-stale-blockers.sh" --self-test
# The licence gate compares generated bytes against a committed file, and the
# two arms of its verdict have opposite remedies, so its self-test asserts that
# each names what the other cannot: a body difference names NO crate, a missing
# crate IS named. Its third arm is the one that made it necessary — a
# CRLF-only difference must pass, because `.gitattributes` strips exactly those
# carriage returns on commit and the gate was otherwise green only on whichever
# machine had last run the generator.
run "check-third-party-licences --self-test" bash "$HERE/check-third-party-licences.sh" --self-test

# Both of these had written a `--self-test`, and neither was reached: a
# self-test absent from this list is a file nobody runs, and writing one is
# half the work. `check-engine-api-drift` plants a new item in an EXISTING
# module, which is the case its module rule could swallow;
# `check-unreachable-refusals` plants both directions.
run "check-engine-api-drift --self-test" bash "$HERE/check-engine-api-drift.sh" --self-test
run "check-unreachable-refusals --self-test" bash "$HERE/check-unreachable-refusals.sh" --self-test

# R7's only mechanical guard, and it had never been falsified. Seven arms,
# because the gate has seven outcomes and a plant fires exactly one: three
# planted domain dependencies (a manifest key, a `[dependencies.pdfcer-*]`
# table header, a source import past a clean manifest), the comment exemption
# asserted to PASS, both absent-precondition states asserted to exit 2 rather
# than 0, and the real crate as the unplanted control. The exemption arm and
# the two skip arms are the ones nobody writes by hand, and blinding the gate
# four different ways fires a different single arm each time.
run "check-shell-purity --self-test" bash "$HERE/check-shell-purity.sh" --self-test

# Nine arms against a stub instrument, so this costs a second rather than the
# gate's own two minutes. Five pin the ACCOUNTING rule that had existed only as
# a comment — a verb named in prose cannot discharge it, nor can a row without
# backticks, while a row's reason cell still can, because the register's
# alternate-spellings table discharges verbs that way. The other four are the
# nothing-was-measured states, including an instrument that prints its summary
# and then dies mid-list: that one used to pass, because the exit status being
# read belonged to the `tr` at the end of the pipe.
run "check-verb-coverage --self-test" bash "$HERE/check-verb-coverage.sh" --self-test

# --- 1. the gates themselves ------------------------------------------------
run "check-ui-strings"   bash "$HERE/check-ui-strings.sh"
run "check-theme-colors" bash "$HERE/check-theme-colors.sh"
# ★★ `check-unit-conversion` — O194 step 2.
#
# One length-conversion table, one rounding rule. Thirteen private copies
# of the same two constants had accumulated, and the one that reached the
# operator was not the arithmetic but the ROUNDING: a 210.5 mm sheet read
# 210 in the page thumbnails and 211 in the print dialogue, because half
# the sites used `.round()` and the other half `{:.0}`, which is half to
# EVEN. Nobody chose half-to-even; it is simply what you type.
run "check-unit-conversion" bash "$HERE/check-unit-conversion.sh"
# ★ DEFECTS.md D11, mechanised. **A rule that lives only in a document is
# enforced as often as somebody remembers to read it** — this one was broken
# again within days by a reader who had read it.
run "check-strong-text" bash "$HERE/check-strong-text.sh"
# ★★★ `check-plate-colour` — DEFECTS.md D2 for the third
# time, and the first two fixes did not generalise.
#
# `Palette::on_accent` means "drawn ON the accent". On anything else it is a
# pale glyph on a pale surface: present, correctly sized, and invisible. An
# outside reviewer found two more instances in screenshots — the selected dock
# tab and the document tab's close ✕, the latter white-on-white to within five
# levels of luminance under the Airy preset.
#
# ★★ The contrast gate structurally cannot see them. It enumerates widget
# states against widget fills, and both of these are a colour the CALLER
# supplied against a background chosen by geometry. A perceptual gate asks
# "is this readable"; this one asks "were these two things ever paired at all",
# which is the question that kept going wrong.
#
# ★ Its non-vacuity evidence is real sites rather than a planted one: run
# against a tree with the two known fixes reverted, it names exactly the three
# defective drawing sites and passes the four correct ones.
run "check-plate-colour --self-test" bash "$HERE/check-plate-colour.sh" --self-test
run "check-plate-colour" bash "$HERE/check-plate-colour.sh"
# ★★★ `check-selection-channel` — `check-plate-colour`'s twin, one
# channel over.
#
# `egui::Visuals::selection` is egui's styling channel for a selected WIDGET:
# `Style::button_style` takes both fills AND the text colour from it for every
# `Button::selected(true)` and every `ui.selectable_label(true, …)`
# (`egui-0.35.0/src/widget_style.rs:151-154`). This theme had handed that channel to the
# CANVAS — a 27 % object tint and the canvas outline ink — because 33 readers
# across 13 files depended on it. The canvas won, so nineteen selected chrome
# controls painted accent text on a wash: luminance gap 72.5 under Dark,
# against a floor of 90.
#
# ★★ Neither existing gate could see it, and the reason is worth keeping.
# `check-theme-colors` forbids INVENTED colours and both values were correctly
# sourced from the palette. `theme::contrast` enumerates five widget states ×
# two fills, reading `fg_stroke` against `bg_fill` — and the selected pair is
# in none of them, because egui substitutes it at PAINT time, after the style
# has been read. A gate that reads a `Style` back cannot reach a pair that was
# never in the struct. Correctly sourced, gate green, unreadable on screen —
# for the fourth time (`DEFECTS.md` D2).
#
# ★ Its one file-level exemption's premise is pinned by a Rust test rather than
# by prose, which is T1's lesson taken literally: `check-strong-text.sh` had
# blessed a site for a reason that had silently stopped being true.
run "check-selection-channel --self-test" bash "$HERE/check-selection-channel.sh" --self-test
run "check-selection-channel" bash "$HERE/check-selection-channel.sh"
# ★★ The operator found this class TWICE, three weeks apart, and the second
# instance sat a hundred lines under a comment block explaining the
# mechanism. A finding written next to the code does not apply itself.
run "check-scroll-row-wrapping" bash "$HERE/check-scroll-row-wrapping.sh"
# ★★★ `check-custom-kind-drawn` — the one ribbon defect that shipped for a
# whole release with every gate green.
#
# `Item::Custom` is the seam where the manifest stops describing a control and
# starts trusting the application to draw one: the shell reserves a slot and
# emits a `kind` string, and if nothing matches that string the slot draws
# empty under its caption. Markup ▸ Style emitted the literal `"colour_swatch"`
# and no renderer ever matched it, for the whole of v0.1.0.
#
# ★★ Nothing else in the tree asks the question. Every reachability check is
# built on `Shell::command_references`, which walks tabs, the QAT and the
# keymap — the places a command *id* appears — and a `Custom` item carries no
# id by design. The manifest was well-formed, the group reachable, the item
# declared and the caption drawn; the only missing thing was the widget.
#
# ★ Keyed on the `Item::custom(` call, which is the moving side: a thirteenth
# control is added by writing one of those. A register, a marker comment or a
# naming convention would each be something the thirteenth control could be
# added without touching. And a mention in a doc COMMENT does not count —
# by the time the defect was found, `COLOUR_SWATCH` was discussed by name in
# half a dozen of them, so a laxer gate would have been green on the strength
# of prose explaining that the widget did not exist.
run "check-custom-kind-drawn --self-test" bash "$HERE/check-custom-kind-drawn.sh" --self-test
run "check-custom-kind-drawn" bash "$HERE/check-custom-kind-drawn.sh"
# ★★ `check-escape-disposition` — O223, standing.
#
#   "it is easy to accidentally press escape and lose a lot of text that has
#    been entered."
#
# The scope he gave is *"any tool that has text"*, and contract clause 7 says a
# scope like that is COUNTED rather than discovered one complaint at a time.
# Counted once by hand is a paragraph that is true on the day it is written and
# silently false the next time somebody adds a field; this is the counting, kept
# running. Every text field states what Escape does to what has been typed, in
# one of five words the gate checks against a closed list.
#
# ★ It cannot tell whether the answer is TRUE — a site labelled `commits` whose
# commit path is broken passes here. That is `ui-verify`'s question. This one
# asks only whether a human being asked the operator's question at this call
# site, which is answerable by grep and was, for all sixty-three, unasked.
run "check-escape-disposition --self-test" bash "$HERE/check-escape-disposition.sh" --self-test
run "check-escape-disposition" bash "$HERE/check-escape-disposition.sh"
run "check-file-size"    bash "$HERE/check-file-size.sh"
run "check-shell-purity" bash "$HERE/check-shell-purity.sh"
run "check-shipped-assets" bash "$HERE/check-shipped-assets.sh"
run "check-string-gaps"  bash "$HERE/check-string-gaps.sh"
# ★★★ The first gate here aimed at PROSE being false rather than at code being
# wrong. When a capability arrives, every place that describes its ABSENCE
# becomes a lie at once: an on-screen explainer, a module header that FORBIDS
# the feature, a passing test that would fail it, a FEATURES row. **All of them
# were correct when written, which is why nothing looks wrong and no other gate
# can see them.** This catches the one mechanical part — a row that says
# BLOCKED and names a request the channel shows we have CONSUMED.
run "check-stale-blockers" bash "$HERE/check-stale-blockers.sh"
# ★ Two gates born of an operator report rather than of a design.
# `check-typing-guard` keeps "is the operator typing?" a single
# predicate, after the space bar was stolen by the pan tool for a fortnight;
# `check-conventions` makes every interactive surface answer, row by row, the
# conventions its gesture class carries - because every convention he had to
# report was one nobody had ASKED about, not one somebody decided against.
run "check-typing-guard" bash "$HERE/check-typing-guard.sh"
run "check-conventions"  bash "$HERE/check-conventions.sh"
# ★ Born the same way and from the same failure mode as `check-typing-guard`:
# **a finding recorded in one module and never applied to its siblings.**
# Ctrl+C/X/V never arrive as key events - egui-winit intercepts them - so
# `key_pressed(Key::C)` is permanently false in a real window. A test that
# injects the key event winit never sends certifies the mistake green, so the
# grep is the only instrument that reaches every site at once.
run "check-clipboard-chords" bash "$HERE/check-clipboard-chords.sh"

# `check-suite-name-absent` keeps a LICENSED print-conformance suite's name out
# of this repository entirely -- contents and file names -- per the operator's
# ruling. Shared with `D:/Dev/pdfcer/tools/` rather than re-derived, because it
# already carries the fixes for two defects a careful reader makes anyway.
#
# ★ It looks odd on purpose. Its needles are base64-encoded, because a gate for
# "this word must never appear" that greps for the word in plain text becomes
# its own first violation. And its failure output is MASKED, because a path can
# itself be the violation -- the engine's version once printed a violating file
# name into a public CI log, which is an artifact that outlives the fix.
run "check-suite-name-absent" python "$ROOT/tools/check-suite-name-absent.py"

# `check-trace-names` stops a module's own trace line sharing its first token
# with a `vector_edit` label. `ui-verify` reads a trace by that token and the
# funnel writes `<label> page=... n=... epoch=...` for the same edit, so two
# lines with one name means `.last(<label>)` returns the FUNNEL's and a check
# asking for the module's keys finds none - and reports "the verb did nothing"
# about a verb that worked. A confident false negative.
#
# Three instances in two days, the second written by the session that had just
# written up the first, the third hours after that. A convention held by memory
# failed once a day; this is the grep that replaces it.
#
# It found three more the moment it worked, one of them written the same hour,
# and two that were correct only by STATEMENT ORDER - the module's line happened
# to be traced after the funnel's, so `.last()` returned the right one by luck.
#
# ★★ Mechanism 3: a trace name that reads like a debugging leftover -- any
# capital letter, or a tmp/temp/dbg/debug/xxx/todo/fixme/hack prefix -- is a
# violation in its own right.
#
# ⚠ Such a name survives everything that is not a grep for it. A `TMPASK` in a
# RELEASE binary passed this runner, 4,190 unit tests, a project-wide rename,
# and eight appearances per dialog in captured traces several sessions had
# read; an ordinary ninety-second smoke launch is what found it.
#
# The instructive half is why THIS gate had never seen it: `FIRST_TOKEN` is
# anchored `[a-z]`, because every deliberate trace name in the crate is
# lowercase-and-hyphens. An all-caps scratch name never entered the scan. The
# gate was not silent because the rule was weak; it was silent because the name
# did not look like a trace name, which is exactly what makes a leftover a
# leftover.
#
# ★★ Mechanism 5: a name a module header documents that NOTHING emits. The
# header's sample block is where a harness author reads a name from, so a
# rename that misses it produces the identical false negative by a fourth
# route, and the most tempting one, because a header is written to be
# authoritative. `form-abandon` outlived its emitter exactly this way.
#
# ⚠ The obvious design -- compare the headers against a census of emitted
# names -- is UNSOUND and was rejected after it accused a live name. Names
# reach the channel through `format!`, a bare literal, `eprintln!`, and helpers
# taking a RUNTIME string no static census can enumerate. An incomplete census
# does not miss cases, it blames live ones. The emitted side is a plain
# substring search instead, loose in the safe direction.
#
# The self-test is registered up in section 0, per this repository's
# rule that a `--self-test` absent from this list does not exist. Its planted
# inputs pair a violation with the correct construct that most resembles it,
# including a PDF content stream (`BT /F1 12 Tf`) that pins mechanism 3's
# anchor -- the first cut of that rule scanned whole files and reported 31 hits
# of which one was real. Mechanism 5 was additionally falsified against the
# real sources at the commit before its defect was fixed, because a self-test
# proves only that a mechanism fires on input its own author planted.
run "check-trace-names" python "$HERE/check-trace-names.py"

# `check-texture-census` protects an argument from ELIMINATION, which is the
# kind that rots quietly.
#
# `render::pressure` reads the OpenGL error flag once a frame and tries to say
# which upload raised it. GL's flag carries no provenance, so the module works
# by elimination: it blames the canvas's page raster only when that raster was
# the frame's ONLY upload. Every uncounted upload therefore does not merely go
# unseen -- it makes a two-upload frame look like a one-upload frame, so the
# module stops refusing to guess and blames the one it can still see.
#
# The completeness that argument rests on is exactly what a hand-written list
# of upload sites cannot hold, and there is no test that can see a call site.
# Hence a gate, reading the source. It deliberately does NOT check that the
# `Surface` named is the right one -- that is a judgement about what an upload
# is FOR, and `render::pressure`'s own tests pin the consequence instead.
run "check-texture-census" python "$HERE/check-texture-census.py"
# ★★★ `check-orphan-docs` — and it is the only gate here
# aimed at documentation being attached to the WRONG ITEM rather than at its
# being absent or false.
#
# A contiguous run of `///` lines is ONE doc comment to Rust, so an item
# inserted below an existing item's doc comment — rather than below that item
# — adopts its documentation. The owner is left with none. TWELVE instances
# had accumulated in this crate over thirteen days and shipped in four
# releases.
#
# ★★ No part of the toolchain can see it. `rustc` is happy; a doc comment is
# valid prose wherever it sits. `cargo fmt` is happy. `clippy -D warnings` is
# happy. `cargo doc` would show it and nobody runs that on a binary crate. The
# one that surfaced was luck: deleting an absorbing item left a blank line
# after a doc comment, which clippy DOES lint. Chasing that lint rather than
# silencing it found the other eleven.
#
# ★ Its self-test falsifies in both directions, because this gate's history is
# two wrong drafts. Aimed at a bold title AFTER a paragraph break it found
# ZERO — that is the shape of an ordinary mid-doc heading, so the detector was
# pointed at the normal case and a clean report meant nothing. Aimed at `**`
# anywhere it found 717, all but none of them mid-sentence emphasis wrapped
# across a line. So the self-test plants an orphan AND asserts that the
# ordinary heading, the wrapped emphasis and an indented bullet continuation
# are all left alone.
#
# ★ Its one exemption is checked for STALENESS: an entry matching nothing
# fails the gate. `check-strong-text.sh` had carried a carve-out whose premise
# had silently stopped being true, and that is the lesson taken literally.
run "check-orphan-docs" python "$HERE/check-orphan-docs.py"

# check-region-names - the class no compiler can see.
#
# A trace region name is a `pub const`, and `pub` suppresses `dead_code`. So a
# name declared, documented and published by NOTHING compiles clean, passes
# clippy at `-D warnings`, and passes every other gate here — the Export-image
# window carried three such names across four releases.
#
# The symptom is worse than a missing rectangle: a driven check that presses
# `export-image.pages.typed`, finds nothing and reports "the control is
# missing" is describing a radio button drawn on screen every time the window
# opens. The next session goes hunting a layout bug that does not exist.
#
# Its self-test carries eight cases because THREE earlier cuts of this rule
# each looked like a working instrument and were each wrong - by 73, by 9, and
# by 0. The last of those is the one worth remembering: a bare-identifier
# search over the workspace let `export_text.rs`'s healthy `REGION_PAGES`
# discharge `export_image.rs`'s dead twin of the same name.
run "check-region-names" python "$HERE/check-region-names.py"

# ★★★ `check-memory-index`, and it is the gate above's twin
# pointed at a folder nobody thought of as source.
#
# `check-orphan-docs` finds a document nothing links to. This finds a MEMORY
# nothing indexes — and the consequence is worse, because of how agent memory
# is loaded. Only `.claude/agent-memory/<agent>/MEMORY.md` is injected into a
# session. The topic files are read on demand, by a session that learns they
# exist from the index and from nowhere else. A topic file the index does not
# name is therefore not "unlinked"; it is UNREACHABLE. Written, committed,
# reviewed, never read again.
#
# ★★ It was written after finding one. `user_he_is_not_at_the_keyboard_unless
# _he_says_so.md` — the standing rule that decides whether a session drives the
# release binary or defers the work back to the operator — was on disk, in git,
# and named nowhere in the index. It surfaced from a `comm` run done for an
# unrelated reason. **Writing the artifact and registering the artifact are two
# edits, and the second is the one that gets skipped**, because the first is
# where the thinking was. This repository has now recorded that shape under at
# least three other names.
#
# ★ The second rule it enforces is SIZE, and it guards a failure that is
# invisible by construction: the index is truncated from the END when it is too
# large, so the entries that disappear are the NEWEST — exactly the ones a cold
# session needs. Nothing about the loaded text looks wrong; the oldest hundred
# entries are all present. The byte limit is the harness's, not ours, and the
# gate quotes it with its date and source so the next reader re-measures it
# rather than inheriting it.
#
# ⇒ Six sabotages in its self-test, and the CLEAN case is one of them: a gate
# that answered 1 unconditionally would pass five of six.
run "check-memory-index" bash "$HERE/check-memory-index.sh"


# ★★★ `check-doc-markup` — and it is the only gate
# here that asks whether a sentence is VISIBLE rather than whether it is true.
#
# GitHub-flavoured Markdown pads a table row that has FEWER cells than its
# header and **discards** the excess from a row that has more. So a single
# unescaped pipe inside a cell does not shift the layout — it deletes every
# character after the header's last column boundary, for the reader, while the
# file on disk stays complete and the editor shows an ordinary line.
#
# ★★ Measured the day it was written: FIVE rows in four of this project's
# own documents were being truncated, including the whole superseded stack of
# `FEATURES.md`'s Source row — about two thousand characters — and the entire
# *which side moved, and why* cell of a `RIBBON_IA.md` row, which is the
# column that row exists for. `FEATURES.md` ships inside the release zip.
#
# ★ Four of the five were broken by **a document quoting a command or a
# literal that contains a pipe**: a shell pipeline, the PDF flag pair `Print`
# and `NoZoom`, a Rust closure parameter, another table row. That is the same
# cause as a `Source` row broken by the `xargs` invocation it
# quoted, and it is why the message says *escape the pipe* rather than
# *reword it* — the quotation is usually the point.
#
# Its second mechanism is a `**` that can neither open nor close because it
# has whitespace on the wrong side — what a hand-wrapped bold heading leaves
# behind, and what no renderer will ever complain about.
#
# ⇒ Deliberately silent on a row with FEWER cells than its header: that
# renders correctly, and a gate that reports correct files is a gate that gets
# carved out until it means nothing. The asymmetry IS the finding.
run "check-doc-markup" python "$HERE/check-doc-markup.py"

# ★★★ `check-patch-residue` - the gate that audits the
# TOOL that writes this repository, rather than anything the repository says.
#
# Nearly every source edit here is applied by a short Python script. Two of
# those scripts' habits have silently corrupted committed source: a helper that
# translates an ASCII marker into a star does not know what a word is, and a
# backslash is eaten by a quoted heredoc, eaten again by a non-raw string, and
# not decoded at all by a raw one.
#
# ★★ Measured the day it was written: the word ASSERTION had been shipping
# as `A<star><star>ERTION` in a ui-verify check, and STALENESS as
# `STALENE<star><star>` in a canvas module, for days. Both compile. Both pass
# fmt, clippy, every test and every other gate in this file, because the damage
# is inside prose and no machine downstream has an opinion about prose.
#
# The undecoded-escape half is checked in `.rs` ONLY. The same sequence is
# correct in Python - it is how the patch scripts spell their own markers - and
# ambiguous in Markdown, which may be quoting Python. Rust is the one language
# walked in which it cannot be right.
run "check-patch-residue" python "$HERE/check-patch-residue.py"

# ★★★ `check-gate-input-scope` — the gate that audits the
# other gates' INPUT SETS, because the same defect has now been written four
# times by people who had read the warning.
#
# The mechanism: a checker that asks git which files exist is asking about the
# INDEX. A file written and not yet added is invisible to it — and that is
# every file the current session wrote. So the suite goes green, the commit
# lands, and the same tree is red afterwards with nothing edited.
#
# ★★ The four instances, in order:
#   1. `tools/check-suite-name-absent.py` — paid for with a red CI run, then
#      wrote the correct generalisation into its own docstring.
#   2. `check-old-name-absent.sh` — an untracked file under `evidence/`. Repaired
#      by excluding that one directory: the instance treated, the mechanism left.
#   3. the same gate again — this suite reported 41 of 41 green, the commit
#      added two documents, and `package-portable.py`'s pre-flight failed the
#      SAME tree half an hour later.
#   4. `check-doc-markup.py` — found by audit, repaired the same day. It had
#      never once fired, because the .md most likely to carry the defect it
#      hunts is the one just written.
#
# ⇒ **A lesson in a docstring is not an instrument.** Instance 1 recorded the
# generalisation BEFORE instances 2, 3 and 4 were written. Nothing swept for the
# pattern, so nothing found it. This file is that sweep.
#
# The question it makes every author answer: *which side of `git add` does this
# check's subject live on?* A gate about what a reader sees, or what ships, or
# what is on disk, wants the working tree. Only a gate about what has been
# RECORDED wants the index — and then it says so, on the call's own line or the
# one above, with `gate-input-scope-exempt: <reason>`. Two gates read the engine
# at a revision deliberately and are exempt for that reason, which is the
# correct use of the marker rather than a loophole in it.
#
# ★ Falsify this class in ONE step: plant the violation in an **untracked**
# file. A gate with the hole cannot see one at all, so the planted defect is
# reported by a sound gate and invisible to a broken one. That is also why this
# gate walks `tools/` with `os.walk` and never asks git anything — an auditor
# carrying the defect it audits is worthless.
run "check-gate-input-scope" python "$HERE/check-gate-input-scope.py"

# ★★★ `check-test-temp-paths` — and it is here, beside
# `check-gate-input-scope`, because it is the same species: a defect whose
# correct generalisation was already written into this tree, in a comment,
# beside a fix that handled half of it.
#
# `protect::tests` and `dialogs::protect::tests` each carried a confident note
# saying a per-caller tag solved the parallelism hazard. It does — for THREADS.
# Two `cargo test` PROCESSES have the same callers as each other, so they ask
# for the same filenames under `%TEMP%`, and eleven sites in this tree did
# that. Two overlapping workspace runs are enough to turn
# `protect::…::changing_the_password_keeps_what_the_document_allowed` red while
# it passes alone.
#
# ★ The reason it needs a gate rather than a fix: the symptom is an unrelated
# assertion failing several lines downstream, in whichever process lost the
# race, which reads as a regression in that feature and then as a flake when it
# passes on re-run. Nothing about the presentation points at shared state, so
# the class comes back the moment someone writes the twelfth site.
#
# It prints how many sites it audited even when clean, because a pattern that
# stops matching prints what a clean run prints — and it FAILS on zero sites
# for the same reason.
run "check-test-temp-paths" python "$HERE/check-test-temp-paths.py"

# `check-verb-coverage` fails when `pdfcer-core` has a verb this shell names
# nowhere AND `EDITABLE_SURFACES.md` says nothing about it either.
#
# ⚠ The hazard it closes: the engine can answer a request from this shell, with
# a reply saying in as many words *"please check your own copy"*, and nothing on
# this side reads it. `EditSession::set_button_action` sat unconsumed while the
# Button tool stayed greyed and its dialog told the operator that pdfcer
# "cannot give a button something to do yet" -- false, on a capability two open
# operator rows were waiting for, and found only because `tools/verb-
# coverage.py` was run for an unrelated reason.
#
# **The instrument existing is not the same as the instrument being run**, and
# it is the same lesson `check-string-gaps` carries: a convention held by
# memory fails, and the replacement is never a note.
#
# It found five more the moment it worked -- three attachment-clipboard verbs
# and two cut verbs -- none of which had a sentence anywhere.
run "check-verb-coverage" bash "$HERE/check-verb-coverage.sh"

# ★★★ `check-completeness-tests`, and it is the SIXTH time
# this defect was found and the first time an instrument was built for it.
#
# A test named `every_unit_is_named_distinctly` promises, in its name, to go red
# when a unit arrives without a label. On the morning this was written the engine
# shipped three new units and that test stayed green — because it iterated a
# HAND-WRITTEN array of the six variants that existed the day it was typed. What
# actually caught the three unlabelled units was an exhaustive `match` in the
# function above it: the compiler, not the test.
#
# ★★ **A completeness test that carries its own copy of the set is testing the
# copy.** It is invisible for exactly as long as the set is stable, which is
# exactly as long as nobody needs it. The lesson had been written into agent
# memory five times under five different incidents; the sixth paid for the gate.
#
# ★ The predicate is deliberately narrow, and the first draft is the reason.
# Aimed at "any literal array inside a completeness-named test" it returned 53
# hits, most of them lists of test INPUTS — widths, drag corners — and a gate
# that fires on those teaches people to write exemptions, which is how a gate
# becomes scenery. It now requires three or more elements to be `Type::Variant`
# paths sharing one `Type`. That is not a list of inputs; that is a private copy
# of an enumeration.
#
# ★ `completeness-snapshot.txt` is a DEBT REGISTER, not an exemption list, and
# the run prints the outstanding number every time so it stays in front of
# whoever reads it: 29 sites, 9 of them copying a type this repository does not
# declare. The foreign nine are the severe ones — an engine enum grows on a
# branch pin that moves without a `cargo update`, and nothing on this side is
# touched on the day it happens.
#
# ⇒ Both directions are red: a site missing from the register (the debt grew)
# and a register line matching nothing (the register stopped describing the
# tree). The second is the `check-strong-text.sh` lesson taken literally — a
# carve-out whose premise had quietly stopped being true.
run "check-completeness-tests" python "$HERE/check-completeness-tests.py"

# ★★★ `check-forwarded-features` — and it is `check-verb-
# coverage`'s twin one layer down.
#
# That one asks *"is there an engine VERB nothing here calls?"*. This one asks
# the question underneath it: *"is there an engine CAPABILITY that is not even
# COMPILED IN?"* — because a verb inside a stripped Cargo feature does not
# exist to be called, and `verb-coverage.py` cannot tell that from a verb this
# shell has simply not reached yet.
#
# It was written the day the whole digital-signing subsystem was found missing
# from the binary: `pdfcer-core`'s `signing` is default-on, this manifest takes
# the crate with `default-features = false`, and forwarding `jpx` and `ocrs`
# and not `signing` removed 101 public items with nothing failing anywhere.
# The identical omission had removed JPEG 2000 three weeks earlier, and the
# COMMENT warning about that incident was forty lines above the block that
# repeated it. ⇒ A warning does not protect a code path written after it.
run "check-forwarded-features" bash "$HERE/check-forwarded-features.sh"

# `check-old-name-absent` guards the project rename, and it exists because a
# rename is exactly the operation whose completeness cannot be
# checked by the obvious means: `pdfcer` CONTAINS the old stem, so a naive grep
# matches every correct occurrence as well as every stale one and returns
# thousands of hits on a clean tree. The gate uses the only honest pattern --
# the stem not followed by `r` -- and carries a written reason for each of the
# references that legitimately survive. * It checks its own scan's EXIT STATUS,
# because a scan that fails to run reports `clean` exactly like a scan that
# found nothing -- the mechanism, not the intention.
#
# ** A TEMPORARY SHIM NEEDS A TRIPWIRE THAT NAMES ITS OWN DELETION -- and
# a comment is not one. The `package = "pdfce-*"`  # old-name-exempt: naming the retired shim key is the explanation
# bridge in the GUI manifest was a shim to an engine that had not renamed yet;
# its gate's whole job was to fail the build the moment the shim outlived its
# cause, and to be deleted along with the shim when it did. That is what a
# tripwire is for, and why none of this survives as a running gate.
# Its `--self-test` plants in the REAL TREE, because `git grep` is where this
# gate looks and a fixture elsewhere would exercise none of it. Both arms are
# asserted: a stale name is caught, and a stale name sharing a line with an
# exempt substring is NOT -- the cost of a line-oriented filter, asserted so
# that it is a cost somebody can see rather than a surprise somebody finds.
run "check-old-name-absent --self-test" bash "$HERE/check-old-name-absent.sh" --self-test
run "check-old-name-absent" bash "$HERE/check-old-name-absent.sh"

# ★★★ `check-engine-backlog` — `check-verb-coverage`'s twin
# for the channel that had no gate at all.
#
# `check-verb-coverage` reads the engine's API and fails when this shell names
# none of a verb. It caught `set_encryption` and `set_permissions` within hours
# of their arrival, unannounced.
#
# ★★ A capability announced in PROSE has no such gate, and the hole is wide
# enough to swallow an operator request whole: he can ask the ENGINE directly
# for a feature, the engine can ship it the same day with a note saying exactly
# what to wire, and **this shell can build none of it and file no row** — found
# only because a session read the request folder looking for something else.
#
# The engine's own `docs/FEATURES.md` states the gap in a machine-readable
# place — every row reading `[x] core` / `[ ] gui` — and until this gate nothing
# on this side read it. `ENGINE_BACKLOG.md` accounts for all ninety, each with a
# verdict and an argument; this fails when a ninety-first appears.
#
# ★ It also found the column is stale in the OTHER direction: 64 of the 90 are
# already shipped here. That correction went back to the engine as a request
# rather than being fixed locally, because `D:\Dev\pdfcer\` is read-only to us.
run "check-engine-backlog --self-test" bash "$HERE/check-engine-backlog.sh" --self-test
run "check-engine-backlog" bash "$HERE/check-engine-backlog.sh"

# ★★★ `walk-engine-backlog --check` — and the delay is
# the finding.
#
# The gate above asks whether `ENGINE_BACKLOG.md` ACCOUNTS for every capability
# the engine says it has. This one asks whether the register is internally
# honest: that its five section headings equal a real walk, and that no row has
# grown past the 1,200-character cap that keeps a verdict from turning into an
# essay.
#
# Both rules already had this checker. `RESUME.md` names it for both.
# `ENGINE_BACKLOG.md` names it in all five heading comments. **Nothing ran it**,
# because it was never registered here — and the header of
# `check-engine-backlog.sh`, in this same directory, states exactly what that
# means: *"A gate nobody runs is a gate that does not exist."* That sentence was
# written about itself, it was acted on for itself, and its twin was left out.
#
# ★★ What the unregistered checker was hiding, found the first time it was run:
#
#   * **25 rows filed under `wanted` whose own cells opened `✅ WIRED`** — the
#     section that tells a reader *"these are the rows to read if you are
#     choosing what to build next"* read **70** where the real gap was **48**.
#     22 were moved; 3 stay for stated reasons.
#   * **One row 3,856 characters long** against the 1,200 cap — written that
#     morning, by the session that added this line.
#
# ★ It is `--check`, so a disagreeing heading or an over-long row FAILS rather
# than printing and exiting zero. The report of rows whose cell says consumed in
# a section that says otherwise is deliberately NOT a failure: a partly-consumed
# row may legitimately open with a tick, and a gate that forced those to move
# would teach people to re-baseline it.
run "walk-engine-backlog" python "$ROOT/tools/walk-engine-backlog.py" --check

# ★★★ `check-backlog-verdict-drift` — the blind spot the two
# gates above both document and neither measures.
#
# `check-engine-backlog` fails when a capability is discussed NOWHERE; it does
# not judge the verdict. `walk-engine-backlog` counts a row by the section it
# SITS IN, never by what its body states. Both prove a capability is accounted
# for. Neither proves the account is TRUE — so a row can say `wanted` for weeks
# after the shell shipped the verb, and a register wrong in the SHIPPED
# direction is worse than one that is incomplete: it schedules work already
# done, and under-reports the program to the person who paid for it.
#
# ★★ This one falsifies the absence. For every row under `wanted` or `blocked`
# — the only two verdicts that assert an absence — it takes the engine symbols
# named in the row's first cell and looks for them in `crates/pdfcer-gui/src`.
# A hit means the source contradicts the register.
#
# ★ The hit rule, and why it is shaped this way. `.ident` counts a METHOD CALL
# **and a FIELD READ** — `SignReport::appearance_lines` is consumed as a field,
# which is exactly how that row stayed `wanted` after it shipped. `::ident`
# counts a qualified path. A bare identifier counts only inside a file that
# imports it from an engine crate, because an absence claim has to name the
# RECEIVER: every `page_objects` hit in this repository is the shell's own
# `OpenDoc::page_objects`, never the engine's. Comment lines are excluded, and
# identifiers under six characters are skipped as namesakes.
#
# ★ Scope is a claim, so it is written here to be checked: `crates/pdfcer-gui/src/**/*.rs`
# only. NOT `tools/ui-verify` — a driven check is not operator reach. NOT
# `crates/egui-shell` — R7 means it cannot name an engine symbol at all. NOT
# `tests/` — `split_text_object` is the live example, called from a probe test
# and correctly still `wanted`.
#
# ★ An exemption is per SYMBOL and never per row: `<!--namesake:IDENT-->` with
# the reason beside it. A row-wide mute would hide the day that row's OTHER
# symbols get wired. The exempt count prints on a clean run, because an
# exemption nobody can see is a rule quietly narrowing itself.
run "check-backlog-verdict-drift --self-test" python "$ROOT/tools/check-backlog-verdict-drift.py" --self-test
run "check-backlog-verdict-drift" python "$ROOT/tools/check-backlog-verdict-drift.py"

# ★★★ `check-engine-api-drift` — and it exists because the
# two gates immediately above are BLIND TO THE SAME THING.
#
# `pdfcer_core::text_edit::RefusalKind`, a coarse discriminant over
# `EditError`, arrived in direct answer to a request this project filed — and
# sat pinned and unconsumed beside
# `crate::text::status::edit_declined_by_engine`, whose own doc comment said it
# was "written to be deleted" the day `EditError` gained a coarse kind. ⚠ **A
# comment naming the condition of its own deletion does not observe that
# condition.**
#
# ★★ Neither gate above could have noticed. `check-verb-coverage` reads
# `impl EditSession`'s `pub fn`s; `check-engine-backlog` reads the engine's
# prose feature table. **Both are keyed on EditSession's verbs**, so a new
# TYPE, a new VARIANT, a new FIELD on an existing type and a new FREE FUNCTION
# are invisible to the pair. Each previous fix widened the key by one notch and
# left the next notch uncovered.
#
# This one does not pick a notch: it enumerates every `pub` item — 6,868 of
# them — in every engine crate the manifest names, at the revision `Cargo.lock`
# pins, and diffs against a committed snapshot. New, unconsumed and unwritten-
# about is RED.
#
# ★ It found two things on its first real run. `EncryptError::RedactionPending`
# — the Pass 250.3 refusal that says a deferred redaction is still armed — has
# no arm in `protect/mod.rs`'s flattening and reaches the operator classed as a
# WRITER error, while that enum's own doc comment claims a new engine variant
# "turns into a compile error here"; it cannot, because `#[non_exhaustive]`
# makes the catch-all mandatory. And `RenderPolicy::stroke_display` is the
# engine-internal mirror of the field the O137 track wired. Both carry a
# written verdict in the snapshot, printed on every run.
#
# ★★ ONE dispatch line, not two, and deliberately: the gate runs its own
# self-test before it will measure anything. The ordering guarantee lives in
# the gate rather than in this file, so it cannot be lost by an edit here —
# which is the defect the "off by one, then by two" paragraph at the top of
# this file records.
run "check-engine-api-drift" bash "$HERE/check-engine-api-drift.sh"
# ★★★ `check-ui-toolkit-drift`, and it is the sibling of
# the line above. `check-engine-api-drift` watches `pdfcer-core`, a PATH
# dependency that moves hourly and cannot move unnoticed. This one watches
# `egui`, a VERSIONED dependency that can ONLY move unnoticed — a caret
# requirement of `"0.35"` resolves `<0.36.0`, so `cargo update` reports
# everything current while a whole minor release goes by. The operator
# asked "does our project check for the latest version of egui to compile
# with?" and the honest answer was no. It is a report, not a policy: being
# behind is fine, being behind without knowing is what fails here.
run "check-ui-toolkit-drift" bash "$HERE/check-ui-toolkit-drift.sh"

# ★★★ `check-unreachable-refusals`, and it is the FOURTH
# member of the drift family -- watching the one kind of drift the other
# three structurally cannot see: a symbol that still EXISTS on the engine
# side, with an unchanged signature, that has quietly stopped being produced.
#
# `check-engine-api-drift` enumerates every public item the engine GAINS. A
# variant whose last constructor was deleted gains nothing, loses nothing
# public, and is invisible to it. `check-pin-citation` watches that documents
# quote the right pin, not what the pin means. And `cargo` is perfectly happy:
# the arm still compiles, the test still passes, the sentence still renders.
#
# This shell keeps such sentences on purpose -- the `match` over
# `ReflowDecline` is compiler-proved complete so the arm is MANDATORY, and
# `unreachable!()` would turn a future engine reinstating the guard into a
# crash on a refusal path. What was missing is any way to tell they are dead.
#
# ★★ Three times paid for. `PageAlreadyEdited`'s doc argued for a shell
# forecast deleted nine days earlier; `PageSetChanged` went unreachable at the
# engine's `Pass 257.0` and was found months later by a reader looking for
# something else; and on the morning this gate was written `G015` deleted the
# only producer of `ReflowApplyError::PageEditedThisSession` and said so in
# its own commit message -- in a repository this one is forbidden to write to
# and nobody is obliged to read.
#
# ★ It DIFFS rather than classifies. Deciding which of five lines mentioning
# a variant is a constructor needs a parser, and a regex that guessed would be
# confidently wrong on `matches!`, on `Err(X::Y) =>` and on a doctest. The
# gate's claim is the smaller, honest one: the engine's code around a symbol
# somebody wrote a "this cannot happen" paragraph about has moved. A human
# reads the paragraph.
#
# Its --self-test runs inside the wrapper, for the reason stated there.
run "check-unreachable-refusals" bash "$HERE/check-unreachable-refusals.sh"

# ★★★ `check-settings-funnel`, the FIFTH member of that
# family, and it watches the drift that runs the other way. The four above
# ask what the ENGINE did that we have not noticed. This one asks what the
# engine already offers that we accept from the operator and then throw
# away.
#
# It found six on its first run. `page_blend_space_source`,
# `overprint_zero_tint_scope`, `spot_colorant_device_model` and
# `mesh_patch_padding` had controls in Settings > Colour, were written to
# the settings file, were read back on the next launch, and were never
# chained onto `RenderOptions`. `widget_tab_tail` and `tab_row_tolerance`
# had controls in Settings > Forms and never reached an `EditSession`, so
# an operator who widened the tab row tolerance got the engine default in
# the very tab ring he set it for.
#
# ★★ There WAS a guard, and it is a good one keyed on the wrong half:
# `no_call_site_builds_its_own_options` forbids constructing an options
# struct outside the funnel, which proves nobody BYPASSES it and says
# nothing about whether the funnel ASSIGNS every field. What was supposed
# to cover the gap was prose, and the prose was three separate counts --
# "thirteen operator choices" for a struct of twenty-three, "Five
# settings" for a chain of six, and "fifteen setters ... and nothing else"
# for twenty-nine. All three were written as measurements and all three
# were wrong, because no count in a comment can fail a build.
#
# ★ The oracle is `Cargo.lock`, not the engine working tree: the operator
# can only set a setting that exists in the crate we compile against. A
# revision absent from the clone is the gate’s ONLY skip.
run "check-settings-funnel" python "$HERE/check-settings-funnel.py"

# ★★★ `check-pin-citation`, and it is the THIRD member of
# the drift family above -- but it watches a different kind of drift, and the
# difference is the reason it exists.
#
# The two gates above ask whether the CODE is current. This one asks whether
# the DOCUMENTS agree with the code about which engine commit was consumed.
# `FEATURES.md` is the file that ships to the operator; its revision header
# names the pin by hand, in seven characters, typed once at the moment a
# human happened to look. `cargo update` then rewrites `Cargo.lock` silently
# and correctly, and the header becomes a confident lie with nothing in a
# position to notice -- because THE FILE THAT CHANGED DOES NOT CONTAIN THE
# NUMBER THAT WENT WRONG.
#
# It was written after the eighth recorded instance of that shape in this
# repository, and it caught a live one on its first run: the header said
# `d2465f5`, measured at 19:52, while the lock had moved to `01c4a10` at
# 22:01 and was about to move again. The previous seven were all corrected
# by hand after somebody noticed, which is luck with a commit message.
#
# It matters more than an ordinary stale number for one specific reason:
# when the operator reports a defect, the first question back is always
# *which build and which engine pin*, and the answer he reaches for is the
# one in the document that shipped beside the exe.
#
# ★ Absence is RED here, never green and never a skip. If a future rewording
# drops the phrase from the header, this gate fails and says so in those
# words -- a gate keyed on a name is otherwise discharged by prose that
# stops using the name, which this repository has been bitten by before.
run "check-pin-citation" bash "$HERE/check-pin-citation.sh"
# And its own self-test, in the same file as the line above it, because
# writing a --self-test is half the work and registering it is the other
# half. Six sabotages, including the one case the gate is required to stay
# QUIET about: a retained older revision header legitimately quoting an
# older pin, which a naive whole-file grep would fail on for ever.
run "check-pin-citation --self-test" bash "$HERE/check-pin-citation.sh" --self-test

# `check-engine-citation` asserts the OTHER half of the same problem. The gate
# above measures whether a document quoting the pin quotes the right one;
# this one measures whether a document cites the engine in a form that can go
# wrong at all. A line number into a branch-pinned dependency rots with no
# local event -- no cargo update here, no edit here -- and it rots into
# readable prose about a real function rather than a dangling reference, which
# is why it survives review. On the sweep this was written for, 66 of 92
# citations had drifted and two pairs had come to name each other's type.
#
# The archived GUI is deliberately IN scope: a frozen tree is not a
# frozen-correct one, because its citations kept drifting until the freeze, so
# what froze was the error.
run "check-engine-citation" bash "$HERE/check-engine-citation.sh"
# Four detectors, four sabotages, plus the case it must stay QUIET about -- a
# version-anchored vendor path, which cannot move without a Cargo.toml bump --
# and the case where it must SKIP rather than pass: D3 and D4 borrow
# check-file-size's invariant, so a tree carrying an over-limit .rs has
# falsified their premise and a clean report would not be earned.
run "check-engine-citation --self-test" bash "$HERE/check-engine-citation.sh" --self-test

# `check-third-party-licences` regenerates THIRD_PARTY_LICENSES.md and fails if
# the committed one differs. It is a second gate for the same underlying shape
# as `check-verb-coverage`: an ADDITION on the other side
# of a boundary is silent, because a removed dependency breaks the build and an
# added one does not.
#
# The release that morning pulled in three MIT crates with a colour-management
# engine, and the attribution file shipped beside the exe named none of them. It
# was caught because those three "Adding" lines happened to be in output being
# read for an unrelated reason.
#
# ★ It costs about a minute -- cargo-about resolves the whole graph per target
# -- which is why it is last in this section rather than first.
run "check-third-party-licences" bash "$HERE/check-third-party-licences.sh"

# ★★★ `package-portable --self-test` — and, exactly like
# `walk-engine-backlog` eight hours earlier, the delay IS the finding.
#
# `tools/package-portable.py` has carried a `--self-test` since it was written.
# It asserts things nothing else can see: that a build folder name cannot be
# swallowed by pdfcer's OWN packager's `pdfcer-*` glob, that the source digest
# is deterministic and moves on a renamed file, and that the asset-copy loop
# works even though `PAYLOAD_ASSET_DIRS` is empty so it never runs in a real
# package. **Nothing ran it.** It was reachable only by a session that happened
# to type the flag, which is to say by a session that already suspected
# something.
#
# ⇒ That is the same shape twice in one day, and the shape is not "somebody
# forgot". It is that a self-test is written by the person fixing a bug, at the
# moment the bug is fresh, and registering it is a SEPARATE file's edit. The
# rule this repository now holds: a `--self-test` that is not in this list does
# not exist, and adding one means editing two files or neither.
#
# ★ What registering it now protects. A fourth invariant was added the same
# hour: the GitHub release asset must be ROOTED AT THE BUILD FOLDER. For the
# first eleven releases the zip was made by hand at publish time, and the
# difference between right and wrong is one argument to `shutil.make_archive` —
# an archive of loose contents is a perfectly valid archive that scatters an exe
# and eight documents across whatever directory the operator was standing in,
# reports no error, and is discovered by him rather than here.
#
# It is cheap: three temporary directories and a 1 KB zip, well under a second,
# and it touches neither `D:\Dev\pdfcer` nor OneDrive.
run "package-portable --self-test" python "$ROOT/tools/package-portable.py" --self-test

# --- 2. cargo fmt / clippy --------------------------------------------------
#
# Both are wrapped in a workspace-loadability probe. If a member crate listed
# in the root Cargo.toml has no manifest yet — normal while several agents are
# building different crates — cargo cannot load the workspace at all, and its
# error has nothing to do with formatting or lints. Reporting that as a fmt
# FAILURE would be a false accusation against whoever is mid-write, so it is
# reported as a SKIP with the real reason.
#
# ★ "cargo is not on PATH" is NOT "the workspace does not load", and the two
# are separated here because merging them produced a flatly false message.
#
# `tools/package-portable.py` runs this script through `subprocess`, and the
# bash it spawns does not inherit the PATH entry for `~/.cargo/bin`. The probe
# below then failed with `cargo: command not found` and this script reported
# "the workspace does not currently load", followed by advice about a member
# crate being mid-write. Every word of that was wrong: the workspace was fine,
# nothing was mid-write, and the reader was pointed at the one place the
# problem was not.
#
# A skip reason is read precisely when someone cannot see the machine. It has
# to name the actual fact.
if [ "$RUN_CARGO" -eq 1 ] && ! command -v cargo >/dev/null 2>&1; then
    rule
    echo ">> cargo fmt / cargo clippy"
    rule
    echo "SKIPPED — cargo is not on PATH in this shell."
    echo ""
    echo "  The workspace is not implicated: nothing was parsed, because the"
    echo "  tool that would parse it was never found. If this ran from a script,"
    echo "  the spawned shell probably did not inherit ~/.cargo/bin."
    echo ""
    SKIPPED+=("cargo fmt (cargo not on PATH)")
    SKIPPED+=("cargo clippy (cargo not on PATH)")
elif [ "$RUN_CARGO" -eq 1 ]; then
    if ! probe=$(cargo metadata --no-deps --format-version 1 2>&1 >/dev/null); then
        rule
        echo ">> cargo fmt / cargo clippy"
        rule
        echo "SKIPPED — the workspace does not currently load:"
        printf '%s\n' "$probe" | sed 's/^/  /' | head -20
        echo ""
        echo "  This is expected while a member crate is being written. Neither fmt"
        echo "  nor clippy can say anything about a workspace cargo cannot parse,"
        echo "  and calling that a formatting failure would blame the wrong file."
        echo ""
        SKIPPED+=("cargo fmt")
        SKIPPED+=("cargo clippy")
    else
        run "cargo fmt" cargo fmt --all --check
        run "cargo clippy" cargo clippy --workspace --all-targets -- -D warnings
    fi
else
    SKIPPED+=("cargo fmt (--no-cargo)")
    SKIPPED+=("cargo clippy (--no-cargo)")
fi

# ---------------------------------------------------------------------------
# SUMMARY
# ---------------------------------------------------------------------------
rule
echo "SUMMARY"
rule
for g in "${PASSED[@]:-}";  do [ -n "$g" ] && echo "  PASS     $g"; done
for g in "${SKIPPED[@]:-}"; do [ -n "$g" ] && echo "  SKIPPED  $g"; done
for g in "${FAILED[@]:-}";  do [ -n "$g" ] && echo "  FAIL     $g"; done
echo ""
np=${#PASSED[@]}; nf=${#FAILED[@]}; ns=${#SKIPPED[@]}
echo "  $np passed, $nf failed, $ns skipped"
echo ""

if [ "$nf" -gt 0 ]; then
    echo "RESULT: FAIL — $nf gate(s) found a violation."
    exit 1
fi
if [ "$ns" -gt 0 ]; then
    echo "RESULT: INCOMPLETE — nothing failed, but $ns gate(s) never ran."
    echo ""
    echo "  This is NOT a pass. A gate whose precondition was absent has told you"
    echo "  nothing, and 'told you nothing' printed as green is the exact defect"
    echo "  PROJECT_PLAN.md §4.1 exists to remove. Read each SKIPPED reason above"
    echo "  and decide whether it is expected."
    exit 3
fi
echo "RESULT: PASS — every gate ran and every gate is clean."
exit 0
