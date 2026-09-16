#!/usr/bin/env bash
#
# check-scroll-row-wrapping.sh — no plain `ui.horizontal` inside the print dialog.
#
# ═══════════════════════════════════════════════════════════════════════════
# WHAT THIS GATE IS FOR
# ═══════════════════════════════════════════════════════════════════════════
#
# `ui.horizontal` does **not** clamp to its available width. It lays its
# children out past the end of the column it was given and reports a
# `min_rect` that wide, and that overflow propagates outward as the enclosing
# `ScrollArea`'s content size. In a **fixed-size** window the result is a
# scrollbar the operator cannot dismiss by resizing anything, because both
# sides of the comparison scale together.
#
# `ui.horizontal_wrapped` is bounded by its column *by construction*: there is
# no number to get wrong, and when the row fits it lays out identically. The
# only behavioural difference is that a long label may wrap instead of
# extending past the edge — which is the desired outcome here, not a cost.
#
# ═══════════════════════════════════════════════════════════════════════════
# ★★★ WHY A GATE, AND NOT THE THREE PARAGRAPHS THAT WERE ALREADY THERE
# ═══════════════════════════════════════════════════════════════════════════
#
# This defect has been found by the operator twice, three weeks apart, both
# times reported in the same words: *"two scroll bars that won't go away"*. The
# second instance was in the same file as the first, about a hundred lines
# below it, under a comment block explaining the mechanism in full. That row
# laid out 402.5 pt inside a 340 pt column.
#
# A finding written next to the code does not apply itself. The second
# occurrence of a mechanism is the announcement that it is a class, and a class
# needs an instrument. This is the instrument.
#
# ═══════════════════════════════════════════════════════════════════════════
# SCOPE, WHICH IS A CLAIM
# ═══════════════════════════════════════════════════════════════════════════
#
# It checks `crates/pdfcer-gui/src/dialogs/print/` — a DIRECTORY, so a module
# added there is covered the day it is added and no list has to be maintained.
#
# It does NOT check the panels, the ribbon, or any other scroll area, and that
# is deliberate rather than an omission waiting to be fixed. The distinguishing
# property is not "is inside a scroll area" but "is inside a window whose size
# the operator cannot change": a dock panel that overflows raises a bar the
# operator dismisses by widening the panel, while this dialog inherits its
# declared size and the bar is permanent. Crate-wide the plain form appears
# about 145 times; a gate over all of them would be a debt register, not a rule.
#
# If another fixed-size window is added, add its directory to `ROOTS` below.
#
# ═══════════════════════════════════════════════════════════════════════════
# THE EXEMPTION, AND WHY IT CARRIES A REASON RATHER THAN A NAME
# ═══════════════════════════════════════════════════════════════════════════
#
#     // scroll-row-exempt: <reason>
#     ui.horizontal(|ui| {
#
# on any of the six lines above the call. A reason, not a file name and not a
# line number: an exclusion list is how the previous generation of gates in
# this repository went quietly blind, and a reason stays legible when the code
# moves. Two rows are legitimately exempt today — the row holding the two
# explicitly sized columns, and the footer, which is drawn outside the body.
#
# ★★ BOTH DIRECTIONS ARE RED. A plain row with no marker fails, AND a marker
# that no longer sits above a plain row fails. Without the second half the
# markers rot into decoration the moment someone converts a row and leaves the
# comment behind, and the gate would then be excusing a site that no longer
# exists while reporting green.
#
# Exit: 0 clean · 1 violation · 2 precondition absent (SKIPPED).
# `--self-test` plants every case in a temporary tree; it must print
# SELF-TEST OK.

set -uo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"

ROOTS=("crates/pdfcer-gui/src/dialogs/print")

# The forms that do not clamp. `horizontal_wrapped` is absent on purpose: it is
# the remedy. `horizontal_centered` and `horizontal_top` differ from
# `horizontal` only in cross-axis alignment and overflow exactly the same way.
BAD_RE='ui\.horizontal(_top|_centered)?\('
MARKER='scroll-row-exempt:'
LOOKBACK=6

# Walks one tree and prints one line per violation. Never asks git anything: a
# gate whose input set is "what is already committed" cannot see the commit
# being made — see check-gate-input-scope.py.
scan() {
    local base="$1"
    local found=0 bad=0 stale=0
    local file line start end

    while IFS= read -r file; do
        found=1

        # Violations: a non-wrapped row with no marker above it.
        while IFS= read -r line; do
            start=$(( line - LOOKBACK ))
            if (( start < 1 )); then start=1; fi
            if ! sed -n "${start},$((line - 1))p" "$file" | grep -q "$MARKER"; then
                printf '  %s:%s  plain horizontal row with no `%s <reason>` above it\n' \
                       "${file#"$ROOT/"}" "$line" "$MARKER"
                bad=1
            fi
        done < <(grep -nE "$BAD_RE" "$file" | grep -v '_wrapped' | cut -d: -f1)

        # Stale markers: an exemption that no longer sits above a plain row.
        while IFS= read -r line; do
            end=$(( line + LOOKBACK ))
            if ! sed -n "$((line + 1)),${end}p" "$file" \
                 | grep -E "$BAD_RE" | grep -qv '_wrapped'; then
                printf '  %s:%s  `%s` marker with no plain horizontal row below it\n' \
                       "${file#"$ROOT/"}" "$line" "$MARKER"
                stale=1
            fi
            # An empty reason is not a reason.
            if sed -n "${line}p" "$file" | grep -qE "${MARKER}[[:space:]]*$"; then
                printf '  %s:%s  `%s` with no reason after the colon\n' \
                       "${file#"$ROOT/"}" "$line" "$MARKER"
                stale=1
            fi
        done < <(grep -n "$MARKER" "$file" | cut -d: -f1)
    done < <(find "$base" -name '*.rs' -type f | sort)

    if (( found == 0 )); then return 2; fi
    if (( bad == 1 || stale == 1 )); then return 1; fi
    return 0
}

# One planted case: write $2 into the temp tree, scan it, compare the exit code.
# Cases live in a function rather than a heredoc chain so the file stays
# editable without nested-heredoc traps.
SELF_FAIL=0
plant() {
    local name="$1" want="$2" body="$3"
    printf '%s\n' "$body" > "$SELF_TMP/a.rs"
    scan "$SELF_TMP" >/dev/null 2>&1
    local got=$?
    if [[ "$got" != "$want" ]]; then
        printf 'self-test: %s expected exit %s, got %s\n' "$name" "$want" "$got"
        SELF_FAIL=1
    fi
}

self_test() {
    SELF_TMP="$(mktemp -d)" || { echo "self-test: cannot make a temp dir"; return 1; }
    trap 'rm -rf "$SELF_TMP"' RETURN

    local plain='fn draw(ui: &mut Ui) {
    ui.horizontal(|ui| { ui.label("x"); });
}'
    local marked='fn draw(ui: &mut Ui) {
    // scroll-row-exempt: children are explicitly sized.
    ui.horizontal(|ui| { ui.label("x"); });
}'
    local reasonless='fn draw(ui: &mut Ui) {
    // scroll-row-exempt:
    ui.horizontal(|ui| { ui.label("x"); });
}'
    local stale='fn draw(ui: &mut Ui) {
    // scroll-row-exempt: children are explicitly sized.
    ui.horizontal_wrapped(|ui| { ui.label("x"); });
}'
    local wrapped='fn draw(ui: &mut Ui) {
    ui.horizontal_wrapped(|ui| { ui.label("x"); });
}'
    local topform='fn draw(ui: &mut Ui) {
    ui.horizontal_top(|ui| { ui.label("x"); });
}'

    plant "unmarked plain row"  1 "$plain"
    plant "marked plain row"    0 "$marked"
    plant "reasonless marker"   1 "$reasonless"
    plant "stale marker"        1 "$stale"
    plant "wrapped only"        0 "$wrapped"
    plant "horizontal_top"      1 "$topform"

    # An empty tree is a PRECONDITION failure, not a pass: a find that walks
    # nothing prints exactly what a clean run prints.
    rm -f "$SELF_TMP/a.rs"
    scan "$SELF_TMP" >/dev/null 2>&1
    if [[ $? != 2 ]]; then
        echo "self-test: an empty tree must SKIP (2), not pass"
        SELF_FAIL=1
    fi

    if (( SELF_FAIL )); then
        echo "SELF-TEST FAILED"
        return 1
    fi
    echo "SELF-TEST OK: catches unmarked, reasonless, stale, and the _top form; skips an empty tree"
    return 0
}

if [[ "${1:-}" == "--self-test" ]]; then
    self_test
    exit $?
fi

status=0
for rel in "${ROOTS[@]}"; do
    base="$ROOT/$rel"
    if [[ ! -d "$base" ]]; then
        echo "SKIPPED: $rel does not exist, so this gate's subject is absent"
        exit 2
    fi
    out="$(scan "$base")"
    rc=$?
    case "$rc" in
        2) echo "SKIPPED: $rel contains no .rs files"; exit 2 ;;
        1) printf 'FAIL: plain horizontal rows in a fixed-size window\n%s\n' "$out"; status=1 ;;
        0) ;;
        *) echo "FAIL: scan returned $rc"; status=1 ;;
    esac
done

if (( status == 0 )); then
    echo "OK: every horizontal row under ${ROOTS[*]} is wrapped or has a stated reason"
fi
exit $status
