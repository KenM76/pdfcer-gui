#!/usr/bin/env bash
#
# check-engine-rename-shim.sh — the tripwire on a temporary shim, which names
# its own deletion.
#
# ═══════════════════════════════════════════════════════════════════════════
# WHAT IT GUARDS
# ═══════════════════════════════════════════════════════════════════════════
#
# This project renamed to `pdfcer` on 2026-09-03, BEFORE the engine did. The
# engine's own rename is its `Pass 247.1`, which clones `D:\Dev\pdfce` to
# `D:\Dev\pdfcer` and renames `pdfce-core` / `pdfce-render` / `pdfce-print`.
#
# Until that lands, `crates/pdfcer-gui/Cargo.toml` carries three
# `package = "pdfce-*"` keys: the dependency is NAMED `pdfcer-core` locally, so
# every `use pdfcer_core::…` in the source is the final spelling, and RESOLVES
# to the crate that exists today. One `is_pdfce_choice` call site is spelled the
# old way for the same reason — that method belongs to the engine.
#
# ★★★ WHY THIS IS A GATE AND NOT A COMMENT
#
# Because this project has paid, repeatedly, for the difference. Its own record:
#
#   · "A note is not a mechanism." A `Pass 182.0` reply sat unconsumed for two
#     days while a ribbon control stayed greyed and a dialog told the operator
#     something untrue — with 2,181 tests passing. What fixed it was
#     `check-verb-coverage.sh`, not the note.
#   · "A temporary shim needs a tripwire that names its own deletion." A
#     `debug_assert` on the condition that makes a shim unnecessary fired two
#     hours after it was written.
#   · "Delete the workaround when the cause is removed." The engine answers
#     within the hour; a mechanism with no remaining cause rots in place.
#
# A shim whose removal depends on somebody remembering is a shim that becomes
# permanent. This one fails the build the moment it is unnecessary.
#
# ═══════════════════════════════════════════════════════════════════════════
# WHAT IT DOES
# ═══════════════════════════════════════════════════════════════════════════
#
# FAILS when `D:/Dev/pdfcer` exists AND the shim is still in place — because at
# that moment the shim has stopped being a bridge and started being a lie about
# which crate this builds against.
#
# PASSES, quietly, in the two states that are honest:
#
#   1. the engine has not renamed and the shim is present  — the state today;
#   2. the engine has renamed and the shim is gone         — the destination.
#
# ★ It also fails the OTHER way: shim gone while the engine's clone is absent
# means the manifest names a repository that does not exist, and the failure a
# reader would otherwise get is a Cargo error about a missing git remote — which
# says nothing about why.
#
# ═══════════════════════════════════════════════════════════════════════════
# WHEN IT FIRES, THIS IS THE WHOLE FIX
# ═══════════════════════════════════════════════════════════════════════════
#
#   1. `crates/pdfcer-gui/Cargo.toml` — delete the three `package = "pdfce-*",`
#      fragments and change `file:///D:/Dev/pdfce` to `file:///D:/Dev/pdfcer`
#      on the same three lines. Delete the shim comment block above them.
#   2. `crates/pdfcer-gui/src/panels/forms/mod.rs` — `is_pdfce_choice` becomes
#      `is_pdfcer_choice`, and its explaining comment goes.
#   3. `cargo update -p pdfcer-core -p pdfcer-render -p pdfcer-print`
#   4. Delete this gate and its line in `run-all.sh`.
#
# The engine said it would post the exact commit when `247.1` lands. Take that
# commit, not `main`, so the version this shell builds against is one somebody
# named.

set -uo pipefail

MANIFEST="crates/pdfcer-gui/Cargo.toml"
ENGINE_NEW="/d/Dev/pdfcer"

if [[ ! -f "$MANIFEST" ]]; then
    echo "engine-rename-shim: FAIL — $MANIFEST not found; run from the repository root."
    exit 1
fi

# The shim is present if any dependency line still maps a `pdfcer-*` name onto a
# `pdfce-*` package. Matched on the `package =` key rather than on the whole
# line, so reformatting the manifest cannot make this gate blind.
SHIM_LINES=$(grep -cE '^[[:space:]]*pdfcer-(core|render|print)[[:space:]]*=.*package[[:space:]]*=[[:space:]]*"pdfce-' "$MANIFEST" || true)

# ★★★ THE CONDITION IS THE ENGINE'S *CRATE*, NOT ITS DIRECTORY, and the first
# version of this gate got that wrong within the hour.
#
# It tested `-d D:/Dev/pdfcer` and fired the moment that folder appeared. But
# the engine's rename is two Passes: `247.0` CLONES the tree, `247.1` renames
# the crates inside it. Between them — which is where this gate first fired —
# `D:\Dev\pdfcer` exists and still contains `crates/pdfce-core`, so the shim
# was still needed and the gate said it had outlived its cause.
#
# ⇒ A proxy condition standing in for the real one, which is precisely the
# family this file was written to guard. The real question is "has the engine
# crate been renamed", and the only thing that answers it is the crate.
ENGINE_RENAMED=0
[[ -d "$ENGINE_NEW/crates/pdfcer-core" ]] && ENGINE_RENAMED=1

if [[ "$ENGINE_RENAMED" -eq 1 && "$SHIM_LINES" -gt 0 ]]; then
    cat <<'MSG'
engine-rename-shim: FAIL — the shim has outlived its cause.

D:\Dev\pdfcer/crates/pdfcer-core now EXISTS, so the engine's rename has landed
and this project is
still building against the old `pdfce-*` crates through the `package =` bridge
in crates/pdfcer-gui/Cargo.toml.

That is no longer a bridge. It is a manifest that says `pdfcer-core` and links
something else, which is exactly the kind of quiet divergence this project
writes gates to prevent.

The fix is four steps and they are listed in this script's header. The engine
said it would post the exact commit for `Pass 247.1`; use that commit rather
than `main`.
MSG
    exit 1
fi

if [[ "$ENGINE_RENAMED" -eq 0 && "$SHIM_LINES" -eq 0 ]]; then
    cat <<'MSG'
engine-rename-shim: FAIL — the shim is gone and the engine has not renamed.

crates/pdfcer-gui/Cargo.toml names `pdfcer-core` / `pdfcer-render` /
`pdfcer-print` with no `package =` bridge, and the renamed engine crate is not
there —
so the manifest points at a repository that is not there.

Cargo's own error for this talks about a git remote and says nothing about why,
which is the reason this gate exists. Either restore the three `package =`
fragments, or wait for the engine's `Pass 247.1`.
MSG
    exit 1
fi

if [[ "$ENGINE_RENAMED" -eq 1 ]]; then
    echo "engine-rename-shim: clean — the engine has renamed and the shim is gone."
else
    echo "engine-rename-shim: clean — the shim is in place and still needed"
    echo "                    ($SHIM_LINES dependency line(s); D:\\Dev\\pdfcer does not exist yet)."
fi
exit 0
