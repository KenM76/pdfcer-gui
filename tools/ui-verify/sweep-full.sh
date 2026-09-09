#!/usr/bin/env bash
#
# sweep-full.sh — drive EVERY ui-verify check against a scratch copy of the
# release build, in chunks, and leave one log behind.
#
# ## Why this exists
#
# `ui-verify` takes a `--check NAME` per run and the suite is ~200 checks. One
# invocation naming all of them is a single process that launches, drives and
# tears down the application two hundred times; when it dies half way through —
# a hung window, an OOM, a stray focus steal — the whole run is lost and the log
# ends mid-sentence. Chunking makes a failure cost twenty checks instead of two
# hundred, and gives the log a `rc=` line per chunk so a crash is visible as a
# non-zero code rather than as an absence.
#
# ## The two rules this script exists to hold
#
# 1. **It never drives the published build.** `--exe` points at a copy under
#    `target/scratch/drive/`, never at `target/release/pdfcer-gui.exe` and never
#    at the OneDrive copy. A driven sweep writes preferences, layout and recent
#    files as it goes; run against the operator's own binary it edits the state
#    he comes back to. Copy first — the copy is made below, not assumed.
#
# 2. **The check list is asked for, never written down.** It comes from
#    `ui-verify --list`, so a check added this morning is swept this afternoon
#    with no edit here. A hand-maintained list inside a completeness runner is
#    the exact shape of gap this project has been bitten by: the new module is
#    invisible to the instrument built to find it, and the count still adds up.
#
# ## Usage
#
#   bash tools/ui-verify/sweep-full.sh
#
# Requires `cargo build --release` to have produced both `ui-verify.exe` and
# `pdfcer-gui.exe`. Takes the desktop for the duration — real cursor moves, real
# clicks, real focus changes — so do not start it while the operator is at the
# machine.
#
# ## Output
#
#   target/scratch/sweep-full.log   the whole transcript, chunk-delimited
#   target/scratch/uv-full/         per-check captures and traces
#
# The log ends with `=== SWEEP-DONE`. Its absence means the sweep was killed,
# which is a different fact from "every check passed" and must not be read as
# one.
set -u

cd "$(dirname "$0")/../.." || exit 1

OUT=target/scratch
LOG=$OUT/sweep-full.log
LIST=$OUT/checks.txt
DRIVE=$OUT/drive

mkdir -p "$OUT" "$DRIVE" || exit 1
: > "$LOG"

# Rule 1: drive a copy. `cp` every run, so the copy cannot be an old build
# quietly answering for a new one.
cp target/release/pdfcer-gui.exe "$DRIVE/pdfcer-gui.exe" || exit 1

# Rule 2: ask the harness what it has. `--list` prints a `Checks:` header, then
# a name at two spaces of indent and its description at six, then a `Profiles:`
# section in the same shape; the filter takes exactly the two-space names under
# the first header.
./target/release/ui-verify.exe --list \
    | awk '/^Checks:/ {on=1; next} /^[A-Za-z]/ {on=0} on && /^  [a-z_]+$/ {print $1}' \
    > "$LIST"

N=$(wc -l < "$LIST")
if [ "$N" -lt 1 ]; then
    echo "sweep-full: --list produced no check names; the parse above is wrong" >&2
    exit 1
fi
echo "=== sweeping $N checks" | tee -a "$LOG"

for start in $(seq 1 20 "$N"); do
    ARGS=$(sed -n "${start},$((start + 19))p" "$LIST" | sed 's/^/--check /' | tr '\n' ' ')
    echo "=== chunk $start" >> "$LOG"
    # shellcheck disable=SC2086
    ./target/release/ui-verify.exe --exe "$DRIVE/pdfcer-gui.exe" \
        --pdf fixtures/a1-titleblock.pdf --doc-point 0,2000,320 \
        --second-pdf fixtures/four-pages.pdf --out "$OUT/uv-full" \
        $ARGS >> "$LOG" 2>&1
    echo "=== chunk $start rc=$?" >> "$LOG"
done

# ★ One check needs the pointer somewhere else. `resize_scales_a_shape` aims at
# a shape near the top-left of the title-block sheet, and the sweep's own
# `--doc-point` is chosen for the checks that need a point out in the drawing.
# Re-run it alone rather than move the shared aim, because moving the shared aim
# to suit one check silently re-aims the other nineteen in its chunk.
echo "=== resize aim" >> "$LOG"
./target/release/ui-verify.exe --exe "$DRIVE/pdfcer-gui.exe" \
    --pdf fixtures/a1-titleblock.pdf --doc-point 0,300,500 \
    --check resize_scales_a_shape --out "$OUT/uv-full-resize" >> "$LOG" 2>&1
echo "=== resize aim rc=$?" >> "$LOG"

echo "=== SWEEP-DONE" >> "$LOG"
