#!/usr/bin/env bash
#
# check-strong-text.sh — `RichText::strong()` must take its colour back.
#
# WHAT THIS GATE IS FOR
# =====================
#
# `DEFECTS.md` D11 ends with a rule stated in the imperative:
#
#     Do not use `RichText::strong()` in this application. There is no colour
#     it can resolve to that is correct on both an accent fill and a panel.
#
# It was written on 2026-08-14 after six labels shipped near-invisible, and it
# was enforced by nothing. On 2026-08-17 the Settings window was built and its
# group headings and setting titles used `.strong()` — and rendered pale grey
# on pale grey, while the radio labels beneath them read normally.
#
# That is the seventh instance of a defect whose rule was already written down,
# in a file whose whole purpose is to stop repeats. **A rule that lives only in
# a document is enforced exactly as often as someone remembers to read it**,
# which is the same finding `check-theme-colors.sh`'s own header records about
# arriving late, and the reason this gate exists rather than a stronger warning.
#
# THE MECHANISM, BECAUSE THE RULE LOOKS ARBITRARY WITHOUT IT
# ==========================================================
#
# `egui` has no separate role for emphasised text. From its `style.rs`:
#
#     pub fn strong_text_color(&self) -> Color32 {
#         self.widgets.active.text_color()   // widgets.active.fg_stroke.color
#     }
#
# So `.strong()` borrows the **active widget** foreground. `egui-shell`'s theme
# sets that to `palette.on_accent`, which is correct and necessary — the active
# state is the accent-FILLED one, and text on an accent fill must be
# `on_accent`. Every preset this project ships fills its active state with the
# accent.
#
# Therefore `.strong()` on an ordinary panel resolves to a colour chosen for a
# background the text is not on. It also survives `override_text_color`.
#
# WHAT IS ALLOWED, AND WHY IT IS NOT AN ESCAPE HATCH
# ==================================================
#
# `.strong()` followed by an explicit `.color(...)` is fine, because the colour
# is then the caller's rather than `egui`'s inference — the weight is kept and
# the broken half is taken back. Both legitimate uses in the workspace are this
# shape, and both are correct for a stronger reason than convenience:
#
#   egui-shell/src/ribbon/tabs.rs   selected ribbon tab
#   egui-shell/src/dock/tabs.rs     selected dock tab
#
# Both are drawn ON the accent fill, so `on_accent` is the right colour anyway,
# and both state it rather than inheriting it. R84 is why they want the weight
# at all: selected state is never colour alone, because weight is the cue that
# survives greyscale and colour-vision deficiency.
#
# ★★★ THAT SENTENCE WAS FALSE FOR ONE OF THE TWO UNTIL 2026-09-03, and this
# gate was blessing the site on it.
#
# `ribbon/tabs.rs` fills: `Button::selectable(...).fill(accent)`. `dock/tabs.rs`
# contained **no `.fill(` at all** — it passed `.selected(true)` and let
# `egui::Style::button_style` choose, which takes the fill from
# `visuals.selection.bg_fill`. This theme points that at a 27 %-alpha CANVAS
# tint, so the "accent fill" this paragraph asserted did not exist and
# `on_accent` was landing on a pale wash: a luminance gap of 44.8 / 28.2 / 52.6
# across the three presets, against a readable floor of 90.
#
# Both files now state their fill and the sentence above is true of both. It is
# kept, with this correction beneath it, because the useful record is not that
# the rule was right — it is that **a gate can pass a site for a reason that
# stopped being true, and the passing tells you nothing about the reason.**
#
# The window is deliberately narrow — the SAME statement, or the next two
# lines. A `.color()` five lines away is not a pairing a reader can see, and
# this gate's job is to make the pairing visible, not merely present.
#
# WHAT THIS GATE CANNOT SEE
# =========================
#
# It reads text, not a syntax tree, so it cannot tell a `.strong()` in a string
# literal from one in a call. That is acceptable here and it is not acceptable
# for `shell::commands::reach` — the difference is the direction of the error.
# A false positive here costs a comment rewrite; a false NEGATIVE in a
# reachability check is a shipped inert control. Doc comments and line comments
# are stripped first, which removes every occurrence in the tree today.

set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT" || exit 2

SELF_TEST=0
[ "${1:-}" = "--self-test" ] && SELF_TEST=1

# ---------------------------------------------------------------------------
# The scan.
#
# For every `.strong()` outside a comment, look at the two lines BEFORE it, its
# own line, and the two AFTER, for an explicit `.color(`.
#
# Two lines rather than one because `rustfmt` breaks a builder chain across
# lines, so the `.color()` that pairs with a `.strong()` is routinely on the
# next line — and occasionally the one after, when a conditional sits between
# them.
#
# BOTH directions, and the backward half was added by the gate's own first real
# run. `egui-shell`'s ribbon tab states the colour and then applies the weight
# inside a nested `if` — the *better* shape, because it makes the weight
# unreachable without the colour structurally rather than by two sibling
# statements staying adjacent — and a forward-only window called it a
# violation. A gate that refuses the safest available pattern trains people to
# work around the gate.
#
# ★ AND THE WINDOW IS COUNTED IN CODE LINES, NOT SOURCE LINES.
#
# The same run found the second half of that. Once comments are stripped, this
#
#     text = text.color(ctx.theme.palette.on_accent);
#     if cues.emphasised_text {
#         // four lines of comment explaining exactly why this is safe
#         text = text.strong();
#     }
#
# has its `.color()` SIX source lines above its `.strong()` and one *statement*
# above it. Measuring source lines would mean a well-documented pairing fails a
# gate that a terse one passes — which is precisely backwards for this project,
# and would push the next person to delete the explanation to appease the tool.
#
# So blank and comment-only lines are dropped before the window is taken, and
# the original line number is carried alongside so a violation still reports
# where it actually is.
# ---------------------------------------------------------------------------
scan() {
    local root="$1"
    local violations=0
    while IFS= read -r file; do
        # Strip doc comments and line comments before looking. Every mention of
        # `.strong()` in this tree that is NOT a call is one of these — five
        # panels carry a comment saying they deliberately do not use it.
        local stripped
        stripped="$(sed -e 's://!.*::' -e 's:///.*::' -e 's://.*::' "$file")"
        local raw
        mapfile -t raw <<< "$stripped"

        # Keep only lines with code on them, remembering where each came from.
        # `code[k]` is the text; `where[k]` is its 1-based line in the FILE, so
        # a violation is reported at its real position even though the window
        # is measured over statements.
        local code=() where=()
        local i
        for i in "${!raw[@]}"; do
            case "${raw[$i]}" in
                *[![:space:]]*)
                    code+=("${raw[$i]}")
                    where+=("$((i + 1))")
                    ;;
            esac
        done

        local k
        for k in "${!code[@]}"; do
            case "${code[$k]}" in
                *".strong()"*) ;;
                *) continue ;;
            esac
            local lo=$((k - 2))
            [ "$lo" -lt 0 ] && lo=0
            local window="" j
            for j in $(seq "$lo" $((k + 2))); do
                window="$window${code[$j]:-}"
            done
            case "$window" in
                *".color("*) continue ;;
            esac
            echo "  $file:${where[$k]}: .strong() with no explicit .color() nearby"
            violations=$((violations + 1))
        done
    done < <(find "$root" -name '*.rs' -not -path '*/target/*' 2>/dev/null)
    return "$violations"
}

# ---------------------------------------------------------------------------
# Self-test — the same discipline the other gates hold themselves to.
#
# A gate that has only ever been seen to pass is indistinguishable from a gate
# that cannot fail. This project has a recorded incident of exactly that: a
# deliberately planted violation that `check-ui-strings.sh` failed to detect,
# briefly making it look as though a fix had produced a working gate.
# ---------------------------------------------------------------------------
if [ "$SELF_TEST" = "1" ]; then
    tmp="$(mktemp -d)"
    trap 'rm -rf "$tmp"' EXIT
    mkdir -p "$tmp/dirty" "$tmp/clean"

    cat > "$tmp/dirty/bad.rs" <<'EOF'
fn draw(ui: &mut Ui) {
    ui.label(RichText::new("Appearance").strong());
}
EOF
    cat > "$tmp/clean/good.rs" <<'EOF'
// A comment mentioning .strong() must not trip the gate.
/// Nor must a doc comment saying `.strong()` is unusable — DEFECTS.md D11.
fn draw(ui: &mut Ui, theme: &Theme) {
    ui.label(RichText::new("Tab").strong().color(theme.palette.on_accent));
    let text = RichText::new("Split")
        .strong()
        .color(theme.palette.on_accent);
    ui.label(text);
    ui.label(RichText::new("Plain"));
}
EOF

    fail=0
    if scan "$tmp/dirty" > /dev/null; then
        echo "SELF-TEST FAILED: a bare .strong() was not detected"
        fail=1
    fi
    if ! scan "$tmp/clean" > /dev/null; then
        echo "SELF-TEST FAILED: a paired .strong()/.color(), or a comment, was reported"
        scan "$tmp/clean"
        fail=1
    fi
    if [ "$fail" = "0" ]; then
        echo "check-strong-text --self-test: OK"
        exit 0
    fi
    exit 1
fi

echo ">> check-strong-text"
found=0
for dir in crates tools; do
    [ -d "$dir" ] || continue
    scan "$dir" || found=$((found + $?))
done

if [ "$found" != "0" ]; then
    cat <<'EOF'

`RichText::strong()` resolves to `widgets.active.fg_stroke` — the foreground of
the ACCENT-FILLED widget state — because egui has no separate role for
emphasised text. On an ordinary panel that is pale text on a pale background,
and it survives `override_text_color`.

DEFECTS.md D11 records six labels that shipped this way, and the Settings
window's headings that repeated it three days after the rule was written.

Two ways to fix it, in order of preference:

  1. Drop `.strong()`. In every observed case the emphasis it asked for was
     invisible anyway, so the label reads BETTER without it. Carry the
     hierarchy in layout and wording — a disclosure triangle, or being the one
     line in a group that is not `.small().weak()`.

  2. If the text really is drawn ON the accent fill — a selected tab, a filled
     button — keep `.strong()` for the weight R84 asks for and state the colour
     explicitly on the same statement or the next line:

         RichText::new(label).strong().color(theme.palette.on_accent)

EOF
    exit 1
fi

echo "   no bare .strong() found"
exit 0
