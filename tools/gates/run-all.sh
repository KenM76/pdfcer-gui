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

# --- 1. the gates themselves ------------------------------------------------
run "check-ui-strings"   bash "$HERE/check-ui-strings.sh"
run "check-theme-colors" bash "$HERE/check-theme-colors.sh"
# ★ DEFECTS.md D11, mechanised. The rule was written on 2026-08-14 and broken
# again on 2026-08-17 by someone who had read it; a rule that lives only in a
# document is enforced as often as somebody remembers to read it.
run "check-strong-text" bash "$HERE/check-strong-text.sh"
# ★★★ `check-plate-colour`, added 2026-09-03 — DEFECTS.md D2 for the third
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
# against the tree with the two 2026-09-03 fixes reverted, it names exactly the
# three defective drawing sites and passes the four correct ones.
run "check-plate-colour --self-test" bash "$HERE/check-plate-colour.sh" --self-test
run "check-plate-colour" bash "$HERE/check-plate-colour.sh"
run "check-file-size"    bash "$HERE/check-file-size.sh"
run "check-shell-purity" bash "$HERE/check-shell-purity.sh"
run "check-shipped-assets" bash "$HERE/check-shipped-assets.sh"
run "check-string-gaps"  bash "$HERE/check-string-gaps.sh"
# ★★★ The gate of 2026-09-02, and it is the first one aimed at PROSE being
# false rather than at code being wrong. `reorder_annotations` shipped hours
# after the request for it, and four separate places went on asserting the gap:
# an on-screen explainer, a module header that FORBADE the feature, a passing
# test that would have failed it, and a nineteen-day-old FEATURES row. All four
# were correct when written, which is why nothing looked wrong and no other gate
# could see them. This catches the one mechanical part — a row that says BLOCKED
# and names a request the channel shows we have CONSUMED.
run "check-stale-blockers" bash "$HERE/check-stale-blockers.sh"
# ★ The two gates of 2026-08-20, both born of an operator report rather than of
# a design. `check-typing-guard` keeps "is the operator typing?" a single
# predicate, after the space bar was stolen by the pan tool for a fortnight;
# `check-conventions` makes every interactive surface answer, row by row, the
# conventions its gesture class carries - because every convention he had to
# report was one nobody had ASKED about, not one somebody decided against.
run "check-typing-guard" bash "$HERE/check-typing-guard.sh"
run "check-conventions"  bash "$HERE/check-conventions.sh"
# ★ The gate of 2026-08-21, born the same way and from the same failure mode as
# `check-typing-guard`: a finding was recorded in one module and never applied to
# its siblings. Ctrl+C/X/V never arrive as key events - egui-winit intercepts
# them - so `key_pressed(Key::C)` is permanently false in a real window. That was
# written up in capitals in `app::keyboard` on 2026-08-20, and the identical
# mistake sat one grep away in `canvas::textsel::clipboard` for a further day,
# certified green by tests injecting the key event winit never sends.
run "check-clipboard-chords" bash "$HERE/check-clipboard-chords.sh"

# `check-suite-name-absent` keeps a LICENSED print-conformance suite's name out
# of this repository entirely -- contents and file names -- per the operator's
# ruling of 2026-08-25. Carried across from `D:/Dev/pdfcer/tools/`, where it was
# written, rather than re-derived: it already carries the fixes for two defects
# that a careful reader makes anyway.
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
run "check-trace-names" python "$HERE/check-trace-names.py"

# `check-verb-coverage` fails when `pdfcer-core` has a verb this shell names
# nowhere AND `EDITABLE_SURFACES.md` says nothing about it either.
#
# The two days that bought it: `EditSession::set_button_action` shipped
# 2026-08-30, in answer to this shell's own request, with a reply that said in
# as many words *"please check your own copy."* It was consumed 2026-09-01, and
# only because `tools/verb-coverage.py` was run for an unrelated reason. In
# between, the Button tool stayed greyed and its dialog told the operator that
# pdfcer "cannot give a button something to do yet" -- false, on a capability
# two open operator rows were waiting for.
#
# The instrument existed. Nobody ran it. That is the whole lesson, and it is
# the same one `check-string-gaps` learned: a convention held by memory fails,
# and the replacement is never a note.
#
# It found five more the moment it worked -- three attachment-clipboard verbs
# and two cut verbs -- none of which had a sentence anywhere.
run "check-verb-coverage" bash "$HERE/check-verb-coverage.sh"

# `check-old-name-absent` is what the 2026-09-03 rename left behind, and it
# exists because a rename is exactly the operation whose completeness cannot be
# checked by the obvious means: `pdfcer` CONTAINS the old stem, so a naive grep
# matches every correct occurrence as well as every stale one and returns
# thousands of hits on a clean tree. The gate uses the only honest pattern --
# the stem not followed by `r` -- and carries a written reason for each of the
# references that legitimately survive. * It reported `clean` on its own first
# run while its scan was failing; it now checks the scan's exit status, which is
# the mechanism rather than the intention.
#
# ** ITS SIBLING, `check-engine-rename-shim`, WAS DELETED ON 2026-09-03 -- by
# its own instruction, and that is the point of it. The `package = "pdfce-*"`  # old-name-exempt: naming the retired shim key is the explanation
# bridge in the GUI manifest was a temporary shim to an engine that had not
# renamed yet, and the gate's job was to fail the build the moment the shim
# outlived its cause. The engine's `Pass 247.1` landed mid-session (`4db298d`,
# engine v0.28.0), the gate fired, the shim came out, and the gate went with it.
# A temporary shim needs a tripwire that names its own deletion; a comment is
# not one, and this one worked exactly as written.
run "check-old-name-absent" bash "$HERE/check-old-name-absent.sh"

# `check-third-party-licences` regenerates THIRD_PARTY_LICENSES.md and fails if
# the committed one differs. It is the SECOND gate written on 2026-09-01 for the
# same underlying shape as `check-verb-coverage`: an ADDITION on the other side
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
