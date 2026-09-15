#!/usr/bin/env bash
#
# check-forwarded-features.sh — every capability the ENGINE has on by default is
# either forwarded into this build or refused here in writing.
#
# ═══════════════════════════════════════════════════════════════════════════
# THE PROPERTY ASSERTED
# ═══════════════════════════════════════════════════════════════════════════
#
# For every name in `pdfcer-core`'s `default = [ … ]` list, this crate declares
# a feature of the same name forwarding `pdfcer-core/<name>`, AND carries that
# name in its own `default` list — or names it in DELIBERATELY_NOT_FORWARDED
# below with a reason.
#
#   ENGINE  the engine's `crates/pdfcer-core/Cargo.toml`   `default = [ … ]`
#   OURS    `crates/pdfcer-gui/Cargo.toml`                 `<name> = [ … ]`
#
# Both halves are demanded because both can fail alone. Declaring
# `signing = ["pdfcer-core/signing"]` and leaving `signing` out of this crate's
# `default` is the same regression one level down: the feature exists, nothing
# turns it on, and an ordinary `cargo build` produces the lite build.
#
# ═══════════════════════════════════════════════════════════════════════════
# WHY A HUMAN CANNOT HOLD IT
# ═══════════════════════════════════════════════════════════════════════════
#
# `pdfcer-core` declares its strippable capabilities as Cargo features, all of
# them default on, and its manifest states the rule that binds every consumer:
# because Cargo unifies features across the whole graph, an intermediate crate
# must (a) take `pdfcer-core` with `default-features = false` and (b) re-export
# each capability it forwards.
#
# Clause (a) without clause (b) is the trap, and its shape is what makes it
# unholdable. It does not fail to compile. It does not fail a test. It does not
# warn. It REMOVES A CAPABILITY FROM THE BINARY, and the only way to notice is
# a dependency query nobody runs or a document that happens to need the missing
# thing. The absent half is invisible precisely because absence has no line of
# code to review: a diff shows what was added, never what should have been.
#
# Nor does a written warning help. A comment in a manifest protects the code
# above it and nothing written afterwards — a paragraph explaining this exact
# trap can sit forty lines above the feature block that falls into it, and the
# person adding the block is looking at the block. That is this project's most
# expensive recurring finding, and the remedy is always the same shape: replace
# the paragraph with a mechanism that reads BOTH sides and fails when they
# disagree.
#
# The list is read from the ENGINE rather than kept here for the same reason. A
# capability the engine adds tomorrow fails this gate the first time it runs,
# with nobody having remembered anything. A hard-coded list on this side would
# be one more place to forget.
#
# Only the DEFAULT list is demanded, not every feature the engine declares. A
# feature that is off by default is one the engine has decided is opt-in, and
# not forwarding it is a decision. A feature that is ON by default and missing
# here is a regression against the engine's own rule that a build which omits
# nothing behaves exactly as it did before the feature existed.
#
# ═══════════════════════════════════════════════════════════════════════════
# WHAT IT PROVABLY CANNOT SEE
# ═══════════════════════════════════════════════════════════════════════════
#
#   * Clause (a). Nothing here checks that this crate actually takes
#     `pdfcer-core` with `default-features = false`. The premise of the whole
#     gate is assumed, not measured.
#   * Whether the forwarded feature reaches the shipped binary. Cargo unifies
#     across the graph and a third crate in the middle can still strip it.
#     `cargo tree -i <backend-crate>` answers that; this cannot.
#   * The LOCKED engine revision. It reads the engine checkout's WORKING TREE
#     manifest, so a default feature added since the pin is demanded before it
#     can be forwarded, and one deleted since the pin stops being demanded
#     while the pinned build still carries it.
#   * A relocated or renamed engine checkout. The path is a literal here rather
#     than derived from `Cargo.lock` the way `tools/engine_path.py` derives it,
#     so an engine that has moved reads as an engine that is absent, and the
#     gate skips instead of failing.
#   * Anything a line-wise grep cannot reach. `default = [` split across lines,
#     a feature forwarded under a different name, a capability guarded by a
#     `cfg` rather than a feature.
#   * Whether a DELIBERATELY_NOT_FORWARDED reason is a good one. It checks only
#     that the name is followed by an em dash and something.
#
# ═══════════════════════════════════════════════════════════════════════════
# THE EXIT CONTRACT, AND HOW TO FALSIFY IT
# ═══════════════════════════════════════════════════════════════════════════
#
#   0  every engine default capability is forwarded and on by default here
#   1  one is not forwarded, or is forwarded and not on by default here, or the
#      engine manifest exists and no `default = [...]` line could be read out
#      of it — that last one is the gate failing to parse its own input, which
#      must be loud rather than green, and is deliberately not a skip
#   2  SKIPPED — an input is missing: no engine manifest on this machine, or no
#      `crates/pdfcer-gui/Cargo.toml`. NOT a pass. A gate that cannot read one
#      of its two sides has learned nothing, and `run-all.sh` prints skips in
#      their own block and exits 3. It does NOT fall back to a list kept here,
#      because a fallback list is the thing this gate exists to replace.
#
# To falsify: delete one forwarded feature line from
# `crates/pdfcer-gui/Cargo.toml` — it must name that capability and exit 1.
# Remove a name from this crate's own `default` list while leaving its feature
# declared, and it must report that separately. Point `ENGINE_MANIFEST` at a
# path that does not exist and it must print SKIPPED and exit 2.

set -u

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"

ENGINE_MANIFEST="D:/Dev/pdfcer/crates/pdfcer-core/Cargo.toml"
OUR_MANIFEST="$ROOT/crates/pdfcer-gui/Cargo.toml"

# ---------------------------------------------------------------------------
# DELIBERATELY NOT FORWARDED
#
# One entry per line: `<feature> — <reason>`. An entry must state WHY, in a
# sentence, and the reason must be about the CAPABILITY rather than about the
# schedule; "not needed yet" is not a reason, it is a work item, and a work item
# belongs in `ENGINE_BACKLOG.md` where something reads it.
#
# Empty today. Every default capability the engine has is forwarded.
# ---------------------------------------------------------------------------
DELIBERATELY_NOT_FORWARDED=""

if [ ! -f "$ENGINE_MANIFEST" ]; then
    echo "forwarded-features: SKIPPED — the engine manifest is not at $ENGINE_MANIFEST." >&2
    echo "  This gate compares the engine's own default feature list against ours." >&2
    echo "  With one side missing it can only guess, and a guess rendered as a" >&2
    echo "  green tick is what it exists to prevent. Exiting 2, not 0." >&2
    exit 2
fi
if [ ! -f "$OUR_MANIFEST" ]; then
    echo "forwarded-features: SKIPPED — $OUR_MANIFEST is missing." >&2
    exit 2
fi

# The engine's `default = [ ... ]`, as bare names.
#
# Anchored to the start of the line so a `default` mentioned inside a comment or
# inside another feature's list cannot be picked up. The engine's manifest
# carries long commentary about these features, and much of it uses the word.
engine_default="$(grep -m1 -E '^default[[:space:]]*=' "$ENGINE_MANIFEST" \
    | sed -E 's/^default[[:space:]]*=[[:space:]]*\[//; s/\].*$//' \
    | tr -d '" ' | tr ',' '\n' | grep -v '^$')"

if [ -z "$engine_default" ]; then
    echo "forwarded-features: FAIL — no \`default = [...]\` line was found in" >&2
    echo "  $ENGINE_MANIFEST." >&2
    echo >&2
    echo "  That is not 'the engine has no default features': it is this gate" >&2
    echo "  failing to read its own input, which must be loud rather than green." >&2
    exit 1
fi

our_default="$(grep -m1 -E '^default[[:space:]]*=' "$OUR_MANIFEST" \
    | sed -E 's/^default[[:space:]]*=[[:space:]]*\[//; s/\].*$//' \
    | tr -d '" ' | tr ',' '\n' | grep -v '^$')"

missing=""
not_default=""
exempted=""

for name in $engine_default; do
    if printf '%s\n' "$DELIBERATELY_NOT_FORWARDED" | grep -q "^$name —"; then
        exempted="${exempted}${name}
"
        continue
    fi
    # (b) a feature of the same name that forwards the engine's.
    if ! grep -qE "^$name[[:space:]]*=.*pdfcer-core/$name" "$OUR_MANIFEST"; then
        missing="${missing}${name}
"
        continue
    fi
    # …and it is on by default here too.
    if ! printf '%s\n' "$our_default" | grep -qx "$name"; then
        not_default="${not_default}${name}
"
    fi
done

status=0

if [ -n "$missing" ]; then
    status=1
    echo "forwarded-features: FAIL — $(printf '%s' "$missing" | grep -c '^') engine capability(ies) are ON BY DEFAULT and are not forwarded:"
    printf '%s' "$missing" | sed 's/^/    /'
    cat <<'EOF'

`pdfcer-core` takes each of these as a Cargo feature, has it ON by default, and
this crate takes `pdfcer-core` with `default-features = false` — so a name that
is not re-declared here is STRIPPED FROM THE BINARY. It does not fail to
compile. It does not fail a test. The capability is simply gone.

Add to `crates/pdfcer-gui/Cargo.toml`:

    <name> = ["pdfcer-core/<name>"]

and put `<name>` in this crate's own `default` list — or, if not forwarding it
is a decision, add it to DELIBERATELY_NOT_FORWARDED in this script WITH THE
REASON. "Not needed yet" is not a reason; that is a work item and belongs in
ENGINE_BACKLOG.md where something reads it.

This has happened twice: JPEG 2000 in 2026-08, and the entire digital-signing
subsystem in 2026-09 — the second three days after a comment warning about the
first was written into the very manifest that repeated it.
EOF
fi

if [ -n "$not_default" ]; then
    status=1
    echo "forwarded-features: FAIL — $(printf '%s' "$not_default" | grep -c '^') capability(ies) are forwarded but NOT ON BY DEFAULT here:"
    printf '%s' "$not_default" | sed 's/^/    /'
    cat <<'EOF'

The feature exists and nothing turns it on, so an ordinary `cargo build`
produces the lite build. That is the same regression one level down, and rule 1
of the engine's own strippable-capability convention forbids it: "a build that
omits nothing must behave exactly as it did before the feature existed."
EOF
fi

if [ "$status" -eq 0 ]; then
    echo "forwarded-features: clean — $(printf '%s\n' "$engine_default" | grep -c '^') engine default capability(ies), all forwarded and on by default here:"
    printf '%s\n' "$engine_default" | sed 's/^/    /'
    if [ -n "$exempted" ]; then
        echo "  deliberately not forwarded:"
        printf '%s' "$exempted" | sed 's/^/    /'
    fi
fi

exit "$status"
