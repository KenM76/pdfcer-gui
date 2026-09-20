#!/usr/bin/env bash
#
# check-engine-citation.sh — nothing in this repository may cite a line number
# in a tree this repository does not own.
#
# ===========================================================================
# WHY THIS GATE EXISTS
# ===========================================================================
#
# `Cargo.lock` takes the engine as
# `git+file:///D:/Dev/pdfcer?branch=main#<sha>` with **no `rev`**. The source
# behind that dependency therefore moves with no command run in this
# repository — no `cargo update`, no fetch, no edit here. A `<file>.rs:<N>`
# citation into it rots with no local event to notice.
#
# ★ And it rots into something WORSE than a dangling reference. Upstream
# *insertion* shifts a whole file uniformly in one direction rather than
# scrambling it, so a drifted number still lands inside readable prose about a
# real function in the right file. It reads exactly as a correct citation
# reads. Nothing about it looks wrong, which is why the class survives review
# and why it needs a machine rather than a careful reader.
#
# Measured on one sweep of four living documents plus the forms parity table:
# **66 of 92** engine citations had drifted, none of them into a dangling
# reference, and two pairs had come to name each OTHER'S symbol — two live
# line numbers, each attached to the wrong type. A repair keyed on *"does this
# line exist"* would have passed every one of them.
#
# The same argument applies verbatim to `D:\Dev\pdfce`, the archived GUI this  (old-name-exempt: the archive's real path, which is what this paragraph is about)
# project replaces. A frozen tree is NOT frozen-correct: citations into it
# were written against an in-flight tree and kept drifting until the freeze,
# so what froze was the error. Seven of eleven measured were wrong by +2 to
# +29 lines. **The archive is IN this gate's scope, deliberately.**
#
# R5 already says cite the engine by SYMBOL. This makes the tree obey it.
#
# ===========================================================================
# WHAT IS ALLOWED, AND WHY
# ===========================================================================
#
# A **version-anchored vendor path** — `egui-0.35.0/src/style.rs:1135` — is
# allowed and is the shape to use for a dependency pinned by version. It
# cannot move without a deliberate `Cargo.toml` bump, and the bump then has a
# single grep as its worklist. The detector is the `<name>-<major>.<minor>.<patch>/`
# segment, which is exactly how `~/.cargo/registry/src/.../` names a crate.
#
# An **unversioned** vendor citation (`style.rs:1135`) is NOT the same thing
# and is not allowed by that rule: it names no revision, so it makes the same
# unfalsifiable claim an engine citation makes.
#
# ===========================================================================
# THE FOUR DETECTORS
# ===========================================================================
#
# Each is independent, each has its own self-test case, and each is stated
# with what it CANNOT see.
#
# **D1 — an explicitly engine-owned or archive-owned path.** The citation's
# path contains `pdfcer-core`, `pdfcer-render`, `pdfcer-print`, `pdfcer-cli`,
# `pdfce-gui`, or a `D:\Dev\pdfcer` / `D:\Dev\pdfce` prefix. Zero ambiguity:  (old-name-exempt: the detector's own token list, quoted)
# no file in this repository lives under any of those. Blind to a citation
# that names only a bare file name.
#
# **D2 — an `ENGINE:`-tagged citation carrying a line number.** `FORMS_PARITY.md`
# and the evidence artifacts tag every engine reference `ENGINE:`…``. The tag
# is the author saying which tree they mean, so a line number inside one is
# unambiguous. Blind to a document that does not use the tag.
#
# **D3 — a line number no file in this repository could have.** Rule R2 caps
# every `.rs` file here at `check-file-size.sh`'s limit, so a `.rs:<N>` with
# `N` past that limit provably names a file somewhere else. This is the
# detector that catches a bare `edit.rs:25026`, where D1 and D2 both see
# nothing and where this repository's own `panels/forms/edit.rs` makes the
# basename look local.
#
#   ★ D3 BORROWS ANOTHER GATE'S INVARIANT, so it asserts it rather than
#   assuming it. Before scanning, this gate measures the largest tracked `.rs`
#   file. If that exceeds the limit D3 is keyed on, D3's premise is false and
#   this gate exits 2 rather than reporting a clean scan it did not earn. A
#   detector whose premise has quietly stopped holding is worse than no
#   detector, because it still prints green.
#
# **D4 — an orphan continuation citation.** A backticked `` `:25736` `` with
# no file name beside it, reading as the second half of the citation before it.
# It is invisible to D1, D2 and D3 alike because it contains no `.rs` at all,
# and it is the shape that accumulates fastest, because a cell that already
# cited a file adds its next two write sites this way. Same threshold and the
# same borrowed invariant as D3, so the SKIP above covers it too.
#
# ===========================================================================
# WHAT IT PROVABLY CANNOT SEE
# ===========================================================================
#
#   * A BARE ENGINE CITATION WITH A SMALL LINE NUMBER and no `ENGINE:` tag —
#     `hit.rs:411`, say. D1 sees no owning path, D2 sees no tag, and D3 sees
#     a number a local file could plausibly hold. This is the gate's real
#     blind spot and it is not small: it is exactly the shape a citation into
#     a SMALL engine file takes. `forms.rs`, `form_script/mod.rs` and
#     `dimension/length_parse.rs` are all under 5,000 lines, and on the sweep
#     that motivated this gate every citation into those three still LANDED —
#     which is the point: they are the ones a reader will not re-check, and
#     they are the ones this gate cannot guard.
#   * WHETHER A CITATION THAT SURVIVES IS CORRECT. It measures the shape of
#     the reference, not the truth of the claim. A version-anchored vendor
#     citation can name the wrong line and pass forever; opening it is the
#     only thing that finds that, and the moment you are touching one is the
#     cheapest that will ever be.
#   * A CITATION BY SYMBOL THAT NAMES A SYMBOL THE ENGINE NO LONGER HAS. That
#     is a different gate and a harder one, because it needs the engine tree
#     present. This gate needs nothing outside this repository.
#
# ===========================================================================
# THE EVIDENCE EXEMPTION, AND WHY IT IS CONDITIONAL
# ===========================================================================
#
# A file under `evidence/` may cite line numbers freely **if it declares the
# engine revision it was measured at** — a 7-to-40 character hex sha in its
# first 40 lines. That is the correct form for a snapshot: its numbers are
# honest against a named revision and a reader can check them out.
#
# The exemption is conditional on purpose. A blanket `evidence/` skip would
# mean the first evidence file written without provenance silently inherits
# the licence the provenance bought, and nothing would say so.
#
# ===========================================================================
# THE EXIT CONTRACT
# ===========================================================================
#
#   0  scanned, and no forbidden citation found
#   1  at least one forbidden citation
#   2  SKIPPED — could not measure: not run from a repository root, or D3's
#      premise (R2's file-size cap) does not currently hold
#
# `--self-test` plants one violation per detector plus the three shapes that
# must stay quiet, in a scratch directory, and requires the checker to call
# each correctly. The working tree is never touched.

set -u

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"

# D3's threshold. Read from `check-file-size.sh` rather than restated, so the
# two cannot drift apart silently.
SIZE_LIMIT="$(grep -m1 '^LIMIT="\${1:-' "$HERE/check-file-size.sh" 2>/dev/null \
    | sed 's/.*:-//; s/}".*//')"
case "${SIZE_LIMIT:-}" in
    ''|*[!0-9]*) SIZE_LIMIT=1500 ;;
esac

# ---------------------------------------------------------------------------
# ★ EVERY PREDICATE BELOW IS PURE BASH, WITH NO SUBPROCESS, ON PURPOSE.
#
# The first working version of this gate shelled out per citation and took
# **five minutes** on 1,378 files — roughly four thousand process spawns,
# which on Windows is the whole cost. A gate that slow gets skipped, and a
# skipped gate is not a gate. The scan is now three tree-wide greps and a
# loop that spawns nothing.
# ---------------------------------------------------------------------------

# is_version_anchored <citation-path>
#
# True for `egui-0.35.0/src/style.rs`, `.cargo/registry/src/x/serde-1.0.2/…`.
# False for `edit.rs`, `src/style.rs`, `pdfcer-core/src/edit.rs`.
#
# The `-` before the version matters: `pdfcer-core/src/…` must NOT match, and
# it does not, because `core` is not `<digits>.<digits>.<digits>`.
is_version_anchored() {
    [[ "$1" =~ [A-Za-z0-9_]-[0-9]+\.[0-9]+\.[0-9]+(-[A-Za-z0-9.]+)?/ ]]
}

# is_foreign_path <citation-path> — D1
is_foreign_path() {
    [[ "$1" =~ (pdfcer-core|pdfcer-render|pdfcer-print|pdfcer-cli|pdfce-gui) ]] && return 0 # old-name-exempt: the archived crate's real directory name, which this detector exists to match
    [[ "$1" =~ [Dd]:.[Dd]ev.pdfcer?[\\/] ]]
}

# ---------------------------------------------------------------------------
# scan_file <file> — print one `line|citation|detector` record per violation
#
# The citation grammar is deliberately narrow: a path made of the characters a
# Rust path uses, ending `.rs`, followed by `:` and digits. A trailing
# `-<digits>` range is consumed so `edit.rs:3151-3171` is reported whole.
# ---------------------------------------------------------------------------
# classify <citation-text> — sets DET to the detector that fires, or empty.
#
# ★ IT SETS A VARIABLE RATHER THAN ECHOING, because the caller runs it once
# per citation in the tree — several thousand times — and `$(classify …)`
# would fork a subshell for every one of them. Same reasoning as the note
# above the predicates.
#
# `ENGINE:`…`` is tested first: a tagged citation is unambiguous at any line
# number and would otherwise fall through D3's threshold.
DET=""
classify() {
    local cite="$1" path num first
    DET=""
    case "$cite" in
        'ENGINE:`'*)
            case "$cite" in
                *.rs:[0-9]*) DET="D2" ;;
            esac
            return ;;
        '`:'*)
            # D4 — a bare continuation citation. Judged on its first number so
            # a range is reported whole.
            first="${cite#\`:}"
            first="${first%%-*}"
            first="${first%\`}"
            case "$first" in ''|*[!0-9]*) return ;; esac
            [ "$first" -gt "$SIZE_LIMIT" ] && DET="D4"
            return ;;
    esac
    num="${cite##*.rs:}"
    path="${cite%:"$num"}"
    is_version_anchored "$path" && return
    if is_foreign_path "$path"; then DET="D1"; return; fi
    first="${num%%-*}"
    case "$first" in ''|*[!0-9]*) return ;; esac
    [ "$first" -gt "$SIZE_LIMIT" ] && DET="D3"
    return 0
}

# ---------------------------------------------------------------------------
# declares_a_revision <file> — the conditional evidence exemption
# ---------------------------------------------------------------------------
declares_a_revision() {
    head -40 "$1" 2>/dev/null | grep -qE '\b[0-9a-f]{7,40}\b'
}

# ---------------------------------------------------------------------------
# run_checks <root>
# ---------------------------------------------------------------------------
run_checks() {
    local root="$1" rc=0 scanned=0 exempt=0 hits=0
    if [ ! -d "$root" ]; then
        echo "engine-citation: SKIPPED — no such root: $root" >&2
        return 2
    fi

    # D3's premise, asserted rather than assumed.
    #
    # ★ `xargs` BATCHES, so the per-batch `total` rows are excluded by name
    # rather than by position. Reading the last line instead would read one
    # batch's total and call it the maximum.
    local biggest
    biggest=$(find "$root" \( -name target -o -name .git \) -prune -o \
        -type f -name '*.rs' -print0 2>/dev/null \
        | xargs -0 wc -l 2>/dev/null \
        | awk '$2 != "total" { if ($1 + 0 > m) m = $1 + 0 } END { print m + 0 }')
    case "${biggest:-}" in
        ''|*[!0-9]*) biggest=0 ;;
    esac
    if [ "$biggest" -gt "$SIZE_LIMIT" ]; then
        echo "engine-citation: SKIPPED — D3's premise does not hold." >&2
        echo "  Detector D3 reads 'a .rs line number above $SIZE_LIMIT cannot" >&2
        echo "  name a file in this repository', which rests on rule R2. The" >&2
        echo "  largest .rs file here is $biggest lines. Fix check-file-size" >&2
        echo "  first; this gate will not report a clean scan it did not earn." >&2
        return 2
    fi

    # The exempt set, computed once: an evidence artifact that declares the
    # revision it was measured at.
    local exempt_list=""
    while IFS= read -r -d '' f; do
        scanned=$((scanned + 1))
        case "$f" in
            */evidence/*)
                if declares_a_revision "$root/${f#./}"; then
                    exempt=$((exempt + 1))
                    exempt_list="$exempt_list|$f|"
                fi
                ;;
        esac
    done < <(cd "$root" && find . \( -name target -o -name .git \) -prune -o \
        -type f \( -name '*.rs' -o -name '*.md' -o -name '*.sh' -o -name '*.py' \) \
        -print0 2>/dev/null)

    # One sweep for all four detectors. `-o` puts each citation on its own
    # `path:line:match` record, so a row carrying two is reported twice.
    #
    # ★ THE SWEEP RUNS FROM INSIDE `$root` AND GREPS `.`, so every record
    # begins with a relative path. An absolute one would start `D:/…` on this
    # platform, and splitting the record on its first colon would then yield
    # the drive letter as the file name and silently misparse every hit.
    local f n cite det
    while IFS= read -r rec; do
        f="${rec%%:*}";     rec="${rec#*:}"
        n="${rec%%:*}";     cite="${rec#*:}"
        case "$f" in
            */check-engine-citation.sh) continue ;;   # holds the patterns
        esac
        case "$exempt_list" in *"|$f|"*) continue ;; esac
        classify "$cite"; det="$DET"
        [ -z "$det" ] && continue
        hits=$((hits + 1))
        rc=1
        printf '  %s:%s  %s  [%s]\n' "${f#./}" "$n" "$cite" "$det" >&2
    done < <(cd "$root" && grep -rnoE \
        'ENGINE:`[^`]*\.rs:[0-9]+(-[0-9]+)?|[A-Za-z0-9_./\\:-]+\.rs:[0-9]+(-[0-9]+)?|`:[0-9]+(-[0-9]+)?`' \
        --include='*.rs' --include='*.md' --include='*.sh' --include='*.py' \
        --exclude-dir=target --exclude-dir=.git \
        . 2>/dev/null | sort)

    if [ "$rc" -ne 0 ]; then
        echo "engine-citation: FAIL — $hits citation(s) name a line in a tree this repository does not own." >&2
        echo >&2
        echo "  Replace each with the SYMBOL that owns the claim, having re-read" >&2
        echo "  that symbol in the engine first: the number is stale by" >&2
        echo "  construction, so it is a hint about which region the code USED" >&2
        echo "  to be in and nothing more. Two of the citations this gate was" >&2
        echo "  written for had come to name each other's type." >&2
        echo >&2
        echo "  [D1] the path itself names the engine or the archived GUI." >&2
        echo "  [D2] an ENGINE:-tagged citation carrying a line number." >&2
        echo "  [D3] a .rs line number above $SIZE_LIMIT — no file here is that long," >&2
        echo "       so it cannot be a file here." >&2
        echo "  [D4] a bare \`:NNNN\` continuing the citation before it, above the" >&2
        echo "       same cap." >&2
        echo >&2
        echo "  A vendor citation stays, but only VERSION-ANCHORED:" >&2
        echo "  egui-0.35.0/src/style.rs:1135, never style.rs:1135." >&2
        return 1
    fi
    echo "engine-citation: clean — $scanned file(s) scanned, $exempt evidence artifact(s) exempt by declared revision."
    return 0
}

# ---------------------------------------------------------------------------
# --self-test
# ---------------------------------------------------------------------------
self_test() {
    local tmp fails=0
    tmp="$(mktemp -d)" || { echo "self-test: mktemp failed" >&2; return 1; }
    trap 'rm -rf "$tmp"' RETURN

    expect() { # <name> <expected-rc> <root>
        local name="$1" want="$2" root="$3" got
        run_checks "$root" >/dev/null 2>&1 && got=0 || got=$?
        if [ "$got" -ne "$want" ]; then
            echo "self-test FAILED: $name — wanted rc=$want, got rc=$got" >&2
            fails=$((fails + 1))
        else
            echo "  ok  $name (rc=$got)"
        fi
    }

    # Case 0 — the real tree. The control. Reported, not asserted: if the
    # repository is currently dirty the self-test must not pretend otherwise.
    if run_checks "$ROOT" >/dev/null 2>&1; then
        echo "  ok  the real tree is clean"
    else
        echo "  --  the real tree currently FAILS (that is what the gate is for)"
    fi

    # A synthetic root holding every shape that must stay QUIET.
    local good="$tmp/good"
    mkdir -p "$good/evidence"
    printf 'fn main() {}\n' > "$good/small.rs"
    {
        printf '# quiet shapes\n'
        printf 'A version-anchored vendor citation: `egui-0.35.0/src/style.rs:1135`.\n'
        printf 'A local citation with a plausible number: `canvas/mapping.rs:912`.\n'
        printf 'A symbol citation with no number at all: `EditSession::edit_text`.\n'
        printf 'A plain engine file name, no number: `pdfcer-core/src/edit.rs`.\n'
        printf 'A local continuation citation: `boxes/mod.rs` (`:699-706`).\n'
    } > "$good/quiet.md"
    {
        printf '# snapshot\n\nMeasured against engine `c5a80c3b`.\n\n'
        printf 'ENGINE:`edit.rs:25971` and `pdfcer-core/src/edit.rs:41035`.\n'
    } > "$good/evidence/dated.md"
    expect "every allowed shape is left alone" 0 "$good"

    # D1 — an explicitly engine-owned path.
    local s1="$tmp/s1"; cp -r "$good" "$s1"
    printf 'The verb lives at `pdfcer-core/src/edit.rs:41035`.\n' > "$s1/d1.md"
    expect "D1 catches an engine-owned path" 1 "$s1"

    # D1 again, the archived GUI. The archive is IN scope.
    local s1b="$tmp/s1b"; cp -r "$good" "$s1b"
    printf 'Salvaged from `D:\\Dev\\pdfce\\crates\\pdfce-gui\\src\\main.rs:7031`.\n' > "$s1b/d1b.md" # old-name-exempt: the escaped spelling of the allowed archive path — `printf` needs `\\`, which is why the project-wide allow-list misses it
    expect "D1 catches the ARCHIVED GUI, which is not exempt" 1 "$s1b"

    # D2 — an ENGINE:-tagged citation with a line number, in a file with no
    # provenance. Note the number is SMALL, so only D2 can see it.
    local s2="$tmp/s2"; cp -r "$good" "$s2"
    printf '| Alignment | `/Q` | ENGINE:`forms.rs:375` |\n' > "$s2/d2.md"
    expect "D2 catches an ENGINE:-tagged line number" 1 "$s2"

    # D3 — a BARE citation whose number no file here could have. Neither D1
    # nor D2 can see this one, which is the case D3 exists for.
    local s3="$tmp/s3"; cp -r "$good" "$s3"
    printf 'It writes the key at `edit.rs:25026`.\n' > "$s3/d3.md"
    expect "D3 catches a bare citation with an impossible line number" 1 "$s3"

    # D4 — an orphan continuation citation. No `.rs` anywhere in it, so D1,
    # D2 and D3 are all blind to it by construction.
    local s3b="$tmp/s3b"; cp -r "$good" "$s3b"
    printf 'The verb is `with_tooltip`, writes `:25736`/`:25739`.\n' > "$s3b/d4.md"
    expect "D4 catches a bare continuation citation" 1 "$s3b"

    # The evidence exemption is CONDITIONAL — strip the revision and the same
    # file must fail.
    local s4="$tmp/s4"; cp -r "$good" "$s4"
    {
        printf '# snapshot\n\nMeasured recently.\n\n'
        printf 'ENGINE:`edit.rs:25971` and `pdfcer-core/src/edit.rs:41035`.\n'
    } > "$s4/evidence/dated.md"
    expect "an evidence artifact with NO declared revision is NOT exempt" 1 "$s4"

    # D3's premise, sabotaged. A tree carrying an over-limit .rs must SKIP,
    # not pass — a detector whose premise stopped holding must not print green.
    local s5="$tmp/s5"; cp -r "$good" "$s5"
    awk -v n=$((SIZE_LIMIT + 10)) 'BEGIN{ for (i=0;i<n;i++) print "// line" }' > "$s5/huge.rs"
    expect "an over-limit .rs file SKIPS rather than passing" 2 "$s5"

    if [ "$fails" -ne 0 ]; then
        echo "self-test: $fails case(s) failed" >&2
        return 1
    fi
    echo "self-test: all cases behaved"
    return 0
}

if [ "${1:-}" = "--self-test" ]; then
    self_test
    exit $?
fi

run_checks "$ROOT"
exit $?
