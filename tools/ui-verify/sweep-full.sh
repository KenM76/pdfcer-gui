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
#
# ## ★★★ And its PRESENCE never meant "something ran" — measured 2026-09-11
#
# A sweep was started against a harness whose source was thirty-two seconds
# newer than its binary. `ui-verify` refused, correctly and at length, **once
# per chunk**: eleven chunks and two ALONE records, every one `rc=2`, the whole
# thing over in under a minute, and the log ending in `=== SWEEP-DONE`. Two
# hundred and ten checks were reported as swept and not one of them launched
# anything.
#
# Nothing here was lying. The script printed every `rc=` it promised. But the
# only sentence a reader had been given to check was the sentinel, and the
# sentinel was true. That is this project's most-repeated defect shape wearing a
# new coat: **a runner whose green is a statement about the runner rather than
# about the subject.**
#
# Three changes, and each closes a different half of it:
#
# 1. **The binaries are BUILT here, not assumed.** The stale refusal could only
#    happen because this script took "somebody ran cargo build" on trust. It no
#    longer does, and a build failure stops the sweep before it takes the
#    desktop.
# 2. **`rc=2` aborts immediately.** Exit 2 is `ui-verify` saying *the command
#    line was wrong* — a stale binary, a missing fixture, a renamed flag. It
#    is never a result about the application, so spending twenty more launches
#    on the same mistake buys nothing and buries the one line that names the
#    cause under a thousand lines of usage text.
# 3. **The sentinel is followed by a TALLY that must add up.** The final line
#    counts the PASS / FAIL / SKIP actually present in the log and states the
#    set of chunk return codes. A sweep that ran nothing now says `passed=0`
#    beside `SWEEP-DONE`, and a zero is something a reader can disbelieve.
#
# ⇒ Exit status: **0** only when every chunk returned 0; **1** a check failed,
# **2** a command line was wrong, **3** something did not run.
set -u

cd "$(dirname "$0")/../.." || exit 1

OUT=target/scratch
LOG=$OUT/sweep-full.log
LIST=$OUT/checks.txt
DRIVE=$OUT/drive

mkdir -p "$OUT" "$DRIVE" || exit 1
: > "$LOG"

# **The sweep's own verdict line.** Counts what is in the log rather than what
# the script believes it did — the two came apart on 2026-09-11 and the header
# says how. Printed on every exit path, including the aborts.
tally() {
    passed=$(grep -c "^\[PASS\]" "$LOG" 2>/dev/null || echo 0)
    failed=$(grep -c "^\[FAIL\]" "$LOG" 2>/dev/null || echo 0)
    skipped=$(grep -c "^\[SKIP" "$LOG" 2>/dev/null || echo 0)
    codes=$(grep -o "rc=[0-9]*" "$LOG" 2>/dev/null | sort -u | tr "
" " ")
    echo "=== TALLY passed=$passed failed=$failed skipped=$skipped codes: $codes" | tee -a "$LOG"
    if [ "$passed" -eq 0 ] && [ "$failed" -eq 0 ]; then
        echo "sweep-full: NOTHING RAN. Not one check reached the application." >&2
        echo "            A sweep with no results is not a clean sweep." >&2
    fi
}

# Rule 0: BUILD BOTH BINARIES. Not a convenience — the failure it closes is in
# the header. `ui-verify` refuses to drive a binary older than its own sources,
# and it refuses per invocation, so an unbuilt harness turns a whole sweep into
# eleven identical usage dumps and a SWEEP-DONE.
#
# ★ Built BEFORE the desktop is taken, so a compile error costs seconds rather
# than interrupting the operator for nothing.
echo "=== building the harness and the application"
if ! cargo build --release -p ui-verify -p pdfcer-gui; then
    echo "sweep-full: the build failed, so there is nothing honest to drive." >&2
    exit 2
fi

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

# The worst return code any chunk produced, so this script's exit status is a
# statement about the whole run rather than about its last chunk.
worst=0

for start in $(seq 1 20 "$N"); do
    ARGS=$(sed -n "${start},$((start + 19))p" "$LIST" | sed 's/^/--check /' | tr '\n' ' ')
    echo "=== chunk $start" >> "$LOG"
    # shellcheck disable=SC2086
    ./target/release/ui-verify.exe --exe "$DRIVE/pdfcer-gui.exe" \
        --pdf fixtures/a1-titleblock.pdf --doc-point 0,2000,320 \
        --second-pdf fixtures/four-pages.pdf --out "$OUT/uv-full" \
        $ARGS >> "$LOG" 2>&1
    rc=$?
    echo "=== chunk $start rc=$rc" >> "$LOG"
    [ "$rc" -gt "$worst" ] && worst=$rc
    if [ "$rc" -eq 2 ]; then
        echo "=== ABORTED: chunk $start rejected the command line (rc=2)." >> "$LOG"
        echo "sweep-full: ABORTED at chunk $start — ui-verify rejected the" >&2
        echo "            command line (rc=2). Every later chunk would be" >&2
        echo "            rejected identically. The cause is the last" >&2
        echo "            screenful of $LOG." >&2
        tally
        exit 2
    fi
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
    rc=$?
    echo "=== alone $name rc=$rc" >> "$LOG"
    [ "$rc" -gt "$worst" ] && worst=$rc
    if [ "$rc" -eq 2 ]; then
        echo "=== ABORTED: alone $name rejected the command line (rc=2)." >> "$LOG"
        echo "sweep-full: ABORTED on '$name' — rc=2. Its ALONE arguments name" >&2
        echo "            a fixture or a flag that is wrong. See $LOG." >&2
        tally
        exit 2
    fi
done <<ALONE_TABLE
$ALONE
ALONE_TABLE

echo "=== SWEEP-DONE" >> "$LOG"
tally
exit "$worst"
