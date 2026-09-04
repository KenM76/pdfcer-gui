#!/usr/bin/env bash
# ===========================================================================
# check-engine-backlog.sh — EVERY CAPABILITY THE ENGINE SAYS IT HAS AND THIS
# SHELL DOES NOT OWES A WRITTEN VERDICT.
#
# ---------------------------------------------------------------------------
# ★★★ WHY THIS GATE EXISTS, and the day that bought it
# ---------------------------------------------------------------------------
#
# On 2026-09-03 the operator asked the ENGINE session for PNG / JPEG / SVG
# export and for copy-paste of vector graphics into Word and Inkscape. The
# engine shipped all of it that day, across four passes, and sent a note —
# "here is what a shell wires" — carrying the call for every capability, a
# clipboard format order validated against a real Word paste, and a worked
# example.
#
# **This shell built none of it and had no ticket for it.** The note landed in
# `D:\Dev\FeatureRequests\pdfce_FeatureRequests\open\` marked "informational,
# no reply needed; consume when convenient", and was found a day later only
# because a session read that folder looking for something else.
#
# The same morning two new engine verbs — `set_encryption` and
# `set_permissions` — arrived with NO note at all. Those were caught
# IMMEDIATELY, by `tools/gates/check-verb-coverage.sh`, because that gate reads
# the engine's API and fails when this shell names none of it.
#
# ⇒ ★★★ **A capability announced in an API has a gate. A capability announced
#   in prose has none.** That asymmetry is the defect this gate closes, and it
#   is the fifth time this project has recorded the same shape:
#
#     * `EDITABLE_SURFACES.md` §"The sweep found…" — three of the first four
#       gaps were capabilities the engine shipped BECAUSE this shell asked, and
#       then never consumed. "A reply arriving is not a capability landing."
#     * `check-string-gaps.sh` — a catalogued string that reaches no rectangle.
#     * `check-verb-coverage.sh` — a verb the engine implements that nothing
#       here names.
#     * O119's own row — "it is the fourth time: this landed with no note and
#       no announcement, and the only thing that made a noise was that gate."
#     * This.
#
# ---------------------------------------------------------------------------
# WHAT IT ASSERTS
# ---------------------------------------------------------------------------
#
# `D:\Dev\pdfcer\docs\FEATURES.md` is a table whose first four columns are
# `core | cli | gui | Acrobat`. A row reading `[x]` under `core` and `[ ]`
# under `gui` is the engine stating, in a machine-readable place, that it has
# something this shell does not.
#
#     For every such row, `ENGINE_BACKLOG.md` must carry an entry whose
#     first table cell opens with the same words the row opens with.
#
# That is the whole rule, and — exactly like `check-verb-coverage.sh` — it is
# deliberately weak in one direction and strong in the other:
#
#   * **Weak**: it does not judge the verdict. An entry saying `declined`
#     because somebody could not be bothered passes. This gate cannot read
#     English and must not pretend to.
#   * **Strong**: a capability that appears in the engine's table and is
#     discussed NOWHERE fails the build on the first `git pull` in the engine's
#     checkout that brings it. Somebody has to look at it and write a sentence
#     — which is the entire mechanism, and is exactly what did not happen on
#     2026-09-03.
#
# ★★ The failure is therefore not "you have a gap". It is **"the engine has
# said it has something you do not, and nobody here has said anything about
# it"**, which is a different and much more actionable statement.
#
# ---------------------------------------------------------------------------
# ★★★ THE ROW KEY, AND WHY IT SURVIVES REWORDING
# ---------------------------------------------------------------------------
#
# A gate that re-keys on every prose tweak teaches people to re-baseline it,
# and a gate people re-baseline is a gate that has stopped measuring. So the
# key is chosen against the way this particular file actually churns.
#
#   THE KEY IS THE ROW'S **OPENING CLAUSE**: the first six content words of
#   the Feature cell — markdown stripped, punctuation stripped, lower-cased,
#   a small fixed stopword list dropped — joined with hyphens.
#
#     "Split a document — `EveryN` only; no bookmark- or size-based criteria."
#       → split-document-everyn-only-no-bookmark
#
# Every part of that is a decision, and each one was measured against the real
# file rather than assumed:
#
# ★ **Why not the whole description.** The churn in `FEATURES.md` is ALL in
#   the tail. A row is written as a capability statement and then grows
#   measurements, Pass IDs, corrections, `Acrobat` comparisons and a closing
#   "**Not reachable in `pdfcer-gui`** — …" sentence that is rewritten every
#   time either project moves. The longest target row is **12,043 characters**;
#   a second is 8,708. Any key over the whole cell churns weekly, on rows whose
#   capability has not changed at all. The engine's own header even says the
#   tail is volatile: *"When a row changes, **replace** the sentence — never
#   append a note to it."*
#
# ★ **Why not the section heading.** Rows MOVE between sections. One of the
#   rows this gate reads records, in its own prose, being "moved here from
#   *Planned* this filing". A section-qualified key would go red on a filing
#   that changed nothing about the capability, which is the worst kind of
#   false positive: correct-looking, and about nothing.
#
# ★ **Why not the backticked API symbol.** Not every row has one — "Reflow
#   within a block, including justified alignment." names no symbol at all, and
#   "Set a page's size (`/MediaBox`)" names a PDF key rather than a verb. And
#   the symbols themselves get rewritten in bulk: `Pass 247.1` mechanically
#   renamed `pdfceGUI` → `pdfcer-gui` across the whole file in one filing.
#
# ★ **Why SIX words.** Measured on the 90 live rows: at four content words two
#   rows COLLIDE (`cut-copy-paste-whole`, once for PAGES and once for a
#   BOOKMARK SUBTREE). At five, all 90 are unique. Six is five plus one word of
#   margin against a future collision — and it stops there, because every extra
#   word is one more chance to churn. The gate FAILS on a collision rather than
#   silently accepting one entry for two rows.
#
# ★ **Why a stopword list at all.** So that "Split a document" and "Split the
#   document" are the same key. The list is fixed in this file (`STOPWORDS`
#   below) and **must not be edited casually**: changing it re-keys all 90 rows
#   at once, and a re-key is indistinguishable from 90 new capabilities.
#
# ⇒ **The key is not claimed to be churn-proof.** A row whose opening clause is
# genuinely rewritten WILL go red, and that is correct — a capability that got
# a new name is a thing a person should look at. What matters is that the
# failure message tells you which of the two you are looking at, because
# **a reworded row and a new row are identical to a key and are opposite
# acts.** So the gate prints, for every unaccounted row, any register entry
# sharing its first three words. That is the reworded case, named.
#
# ---------------------------------------------------------------------------
# ★★ PARSING, DEFENSIVELY — the table is not as simple as it looks
# ---------------------------------------------------------------------------
#
#   * Five columns, of which four are the checkbox columns; the fifth is prose.
#   * The prose contains `|` — always escaped as `\|`, inside backticks
#     (`edit-text --target auto\|page\|form:N`). Splitting naively on `|`
#     **truncates the cell at the first escaped pipe**, so `\|` is swapped for
#     a sentinel byte before the split and back after it.
#
#     ★ Being precise about the hazard, because the first draft of this comment
#     overstated it and the self-test then asserted the wrong thing: an escaped
#     pipe in a row's PROSE cannot shift the four checkbox columns, because it
#     is always *after* them — the row is still measured and, if the pipe falls
#     past the sixth content word, still keyed identically. **The hazard is a
#     pipe INSIDE the first six content words**, of a feature row or of a
#     register label, where a naive split truncates the key and turns a
#     correctly-filed row into a false failure. The self-test plants exactly
#     that shape, on an ACCOUNTED row, and asserts it is not reported.
#   * The checkbox columns carry `[x]`, `[ ]`, `—`, `◐` and (in *Planned*) `?`.
#     Only `[x]` in `core` AND `[ ]` in `gui` is a target; every other
#     combination — including `◐` in `gui`, which is a partial and NOT a gap —
#     is left alone. Confusing `[ ]` with `—` is the mistake the engine's own
#     legend exists to prevent, and this gate does not make it.
#   * Header (`| core | cli | gui | …`) and separator (`|:----:|…`) rows fall
#     out for free: neither has `[x]` in its first column.
#   * The whole file is read, not just *Implemented* — three target rows live
#     under *Planned*, and they are the export rows O120 is about.
#
# ---------------------------------------------------------------------------
# WHERE THE ENGINE IS, AND WHY IT IS NOT A LITERAL
# ---------------------------------------------------------------------------
#
# Derived from `crates/pdfcer-gui/Cargo.toml`'s `git = "file:///…"` URL — the
# same answer Cargo itself resolved, and the same rule `tools/engine_path.py`
# was written to enforce after a hard-coded path survived a rename and made a
# gate report `PASS` having examined nothing. **A check that cannot fail is not
# evidence**, so every unreadable-input case below SKIPs loudly instead.
#
# ★ It reads the engine's **working tree**, not the revision `Cargo.lock` pins
# — deliberately, and the opposite choice to `verb-coverage.py`'s. That tool
# asks "could this shell CALL this?", where the lock is the only honest answer.
# This one asks "has the engine SAID it has something we do not?", and a
# statement in a document is made when it is written, not when it is pinned.
#
# ---------------------------------------------------------------------------
# USAGE
# ---------------------------------------------------------------------------
#   tools/gates/check-engine-backlog.sh              measure
#   tools/gates/check-engine-backlog.sh --self-test  prove it can fail
#
# EXIT CODES
#   0  every `[x] core / [ ] gui` row is accounted for in `ENGINE_BACKLOG.md`
#      (or an input was unreadable, which SKIPs — see below)
#   1  at least one row is accounted for nowhere, or a key collided, or the
#      self-test did not detect its plant
#
# ★ SKIPs rather than fails when the engine checkout, the manifest or the
# register is unreadable, and says so loudly. The word SKIP in the output is
# the signal; `run-all.sh` counts them separately from passes.
#
# ★★ NOT YET REGISTERED in `run-all.sh` — another track owns that file as this
# is written, and it is registered at reconciliation. Until then: run it
# standalone. A gate nobody runs is a gate that does not exist, which is the
# whole lesson above.
# ===========================================================================
set -uo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
cd "$ROOT" || exit 1

REGISTER="${PDFCER_ENGINE_BACKLOG:-ENGINE_BACKLOG.md}"
MANIFEST="crates/pdfcer-gui/Cargo.toml"

# How many content words make a key. See "★ Why SIX words" above. Changing
# this re-keys every row at once.
KEYWORDS=6

# The fixed stopword list. Changing it re-keys every row at once; it is here
# rather than in a data file so that a change to it shows up in this gate's own
# diff, next to the paragraph that explains why it must not move.
STOPWORDS="a an the of to in and or its it is for that this with from on at as by be are was"

# ---------------------------------------------------------------------------
# The awk program: parse a pipe table, emit one key per line.
#
# MODE=features  → keys of rows with [x] in column 1 and [ ] in column 3
# MODE=register  → keys of the first cell of every row in a verdict section
# ---------------------------------------------------------------------------
AWKPROG="$(mktemp)"
trap 'rm -f "$AWKPROG" "${SLUGS_F:-}" "${SLUGS_R:-}" "${MISSING:-}"' EXIT

cat > "$AWKPROG" <<'AWK'
BEGIN {
    n = split(STOPWORDS, sw, " ")
    for (i = 1; i <= n; i++) STOP[sw[i]] = 1
    inverdict = (MODE == "features") ? 1 : 0
}

# `\001` stands in for an escaped pipe while the row is split. It cannot occur
# in a markdown document; if it somehow did, the row would key differently and
# be REPORTED, never silently dropped — the safe direction.
function key(s,   t, m, i, w, out, cnt) {
    gsub(/\001/, " ", s)
    gsub(/`/, " ", s)
    gsub(/\*/, " ", s)
    gsub(/\\/, " ", s)
    s = tolower(s)
    gsub(/[^a-z0-9]+/, " ", s)
    m = split(s, w, " ")
    cnt = 0; out = ""
    for (i = 1; i <= m; i++) {
        if (w[i] == "") continue
        if (w[i] in STOP) continue
        out = (cnt == 0) ? w[i] : out "-" w[i]
        cnt++
        if (cnt == KEYWORDS) break
    }
    return out
}

function trim(s) { sub(/^[ \t]+/, "", s); sub(/[ \t]+$/, "", s); return s }

{
    line = $0
    sub(/\r$/, "", line)
}

# In the register, only tables under a `## `verdict`` heading are entries. The
# header's own explanatory tables are prose about the file, not entries in it,
# and keying on them would manufacture orphans on every edit to the header.
MODE == "register" && line ~ /^## `(wanted|declined|blocked|unknown|shipped)`/ { inverdict = 1; next }
MODE == "register" && line ~ /^## / { inverdict = 0; next }

substr(line, 1, 1) != "|" { next }
!inverdict { next }

{
    tmp = line
    gsub(/\\\|/, "\001", tmp)
    n = split(tmp, f, "|")
}

MODE == "features" {
    if (n < 6) next
    core = trim(f[2]); gui = trim(f[4])
    if (core != "[x]") next
    if (gui != "[ ]") next
    feat = f[6]
    for (i = 7; i <= n; i++) feat = feat "|" f[i]
    k = key(feat)
    if (k == "") next
    print k
    next
}

MODE == "register" {
    if (n < 4) next
    label = trim(f[2])
    if (label == "") next
    if (label ~ /^:?-+:?$/) next          # the |---|---| separator
    if (label ~ /^Row \(/) next           # each table's own header row
    k = key(label)
    if (k == "") next
    print k
}
AWK

parse() { # parse <mode> <file>
    awk -v MODE="$1" -v KEYWORDS="$KEYWORDS" -v STOPWORDS="$STOPWORDS" \
        -f "$AWKPROG" "$2" 2>/dev/null | tr -d '\r'
}

# ---------------------------------------------------------------------------
# Locate the engine, the way `tools/engine_path.py` does: from the manifest
# Cargo builds from, never from a literal. Comment lines are skipped — the
# shim's own explanation quotes URLs, and an instrument that read a comment
# would be following documentation instead of the build.
# ---------------------------------------------------------------------------
locate_engine() {
    [ -f "$MANIFEST" ] || return 1
    grep -v '^[[:space:]]*#' "$MANIFEST" \
        | sed -n 's|.*git[[:space:]]*=[[:space:]]*"file:///\([^"]*\)".*|\1|p' \
        | head -1
}

# ---------------------------------------------------------------------------
# report — the whole comparison, so the self-test drives the same code path the
# real run does. Prints its findings; returns 0 clean, 1 unaccounted, 2 could
# not measure.
# ---------------------------------------------------------------------------
report() { # report <features.md> <register.md>
    local features="$1" register="$2"

    SLUGS_F="$(mktemp)"; SLUGS_R="$(mktemp)"; MISSING="$(mktemp)"
    parse features "$features" | sed '/^$/d' > "$SLUGS_F"
    parse register "$register" | sed '/^$/d' > "$SLUGS_R"

    local total dupes
    total=$(wc -l < "$SLUGS_F" | tr -d ' ')
    if [ "$total" -eq 0 ]; then
        echo "SKIP: no \`[x] core / [ ] gui\` rows were parsed out of $features."
        echo "      Either the table's shape changed or the file is not the one"
        echo "      this gate thinks it is. A gate that passes without measuring"
        echo "      is not a gate, so this is a SKIP and not a PASS."
        return 2
    fi

    echo "measured $total row(s) reading [x] core / [ ] gui in $features"
    echo "         against $(wc -l < "$SLUGS_R" | tr -d ' ') entr(ies) in $register"

    # A collision means two capabilities share one key, so one written verdict
    # would silently discharge both. Fail rather than accept it.
    dupes="$(sort "$SLUGS_F" | uniq -d)"
    if [ -n "$dupes" ]; then
        echo
        echo "FAIL: two rows share one key, so one written verdict would silently"
        echo "      account for both:"
        printf '%s\n' "$dupes" | sed 's/^/        /'
        echo
        echo "  Raise KEYWORDS in this gate by one and re-run. The header's"
        echo "  \"★ Why SIX words\" paragraph is the measurement that chose it and"
        echo "  should be updated with the new one."
        return 1
    fi

    comm -23 <(sort -u "$SLUGS_F") <(sort -u "$SLUGS_R") > "$MISSING"

    # An entry matching no row is NOT a failure. `ENGINE_BACKLOG.md` keeps a
    # row after its `gui` box is ticked, deliberately and for the reason
    # `EDITABLE_SURFACES.md` gives: the argument is the valuable part. But it
    # is worth SAYING, because the other cause is a reworded row.
    local orphans
    orphans="$(comm -13 <(sort -u "$SLUGS_F") <(sort -u "$SLUGS_R"))"
    if [ -n "$orphans" ]; then
        local n_orph
        n_orph=$(printf '%s\n' "$orphans" | wc -l | tr -d ' ')
        echo "note: $n_orph register entr(ies) match no current row — either a row whose"
        echo "      \`gui\` box got ticked (keep it, the argument is the valuable part) or"
        echo "      one whose opening words were reworded."
    fi

    [ -s "$MISSING" ] || return 0

    echo
    echo "FAIL: the engine's own feature table says it has capabilities this shell"
    echo "      does not, and $register does not mention them either:"
    echo
    while IFS= read -r k; do
        [ -z "$k" ] && continue
        printf '        %s\n' "$k"
        # The reworded case, named. Three leading words is enough to catch a
        # rewrite of the tail of the clause and tight enough not to fire on
        # every row starting "export page s".
        local pre near
        pre="$(printf '%s\n' "$k" | cut -d- -f1-3)"
        near="$(grep -F -- "$pre" "$SLUGS_R" | head -3)"
        if [ -n "$near" ]; then
            # ★ This exact string is what the self-test asserts on. Do not
            # reword it to match the prose below, and do not let the prose
            # below reproduce it — an assertion that can be satisfied by the
            # explanation of the assertion is not an assertion. It was, once,
            # for about ten minutes, and it passed against a gate that printed
            # no near-miss at all.
            printf '          -> similar entr(ies) already in %s:\n' "$register"
            printf '%s\n' "$near" | sed 's/^/             /'
        fi
    done < "$MISSING"

    cat <<'EOF'

  A key in this list is one of THREE things, and they are opposite acts:

    1. **A row that was REWORDED.** Where a near-miss was printed under the
       key above, read it FIRST: the capability very likely already has a
       verdict and an argument in ENGINE_BACKLOG.md, filed under its old
       opening words. FIX: update that entry's first cell to the row's new
       opening words. Do NOT add a second entry, and do NOT delete the old
       reasoning — the argument is the part worth keeping. (A near-miss can
       also be an innocent sibling: four rows all open "Export page(s) to …".
       Read the entry before you edit it.)

    2. **A capability that landed and nobody noticed.** That is what this gate
       is for. On 2026-09-03 the operator asked the ENGINE for PNG/JPEG/SVG
       export; it shipped the same day, sent a note saying "here is what a
       shell wires", and this shell built none of it and had no row. It was
       found a day later by accident. Go and read the channel at
       `D:\Dev\FeatureRequests\pdfce_FeatureRequests\open\`, then write the
       row — `wanted`, with what it would take.

    3. **A capability this shell genuinely should not have.** Fine — say so, in
       ENGINE_BACKLOG.md, with the argument: `declined`, or `blocked` naming
       what it waits on. A row nobody has an opinion about yet is `unknown`,
       which is an honest verdict and a better one than a guessed `declined`.

  What is NOT allowed is silence, because silence is indistinguishable from
  (2) and reads as (3).
EOF
    return 1
}

# ═══════════════════════════════════════════════════════════════════════════
# ★★ SELF-TEST — a gate that has never been seen to fail is a rumour
# ═══════════════════════════════════════════════════════════════════════════
#
# Both halves are asserted, because only the pair is evidence:
#
#   * it CATCHES a planted row that nothing accounts for, and it catches a
#     REWORDED row and names the entry it is probably a rewording of;
#   * it PASSES the four shapes that are correct — an accounted row, a row
#     whose `gui` box is already ticked, a row the engine does not have either,
#     and a `—` shape mismatch — because a gate that reports the correct shape
#     trains people to ignore it, which is worse than not having the gate.
#
# ★ And it plants the `\|`-inside-backticks shape on an ACCOUNTED row, then
# asserts the measured TOTAL. That is the one failure a "did it report?" check
# cannot see: a parser that split on the escaped pipe would shift the columns,
# drop the row from the measurement entirely, and report a clean PASS — silent
# blindness, which is the failure mode this project has paid for more than once.
# ═══════════════════════════════════════════════════════════════════════════
if [ "${1:-}" = "--self-test" ]; then
    tmp="$(mktemp -d)"
    trap 'rm -rf "$tmp"; rm -f "$AWKPROG" "${SLUGS_F:-}" "${SLUGS_R:-}" "${MISSING:-}"' EXIT

    cat > "$tmp/FEATURES.md" <<'MD'
## Implemented

### Document & pages

| core | cli | gui | Acrobat | Feature |
|:----:|:---:|:---:|:-------:|---------|
| [x] | [x] | [ ] | [x] | Merge several files into one — three verbs share this neighbourhood. |
| [x] | [x] | [ ] | [x] | Sharpen the reticulating splines, which nobody has ever discussed. |
| [x] | [x] | [ ] | [x] | Split a document into sections — bookmark-based criteria at last. |
| [x] | [x] | [x] | [x] | Rotate, delete, reorder and extract pages, all reachable already. |
| [ ] | [ ] | [ ] | [x] | Publish the drawing to a fax machine, which the engine also lacks. |
| — | — | [ ] | [x] | Several documents open at once; a shape mismatch, not a gap. |
| [x] | [x] | [ ] | ◐ | `auto\|warn\|refuse` when bold needs a fallback — a posture, not a gate. |
| [x] | [x] | ◐ | [x] | Type 3 text extracts and searches, partially reachable, not a target. |
MD

    cat > "$tmp/ENGINE_BACKLOG.md" <<'MD'
# ENGINE_BACKLOG.md

| verdict | means |
|---|---|
| `wanted` | a prose table in the header, which must be ignored |

## `wanted` — 1

| Row (`FEATURES.md`, wanted) | Why |
|---|---|
| Merge several files into one — three verbs share … | Wanted: the wiring is missing. |

## `shipped` — 2

| Row (`FEATURES.md`, shipped) | Why |
|---|---|
| Split a document into parts — `EveryN` only; no bookmark-based … | The opening words this row USED to have. |
| `auto\|warn\|refuse` when bold needs a fallback … | Reachable; the escaped pipes are inside this label's first six words. |
MD

    out="$(report "$tmp/FEATURES.md" "$tmp/ENGINE_BACKLOG.md" 2>&1)"
    rc=$?
    fail=0

    if [ "$rc" -ne 1 ]; then
        echo "engine-backlog --self-test: FAIL — the planted rows were not detected (rc=$rc)."
        fail=1
    fi
    if ! printf '%s' "$out" | grep -q 'sharpen-reticulating-splines'; then
        echo "engine-backlog --self-test: FAIL — an unaccounted row was not reported."
        fail=1
    fi
    if ! printf '%s' "$out" | grep -q 'split-document-into-sections-bookmark-based'; then
        echo "engine-backlog --self-test: FAIL — a REWORDED row was not reported."
        fail=1
    fi
    # ★ Asserted on the MARKER LINE, not on the word "similar". The first cut
    # of this check grepped for `similar entr`, which the FAIL message's own
    # explanatory prose also contains — so it passed against a gate that
    # printed no near-miss whatsoever. An assertion satisfiable by the
    # explanation of the assertion is not an assertion.
    if ! printf '%s' "$out" | grep -qF -- '-> similar entr(ies) already in'; then
        echo "engine-backlog --self-test: FAIL — the reworded row was reported with no"
        echo "  near-miss, so the message cannot tell a rewrite from a new capability."
        echo "  That distinction is the whole reason the key scheme is documented."
        fail=1
    fi
    if ! printf '%s' "$out" | grep -q 'split-document-into-parts-everyn'; then
        echo "engine-backlog --self-test: FAIL — the near-miss did not name the entry"
        echo "  the reworded row was filed under, which is the only thing that makes"
        echo "  the hint actionable."
        fail=1
    fi
    # ★ `grep -A1`, not a two-line `grep -F` pattern: `-F` treats each line of
    # the pattern as a SEPARATE alternative, so a two-line pattern matches when
    # either half appears anywhere. That is how this assertion first passed
    # against exactly the behaviour it was written to forbid.
    if printf '%s\n' "$out" | grep -A1 'sharpen-reticulating-splines' \
        | grep -qF -- '-> similar entr(ies) already in'; then
        echo "engine-backlog --self-test: FAIL — a genuinely new capability was given a"
        echo "  near-miss it does not have, which would send somebody to edit an"
        echo "  unrelated entry instead of writing a row."
        fail=1
    fi
    if ! printf '%s' "$out" | grep -q 'measured 4 row'; then
        echo "engine-backlog --self-test: FAIL — expected exactly 4 target rows."
        echo "  Fewer means rows are being DROPPED — a cell mis-split, a shape the"
        echo "  parser stopped recognising — which is silent blindness and the worst"
        echo "  available outcome for a gate. More means the column test has gone"
        echo "  slack: \`—\`, \`◐\`, \`?\` and an already-ticked \`[x]\` in \`gui\` are all"
        echo "  NOT targets, and the engine's own legend says confusing \`[ ]\` with"
        echo "  \`—\` is the mistake it exists to prevent. Measured line was:"
        printf '%s\n' "$out" | grep '^measured' | sed 's/^/    /'
        fail=1
    fi
    for ok in rotate-delete-reorder publish-drawing several-documents-open type-3-text; do
        if printf '%s' "$out" | grep -q "$ok"; then
            echo "engine-backlog --self-test: FAIL — a correct shape ($ok) was reported."
            echo "  A gate that reports the correct shape trains people to ignore it,"
            echo "  which is worse than not having the gate."
            fail=1
        fi
    done
    # ★ The escaped-pipe row is ACCOUNTED for, and its pipes sit inside its
    # first six content words. A naive split truncates the key on BOTH sides —
    # but not identically, because the feature cell and the register label are
    # different strings — and the row is reported as a false failure. This is
    # the assertion that catches losing the sentinel swap.
    if printf '%s' "$out" | grep -q 'auto-warn-refuse'; then
        echo "engine-backlog --self-test: FAIL — an accounted row whose label carries an"
        echo "  escaped pipe inside its first six words was reported. The \`\\|\` → sentinel"
        echo "  swap is what stops a cell being truncated at the pipe; without it every"
        echo "  such row is a false failure."
        fail=1
    fi
    # ★ The register's own header carries explanatory tables. They are prose
    # ABOUT the file, not entries IN it, and reading them manufactures orphans
    # on every edit to the header. Asserted through the orphan COUNT, because
    # an orphan is reported as a number and never by name — which is exactly
    # how the first version of this check (grepping for a header key by name)
    # passed against a gate that read the whole file.
    if ! printf '%s' "$out" | grep -qF 'note: 1 register entr(ies)'; then
        echo "engine-backlog --self-test: FAIL — expected exactly 1 orphan entry (the"
        echo "  reworded row's old filing). A different count means the register parse"
        echo "  is reading rows it should not — most likely the header's own tables,"
        echo "  which the \`## \`verdict\`\` section gating exists to exclude. Note line was:"
        printf '%s\n' "$out" | grep '^note:' | sed 's/^/    /'
        fail=1
    fi

    [ "$fail" -ne 0 ] && exit 1
    echo "engine-backlog --self-test: PASS — catches an unaccounted row and a reworded"
    echo "  one (naming the likely rewrite), measures all 4 targets including the"
    echo "  escaped-pipe row, and reports none of the four correct shapes."
    exit 0
fi

# ═══════════════════════════════════════════════════════════════════════════
# The real run
# ═══════════════════════════════════════════════════════════════════════════
if [ ! -f "$REGISTER" ]; then
    echo "SKIP: $REGISTER is missing, so there is nothing to check verdicts against."
    echo "      That file is the register this gate enforces; without it the gate"
    echo "      has no opinion to compare the engine's table to."
    exit 0
fi

ENGINE="$(locate_engine)"
if [ -z "${ENGINE:-}" ]; then
    echo "SKIP: $MANIFEST names no \`git = \"file:///…\"\` dependency, so this gate"
    echo "      cannot say where the engine is. It refuses to guess: a hard-coded"
    echo "      path is what made check-verb-coverage report PASS having examined"
    echo "      nothing, on the day this project renamed before the engine did."
    exit 0
fi

FEATURES="$ENGINE/docs/FEATURES.md"
if [ ! -f "$FEATURES" ]; then
    echo "SKIP: $FEATURES is unreadable, so nothing was measured."
    echo "      The engine checkout was located at $ENGINE."
    exit 0
fi

if ! command -v awk >/dev/null 2>&1; then
    echo "SKIP: awk is absent. This gate is a table parser and has no other way to"
    echo "      read the engine's feature table."
    exit 0
fi

report "$FEATURES" "$REGISTER"
rc=$?
case "$rc" in
    0) echo; echo "PASS: every \`[x] core / [ ] gui\` row is accounted for in $REGISTER." ;;
    2) exit 0 ;;
esac
exit "$rc"
