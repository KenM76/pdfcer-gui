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
#
#   0  pure — the manifest is clean AND at least one source file was scanned
#   1  a domain dependency was found
#   2  PRECONDITION ABSENT — the crate directory, its manifest, or any `.rs`
#      file under it is missing. NOT a pass: a clean manifest with no source
#      is check 1 passing and check 2 never running, and those two states must
#      not print the same line. `run-all.sh` prints skips in their own block
#      and exits 3.
#
# To falsify: add `pdfcer-core = { workspace = true }` to the shell's manifest,
# or a non-comment line naming `pdfcer_core` in any shell source file — both
# must exit 1. Point it at an empty directory and it must exit 2.

set -euo pipefail

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
