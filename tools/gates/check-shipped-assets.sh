#!/usr/bin/env bash
#
# check-shipped-assets.sh — reach the Python gate, or skip loudly.
#
# ===========================================================================
# THE PROPERTY ASSERTED
# ===========================================================================
#
# `check-shipped-assets.py` is either executed, or its non-execution is
# reported as a skip. The licence property itself — that every non-Cargo file
# this project redistributes has its licence recorded, and that the record
# reaches the person handed the binary — is asserted by that file. Read its
# header for what it checks and what it cannot see. This wrapper asserts only
# that `bash <gate>` reaches it.
#
# The gate is Python because its central check reads `PAYLOAD_DOCS` and
# `PAYLOAD_ASSET_DIRS` out of `tools/package-portable.py` by importing the
# module and reading the actual list objects. Reading a Python list with a
# grep is the silently-rotting pattern every gate here exists to avoid.
# `run-all.sh` invokes every gate as `bash <gate>`, and that uniformity is
# worth keeping — a runner with one special case acquires a second — so the
# wrapper absorbs the difference instead.
#
# ===========================================================================
# WHY A HUMAN CANNOT HOLD IT
# ===========================================================================
#
# An absent interpreter looks like nothing at all. The failure is not that
# somebody forgets to check the licences; it is that a shell spawned by
# another program has a different `PATH` from the one a person typed in, and
# nothing in a green run distinguishes "ran and found nothing wrong" from
# "never ran". `tools/package-portable.py` invokes `run-all.sh` through
# `subprocess`, and the bash it spawns is neither a login nor an interactive
# shell, so a tool installed into a profile-appended directory is not on its
# `PATH` — `cargo` is already known to be reached that way and missing here.
# A skip reason is read precisely by somebody who cannot see the machine, so
# it has to name the actual fact: which interpreters were tried, and what was
# therefore not checked.
#
# A gate that exited 0 because it could not run would be the worst available
# outcome — a licence obligation reported as discharged by a check that never
# happened.
#
# ===========================================================================
# WHAT IT PROVABLY CANNOT SEE
# ===========================================================================
#
#   - Anything about the assets. This file reads no asset, no licence and no
#     manifest; every such judgement belongs to the Python gate.
#   - Whether the interpreter it found is new enough, or is the same one the
#     packager will use. First match on `PATH` wins.
#   - Whether `check-shipped-assets.py` is the gate it expects. Only that a
#     file of that name sits beside this one.
#   - Its own absence. A gate dropped from `run-all.sh`'s dispatch list is not
#     skipped, it is unmentioned, and nothing here can notice that.
#
# ===========================================================================
# THE EXIT CONTRACT, AND HOW TO FALSIFY IT
# ===========================================================================
#
#   0  passed through from the Python gate — every licence check held.
#   1  passed through — at least one check failed.
#   2  SKIPPED. The gate file is missing, no interpreter was found, or the
#      Python gate skipped itself. NOT a pass: `run-all.sh` renders skips in
#      their own block and exits 3, so a run containing one cannot be read as
#      green.
#
# Three interpreter spellings are tried because Windows ships the `py`
# launcher, some environments carry only `python3`, and Git Bash usually has
# `python`.
#
# To falsify: run it with `PATH=/nonexistent` — it must print the SKIPPED
# lines and exit 2, never 0. Rename `check-shipped-assets.py` and it must exit
# 2 rather than reporting success for a gate that no longer exists.
set -uo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GATE="$HERE/check-shipped-assets.py"

if [ ! -f "$GATE" ]; then
    echo "shipped-assets: SKIPPED — $GATE is missing."
    echo "  Nothing was checked, and 'nothing checked' is not 'nothing wrong'."
    exit 2
fi

for candidate in python python3 py; do
    if command -v "$candidate" >/dev/null 2>&1; then
        exec "$candidate" "$GATE" "$@"
    fi
done

echo "shipped-assets: SKIPPED — no Python interpreter on PATH."
echo ""
echo "  Tried: python, python3, py."
echo ""
echo "  The asset licensing is NOT implicated: nothing was read, because the"
echo "  tool that would read it was never found. If this ran from a script,"
echo "  the spawned shell probably did not inherit the interpreter's"
echo "  directory — the same shape as run-all.sh's cargo-on-PATH note."
exit 2
