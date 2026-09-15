#!/usr/bin/env bash
#
# check-engine-api-drift.sh — wrapper around `check-engine-api-drift.py`.
#
# ===========================================================================
# THE PROPERTY ASSERTED
# ===========================================================================
#
# Every public item the engine has gained — struct, enum, trait, variant,
# field, method, free function, const, in every engine crate this shell
# depends on, at the revision `Cargo.lock` pins — is either named somewhere in
# this repository or written down as deliberately unconsumed.
#
# The measurement, its failure modes and the reason for every design choice
# are in the Python file's header. READ THAT, NOT THIS. This file's whole job
# is: find an interpreter, run the self-test, run the gate, or say honestly why
# none of that happened.
#
# ===========================================================================
# WHY A HUMAN CANNOT HOLD IT
# ===========================================================================
#
# The sibling gates are keyed on `EditSession`'s verbs, so a new TYPE, a new
# VARIANT, a new FIELD or a new FREE FUNCTION is invisible to every one of
# them. An engine can therefore grow an entire vocabulary — a refusal kind, a
# report struct, a builder option — while every instrument aimed at the engine
# stays green, because none of them is keyed on the thing that grew.
#
# Nothing in this tree changes on the day it happens. The only evidence is a
# commit in a repository this one does not write to, reaching here as a hash in
# a lock file. Noticing therefore depends on somebody re-reading, without
# prompting, prose that reads as current — including the doc comment that says
# a workaround was "written to be deleted" on exactly the day the replacement
# arrives. That is not something a person can be asked to do reliably.
#
# ===========================================================================
# WHY THE SELF-TEST RUNS HERE AND NOT AS ITS OWN `run-all.sh` ENTRY
# ===========================================================================
#
# The runner's rule is that self-tests run before any gate's verdict is
# trusted: a gate that cannot detect its own planted violation has nothing
# worth saying about the real tree, and learning that after a green run is
# learning it too late.
#
# Every other gate satisfies that with two dispatch lines. This one satisfies
# it with one, by running `--self-test` itself and refusing to measure anything
# if the self-test did not pass. That is not tidiness: it removes the failure
# mode where a gate is registered and its self-test is not. The ordering
# guarantee lives IN THE GATE, so it cannot be lost by an edit to the runner.
#
# ===========================================================================
# WHAT IT PROVABLY CANNOT SEE
# ===========================================================================
#
#   * Everything the Python gate cannot see, unchanged — this wrapper adds no
#     measurement of its own.
#   * Its own absence from `run-all.sh`. A gate that is not dispatched is not
#     skipped; it is unmentioned, and nothing here notices that.
#   * Whether the interpreter it found is the one the rest of the toolchain
#     uses. It takes the first of `python`, `python3`, `py` that answers.
#
# "PYTHON IS NOT ON PATH" IS NOT "THE ENGINE HAS ADDED NOTHING", and the skip
# text below says so in as many words. A skip reason is read precisely when
# nobody can see the machine, so it has to name the actual fact. Three
# interpreter spellings are tried because Windows ships the `py` launcher, some
# environments have only `python3`, and Git Bash usually has `python`; and a
# bash spawned from another tool through `subprocess` is neither a login nor an
# interactive shell, so it need not have inherited the interpreter's directory
# at all.
#
# ===========================================================================
# THE EXIT CONTRACT, AND HOW TO FALSIFY IT
# ===========================================================================
#
# Passed through from the Python gate, plus this file's own skips:
#
#   0  PASS     every item the engine gained is accounted for
#   1  FAIL     something new is unconsumed and unacknowledged, OR the scan
#               collapsed, OR the self-test could not detect its own plants
#   2  SKIPPED  no interpreter, no gate file, no engine checkout, no snapshot.
#               NOT a pass — `run-all.sh` renders skips separately and the
#               whole run exits 3.
#
# Arguments are passed straight through and BYPASS the self-test, so a human
# driving `--update`, `--list` or `--bootstrap` by hand gets the gate's own
# behaviour; the self-test precondition applies to the unattended run.
#
# To falsify this wrapper specifically: run it with `PATH=/nonexistent` and it
# must print SKIPPED and exit 2. Rename the `.py` beside it and it must exit 2
# rather than 0. Break the Python gate's self-test and it must exit 1 without
# measuring anything.
# ===========================================================================
set -uo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GATE="$HERE/check-engine-api-drift.py"

if [ ! -f "$GATE" ]; then
    echo "engine-api-drift: SKIPPED — $GATE is missing."
    echo "  Nothing was measured, and 'nothing measured' is not 'nothing wrong'."
    exit 2
fi

PY=""
for candidate in python python3 py; do
    if command -v "$candidate" >/dev/null 2>&1; then
        PY="$candidate"
        break
    fi
done

if [ -z "$PY" ]; then
    echo "engine-api-drift: SKIPPED — no Python interpreter on PATH."
    echo ""
    echo "  Tried: python, python3, py."
    echo ""
    echo "  The engine's API is NOT implicated: nothing was read, because the"
    echo "  tool that would read it was never found. If this ran from a script,"
    echo "  the spawned shell probably did not inherit the interpreter's"
    echo "  directory — the same shape as run-all.sh's cargo-on-PATH note."
    exit 2
fi

# Any argument is passed straight through, so --update, --list, --bootstrap and
# --self-test all reach the gate unchanged when a human drives it by hand. Note
# that this path does not run the self-test first: an argument means a person is
# driving, and a person can read the gate's own output.
if [ "$#" -gt 0 ]; then
    exec "$PY" "$GATE" "$@"
fi

# The self-test first, and its verdict is a precondition rather than a report.
# A gate that cannot detect its own planted violation has nothing worth saying
# about the engine, so this does not fall through to the measurement.
if ! "$PY" "$GATE" --self-test; then
    echo ""
    echo "engine-api-drift: FAILED — the self-test above did not pass, so the"
    echo "  measurement was NOT run. A gate that cannot be seen to fail is a"
    echo "  rumour, and its verdict on the real engine is worth nothing."
    exit 1
fi
echo ""

exec "$PY" "$GATE"
