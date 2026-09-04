#!/usr/bin/env bash
#
# check-plate-colour.sh — `on_accent` may only be drawn ON the accent.
#
# ═══════════════════════════════════════════════════════════════════════════
# WHAT THIS GATE IS FOR
# ═══════════════════════════════════════════════════════════════════════════
#
# `Palette::on_accent` means, in its own words, *"text and icons drawn ON
# `Self::accent`"*. It is a plate colour: near-white under the light presets,
# near-black under the dark one, chosen to read against `accent` and against
# nothing else.
#
# Drawn on anything else it is unreadable, and it is unreadable *quietly* —
# a pale glyph on a pale panel is present, correctly sized, and invisible.
#
# ★★★ THIS IS DEFECT D2, AND IT HAS NOW HAPPENED THREE TIMES.
#
#   1. The active ribbon tab took `egui`'s selection visuals and a plate colour
#      meant for content, and rendered near-white on light grey. Fixed by
#      moving to `accent` + `on_accent`.
#   2. Every dialog's affirmative button was filled with
#      `visuals.selection.bg_fill` — a 27 % canvas tint — so the default action
#      rendered PALER than the Cancel beside it. The operator pressed it a
#      dozen times and found a dozen queued print jobs. Fixed by
#      `Theme::accent_pair`.
#   3. 2026-09-03, both found by an outside reviewer looking at screenshots:
#      the selected DOCK tab (`on_accent` over a 27 % wash: luminance gap 45 /
#      28 / 53 across the three presets, against a floor of 90) and the
#      document tab's close glyph (`on_accent` on the bare panel: 18 / 5 / 29 —
#      Airy is white on white to within five levels).
#
# ═══════════════════════════════════════════════════════════════════════════
# ★★ WHY A GREP AND NOT A CONTRAST TEST
# ═══════════════════════════════════════════════════════════════════════════
#
# Because the contrast gate structurally cannot see this class, and the reason
# is worth stating precisely.
#
# `theme::contrast::pairs` enumerates `WidgetState::ALL` × `FillKind::ALL` — ten
# pairs, foreground always `fg_stroke.color`. A colour a CALLER supplies through
# `RichText::color(...)` is not in that matrix and never can be: the gate reads
# an `egui::Style`, and the caller's choice is not in the style.
#
# Worse, for case 3b the background is chosen by GEOMETRY. The close button's
# rect is carved out of the tab's rect and drawn `.frame(false)`, so what is
# behind the glyph is whatever the strip painted — a fact no amount of reading
# `Style` can recover.
#
# ⇒ A perceptual gate answers *"is this pair readable"*. This one answers
# *"were these two things ever paired at all"*, which is the question that was
# actually going wrong.
#
# ═══════════════════════════════════════════════════════════════════════════
# THE RULE
# ═══════════════════════════════════════════════════════════════════════════
#
# A FUNCTION that draws with `on_accent` must also name its plate:
#
#   · `palette.accent`  — stated as a fill, a stroke or a painted rect;
#   · `accent_pair`     — the named pairing accessor, which supplies both;
#   · `.fill(`          — a fill is being stated on the widget.
#
# ★★ THE UNIT IS THE FUNCTION, and a line window was tried first and was
# wrong. `check-strong-text.sh` uses four lines either side, on the stated
# ground that *"a `.color()` five lines away is not a pairing a reader can
# see."* That is right for ITS question — two attributes on one widget — and
# wrong for this one. This project writes long comments deliberately, and every
# correct site in the tree separates its colour from its plate by more than
# four lines: `ribbon::tabs` by twelve, `ribbon::mode_selector` by forty. A
# window tuned to pass those would be wider than most functions and would
# assert nothing.
#
# A function is the honest unit because it is the scope a reader checks. If the
# plate is painted in the same function, the pairing is visible; if it is in
# another function, it is exactly the kind of at-a-distance coupling that
# produced all three D2 incidents.
#
# ★ The theme module itself is excluded. It DEFINES the role — the field, the
# per-preset values, the `widgets.active` assignment and the test that pins
# `on_accent` inverting against `accent` — and a definition is not a drawing.
# Excluded by path rather than by a marker on twelve lines, because "this file
# is where the palette lives" is a fact about the file.
#
# ═══════════════════════════════════════════════════════════════════════════
# ★ THE NON-VACUITY EVIDENCE IS REAL SITES, NOT A PLANTED ONE
# ═══════════════════════════════════════════════════════════════════════════
#
# When this gate was written there were seven `on_accent` drawing sites. Four
# stated their plate and three did not, and the three that did not were exactly
# the three defects above. A planted violation would have proved less: it would
# have shown the pattern matches, where the real distribution showed the rule
# discriminates. Both halves are asserted by `--self-test`.
#
# ═══════════════════════════════════════════════════════════════════════════
# USAGE
# ═══════════════════════════════════════════════════════════════════════════
#   tools/gates/check-plate-colour.sh              scan the tree
#   tools/gates/check-plate-colour.sh --self-test  prove it can fail
#
#   0  clean · 1  a violation, or the self-test did not detect its plant

set -uo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
cd "$ROOT" || exit 1

# A line is exempt if it says so, with a reason.
EXEMPT='plate-colour-exempt:'

scan() {
    local root="$1"
    local found=0
    local files
    files=$(find "$root" -name '*.rs' -type f 2>/dev/null | sort)
    [ -z "$files" ] && return 2

    while IFS= read -r file; do
        # ★ The theme module defines the role; a definition is not a drawing.
        case "$file" in
            */theme/*) continue ;;
        esac

        local hits
        hits=$(awk -v exempt="$EXEMPT" '
            # Function boundaries, found by the `fn` keyword at any indent. Crude
            # and sufficient: this is a scope for a proximity question, not a
            # parser. A nested closure inside a function counts as part of it,
            # which is what a reader would say too.
            /(^|[^A-Za-z_])fn[ \t]+[A-Za-z_]/ { fstart[++nf] = NR; fname[nf] = $0 }
            { line[NR] = $0 }
            END {
                fstart[nf + 1] = NR + 1
                for (i = 1; i <= NR; i++) {
                    if (line[i] !~ /on_accent/) continue
                    if (line[i] ~ exempt) continue
                    s = line[i]; sub(/^[ \t]+/, "", s)
                    # Comments explain; they do not draw.
                    if (s ~ /^\/\// || s ~ /^\*/) continue

                    # Which function is this line in?
                    k = 0
                    for (n = 1; n <= nf; n++) if (fstart[n] <= i) k = n
                    lo = (k ? fstart[k] : 1)
                    hi = (k ? fstart[k + 1] - 1 : NR)

                    paired = 0
                    for (j = lo; j <= hi; j++) {
                        t = line[j]; sub(/^[ \t]+/, "", t)
                        if (t ~ /^\/\// || t ~ /^\*/) continue
                        if (t ~ /accent_pair|\.fill\(|rect_filled|palette\.accent/) {
                            paired = 1; break
                        }
                    }
                    if (!paired) printf "%d:%s\n", i, line[i]
                }
            }
        ' "$file")
        if [ -n "$hits" ]; then
            while IFS= read -r hit; do
                printf '  %s:%s\n' "$file" "$hit"
                found=$((found + 1))
            done <<< "$hits"
        fi
    done <<< "$files"
    return $((found > 0 ? 1 : 0))
}

if [ "${1:-}" = "--self-test" ]; then
    tmp=$(mktemp -d)
    trap 'rm -rf "$tmp"' EXIT
    mkdir -p "$tmp/src"

    # (1) A violation: a plate colour with no plate anywhere near it. This is
    #     `tabstrip`'s close glyph as it shipped.
    cat > "$tmp/src/bad.rs" <<'RS'
fn draw(ui: &mut egui::Ui, theme: &Theme, selected: bool) {
    let colour = if selected { theme.palette.on_accent } else { theme.palette.text_muted };
    ui.add(egui::Button::new(RichText::new("x").color(colour)).frame(false));
}
RS

    # (2) Correct, by a fill on the same widget — `ribbon::tabs`' shape.
    cat > "$tmp/src/filled.rs" <<'RS'
fn draw(ui: &mut egui::Ui, theme: &Theme) {
    let text = RichText::new("Tab").color(theme.palette.on_accent);
    let button = egui::Button::new(text).fill(theme.palette.accent);
    ui.add(button);
}
RS

    # (3) Correct, by a painted plate ABOVE the colour choice — the shape a
    #     forward-only scanner would report. This case is why the window looks
    #     both ways.
    cat > "$tmp/src/painted.rs" <<'RS'
fn draw(ui: &mut egui::Ui, theme: &Theme, rect: Rect) {
    ui.painter().rect_filled(rect, 0.0, theme.palette.accent);
    let text = RichText::new("Tab").color(theme.palette.on_accent);
    ui.label(text);
}
RS

    # (4) Correct, by the named pairing accessor.
    cat > "$tmp/src/paired.rs" <<'RS'
fn draw(ui: &mut egui::Ui) {
    let (fill, text) = Theme::accent_pair(ui.ctx());
    let _ = (fill, text);
    let _unused = "on_accent is named only in this comment-free line for the test";
}
RS

    # (5) Exempt, with a reason.
    cat > "$tmp/src/exempt.rs" <<'RS'
fn draw(ui: &mut egui::Ui, theme: &Theme) {
    let c = theme.palette.on_accent; // plate-colour-exempt: handed to a caller that fills
    let _ = c;
}
RS

    out=$(scan "$tmp/src")
    rc=$?

    fail=0
    if [ "$rc" -ne 1 ]; then
        echo "plate-colour --self-test: FAIL — the planted violation was not detected."
        fail=1
    fi
    if ! printf '%s' "$out" | grep -q 'bad.rs'; then
        echo "plate-colour --self-test: FAIL — an unpaired plate colour was not reported."
        fail=1
    fi
    for ok in filled painted paired exempt; do
        if printf '%s' "$out" | grep -q "$ok.rs"; then
            echo "plate-colour --self-test: FAIL — $ok.rs is correct and was reported."
            echo "  A gate that reports the correct shape trains people to ignore it,"
            echo "  which is worse than not having the gate."
            fail=1
        fi
    done
    [ "$fail" -ne 0 ] && exit 1
    echo "plate-colour --self-test: PASS — catches the unpaired plate, passes all four correct shapes."
    exit 0
fi

echo "check-plate-colour: scanning for on_accent drawn without its plate…"
out=$(scan "crates")
rc=$?

if [ "$rc" -eq 2 ]; then
    echo "check-plate-colour: SKIPPED — no .rs files under crates/."
    exit 2
fi
if [ "$rc" -eq 1 ]; then
    echo "check-plate-colour: FAIL — on_accent drawn with no plate stated nearby:"
    printf '%s\n' "$out"
    cat <<'MSG'

`Palette::on_accent` is *"text and icons drawn ON `accent`"*. On any other
background it is a pale glyph on a pale surface — present, correctly sized, and
invisible. That is DEFECTS.md D2, and it has now shipped three times.

State the plate in the SAME FUNCTION: `.fill(theme.palette.accent)` on the
widget, a `rect_filled(.., theme.palette.accent)` behind it, or
`Theme::accent_pair(ctx)`, which supplies both and has no plausible wrong use.

★ Do NOT reach for `visuals.selection.bg_fill` as the plate. `egui` substitutes
it for you on `Button::selected(true)`, and this theme points it at the CANVAS
object-selection tint — 27 % alpha, so it composites to a different colour over
every different background and is a wash rather than a plate.

If a line genuinely hands the colour somewhere else that fills, say so on it
with `plate-colour-exempt:` and the reason.
MSG
    exit 1
fi
echo "check-plate-colour: clean — every on_accent states its plate."
exit 0
