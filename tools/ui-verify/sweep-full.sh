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
#   target/scratch/sweep-skips-foreground-voided.txt
#                                    the checks whose verdict the DESKTOP took,
#                                    one name per line, ready to re-run
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
# ## ★★★ And a TALLY that adds up still is not coverage — measured 2026-09-15
#
# `passed=86 failed=3 skipped=141`, over a roster of 230. Every number true,
# every `rc=` printed, the sentinel present. **128 of those 141 skips were one
# stuck Windows notification toast** holding the foreground from about chunk 121
# to the end, so from there on nothing was clicked and nothing was measured. Per
# chunk the refusals ran 0, 11, 14, 2, 0, 1, then 20, 18, 18, 19, 18 out of 20.
#
# The `NOTHING RAN` guard below could not catch it: it required `passed` **and**
# `failed` to be zero, and the expensive case is the partial one. So the tally is
# now followed by a `=== SKIPS` line splitting the skips into the ones that are
# results about the application and the ones that are results about the desktop,
# and the second set is written out by name so re-running it is a file rather
# than an afternoon.
#
# ★ A skip count that CLIMBS through a run is a shared-resource story and never
#   a per-check one. That shape is the thing to look for, in this runner or any
#   other: check-specific causes scatter, an environmental cause has a start
#   time. Plot skips per chunk before reading a single skip reason.
#
# ⇒ Exit status: **0** only when every chunk returned 0; **1** a check failed,
# **2** a command line was wrong, **3** something did not run.
set -u

cd "$(dirname "$0")/../.." || exit 1

OUT=target/scratch
LOG=$OUT/sweep-full.log
VOIDED=$OUT/sweep-skips-foreground-voided.txt
CLASSIFY=tools/ui-verify/skips-by-cause.sh
LIST=$OUT/checks.txt
DRIVE=$OUT/drive

mkdir -p "$OUT" "$DRIVE" || exit 1
: > "$LOG"

# **The sweep's own verdict line.** Counts what is in the log rather than what
# the script believes it did — the two came apart on 2026-09-11 and the header
# says how. Printed on every exit path, including the aborts.
# ── Splitting the SKIPs by cause ──────────────────────────────────────────
#
# A skip reason is several wrapped lines under its `[SKIP] name` heading, and
# the wrap point moves with whatever coordinates the sentence printed earlier.
# So a phrase is found by gathering the whole block and squashing its
# whitespace, never by grepping for it. Measured 2026-09-15: `grep -c` on the
# refusal sentence answered 15 where the truth was 129, because the phrase fell
# across two lines in 114 of them — and that wrong answer was taken as evidence
# that the harness only sometimes names the culprit. It names it every time.
#
# Emits one `V <name>` line per desktop-voided skip and a final `C <fg> <other>`
# count line. The `tr -d` is load-bearing: awk's file redirection writes CRLF on
# this platform, and a work list with CR on every line matches nothing under the
# `grep -qx` that a re-run uses to select checks — a coverage loss with no
# symptom.
skips_by_cause() {
    # Delegates, so the runner and a re-run cannot drift apart in how they
    # classify a skip. The tool prints the desktop-voided names, then one
    # summary line; it exits 2 when the log holds no verdict at all, which is
    # the case this whole guard exists for and is handled by the caller.
    bash "$CLASSIFY" "$LOG" --names
}

tally() {
    passed=$(grep -c "^\[PASS\]" "$LOG" 2>/dev/null || true)
    failed=$(grep -c "^\[FAIL\]" "$LOG" 2>/dev/null || true)
    skipped=$(grep -c "^\[SKIP" "$LOG" 2>/dev/null || true)
    codes=$(grep -o "rc=[0-9]*" "$LOG" 2>/dev/null | sort -u | tr "
" " ")

    cause=$OUT/.skip-cause
    skips_by_cause > "$cause"
    classified=$?
    # Last line is the summary; everything above it is the voided name list.
    sed "$ d" "$cause" > "$VOIDED"
    summary=$(tail -1 "$cause")
    fg_skips=$(printf "%s" "$summary" | sed -n "s/.*desktop-voided=\([0-9]*\).*/\1/p")
    real_skips=$(printf "%s" "$summary" | sed -n "s/.*genuine=\([0-9]*\).*/\1/p")
    fg_skips=${fg_skips:-0}
    real_skips=${real_skips:-0}
    if [ "$classified" -ne 0 ]; then
        echo "sweep-full: the log holds no verdict line. Nothing was classified." >&2
        : > "$VOIDED"
    fi

    echo "=== TALLY passed=$passed failed=$failed skipped=$skipped codes: $codes" | tee -a "$LOG"
    echo "=== SKIPS  desktop-voided=$fg_skips genuine=$real_skips" | tee -a "$LOG"

    if [ "$passed" -eq 0 ] && [ "$failed" -eq 0 ]; then
        echo "sweep-full: NOTHING RAN. Not one check reached the application." >&2
        echo "            A sweep with no results is not a clean sweep." >&2
    fi

    # ★ The guard above only ever fired on a TOTAL void, and the expensive case
    #   is the partial one. On 2026-09-15 a sweep printed
    #   `passed=86 failed=3 skipped=141` — a tally that reads like a result —
    #   while 128 of those 141 skips were ONE stuck Windows notification toast
    #   holding the foreground from about chunk 121 to the end. More than half
    #   the roster measured nothing, for ninety minutes, and the only sentence
    #   saying so was buried once per skipped check, where nobody reads it
    #   because a SKIP is not red.
    #
    #   A skip count that climbs through a run is a shared-resource story and
    #   never a per-check one, so the cause is named here rather than left to be
    #   reconstructed from two hundred individual reasons.
    if [ "$fg_skips" -gt 0 ]; then
        holder=$(tr -s "[:space:]" " " < "$LOG" \
            | grep -o "THE FOREGROUND IS HELD BY: [^)]*)" | sort | uniq -c \
            | sort -rn | head -1 | sed "s/^ *[0-9]* THE FOREGROUND IS HELD BY: //")
        echo "sweep-full: $fg_skips of $skipped skips are NOT results about the" >&2
        echo "            application. Windows refused to bring the window to" >&2
        echo "            the front, so those checks never clicked anything." >&2
        echo "            Most often: $holder" >&2
        echo "            Those names are in $VOIDED. Re-run them; do not read" >&2
        echo "            the tally above as coverage. Start" >&2
        echo "            target/scratch/toast-watchdog.ps1 alongside the re-run." >&2
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
# ★★ AND WHAT IS NOT IN THIS TABLE, MEASURED 2026-09-12
#
#   `resize_scales_a_shape` used to be here, with
#   `--pdf fixtures/a1-titleblock.pdf --doc-point 0,300,500`. It needed the
#   same document as everybody else and only a different POINT on it, and
#   that is the wrong kind of thing for this table to carry: two sibling
#   checks that make the SAME gesture - `rotate_handle_turns_a_selection`
#   and `shift_constrains_a_resize` - stayed on the shared aim point and
#   both FAILED, each printing several paragraphs naming application
#   functions that are correct. The knowledge of which point works was
#   written down here, attached to one of the three checks that needed it.
#
#   ⇒ **This table is for a check that needs a different DOCUMENT.** A
#   check that needs a different POINT pins the point beside itself - see
#   `fixture::grip_gesture_target`, which all three now call, and which
#   carries the reason so the next person does not have to re-derive it.
#
#   THE ONE EXCEPTION, and it is a charter, not a convenience: a check
#   whose whole purpose is to take whatever document and point the
#   command line names cannot pin either of them, because pinning is the
#   thing it exists not to do.
#   `the_font_controls_are_live_on_the_drawing_you_open` is that check -
#   its module header sets it against a sibling that pins a fixture,
#   precisely so that green on the fixture and red on a real drawing is a
#   readable diagnosis. Its aim therefore belongs here even though its
#   document does not differ.
ALONE='
pages_stay_drawn_when_you_scroll_back|--pdf fixtures/synthetic-image-only-8pages.pdf --doc-point 0,150,200

# The shared aim is bare paper. These five need a click that lands on a
# run of text in a document with real text objects in it; the a1 sheet is
# CAD line-work with a title block, and per-glyph runs at that.
text_edit_on_a_real_drawing|--pdf fixtures/layered-drawing.pdf --doc-point 0,337.8,1505.0
arrow_keys_walk_between_blocks|--pdf fixtures/layered-drawing.pdf --doc-point 0,337.8,1505.0
text_selection_sweeps_and_copies|--pdf fixtures/layered-drawing.pdf --doc-point 0,337.8,1505.0
text_markup_marks_a_selection|--pdf fixtures/layered-drawing.pdf --doc-point 0,337.8,1505.0
text_tool_selects_and_marks_in_edit|--pdf fixtures/layered-drawing.pdf --doc-point 0,337.8,1505.0

# The refusal this check measures is a refusal on the document the
# operator brought, so it names that file and the glyph in it. Its own module header
# states exactly this pair.
his_typo_can_be_corrected_on_his_own_file|--pdf fixtures/per-glyph-twice.pdf --doc-point 0,84.3,703.8

# The exception described above: same document, different point, and the
# point cannot be pinned. See `fixture::a1_text_target`, which carries the
# two requirements any replacement must satisfy.
the_font_controls_are_live_on_the_drawing_you_open|--pdf fixtures/a1-titleblock.pdf --doc-point 0,1845.5,184.7
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
