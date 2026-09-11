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
# ## The three rules this script exists to hold
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
# 3. **A check whose subject cannot exist in the shared fixture is run against
#    its own document.** The shared `--pdf` / `--doc-point` below is one
#    compromise chosen for the majority. A check that needs a different document
#    does not get to quietly SKIP for ever inside somebody else's chunk: it goes
#    in the `ALONE` table, it is *removed* from the chunked list so it cannot be
#    run twice, and it is re-run with its own arguments. If its name is not in
#    `--list`, this script stops with a non-zero exit rather than silently
#    covering one check fewer than it claims.
#
#    ★ This is the rule that was missing, and the cost was measured on
#    2026-09-11. `pages_stay_drawn_when_you_scroll_back` — the regression test
#    for the operator's own 2026-08-19 report about pages constantly redrawing —
#    had been reporting SKIP in **every sweep since it was written**, because
#    the shared fixture `a1-titleblock.pdf` has one page and a page-strip needs
#    several. The check said so in its skip line, correctly and at length.
#    Nobody read it, **because a SKIP is not red.** The fix is not to read
#    harder; it is to stop producing the skip.
#
# ## The ALONE table
#
# One record per line, `name|extra ui-verify arguments`. Those arguments replace
# the shared `--pdf` / `--doc-point` / `--second-pdf` completely, so each record
# must state everything its check needs in order to see its own subject. Keep
# the reason in the comment block beside the table: a future reader has to be
# able to tell "this check needs eight pages" from "somebody had a failing run
# once and moved it out of the way".
#
# Blank lines and lines whose first character is `#` are ignored, so the table
# can carry its own annotations.
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
#   target/scratch/sweep-full.log    the whole transcript, chunk-delimited
#   target/scratch/uv-full/          per-check captures and traces
#   target/scratch/uv-full-<name>/   captures for each ALONE check
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

# Rule 3: the checks that need their own document.
#
#   pages_stay_drawn_when_you_scroll_back
#       Needs a document with enough pages to form a continuous strip; the
#       shared a1-titleblock.pdf has exactly one, so the check's subject — a
#       page scrolled away and brought back — could never occur. The fixture is
#       8 pages of 306 x 396 pt, so the aim point is a page centre.
#
#   resize_scales_a_shape
#       Aims at a shape near the top-left of the title-block sheet, while the
#       shared `--doc-point` is chosen for the checks that need a point out in
#       the drawing. Moving the shared aim to suit this one would silently
#       re-aim the other nineteen checks in its chunk.
ALONE='
pages_stay_drawn_when_you_scroll_back|--pdf fixtures/synthetic-image-only-8pages.pdf --doc-point 0,150,200
resize_scales_a_shape|--pdf fixtures/a1-titleblock.pdf --doc-point 0,300,500
'

# Take each ALONE name out of the chunked list, and refuse to continue when a
# name is not in it. An entry that no longer matches a check is either a rename
# nobody followed through or a deletion; in both cases the honest outcome is a
# stopped sweep, not a sweep that runs one check fewer than it reports.
while IFS='|' read -r name _args; do
    [ -z "${name:-}" ] && continue
    case "$name" in
        '#'*) continue ;;
    esac
    if ! grep -qx -- "$name" "$LIST"; then
        echo "sweep-full: the ALONE table names '$name', which ui-verify --list" >&2
        echo "            does not offer. Renamed? Deleted? Fix the table; do" >&2
        echo "            not let the sweep quietly stop covering it." >&2
        exit 1
    fi
    grep -vx -- "$name" "$LIST" > "$LIST.tmp" && mv "$LIST.tmp" "$LIST"
done <<ALONE_TABLE
$ALONE
ALONE_TABLE

N=$(wc -l < "$LIST")
echo "=== sweeping $N checks in chunks, plus the ALONE table" | tee -a "$LOG"

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

while IFS='|' read -r name args; do
    [ -z "${name:-}" ] && continue
    case "$name" in
        '#'*) continue ;;
    esac
    echo "=== alone $name" >> "$LOG"
    # shellcheck disable=SC2086
    ./target/release/ui-verify.exe --exe "$DRIVE/pdfcer-gui.exe" \
        $args --check "$name" --out "$OUT/uv-full-$name" >> "$LOG" 2>&1
    echo "=== alone $name rc=$?" >> "$LOG"
done <<ALONE_TABLE
$ALONE
ALONE_TABLE

echo "=== SWEEP-DONE" >> "$LOG"
