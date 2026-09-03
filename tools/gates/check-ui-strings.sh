#!/usr/bin/env bash
# check-ui-strings.sh — enforce rule R1:
# every OPERATOR-VISIBLE string in pdfcer-gui lives in the ui_text catalog.
#
# This is a PORT of D:\Dev\pdfcer\tools\check-ui-strings.sh with one bug fixed
# and one exclusion generalised. The original's reasoning is excellent and is
# preserved below almost verbatim, because the reasoning is the valuable part;
# what changed is marked "PORT CHANGE".
#
# ===========================================================================
# PORT CHANGE 1 — THE GATE USED TO FAIL OPEN ON A MODULE TREE
# ===========================================================================
#
# The original scanned with a flat, non-recursive glob:
#
#     for file in "$SRC_DIR"/*.rs; do            # pdfcer check-ui-strings.sh:76
#
# `src/*.rs` does not match `src/app/state.rs`. The moment the crate grows its
# first subdirectory the gate stops seeing almost the whole crate — AND REPORTS
# SUCCESS, because "found no violations" and "looked at almost nothing" produce
# byte-identical output. It would print `ui-strings: clean` having read three
# files out of forty.
#
# This project's crate is a module TREE from day one (PROJECT_PLAN.md §3), so
# the original gate would have been switched off before it ever ran once. The
# fix is `find`, and it is verified by the self-test below rather than asserted
# here: the dirty fixture's only violation is deliberately planted TWO
# directories down, where a flat glob cannot reach it.
#
# ===========================================================================
# PORT CHANGE 2 — THE CATALOG IS A DIRECTORY, NOT A FILE
# ===========================================================================
#
# The original excluded exactly one filename, `ui_text.rs`. pdfcer's catalog is
# already large enough that PROJECT_PLAN.md §9 Q4 contemplates splitting it,
# and this crate starts split: `crates/pdfcer-gui/src/text/` is a directory
# whose `mod.rs` says so in its first line, with one module per surface
# (ribbon, panels, dialogs, tools) to come.
#
# So the exclusion is a LIST, held in `CATALOG_RELPATHS` below, and it is
# ANCHORED AT THE SCAN ROOT. Anchoring is the part that matters. An unanchored
# `*/text/*` would also excuse `src/tools/text.rs` — the text TOOL, which is
# ordinary code full of operator-visible labels and exactly the kind of file
# this gate exists to police. `text/` is the catalog only when it sits directly
# under the crate's `src/`; anywhere else it is just a module called text.
#
# Both the old `ui_text` spelling and the new `text` spelling are honoured, so
# a rename does not silently switch the exclusion off, and neither does the
# eventual split into `text/ribbon.rs`, `text/panels.rs` and so on.
#
# ===========================================================================
# PORT CHANGE 3 — THE GATE PROVES IT CAN FAIL
# ===========================================================================
#
# `--self-test` runs the scanner against two fixtures and asserts BOTH
# directions: clean must pass, dirty must fail. The original file records, in
# its own words, the day a planted violation failed to fire and "for a moment
# it looked as though the fix had produced a gate that could only pass". That
# lesson is now mechanical instead of remembered. A gate that has never been
# observed to fail is not evidence of anything.
#
# ===========================================================================
# WHAT IT SKIPS, AND WHY EACH EXCLUSION IS PRINCIPLED  (from the original)
# ===========================================================================
# 1. The catalog itself — that IS the catalog.
#
# 2. Everything from `#[cfg(test)]` to end of file. Test assertion messages are
#    prose, but they are never rendered to an operator; they are read by whoever
#    is staring at a failing test. Including them was the single biggest source
#    of pdfcer's 140-hit noise floor (125 of them). This codebase puts its test
#    module last, so a truncation is exact rather than a guess.
#
#    LIMIT, found the embarrassing way in pdfcer: because this truncates, any
#    non-test code placed AFTER the test module is invisible to the checker. It
#    surfaced while planting a deliberate violation to prove the gate still bit
#    — the plant was appended to end-of-file, the gate stayed green, and it
#    looked as though the fix had produced a gate that could only pass.
#    Re-planting it above `#[cfg(test)]` caught it correctly.
#
#    Two lessons kept rather than quietly dropped: verify a gate by making it
#    FAIL on purpose, never only by making it pass; and Rust after `mod tests`
#    is unusual but legal, so if that convention is ever broken this exclusion
#    silently stops covering the tail of the file.
#
# 3. Lines inside an `impl ... Display for ...` block. `Display` formats
#    DIAGNOSTIC text — an error's own description of itself — which is a
#    different audience and a different lifecycle from UI copy. Tracked by brace
#    depth so it ends where the impl ends, not at a blank line.
#
#    CAVEAT, stated rather than hidden: if an error's `Display` output is ever
#    shown verbatim in the GUI, that string HAS become operator-visible and
#    belongs in the catalog. This exclusion is not permission to route UI text
#    through an error type.
#
# 4. The body of a `diag::trace(...)` call — stderr diagnostics, never operator
#    copy. Tracked by PAREN depth so the skip ends exactly where the call does.
#
# 5. Comment-only lines, and any line carrying `// ui-text-exempt: <reason>`.
#
# ===========================================================================
# HEURISTIC AND ITS KNOWN LIMIT  (from the original)
# ===========================================================================
# It flags string literals containing whitespace. That is a proxy for "prose",
# and it is leaky in both directions: "Linear" is operator-visible but has no
# space, so the gate would never catch it (it was moved to the catalog anyway,
# because the RULE is about visibility, not about what grep can see); and a
# whitespace-bearing literal may be an egui id or a format spec. The exemption
# comment exists for the latter. Do not mistake a green run here for proof that
# the catalog is complete.
#
# ===========================================================================
# USAGE / EXIT CODES
# ===========================================================================
#   tools/gates/check-ui-strings.sh              scan crates/pdfcer-gui/src
#   tools/gates/check-ui-strings.sh <SRC_DIR>    scan an arbitrary tree
#   tools/gates/check-ui-strings.sh --self-test  prove the gate bites
#
#   0  clean
#   1  violations found (printed), or the self-test failed
#   2  PRECONDITION ABSENT — the tree does not exist yet. Deliberately NOT 0:
#      "nothing to scan" must never read as "scanned and clean", which is the
#      exact failure this port exists to remove. run-all.sh renders 2 as
#      SKIPPED and refuses to call the overall run a pass.

set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# ---------------------------------------------------------------------------
# CATALOG_RELPATHS — what counts as the catalog, relative to the SCAN ROOT.
#
# Shell case-patterns, matched against a path with the scan root stripped off.
# Anchored deliberately: see PORT CHANGE 2. Adding an entry here is a decision
# about what the rule does NOT cover, so it belongs in one reviewable list
# rather than smuggled into a `find -not -path` somewhere.
#
#   text.rs, text/*        the catalog in THIS project (crates/pdfcer-gui/src/text/)
#   ui_text.rs, ui_text/*  the spelling carried over from pdfcer, kept so a
#                          rename in either direction cannot silently disarm
#                          the exclusion
# ---------------------------------------------------------------------------
CATALOG_RELPATHS=('text.rs' 'text/*' 'ui_text.rs' 'ui_text/*')

# is_catalog <relative-path> — 0 if this file IS the catalog, 1 otherwise.
is_catalog() {
    local rel="$1" pat
    for pat in "${CATALOG_RELPATHS[@]}"; do
        # shellcheck disable=SC2254   # $pat is a glob on purpose
        case "$rel" in
            $pat) return 0 ;;
        esac
    done
    return 1
}

# ---------------------------------------------------------------------------
# scan_tree <dir> — print one `file:line:text` per violation, return 1 if any.
#
# Split out from the old inline loop so the self-test can aim it at a fixture.
# A gate whose scanner cannot be pointed anywhere else cannot be tested.
# ---------------------------------------------------------------------------
scan_tree() {
    local src_dir="$1"
    local hits="" file file_hits

    # PORT CHANGE 1: `find`, not `"$src_dir"/*.rs`. `-print0`/`read -d ''` so a
    # path containing a space cannot split into two nonexistent paths. `sort -z`
    # so the report is stable between machines and reviewable in a diff.
    while IFS= read -r -d '' file; do
        # PORT CHANGE 2: the catalog is a file OR a directory, matched against
        # the path RELATIVE to the scan root so the anchoring holds.
        is_catalog "${file#"$src_dir"/}" && continue

        file_hits=$(awk -v fname="$file" '
            # exclusion 2: stop at the test module — everything after is test-only.
            /^#\[cfg\(test\)\]/ { exit }

            # exclusion 2b: a WHOLE FILE gated out of release builds, marked by
            # the INNER attribute `#![cfg(test)]`. Everything in it is test-only
            # for exactly the reason exclusion 2 gives, and the reason has to be
            # recognised from the file rather than from its name.
            #
            # ★ Added 2026-08-18, when `canvas/selection/tests.rs` was split out
            # under R2 and this gate reported 28 assertion messages as
            # operator-facing copy. The gate was right that they were string
            # literals and wrong that anybody would ever read them on screen —
            # and the noise is the actual hazard: 125 of pdfcer'"'"'s old 140-hit
            # floor were test assertions, which is what exclusion 2 was written
            # to remove. A split that reintroduced them would have trained
            # people to ignore the report.
            #
            # `check-theme-colors.sh` already recognises this exact marker, from
            # the AST, and states why it is the marker rather than a filename:
            # the property that earns the exemption is "not in the shipped
            # binary", and a filename is a restatement of that which goes stale
            # the moment a third such module is written. Same rule here, matched
            # on the line because this scanner is awk rather than syn.
            /^#!\[cfg\(test\)\]/ { exit }

            {
                line = $0

                # exclusion 3: track an `impl ... Display for ...` block by brace
                # depth, so the skip ends exactly where the impl does.
                if (in_display) {
                    depth += gsub(/\{/, "{", line)
                    depth -= gsub(/\}/, "}", line)
                    if (depth <= 0) { in_display = 0 }
                    next
                }
                if (line ~ /impl[[:space:]].*Display[[:space:]]+for[[:space:]]/) {
                    in_display = 1
                    depth = gsub(/\{/, "{", line) - gsub(/\}/, "}", line)
                    if (depth <= 0) { in_display = 0 }
                    next
                }

                # exclusion 4: the body of a `diag::trace(...)` call.
                #
                # These are stderr diagnostics, never operator copy. They are
                # excluded as a CATEGORY rather than by tagging each one, for two
                # reasons: the offending literal often sits inside a multi-line
                # `format!`, where a trailing `// ui-text-exempt:` cannot reach it
                # and a comment block above would exempt the `diag::trace(` line
                # rather than the string; and a rule that has to be re-stated at
                # every call site is a rule that will be forgotten at the next one.
                #
                # Tracked by PAREN depth, so the skip ends exactly where the call
                # does and a real operator string on the following line is still
                # caught. Deliberately NOT keyed on `format!` alone, which would
                # excuse every formatted string in the crate.
                if (in_diag) {
                    depth_diag += gsub(/\(/, "(", line)
                    depth_diag -= gsub(/\)/, ")", line)
                    if (depth_diag <= 0) { in_diag = 0 }
                    next
                }
                if (line ~ /diag::trace\(/) {
                    in_diag = 1
                    depth_diag = gsub(/\(/, "(", line) - gsub(/\)/, ")", line)
                    if (depth_diag <= 0) { in_diag = 0 }
                    next
                }

                # exclusion 5: comment-only lines, and explicit exemptions.
                #
                # An exemption counts either on the offending line itself, or
                # anywhere in the contiguous comment block immediately above it.
                # The block form exists because this project asks for reasons, not
                # tokens: "// ui-text-exempt: stderr diagnostic" trailing a line is
                # fine, but a real justification runs several lines and belongs
                # above the code rather than smeared past column 100.
                if (line ~ /^[[:space:]]*\/\//) {
                    if (line ~ /ui-text-exempt:/) { block_exempt = 1 }
                    next            # still a comment line: block continues
                }
                if (line ~ /ui-text-exempt:/) { block_exempt = 0; next }
                if (block_exempt) { block_exempt = 0; next }

                # The heuristic: a string literal containing whitespace.
                #
                # Scan the line character by character rather than regex-matching
                # `"[^"]*[[:space:]][^"]*"`. That pattern is wrong in a way that
                # matters: it happily starts at one literal CLOSING quote and ends
                # at the next literal OPENING quote, so `"svg" | "?xml"` reads as a
                # single literal containing " | ". Three of the four remaining hits
                # when pdfcer first ran this were exactly that artefact — i.e. most
                # of what was left after the real exclusions was the detector
                # misreading Rust, not the code violating the rule.
                #
                # A scanner that toggles on unescaped quotes cannot make that
                # mistake, because it knows which quotes open and which close.
                n = length(line)
                in_str = 0
                lit = ""
                for (i = 1; i <= n; i++) {
                    ch = substr(line, i, 1)
                    if (in_str) {
                        if (ch == "\\") { i++; lit = lit "x"; continue }
                        if (ch == "\"") {
                            in_str = 0
                            if (lit ~ /[[:space:]]/) {
                                printf "%s:%d:%s\n", fname, NR, line
                                next
                            }
                            continue
                        }
                        lit = lit ch
                    } else if (ch == "\"") {
                        in_str = 1
                        lit = ""
                    }
                }
            }
        ' "$file")

        if [ -n "$file_hits" ]; then
            hits="${hits}${file_hits}
"
        fi
    done < <(find "$src_dir" -type f -name '*.rs' -print0 | sort -z)

    hits=$(printf '%s' "$hits" | sed '/^$/d')
    if [ -n "$hits" ]; then
        printf '%s\n' "$hits"
        return 1
    fi
    return 0
}

# ---------------------------------------------------------------------------
# count_files <dir>  — how many .rs files the recursive scan actually reads.
#
# Reported on every clean run. PROJECT_PLAN.md §4.1 asks for gates that are
# "demonstrably scanning the module tree", and the only honest way to
# demonstrate it is to say how many files were read. `clean — 0 files scanned`
# is the sound of a gate guarding nothing, and now it is audible.
# ---------------------------------------------------------------------------
count_files() {
    local src_dir="$1" f n=0
    while IFS= read -r -d '' f; do
        is_catalog "${f#"$src_dir"/}" && continue
        n=$((n + 1))
    done < <(find "$src_dir" -type f -name '*.rs' -print0)
    printf '%s' "$n"
}

# ---------------------------------------------------------------------------
# self_test — the gate demonstrates it detects its own violation.
#
# Three assertions, and the third is the one that would have caught the port's
# original bug:
#
#   A. the CLEAN fixture passes            (no false positives)
#   B. the DIRTY fixture fails             (the gate can bite at all)
#   C. the dirty fixture's ONLY violation lives at src/app/state.rs — two
#      levels down — and `src/*.rs` provably does not match it. So B can only
#      succeed with a recursive scan. Assertion C is what makes B a regression
#      test for PORT CHANGE 1 rather than a generic smoke test.
# ---------------------------------------------------------------------------
self_test() {
    local fx="$HERE/fixtures/ui-strings"
    local rc=0

    if [ ! -d "$fx/clean/src" ] || [ ! -d "$fx/dirty/src" ]; then
        echo "ui-strings self-test: FAIL — fixtures missing under $fx" >&2
        return 1
    fi

    echo "ui-strings self-test:"

    # --- A. clean must pass -------------------------------------------------
    if scan_tree "$fx/clean/src" > /dev/null; then
        echo "  [ok]   clean fixture passes ($(count_files "$fx/clean/src") files scanned)"
    else
        echo "  [FAIL] clean fixture reported violations — the gate has false positives:"
        scan_tree "$fx/clean/src" | sed 's/^/         /' || true
        rc=1
    fi

    # --- B. dirty must fail -------------------------------------------------
    local dirty_hits
    dirty_hits=$(scan_tree "$fx/dirty/src" || true)
    if [ -n "$dirty_hits" ]; then
        echo "  [ok]   dirty fixture fails as designed:"
        printf '%s\n' "$dirty_hits" | sed 's/^/         /'
    else
        echo "  [FAIL] dirty fixture reported CLEAN — the gate cannot detect its own violation."
        echo "         This is the pdfcer failure mode verbatim: a green gate that guards nothing."
        rc=1
    fi

    # --- C. the violation is out of a flat glob's reach ---------------------
    #
    # Asserted mechanically, not by comment. `printf '%s\n' "$fx"/dirty/src/*.rs`
    # is exactly the expression the original gate used; if it ever matches the
    # planted file, assertion B has stopped proving recursion and this self-test
    # says so instead of quietly degrading.
    local planted="$fx/dirty/src/app/state.rs"
    if [ ! -f "$planted" ]; then
        echo "  [FAIL] the planted violation $planted is gone; assertion C is vacuous"
        rc=1
    else
        local flat_reach=0 f
        for f in "$fx"/dirty/src/*.rs; do
            [ "$f" = "$planted" ] && flat_reach=1
        done
        if [ "$flat_reach" -eq 0 ]; then
            echo "  [ok]   the planted violation is at src/app/state.rs, which a flat"
            echo "         \`src/*.rs\` glob cannot reach — so assertion B proves recursion,"
            echo "         which is the bug this port exists to fix."
        else
            echo "  [FAIL] the planted violation is reachable by a flat glob; assertion B"
            echo "         no longer proves the recursion fix. Move it back into a subdir."
            rc=1
        fi
    fi

    if [ "$rc" -eq 0 ]; then
        echo "  self-test: PASS"
    else
        echo "  self-test: FAIL"
    fi
    return "$rc"
}

# ---------------------------------------------------------------------------
# main
# ---------------------------------------------------------------------------
if [ "${1:-}" = "--self-test" ]; then
    self_test
    exit $?
fi

SRC_DIR="${1:-crates/pdfcer-gui/src}"

if [ ! -d "$SRC_DIR" ]; then
    echo "ui-strings: SKIPPED — no $SRC_DIR" >&2
    echo "  Run from the repository root, or pass a tree to scan." >&2
    echo "  Exiting 2, not 0: an unscanned tree is not a clean tree." >&2
    exit 2
fi

if hits=$(scan_tree "$SRC_DIR"); then
    echo "ui-strings: clean — $(count_files "$SRC_DIR") .rs file(s) scanned recursively under $SRC_DIR,"
    echo "            no operator-visible literals outside the catalog"
    echo "            catalog (excluded, relative to $SRC_DIR): ${CATALOG_RELPATHS[*]}"
    exit 0
fi

printf '%s\n' "$hits"
count=$(printf '%s\n' "$hits" | grep -c '^')
echo ""
echo "error: $count user-facing string literal(s) outside the ui_text catalog."
echo "Move each into the catalog (rule R1), or, if it is genuinely not"
echo "operator-visible, append '// ui-text-exempt: <reason>' to the line"
echo "(or put the reason in the comment block directly above it)."
exit 1
