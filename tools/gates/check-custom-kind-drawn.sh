#!/usr/bin/env bash
#
# check-custom-kind-drawn.sh — every ribbon control the manifest reserves space
# for must have something that draws it.
#
# ═══════════════════════════════════════════════════════════════════════════
# WHAT THIS GATE IS FOR
# ═══════════════════════════════════════════════════════════════════════════
#
# `Item::Custom` is the one place the ribbon manifest stops describing a
# control and starts trusting the application to draw one. The shell lays out
# a slot, emits a `kind` string, and hands it to the application's custom-item
# closure. If nothing in that closure matches the string, the slot is drawn
# empty — and a captioned group with an empty band under it looks, at a
# glance, exactly like a group whose control has not loaded yet.
#
# ★★★ THIS HAS SHIPPED ONCE, FOR A WHOLE RELEASE.
#
# The Markup ▸ Style group emitted the literal `"colour_swatch"` and no
# renderer ever matched it. Every test was green: the manifest was
# well-formed, the group was reachable, the item was declared, the caption
# drew. The only thing missing was the widget, and nothing in the tree was
# asking whether a widget existed.
#
# ═══════════════════════════════════════════════════════════════════════════
# ★★ THE RULE, IN TWO HALVES
# ═══════════════════════════════════════════════════════════════════════════
#
# 1. **A kind is a named constant, never a string literal.** `Item::custom(
#    super::MARKUP_FILL)`, not `Item::custom("markup_fill")`. A literal is how
#    the shipped defect was written, and it is worse than a typo: there is no
#    symbol for a renderer to match on, so the two sides agree only by two
#    people spelling the same word, and nothing in the language notices when
#    they stop.
#
# 2. **The constant is named again, outside the manifest, in code.** That is
#    the renderer — `app::ocrband`'s `if kind != OCR_BLEND`, `app::fontband`'s
#    `command_for`, `app::surfaces`' two inline arms. It does not matter which
#    file, and this gate deliberately does not say: naming the file would make
#    the gate a second, staler statement of the dispatch chain.
#
# ═══════════════════════════════════════════════════════════════════════════
# ★ WHY A COMMENT DOES NOT COUNT
# ═══════════════════════════════════════════════════════════════════════════
#
# Because this project writes long ones, and by the time the defect above was
# found, `COLOUR_SWATCH` was discussed by name in half a dozen doc comments
# across the crate. A gate that accepted any mention would have been green
# throughout, on the strength of prose explaining that the widget did not
# exist.
#
# ⇒ Lines whose first non-space characters are `//`, `*` or `/*` are not
# evidence that anything is drawn. Only code is.
#
# ═══════════════════════════════════════════════════════════════════════════
# WHAT THIS GATE DOES NOT CLAIM
# ═══════════════════════════════════════════════════════════════════════════
#
# That the renderer draws the RIGHT control, or that it draws one under the
# conditions the item is shown in. A kind matched by a renderer that returns
# early on every frame would pass here. That question belongs to the
# renderer's own tests — `ocrband`'s `a_foreign_kind_draws_nothing_even_with_a
# _blend_to_draw` is the shape — and to a driven check.
#
# This gate answers the narrower question nothing else was asking: *is there
# anything at the other end of this string at all.*
#
# ═══════════════════════════════════════════════════════════════════════════
# USAGE
# ═══════════════════════════════════════════════════════════════════════════
#   tools/gates/check-custom-kind-drawn.sh              scan the tree
#   tools/gates/check-custom-kind-drawn.sh --self-test  prove it can fail
#
#   0  clean · 1  a violation, or the self-test did not detect its plant
#   2  SKIPPED — the manifest emitted no custom items at all

set -uo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
cd "$ROOT" || exit 1

# Collect every kind the manifest emits, as written.
#
# The unit is the `Item::custom(` call, because that is the moving side: a
# thirteenth control is added by writing one of these, and a gate keyed on
# anything else — a register, a marker comment, a constant naming convention —
# is a gate the thirteenth control can be added without touching.
#
# A call inside the manifest's own `#[cfg(test)]` block is collected too. It
# names the same constants production code does, so it deduplicates away; and
# a test that emitted a kind no tab emits would be asserting against a control
# that is not in the product, which is worth a red line of its own.
emissions() {
    local mdir="$1"
    grep -rh --include='*.rs' 'Item::custom(' "$mdir" 2>/dev/null |
        sed -E 's/^[[:space:]]+//' |
        grep -vE '^(//|\*|/\*)' |
        sed -E 's/.*Item::custom\(([^)]*)\).*/\1/' |
        sed -E 's/^[[:space:]]*//; s/[[:space:]]*$//'
}

scan() {
    local mdir="$1" tree="$2" mdirname="$3"
    local found=0 seen="" raw id

    local all
    all=$(emissions "$mdir" | sort -u)
    [ -z "$all" ] && return 2

    while IFS= read -r raw; do
        [ -z "$raw" ] && continue

        # Half one: a literal is a kind with no symbol behind it.
        case "$raw" in
            \"*)
                printf '  %s  — a string literal, not a named constant\n' "$raw"
                found=$((found + 1))
                continue
                ;;
        esac

        # Strip whatever path the call site used to reach the constant —
        # `super::`, `crate::shell::manifest::`, `egui_shell::manifest::` — so
        # that the same constant reached two ways is one entry.
        id="${raw##*::}"

        # Anything that is not a SCREAMING_SNAKE constant is a computed kind,
        # and a computed kind cannot be checked by this gate at all. Say so
        # rather than passing it silently.
        if ! printf '%s' "$id" | grep -qE '^[A-Z][A-Z0-9_]*$'; then
            printf '  %s  — not a named constant, so no renderer can be found for it\n' "$raw"
            found=$((found + 1))
            continue
        fi

        case " $seen " in
            *" $id "*) continue ;;
        esac
        seen="$seen $id"

        # Half two.
        if ! grep -rh --include='*.rs' -w "$id" "$tree" --exclude-dir="$mdirname" 2>/dev/null |
            sed -E 's/^[[:space:]]+//' |
            grep -qvE '^(//|\*|/\*)'; then
            printf '  %s  — emitted by the manifest, named by no code outside it\n' "$id"
            found=$((found + 1))
        fi
    done <<< "$all"

    return $((found > 0 ? 1 : 0))
}

if [ "${1:-}" = "--self-test" ]; then
    tmp=$(mktemp -d)
    trap 'rm -rf "$tmp"' EXIT
    mkdir -p "$tmp/src/shell/manifest" "$tmp/src/app"

    # The manifest side: four emissions, two of them wrong.
    cat > "$tmp/src/shell/manifest/format.rs" <<'RS'
fn group() -> Group {
    group(
        "format.style",
        "Style",
        [
            Item::custom(super::REAL_WIDGET),
            Item::custom(super::GHOST_WIDGET),
            Item::custom("hand_written"),
            Item::custom(crate::shell::manifest::PROSE_ONLY),
        ],
    )
}
RS

    # A renderer for one of them, in code.
    cat > "$tmp/src/app/band.rs" <<'RS'
fn draw(ui: &mut egui::Ui, kind: &str) {
    if kind == crate::shell::manifest::REAL_WIDGET {
        ui.label("drawn");
    }
}
RS

    # …and a file that discusses another at length and draws nothing. This is
    # the case the whole gate turns on.
    cat > "$tmp/src/app/notes.rs" <<'RS'
/// The Style group's swatch.
///
/// PROSE_ONLY is the kind the manifest emits for it. The control it names has
/// not been built yet, which is why PROSE_ONLY appears nowhere else.
// PROSE_ONLY
fn unrelated() {}
RS

    out=$(scan "$tmp/src/shell/manifest" "$tmp/src" "manifest")
    rc=$?

    fail=0
    if [ "$rc" -ne 1 ]; then
        echo "custom-kind-drawn --self-test: FAIL — the planted violations were not detected."
        fail=1
    fi
    if ! printf '%s' "$out" | grep -q 'GHOST_WIDGET'; then
        echo "custom-kind-drawn --self-test: FAIL — a kind with no renderer was not reported."
        fail=1
    fi
    if ! printf '%s' "$out" | grep -q 'hand_written'; then
        echo "custom-kind-drawn --self-test: FAIL — a string-literal kind was not reported."
        fail=1
    fi
    if ! printf '%s' "$out" | grep -q 'PROSE_ONLY'; then
        echo "custom-kind-drawn --self-test: FAIL — a kind named only in comments was accepted."
        echo "  That is the exact way this gate would have been green through the"
        echo "  release that shipped an empty band, so it is the case that matters."
        fail=1
    fi
    if printf '%s' "$out" | grep -q 'REAL_WIDGET'; then
        echo "custom-kind-drawn --self-test: FAIL — a kind with a renderer was reported."
        echo "  A gate that reports the correct shape trains people to ignore it."
        fail=1
    fi

    # And the empty case really does SKIP rather than pass.
    mkdir -p "$tmp/empty/src/shell/manifest"
    scan "$tmp/empty/src/shell/manifest" "$tmp/empty/src" "manifest" >/dev/null
    erc=$?
    if [ "$erc" -ne 2 ]; then
        echo "custom-kind-drawn --self-test: FAIL — a manifest with no custom items did not SKIP."
        fail=1
    fi

    [ "$fail" -ne 0 ] && exit 1
    echo "custom-kind-drawn --self-test: PASS — catches the unrendered kind, the string"
    echo "  literal and the comment-only mention; passes the kind that is really drawn;"
    echo "  SKIPs rather than passes when there is nothing to scan."
    exit 0
fi

MDIR="crates/pdfcer-gui/src/shell/manifest"
TREE="crates/pdfcer-gui/src"

echo "check-custom-kind-drawn: checking every Item::custom kind has a renderer…"
out=$(scan "$MDIR" "$TREE" "manifest")
rc=$?

if [ "$rc" -eq 2 ]; then
    echo "check-custom-kind-drawn: SKIPPED — the manifest emits no custom items."
    echo "  If that is a surprise, the scan root moved: $MDIR"
    exit 2
fi
if [ "$rc" -eq 1 ]; then
    echo "check-custom-kind-drawn: FAIL — the ribbon reserves space for a control nothing draws:"
    printf '%s\n' "$out"
    cat <<'MSG'

An `Item::Custom` is a slot plus a string. The shell lays out the slot and
hands the string to the application's custom-item closure; if nothing there
matches, the slot draws empty under its group caption, which reads as a
control that has not loaded rather than a control that does not exist.

That shipped once, for the whole of v0.1.0, as `"colour_swatch"`.

To fix:

  · A kind is a CONSTANT in `shell::manifest`, never a string literal. The
    constant is what a renderer matches on, and it is what makes the two sides
    fail to compile when they stop agreeing.

  · Something outside the manifest must name that constant in CODE. A doc
    comment explaining the control is not a control. See `app::ocrband` for
    the smallest complete example: a kind guard, a renderer, and a test that
    a foreign kind draws nothing.

  · If the control is genuinely not built yet, do not emit the item. R9: an
    unavailable capability renders nothing — including no slot, no caption
    line under it, and no space held for it.
MSG
    exit 1
fi
# The count is printed on the CLEAN line and not only on failure, because a
# gate that says "clean" without saying what it looked at is indistinguishable
# from a gate whose scan root has moved and which now looks at one file.
# Counted after the path a call site used to reach the constant is stripped,
# the same way `scan` deduplicates: `super::COLOUR_SWATCH` and a bare
# `COLOUR_SWATCH` in the manifest's own tests are one kind, not two.
count=$(emissions "$MDIR" | sed -E 's/.*:://' | sort -u | wc -l | tr -d ' ')
echo "check-custom-kind-drawn: clean — all $count custom kinds the manifest emits are drawn."
exit 0
