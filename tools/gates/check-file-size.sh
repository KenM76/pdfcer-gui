#!/usr/bin/env bash
# check-file-size.sh — no .rs file may exceed 1,500 lines. Standing rule R2.
#
# ===========================================================================
# THE PROPERTY ASSERTED
# ===========================================================================
#
# Every `.rs` file under `crates/` and `tools/` is within the line limit, so
# that each one has a single subject a reader can hold whole.
#
# ===========================================================================
# WHY A HUMAN CANNOT HOLD IT
# ===========================================================================
#
# A file does not become unreadable at a moment anybody can point at. It grows
# fifty lines at a time, each addition locally reasonable, and the cost falls
# on a different person later — so nobody is ever holding both the growth and
# its consequence at once. The consequences are of a shape review cannot catch
# either:
#
#   * The same concept re-implemented in two places, because nobody could see
#     the first one, and the copies then drift. DEFECTS.md D5 is that shape: a
#     hand-maintained keyboard reference disagreeing with the keymap.
#   * Two individually correct lines that are only wrong together, too far
#     apart for any reviewer to have been expected to see both. DEFECTS.md D1
#     is that shape: two spellings of "is the operator typing?", each
#     defensible, that between them suppress every unmodified key.
#   * Tooling degrades. `cargo fmt` on a file that size, an LLM reading it, a
#     grep for a symbol — all become approximate.
#
# The GUI this project replaces carries a single `main.rs` in the tens of
# thousands of lines. That size was never a decision anybody made; it is a
# limit nobody ever set. The value of this gate is that the limit is enforced
# from the first commit rather than adopted once a file is already
# unmanageable.
#
# 1,500 lines is not a magic number. It is roughly one sitting, and small
# enough that a file has to have one subject.
#
# ===========================================================================
# WHAT IT COUNTS, AND WHAT IT PROVABLY CANNOT SEE
# ===========================================================================
#
# Total physical lines, comments and blanks included — deliberately NOT "code
# lines". This project asks for verbose documentation, so a code-line metric
# would be the one that quietly rewards deleting the docs to get under a
# threshold, and a file whose comments make it 2,900 lines long is still a
# file nobody can navigate. The READING COST is the thing being limited.
#
# Blind spots, all of them real:
#
#   * Complexity. A 200-line file can be worse than a 1,400-line one, and this
#     gate has no opinion about either. Length is a proxy for "one subject"
#     and nothing more.
#   * Anything outside `crates/` and `tools/`, anything not named `*.rs`, and
#     anything on a `fixtures/` or `target/` path — see `is_exempt()`.
#   * A file at 1,499 lines. It prints the three largest on success so that a
#     file one feature away from firing is visible, but it cannot fail on one,
#     and the cheapest moment to split a module is before it has to be split
#     in a hurry.
#
# The right response to a failure is to SPLIT THE MODULE, not to shrink the
# prose. A file that genuinely cannot be split — a generated table, a large
# const catalog — is an operator decision and belongs in `is_exempt()` with
# its reason written down, never in a silent threshold bump.
#
# ===========================================================================
# USAGE, THE EXIT CONTRACT, AND HOW TO FALSIFY IT
# ===========================================================================
#   tools/gates/check-file-size.sh [LIMIT]
#
#   0  every .rs file scanned is within the limit
#   1  at least one file is over
#   2  PRECONDITION ABSENT — no `crates/` or `tools/` directory here, no
#      `.rs` file under them, or LIMIT is not a positive integer. NOT a pass:
#      "nothing over the limit" and "nothing at all" must not produce the same
#      green tick, and `run-all.sh` prints skips in their own block and
#      exits 3.
#
# There is no `--self-test`, and passing one is now a skip rather than a pass.
# Falsification here is against the real tree, because the property is a line
# count and the tree is the only corpus whose counts matter: run
# `tools/gates/check-file-size.sh 50` and it must go red and name files; run it
# from a directory with no `crates/` and it must exit 2; hand it `abc`, `0` or
# a mistyped flag and it must exit 2 rather than printing a clean line over
# comparisons that all failed.

set -euo pipefail

LIMIT="${1:-1500}"

# ★ The argument is a LINE COUNT, and anything else must stop the run here.
#
# Not a style check. `[ "$n" -gt "$LIMIT" ]` sits inside an `if`, where a
# non-integer makes `test` write "integer expression expected" to stderr and
# return 2 — and `set -e` does not fire on a condition. Every comparison then
# fails, `offenders` stays empty, and the gate prints its CLEAN line having
# measured nothing. A mistyped flag (`--self-test`, `--limit 900`) is the
# ordinary way in, and it reads as a pass.
#
# Exit 2 rather than 1 for the same reason an empty tree does: nothing was
# compared, so no file has been cleared.
case "$LIMIT" in
    '' | *[!0-9]*)
        echo "file-size: SKIPPED — '$LIMIT' is not a line count." >&2
        echo "  The only argument is the limit, as a positive integer:" >&2
        echo "      tools/gates/check-file-size.sh [LIMIT]   (default 1500)" >&2
        echo "  There is no --self-test here; falsify it with a small LIMIT." >&2
        echo "  Exiting 2, not 0: nothing was compared, so nothing is clean." >&2
        exit 2
        ;;
esac
if [ "$LIMIT" -eq 0 ]; then
    echo "file-size: SKIPPED — a limit of 0 clears no file and fails every one." >&2
    echo "  Exiting 2, not 1: that is a mistyped argument, not a tree defect." >&2
    exit 2
fi

# Roots to scan. `target/` is excluded below: build artefacts include generated
# .rs files that nobody wrote and nobody can split.
ROOTS=()
[ -d crates ] && ROOTS+=(crates)
[ -d tools ] && ROOTS+=(tools)

if [ "${#ROOTS[@]}" -eq 0 ]; then
    echo "file-size: SKIPPED — no crates/ or tools/ directory here." >&2
    echo "  Run from the repository root. Exiting 2, not 0: an unscanned tree" >&2
    echo "  is not a compliant tree." >&2
    exit 2
fi

# EXEMPT — paths whose size is not a maintenance problem. One entry, one
# reason, reviewed like any other rule change.
#
#   fixtures/  — gate fixtures are inputs to shell scripts, not modules anybody
#                navigates. (They are all tiny anyway; the exclusion is for the
#                day one of them is a generated corpus.)
is_exempt() {
    case "$1" in
        */fixtures/*) return 0 ;;
        */target/*) return 0 ;;
        *) return 1 ;;
    esac
}

scanned=0
offenders=""
# Every file with its count, for the "largest files" report. Kept in one string
# rather than an array so the sort below is a single pipeline.
all=""

while IFS= read -r -d '' file; do
    is_exempt "$file" && continue
    scanned=$((scanned + 1))
    n=$(wc -l < "$file" | tr -d ' ')
    all="${all}${n} ${file}
"
    if [ "$n" -gt "$LIMIT" ]; then
        offenders="${offenders}${n} ${file}
"
    fi
done < <(find "${ROOTS[@]}" -type f -name '*.rs' -not -path '*/target/*' -print0 | sort -z)

if [ "$scanned" -eq 0 ]; then
    echo "file-size: SKIPPED — no .rs files found under ${ROOTS[*]}." >&2
    echo "  Exiting 2, not 0: 'nothing over the limit' and 'nothing at all'" >&2
    echo "  must not produce the same green tick." >&2
    exit 2
fi

if [ -n "$offenders" ]; then
    echo "file-size: FAIL — $(printf '%s' "$offenders" | grep -c '^') file(s) over $LIMIT lines:"
    printf '%s' "$offenders" | sort -rn | awk '{ printf "  %7d  %s\n", $1, $2 }'
    cat <<EOF

Rule R2: no .rs file over $LIMIT lines. Split the module along its seams —
one subject per file — rather than raising the limit.

The GUI this project replaces reached 25,005 lines in a single main.rs. That
was never a decision anybody made; it was a limit nobody ever set. Two of the
defects in DEFECTS.md are pairs of lines thousands of lines apart that no
reviewer could have been expected to see together.

If a file genuinely cannot be split (a generated table, a large const
catalog), that is an operator decision: add it to is_exempt() in this script
with the reason written down.
EOF
    exit 1
fi

# Report the three largest even on success. A gate that only speaks when it
# fails gives no warning that a file is at 1,480 lines and one feature away
# from firing — and the cheapest moment to split a module is before it has to
# be split in a hurry.
echo "file-size: clean — $scanned .rs file(s) scanned, none over $LIMIT lines"
if [ -n "$all" ]; then
    echo "           largest:"
    printf '%s' "$all" | sort -rn | head -3 | awk '{ printf "             %7d  %s\n", $1, $2 }'
fi
exit 0
