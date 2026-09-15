#!/usr/bin/env bash
#
# check-unreachable-refusals.sh — run `check-unreachable-refusals.py`, after
# proving it can fail.
#
# ===========================================================================
# THE PROPERTY ASSERTED
# ===========================================================================
#
# Every paragraph in this shell claiming that an engine symbol has no producer
# carries a machine-readable marker naming that symbol, and the claim still
# holds against the engine source at the revision `Cargo.lock` pins.
#
# The measurement itself, the reason it DIFFS rather than classifies, and the
# exact Rust constructs its scanner does not parse are in the Python file's
# header. Read that for the gate; this file is only the wrapper.
#
# ===========================================================================
# WHY A HUMAN CANNOT HOLD IT
# ===========================================================================
#
# A refusal sentence outlives the error behind it, and keeping it is usually
# correct: the `match` over `ReflowDecline` is compiler-proved complete so the
# arm is mandatory, and `unreachable!()` would turn a future engine
# reinstating the guard into a crash on a refusal path. So the dead sentence
# stays, rightly, and becomes invisible. Nothing in this repository changes on
# the day the engine stops producing the error — the only evidence is a commit
# in a repository this one does not write to — so noticing depends on somebody
# happening to re-read prose that reads as current. That is not a thing a
# person can be asked to do reliably; it is a thing an instrument re-measures
# on every commit.
#
# ===========================================================================
# WHAT IT PROVABLY CANNOT SEE
# ===========================================================================
#
#   - A dead refusal with no marker. The gate re-measures claims that were
#     written down; an unmarked paragraph is outside its corpus entirely.
#   - Whether the surviving sentence is worded right for the state it now
#     describes. It measures whether a producer exists, not phrasing.
#   - The engine's working tree. Its oracle is the locked revision, because
#     that is the API this shell could actually be compiled against.
#   - Its own absence from `run-all.sh`. A gate that is not dispatched is not
#     skipped; it is unmentioned, and nothing here notices that.
#
# ===========================================================================
# THE EXIT CONTRACT, AND HOW TO FALSIFY IT
# ===========================================================================
#
#     0  PASS      every marker re-measured and unchanged
#     1  FAIL      a marker drifted, is unrecorded, its symbol is gone, or the
#                  self-test could not detect its own planted violation
#     2  SKIPPED   no interpreter, no engine, no git, or no pinned revision.
#                  NOT a pass — `run-all.sh` prints skips in their own block
#                  and exits 3, so a run containing one is incomplete.
#
# The self-test runs here rather than as a second `run-all.sh` entry, and its
# verdict is a precondition rather than a report: a gate that cannot detect
# its own planted violation has no verdict on the real tree worth reading, and
# learning that after a green run is learning it too late. Keeping the
# ordering inside the gate removes the failure mode where a gate is registered
# and its self-test is not.
#
# To falsify: write an `UNREACHABLE-FROM:` marker naming a symbol the engine
# still produces — this must exit 1. Run it with `PATH=/nonexistent` and it
# must print SKIPPED and exit 2. Break the Python scanner and the self-test
# must refuse the run rather than report on it.
#
set -uo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"

# ★ Find an interpreter the same way the other Python gates do. A missing
#   Python is a SKIP and says so: it is a real state on a fresh machine, and
#   reporting it as a pass is the precise failure this directory has a rule
#   about.
PY=""
for cand in python python3 py; do
  if command -v "$cand" >/dev/null 2>&1; then PY="$cand"; break; fi
done
if [ -z "$PY" ]; then
  echo "SKIPPED: check-unreachable-refusals — no python interpreter on PATH."
  exit 2
fi

# UTF-8 is not optional: the gate prints ★ and ⇒, and a Windows console
# defaulting to cp1252 turns a real finding into a UnicodeEncodeError
# traceback, which reads as a broken tool and sends the reader to debug the
# instrument while the named row sits unfixed.
export PYTHONIOENCODING=utf-8

cd "$ROOT" || exit 2

if ! "$PY" "$HERE/check-unreachable-refusals.py" --self-test; then
  echo "check-unreachable-refusals: the self-test FAILED, so the real run is"
  echo "  not trusted and was not made. Fix the gate before reading its verdict."
  exit 1
fi

"$PY" "$HERE/check-unreachable-refusals.py" "$@"
exit $?
