# `ui-verify/checks/foreign_icon_name`

`a_foreign_icon_name_reaches_the_panel` — **a sticky note whose icon name
pdfcer does not model shows that name, and is not silently renamed.**

# What this is for

ISO 32000-1 §12.5.6.4 gives seven icon names for a `/Text` annotation and
says they are *"a standard set, not a closed one"* — a producer's own name
is **conforming**. Until `pdfcer-core` `Pass 253.5` the engine's reader ran
`/Name` through `StickyIcon::from_name` and `.unwrap_or(Note)`, so a note
carrying `/Sparkle` read back as `Note` and a restyle of its **colour
alone** wrote `/Name /Note` into the operator's file. This shell filed it
(`request_set_text_annot_style_rewrites_a_foreign_icon_name.md`) and worked
around it by reading the raw dictionary beside the spec.

The engine now carries the bytes (`StickyIcon::Other`), the workaround is
deleted, and this check is what says the whole chain arrived.

## Why the in-process tests are not enough

`tests/engine_overlay_skew.rs`'s
`a_foreign_icon_name_survives_a_colour_only_restyle` proves the **engine**
keeps the name through a colour change, end to end, on this fixture. It
cannot prove any of what an operator meets:

| # | link | its own test |
|---|---|---|
| 1 | a click on the note selects it as an annotation | `selection::annot` — the geometry, given candidates |
| 2 | the Properties panel resolves it to `Reach::TextAnnot` | `markup::tests` — the routing, given a spec |
| 3 | **the icon subsection actually draws** | **nothing** — it is behind a dock tab, a mode and a scroll |
| 4 | **the chooser shows the FILE's name and not a variant label** | **nothing** |
| 5 | the foreign-icon note is shown beside it | **nothing** |

**Link 3 is the one that would ship as silence.** In Review the right dock
opens on Comments, and a dock draws only its active tab — so a panel that is
perfect and a panel that is broken produce the same empty trace. This check
brings the tab forward before it reads anything, which is a lesson this
project has paid for three times in one afternoon.

## The oracle is a trace line, and it has to be

A build that flattened `/Sparkle` to `Note` and one that carried it draw
**the same rectangle, in the same place, with the same controls**, differing
only in the words inside a combo box. No published region can see that, and
a pixel comparison of rendered text at this size is not an assertion anybody
should build on. `textannot-rows` carries `icon=` as the file spells it, so
the check reads the name and not the layout.

## The fixture is PLANTED, and a weaker one would make this vacuous

`fixtures/foreign-icon-name.pdf` is `comment-note.pdf` with one `/Comment`
rewritten to `/Sparkle` — the same length, so every byte offset in the file
is preserved. A fixture that merely *omitted* `/Name` would not defeat the
old behaviour at all: Table 172's default is `Note`, so the absent case and
the flattened case produce the identical value. The name has to be present,
conforming, and outside the seven.

⚠ It carries **two** sticky notes — an ordinary `/Note` and the `/Sparkle` —
which is deliberate and is why this check aims at a rect it derives from the
trace rather than at "the first annotation". The in-process test's first
draft took the first `/Text` on the page and reported, confidently and
wrongly, that the engine had flattened a name.
