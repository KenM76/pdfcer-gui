#!/usr/bin/env bash
#
# check-unreachable-refusals.sh — wrapper around `check-unreachable-refusals.py`.
#
# ===========================================================================
# WHAT THE GATE IS
# ===========================================================================
#
# This shell keeps refusal sentences alive after the engine stops producing
# the error behind them, and that is usually correct: the `match` over
# `ReflowDecline` is compiler-proved complete so the arm is mandatory,
# `unreachable!()` would turn a future engine reinstating a guard into a crash
# on a refusal path, and the remedy wording took two corrections to get right.
#
# ★★★ The defect is that nobody can tell the sentences are dead. It has cost
# this project three times — twice found by a reader stumbling on it months
# later, and once (2026-09-14, `G015`) where noticing would have depended on
# somebody reading a commit message in a repository this one may not write to.
#
# So a paragraph claiming an engine symbol has no producer must carry a
# machine-readable marker naming that symbol, and this gate re-measures the
# claim against the engine source at the revision `Cargo.lock` pins, on every
# commit. The long argument — including why it DIFFS rather than classifies,
# and exactly which constructs it does not parse — is in the Python file's
# header. Read that, not this.
#
# ===========================================================================
# ★★ WHY THE SELF-TEST RUNS HERE, IN THE SAME FILE
# ===========================================================================
#
# `run-all.sh`'s own rule: a gate that cannot detect its own planted violation
# is worth nothing on the real crate, and finding that out after a green run
# is finding it out too late. This wrapper therefore refuses to measure
# anything if the self-test did not pass, which removes the failure mode where
# a `--self-test` exists, is never registered, and is reachable only by a
# session that already suspected something.
#
# ===========================================================================
# EXIT CODES
# ===========================================================================
#
#     0  PASS      every marker re-measured and unchanged
#     1  FAIL      a marker drifted, is unrecorded, or its symbol is gone
#     2  SKIPPED   no interpreter, no engine, no git, or no pinned revision
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
