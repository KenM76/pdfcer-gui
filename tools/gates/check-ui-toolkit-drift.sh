#!/usr/bin/env bash
# check-ui-toolkit-drift.sh — is this shell still building against the egui it
# thinks it is, and does it know how far behind the published one it has drifted?
#
# ===========================================================================
# ★★★ WHY THIS GATE EXISTS
# ===========================================================================
#
# On 2026-09-08 the operator asked, in one line:
#
#   > *"does our project check for the latest version of egui to compile with?"*
#
# The answer was **no, and worse than no.** Four separate things were true at
# once and nothing in the repository could have told anybody:
#
#   1. `Cargo.toml` requires `egui = "0.35"`, which is `^0.35` — semver-caret,
#      so it resolves `>=0.35.0, <0.36.0`. **`cargo update` can never move it
#      to 0.36.** The pin is not a floor with a moving ceiling; it is a wall,
#      and a `cargo update` that reports "everything is current" is telling
#      the truth about a question nobody meant to ask.
#   2. egui/eframe 0.36.1 had been published, so the drift was one minor
#      release, unnoticed.
#   3. `check-engine-api-drift` exists and does exactly this job for
#      `pdfcer-core` — the crate consumed by PATH, which moves hourly. The
#      toolkit consumed by VERSION, which owns every layout, event and paint
#      decision in the shell, had no equivalent. ⇒ **The dependency that
#      cannot drift silently got a drift gate; the one that can only drift
#      silently did not.** That asymmetry is the finding.
#   4. `[workspace.dependencies]` pinned `egui_tiles = "0.16.0"` and **no
#      crate in the workspace depends on it**, so it is not in `Cargo.lock`
#      at all. The dock was hand-built instead (`egui-shell/src/dock/mod.rs`
#      states the decision). A pinned version nobody uses is a claim about
#      the build that is not true, and `MODES_AND_PANELS.md` was still
#      sourcing its feasibility verdicts to "egui 0.35 / egui_tiles 0.16".
#
# ===========================================================================
# WHAT IT CHECKS, IN THREE PARTS
# ===========================================================================
#
#   A0. **Every egui-family workspace pin resolves to something.** A pin with
#       no consumer is either a dependency that was dropped without its
#       manifest line, or one that is about to be adopted — and the two want
#       opposite actions. `UI_TOOLKIT_PINS.md` is where the answer lives.
#
#   A.  **Coherence, offline, always.** The version `Cargo.lock` resolved must
#       match what the project's own documents claim their verdicts were
#       measured against. This is the half that runs with no network and it
#       is the half that catches a bump landing without its documentation.
#
#   B.  **Drift from crates.io, when the network is there.** How far behind
#       the published release the pin is. **This half never fails for being
#       behind.** A newer egui is not a defect; an *unrecorded* one is. It
#       fails only when the drift is absent from `UI_TOOLKIT_PINS.md`.
#
# ⚠ It does NOT try to upgrade, and being behind is not a failure. This
# project ships a binary the operator uses daily on real drawings; an egui
# minor is a change to every layout, event and paint path in the shell, and
# the standing rule is that such a thing is verified by DRIVING the binary,
# not by a green `cargo check`. The gate's job is to make sure nobody can be
# *surprised* — not to make the decision.
#
# ===========================================================================
# THE THREE OUTCOMES
# ===========================================================================
#
#   PASS  the lock, the manifest and the docs agree, and any drift is
#         recorded in UI_TOOLKIT_PINS.md with a reason
#   SKIP  no network — part B could not run; A0 and A still ran and passed.
#         ★ Printed as SKIP rather than PASS deliberately: a check that
#         quietly downgrades to "the part I could do" is how a gate stops
#         running and nobody notices. This project has the receipts.
#   FAIL  the lock disagrees with the manifest or the docs, a pin resolves to
#         nothing without being declared unused, or crates.io has a newer
#         release UI_TOOLKIT_PINS.md does not mention
#
set -uo pipefail

cd "$(dirname "$0")/../.." || exit 2

MANIFEST="Cargo.toml"
LOCK="Cargo.lock"
PINS="UI_TOOLKIT_PINS.md"

fail=0
skipped=0

if [ ! -f "$PINS" ]; then
    echo "ui-toolkit: FAIL — $PINS does not exist."
    echo "  It is where the decision to stay on a version lives. Without it,"
    echo "  'we are one release behind' and 'nobody looked' are the same state."
    exit 1
fi

# The version `Cargo.lock` actually resolved. This is the only version that is
# TRUE: the manifest states a REQUIREMENT, the lock states a FACT, and they
# answer different questions.
locked_version() {
    awk -v want="$1" '
        $0 == "name = \"" want "\"" { found = 1; next }
        found && $1 == "version" { gsub(/"/, "", $3); print $3; exit }
    ' "$LOCK"
}

# Every egui-family crate named in `[workspace.dependencies]`. Derived from the
# manifest rather than hard-coded, so adopting `egui_extras` or `egui_plot`
# brings it under this gate without anybody remembering to add it here.
#
# ★ A hand-written list inside a completeness check is the gap it was built to
# find — this project has the receipt for that too.
PINNED="$(awk '
    /^\[workspace\.dependencies\]/ { in_ws = 1; next }
    /^\[/ { in_ws = 0 }
    in_ws && /^(egui|eframe)/ && !/path *=/ { sub(/ .*/, "", $0); print }
' "$MANIFEST" | sed 's/=.*//' | tr -d ' ' | sort -u)"

if [ -z "$PINNED" ]; then
    echo "ui-toolkit: FAIL — no egui-family pin found in [workspace.dependencies]."
    echo "  Either the manifest was restructured or this scan is reading nothing,"
    echo "  and a scan that reads nothing reports success."
    exit 1
fi

echo "ui-toolkit: pins declared in $MANIFEST, resolved through $LOCK"
declare -A LOCKED
for c in $PINNED; do
    v="$(locked_version "$c")"
    if [ -z "$v" ]; then
        # A0: pinned with no consumer.
        if grep -qE "^\| *\`?$c\`? *\|.*(unused|not used|no consumer)" "$PINS"; then
            printf '  %-12s (pinned, no consumer — declared in %s)\n' "$c" "$PINS"
        else
            echo "  FAIL: $c is pinned in $MANIFEST but resolves to nothing"
            echo "        No crate in the workspace depends on it, so the pin is"
            echo "        a claim about the build that is not true. Say which it"
            echo "        is in $PINS — dropped, or not yet adopted."
            fail=1
        fi
        continue
    fi
    LOCKED[$c]="$v"
    printf '  %-12s %s\n' "$c" "$v"
done

# --- part A: the docs' claim about the measured platform ------------------
#
# The docs make a claim about which toolkit the feasibility verdicts were
# measured against. A bump that leaves that sentence behind turns a measured
# verdict into an unsourced one — the defect class this project has corrected
# more times than any other.
#
# ★ Matched on MAJOR.MINOR. A patch bump changes no verdict, and requiring a
# doc edit for one would train everybody to edit the sentence without reading
# it, which is worse than not checking.
for doc in MODES_AND_PANELS.md; do
    [ -f "$doc" ] || continue
    for c in $PINNED; do
        [ -n "${LOCKED[$c]+set}" ] || continue
        want="${LOCKED[$c]%.*}"
        # Only enforce where the document already names the crate — this gate
        # does not decide which documents must cite a version.
        if grep -qE "\b$c [0-9]+\.[0-9]+" "$doc" && ! grep -qF "$c $want" "$doc"; then
            echo "  FAIL: $doc names a $c version, and it is not $want"
            echo "        The lock resolved ${LOCKED[$c]}; the document sources its"
            echo "        verdicts to something else."
            fail=1
        fi
    done
done

# --- part B: drift from crates.io -----------------------------------------

echo "ui-toolkit: latest published"
for c in $PINNED; do
    [ -n "${LOCKED[$c]+set}" ] || continue
    # `cargo search` needs the network. A failure here is "offline", not
    # "clean" — and it must SAY so, because a check that reports green while
    # measuring nothing is how a gate stops running unnoticed.
    line="$(cargo search "$c" --limit 1 2>/dev/null | head -1)"
    v="$(printf '%s' "$line" | sed -n "s/^$c = \"\([0-9.]*\)\".*/\1/p")"
    if [ -z "$v" ]; then
        echo "  SKIP: could not reach crates.io for $c"
        skipped=1
        continue
    fi
    if [ "$v" = "${LOCKED[$c]}" ]; then
        printf '  %-12s %s (current)\n' "$c" "$v"
    else
        printf '  %-12s %s   <- pinned at %s\n' "$c" "$v" "${LOCKED[$c]}"
        # ★ Matched on MAJOR.MINOR, not on the exact version, and the
        # reason is that egui publishes patches often — 0.36.1 became 0.36.2
        # during the hour this gate was written. An exact match would go red
        # on every patch release, and a gate that goes red for no reason
        # trains everybody to edit the file it points at without reading it.
        # A new MINOR is the event that deserves a decision.
        # ★★ It reads the REGISTER TABLE's own row, not the file at large.
        # The first draft grepped the whole document for "egui 0.36" and
        # passed — on a sentence in the prose that happened to contain those
        # words. That is a gate discharged by narrative, which this project
        # has already been bitten by once (25 verbs scored "consumed" on doc
        # comments). The row is the contract; the prose is the argument.
        if ! grep -qE "^\| *\`?$c\`? *\|[^|]*\|[^|]*${v%.*}" "$PINS"; then
            echo "  FAIL: $c ${v%.*}.x is published and $PINS has no register row for it"
            echo "        Staying behind is a decision. A decision nobody wrote"
            echo "        down is indistinguishable from not having noticed."
            fail=1
        fi
    fi
done

if [ "$fail" -ne 0 ]; then
    cat <<'EOT'

ui-toolkit: FAIL

  Either the lock, the manifest and the docs disagree about which toolkit this
  shell is built against, or a newer one is published and UI_TOOLKIT_PINS.md
  does not say so.

  This gate never asks you to upgrade. It asks you to KNOW. Add a row to
  UI_TOOLKIT_PINS.md naming the version and the reason — "measured, not yet
  verified by driving the binary" is a perfectly good reason and the usual one.
EOT
    exit 1
fi

if [ "$skipped" -ne 0 ]; then
    echo "ui-toolkit: SKIP — the lock and the docs agree, but crates.io was unreachable"
    exit 0
fi

echo "ui-toolkit: clean"
exit 0
