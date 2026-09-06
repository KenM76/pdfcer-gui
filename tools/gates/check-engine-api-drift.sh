#!/usr/bin/env bash
#
# check-engine-api-drift.sh — wrapper around `check-engine-api-drift.py`.
#
# ===========================================================================
# WHAT THE GATE IS
# ===========================================================================
#
# It enumerates EVERY public item — struct, enum, trait, variant, field,
# method, free function, const — in every engine crate this shell depends on,
# at the revision `Cargo.lock` pins, and fails when the engine has grown one
# that this repository names nowhere and says nothing about.
#
# ★★★ It exists because on 2026-09-04 `pdfcer_core::text_edit::RefusalKind`
# landed in answer to this project's own request, sat unconsumed for a day
# beside a function whose doc comment said it was "written to be deleted" the
# day that type arrived, and **two gates aimed at exactly this question stayed
# silent** — `check-verb-coverage.sh` and `check-engine-backlog.sh` are both
# keyed on `EditSession`'s verbs, so a new TYPE, a new VARIANT, a new FIELD or
# a new FREE FUNCTION is invisible to both.
#
# The long argument, the exact failure modes and the reason for every design
# choice are in the Python file's header. **Read that, not this.** This file's
# whole job is: find an interpreter, run the self-test, run the gate, or say
# honestly why none of that happened.
#
# ===========================================================================
# ★★ WHY THE SELF-TEST RUNS HERE AND NOT AS ITS OWN `run-all.sh` ENTRY
# ===========================================================================
#
# `run-all.sh`'s header states the rule this gate obeys:
#
#   > The self-tests run FIRST, before any gate is trusted. If a gate cannot
#   > detect its own planted violation, its verdict on the real crate is worth
#   > nothing, and finding that out after a green run is finding it out too
#   > late.
#
# Every other gate satisfies that with two dispatch lines. This one satisfies
# it with one, by running `--self-test` itself and refusing to measure anything
# if the self-test did not pass. The reason is not tidiness: it removes the
# failure mode where a gate is registered and its self-test is not — which has
# already happened here once, and is recorded in `run-all.sh`'s own header as
# the paragraph that "was off by one, then by two".
#
# ⇒ The ordering guarantee is IN THE GATE, so it cannot be lost by an edit to
#   the runner.
#
# ===========================================================================
# "PYTHON IS NOT ON PATH" IS NOT "THE ENGINE HAS ADDED NOTHING"
# ===========================================================================
#
# The lesson is already written down in `run-all.sh`, against `cargo`, and
# again in `check-shipped-assets.sh`, against this same interpreter question:
#
#   > A skip reason is read precisely when someone cannot see the machine. It
#   > has to name the actual fact.
#
# `tools/package-portable.py` runs `run-all.sh` through `subprocess`, and the
# bash it spawns is neither a login nor an interactive shell — it has already
# been observed not to inherit `~/.cargo/bin`. Three interpreter spellings are
# tried because Windows ships the `py` launcher, some environments have only
# `python3`, and Git Bash usually has `python`.
#
# ===========================================================================
# EXIT CODES — passed through from the Python gate, plus:
#   0  PASS     every item the engine gained is accounted for
#   1  FAIL     something new is unconsumed and unacknowledged, OR the scan
#               collapsed, OR the self-test could not detect its own plants
#   2  SKIPPED  no interpreter, no gate file, no engine checkout, no snapshot.
#               NOT a pass — `run-all.sh` renders it separately and the whole
#               run exits 3.
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
# --self-test all reach the gate unchanged when a human drives it by hand.
if [ "$#" -gt 0 ]; then
    exec "$PY" "$GATE" "$@"
fi

# ★ The self-test first, and its verdict is a precondition rather than a
# report. A gate that cannot detect its own planted violation has nothing worth
# saying about the engine, so this does not fall through to the measurement.
if ! "$PY" "$GATE" --self-test; then
    echo ""
    echo "engine-api-drift: FAILED — the self-test above did not pass, so the"
    echo "  measurement was NOT run. A gate that cannot be seen to fail is a"
    echo "  rumour, and its verdict on the real engine is worth nothing."
    exit 1
fi
echo ""

exec "$PY" "$GATE"
