#!/usr/bin/env bash
# check-pin-citation.sh — the engine commit this shell is BUILT against, and
# the engine commit its operator-facing documents SAY it is built against,
# must be the same forty characters.
#
# ===========================================================================
# ★★★ WHY THIS GATE EXISTS
# ===========================================================================
#
# Measured 2026-09-11, 22:55, while preparing the fifth release of the day.
#
# `FEATURES.md` is the document that ships to the operator. Its header opens
# with a dated revision note naming the engine version and the exact commit
# the build consumed:
#
#   **Updated:** 2026-09-11, evening (twenty-fourth revision — re-measured
#   against the build about to ship, engine `pdfcer-core` v0.53.0, a git
#   dependency on the local engine repository, pinned at `d2465f5` which is
#   the tip of its `main`. …
#
# That header was written at 19:52. The pin moved to `01c4a10` at 22:01 and
# was about to move again to `a431641`. **Nothing noticed, and nothing could
# have**, because the number lives in one file and the fact lives in another:
#
#   - `Cargo.lock` holds the truth, as a `git+file:///D:/Dev/pdfcer?branch=main#<sha>`
#     source line. It is rewritten by `cargo update`, silently and correctly.
#   - `FEATURES.md` holds a hand-typed seven-character copy of it, made once,
#     at the moment a human happened to look.
#
# ⇒ **The file that changed does not contain the number that went wrong.**
# This is the eighth recorded instance in this repository of a verbatim
# quotation of another artefact's state going stale invisibly, and the
# previous seven were all corrected by hand after somebody noticed. A
# correction that depends on noticing is not a correction; it is luck with a
# commit message.
#
# It matters more here than in the general case for one specific reason:
# **the pin in that header is a claim to the operator about what he is
# running.** When he reports a defect, the first question is always *which
# build and which engine pin* — and the answer he can reach for is the one in
# the document that shipped beside the exe. A wrong answer there does not
# merely mislead him; it misleads the next session reading his report.
#
# ===========================================================================
# WHAT IT CHECKS
# ===========================================================================
#
#   1. `Cargo.lock` names exactly one `pdfcer?branch=main#<sha>` revision.
#      Zero means the dependency shape changed (a path dependency, a
#      different branch) and every assumption below is void — FAIL, loudly,
#      rather than skip. More than one distinct sha would mean the three
#      engine crates had diverged, which is its own defect.
#
#   2. `FEATURES.md`'s **current** header — the FIRST line beginning
#      `**Updated:**`, and only that one — names a pin, and it is a prefix
#      of (or equal to) the locked sha. Later `**Updated:**` lines in that
#      file are RETAINED HISTORY of earlier revisions and legitimately quote
#      older pins; checking them would be checking that the past has not
#      changed, which is both true and useless.
#
#   3. `RESUME.md`'s measured-state table carries an `| Engine pin |` row
#      whose recorded value matches. That row prints its own re-measurement
#      command in the adjacent cell, which is the right design and is exactly
#      why the stale value in it is so convincing: a number sitting beside
#      the command that would disprove it reads as having been measured.
#
# ===========================================================================
# WHAT IT DELIBERATELY DOES NOT DO
# ===========================================================================
#
# It does not scan every markdown file for every hex string. Most of the
# hashes in this repository are historical citations inside landed-feature
# rows — `engine Pass 287.0 (a705d14)` — and those are records. A gate that
# forced records to move would teach re-baselining, which is the failure mode
# `check-engine-backlog.sh` already warns about in its own header.
#
# ⚠ AND IT CANNOT SEE THE ONE HAZARD WORTH NAMING, so name it here.
#
# `Cargo.lock` is this gate's oracle, and `Cargo.lock` is not stable between
# two commands in the same session. `crates/pdfcer-gui/Cargo.toml` takes the
# three engine crates as `{ git = …, branch = "main" }` — a BRANCH, with no
# `rev` — and cargo re-resolves such a dependency opportunistically, without
# anybody typing `cargo update`. That was measured on 2026-09-11: the lock
# moved `d2465f5` → `f8f9a26` at 20:04 with no update command in the session's
# history.
#
# So the strictly correct oracle is *the revision compiled into the exe*, and
# what we have is *the revision the lock names now*. The two diverge in the
# window between `cargo build` and this gate. That window is small, the gate
# and `tools/package-portable.py`'s `locked_engine_rev` read the SAME source,
# so `FEATURES.md` and `BUILD-INFO.txt` will at least agree with each other —
# and a shipped pair that agree wrongly is a far smaller problem than a pair
# that disagree. But if this gate ever demands an edit that surprises you,
# **suspect the lock moved under the build** before editing the document:
# re-run the build, then the gate, in that order.
#
# It also does not accept an ABSENT pin as a pass. If a future rewording
# drops the phrase from the header, this gate FAILS and says so in those
# words. A gate keyed on a name is discharged by prose that stops using the
# name, and this repository has been bitten by that too: the remedy is that
# absence is red, never green and never a skip.
#
# ===========================================================================
# SELF-TEST
# ===========================================================================
#
# `--self-test` copies the two documents to a scratch directory, sabotages
# each in turn, and asserts the checker reports the sabotage. A checker that
# has never been observed to fail is not evidence that the thing it checks is
# healthy; it is evidence that it ran.
#
# Exit status: 0 all citations agree with the lock · 1 a citation is stale,
# missing, or the lock itself is not the shape this gate understands.

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
cite_resume() {
    local f="$1"
    [ -f "$f" ] || return 0
    grep -m1 '^| *Engine pin *|' "$f" \
        | grep -o '`[0-9a-f]\{7,40\}`' \
        | tail -1 \
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
exit $?
