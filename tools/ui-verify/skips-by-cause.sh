#!/usr/bin/env bash
#
# Split a ui-verify sweep log's SKIPs into the two kinds that look identical in
# a tally and mean opposite things.
#
#   desktop-voided  the check never reached the application. Windows refused
#                   SetForegroundWindow because something else owned the
#                   desktop -- typically one stuck notification toast. This is
#                   not a result about the program. It is absent coverage.
#
#   genuine         the check ran and declined to judge: wrong fixture, absent
#                   capability, a platform that cannot enumerate windows. This
#                   IS a result, and its text says what would make it judge.
#
# A sweep printing `passed=86 failed=3 skipped=141` where 128 of the skips were
# one toast reads exactly like a sweep that measured something. Splitting the
# count is the only thing that tells the two apart at a glance.
#
# Usage:
#   skips-by-cause.sh <log> [--names]
#
#   --names   also list the desktop-voided check names, one per line, on stdout
#             before the summary. That list is what a re-run selects on.
#
# Output (stdout, last line, always):
#   desktop-voided=<n> genuine=<n> verdicts=<n>
#
# Exit status:
#   0  the log contains verdict lines and was classified
#   2  the log is missing, unreadable, or contains no verdict line at all
#
#      ★ 2 rather than 0-with-zeroes, because "no skips" and "no log" are the
#      same three zeroes, and a caller that cannot tell them apart will read a
#      runner that never started as a clean sweep. The one failure this whole
#      script exists to prevent.
#
# It does not write to the log. A classifier that appends to its own input
# corrupts the artifact it was asked to explain -- and one that did exactly
# that, through a `tee -a` inherited from the runner it was lifted out of, is
# why this is stated rather than assumed.

set -u

LOG=${1:-}
NAMES=${2:-}

if [ -z "$LOG" ] || [ ! -r "$LOG" ]; then
    echo "skips-by-cause: no readable log at '${LOG:-<none>}'" >&2
    echo "desktop-voided=0 genuine=0 verdicts=0"
    exit 2
fi

classified=$(
    awk '
        function flush(   s) {
            if (name == "") return
            # A refusal sentence is wrapped by the runner, so the phrase it is
            # matched on is split across lines in the file and no line-oriented
            # grep can find it. Collapse the block to one line before testing.
            s = buf
            gsub(/[ \t\r\n]+/, " ", s)
            if (kind == "SKIP") {
                if (index(s, "THE FOREGROUND IS HELD BY")) { print "V " name; fg++ }
                else { other++ }
            }
            name = ""
        }
        /^\[(PASS|FAIL|SKIP)/ { verdicts++; flush(); kind = substr($1, 2, 4); name = $2; buf = ""; next }
        { buf = buf " " $0 }
        END { flush(); print "C " fg + 0 " " other + 0 " " verdicts + 0 }
    ' "$LOG" | tr -d '\r'
)

summary=$(printf '%s\n' "$classified" | sed -n 's/^C //p')
fg=$(printf '%s\n' "$summary" | cut -d' ' -f1)
genuine=$(printf '%s\n' "$summary" | cut -d' ' -f2)
verdicts=$(printf '%s\n' "$summary" | cut -d' ' -f3)

if [ "${NAMES:-}" = "--names" ]; then
    printf '%s\n' "$classified" | sed -n 's/^V //p'
fi

echo "desktop-voided=${fg:-0} genuine=${genuine:-0} verdicts=${verdicts:-0}"

[ "${verdicts:-0}" -gt 0 ] || exit 2
