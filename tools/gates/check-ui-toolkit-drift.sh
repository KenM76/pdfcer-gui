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
#   0  PASS  the lock, the manifest and the docs agree, and any drift is
#            recorded in UI_TOOLKIT_PINS.md with a reason
#   1  FAIL  the lock disagrees with the manifest or the docs, a pin resolves
#            to nothing without being declared unused, or crates.io has a
#            newer release UI_TOOLKIT_PINS.md does not mention
#   2  SKIP  no network — part B could not run; A0 and A still ran and passed
#
# ★ The SKIP is exit **2**, and it used to be exit 0. The header above it has
# always said that printing SKIP rather than PASS was deliberate, "because a
# check that quietly downgrades to the part I could do is how a gate stops
# running and nobody notices" — and then it exited 0, which is the only thing
# `run-all.sh` reads. The word landed in the passed column. So on any offline
# run the entire crates.io half — the half the operator's question was about —
# did not run and the roster said PASS. A three-state model stated in prose
# and not in the exit code is a two-state model with a comment.
#
# ===========================================================================
# HOW TO FALSIFY
# ===========================================================================
#
#   bash tools/gates/check-ui-toolkit-drift.sh --self-test
#
# Twelve arms against synthetic roots and a stub crates.io, so it costs no
# network and does not read this repository. Two seams make that possible,
# both the same shape as `PDFCER_VERB_INSTRUMENT` next door:
#
#   PDFCER_TOOLKIT_ROOT     directory to read the four files from
#   PDFCER_TOOLKIT_SEARCH   a program run as `<prog> <crate>`, standing in
#                           for `cargo search` and printing its output shape
#
set -uo pipefail

if [ "${1:-}" = "--self-test" ]; then
    SELF="$(cd "$(dirname "$0")" && pwd)/$(basename "$0")"
    TD="$(mktemp -d)"
    trap 'rm -rf "$TD"' EXIT
    fails=0

    # --- a fixture root -----------------------------------------------------
    #
    # Four files, because the gate reads four and they answer four different
    # questions: the manifest states a REQUIREMENT, the lock states the FACT,
    # MODES_AND_PANELS.md makes the claim part A checks, and
    # UI_TOOLKIT_PINS.md is the register that part B demands a row in.
    # The register arrives on stdin so every arm can state its own row.
    mkroot() {   # mkroot <name> <locked-egui|none> <doc-claim|none>  <<'EOT' register EOT
        local d="$TD/$1"
        mkdir -p "$d"
        printf '[workspace.dependencies]\negui = { version = "0.36" }\n' > "$d/Cargo.toml"
        if [ "$2" = none ]; then
            printf '[[package]]\nname = "serde"\nversion = "1.0.0"\n' > "$d/Cargo.lock"
        else
            printf '[[package]]\nname = "egui"\nversion = "%s"\n' "$2" > "$d/Cargo.lock"
        fi
        if [ "$3" = none ]; then
            echo 'This document names no toolkit version at all.' > "$d/MODES_AND_PANELS.md"
        else
            echo "Feasibility verdicts measured against egui $3." > "$d/MODES_AND_PANELS.md"
        fi
        cat > "$d/UI_TOOLKIT_PINS.md"
    }

    # --- a stub crates.io ---------------------------------------------------
    #
    # `cargo search` prints `egui = "0.36.2"    # An immediate mode GUI`, and
    # the gate parses the version off the front of that line. The stubs print
    # exactly that shape; the offline one prints nothing, which is also what
    # `cargo search` does with no network.
    stub() {   # stub <name> <version|offline>
        if [ "$2" = offline ]; then
            printf '#!/usr/bin/env bash\nexit 0\n' > "$TD/$1"
        else
            printf '#!/usr/bin/env bash\nprintf "%%s = \\"%s\\"    # stub\\n" "$1"\n' "$2" > "$TD/$1"
        fi
        chmod +x "$TD/$1"
    }
    stub pub0361 0.36.1
    stub pub0362 0.36.2
    stub pub0370 0.37.0
    stub offline offline

    arm() {   # arm <label> <expected-rc> <root> [search-stub]
        local got=0 search=""
        [ -n "${4:-}" ] && search="$TD/$4"
        PDFCER_TOOLKIT_ROOT="$TD/$3" PDFCER_TOOLKIT_SEARCH="$search" \
            bash "$SELF" >/dev/null 2>&1 || got=$?
        if [ "$got" -eq "$2" ]; then
            printf '  ok    %-42s rc=%d\n' "$1" "$got"
        else
            printf '  FAIL  %-42s rc=%d, expected %d\n' "$1" "$got" "$2"
            fails=$((fails + 1))
        fi
    }

    # 1. the unplanted control.
    mkroot agree 0.36.1 0.36 <<'EOT'
| crate | pinned | published | status |
|---|---|---|---|
| `egui` | 0.36.1 | 0.36 | current |
EOT
    arm "the lock, the doc and the register agree" 0 agree pub0361

    # 2. part A. A bump that leaves the sentence behind turns a measured
    #    verdict into an unsourced one.
    mkroot stale 0.36.1 0.35 <<'EOT'
| crate | pinned | published | status |
|---|---|---|---|
| `egui` | 0.36.1 | 0.36 | current |
EOT
    arm "a doc sourcing verdicts to a stale minor" 1 stale pub0361

    # 3. and the limit of part A, stated: this gate does not decide which
    #    documents must cite a version, only that one which does is right.
    mkroot silent 0.36.1 none <<'EOT'
| crate | pinned | published | status |
|---|---|---|---|
| `egui` | 0.36.1 | 0.36 | current |
EOT
    arm "a doc naming no version is not its business" 0 silent pub0361

    # 4/5. A0 — a pin with no consumer is either a dependency dropped without
    #    its manifest line or one about to be adopted, and the two want
    #    opposite actions, so the register must say which.
    mkroot orphan none none <<'EOT'
| crate | pinned | published | status |
|---|---|---|---|
| `egui` | 0.36.0 | 0.36 | current |
EOT
    arm "a pin resolving to nothing, undeclared" 1 orphan

    mkroot orphan-ok none none <<'EOT'
| crate | pinned | published | status |
|---|---|---|---|
| `egui` | 0.36.0 | 0.36 | pinned but unused — no consumer, absent from `Cargo.lock` |
EOT
    arm "a pin resolving to nothing, declared unused" 0 orphan-ok

    # 6/7. part B. Being behind is not a defect; being behind unrecorded is.
    mkroot newminor 0.36.1 0.36 <<'EOT'
| crate | pinned | published | status |
|---|---|---|---|
| `egui` | 0.36.1 | 0.36 | current |
EOT
    arm "a new MINOR with no register row" 1 newminor pub0370

    mkroot newminor-ok 0.36.1 0.36 <<'EOT'
| crate | pinned | published | status |
|---|---|---|---|
| `egui` | 0.36.1 | 0.37 | deliberately behind — not yet driven |
EOT
    arm "a new MINOR with a register row" 0 newminor-ok pub0370

    # 8. ★ the window edge, and the reason the gate matches on MAJOR.MINOR:
    #    egui publishes patches often — 0.36.1 became 0.36.2 during the hour
    #    the gate was written. A gate that goes red for no reason trains
    #    everybody to edit the file it points at without reading it. Tightening
    #    the match to the exact version reddens this arm and only this arm.
    mkroot newpatch 0.36.1 0.36 <<'EOT'
| crate | pinned | published | status |
|---|---|---|---|
| `egui` | 0.36.1 | 0.36 | current |
EOT
    arm "a new PATCH must not go red" 0 newpatch pub0362

    # 9. ★★ the register TABLE discharges, not the document. The first draft
    #    of part B grepped the whole file and passed on a sentence in the prose
    #    that happened to contain the words. A gate discharged by narrative is
    #    a gate that can be argued with.
    mkroot prose 0.36.1 0.36 <<'EOT'
egui 0.37 is published and we have decided to stay where we are for now.

| crate | pinned | published | status |
|---|---|---|---|
| `egui` | 0.36.1 | 0.36 | current |
EOT
    arm "prose cannot discharge the register row" 1 prose pub0370

    # 10. the defect this self-test was written for.
    mkroot nonet 0.36.1 0.36 <<'EOT'
| crate | pinned | published | status |
|---|---|---|---|
| `egui` | 0.36.1 | 0.36 | current |
EOT
    arm "crates.io unreachable is not a pass" 2 nonet offline

    # 11. a scan that reads nothing reports success unless it is made not to.
    mkroot nopin 0.36.1 0.36 <<'EOT'
| crate | pinned | published | status |
|---|---|---|---|
| `egui` | 0.36.1 | 0.36 | current |
EOT
    printf '[workspace.dependencies]\nserde = "1"\n' > "$TD/nopin/Cargo.toml"
    arm "no egui-family pin in the manifest" 1 nopin pub0361

    # 12. without the register, "we are one release behind" and "nobody
    #     looked" are the same state.
    mkroot noreg 0.36.1 0.36 <<'EOT'
placeholder
EOT
    rm -f "$TD/noreg/UI_TOOLKIT_PINS.md"
    arm "the register absent" 1 noreg pub0361

    if [ "$fails" -ne 0 ]; then
        echo "check-ui-toolkit-drift --self-test: FAIL — $fails arm(s) disagreed."
        exit 1
    fi
    echo "check-ui-toolkit-drift --self-test: PASS — 12 arms: part A's claim and"
    echo "  its limit, A0 both ways, a new minor both ways, a patch that must"
    echo "  stay green, prose that cannot discharge a row, and an unreachable"
    echo "  crates.io that exits 2 rather than passing."
    exit 0
fi

cd "${PDFCER_TOOLKIT_ROOT:-$(dirname "$0")/../..}" || exit 2

# The stand-in for `cargo search`, empty in every real run. Part B is the only
# half that needs a network, so it is the only half that needs a seam.
SEARCH="${PDFCER_TOOLKIT_SEARCH:-}"

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
    if [ -n "$SEARCH" ]; then
        line="$("$SEARCH" "$c" 2>/dev/null | head -1)"
    else
        line="$(cargo search "$c" --limit 1 2>/dev/null | head -1)"
    fi
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
    echo "  Parts A0 and A ran and passed. Part B — how far behind the published"
    echo "  release this shell is — did not run, so nothing here says the pin is"
    echo "  current. Exiting 2 rather than 0, because the runner classifies on"
    echo "  the exit code and the word SKIP above an exit 0 lands in the passed"
    echo "  column."
    exit 2
fi

echo "ui-toolkit: clean"
exit 0
