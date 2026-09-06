#!/usr/bin/env bash
# Drive the suite ONE CHECK AT A TIME, appending each verdict as it lands.
#
# ★★★ THE LOCK IS NOT OPTIONAL. Three copies of this script were running at
# once on 2026-09-06, because the harness's "killed for memory" notification
# stops the tool wrapper and NOT the detached shell under it. Three runners
# means three processes moving one pointer and contending on one portable
# userdata/ directory, so every verdict they produced was worthless — and the
# only tell was an impossible line count in the results file. A driven run
# competes for a resource there is exactly one of; it must hold a lock that
# says so.
#
# Two other shapes learned the same afternoon:
#  * The whole-suite invocation buffers its report. Killed twice, after 93 and
#    20 checks, taking every verdict with it. A batch of one costs one check.
#  * A survivor window steals the pointer from the next check, so every check
#    is followed by a kill.
#
# Fixture and point come from RESUME.md's "THE SWEEP NEEDS THREE FIXTURES"
# table. This is the "everything else" family; checks that decline on aim are
# re-run against their own fixture afterwards, which is the documented
# procedure a previous session skipped — producing seven failures that were the
# harness pointing at the wrong thing.
set -u
SC="C:/Users/Ken/AppData/Local/Temp/claude/D--Dev-pdfcer-gui/0a342979-67dc-4ff5-bf1b-8a553cac669d/scratchpad/sweep"
# ⚠ `flock` DOES NOT EXIST in Git Bash on this machine. The first version of
# this guard used it, and because a missing command exits non-zero the lock
# refused BOTH runners — including the one that was supposed to run. A lock
# whose failure mode is "nothing runs" is quieter than no lock and just as
# wrong. `mkdir` is atomic on every filesystem this will ever see.
LOCK="$SC/runner.lock.d"
if ! mkdir "$LOCK" 2>/dev/null; then
  echo "REFUSED: another runner holds $LOCK. One driven run at a time." >&2
  echo "  If no runner is alive, remove it: rm -rf '$LOCK'" >&2
  exit 4
fi
echo "$$" > "$LOCK/pid"
trap 'rm -rf "$LOCK"' EXIT INT TERM

UV="D:/Dev/pdfcer-gui/target/release/ui-verify.exe"
EXE="$SC/pdfcer-gui.exe"
PDF="${SWEEP_PDF:-D:/Dev/pdfcer-gui/fixtures/a1-titleblock.pdf}"
PDF2="D:/Dev/pdfcer-gui/fixtures/four-pages.pdf"
POINT="${SWEEP_POINT:-0,300,500}"
LIST="${SWEEP_LIST:-/tmp/checks.txt}"
RESULTS="${SWEEP_RESULTS:-$SC/results.txt}"
mkdir -p "$SC/out"
touch "$RESULTS"
while read -r c; do
  [ -z "$c" ] && continue
  # Resume, never redo — a killed run picks up where it stopped.
  if cut -f2 "$RESULTS" | grep -qx "$c"; then continue; fi
  # A FRESH PROFILE PER CHECK. The binary is portable, so it keeps its settings
  # in `userdata/` BESIDE THE EXE -- one directory shared by every check in the
  # sweep. Harmless only while the application threw the stored mode away on
  # every launch. It stopped being harmless on 2026-09-06 when that was fixed:
  # a check that clicks the Edit segment now leaves mode=edit on disk and every
  # later check starts in Edit.
  #
  # It cost an investigation the same afternoon. a_link_goes_to_the_page_it_names
  # reported that a click on a link "produced nothing" -- no link-click, no
  # page-links -- which reads exactly like a regression in the press ladder. It
  # was not: in Edit a click SELECTS a link rather than following it, so the hit
  # test is never called. On a fresh profile the same binary reaches the link.
  #
  # => A SUITE THAT SHARES PERSISTENT STATE MEASURES THE ORDER IT RAN IN, and
  # the contamination is invisible in the failing check's own report.
  CHECKDIR="$SC/profiles/$c"
  rm -rf "$CHECKDIR"; mkdir -p "$CHECKDIR"
  cp "$EXE" "$CHECKDIR/pdfcer-gui.exe"
  out=$("$UV" --exe "$CHECKDIR/pdfcer-gui.exe" --pdf "$PDF" --second-pdf "$PDF2" \
        --doc-point "$POINT" --out "$SC/out" --check "$c" 2>&1)
  code=$?
  verdict=$(printf '%s\n' "$out" | grep -oE '^\[(PASS|FAIL|SKIP)\]' | head -1 | tr -d '[]')
  [ -z "$verdict" ] && verdict="NOVERDICT"
  printf '%s\t%s\texit=%s\t%s\n' "$verdict" "$c" "$code" "$(basename "$PDF")@$POINT" >> "$RESULTS"
  printf '%s\n' "$out" > "$SC/out/$c.report.txt"
  taskkill //F //IM pdfcer-gui.exe > /dev/null 2>&1
  rm -rf "$CHECKDIR"
done < "$LIST"
echo "SWEEP-COMPLETE	-	-	$(basename "$PDF")@$POINT" >> "$RESULTS"
