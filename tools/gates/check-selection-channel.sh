#!/usr/bin/env bash
#
# check-selection-channel.sh — `visuals.selection` belongs to egui's WIDGETS.
#
# ═══════════════════════════════════════════════════════════════════════════
# WHAT THIS GATE IS FOR
# ═══════════════════════════════════════════════════════════════════════════
#
# `egui::Visuals::selection` is `egui`'s styling channel for **selected
# widgets**. Its own doc comment says "Selected text, selected elements etc",
# and `egui` reads it in exactly the places that sentence implies:
#
#   · `Style::button_style`, `SELECTED_CLASS` branch — `widget_style.rs:151-154`
#   · the selected-text highlight — `text_selection/visuals.rs:39-40`
#   · the progress bar's fill and its label — `widgets/progress_bar.rs:159,199`
#   · the slider's trail — `widgets/slider.rs:802`
#   · a focused `TextEdit`'s frame stroke — `widgets/text_edit/builder.rs:704`
#
# ★★★ THIS PROJECT HANDED THAT CHANNEL TO THE CANVAS, AND THE CANVAS WON.
#
# `REVIEW_TRIAGE.md` §2b defect **T2**. `egui-shell`'s theme set
# `selection.bg_fill` to `palette.selection_fill` — a 27 %-alpha tint whose job
# is washing a selected object on a drawing — and `selection.stroke` to
# `palette.accent`, the ink the canvas outlines things with. Thirty-three
# readers across thirteen files depended on those two values — every one of
# them a content-area painter except one glyph tint, which now goes through a
# named accessor like the rest (see "there is no file-level exemption").
#
# The consequence was mechanical and it was everywhere. `Style::button_style`
# takes BOTH fills *and the text colour* from that channel for anything drawn
# with `Button::selected(true)` or `ui.selectable_label(true, …)`, so nineteen
# chrome controls — ribbon toggles, menu rows, eight panels' list selections —
# painted accent-coloured text on a 27 % wash. Measured luminance gap in the
# Dark preset: **72.5**, against this project's own readable floor of 90.
#
# Not one of those call sites was wrong. They asked `egui` for "selected"; the
# theme was answering with the wrong pair.
#
# ═══════════════════════════════════════════════════════════════════════════
# ★★ WHY A GREP, WHEN THE PROJECT ALREADY HAS A PERCEPTUAL GATE
# ═══════════════════════════════════════════════════════════════════════════
#
# `theme::contrast::pairs` enumerates `WidgetState::ALL` × `FillKind::ALL` —
# ten pairs, foreground always `fg_stroke.color`, background always one of the
# state's own fills. **The selected pair is in none of them**, because `egui`
# does not store it: it is substituted at paint time, after the `Style` has
# been read. A gate that reads a `Style` back cannot reach a pair that was
# never in the struct.
#
# And `check-theme-colors.sh` had nothing to say either: both values were
# correctly sourced from the palette. Its rule is *"no invented colours"*; it
# cannot express *"the right role"*. That is the same hole `check-plate-colour`
# was written for, one channel over. RESUME.md states it in one line:
# **`check-theme-colors.sh` forbids invented values, NOT wrong roles.**
#
# ⇒ A perceptual gate answers *"is this pair readable"*. This one answers
# *"is this code reading the channel that belongs to it"*, which is the
# question that was actually going wrong.
#
# ═══════════════════════════════════════════════════════════════════════════
# THE RULE
# ═══════════════════════════════════════════════════════════════════════════
#
# **No source file outside `crates/egui-shell/src/theme/` may READ
# `visuals().selection`, `visuals.selection`, or any `.selection.bg_fill` /
# `.selection.stroke` through a binding that holds `egui::Visuals`.**
#
# The theme module is excluded because it DEFINES the channel — it is the one
# place whose job is to decide what `egui` will paint a selected widget with.
# Excluded by path rather than by a marker on the assignment lines, because
# "this directory is where the style is written" is a fact about the directory
# (the same reasoning `check-plate-colour.sh` uses for `on_accent`).
#
# What a canvas or a panel should say instead, and the whole point of the fix:
#
#   · `Theme::canvas_selection_ink(ctx)`   — the content-area outline colour
#   · `Theme::canvas_selection_fill(ctx)`  — the content-area wash
#   · `Theme::canvas_selection_pair(ctx)`  — both, for a washed-and-outlined shape
#
# and what a piece of CHROME should say:
#
#   · nothing at all — `egui` already styles a selected widget correctly;
#   · `Theme::accent_pair(ctx)` when it states its own fill and ink; or
#   · `Theme::selected_widget_ink(ctx)` / `Theme::selected_widget_pair(ctx)`
#     when it hand-draws a mark ON the plate `egui` painted — a tinted glyph
#     on a toggle, a custom check mark. That is the one case that used to need
#     the raw read, and it is now the *only* thing those two accessors are for.
#
# ★ Comments are not readings. This project writes long explanatory comments
# on purpose, and several of them quote the old spelling in order to say why
# it was wrong — including this gate's own reason for existing. A gate that
# forbade a file from *explaining itself* would be paid for in deleted
# explanations, which is a bad trade.
#
# ═══════════════════════════════════════════════════════════════════════════
# ★★★ THERE IS NO FILE-LEVEL EXEMPTION. THERE WAS ONE, AND CLOSING IT IS
# WORTH MORE THAN THE LINE IT SAVED.
# ═══════════════════════════════════════════════════════════════════════════
#
# When this gate was written it exempted one path outright:
# `crates/pdfcer-gui/src/icons/mod.rs`, whose `icons::selected_image` tinted a
# toggle's glyph with `ui.visuals().selection.stroke.color`. The read was
# CORRECT — the glyph sits on the fill `egui` paints behind a selected toggle,
# so that channel's ink is exactly right for it — and the file was left alone
# only because another track owned that directory that day.
#
# ★★ THE EXEMPTION'S OWN PREMISE EXPIRED WITHIN A DAY, WHICH IS THE ARGUMENT.
# It was written on the ground that "that channel carries `accent` +
# `on_accent`". Within twenty-four hours the channel was re-pointed a second
# time — to `selected_plate` + `accent` — because `on_accent` there had made
# the focused-`TextEdit` frame stroke `egui` draws from the SAME field
# unreadable (luminance gaps 17.9 / 5.0 / 29.1, floor 90). The exempted line
# would have gone on compiling, gone on passing, and gone on tinting a glyph
# with whatever the channel happened to hold.
#
# That is `REVIEW_TRIAGE.md` **T1** in one file: `check-strong-text.sh` blessed
# two sites on the stated ground that *"both are drawn ON the accent fill, so
# `on_accent` is the right colour anyway"* — true of one of them, silently
# false of the other, which passes `.selected(true)` and lets `egui` choose.
# The gate went on passing a defective site for a reason that had expired,
# which is why A15a shipped in all three presets.
#
# So the line was converted rather than re-blessed. `selected_image` now calls
# `Theme::selected_widget_ink(ui.ctx())`, and
# `egui_shell::theme::tests::the_selected_widget_accessors_agree_with_the_style_egui_will_paint`
# asserts that accessor equals `visuals.selection.stroke.color` in every
# preset. Identical pixels; the difference is that the promise is now checked
# by a test instead of by a comment in a gate.
#
# ⇒ **The gate has no holes.** Only two escapes remain, both narrow and both
# self-announcing: the theme directory, which DEFINES the channel, and a
# per-line `selection-channel-exempt:` marker with a reason written on the
# line. If a future case genuinely needs a third, the bar is the one that was
# just failed: a written reason AND a Rust assertion that pins its premise —
# and note that even that bar did not hold here.
#
# ═══════════════════════════════════════════════════════════════════════════
# ★ NON-VACUITY
# ═══════════════════════════════════════════════════════════════════════════
#
# `--self-test` plants two violations in the two shapes that actually shipped —
# the direct `ui.visuals().selection.stroke.color` read and the one through a
# `let v = ui.visuals()` binding — and four correct shapes it must NOT report:
# the named accessor, an explanatory comment, an exempted line, and a file in
# the theme directory. A gate that reports a correct shape trains people to
# ignore it, which is worse than not having the gate.
#
# The stronger evidence is historical rather than planted. Run against the tree
# as it stood before 2026-09-04 (`git grep` over `HEAD`, comments filtered the
# same way this gate filters them), it names **33 readings in 13 files** — 28
# of `selection.stroke` and 5 of `selection.bg_fill`, 32 in `pdfcer-gui` and
# one in `egui-shell`'s own `ribbon::mode_selector`. Thirty-two of the 33 were
# converted by the fix this gate arrived with, and the 33rd — the glyph tint —
# was converted the following day when the exemption was closed, so the gate
# now runs with **zero** file-level escapes against a tree that once had 33
# violations. A planted violation would have shown only that the pattern
# matches; the real distribution shows the rule discriminates.
#
# ═══════════════════════════════════════════════════════════════════════════
# USAGE
# ═══════════════════════════════════════════════════════════════════════════
#   tools/gates/check-selection-channel.sh              scan the tree
#   tools/gates/check-selection-channel.sh --self-test  prove it can fail
#
#   0  clean · 1  a violation, or the self-test did not detect its plant
#   2  SKIPPED — no .rs files were found, so nothing was checked

set -uo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
cd "$ROOT" || exit 1

# A line is exempt if it says so, with a reason.
EXEMPT='selection-channel-exempt:'

# The needles. Two spellings, because both shipped:
#   · `visuals().selection` / `visuals.selection` — the direct read;
#   · `.selection.bg_fill` / `.selection.stroke`  — the read through a binding
#     (`let v = ui.visuals(); … v.selection.bg_fill`), which the first pattern
#     cannot see and which is the shape the theme module itself uses.
NEEDLE='visuals\(\)\.selection|visuals\.selection|\.selection\.bg_fill|\.selection\.stroke'

# scan <root> — print every offending `file:line:text`.
#   0 clean · 1 at least one violation · 2 no .rs files at all
scan() {
    local root="$1"
    local found=0
    local files
    files=$(find "$root" -name '*.rs' -type f 2>/dev/null | sed 's#^\./##' | sort)
    [ -z "$files" ] && return 2

    while IFS= read -r file; do
        # The theme module DEFINES the channel; a definition is not a reading.
        case "$file" in
            */egui-shell/src/theme/*) continue ;;
            # The self-test's miniature of that directory.
            */theme/*) continue ;;
        esac
        local hits
        hits=$(grep -nE "$NEEDLE" "$file" 2>/dev/null | while IFS= read -r hit; do
            # `grep -n` over ONE file prints `LINENO:TEXT`, so exactly one
            # colon is the prefix. ★ Stripping twice — the shape a reader
            # expects from `grep -n` over a file LIST — silently ate everything
            # up to the next `::` in the line, which is `overlay::ink` in half
            # the comments in this tree. It reported six explanatory comments
            # as violations on its first run.
            local text="${hit#*:}"
            # Strip leading whitespace for the comment test.
            local trimmed="${text#"${text%%[![:space:]]*}"}"
            case "$trimmed" in
                //*|\**) continue ;;          # a comment explains; it does not draw
            esac
            case "$text" in
                *"$EXEMPT"*) continue ;;      # exempt, with a reason on the line
            esac
            printf '%s\n' "$hit"
        done)

        if [ -n "$hits" ]; then
            while IFS= read -r hit; do
                [ -z "$hit" ] && continue
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
    mkdir -p "$tmp/src" "$tmp/src/theme"

    # (1) A violation, direct: the shape every canvas painter shipped with.
    cat > "$tmp/src/bad_direct.rs" <<'RS'
fn draw(ui: &egui::Ui, painter: &egui::Painter, rect: egui::Rect) {
    let stroke = egui::Stroke::new(1.5, ui.visuals().selection.stroke.color);
    painter.rect_stroke(rect, 0.0, stroke, egui::StrokeKind::Middle);
}
RS

    # (2) A violation through a BINDING, which the `visuals.selection` pattern
    #     alone would miss. This is the `panels/pages` thumbnail shape.
    cat > "$tmp/src/bad_binding.rs" <<'RS'
fn draw(ui: &egui::Ui, rect: egui::Rect) {
    let v = ui.visuals().clone();
    ui.painter().rect_filled(rect, 2.0, v.selection.bg_fill);
}
RS

    # (3) Correct: the purpose-named accessor.
    cat > "$tmp/src/named.rs" <<'RS'
fn draw(painter: &egui::Painter, rect: egui::Rect) {
    let ink = egui_shell::theme::Theme::canvas_selection_ink(painter.ctx());
    let fill = egui_shell::theme::Theme::canvas_selection_fill(painter.ctx());
    painter.rect_filled(rect, 0.0, fill);
    painter.rect_stroke(rect, 0.0, egui::Stroke::new(1.0, ink), egui::StrokeKind::Middle);
}
RS

    # (4) Correct: a comment that quotes the old spelling in order to explain
    #     why it is wrong. This project writes those on purpose, and a gate
    #     that forbade them would be paid for in deleted explanations.
    cat > "$tmp/src/comment.rs" <<'RS'
/// The colour is the theme's content-area ink.
///
/// It was `visuals.selection.stroke.color` until T2 — `egui`'s selected-WIDGET
/// channel, which this theme had handed to the canvas. Same value, new name.
fn draw() {}
RS

    # (5) Exempt, with a reason on the line.
    cat > "$tmp/src/exempt.rs" <<'RS'
fn tint(ui: &egui::Ui) -> egui::Color32 {
    ui.visuals().selection.stroke.color // selection-channel-exempt: drawn on the selected fill egui paints
}
RS

    # (6) Correct: the theme module, which DEFINES the channel.
    cat > "$tmp/src/theme/mod.rs" <<'RS'
fn write_style(style: &mut egui::Style, p: &Palette) {
    let v = &mut style.visuals;
    v.selection.bg_fill = p.selected_plate;
    v.selection.stroke = egui::Stroke::new(1.0, p.accent);
}
RS

    out=$(scan "$tmp/src")
    rc=$?

    fail=0
    if [ "$rc" -ne 1 ]; then
        echo "selection-channel --self-test: FAIL — the planted violations were not detected."
        fail=1
    fi
    for bad in bad_direct bad_binding; do
        if ! printf '%s' "$out" | grep -q "$bad.rs"; then
            echo "selection-channel --self-test: FAIL — $bad.rs reads the widget channel and was not reported."
            fail=1
        fi
    done
    for ok in named comment exempt mod; do
        if printf '%s' "$out" | grep -q "$ok.rs"; then
            echo "selection-channel --self-test: FAIL — $ok.rs is correct and was reported."
            echo "  A gate that reports the correct shape trains people to ignore it,"
            echo "  which is worse than not having the gate."
            fail=1
        fi
    done
    [ "$fail" -ne 0 ] && exit 1
    echo "selection-channel --self-test: PASS — catches both readings, passes all four correct shapes."
    exit 0
fi

echo "check-selection-channel: scanning for canvas code reading egui's widget channel…"
out=$(scan "crates"; )
rc=$?
if [ "$rc" -ne 2 ]; then
    tools_out=$(scan "tools")
    trc=$?
    if [ "$trc" -eq 1 ]; then
        out="$out
$tools_out"
        rc=1
    fi
fi

if [ "$rc" -eq 2 ]; then
    echo "check-selection-channel: SKIPPED — no .rs files under crates/."
    exit 2
fi
if [ "$rc" -eq 1 ]; then
    echo "check-selection-channel: FAIL — egui's selected-WIDGET channel is being read outside the theme:"
    printf '%s\n' "$out"
    cat <<'MSG'

`egui::Visuals::selection` is how `egui` styles a SELECTED WIDGET. It supplies
both fills AND the text colour for every `Button::selected(true)` and every
`ui.selectable_label(true, …)` in the application — `widget_style.rs:151-154`.
It is not a canvas role, it is not a general-purpose accent, and it is not a
focus-ring colour.

Reading it from content code is how defect T2 happened: the theme pointed the
channel at the canvas to satisfy ~33 readers like this one, and every selected
chrome control in the application was then painted with canvas ink — accent
text on a 27 % wash, luminance gap 72.5 in the Dark preset against a floor
of 90. Every gate stayed green, because every colour involved was correctly
sourced from the palette. Correctly sourced, wrong role.

Say which role you actually want:

  · content area, an outline/grip/caret  → Theme::canvas_selection_ink(ctx)
  · content area, a wash/tint            → Theme::canvas_selection_fill(ctx)
  · content area, both at once           → Theme::canvas_selection_pair(ctx)
  · chrome, emphasised action or plate   → Theme::accent_pair(ctx)
  · chrome, an ordinary selected widget  → say nothing; egui already does it
  · chrome, a mark drawn ON the selected
    plate egui painted (a tinted glyph)  → Theme::selected_widget_ink(ctx)
                                           Theme::selected_widget_pair(ctx)

The last one is what the single historical exemption became. There is no
file-level exemption left. A per-line `selection-channel-exempt:` marker with
a written reason still exists, but the bar for using it is high: the previous
blessing's premise ("the channel carries accent + on_accent") expired within a
day, so a reason in prose is not enough — pin it with a Rust assertion too.
MSG
    exit 1
fi
echo "check-selection-channel: clean — the widget channel is read only where it is defined."
exit 0
