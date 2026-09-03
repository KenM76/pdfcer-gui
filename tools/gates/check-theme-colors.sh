#!/usr/bin/env bash
#
# check-theme-colors.sh — no raw colour outside the theme module.
#
# WHAT THIS GATE IS FOR
# =====================
#
# This application's appearance is data, in `crates/egui-shell/src/theme/`,
# so that changing the look is a one-module edit instead of a sweep through
# ~69,000 lines. That property survives only if new colours keep landing
# there.
#
# The failure this catches is not a crash and not a wrong colour — it is a
# colour that works fine today and is invisible to the next restyle. Six
# months of "just this one literal, it's only a hairline" and the theme
# module is decoration: the app has a palette AND a scattering of
# hard-coded colours that no longer match it, and nobody finds out until
# someone switches preset and half the canvas stays light.
#
# That is not hypothetical here. Defect D2 — the one this whole project
# started from — was section headings and dock tab labels rendering
# invisible in the default preset, because a role was assigned a
# near-white colour while a companion role was never assigned at all. A
# theme that is only *mostly* the single source of truth produces exactly
# that class of bug, and produces it in the preset nobody tests in.
#
# WHY IT ARRIVED LATE, WHICH IS ITSELF THE FINDING
# ================================================
#
# `D:\Dev\pdfcer` has had this gate for some time. It was never ported when
# this workspace was created, so from S0 until 2026-08-13 the theme rule
# was enforced by nothing but habit — through the entire construction of
# the ribbon, the dock, the panels and the canvas.
#
# It surfaced only because the icon salvage was told to respect it, went
# looking for `tools/gates/check-theme-colors.sh`, and reported that the
# file did not exist. The agent had emulated pdfcer's copy by hand and
# found its own work clean; nothing else in the tree had been checked at
# all. An instruction to obey a gate is not a gate, and a rule that lives
# only in agent prompts is enforced exactly as often as someone remembers
# to write it into one.
#
# Same shape as `check-ui-strings.sh`, deliberately. That gate keeps every
# operator-visible string in one module, which is what makes rewording
# safe; this one keeps every colour in one module, which is what makes
# restyling safe. Both are cheap greps that catch a class of drift no
# compiler and no unit test can see.
#
# ★ THE ESCAPE HATCH IS NOT A LOOPHOLE — IT IS THE POINT
# ======================================================
#
# Some colours in a PDF editor are written INTO THE DOCUMENT: the colour
# of an annotation the operator authors, an appearance-stream colour,
# anything that reaches a saved file. Those are the operator's choice
# about document content. They are not chrome, and a theme must never
# move them — restyling the application would change the colour of markup
# about to be committed to a file, and the change would only become
# visible after saving.
#
# So a line carrying `DOCUMENT COLOUR:` or `NOT A THEME COLOUR:` is
# allowed. The gate's job is to catch the colour someone forgot to name —
# not to forbid the ones that must stay exactly where they are. A gate
# with no way to say "this one is different" gets switched off the first
# time it is right about the wrong thing.
#
# EXIT CODES  (the three-state model — see run-all.sh)
# ===================================================
#   0  clean
#   1  at least one un-named colour outside the theme module
#   2  SKIPPED — the theme module does not exist, so there is nothing to
#      protect and a "clean" verdict would be a lie. NOT a pass.
set -uo pipefail

cd "$(dirname "$0")/../.." || exit 1

THEME_DIR="crates/egui-shell/src/theme"
# Both crates are scanned. Scoping this to `pdfcer-gui` alone would have
# left the reusable shell — the crate that OWNS the palette and draws the
# ribbon, dock and tab labels D2 was about — as the one place a raw
# colour could hide.
SRC_ROOTS=("crates/egui-shell/src" "crates/pdfcer-gui/src")

# `from_gray` and the named constants (`Color32::RED`) count too: a
# literal grey is exactly as invisible to a restyle as a literal blue.
PATTERN='Color32::(from_rgb|from_rgba_unmultiplied|from_rgba_premultiplied|from_gray|from_black_alpha|from_white_alpha|BLACK|WHITE|GRAY|LIGHT_GRAY|DARK_GRAY|RED|GREEN|BLUE|YELLOW|BROWN|GOLD|KHAKI|ORANGE|PURPLE)'
#
# `TRANSPARENT` is deliberately NOT in that list. It is the absence of a
# colour, not a choice of one — no theme would ever change it, and
# requiring a marker on every transparent fill would train people to add
# markers without reading them, which is how a gate stops being read.

MARKER='DOCUMENT COLOUR:|NOT A THEME COLOUR:'

scan() {
    # $1 = a root to scan. Emits `file:line:text` for each offender.
    #
    # The marker is honoured on the offending line OR on any of the seven
    # lines above it, because that is where a comment explaining a line
    # actually goes — and an explanation worth writing is usually longer
    # than one line. pdfcer's first version allowed three, which rejected a
    # five-line comment saying exactly what the gate asks for.
    find "$1" -name '*.rs' -not -path "$THEME_DIR/*" -print0 \
    | xargs -0 awk -v pat="$PATTERN" -v marker="$MARKER" '
        FNR == 1 { for (i = 0; i < 8; i++) recent[i] = "" }
        {
          marked = ($0 ~ marker)
          for (i = 0; i < 8; i++) if (recent[i] ~ marker) marked = 1
          # A comment mentioning a colour is prose, not a drawn colour.
          is_comment = ($0 ~ /^[ \t]*(\/\/|\*)/)
          if (!marked && !is_comment && $0 ~ pat)
            printf "%s:%d:%s\n", FILENAME, FNR, $0
          for (i = 7; i > 0; i--) recent[i] = recent[i-1]
          recent[0] = $0
        }
      '
}

# ---------------------------------------------------------------------------
# --self-test — prove the gate can catch its own planted violation.
#
# Runs FIRST in run-all.sh, before this gate's verdict on the real tree is
# trusted. PROJECT_PLAN.md §4.1 records a sibling gate that printed
# "clean" while checking a handful of files: finding nothing looks exactly
# like finding no violations, and the only way to tell them apart is to
# hand the gate something it MUST find.
#
# Three cases, because two of them are the ways this gate could rot into
# uselessness without anyone noticing:
#   1. a plain raw colour            -> must be caught
#   2. the same, with a marker       -> must NOT be caught (escape works)
#   3. the same, inside a comment    -> must NOT be caught (prose is prose)
# ---------------------------------------------------------------------------
if [ "${1:-}" = "--self-test" ]; then
    tmp=$(mktemp -d) || { echo "theme-colors self-test: FAIL — no temp dir"; exit 1; }
    trap 'rm -rf "$tmp"' EXIT
    mkdir -p "$tmp/src"
    cat > "$tmp/src/planted.rs" <<'RS'
fn caught() {
    let c = egui::Color32::from_rgb(1, 2, 3);
}
fn exempt() {
    // NOT A THEME COLOUR: an exact value asserted by a test fixture.
    let c = egui::Color32::from_rgb(4, 5, 6);
}
// Prose: a comment may say Color32::RED without drawing anything.
RS
    found=$(scan "$tmp/src")
    n=$(printf '%s' "$found" | grep -c . || true)
    fails=0
    if ! printf '%s' "$found" | grep -q ':2:'; then
        echo "theme-colors self-test: FAIL — did not catch the planted raw colour"
        fails=1
    fi
    if printf '%s' "$found" | grep -q ':6:'; then
        echo "theme-colors self-test: FAIL — the marker escape did not exempt line 6"
        fails=1
    fi
    if printf '%s' "$found" | grep -q ':8:'; then
        echo "theme-colors self-test: FAIL — a colour named in a comment was reported"
        fails=1
    fi
    if [ "$n" -ne 1 ]; then
        echo "theme-colors self-test: FAIL — expected exactly 1 offender, got $n:"
        printf '%s\n' "$found" | sed 's/^/  /'
        fails=1
    fi
    [ "$fails" -ne 0 ] && exit 1
    echo "theme-colors self-test: the gate catches its own planted violation."
    exit 0
fi

# ---------------------------------------------------------------------------
# The real run.
# ---------------------------------------------------------------------------
if [ ! -d "$THEME_DIR" ]; then
    echo "theme-colors: SKIPPED — $THEME_DIR does not exist."
    echo "  The gate has nothing to protect, and reporting 'clean' here would"
    echo "  mean 'no theme module' — which is not the same as 'no violations'."
    exit 2
fi

present=()
for root in "${SRC_ROOTS[@]}"; do
    [ -d "$root" ] && present+=("$root")
done
if [ ${#present[@]} -eq 0 ]; then
    echo "theme-colors: SKIPPED — none of the source roots exist: ${SRC_ROOTS[*]}"
    exit 2
fi
if [ ${#present[@]} -ne ${#SRC_ROOTS[@]} ]; then
    # Deliberately a SKIP, not a pass. A partial checkout that silently
    # checked half the tree is precisely the failure §4.1 is about.
    echo "theme-colors: SKIPPED — only ${#present[@]} of ${#SRC_ROOTS[@]} source roots exist."
    echo "  present: ${present[*]}"
    echo "  A verdict over part of the tree would read as a verdict over all of it."
    exit 2
fi

offenders=""
for root in "${present[@]}"; do
    out=$(scan "$root")
    [ -n "$out" ] && offenders="${offenders}${out}"$'\n'
done
offenders=$(printf '%s' "$offenders" | grep -c . >/dev/null 2>&1 && printf '%s' "$offenders" || printf '%s' "$offenders")

if printf '%s' "$offenders" | grep -q .; then
    echo "theme-colors: FAIL — raw colours outside $THEME_DIR:"
    printf '%s' "$offenders" | grep . | sed 's/^/  /'
    cat <<'EOF'

Every colour this application draws belongs to a named role in
crates/egui-shell/src/theme/, so that a restyle is one module rather than
a sweep. Add a role to the palette and use it here.

If this colour is written INTO THE DOCUMENT rather than drawn as chrome —
annotation colour, appearance-stream colour, anything that reaches a
saved file — it must NOT be themed. Mark the line:

    // DOCUMENT COLOUR: <why it reaches the file>

If it is neither chrome nor document content (arithmetic on an existing
colour, a mask, a test fixture asserting an exact value), mark it:

    // NOT A THEME COLOUR: <why>
EOF
    exit 1
fi

echo "theme-colors: clean — every colour is a named role in $THEME_DIR"
exit 0
