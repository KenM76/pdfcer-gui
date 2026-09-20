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
# 2. The BODY of a `#[cfg(test)]` item, and nothing else. Test assertion
#    messages are prose, but they are never rendered to an operator; they are
#    read by whoever is staring at a failing test. Including them was the single
#    biggest source of pdfcer's 140-hit noise floor (125 of them).
#
#    ★ The skip is BOUNDED. A scanner that stops at the first column-0
#    `#[cfg(test)]` and never resumes fails open across most of this tree: 508
#    files here carry such a line, and 66 of them declare shipped items below
#    it. "Found no violations" and "looked at almost nothing" are byte-identical
#    output, so the bound is the whole safety property.
#
#    ★★ The dominant shape is `#[cfg(test)] mod tests;` — a one-line
#    declaration whose body lives in a sibling FILE — sitting with the other
#    `mod` lines near the TOP. A "keep the test module last" convention does not
#    describe those files at all: they have no test module in them to be last,
#    and they gate out nothing. So the attribute line is inspected. A
#    `;`-terminated declaration skips that line only; a braced item is skipped
#    to its close.
#
#    ★★★ The close is `}` at COLUMN 0, not a brace counter. `cargo fmt` is a
#    gate here, so a column-0 item closes with a bare `}` at column 0, whereas a
#    counter desyncs silently on a brace inside a string literal and is
#    confidently wrong for the rest of the file. A `#[cfg(test)]` indented
#    inside another item is not matched by this rule and never was.
#
#    2c. A cfg-gated FILE is dropped whole. `#[cfg(test)] mod tests;` names a
#    file the compiler never emits into a release build, so nothing in it can
#    reach the operator by any route; the in-file skip cannot see that, because
#    it reads the named file from line 1 as ordinary shipped source. Each such
#    declaration is resolved to `<name>.rs` or `<name>/mod.rs`, and a module
#    directory reached only that way is dropped with everything under it.
#
#    Verify a gate by making it FAIL on purpose, never only by making it pass.
#    `--self-test` falsifies each of these branches separately, because a skip
#    that never starts and a skip that never ends both leave a merely
#    "does the dirty fixture fail?" assertion green.
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
# 4. The body of a `diag::trace(...)` call, and of its de-duplicating twins
#    `diag::trace_changed(...)` and `diag::trace_on_change(...)` — stderr
#    diagnostics, never operator copy. Tracked by PAREN depth so the skip ends
#    exactly where the call does.
#
#    `debug_assert!` / `debug_assert_eq!` / `debug_assert_ne!` are in the same
#    category on a stronger argument: they expand to nothing in a release
#    build, so the message is not in the shipped binary by any route.
#    `assert!`, `panic!`, `unreachable!` and `expect()` are NOT — they survive
#    into release and their text reaches stderr, so each is tagged by hand.
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
# A SECOND LEAK, IN THE SHAPE OF THE LINE RATHER THAN THE SHAPE OF THE STRING.
# The scanner is line-based: it sees a literal only when the opening AND the
# closing quote sit on ONE physical line. A backslash-continued Rust string is
# therefore invisible to it, and the identical prose written on one line is
# flagged. Two neighbours in this crate demonstrate both halves -- a long
# `#[expect(..., reason = "...")]` attribute passes when its reason is
# continued and fails when it is not, and nothing about the audience changed.
# So: a hit on a compiler-facing attribute is the gate measuring typography,
# not visibility. Exempt it with a `// ui-text-exempt:` block INSIDE the
# attribute (a trailing comment cannot reach a multi-line construct) rather
# than re-wrapping to dodge the scanner, because the wrap is what makes the
# next reader think the rule is about line length.
#
# A THIRD LEAK, AND IT IS A DELIBERATE BOUND ON THE RAW-STRING TRACKER.
# The skip in exclusion 2 has to know when a `}` at column 0 is inside a raw
# string rather than closing the test item, so it tracks raw-string delimiters
# — but only INSIDE a test item, and only for openers carrying at least one
# hash (`r#"`, `r##"`, …).
#
# Both bounds are there because the obvious pattern is unsafe. `r#*"` is
# exactly the grammar and it also matches the last two characters of any
# ordinary literal ending in `r`: `"reviewer"` opens a phantom zero-hash raw
# string that never closes, and every remaining line of that file is read as
# string interior. Measured over `crates/pdfcer-gui/src`: 25 openers carry at
# least one hash, while the naive `r#*"` pattern matches 1,066 times — so
# roughly 1,041 of its matches are the tail of an ordinary literal. So the
# opener is
# `/(^|[^A-Za-z0-9_])r#+"/` — one hash minimum, and a non-identifier character
# before the `r`, checked inside the match because POSIX regex has no
# lookbehind.
#
# What that COSTS, stated rather than hidden: a genuine zero-hash raw string
# (`r"…"`) inside a test item is not recognised, so a `}` at column 0 inside
# one would end the skip early — the direction that reports MORE, never less.
# Outside a test item raw strings are not tracked at all, which is harmless
# because nothing outside a test item depends on knowing.
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
# TEST_ONLY_PATHS — files that are not in the shipped binary at all.
#
# A module declared `#[cfg(test)] mod fixture;` lives in its own FILE. The
# in-file skip below cannot see that: it reads `fixture.rs` from line 1 as
# ordinary shipped source, and every PDF byte-string and every `expect()`
# message in it is reported as operator-facing copy. It is none of those
# things — the compiler never emits that file into a release build, so no
# string in it can ever reach the operator.
#
# So the exclusion is drawn where the compiler draws it: resolve each
# `#[cfg(test)] mod NAME;` declaration to the file it names and drop that file
# from the scan. A module directory reached only through such a declaration is
# dropped whole, because every file under it is reachable only through the
# cfg-gated parent.
#
# This is narrower than it looks. Rust permits exactly one declaration per
# module path, so a file cannot be both cfg-gated here and shipped elsewhere;
# there is no second route by which the excluded file re-enters the build.
#
# Populated by collect_test_only(); one absolute path or directory prefix per
# line, each ending without a trailing slash.
# ---------------------------------------------------------------------------
TEST_ONLY_PATHS=""

# collect_test_only <dir> — fill TEST_ONLY_PATHS for one scan root.
#
# Two spellings of the declaration are recognised, because both occur here:
# the one-liner `#[cfg(test)] mod tests;` and the attribute on its own line
# above the `mod`. The braced form `#[cfg(test)] mod tests { ... }` is NOT
# matched — that module has no file of its own and is handled by the in-file
# skip instead.
collect_test_only() {
    local src_dir="$1" decl name dir base cand p
    TEST_ONLY_PATHS=""

    while IFS=$'\t' read -r decl name; do
        [ -n "$decl" ] || continue
        dir="$(dirname "$decl")"
        base="$(basename "$decl" .rs)"
        # A `mod.rs`/`lib.rs`/`main.rs` owns its OWN directory; any other file
        # owns a sibling directory named after it.
        case "$base" in
            mod | lib | main) cand="$dir" ;;
            *) cand="$dir/$base" ;;
        esac
        for p in "$cand/$name.rs" "$cand/$name/mod.rs"; do
            if [ -f "$p" ]; then
                TEST_ONLY_PATHS="${TEST_ONLY_PATHS}${p}
"
                # The whole subtree, not just the entry file.
                TEST_ONLY_PATHS="${TEST_ONLY_PATHS}${cand}/${name}/
"
            fi
        done
    done < <(find "$src_dir" -type f -name '*.rs' -print0 | xargs -0 awk '
        FNR == 1 { armed = 0 }
        /^[[:space:]]*#\[cfg\(test\)\]/ {
            rest = $0
            sub(/^[[:space:]]*#\[cfg\(test\)\][[:space:]]*/, "", rest)
            if (rest ~ /^(pub[[:space:]]+)?mod[[:space:]]+[A-Za-z_][A-Za-z0-9_]*[[:space:]]*;[[:space:]]*$/) {
                sub(/^(pub[[:space:]]+)?mod[[:space:]]+/, "", rest)
                sub(/[[:space:]]*;.*$/, "", rest)
                print FILENAME "\t" rest
                armed = 0
                next
            }
            armed = (rest == "") ? 1 : 0
            next
        }
        armed {
            if ($0 ~ /^[[:space:]]*(pub[[:space:]]+)?mod[[:space:]]+[A-Za-z_][A-Za-z0-9_]*[[:space:]]*;[[:space:]]*$/) {
                rest = $0
                sub(/^[[:space:]]*(pub[[:space:]]+)?mod[[:space:]]+/, "", rest)
                sub(/[[:space:]]*;.*$/, "", rest)
                print FILENAME "\t" rest
            }
            armed = 0
        }
    ')
}

# is_test_only <path> — 0 if this file is cfg(test)-gated, 1 otherwise.
is_test_only() {
    local path="$1" entry
    while IFS= read -r entry; do
        [ -n "$entry" ] || continue
        case "$entry" in
            */) case "$path" in "$entry"*) return 0 ;; esac ;;
            *) [ "$path" = "$entry" ] && return 0 ;;
        esac
    done <<< "$TEST_ONLY_PATHS"
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

    collect_test_only "$src_dir"

    # PORT CHANGE 1: `find`, not `"$src_dir"/*.rs`. `-print0`/`read -d ''` so a
    # path containing a space cannot split into two nonexistent paths. `sort -z`
    # so the report is stable between machines and reviewable in a diff.
    while IFS= read -r -d '' file; do
        # PORT CHANGE 2: the catalog is a file OR a directory, matched against
        # the path RELATIVE to the scan root so the anchoring holds.
        is_catalog "${file#"$src_dir"/}" && continue
        is_test_only "$file" && continue

        file_hits=$(awk -v fname="$file" '
            # raw_scan(line) — advance `rawhash` across one line.
            #
            # A Rust raw string can hold ANY text, including a bare `}` at
            # column 0, and `icons/glyphs.rs` holds exactly that: its own
            # fixture for this gate is a miniature source file embedded in an
            # `r#"..."#`. Without this, the test-module skip below resumed on
            # that brace and reported the fixture back as operator copy. So the
            # skip has to know where a raw string begins and ends.
            #
            # `rawhash` is the number of `#`s of the raw string currently open,
            # or -1 for none. Matched loosely: an `r#*"` inside an ORDINARY
            # string literal would be read as an opener, which widens the skip
            # rather than narrowing it, and only within a test item.
            function raw_scan(line,   i, n, rest, idx, endmark) {
                if (rawhash >= 0) {
                    endmark = "\""
                    for (i = 0; i < rawhash; i++) endmark = endmark "#"
                    idx = index(line, endmark)
                    if (idx == 0) return
                    rawhash = -1
                    line = substr(line, idx + length(endmark))
                }
                while (match(line, /(^|[^A-Za-z0-9_])r#+"/)) {
                    n = RLENGTH - 3
                    if (substr(line, RSTART, 1) == "r") { n = RLENGTH - 2 }
                    rest = substr(line, RSTART + RLENGTH)
                    endmark = "\""
                    for (i = 0; i < n; i++) endmark = endmark "#"
                    idx = index(rest, endmark)
                    if (idx == 0) { rawhash = n; return }
                    line = substr(rest, idx + length(endmark))
                }
            }

            BEGIN { rawhash = -1 }


            # exclusion 2: skip the test item and RESUME after it. See the
            # header for why this is bounded rather than an exit: the exiting
            # version left shipped items in 66 files unscanned while reporting
            # clean. Three shapes, in the order they are decided:
            #
            #   `#[cfg(test)] mod tests;`   a declaration with no body here —
            #                               skip the line, nothing else
            #   `#[cfg(test)] mod t { ..`   brace opened on the attribute line
            #   `#[cfg(test)]` then a decl  the usual multi-line form, which may
            #                               carry further attributes before its
            #                               `{` or its `;`
            !in_test && /^#\[cfg\(test\)\]/ {
                rest = $0
                sub(/^#\[cfg\(test\)\][[:space:]]*/, "", rest)
                if (rest ~ /;[[:space:]]*$/) next
                if (rest ~ /\{/) { in_test = 1; pending = 0; next }
                in_test = 1; pending = 1; next
            }
            in_test {
                was_raw = rawhash
                raw_scan($0)
                if (was_raw >= 0) { next }
                # `pending` means the opening `{` of the item has not been seen
                # yet, so a `;` here ends a bodyless declaration rather than a
                # statement inside a body. (No apostrophes in this awk program:
                # it is a single-quoted shell string, and one would close it.)
                if (pending && $0 ~ /;[[:space:]]*$/ && $0 !~ /\{/) {
                    in_test = 0; pending = 0; rawhash = -1; next
                }
                if ($0 ~ /\{/) { pending = 0 }
                # A BARE `}` at column 0 and nothing else. `^\}` alone is too
                # loose: `redact/sealed.rs` closes a nested raw string with `}}`
                # at column 0 inside its test module, and the looser rule
                # resumed scanning 139 lines early and reported 87 assertion
                # messages as operator copy. What cargo fmt guarantees is the
                # bare form, so that is what this matches.
                if ($0 ~ /^\}[[:space:]]*$/) { in_test = 0; pending = 0; rawhash = -1 }
                next
            }

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
            !in_test && /^#!\[cfg\(test\)\]/ { exit }

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

                # exclusion 4: the body of a `diag::trace(...)` call, and of
                # `diag::trace_changed(...)` / `diag::trace_on_change(...)`.
                #
                # All three write the same stderr line; the twins only add a
                # de-duplication slot. Keying this on `diag::trace(` alone left
                # 49 call sites re-stating an exemption the category already
                # grants, which is the outcome the paragraph below says the
                # category exists to prevent — and the fiftieth forgot it.
                #
                # These are stderr diagnostics, never operator copy. They are
                # excluded as a CATEGORY rather than by tagging each one, for two
                # reasons: the offending literal often sits inside a multi-line
                # `format!`, where a trailing `// ui-text-exempt:` cannot reach it
                # and a comment block above would exempt the `diag::trace(` line
                # rather than the string; and a rule that has to be re-stated at
                # every call site is a rule that will be forgotten at the next one.
                #
                # `debug_assert!` / `debug_assert_eq!` / `debug_assert_ne!` join
                # the same category on a STRONGER argument: those macros expand
                # to nothing in a release build, so their message is not in the
                # shipped binary at all and cannot reach anyone by any route.
                # `assert!`, `panic!`, `unreachable!` and `expect()` deliberately
                # do NOT join it — those survive into the release binary and
                # their text does reach stderr, so each one is tagged by hand.
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
                if (line ~ /(diag::trace(_changed|_on_change)?|debug_assert(_eq|_ne)?!)[[:space:]]*\(/) {
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
        ' "$file") || {
            # A SCANNER THAT CRASHES MUST NOT REPORT CLEAN.
            #
            # awk writes its diagnostics to stderr and exits nonzero, and this
            # assignment used to discard both: the empty hit list read as
            # success and the gate printed `clean` over a file it had not
            # parsed. Twice during this scanner's development a syntax error in
            # the awk program made every file fail and the gate still exited 0.
            # That is the same fail-open shape the gate exists to close, so it
            # is fatal here rather than a warning: no report at all is safer
            # than a clean report nobody can trust.
            echo "ui-strings: SCANNER FAILED — awk exited nonzero reading" >&2
            echo "            $file" >&2
            echo "            The awk program above is broken; its stderr is in this output." >&2
            echo "            This is a fault in the GATE, not a violation in the tree." >&2
            exit 1
        }

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
    collect_test_only "$src_dir"
    while IFS= read -r -d '' f; do
        is_catalog "${f#"$src_dir"/}" && continue
        is_test_only "$f" && continue
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

    # --- D/E/F. the three test-item shapes, asserted on the hit LIST --------
    #
    # Assertion B only requires the dirty fixture to fail SOMEHOW, and it would
    # stay green while the scanner stopped reading at the first `#[cfg(test)]`
    # — which is exactly what it used to do. These assertions name what the
    # hit list must and must not contain, so each shape is measured separately:
    #
    #   D  a one-line `#[cfg(test)] mod X;` declaration must not blind the
    #      scanner to the shipped items below it. This is the shape 66 files in
    #      the real tree were hiding behind.
    #   E  a braced `#[cfg(test)] mod X { ... }` must skip its BODY and resume
    #      at the item's closing brace — both halves, because a skip that never
    #      ends and a skip that never starts both make assertion B pass. Both
    #      spellings are exercised: the attribute on its own line, and the
    #      attribute and item on one line. The scanner takes a different branch
    #      for each, and only the two-line branch was covered until a
    #      falsification of the one-line branch changed nothing and said so.
    #   F  a file reached only through a cfg-gated declaration is excluded
    #      whole, so its fixture prose is not reported as operator copy.
    local want_shape1="Save a copy of this drawing"
    local want_shape2="Close without saving"
    local want_shape3="Print the current sheet"
    local never_body="this assertion message must never be reported"
    local never_oneline="this one-line module message must never be reported"

    if printf '%s\n' "$dirty_hits" | grep -qF "$want_shape1"; then
        echo "  [ok]   a one-line \`#[cfg(test)] mod X;\` does not blind the scanner"
    else
        echo "  [FAIL] the violation below \`#[cfg(test)] mod tests;\` was NOT reported."
        echo "         The scanner stops at the declaration instead of skipping it."
        rc=1
    fi

    if printf '%s\n' "$dirty_hits" | grep -qF "$want_shape2"; then
        echo "  [ok]   the braced test module's skip ends at its closing brace"
    else
        echo "  [FAIL] the violation below \`mod inline_tests { ... }\` was NOT reported."
        echo "         The skip is running past the item's closing brace at column 0."
        rc=1
    fi

    if printf '%s\n' "$dirty_hits" | grep -qF "$want_shape3"; then
        echo "  [ok]   a one-line \`#[cfg(test)] mod X { ... }\` skip ends at its brace"
    else
        echo "  [FAIL] the violation below the one-line \`#[cfg(test)] mod oneline_tests\`"
        echo "         was NOT reported. The one-line branch of the skip is broken."
        rc=1
    fi

    if printf '%s\n' "$dirty_hits" | grep -qF "$never_oneline"; then
        echo "  [FAIL] a message inside the ONE-LINE braced test module was reported."
        echo "         That branch is not starting the skip at the attribute."
        rc=1
    else
        echo "  [ok]   messages inside a one-line braced test module are skipped"
    fi

    if printf '%s\n' "$dirty_hits" | grep -qF "$never_body"; then
        echo "  [FAIL] an assertion message INSIDE the braced test module was reported."
        echo "         The skip is not starting at the \`#[cfg(test)]\` attribute."
        rc=1
    else
        echo "  [ok]   assertion messages inside a braced test module are skipped"
    fi

    if printf '%s\n' "$dirty_hits" | grep -q 'state/tests\.rs'; then
        echo "  [FAIL] \`state/tests.rs\` was reported. It is declared"
        echo "         \`#[cfg(test)] mod tests;\`, so it is not in a release build at all."
        rc=1
    else
        echo "  [ok]   a cfg-gated FILE is excluded whole, not scanned as shipped source"
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

# Default: BOTH GUI crates. The floor crate carries 30 `ui-text-exempt:`
# markers, and a marker is only meaningful while something scans the file it
# sits in — drop that root and the exemptions become decorative while the
# literals beside them go unchecked, with no gate turning red.
#
# An explicit argument list overrides the default, which is what an ad-hoc
# scan of one tree uses.
if [ "$#" -gt 0 ]; then
    SRC_DIRS=("$@")
else
    SRC_DIRS=("crates/pdfcer-gui/src" "crates/pdfcer-gui-base/src")
fi

for SRC_DIR in "${SRC_DIRS[@]}"; do
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
    continue
fi

printf '%s\n' "$hits"
count=$(printf '%s\n' "$hits" | grep -c '^')
echo ""
echo "error: $count user-facing string literal(s) outside the ui_text catalog."
echo "Move each into the catalog (rule R1), or, if it is genuinely not"
echo "operator-visible, append '// ui-text-exempt: <reason>' to the line"
echo "(or put the reason in the comment block directly above it)."
exit 1
done

# Every root scanned clean.
exit 0
