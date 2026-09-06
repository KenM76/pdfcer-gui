#!/usr/bin/env bash
#
# check-forwarded-features.sh — every capability the ENGINE has on by default is
# either forwarded into this build or refused here in writing.
#
# ═══════════════════════════════════════════════════════════════════════════
# ★★★ WHY THIS GATE EXISTS: THE SAME MISTAKE, TWICE, THREE DAYS APART
# ═══════════════════════════════════════════════════════════════════════════
#
# `pdfcer-core` declares STRIPPABLE CAPABILITIES as Cargo features, all of them
# **default on**, and its own manifest states the rule that binds every consumer:
#
#   > Cargo unifies features across the whole graph, so every intermediate crate
#   > must (a) take `pdfcer-core` with `default-features = false` and (b)
#   > re-export each capability it forwards.
#
# Clause (a) without clause (b) does not fail to compile. It does not fail a
# test. It does not warn. It **removes a capability from the binary**, and the
# only way to notice is a dependency query or a document that needs it.
#
# It has now happened twice:
#
#   1. **JPEG 2000, 2026-08.** The feature block was missing entirely,
#      `pdfcer-core` was taken with `default-features = false`, and the GUI
#      silently lost JPX decoding. `cargo tree -p pdfcer-gui -i hayro-jpeg2000`
#      came back EMPTY. The fix added a `[features]` block AND a long comment
#      warning the next reader, ending: *"forgetting to forward does not fail
#      to compile."*
#
#   2. **SIGNING, 2026-09-05.** The engine shipped `pdfcer_core::sign` — 101
#      public items, PKCS#12 import, CAdES `SignedData`, `EditSession::sign` —
#      written in answer to *this shell's own* request of 2026-09-03, *"a
#      document cannot be signed."* Its `signing` feature is default on. This
#      manifest forwarded `jpx` and `ocrs` and not it, so the whole subsystem
#      was absent from the binary for three days. Nothing failed. The comment
#      from incident 1 was forty lines above the block that repeated it.
#
# ⇒ **A WARNING DOES NOT PROTECT A CODE PATH WRITTEN AFTER IT.** That sentence
# is this project's most expensive recurring finding — the rotation-button gate
# had its rule in its own module header sixty lines above the code that broke
# it — and the remedy is always the same shape: replace the paragraph with a
# mechanism that reads BOTH sides and fails when they disagree.
#
# ═══════════════════════════════════════════════════════════════════════════
# WHAT IT READS
# ═══════════════════════════════════════════════════════════════════════════
#
#   ENGINE  D:/Dev/pdfcer/crates/pdfcer-core/Cargo.toml   `default = [ … ]`
#   OURS    crates/pdfcer-gui/Cargo.toml                  `<name> = [ … ]` lines
#
# For each name in the engine's default list, this build must either
#
#   * declare a feature of the same name that forwards `pdfcer-core/<name>`, or
#   * name it in DELIBERATELY_NOT_FORWARDED below, with the reason.
#
# ★ It reads the ENGINE'S OWN LIST rather than a list kept here, which is the
# whole point: a capability the engine adds tomorrow fails this gate the first
# time it is run, without anybody having remembered to add it. A hard-coded list
# on this side would be a fourth place to forget.
#
# ★★ It checks the DEFAULT list specifically, not every feature the engine
# declares. A feature that is off by default is a capability the engine has
# decided is opt-in, and not forwarding one is a decision rather than an
# omission. A feature that is ON by default and missing here is a REGRESSION —
# rule 1 of the engine's own convention: *"a build that omits nothing must
# behave exactly as it did before the feature existed."*
#
# ═══════════════════════════════════════════════════════════════════════════
# WHY IT ALSO CHECKS THAT `default` HERE LISTS THEM
# ═══════════════════════════════════════════════════════════════════════════
#
# Declaring `signing = ["pdfcer-core/signing"]` and leaving it out of this
# crate's own `default` is the same regression one level down: the feature
# exists, nothing turns it on, and an ordinary `cargo build` produces the lite
# build. Both halves are checked because both have to be right.
#
# ═══════════════════════════════════════════════════════════════════════════
# WHEN THE ENGINE IS NOT ON THIS MACHINE
# ═══════════════════════════════════════════════════════════════════════════
#
# Exit 2 — SKIPPED — and say so. A gate that cannot read one of its two inputs
# has learned nothing, and `run-all.sh` counts a skip separately from a pass for
# exactly that reason. It does NOT fall back to a list kept here, because a
# fallback list is the thing this gate exists to replace.

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
# ★ Anchored to the start of the line so a `default` mentioned inside a comment
# or inside another feature's list cannot be picked up. The engine's manifest
# carries several hundred lines of commentary about these features and half of
# them contain the word.
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
