#!/usr/bin/env bash
#
# check-unit-conversion.sh — one length-conversion table, one rounding rule.
#
# WHAT THIS GATE IS FOR
# =====================
#
# Every conversion between PDF points and a length an operator reads or types
# belongs in `crates/pdfcer-gui-base/src/units.rs`. This gate fails the build when a
# second copy appears.
#
# It exists because the first copy was not the problem — the thirteenth was.
# `UNIT_SURFACES.md` §3, written 2026-09-13, enumerated what this program held
# before `units.rs`:
#
#   * six private constants under two names (`PTS_PER_MM`, `PT_PER_MM`), in six
#     files, two of them `f32` and four `f64`;
#   * seven inline closures re-declaring `|pt: f64| pt * 25.4 / 72.0`, five of
#     them in a single file;
#   * and `pdfcer_core`'s `Unit::baseline_per_point`, which is the value all
#     thirteen were approximating.
#
# None of those thirteen was written carelessly. Each was one line, obviously
# correct, and cheaper to type than to find. That is the shape of this defect:
# it is never worth stopping for, and the thirteenth is indistinguishable from
# the first.
#
# ★★★ WHAT IT ACTUALLY COST, MEASURED
# ===================================
#
# A sheet authored at exactly 210.5 mm rendered `210` in the page-thumbnail
# tooltip and `211` in the print dialogue. One document, one sheet, two whole
# numbers, on two surfaces the operator can have open at the same time.
#
# And the cause was not the arithmetic — both surfaces computed exactly
# 210.5000000000 mm. It was the ROUNDING RULE. Half the sites wrote
# `.round() as i64` (half away from zero, the CAD convention); the other half
# wrote `{:.0}` inside a format string, which is Rust's default and rounds half
# to EVEN. Nobody chose half-to-even. It arrived because `{:.0}` is what you
# type when you want no decimals.
#
# So this gate checks two things, and the second is the one that bit:
#
#   1. no second conversion constant   — the `25.4` / `0.0254` / `*_PER_MM` half
#   2. no `{:.0}` on a length          — the rounding-rule half
#
# A gate that only did (1) would have consolidated the arithmetic and left the
# operator-visible defect exactly where it was.
#
# ★ THE ESCAPE HATCHES, AND THE TWO CLASSES THEY EXIST FOR
# ========================================================
#
# A line carrying either marker is allowed, on the line itself or within the
# seven lines above it (same convention as `check-theme-colors.sh`, because an
# explanation worth writing is usually longer than one line).
#
#   NOT A DOCUMENT LENGTH:       — this is a TYPE SIZE, not a length
#   ORACLE, NOT A CONVERSION:    — this is a TEST'S EXPECTED VALUE
#
# **Class 1 — type size.** Named in `UNIT_SURFACES.md` §4. A font size printed
# as `12 pt` is the typographic point. It is arithmetically the same 1/72 inch
# and it must never be offered in millimetres, metres or kilometres — no
# drawing or publishing program does this and Acrobat does not. Those surfaces
# are excluded deliberately, and an un-annotated exclusion is
# indistinguishable from an omission: a later reader re-opens the question,
# re-derives the same answer, or worse, does not.
#
# **Class 2 — a test's oracle.** ★ This class was not anticipated when the gate
# was written; it was DISCOVERED by running the gate on a clean tree, where all
# four remaining hits turned out to be tests. Two shapes:
#
#   * an expected value stated as a literal —  `assert_eq!(mm.label(72.0),
#     "25.40 mm")`, where the 25.4 is the answer, not an arithmetic step;
#   * an oracle computed by hand — `let expected = in_inches * 0.0254;`, where
#     the whole value of the assertion is that the test did NOT go through the
#     code under test.
#
# Routing either through `units.rs` would make the test assert that `units.rs`
# equals `units.rs`. That is not a weaker test, it is not a test. The standing
# lesson this project already carries — *an oracle built from the system under
# test needs an independent calibration* — says the hand-spelled constant in a
# test is the point, so the gate must be able to hear "deliberately, and here
# is why" rather than be argued with.
#
# ⚠ What class 2 does NOT license is a test that pins a FORMAT. A test
# asserting `"210"` out of a `{width_mm:.0}` is not an oracle, it is the
# half-to-even defect written down as an expectation. Mark the arithmetic;
# never mark the rounding.
#
# A gate with no way to say "this one is different" gets switched off the first
# time it is right about the wrong thing.
#
# WHAT IT DELIBERATELY DOES NOT CATCH
# ===================================
#
# `/ 72.0` on its own. The points-per-inch divide appears in render scaling,
# DPI arithmetic and coordinate transforms, most of which are not lengths an
# operator reads. Flagging all of them would return dozens of hits on a clean
# tree and teach people to write exemptions, which is how a gate turns into
# scenery. `units.rs` offers `scale_from_dpi` and `pixels_per_metre` for the
# DPI cases and the three sites that had them now call it, but the gate's
# patterns stay narrow enough that every hit is a genuine second copy.
#
# ⚠ That is a known, deliberate hole, stated here rather than left for someone
# to discover as a surprise. If a fourteenth spelling ever appears as bare
# `/ 72.0`, this gate will not see it.
#
# ⚠ And the MIRROR of the type-size exclusion. This gate fails a type size that
# converts itself by hand; it cannot fail a type size routed CORRECTLY through
# `units.rs` into millimetres — `units::mm_from_points(font_size_pt)` is a
# clean line by every pattern here, and it is precisely the mistake
# `UNIT_SURFACES.md` §4 exists to prevent. Every candidate identifier (`size`,
# `size_pt`, `height`) is also a legitimate page-size name in this tree, so a
# pattern for it would return more false hits than real ones. The instrument
# for that one is §4 and a reader; it is not this file, and pretending
# otherwise would be worse than saying so.
#
# EXIT CODES  (the three-state model — see run-all.sh)
# ===================================================
#   0  clean
#   1  a second conversion copy, or a length formatted with `{:.0}`
#   2  SKIPPED — `units.rs` does not exist, so there is nothing to centralise
#      on and a "clean" verdict would be a lie. NOT a pass.
set -uo pipefail

cd "$(dirname "$0")/../.." || exit 1

# The table lives in the base crate; the callers are spread over both. SCAN
# BOTH ROOTS. A root list that names only the crate the table used to live in
# would not fail — it would find nothing and report clean, which is the one
# outcome indistinguishable from a pass.
TABLE="crates/pdfcer-gui-base/src/units.rs"
SRC_ROOTS=("crates/pdfcer-gui/src" "crates/pdfcer-gui-base/src")

# Three families, all narrow enough that a hit is always a real second copy.
#
#   25\.4     millimetres per inch — the constant every private copy spelled
#   0\.0254   metres per inch — the SI spelling, for pixels-per-metre
#   PTS?_PER_MM  the two names the six private constants actually used, caught
#                by name so that re-introducing one is flagged even if the
#                author computes it some other way
PATTERN_CONST='25\\.4|0\\.0254|PTS?_PER_MM'

# A named length formatted with no decimals. `{width_mm:.0}`, `{h_in:.0}`,
# `{:.0}` is NOT matched — a positional format carries no name, so the gate
# cannot tell a length from a percentage and a guess would be noise.
#
# ⚠ That is the second deliberate hole. It is narrower than it looks: this
# program's convention is captured identifiers, so a length reaching a format
# string almost always arrives as `{something_mm:.0}`.
PATTERN_FMT='\\{[A-Za-z_][A-Za-z0-9_]*_(mm|cm|in|inches|ft|m):\\.0\\}'

# Two markers, one alternation — `awk`'s `~` takes an ERE, so the `|` is the
# regex alternation and everything else here is literal. See the escape-hatch
# section above for what each one licenses and what neither of them does.
MARKER='NOT A DOCUMENT LENGTH:|ORACLE, NOT A CONVERSION:'

scan() {
    # $1 = a root to scan. Emits `file:line:text` for each offender.
    find "$1" -name '*.rs' -not -path "*/units.rs" -print0 \
    | xargs -0 awk -v pc="$PATTERN_CONST" -v pf="$PATTERN_FMT" -v marker="$MARKER" '
        FNR == 1 { for (i = 0; i < 8; i++) recent[i] = "" }
        {
          marked = ($0 ~ marker)
          for (i = 0; i < 8; i++) if (recent[i] ~ marker) marked = 1
          # Prose naming a constant is not arithmetic on one. Every one of the
          # thirteen replaced sites carried a doc comment explaining 25.4, and
          # a gate that reported those would have returned more comment hits
          # than code hits on the day it was written.
          is_comment = ($0 ~ /^[ \t]*(\/\/|\*|#)/)
          if (!marked && !is_comment && ($0 ~ pc || $0 ~ pf))
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
# trusted. A gate that finds nothing looks exactly like a gate that is not
# looking, and the only way to tell them apart is to hand it something it MUST
# find. This project has caught four inert checks that way.
#
# Five cases, one per way this gate could rot:
#   1. a raw 25.4 constant             -> caught
#   2. a `{w_mm:.0}` format            -> caught   (the rounding-rule half)
#   3. the same constant, marked       -> NOT caught (escape hatch 1 works)
#   4. a constant named in a comment   -> NOT caught (prose is prose)
#   5. a `{:.0}` with no identifier    -> NOT caught (the stated hole, asserted
#                                         so that closing it is a deliberate
#                                         edit here rather than a surprise)
#   6. a test oracle, marked           -> NOT caught (escape hatch 2 works)
#
# ★ Cases 3 and 6 are separate on purpose. One marker regex serving two
# markers is exactly the shape that survives a narrowing unnoticed: if someone
# later rewrites MARKER and drops an alternative, a self-test that only planted
# one of them would stay green while half the escape hatch stopped existing.
# ---------------------------------------------------------------------------
if [ "${1:-}" = "--self-test" ]; then
    tmp=$(mktemp -d) || { echo "unit-conversion self-test: FAIL — no temp dir"; exit 1; }
    trap 'rm -rf "$tmp"' EXIT
    mkdir -p "$tmp/src"
    cat > "$tmp/src/planted.rs" <<'RS'
fn caught_constant() {
    let mm = pt * 25.4 / 72.0;
}
fn caught_format(width_mm: f64) -> String {
    format!("{width_mm:.0} mm")
}
fn exempt() {
    // NOT A DOCUMENT LENGTH: the typographic point, never shown in mm.
    let pts = 25.4;
}
// Prose: a comment may say 25.4 or PTS_PER_MM without converting anything.
fn positional(v: f64) -> String {
    format!("{:.0} %", v)
}
fn oracle() {
    // ORACLE, NOT A CONVERSION: a test's expected value, computed by hand so
    // that the assertion does not go through the code it is testing.
    let expected = 100.0 * 0.0254;
}
RS
    found=$(scan "$tmp/src")
    fails=0
    for want in ':2:' ':5:'; do
        if ! printf '%s' "$found" | grep -q "$want"; then
            echo "unit-conversion self-test: FAIL — did not catch the planted violation at line ${want//:/}"
            fails=1
        fi
    done
    for reject in ':9:' ':11:' ':13:' ':18:'; do
        if printf '%s' "$found" | grep -q "$reject"; then
            echo "unit-conversion self-test: FAIL — line ${reject//:/} should not have been reported"
            fails=1
        fi
    done
    n=$(printf '%s' "$found" | grep -c . || true)
    if [ "$n" -ne 2 ]; then
        echo "unit-conversion self-test: FAIL — expected exactly 2 offenders, got $n:"
        printf '%s\n' "$found" | sed 's/^/  /'
        fails=1
    fi
    [ "$fails" -ne 0 ] && exit 1
    echo "unit-conversion self-test: the gate catches a second constant AND a {:.0} length."
    exit 0
fi

# ---------------------------------------------------------------------------
# The real run.
# ---------------------------------------------------------------------------
if [ ! -f "$TABLE" ]; then
    echo "unit-conversion: SKIPPED — $TABLE does not exist."
    echo "  There is no table to centralise on, so 'clean' here would mean"
    echo "  'no table' — which is not the same as 'no second copies'."
    exit 2
fi
for root in "${SRC_ROOTS[@]}"; do
    if [ ! -d "$root" ]; then
        echo "unit-conversion: SKIPPED — $root does not exist."
        exit 2
    fi
done

offenders=$(for root in "${SRC_ROOTS[@]}"; do scan "$root"; done)

if printf '%s' "$offenders" | grep -q .; then
    echo "unit-conversion: FAIL — a second length conversion outside $TABLE:"
    printf '%s' "$offenders" | grep . | sed 's/^/  /'
    cat <<'EOF'

Every conversion between PDF points and a length an operator reads or types
goes through crates/pdfcer-gui-base/src/units.rs:

    units::mm_from_points(pt)          points -> millimetres
    units::points_from_mm(mm)          millimetres -> points
    units::from_points(pt, unit)       points -> any engine Unit
    units::whole_mm_from_points(pt)    points -> whole millimetres, rounded
                                       half AWAY FROM ZERO

⚠ If the hit is a `{something_mm:.0}` format, do not simply widen the format
spec. `{:.0}` rounds half to EVEN and disagrees with the rest of this program
on every tie — a 210.5 mm sheet printed 210 on one surface and 211 on another
for exactly this reason. Call `units::whole_mm_from_points` and print it
with `{}`.

If this is a TYPE SIZE rather than a document length — a font size, a stamp's
point size, a dimension group's paper-relative text height — it is deliberately
excluded. Mark the line:

    // NOT A DOCUMENT LENGTH: <why this is a typographic point>

See UNIT_SURFACES.md §3 for the enumeration and §4 for the exclusions.
EOF
    exit 1
fi

echo "unit-conversion: clean — one length table in $TABLE, and no length formatted with {:.0}"
exit 0
