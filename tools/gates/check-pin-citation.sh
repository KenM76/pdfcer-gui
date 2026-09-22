#!/usr/bin/env bash
# check-pin-citation.sh — the engine commit this shell is BUILT against, and
# the engine commit its operator-facing documents SAY it is built against,
# must be the same forty characters.
#
# ===========================================================================
# THE PROPERTY ASSERTED
# ===========================================================================
#
#   1. `Cargo.lock` names exactly one `pdfcer?branch=main#<sha>` revision.
#      Zero means the dependency shape changed — a path dependency, a
#      different branch — and every assumption below is void: that is a FAIL,
#      loudly, rather than a skip. More than one distinct sha would mean the
#      three engine crates had diverged, which is its own defect.
#
#   2. `FEATURES.md`'s CURRENT header — the FIRST line beginning
#      `**Updated:**`, and only that one — names a pin, and it is a prefix of
#      the locked sha. Later `**Updated:**` lines are RETAINED HISTORY of
#      earlier revisions and legitimately quote older pins; checking those
#      would be checking that the past has not changed, which is both true
#      and useless.
#
#   3. `RESUME.md`'s measured-state table carries an `| Engine pin |` row
#      whose value matches — the FIRST backticked sha on the row, because the
#      row's house style is value-then-explanation and the last sha on it is
#      the one the pin moved away from.
#
# An ABSENT pin is not a pass. If a rewording drops the phrase, or the
# document is not there at all, this gate FAILS and says so in those words. A
# gate keyed on a name is discharged by prose that stops using the name, so
# absence is red — never green, and never a skip.
#
# ===========================================================================
# WHY A HUMAN CANNOT HOLD IT
# ===========================================================================
#
# The file that changes does not contain the number that goes wrong.
# `Cargo.lock` holds the truth, as a `git+file:///…?branch=main#<sha>` source
# line that cargo rewrites silently and correctly. `FEATURES.md` holds a
# hand-typed seven-character copy of it, made once, at the moment somebody
# happened to look. Nothing links the two, so there is no event at which the
# copy becomes wrong — it simply is, some minutes later, and it reads exactly
# as it did when it was right.
#
# The dependency is taken on a BRANCH with no `rev`, so cargo re-resolves it
# opportunistically: the pin can move between two commands in one session with
# nobody typing `cargo update`. A number transcribed in the evening can be
# stale before the release it describes has finished building.
#
# And the copy is a claim to the operator about what he is running. When he
# reports a defect, the first question is which build and which engine pin,
# and the answer he can reach for is the one in the document that shipped
# beside the exe. A wrong answer there does not merely mislead him; it
# misleads the next session that reads his report.
#
# ===========================================================================
# IT IS SILENT ON SUCCESS, SO IT IS NOT EVIDENCE UNTIL FALSIFIED
# ===========================================================================
#
# A passing run prints NOTHING and exits 0. A transcript showing this gate
# green is therefore indistinguishable from a transcript of this gate doing
# nothing whatever: an extractor whose pattern stopped matching, a checker
# short-circuited by an early return, a document renamed out from under it —
# each of those is also silent, and also exits 0. Silence is the output of
# success and of total failure alike, and a reader cannot tell them apart.
#
# That is why `--self-test` is registered as a SECOND `run-all.sh` entry and
# is not optional. It copies the documents to a scratch directory, sabotages
# each in turn, and asserts the checker reports the sabotage; its output is
# the only evidence in a green run that the instrument is connected to
# anything at all. READ THE SELF-TEST'S LINES, NOT THIS GATE'S BLANK.
#
# To falsify it by hand, each of these must exit 1 and name the file:
#
#   * alter by one character the sha in `FEATURES.md`'s first `**Updated:**`
#     line;
#   * alter the first backticked sha in `RESUME.md`'s `| Engine pin |` row;
#   * delete the pin phrase from that header entirely, or move the file away
#     — absence must be red;
#   * put two different engine revisions in `Cargo.lock`;
#   * make `Cargo.lock` name no git-on-main engine source at all.
#
# `--self-test` does all five on copies in a scratch directory and leaves the
# working tree untouched, which is the only form of falsification safe to run
# in a tree carrying somebody else's uncommitted work.
#
# ===========================================================================
# WHAT IT PROVABLY CANNOT SEE
# ===========================================================================
#
#   * THE REVISION ACTUALLY COMPILED INTO THE EXE. Its oracle is the revision
#     the lock names NOW, and the two diverge in the window between
#     `cargo build` and this run. The window is small, and this gate and
#     `tools/package-portable.py`'s `locked_engine_rev` read the SAME source,
#     so the shipped documents will at least agree with each other — a pair
#     that agree wrongly is a far smaller problem than a pair that disagree.
#     But if this gate ever demands an edit that surprises you, SUSPECT THE
#     LOCK MOVED UNDER THE BUILD before editing the document: re-run the
#     build, then the gate, in that order.
#   * Every other hex string in the repository. It does not scan markdown for
#     hashes. Most of them are historical citations inside landed-feature rows
#     — `engine Pass 287.0 (a705d14)` — and those are records. A gate that
#     forced records to move would teach re-baselining, which is the failure
#     mode `check-engine-backlog.sh` exists to resist.
#   * Whether the pin is the RIGHT one. It checks that two places agree, not
#     that the engine revision is the one anybody intended to ship.
#   * Any citation outside the two sentences it aims at. A third document
#     quoting the pin is outside its corpus entirely, and so is a header that
#     names the pin in some other wording — the extractor matches one phrasing
#     and nothing else.
#
# ===========================================================================
# THE EXIT CONTRACT
# ===========================================================================
#
#   0  every citation agrees with the lock — rendered as SILENCE, see above
#   1  a citation is stale, a citation is absent, a named document is absent,
#      or the lock is not the shape this gate understands
#
# There is no exit 2. Every input it needs is inside this repository, so there
# is no "could not measure" state to distinguish: a missing document is a
# missing citation, and a missing citation is a failure.

set -u

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"

# ---------------------------------------------------------------------------
# read_locked_pin_in <root> — the one true sha, out of that root's Cargo.lock
#
# The `source` line looks like
#   source = "git+file:///D:/Dev/pdfcer?branch=main#01c4a101a9…"
# and appears once per engine crate (core, render, print), all three carrying
# the same revision because they come from one repository. We take the set of
# distinct values so a divergence is a FAIL rather than a coin toss on
# whichever line `grep -m1` happened to reach first.
#
# ★ It takes the root as an ARGUMENT rather than reading the global, and the
# reason is the self-test: a checker that can only ever be pointed at the real
# repository cannot be sabotaged, and a check that cannot be made to fail is
# not evidence. Every other function below is parameterised for the same
# reason.
# ---------------------------------------------------------------------------
read_locked_pin_in() {
    local lock="$1/Cargo.lock"
    if [ ! -f "$lock" ]; then
        echo "PIN: Cargo.lock not found at $lock" >&2
        return 1
    fi
    local shas n
    shas="$(grep -o 'pdfcer?branch=main#[0-9a-f]\{7,40\}' "$lock" | sed 's/.*#//' | sort -u)"
    n="$(printf '%s\n' "$shas" | grep -c '[0-9a-f]' || true)"
    if [ "$n" -eq 0 ]; then
        echo "PIN: Cargo.lock names no 'pdfcer?branch=main#<sha>' revision." >&2
        echo "     The engine dependency is not the shape this gate understands" >&2
        echo "     (a git dependency on branch main). If that changed on purpose," >&2
        echo "     this gate needs rewriting, not silencing." >&2
        return 1
    fi
    if [ "$n" -gt 1 ]; then
        echo "PIN: Cargo.lock names $n DIFFERENT engine revisions:" >&2
        printf '     %s\n' $shas >&2
        echo "     The three engine crates come from one repository and must" >&2
        echo "     share one revision. Run cargo update on all three together." >&2
        return 1
    fi
    printf '%s' "$shas"
}

# ---------------------------------------------------------------------------
# check_citation <label> <file> <extractor-output> <locked>
#
# `cited` may be shorter than `locked` — the documents quote seven characters,
# which is what `git log --oneline` prints and what a human can hold in mind.
# A prefix match is therefore the correct comparison, not equality. It is also
# strictly safe: a seven-character prefix of a forty-character sha identifies
# the commit unless the repository has a collision at that length, in which
# case git itself would refuse to abbreviate to seven.
# ---------------------------------------------------------------------------
check_citation() {
    local label="$1" file="$2" cited="$3" locked="$4"
    if [ -z "$cited" ]; then
        echo "PIN: $label no longer names the engine pin." >&2
        echo "     file   : $file" >&2
        echo "     This gate exists because that number is a claim to the" >&2
        echo "     operator about what he is running. If the claim has been" >&2
        echo "     deliberately dropped, delete the corresponding block in" >&2
        echo "     check-pin-citation.sh in the same commit -- do not leave a" >&2
        echo "     gate aiming at a sentence that no longer exists." >&2
        return 1
    fi
    case "$locked" in
        "$cited"*) return 0 ;;
    esac
    echo "PIN: $label quotes an engine pin the build does not use." >&2
    echo "     file   : $file" >&2
    echo "     says   : $cited" >&2
    echo "     locked : $locked" >&2
    echo "     Cargo.lock is authoritative. Re-measure the document against" >&2
    echo "     the build it describes; do not edit Cargo.lock to agree." >&2
    return 1
}

# --- the two extractors, each aimed at exactly one sentence ----------------

# FEATURES.md: the FIRST `**Updated:**` line only. Later ones are history.
cite_features() {
    local f="$1"
    [ -f "$f" ] || return 0
    grep -m1 '^\*\*Updated:\*\*' "$f" \
        | grep -o 'pinned at \*\*`[0-9a-f]\{7,40\}`\*\*' \
        | grep -o '[0-9a-f]\{7,40\}' \
        | head -1
}

# RESUME.md: the measured-state row whose first cell is `Engine pin`.
#
# The FIRST backticked sha on that row, not the last, and the difference is not
# cosmetic. The row's house style is value-then-explanation:
#
#   | Engine pin | <re-measure command> | `3e73a02` -- moved from `d86cb19`
#     at 09:40 today |
#
# so the last sha on the row is the one the pin moved AWAY from. Reading the
# last one does not merely mis-read a narrating row: it reads the sha a STALE
# row is most likely to carry, so a row that had genuinely gone stale -- old
# sha in the value, new sha mentioned in the prose after it -- would pass. An
# assertion that both outcomes satisfy is not a measurement of which one
# shipped. Self-test cases 7 and 8 below hold both directions.
#
# `head -1` is safe against the command cell for a structural reason worth
# stating: that cell holds ONE backticked span containing a shell command, and
# the pattern needs a backtick immediately either side of the hex. A sha quoted
# inside the command would be surrounded by the command's own text, not by
# backticks. Field-splitting on `|` was considered and rejected -- the adjacent
# command cell is allowed to contain a pipe (the row below it does).
cite_resume() {
    local f="$1"
    [ -f "$f" ] || return 0
    grep -m1 '^| *Engine pin *|' "$f" \
        | grep -o '`[0-9a-f]\{7,40\}`' \
        | head -1 \
        | tr -d '`'
}

run_checks() {
    local root="$1"
    local locked rc=0
    locked="$(read_locked_pin_in "$root")" || return 1
    local fpin rpin
    fpin="$(cite_features "$root/FEATURES.md")"
    rpin="$(cite_resume "$root/RESUME.md")"
    check_citation "FEATURES.md's current revision header" "$root/FEATURES.md" "$fpin" "$locked" || rc=1
    check_citation "RESUME.md's measured-state table"      "$root/RESUME.md"   "$rpin" "$locked" || rc=1
    return $rc
}

# ---------------------------------------------------------------------------
# --self-test — sabotage each input in turn and require the checker to notice
# ---------------------------------------------------------------------------
self_test() {
    local tmp fails=0
    tmp="$(mktemp -d)" || { echo "self-test: mktemp failed" >&2; return 1; }
    trap 'rm -rf "$tmp"' RETURN

    expect() { # <name> <expected-rc> <root>
        local name="$1" want="$2" root="$3" got
        run_checks "$root" >/dev/null 2>&1 && got=0 || got=1
        if [ "$got" -ne "$want" ]; then
            echo "self-test FAILED: $name -- wanted rc=$want, got rc=$got" >&2
            fails=$((fails + 1))
        else
            echo "  ok  $name (rc=$got)"
        fi
    }

    # Case 0 — the real tree. This is the control: if the repository is
    # currently stale the self-test must not pretend otherwise, so this case
    # is reported and not asserted.
    if run_checks "$ROOT" >/dev/null 2>&1; then
        echo "  ok  the real tree agrees with its lock"
    else
        echo "  --  the real tree is CURRENTLY STALE (that is what the gate is for)"
    fi

    # Build a synthetic, known-good root.
    local good="$tmp/good"
    mkdir -p "$good"
    printf 'source = "git+file:///D:/Dev/pdfcer?branch=main#abcdef1234567890abcdef1234567890abcdef12"\n' > "$good/Cargo.lock"
    printf '**Updated:** 2026-01-01 (first revision, pinned at **`abcdef1`** which is the tip of its `main`.)\n' > "$good/FEATURES.md"
    printf '| Engine pin | `grep -m1 x Cargo.lock` | `abcdef1` |\n' > "$good/RESUME.md"
    expect "a synthetic tree whose citations agree" 0 "$good"

    # Sabotage 1 — FEATURES.md quotes an older pin.
    local s1="$tmp/s1"; cp -r "$good" "$s1"
    printf '**Updated:** 2026-01-01 (first revision, pinned at **`0000000`** which is the tip of its `main`.)\n' > "$s1/FEATURES.md"
    expect "FEATURES.md quoting a stale pin is caught" 1 "$s1"

    # Sabotage 2 — RESUME.md quotes an older pin.
    local s2="$tmp/s2"; cp -r "$good" "$s2"
    printf '| Engine pin | `grep -m1 x Cargo.lock` | `0000000` |\n' > "$s2/RESUME.md"
    expect "RESUME.md quoting a stale pin is caught" 1 "$s2"

    # Sabotage 3 — the phrase is reworded away entirely. Absence must be RED.
    local s3="$tmp/s3"; cp -r "$good" "$s3"
    printf '**Updated:** 2026-01-01 (first revision, built against the current engine.)\n' > "$s3/FEATURES.md"
    expect "a header that stopped naming the pin is caught" 1 "$s3"

    # Sabotage 4 — the lock stops being a git-on-main dependency.
    local s4="$tmp/s4"; cp -r "$good" "$s4"
    printf 'source = "registry+https://example.invalid/"\n' > "$s4/Cargo.lock"
    expect "a lock this gate cannot read is caught, not skipped" 1 "$s4"

    # Sabotage 5 — the three engine crates disagree with each other.
    local s5="$tmp/s5"; cp -r "$good" "$s5"
    {
        printf 'source = "git+file:///D:/Dev/pdfcer?branch=main#abcdef1234567890abcdef1234567890abcdef12"\n'
        printf 'source = "git+file:///D:/Dev/pdfcer?branch=main#1111111111111111111111111111111111111111"\n'
    } > "$s5/Cargo.lock"
    expect "two different engine revisions in one lock is caught" 1 "$s5"

    # Sabotage 6 — a LATER `**Updated:**` line holds an old pin. That is
    # retained history and must NOT fail. This is the one case where the gate
    # is asserted to stay quiet, and it is the case a naive whole-file grep
    # would get wrong.
    local s6="$tmp/s6"; cp -r "$good" "$s6"
    {
        printf '**Updated:** 2026-01-02 (second revision, pinned at **`abcdef1`** which is the tip of its `main`.)\n'
        printf '\n'
        printf '**Updated:** 2026-01-01 (first revision, pinned at **`0000000`** which is the tip of its `main`.)\n'
    } > "$s6/FEATURES.md"
    expect "an older revision header quoting an older pin is LEFT ALONE" 0 "$s6"

    # Case 7 -- the row NARRATES a move. Current pin first, superseded pin in
    # the prose after it. Must stay quiet: that is the shape a CORRECT row
    # takes, and it is the cheaper half of the pair.
    local s7="$tmp/s7"; cp -r "$good" "$s7"
    printf '| Engine pin | `grep -m1 x Cargo.lock` | `abcdef1` -- moved from `0000000` at 09:40 today |\n' > "$s7/RESUME.md"
    expect "a row that narrates its own move is read at its CURRENT value" 0 "$s7"

    # Case 8 -- the falsification. The value cell is STALE and the locked sha
    # appears later in the same row's prose. A reader that took the last sha on
    # the row would call this agreement; it is the exact opposite.
    #
    # This case is why case 7 was not fixed by loosening the match. A gate can
    # be made to stop complaining about a correct row by reading anywhere on
    # it, and that same looseness is what lets a wrong row through.
    local s8="$tmp/s8"; cp -r "$good" "$s8"
    printf '| Engine pin | `grep -m1 x Cargo.lock` | `0000000` -- will move to `abcdef1` when the next build runs |\n' > "$s8/RESUME.md"
    expect "a stale value cell is caught even when the row later names the locked pin" 1 "$s8"

    if [ "$fails" -ne 0 ]; then
        echo "self-test: $fails case(s) failed" >&2
        return 1
    fi
    echo "self-test: all cases behaved"
    return 0
}

if [ "${1:-}" = "--self-test" ]; then
    self_test
    exit $?
fi

run_checks "$ROOT"
rc=$?
# ★ The clean line names the pin and both documents it read.
#
# `run_checks` speaks only on failure, which is ordinary tool manners and the
# wrong manners for a check: exit 0 with no output is satisfied equally by "it
# compared both citations and they agreed" and by "the scan root moved, it read
# nothing, and nothing disagreed with nothing". The value and the two document
# names are what make the second case visible without running `--self-test`.
if [ "$rc" -eq 0 ]; then
    echo "check-pin-citation: clean — FEATURES.md and RESUME.md both quote" \
         "$(read_locked_pin_in "$ROOT" | cut -c1-7), the engine pin in Cargo.lock."
fi
exit $rc
