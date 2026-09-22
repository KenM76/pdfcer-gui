#!/usr/bin/env bash
# check-conventions.sh — every interactive surface has answered, row by row,
# the conventions its gesture class carries.
#
# ---------------------------------------------------------------------------
# THE PROPERTY ASSERTED
# ---------------------------------------------------------------------------
#
# Every file listed in `tools/gates/conventions.list` carries a
# `conventions: <class>` block that answers, by id, every rule its gesture
# class defines — and a `WAIVED` answer carries a reason on the same line.
#
# The corpus is `D:/dev/rag/ui-conventions/`, overridable with
# `UI_CONVENTIONS_DIR`: one file per gesture class, each a numbered list of
# the rules every program in the class already follows. The registry says
# which surface answers to which class and carries the full argument for why
# this exists.
#
# ---------------------------------------------------------------------------
# WHY A HUMAN CANNOT HOLD IT
# ---------------------------------------------------------------------------
#
# A convention is invisible precisely because every other program honours it.
# Nobody notices the rule; they notice its absence, and only while using the
# surface. A reviewer reading a diff has no prompt to ask "does this drag
# preview at 1:1, and does Escape cancel it?", and the author had no prompt
# either — which is why every convention the operator has had to report was
# one nobody had asked about, not one somebody decided against. The corpus
# also grows: a class gains a row and every surface already written to that
# class is silently one answer short. Deriving the row list from the corpus on
# every run is what turns that from a memory problem into a red build.
#
# ---------------------------------------------------------------------------
# WHAT IT PROVABLY CANNOT SEE
# ---------------------------------------------------------------------------
#
#   * BEHAVIOUR, entirely. No grep can tell whether a preview tracks the
#     pointer at 1:1, and pretending otherwise would be worse than useless — a
#     green gate asserting something it never measured is the exact failure
#     this project exists to remove. What it CAN check is that somebody
#     consciously answered the question for each row, and it makes an
#     unanswered one visible. That is the whole of the value, and it is
#     enough. This gate is the floor, not the ceiling: rows that can be
#     verified get a driven check or a unit test as well.
#   * Whether an answer is TRUE. `D1 live-preview: yes` passes whatever the
#     code does.
#   * Whether a waiver's reason is a good one. It checks only that the word
#     `WAIVED` is not the end of the line.
#   * A surface missing from the registry, or one given the wrong class. The
#     registry is hand-maintained, and nothing here can tell that a new panel
#     should have been added to it.
#
# ---------------------------------------------------------------------------
# THE FORM OF AN ANSWER
# ---------------------------------------------------------------------------
#
#     // conventions: drag-moves
#     // - D1 live-preview: <how>
#     // - D3 escape-cancels: WAIVED — <why>
#
# A row is answered when its id appears after `- ` in a comment in the file.
# The row ids of a class are read from its corpus file's `## <ID>.` headings,
# with any leading ★ decoration stripped, so a row added to the corpus is
# demanded of every surface in that class without anybody remembering to.
#
# ---------------------------------------------------------------------------
# THE EXIT CONTRACT, AND HOW TO FALSIFY IT
# ---------------------------------------------------------------------------
#
#   0  every registered surface has answered every row of its class
#   1  a registered file is not on disk, names a class with no corpus file,
#      names a class whose corpus file yields no rows, leaves a row
#      unanswered, or waives one with no reason
#   2  the corpus directory is unreadable, the registry is missing, or the
#      registry lists no surfaces. NOT a pass: with no oracle — or with
#      nothing to hold to it — this gate has no opinion at all, and
#      `run-all.sh` prints skips in their own block and exits 3.
#
# ★ "The corpus file yields no rows" is its OWN outcome, and it used to be
# reported as *"names class '<c>' and <file> does not exist"* — a sentence
# that is false, about a file sitting right there. `rows_for` returned
# non-zero for both states because `pipefail` carries the inner `grep`'s
# failure out of the pipeline, and the caller had one message for the whole
# non-zero case. Whoever hit it would have gone looking for a missing file.
# The two states want opposite actions: write the corpus file, or find out
# why its headings stopped parsing.
#
# To falsify: `bash tools/gates/check-conventions.sh --self-test` — arms
# against a synthetic corpus and registry, needing neither. Two of them
# read the MESSAGE rather than the exit code, because the two states above
# share an exit code and it was the sentence that was wrong.

set -uo pipefail

if [ "${1:-}" = "--self-test" ]; then
    SELF="$(cd "$(dirname "$0")" && pwd)/$(basename "$0")"
    TD="$(mktemp -d)"
    trap 'rm -rf "$TD"' EXIT
    fails=0
    arms=0

    # A one-class corpus. `## ★★ D1. …` must yield `D1`, so the decoration is
    # carried in the fixture rather than assumed away.
    mkdir -p "$TD/corpus"
    {
        echo '# drag-moves'
        echo
        echo '## Overview'
        echo 'Prose that names no row, which must not become one.'
        echo
        echo '## ★★ D1. The preview tracks the pointer at 1:1.'
        echo
        echo '## D2. Escape cancels the gesture.'
    } > "$TD/corpus/drag-moves.md"

    # A surface that answers both rows. Written once and copied, so every
    # planted arm differs from the control by exactly the plant.
    {
        echo '// conventions: drag-moves'
        echo '// - D1 live-preview: the outline follows the pointer.'
        echo '// - D2 escape-cancels: the gesture machine drops the drag.'
        echo 'fn main() {}'
    } > "$TD/good.rs"

    mkreg() {   # mkreg <name> <line>...
        local f="$TD/$1.list"; shift
        : > "$f"
        local l
        for l in "$@"; do echo "$l" >> "$f"; done
    }

    arm() {   # arm <label> <expected-rc> <registry> <corpus> [must-match] [must-not-match]
        local got=0 out="$TD/out.txt"
        arms=$((arms + 1))
        UI_CONVENTIONS_DIR="$4" CONVENTIONS_REGISTRY="$3" \
            bash "$SELF" >"$out" 2>&1 || got=$?
        local why=""
        [ "$got" -ne "$2" ] && why="rc=$got, expected $2"
        if [ -z "$why" ] && [ -n "${5:-}" ] && ! grep -qE "$5" "$out"; then
            why="output never said /$5/"
        fi
        if [ -z "$why" ] && [ -n "${6:-}" ] && grep -qE "$6" "$out"; then
            why="output wrongly said /$6/"
        fi
        if [ -z "$why" ]; then
            printf '  ok    %-44s rc=%d\n' "$1" "$got"
        else
            printf '  FAIL  %-44s %s\n' "$1" "$why"
            fails=$((fails + 1))
        fi
    }

    # 1. the unplanted control.
    mkreg clean "# a comment line, which must not count as a surface" "$TD/good.rs drag-moves"
    arm "a surface answering every row" 0 "$TD/clean.list" "$TD/corpus"

    # 2. the falsification the header has always named.
    grep -v '\- D2' "$TD/good.rs" > "$TD/unanswered.rs"
    mkreg unanswered "$TD/unanswered.rs drag-moves"
    arm "a row left unanswered" 1 "$TD/unanswered.list" "$TD/corpus" 'unanswered.*D2'

    # 3. ★ the gate's central claim, and it had never been made to happen: a
    #    row ADDED to the corpus must redden a surface that was green, with no
    #    edit to the surface and nobody remembering to go looking.
    cp -r "$TD/corpus" "$TD/corpus-grown"
    echo '## D3. The drop point snaps to the nearest guide.' >> "$TD/corpus-grown/drag-moves.md"
    arm "a row added to the corpus reddens a green file" 1 "$TD/clean.list" "$TD/corpus-grown" 'unanswered.*D3'

    # 4. the block itself missing, which is a different message from a row
    #    missing — the surface never joined the class at all.
    grep -v 'conventions: drag-moves' "$TD/good.rs" > "$TD/noblock.rs"
    mkreg noblock "$TD/noblock.rs drag-moves"
    arm "no conventions block at all" 1 "$TD/noblock.list" "$TD/corpus" 'no .conventions: drag-moves. block'

    # 5. a registry naming a file nobody has written yet.
    mkreg ghost "$TD/nosuchfile.rs drag-moves"
    arm "a registered file not on disk" 1 "$TD/ghost.list" "$TD/corpus" 'not on disk'

    # 6/7. ★ the two states that used to share one — and one sentence, which
    #    was false for the second. They differ only in whether the file is
    #    there, so the arms assert on the MESSAGE.
    mkreg noclass "$TD/good.rs no-such-class"
    arm "a class with no corpus file" 1 "$TD/noclass.list" "$TD/corpus" 'does not exist'

    cp -r "$TD/corpus" "$TD/corpus-blind"
    sed -i 's/^## /### /' "$TD/corpus-blind/drag-moves.md"
    arm "a corpus file that yields no rows" 1 "$TD/clean.list" "$TD/corpus-blind" 'no rows' 'does not exist'

    # 8/9. an exemption without its argument.
    sed 's/- D2 escape-cancels: .*/- D2 escape-cancels: WAIVED/' "$TD/good.rs" > "$TD/bare.rs"
    mkreg bare "$TD/bare.rs drag-moves"
    arm "a waiver ending at the word WAIVED" 1 "$TD/bare.list" "$TD/corpus" 'waived with no reason'

    sed 's/- D2 escape-cancels: .*/- D2 escape-cancels: WAIVED — the gesture machine owns Escape./' \
        "$TD/good.rs" > "$TD/waived.rs"
    mkreg waived "$TD/waived.rs drag-moves"
    arm "a waiver carrying a reason" 0 "$TD/waived.list" "$TD/corpus"

    # 10/11/12. the three ways this gate can hold nothing to nothing. All
    #    exit 2: the runner classifies on the code alone, and a gate with no
    #    oracle has no opinion rather than a good one.
    mkdir -p "$TD/empty-corpus"
    arm "an empty corpus directory" 2 "$TD/clean.list" "$TD/empty-corpus"
    arm "no corpus directory at all" 2 "$TD/clean.list" "$TD/nosuchdir"

    mkreg onlycomments "# every line here is a comment" "" "# and a blank one above"
    arm "a registry listing no surfaces" 2 "$TD/onlycomments.list" "$TD/corpus" 'lists no surface'
    # ★ Both of these exit 2, and with the missing-registry guard removed this
    #   one still did — the loop reads nothing, `surfaces` stays 0, and the
    #   zero-surface guard exits 2 in its place, reporting that a file which is
    #   not there "exists and lists no surface". Two guards, one exit code, and
    #   an arm that asserted only the code could not tell which had fired. It
    #   asserts the sentence.
    arm "no registry at all" 2 "$TD/nosuch.list" "$TD/corpus" 'registry is not at' 'lists no surface'

    if [ "$fails" -ne 0 ]; then
        echo "check-conventions --self-test: FAIL — $fails arm(s) disagreed."
        exit 1
    fi
    echo "check-conventions --self-test: PASS — $arms arms, including the claim this"
    echo "  gate is FOR (a row added to the corpus reddens an untouched file), the"
    echo "  two states that used to share one false sentence, a waiver both ways,"
    echo "  and the three ways it can hold nothing to nothing, all exiting 2."
    exit 0
fi

cd "$(dirname "$0")/../.." || exit 2

CORPUS="${UI_CONVENTIONS_DIR:-D:/dev/rag/ui-conventions}"
REGISTRY="${CONVENTIONS_REGISTRY:-tools/gates/conventions.list}"

echo "check-conventions: every interactive surface answers its gesture class…"

if [ ! -d "$CORPUS" ]; then
    echo "  the conventions corpus is not at $CORPUS."
    echo "  Set UI_CONVENTIONS_DIR, or clone it. This gate has no oracle without it,"
    echo "  and reporting PASS with no oracle is the failure it exists to prevent."
    exit 2
fi

# A corpus directory holding no class file is the same state as no corpus
# directory, and must not be reported as every registered surface naming a
# class that does not exist — that sentence sends the reader to sixteen
# source files when the answer is that the corpus was never cloned.
if [ -z "$(find "$CORPUS" -maxdepth 1 -name '*.md' -print -quit 2>/dev/null)" ]; then
    echo "  $CORPUS exists and holds no class file."
    echo "  There is no oracle here, so this gate has no opinion about any surface."
    echo "  Exiting 2, not 0 and not 1: nothing was compared."
    exit 2
fi

if [ ! -f "$REGISTRY" ]; then
    echo "  the registry is not at $REGISTRY."
    echo "  It is the list of surfaces this gate holds to the corpus. Without it the"
    echo "  loop below reads nothing, finds no violations, and prints PASS over zero"
    echo "  files. Exiting 2, not 0."
    exit 2
fi

# Rule ids for a class: every `## <ID>.` heading in its corpus file, with any
# leading ★ decoration stripped. `## ★★ D1. …` yields `D1`.
#
# Three outcomes, because there are three states and two of them used to share
# a sentence: 0 with the rows on stdout, 1 for no corpus file at all, 3 for a
# corpus file that exists and yields nothing. `pipefail` turns the inner
# `grep`'s "no match" into a non-zero pipeline, which is why the second state
# was indistinguishable from the first.
rows_for() {
    local class="$1"
    local file="$CORPUS/$class.md"
    [ -f "$file" ] || return 1
    local out
    out="$(grep -E '^## ' "$file" \
        | sed -E 's/^## //; s/^[★ ]*//; s/^([A-Z][0-9]+[a-z]?)\..*$/\1/' \
        | grep -E '^[A-Z][0-9]+[a-z]?$' \
        | sort -u)"
    [ -n "$out" ] || return 3
    printf '%s\n' "$out"
}

violations=0
surfaces=0

while read -r path class; do
    case "$path" in ''|'#'*) continue ;; esac
    surfaces=$((surfaces + 1))

    if [ ! -f "$path" ]; then
        echo "  $path: listed in the registry and not on disk."
        violations=$((violations + 1))
        continue
    fi
    rows="$(rows_for "$class")" || rc=$?
    if [ "${rc:-0}" -eq 1 ]; then
        rc=0
        echo "  $path: names class '$class' and $CORPUS/$class.md does not exist."
        violations=$((violations + 1))
        continue
    elif [ "${rc:-0}" -eq 3 ]; then
        rc=0
        echo "  $path: names class '$class', and $CORPUS/$class.md has no rows."
        echo "      The file is there. Its headings are not being read: a row is a"
        echo "      line \`## <ID>. …\`, optionally decorated, where <ID> is a capital"
        echo "      letter and digits. A class with no rows demands nothing of any"
        echo "      surface in it, which is this gate's oracle having gone blind."
        violations=$((violations + 1))
        continue
    fi
    rc=0

    # The file's answers: every `- <ID> ` inside a comment.
    answers="$(grep -oE '^[[:space:]]*(//[/!]?|#)[[:space:]]*-[[:space:]]*[A-Z][0-9]+[a-z]?[[:space:]:]' "$path" 2>/dev/null \
        | grep -oE '[A-Z][0-9]+[a-z]?' | sort -u)"

    if ! grep -qE '(//[/!]?|#)[[:space:]#]*conventions:[[:space:]]*'"$class" "$path"; then
        echo "  $path: no \`conventions: $class\` block."
        echo "      Its class carries these rows: $(echo "$rows" | tr '\n' ' ')"
        echo "      See $CORPUS/$class.md"
        violations=$((violations + 1))
        continue
    fi

    missing=""
    for row in $rows; do
        echo "$answers" | grep -qx "$row" || missing="$missing $row"
    done
    if [ -n "$missing" ]; then
        echo "  $path ($class): unanswered —$missing"
        violations=$((violations + 1))
    fi

    # A waiver with no reason is the exemption without the argument.
    while IFS= read -r bare; do
        [ -z "$bare" ] && continue
        echo "  $path: waived with no reason — $(printf '%s' "$bare" | sed 's/^[[:space:]]*//')"
        violations=$((violations + 1))
    done < <(grep -nE '^[[:space:]]*(//[/!]?|#).*WAIVED[[:space:]]*$' "$path" 2>/dev/null | cut -d: -f2-)

done < "$REGISTRY"

echo "  $surfaces surface(s) registered."

# A registry that parsed to nothing reads exactly like a clean sweep: no
# violations found, because nothing was looked at. It is the same state as a
# missing registry and gets the same code.
if [ "$surfaces" -eq 0 ]; then
    echo "  …and that is none. $REGISTRY exists and lists no surface, so this run"
    echo "  compared nothing against the corpus. Exiting 2, not 0."
    exit 2
fi

if [ "$violations" -gt 0 ]; then
    cat <<'MSG'

A surface listed in tools/gates/conventions.list must answer every row of its
gesture class, in a comment block:

    // conventions: drag-moves
    // - D1 live-preview: the polyline follows the pointer, previewed from the
    //   same function a committed dimension is drawn from.
    // - D3 escape-cancels: WAIVED — the gesture machine owns Escape.

This gate cannot check behaviour and does not pretend to. It checks that the
question was ASKED. Every convention the operator has had to report was one
nobody had asked about — not one somebody decided against.
MSG
    echo
    echo "check-conventions: FAIL — $violations surface(s) with an unanswered class."
    exit 1
fi

echo "check-conventions: PASS — every registered surface has answered."
exit 0
