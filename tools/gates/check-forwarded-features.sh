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
#   * Anything a line-wise grep cannot reach: a feature forwarded under a
#     different name, a capability guarded by a `cfg` rather than a feature. A
#     `default = [` that opens on one line and closes on a later one is the
#     exception — it is the only one of these that would read HALF a list and
#     report clean, so it is detected and failed rather than tolerated.
#   * Whether a DELIBERATELY_NOT_FORWARDED reason is a good one. It checks only
#     that the name is followed by an em dash and something.
#
# ═══════════════════════════════════════════════════════════════════════════
# THE EXIT CONTRACT, AND HOW TO FALSIFY IT
# ═══════════════════════════════════════════════════════════════════════════
#
#   0  every engine default capability is forwarded and on by default here
#   1  one is not forwarded; or is forwarded and not on by default here; or is
#      refused with no reason; or is refused by an entry that refuses nothing
#      because the engine no longer defaults that name; or the engine manifest
#      exists and its `default = [...]` could not be read whole — no such line,
#      or one that opens its list and does not close it. Those last two are the
#      gate failing to read its own input, which must be loud rather than
#      green, and are deliberately not skips: a half-read list rendered as a
#      green tick is indistinguishable from a list with nothing wrong in it
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

# Both sides are overridable so `--self-test` can point the gate at synthetic
# manifests and exercise every branch without a pdfcer checkout, a network, or
# any dependence on what this repository's own manifest happens to say today.
ENGINE_MANIFEST="${FORWARDED_ENGINE_MANIFEST:-D:/Dev/pdfcer/crates/pdfcer-core/Cargo.toml}"
OUR_MANIFEST="${FORWARDED_OUR_MANIFEST:-$ROOT/crates/pdfcer-gui/Cargo.toml}"

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

# The self-test's third seam. `-` rather than `:-`, so an override that is
# deliberately empty stays empty instead of falling back to the list above and
# quietly testing something other than what the arm asked for.
DELIBERATELY_NOT_FORWARDED="${FORWARDED_EXEMPTIONS-$DELIBERATELY_NOT_FORWARDED}"

# ═══════════════════════════════════════════════════════════════════════════
# SELF-TEST
#
# Every arm runs this script against a synthetic engine manifest and a
# synthetic manifest of our own, so it needs no pdfcer checkout and reads
# nothing from this repository. That matters twice over: the arms keep working
# on a machine where the engine is not cloned — exactly the machine where the
# real gate SKIPS and its correctness is least visible — and the arms do not
# change meaning the next time somebody edits our own feature list.
#
# Two arms demand a GREEN and they are the ones a hand test does not produce:
# an engine feature that is declared and NOT defaulted must not be demanded
# (this gate takes off-by-default as a decision the engine has already made),
# and a `default` inside a comment must not be picked up (the engine's manifest
# carries paragraphs about these features and uses the word throughout).
# ═══════════════════════════════════════════════════════════════════════════
if [ "${1:-}" = "--self-test" ]; then
    SELF="$HERE/$(basename "${BASH_SOURCE[0]}")"
    TD="$(mktemp -d)"
    trap 'rm -rf "$TD"' EXIT
    st=0
    arms=0

    # A manifest is a `[features]` table and the lines given, verbatim.
    mkman() {
        local p="$1"; shift
        {
            echo '[package]'
            echo 'name = "synthetic"'
            echo
            echo '[features]'
            printf '%s\n' "$@"
        } > "$p"
    }

    arm() {
        local label="$1" want="$2" e="$3" o="$4" ex="$5"
        local must="${6:-}" mustnot="${7:-}"
        arms=$((arms + 1))
        local out got
        out="$(FORWARDED_ENGINE_MANIFEST="$e" FORWARDED_OUR_MANIFEST="$o" \
               FORWARDED_EXEMPTIONS="$ex" bash "$SELF" 2>&1)"
        got=$?
        if [ "$got" -ne "$want" ]; then
            echo "  FAIL [$label] expected rc=$want, got rc=$got"
            printf '%s\n' "$out" | sed 's/^/        /'
            st=1
            return
        fi
        if [ -n "$must" ] && ! printf '%s\n' "$out" | grep -qE "$must"; then
            echo "  FAIL [$label] rc=$want as expected, but the message never said /$must/"
            printf '%s\n' "$out" | sed 's/^/        /'
            st=1
            return
        fi
        if [ -n "$mustnot" ] && printf '%s\n' "$out" | grep -qE "$mustnot"; then
            echo "  FAIL [$label] rc=$want as expected, but the message said /$mustnot/, which is the wrong diagnosis"
            printf '%s\n' "$out" | sed 's/^/        /'
            st=1
        fi
    }

    # Engine sides.
    mkman "$TD/e-two.toml"     'default = ["jpx", "ocrs"]' 'jpx = []' 'ocrs = []'
    mkman "$TD/e-decoy.toml"   '# default = ["bogus"]' 'default = ["jpx"]' 'jpx = []'
    mkman "$TD/e-optin.toml"   'default = ["jpx"]' 'jpx = []' 'ocrs = []'
    mkman "$TD/e-open.toml"    'default = ["jpx",' '    "ocrs"]' 'jpx = []'
    mkman "$TD/e-nodefault.toml" 'jpx = []' 'ocrs = []'

    # Our side.
    mkman "$TD/o-two.toml"     'default = ["jpx", "ocrs"]' \
                               'jpx = ["pdfcer-core/jpx"]' 'ocrs = ["pdfcer-core/ocrs"]'
    mkman "$TD/o-one.toml"     'default = ["jpx"]' 'jpx = ["pdfcer-core/jpx"]'
    mkman "$TD/o-notdef.toml"  'default = ["jpx"]' \
                               'jpx = ["pdfcer-core/jpx"]' 'ocrs = ["pdfcer-core/ocrs"]'
    mkman "$TD/o-wrongcrate.toml" 'default = ["jpx", "ocrs"]' \
                               'jpx = ["pdfcer-core/jpx"]' 'ocrs = ["pdfcer-render/ocrs"]'

    # ── the agreement control ──────────────────────────────────────────────
    arm "both sides agree" 0 "$TD/e-two.toml" "$TD/o-two.toml" "" 'clean'

    # ── the regression the gate exists for ─────────────────────────────────
    arm "a default capability not forwarded at all" 1 \
        "$TD/e-two.toml" "$TD/o-one.toml" "" 'are not forwarded'
    arm "forwarded but left out of our own default list" 1 \
        "$TD/e-two.toml" "$TD/o-notdef.toml" "" 'NOT ON BY DEFAULT' 'are not forwarded'
    arm "forwarded from some other crate is not forwarded" 1 \
        "$TD/e-two.toml" "$TD/o-wrongcrate.toml" "" 'are not forwarded'

    # ── greens a hand test does not produce ────────────────────────────────
    arm "an engine feature that is not on by default is not demanded" 0 \
        "$TD/e-optin.toml" "$TD/o-one.toml" "" 'clean'
    arm "a default inside a comment is not the default list" 0 \
        "$TD/e-decoy.toml" "$TD/o-one.toml" "" 'clean'

    # ── the refusal list ───────────────────────────────────────────────────
    arm "a refusal with a reason discharges the row" 0 \
        "$TD/e-two.toml" "$TD/o-one.toml" \
        'ocrs — the OCR backend is chosen at runtime here, so linking it twice is waste' \
        'deliberately not forwarded'
    arm "a refusal with no reason is not a discharge" 1 \
        "$TD/e-two.toml" "$TD/o-one.toml" 'ocrs' \
        'with no reason' 'are not forwarded'
    arm "an em dash with nothing after it is not a reason" 1 \
        "$TD/e-two.toml" "$TD/o-one.toml" 'ocrs — ' \
        'with no reason' 'are not forwarded'
    arm "a refusal that refuses nothing" 1 \
        "$TD/e-two.toml" "$TD/o-two.toml" \
        'signing — the engine stopped defaulting this one and nobody came back' \
        'refuse nothing'

    # ── failing to read its own input is loud, not green and not a skip ────
    arm "a default list that opens and does not close" 1 \
        "$TD/e-open.toml" "$TD/o-two.toml" "" 'does not close on it' 'all forwarded and on by default'
    arm "an engine manifest with no default list" 1 \
        "$TD/e-nodefault.toml" "$TD/o-two.toml" "" 'no .default = ' 'all forwarded and on by default'

    # ── a missing side is a skip, and a skip is not a pass ─────────────────
    arm "no engine manifest on this machine" 2 \
        "$TD/nosuch.toml" "$TD/o-two.toml" "" 'SKIPPED'
    arm "no manifest of our own" 2 \
        "$TD/e-two.toml" "$TD/nosuch.toml" "" 'SKIPPED'

    if [ "$st" -eq 0 ]; then
        echo "check-forwarded-features --self-test: PASS — $arms arms."
        echo "  A capability that is on by default in the engine and absent here is named;"
        echo "  one that is forwarded and not defaulted is reported separately; a refusal"
        echo "  without a reason is not a discharge; and a list this gate can only read"
        echo "  half of fails rather than reporting clean."
    else
        echo "check-forwarded-features --self-test: FAIL"
    fi
    exit "$st"
fi

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
engine_default_line="$(grep -m1 -E '^default[[:space:]]*=' "$ENGINE_MANIFEST")"

# A `default` list that opens on this line and closes on a later one is the one
# shape a line-wise read gets WRONG QUIETLY rather than not at all. The names
# before the newline parse, the names after it are never demanded, and the gate
# prints `clean` over a list it read half of. An empty list is already loud
# below; this half-read is not, so it is made loud here.
if printf '%s' "$engine_default_line" | grep -q '\[' \
   && ! printf '%s' "$engine_default_line" | grep -q '\]'; then
    echo "forwarded-features: FAIL — the \`default = [\` in $ENGINE_MANIFEST opens" >&2
    echo "  on its line and does not close on it." >&2
    echo >&2
    echo "  This gate reads that list one line at a time, so it would demand the" >&2
    echo "  names before the newline and silently ignore every name after it —" >&2
    echo "  reporting clean over a list it read half of. Either put the list on" >&2
    echo "  one line, or teach this parse to span lines. Not a skip: a half-read" >&2
    echo "  input rendered as a green tick is what this gate exists to prevent." >&2
    exit 1
fi

engine_default="$(printf '%s' "$engine_default_line" \
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
unreasoned=""

# Every name the refusal list MENTIONS, in whatever shape the entry is in.
#
# Read separately from the reason because a name mentioned without one must not
# fall through to "not forwarded": that message tells the reader to add a list
# entry that is already sitting there, and a gate whose remedy is already in
# place is a gate the reader concludes is broken. The two states want opposite
# actions — write the reason, or write the forwarding line — so they are
# separate outcomes with separate sentences.
exempt_names="$(printf '%s\n' "$DELIBERATELY_NOT_FORWARDED" \
    | sed -E 's/^[[:space:]]+//' | grep -v '^$' \
    | sed -E 's/^([^[:space:]]+).*$/\1/' | sort -u)"

# A reason is an em dash followed by something that is not more whitespace.
# "Checks only that there IS one", per the header — whether it is a good reason
# is a review question and this cannot be the thing that answers it.
exempt_reason_for() {
    printf '%s\n' "$DELIBERATELY_NOT_FORWARDED" \
        | sed -E 's/^[[:space:]]+//' \
        | grep -E "^$1[[:space:]]*—[[:space:]]*[^[:space:]]" \
        | head -1
}

for name in $engine_default; do
    if printf '%s\n' "$exempt_names" | grep -qx "$name"; then
        if [ -z "$(exempt_reason_for "$name")" ]; then
            unreasoned="${unreasoned}${name}
"
        else
            exempted="${exempted}${name}
"
        fi
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

# A refusal naming a capability the engine does not have on by default refuses
# nothing. It is read as a live decision by everybody who finds it, and it is
# the shape a stale list takes: the engine stops defaulting a feature, or
# renames it, and the paragraph explaining why we decline it stays behind
# forever with nothing to decline.
stale=""
for name in $exempt_names; do
    printf '%s\n' "$engine_default" | grep -qx "$name" || stale="${stale}${name}
"
done

status=0

if [ -n "$unreasoned" ]; then
    status=1
    echo "forwarded-features: FAIL — $(printf '%s' "$unreasoned" | grep -c '^') capability(ies) are named in DELIBERATELY_NOT_FORWARDED with no reason:"
    printf '%s' "$unreasoned" | sed 's/^/    /'
    cat <<'EOF'

The format is `<feature> — <reason>`, with an em dash and a sentence. An entry
that names a capability and stops is a refusal nobody can review: the next
reader cannot tell a decision from an oversight, and the only remedy left is to
find whoever wrote it.

The reason must be about the CAPABILITY, not the schedule. "Not needed yet" is
a work item, and a work item belongs in ENGINE_BACKLOG.md where something reads
it.
EOF
fi

if [ -n "$stale" ]; then
    status=1
    echo "forwarded-features: FAIL — $(printf '%s' "$stale" | grep -c '^') DELIBERATELY_NOT_FORWARDED entry(ies) refuse nothing:"
    printf '%s' "$stale" | sed 's/^/    /'
    cat <<'EOF'

Each names a capability that is not in the engine's `default` list — because it
was renamed, was made opt-in, or was removed. The entry still reads as a live
decision to anybody who finds it, and the reason attached to it is now about a
capability that is not there.

Delete the entry. If the capability came back under a new name, that new name
is a fresh decision and wants its own entry with its own reason.
EOF
fi

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
