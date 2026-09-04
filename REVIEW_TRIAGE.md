# Outside GUI review, 2026-09-03 — triage

**Source:** `D:\Dev\FeatureRequests\pdfcer-gui\REVIEW.md`, from
`D:\Dev\pdfcer-gui-consultant`, against
`pdfcergui-20260903-1605-e27c3b4-49b4b4b-dirty`. Twenty findings, a design
direction ("Board"), 80 screenshots, 65 proposed icons.

**Method.** Every finding was read against the source by a separate agent and
classified. Nothing here is an impression: each row names the mechanism at
`file:line` and, where the project already ruled, quotes the ruling.

**Standing:** `RIBBON_IA.md` is settled and was reviewed by the operator. A
finding that contradicts it is an **amendment for him to rule on**, not a defect
to fix. Those are collected in §3 and none has been actioned.

---

## 1. Fixed this session

| # | What | Where |
|---|---|---|
| **A1** | **The crash.** `Host::show` took the body as `FnOnce` and panicked on egui's permitted second call. Now `FnMut`. | `dialogs/host.rs` |
| **A1b** | **The harness reported PASS on the crashing build** — the whole suite could. `Session::trace` now refuses a trace from a dead process, **fatally**, and all 152 `Err → skip` arms were rewritten. | `tools/ui-verify/src/launch.rs`, `error.rs`, `report.rs` |
| **A3b** | **A stuck status line, from an epoch type-confusion.** `page_texture_epoch` (a `PageEpochs` value since O74) was compared against `edit_epoch`. One edit on another sheet strands *"the picture is catching up"* for the session. | `app/state/heldpreview.rs` |
| **A4** | **The encrypted-file contradiction.** The canvas said the build could not prompt while the prompt was open. | `text/mod.rs`, `app/surfaces.rs` |
| **A5** | **Read mode offered authoring.** Add / Rename / Remove / Copy / Cut / drag hint, all gated on `authors_anything()`. | `panels/bookmarks/mod.rs` |
| **A9b** | **A disclosure triangle onto nothing.** Tab-order drew its header unconditionally, on every document with no form fields. | `panels/forms/tab_order/mod.rs` |
| **A15a** | **Selected dock tab: white on a pale wash.** Fixed by stating the fill — `dock/tabs.rs` contained no `.fill(` at all and let `egui` substitute the 27 % canvas tint. | `egui-shell/src/dock/tabs.rs` |
| **A15b** | **The document-tab close ✕, near-invisible when selected.** D2's exact shape. Fixed by painting the selected tab's plate across the WHOLE rect before it is split, so the ✕ — drawn `.frame(false)` into a carved-out sub-rect with nothing behind it — lands on `accent` like everything else in the tab. | `egui-shell/src/tabstrip/mod.rs` |
| **A15d** | **A new gate, whose evidence is real sites rather than a plant.** `check-plate-colour.sh`: `on_accent` may only be drawn where the same FUNCTION states its plate. When written, four of seven sites stated it and three did not — and the three were exactly the three D2 incidents. | `tools/gates/check-plate-colour.sh` |
| **A15e** | **The contrast gate could not see any of the three — now it sees the one a `Style` can express.** `contrast::pairs` went from **ten** pairs to **twenty-seven**: the widget matrix, plus every foreground `egui` resolves through a `Visuals` *accessor* (body, weak, strong, hyperlink, warn, error) on the grounds each is really drawn on, plus **both** roles of `visuals.selection`. ★ It caught a shipped pair on its first run — Dark's `error_fg_color` on `window_fill` at **89.7** against a floor of 90 — fixed at the ROLE (`danger` `#FF6B6B` → `#FF7B7B`, gap 102.3), not at the threshold. One exemption (`strong_text_color()`, which no theme value can satisfy) with an expiry test. The two defects a `Style`-level gate still cannot see — a caller's `RichText::color`, and a background chosen by geometry — are now named in the module header with the gates that DO cover them. | `egui-shell/src/theme/contrast.rs`, `theme/mod.rs`, `theme/tests.rs` |
| **A14b** | **The zoom readout reserved four characters and can be asked for fourteen.** `ZOOM_READOUT_WIDTH_PTS` documented itself as covering "the whole range `ZOOM_LADDER` can produce"; O24 made the ceiling a preference topping out at 1e12. Now measured per ceiling, floored at the old constant. ★ The review said seven characters — seven was the OLD ceiling's width. | `app/status.rs`, `app/status/zoom.rs` |
| **A18** | **Read mode showed two authoring controls its own dispatch refuses.** `format.select_form` and `format.unshare_form` got `!edit_content` guards on 2026-09-03; a guard stops the act and leaves the CONTROL. Both now `shown_when("mode.edit_content")`. | `shell/manifest/format.rs` |
| **A6** | **Set scale's "Show dimensions in" was a dead control on the ratio path** — and a green test was pinning it. `ScaleEntry::Ratio` has no unit field, so `self.unit` was discarded and the preview labelled inches while the dropdown said Metres. ★ No engine change was needed: `scale` carries the unit and `format.unit` only names it, so the conversion belongs on the scale. The row's suspicion of an engine ask was wrong. | `canvas/measure/scale.rs` |
| **Icons** | **Thirty-six glyphs adopted** from the review's sheet, after `stroke-dasharray` support landed (six of them draw the wrong picture without it). Four form tools and four measure tools stopped sharing one glyph each. See `GLYPH_ADOPTION.md`. | `icons/**`, `shell/commands/catalog/**` |

---

## 2. Confirmed, not yet fixed — the queue, in the order they cost a user

| # | Finding | Mechanism | Cost |
|---|---|---|---|
| **A15f** | **The OCR dialog's spinner is invisible in every preset.** `ui.spinner()` takes `strong_text_color()`, which IS `widgets.active.fg_stroke.color` — `on_accent`, the ink for the ACCENT FILL — and draws it on `window_fill`. Gaps **17.9 / 5.0 / 29.1**, floor 90: the same three numbers as the first, failed attempt at T2, because it is the same pair. Found by A15e's widening, which measures the pair and exempts it (no theme value reads on both grounds). `check-strong-text.sh` cannot see it — a bare `ui.spinner()` names no colour to check. | `dialogs/ocr.rs:599` | 1 call site |
| **A16c** | **The sticky-note dialog opens at the window origin.** `textannot.rs` computes a click-relative position and then `let _ = pos;` discards it. | `dialogs/host.rs:181`, `dialogs/textannot.rs:158` | 2 files |
| **A16a** | **The New-text-field dialog clips.** It *does* scroll; the default 440 × 420 is too small and egui's faint scrollbar hides the fact. | `dialogs/formfield.rs:163` | 1 file |
| **A12b** | **Quadding (`/Q`) is never read by the field editor.** The recorded refusal to make the editor a facsimile is an *arithmetic* argument about glyph advances — it does not reach text alignment or the grey background, so those are an unexamined gap rather than a decision. | `canvas/forms.rs:869-887` | 2 files |
| **A12c** | **The Fill-form panel does not update while you type on the page.** Two independent draft stores; commit is on focus loss only. The sibling-widget version of this lag is disclosed in the header; the panel one is not. | `panels/forms/rows.rs:531` | 3 files |
| **A11** | **About's headline is `Version 0.1.0` in a v0.5.0 release.** The crate version is deliberately not bumped (O110) — but that decision is about `Cargo.toml`, not about what About displays, and it was never carried through to the surface. Nothing in the tree carries the release version; `build.rs` would need to emit one. ⚠ The reviewer's "the title shows the minute, one or the other" is **wrong** — O101 changed it to the minute on the operator's own instruction. | `dialogs/about.rs:134` | 1 label + 1 build input |
| **A20x** | **Four praised behaviours have no regression test**: the Tool panel's Armed block, Settings' explanatory prose, the manual's redaction warning, and — the risky one — **Cancel really reverting a live theme change**, which is a one-line coupling whose failure is silent and whose driven check already opens the dialog and measures the window. | — | ~10 lines for the theme one |
| **T4** | ★★★ **Every driven assertion about a docked panel proves LAYOUT, not VISIBILITY.** `app/surfaces.rs:246` publishes every dock region through `diag::ui_rect`, while `diag::ui_rect_visible` — written for exactly this — is used elsewhere. The RAG's 2026-08-10 entry records three panels (Bookmarks, Layers, Signatures) shipping **unreachable** with a rail entry and every gate green. The channel cannot tell those apart today, so any check written for a new left rail is vacuous before it is written. Fixing it needs the shell's rect sink to carry the CLIP as well as the rect — `FnMut(&str, Rect)` → `FnMut(&str, Rect, Rect)` — which stays domain-free and keeps R7. | `app/surfaces.rs:246`, `egui-shell/src/dock/mod.rs:357` | 4 files |
| **T5** | **The harness cannot read rendered text.** No AccessKit, no OCR — only deliberate trace lines. The Objects panel publishes one aggregate whose `rows=` is a *layout* count, so a check that "the rows are richer" would be asserting against the thing it is testing. Any such check must bring its own oracle, and the pixel channel is the only non-circular one available. | — | a constraint, not a defect |
| **PartC** | **Two harness gaps, both one file**: the **Airy** preset is never driven, and no check samples the **page canvas** under a theme — only the window body. | `tools/ui-verify/src/checks/settings_theme.rs` | 1 file |

---

## 2b. Found while triaging — not in the review, and worse than most of it

Three defects the reviewer never saw, found by reading the code his findings
pointed at. Each is the same family: **a claim that was true when written, cited
later as justification, with nothing re-reading its premise.**

| # | The claim | Why it is false now |
|---|---|---|
| **T1** | `tools/gates/check-strong-text.sh:53` blesses two sites on the ground that *"Both are drawn ON the accent fill, so `on_accent` is the right colour anyway."* | True of `ribbon/tabs.rs`, which does `.fill(accent)` at `:443`. **False of `dock/tabs.rs`, which contains no `.fill(` at all** — it passes `.selected(true)` and lets egui choose, which takes the fill from the 27 % canvas tint. **The gate was passing the site for a reason that had stopped being true**, which is why A15a shipped in all three presets. |
| **T2** | ★★★ **Every bare `Button::selected(true)` / `selectable_label` renders accent text on the wash.** `egui::Style::button_style` overwrites the *text* colour too — `ws.text.color = self.visuals.selection.stroke.color` (`egui-0.35.0/src/widget_style.rs:153`, verified verbatim) — and this theme sets that stroke to `palette.accent`. | **~17 sites**, two in `egui-shell` (`menu/render.rs:459`, `ribbon/control.rs:218`) and ~15 `selectable_label` calls in `pdfcer-gui`. Measured gap in **Dark: 72.5** against a floor of 90. **The two sites the review found are the two that happened to be protected by an explicit colour**; the unprotected majority was never looked at. |
| **T3** | `tools/ui-verify/src/checks/driving.rs:807-810` derives a live threshold partly from *"egui's stock light palette — which is what the built binary actually paints with, **because nothing in `crates/pdfcer-gui` calls `Theme::apply`**"*. | `app/frame.rs:274` calls `theme.apply(&ctx)`. It has since D10 was fixed on 2026-08-14. ★ **The constant itself is safe** — its derivation deliberately covers *both* palettes and says so — so this is a false sentence rather than a wrong number. Correct it in place. |

★★ **And Airy is the worst preset for both reported contrast failures, which the
review did not measure**: gap **28.2** on the selected dock tab and **5.0** on
the close glyph — white on white to within five levels of luminance — because
Airy's panel is pure white and the 27 % wash barely darkens it.

★★★ **The root cause under all of it:** `egui::Visuals::selection` is doing
double duty. It is egui's styling channel for selected *widgets*, and this theme
has handed it to the *canvas* (`selection_fill` is the object-selection tint;
~30 readers depend on `selection.stroke` for canvas ink). The canvas won, so
**every selected chrome control in the application is painted with canvas ink**.
There is no theme-only fix — re-pointing `selection.bg_fill` at `accent` would
paint over page thumbnails (`panels/pages/mod.rs:877`). The fix is at the call
sites, and the gate's job is to make a bare `.selected(true)` impossible.

> ★★ **AMENDMENT, 2026-09-04 — the last sentence above was wrong, and the way
> it was wrong is worth keeping.** The fix *was* theme-only. The thumbnails
> were never at risk because the ~33 canvas readers do not need the channel;
> they need the two VALUES, which now arrive by name
> (`Theme::canvas_selection_ink` / `canvas_selection_fill`, bit-for-bit
> identical). "Re-pointing the channel breaks the canvas" quietly assumed the
> canvas would go on reading the channel.
>
> ★ Re-pointing it took **two** attempts, not one, and that is the durable
> lesson. `visuals.selection.stroke` is spent twice by `egui`: as the ink on a
> selected widget AND as the frame stroke of a focused, mutable `TextEdit`
> (`text_edit/builder.rs:699-706`, no `.frame_stroke()` override). The first
> attempt put `on_accent` there, satisfied the widget half, and made the focus
> ring near-white on a near-white panel — gaps **17.9 / 5.0 / 29.1**, D2's
> shape a fourth time, introduced by the fix for the third. The resolution was
> to dilute the **plate** (`Palette::selected_plate`) rather than the **ink**,
> which frees the ink to be `accent` and satisfies both grounds at once:
> **103.1 / 118.9 / 123.2** selected, **147.3 / 170.2 / 96.0** ring, floor 90.
> Pinned by
> `egui_shell::theme::tests::both_roles_the_selection_channel_serves_are_readable_in_every_preset`.
> The gate's one file-level exemption (`icons/mod.rs`) is closed; it now reads
> `Theme::selected_widget_ink`, and the gate has no holes.

---

## 3. Amendments to settled documents — the operator's call, none actioned

| # | The reviewer asks for | What it contradicts |
|---|---|---|
| **B/mock** | View ▸ Panels gains **Fonts** and **Comments** | **P1, "one command, one tab."** `RIBBON_IA.md:252-255` moved Fonts off View *on purpose*; `manifest/view.rs:47-53` resolved Comments to Markup in writing. ★ **The mock's ribbon does not load** — `no_command_appears_twice_on_the_tabs` and `Shell::validate` both refuse it. |
| **B/mock** | The **Format tab loses its Font group** | Reverses **O37**, the operator's own *"all the font tools Word has"*, shipped 2026-08-27. |
| **B/mock** | **Arrange** moves from Edit to Format | `RIBBON_IA.md:375-378` puts it on Edit. |
| **B/mock** | **pdfcer** group collapses to `Settings… + Help ▾` | `RIBBON_IA.md:228-230`. Also in tension with the review's own A17, which argues *against* burying commands behind carets. |
| **A10** | Page-display buttons get labels | Reverses a measured conversion (`RIBBON_SCALING.md:372-395`). ★ **But `RIBBON_IA.md:143-147` cuts the reviewer's way and predates it** — the two documents were never reconciled, and `RIBBON_SCALING.md` won by being later, not by argument. **This one deserves a ruling.** The reviewer's *"keep four buttons, add labels"* has no conflict; their *"one dropdown"* loses the four-position pressed-state radio and contradicts Acrobat. |

---

## 4. Already decided against, and the reviewer does not defeat it

| # | Claim | The ruling |
|---|---|---|
| **A13** | Find should not float | **The operator retired that invariant himself**: `HANDOFF.md:542-544` — *"Ignore the 'nothing floats over the canvas' stance… the bar floats, and the operator said to drop the argument."* A docked version was built, driven, and took Fit-page zoom 85 % → 81 % on every Ctrl+F. Acrobat, Chrome and Edge all float theirs. |
| **A19** | F11 should hide the panels | Three of three reference applications separate *chrome* from *capability*; F11 + Ctrl+H compose. Renaming F11 "Maximise" would conflate two distinct window states. Salvageable half: **discoverability** — F11's tooltip should mention Ctrl+H. |
| **A9** | Move the teaching text behind a "Why?" | *"Every disclosure above the list, without exception"* is recorded in three module headers with a stated failure mode: **a caveat below a list arrives after the operator has already drawn a conclusion.** The reviewer argues about the repeat visit; the rule is about the first. ★ The reviewer *is* right that no length policy exists, and the project does shorten copy on screenshot evidence — five font verdicts became two words each after exactly this kind of capture. |
| **A3a** | Delete pages needs a confirmation | Ruled at `app/actions/pages.rs:236`; the tooltip states it; the reviewer concedes and asks for inline undo instead, which is additive. |
| **A7** | Objects rows should ellipsize | The project deliberately removed truncation (`SALVAGE.md:44`) and already ships the tooltip. |
| **A7** | Drop the Tool panel's buttons — "they are on the ribbon" | **That is the exact assumption the panel exists to refute.** `panels/tool/mod.rs:11-21`: the text tools were registered, drawn, chorded and driven-verified — *"The feature works. He could not find it."* |
| **A17** | Label collapsed groups as menus | Collapsed width is measured from the caption; a longer label can make a collapsed group wider than its expanded self, which `widening_the_band_never_compacts_a_group_further` forbids. The real cause is the 1100 pt default window (A8). |

---

## 5. Where the reviewer was factually wrong

Recorded because an incorrect finding, uncorrected, becomes a fact.

- **"Escape closes no dialog."** It closes every one — read from the **child** window (`host.rs:901`). Escape was sent to the main window.
- **"Dialogs stack."** 18 of 20 kinds are single-instance-guarded; 2 replace in place. Four windows open at once were four *different* dialogs.
- **"Set scale was cancelled by drawing a rectangle."** No code path does that. There *is* a real stranding on the Escape-during-calibrate route, already written down at `canvas/placing.rs:39-43`.
- **"The suite drives many dialogs but not Keyboard shortcuts."** It drives it, in **three** checks. The truth was worse than the report: it drove it and passed anyway.
- **"Properties gets a quarter of the height", "315 px column", "Objects rows truncate without ellipsis"** — a third, 320, and no truncation exists.
- **"First launch is 1116 × 839"** — the declared constant is 1100 × 800; the difference is DPI inflation.
- **"The title shows the minute although the decision says the day"** — superseded by O101 on the operator's instruction.
- **A16b, "Print's preview column takes half the dialog and should be resizable"** — it is 42.5 %, **and it has been a draggable splitter since earlier the same day**, with a floor and a double-click reset. That the reviewer did not find it is itself a discoverability finding.

---

## 6. The icon set — "can go straight in" is false, in five specific ways

The handoff claims `mockups/glyphs/new.json`'s 65 glyphs "can go straight into
`crates/pdfcer-gui/src/icons/assets`". Verified against the real assets:

1. **Wrong format.** The assets are individual `.svg` files behind
   `include_str!` constants; this is one JSON object. Landing it means 65 files,
   65 constants, 65 enum variants with doc comments, 65 `ALL` entries, 65
   `name()` arms and a bump to the pinned `Icon::ALL.len() == 93`.
2. **Six glyphs use `stroke-dasharray`, which the parser silently ignores.**
   They parse, rasterize, pass both icon tests, and **draw the wrong picture** —
   `new-from-template` loses the dashed box that is the only thing separating it
   from `new-document`; `select-all` loses its marquee.
3. **Two use fill, and the fill set is closed by a test.**
   `fill_is_semantic_and_the_set_that_uses_it_is_closed` fails on them until the
   set is widened with a written reason per member.
4. **All 65 lack the rationale comment the house style requires** — what the
   glyph depicts and which neighbour it was drawn to stay distinguishable from.
5. **Five diverge from the stated 2.5 stroke** with only one of the divergences
   declared.

★ And `thin-lines` is art for a command **deleted six weeks ago with evidence** —
`RenderOptions` has no such field.

**Verdict: good art, a day or two of careful work, not a file copy.**

### ★ Closed 2026-09-04 — see `GLYPH_ADOPTION.md`

All five blockers are discharged and the batch is adjudicated. Blocker 2 was
fixed first and on its own (`stroke-dasharray` is now parsed and drawn, with a
pixel-count test rather than a parse assertion, because a build that parses the
attribute and forgets to hand it to the `Stroke` passes a parse assertion and
draws a solid line). Blocker 3 turned out to have a twist worth keeping: the
test the fill claim cites, `redaction_is_the_only_filled_icon`, **was renamed on
2026-08-19 and four doc comments went on citing the dead name for sixteen
days** — an audit grepped for it, found it in zero test bodies, and correctly
reported that the cited gate did not exist. It did; under a name none of its
citations had been told about.

**36 adopted, 26 deferred**, on one rule: a glyph is adopted only when a command
or role in this build would use it today. `thin-lines` is confirmed dead art.

### ★★★ 6b. …and on 2026-09-04 somebody finally LOOKED at them

Everything above was derived from reading `new.json` and `board-shell.html` as
**source text**. That was the wrong oracle for a question about glyphs and
layout, and this project has a standing rule saying so: *a layout or rendering
defect has exactly one oracle, and it is a rendered screenshot.*

Rendered headlessly (`chrome --headless --screenshot`, no desktop touched) at
`D:\temp\mockshot\` — `board-top.png`, `dark.png`, `glyphs.png`,
`glyphs-tail.png`.

**What looking confirmed, that reading had only inferred:**

- **The dashed-outline problem is real and it is worse than a cosmetic loss.**
  Seen side by side, the dash IS the distinguishing feature:
  `new-from-template` against `new-document`, `unembed-fonts` against
  `embed-fonts`, `redact-selection` against `redact`. Our SVG parser ignores
  `stroke-dasharray` silently, so those pairs would ship as **visual
  duplicates** — and `select-all` would ship as a plain rectangle. They would
  pass every icon test we own.
- **`thin-lines` is in the sheet, marked DRAWN FOR THIS MOCK** — art for a
  command deleted six weeks ago because `RenderOptions` has no such field.
- **The 65 new glyphs fill real gaps rather than restyling existing ones**:
  `merge`, `merge-files`, `move-page-up/down`, `paste-in-place`,
  `pick-form-xobject`, `measure-perimeter`, `measure-radius`, `save-compact`,
  `save-copy`, `render-diagnostics`, `reflow`, `put-down`, `pin`,
  `finish-shape`, `fill-colour`, `stroke-colour`, `opacity`, `line-width`,
  `tab-order`, `wheel-flip`, `zoom-readout`. Those are commands we **have** and
  draw without a proper icon.
- **They are shown at 32 px and at the 16 px they ship at**, side by side, in
  the sheet itself. They read at 16.

**What looking changed about the LAYOUT assessment, which §2 got half wrong:**

§2 concluded the mockup is *"the shipped manifest re-typed"*. That is true of
the **ribbon** and false of the **shell**. Seen rendered, four things are
materially better than what we ship and none of them is a ribbon change:

1. **The left icon+word rail** — Pages / Marks / Layers / Sigs / Fonts, vertical,
   ~50 pt wide, every panel one click away and none of them consuming a tab bar.
   It replaces two stacked left stacks and a `⏷ 3 more` overflow.
2. **Objects over Properties as a real master–detail**, and the Objects rows
   carry *"Text · "A1" · SpaceGrotesk-Bold 8 pt"* and *"Path · stroked #D97706,
   1.00 pt wide · 2 nodes"*. Ours lists far less per row.
3. **A one-line tool strip** at the top of the right dock — *"Select · click to
   pick · drag to marquee… [put down]"* — which is the Tool panel's whole
   content in 28 px instead of a stack.
4. **Rulers on two edges with a corner box**, drawn, not proposed.

★★ **And the dark board keeps the PAGE WHITE.** That is the single invariant a
dark theme in this product must hold — a tinted CAD sheet is an unreadable
drawing — and the mock gets it right. It is also, precisely, the check the test
review said nobody has written and should write first.

⇒ **Revised verdict on Part B:** the ribbon is ours re-drawn and its five
deltas are amendments for the operator (§3, unchanged). The **shell layout is a
genuine proposal, and it is better than ours.** Reading it as source made it
look like a restatement; it is not.

---

## 7. What the review got right that is worth saying plainly

The crash was real, first, and cost the reviewer unsaved work. The
encrypted-screen contradiction, Read-mode authoring, the empty Tab-order
header, the two contrast failures and Read's Format tab are all genuine and
were all invisible to a green test suite. **Three of them were invisible for
the same reason: a rule cited in a comment near the code, and not enforced by a
mechanism inside it.**
