#!/usr/bin/env bash
# check-shell-purity.sh — `crates/egui-shell/` must not know what a PDF is.
#
# ===========================================================================
# THE PROPERTY ASSERTED
# ===========================================================================
#
# The `egui-shell` crate carries no dependency edge to the application's
# domain crates: none in its manifest, none in its source, not through a
# re-export and not through a dev-dependency.
#
# `egui-shell` is a REUSABLE application shell — ribbon, dock, modes, layout
# persistence, theme, command registry — to be extracted to its own MIT
# repository at or before fold-in. The workspace root says it "knows nothing
# about PDF and must never learn", and SHELL_FRAMEWORK.md's whole design rests
# on that.
#
# ===========================================================================
# WHY A HUMAN CANNOT HOLD IT
# ===========================================================================
#
# The failure this catches is not a crash. It is one `use
# pdfcer_core::PageSize` in a layout helper, added because the type was there
# and it was convenient. That single line compiles, is formatted, satisfies
# clippy and passes every test — there is no moment at which anything visibly
# goes wrong, so there is nothing for a reviewer to notice. Its costs all
# arrive later and elsewhere:
#
#   * the shell becomes un-extractable — the standalone repository will not
#     compile — and nobody finds out until extraction day, by which time there
#     are forty such lines and the extraction is cancelled;
#   * the dependency the architecture rests on is inverted. The shell is meant
#     to be the stable substrate the application plugs into, and a shell that
#     depends on the application cannot be that.
#
# A reusable component stays reusable only while something mechanically
# refuses the convenient shortcut. Enough "just this one import" and the shell
# is not a shell, it is the application's other half.
#
# ===========================================================================
# WHAT IS CHECKED
# ===========================================================================
#
# 1. `crates/egui-shell/Cargo.toml` names no `pdfcer-*` dependency. Caught at
#    the manifest, which is where the coupling is cheapest to see and where a
#    reviewer looks first. The dependency KEY is what is matched, so the
#    `{ path = "..." }` and `{ workspace = true }` spellings are covered
#    alike, as is a `[dependencies.pdfcer-...]` table header.
#
# 2. No `.rs` file under the crate mentions `pdfcer_core`, `pdfcer_render` or
#    `pdfcer_print`. The underscore spelling is the one that appears in Rust
#    source; it catches a `use`, a fully-qualified path and a `#[cfg]`-gated
#    import alike. This is the backstop for a type that arrives through a
#    re-export or a dev-dependency while the manifest stays clean.
#
# Comment lines are exempt from check 2. This file's own architecture notes
# name the forbidden crates, and so will the shell's — a rule you cannot
# describe in a doc comment is a rule that will not be described at all.
#
# ===========================================================================
# WHAT IT PROVABLY CANNOT SEE
# ===========================================================================
#
#   * VOCABULARY, on purpose. The word "pdf" in prose, an icon named
#     `pdf.svg`, a doc comment saying "the pdfcer application supplies this" —
#     all pass. Purity is about the DEPENDENCY EDGE. A gate that fired on the
#     word would be switched off within a week, and a gate that has been
#     switched off enforces nothing.
#   * A domain crate reached under another name: a rename, a `package =`
#     alias in the manifest, or a fourth engine crate. Check 1 catches the
#     manifest half by prefix; check 2's three crate names are spelled out and
#     do not grow by themselves.
#   * Conceptual coupling with no import. A shell API shaped around exactly
#     one application's needs is impure in every sense but this one.
#   * Anything under `target/`, which is build output rather than authored
#     code.
#
# ===========================================================================
# USAGE, THE EXIT CONTRACT, AND HOW TO FALSIFY IT
# ===========================================================================
#   tools/gates/check-shell-purity.sh [SHELL_CRATE_DIR]
#   tools/gates/check-shell-purity.sh --self-test
#
#   0  pure — the manifest is clean AND at least one source file was scanned
#   1  a domain dependency was found
#   2  PRECONDITION ABSENT — the crate directory, its manifest, or any `.rs`
#      file under it is missing. NOT a pass: a clean manifest with no source
#      is check 1 passing and check 2 never running, and those two states must
#      not print the same line. `run-all.sh` prints skips in their own block
#      and exits 3.
#
# `--self-test` falsifies the gate against synthetic crates rather than against
# prose: it plants each violation and asserts the gate catches it, plants the
# comment exemption and asserts it does NOT, and asserts both absent-precondition
# states exit 2 rather than 0. It exits 1 if any arm disagrees.

set -euo pipefail

# ===========================================================================
# --self-test
# ===========================================================================
#
# This gate already takes the crate directory as its argument, so falsifying it
# needs no scratch copy of the repository and no edit to a tracked file: the
# self-test builds synthetic crates in a temp directory and points the gate at
# each in turn.
#
# ★ SEVEN ARMS, because this gate has seven outcomes and a plant fires exactly
# ONE of them. A single red run proves the gate can fail; it proves nothing
# about the six arms it did not exercise, and each of those prints a different
# sentence and a different exit code. The arms that are never written by hand
# are the two that exit 2 and the one that must exit 0 — an exemption nobody
# has measured is an opinion, and a precondition-absent state nobody has
# measured is indistinguishable from a pass at the point it matters.
# ===========================================================================
if [ "${1:-}" = "--self-test" ]; then
    SELF="$(cd "$(dirname "$0")" && pwd)/$(basename "$0")"
    REPO="$(cd "$(dirname "$SELF")/../.." && pwd)"
    TD="$(mktemp -d)"
    trap 'rm -rf "$TD"' EXIT
    fails=0

    CLEAN_MANIFEST='[package]
name = "egui-shell"
version = "0.1.0"
edition = "2021"

[dependencies]
egui = "0.28"'

    arm() {   # arm <label> <expected-rc> <crate-dir>
        local got=0
        bash "$SELF" "$3" >/dev/null 2>&1 || got=$?
        if [ "$got" -eq "$2" ]; then
            printf '  ok    %-26s rc=%d\n' "$1" "$got"
        else
            printf '  FAIL  %-26s rc=%d, expected %d\n' "$1" "$got" "$2"
            fails=$((fails + 1))
        fi
    }

    # --- arm 1: a dependency key in the manifest -----------------------------
    mkdir -p "$TD/manifest-key/src"
    printf '%s\npdfcer-core = { workspace = true }\n' "$CLEAN_MANIFEST" \
        >"$TD/manifest-key/Cargo.toml"
    printf 'pub fn shell() {}\n' >"$TD/manifest-key/src/lib.rs"
    arm "manifest dependency key" 1 "$TD/manifest-key"

    # --- arm 2: the [dependencies.pdfcer-*] table-header spelling ------------
    # A separate arm because check 1 matches on two different patterns, and a
    # plant of the first says nothing about the second.
    mkdir -p "$TD/manifest-table/src"
    printf '%s\n\n[dependencies.pdfcer-render]\npath = "../pdfcer-render"\n' \
        "$CLEAN_MANIFEST" >"$TD/manifest-table/Cargo.toml"
    printf 'pub fn shell() {}\n' >"$TD/manifest-table/src/lib.rs"
    arm "manifest table header" 1 "$TD/manifest-table"

    # --- arm 3: an import in source, manifest clean --------------------------
    # The backstop arm: this is the shape a re-export or a dev-dependency takes,
    # and the manifest stays clean throughout.
    mkdir -p "$TD/source-import/src"
    printf '%s\n' "$CLEAN_MANIFEST" >"$TD/source-import/Cargo.toml"
    printf 'use pdfcer_core::PageSize;\npub fn shell(_: PageSize) {}\n' \
        >"$TD/source-import/src/lib.rs"
    arm "source import" 1 "$TD/source-import"

    # --- arm 4: THE EXEMPTION, asserted rather than assumed ------------------
    # The header promises that a doc comment may name the forbidden crates,
    # because a rule you cannot describe is a rule that will not be described.
    # That promise is a claim about behaviour, so it is measured: this crate
    # names all three domain crates in comments only and must pass.
    mkdir -p "$TD/comment-only/src"
    printf '%s\n' "$CLEAN_MANIFEST" >"$TD/comment-only/Cargo.toml"
    printf '%s\n' \
        '//! The shell never depends on pdfcer_core, pdfcer_render or pdfcer_print.' \
        '/* pdfcer_core is the application half; this crate is the substrate. */' \
        ' * pdfcer_print lives on the other side of the seam.' \
        'pub fn shell() {}' >"$TD/comment-only/src/lib.rs"
    arm "comments name it: passes" 0 "$TD/comment-only"

    # --- arm 5: clean manifest, no source at all -----------------------------
    # Check 1 passes and check 2 never runs. The whole argument of this gate's
    # closing block is that those two states must not print the same line.
    mkdir -p "$TD/no-source"
    printf '%s\n' "$CLEAN_MANIFEST" >"$TD/no-source/Cargo.toml"
    arm "no .rs: SKIPPED not pass" 2 "$TD/no-source"

    # --- arm 6: the directory does not exist ---------------------------------
    arm "no crate dir" 2 "$TD/absent"

    # --- arm 7: the real crate, unplanted ------------------------------------
    # The control. Without it a self-test that had somehow been wired to fail
    # on everything would read as six passes.
    arm "real egui-shell passes" 0 "$REPO/crates/egui-shell"

    if [ "$fails" -ne 0 ]; then
        echo "shell-purity --self-test: FAIL — $fails arm(s) disagreed."
        echo "  The gate does not behave as its header claims. Fix the gate, not"
        echo "  the self-test: a check that cannot fail is not evidence, and one"
        echo "  that fails on the exemption will be switched off within a week."
        exit 1
    fi
    echo "shell-purity --self-test: ok — 7 arm(s): 3 planted violations caught,"
    echo "              the comment exemption honoured, both absent-precondition"
    echo "              states exited 2, and the real crate passed unplanted"
    exit 0
fi

SHELL_DIR="${1:-crates/egui-shell}"
MANIFEST="$SHELL_DIR/Cargo.toml"

if [ ! -d "$SHELL_DIR" ]; then
    echo "shell-purity: SKIPPED — no $SHELL_DIR" >&2
    echo "  Exiting 2, not 0: a crate that does not exist has not been checked." >&2
    exit 2
fi

rc=0

# ---------------------------------------------------------------------------
# CHECK 1 — the manifest names no pdfcer-* dependency.
#
# Scanned line-wise rather than by parsing TOML, because the gate must run with
# nothing but bash and awk. That means it looks at dependency KEYS: a line
# whose first token is `pdfcer-<something>` followed by `=`, anywhere in the
# file. `[dependencies.pdfcer-core]` table headers are matched too.
# ---------------------------------------------------------------------------
if [ ! -f "$MANIFEST" ]; then
    echo "shell-purity: SKIPPED — no $MANIFEST" >&2
    echo "  The directory exists but has no manifest; the crate is mid-write." >&2
    exit 2
fi

manifest_hits=$(awk '
    {
        line = $0
        sub(/#.*/, "", line)                       # strip TOML comments
        if (line ~ /^[[:space:]]*pdfcer-[A-Za-z0-9_-]+[[:space:]]*=/) {
            printf "%s:%d:%s\n", FILENAME, FNR, $0
        }
        if (line ~ /^[[:space:]]*\[[^]]*dependencies[^]]*\.pdfcer-/) {
            printf "%s:%d:%s\n", FILENAME, FNR, $0
        }
    }
' "$MANIFEST")

if [ -n "$manifest_hits" ]; then
    echo "shell-purity: FAIL — $MANIFEST declares a domain dependency:"
    printf '%s\n' "$manifest_hits" | sed 's/^/  /'
    rc=1
fi

# ---------------------------------------------------------------------------
# CHECK 2 — no source file names a domain crate.
#
# `-not -path '*/target/*'` because a build artefact is not authored code.
# Comment lines are skipped: the shell's own documentation has to be able to
# say which crates it must never depend on.
# ---------------------------------------------------------------------------
scanned=0
src_hits=""
while IFS= read -r -d '' f; do
    scanned=$((scanned + 1))
    h=$(awk '
        $0 ~ /^[[:space:]]*(\/\/|\/\*|\*)/ { next }     # comments are prose
        $0 ~ /pdfcer_(core|render|print)/ { printf "%s:%d:%s\n", FILENAME, FNR, $0 }
    ' "$f")
    [ -n "$h" ] && src_hits="${src_hits}${h}
"
done < <(find "$SHELL_DIR" -type f -name '*.rs' -not -path '*/target/*' -print0 | sort -z)

src_hits=$(printf '%s' "$src_hits" | sed '/^$/d')
if [ -n "$src_hits" ]; then
    echo "shell-purity: FAIL — shell source references a domain crate:"
    printf '%s\n' "$src_hits" | sed 's/^/  /'
    rc=1
fi

if [ "$rc" -ne 0 ]; then
    cat <<'EOF'

egui-shell is a REUSABLE shell and is extracted to its own repository at or
before fold-in. It must not depend on pdfcer-core, pdfcer-render, pdfcer-print
or pdfcer-gui — not by manifest, not by import, not through a re-export.

If the shell needs something the application knows, INVERT IT: the shell
declares a trait or a manifest type, and pdfcer-gui supplies the value. That is
what the shell manifest, the panel-body callbacks and the command registry are
for. The correct fix is never an import; it is a seam.
EOF
    exit 1
fi

# "The manifest is clean and there is no source" is NOT a pass. It is check 1
# passing and check 2 never running, and those two states must not print the
# same line — that conflation is the whole subject of check-ui-strings.sh's
# header and of PROJECT_PLAN.md §4.1.
if [ "$scanned" -eq 0 ]; then
    echo "shell-purity: SKIPPED — $MANIFEST is clean, but $SHELL_DIR contains no" >&2
    echo "  .rs files, so the source check did not run. Exiting 2, not 0." >&2
    exit 2
fi

echo "shell-purity: clean — $MANIFEST declares no pdfcer-* dependency,"
echo "              and $scanned .rs file(s) under $SHELL_DIR name no domain crate"
exit 0
