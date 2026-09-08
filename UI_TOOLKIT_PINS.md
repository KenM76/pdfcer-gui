# UI_TOOLKIT_PINS.md — which egui this shell is built against, and why

**The register `tools/gates/check-ui-toolkit-drift.sh` reads.** One row per
egui-family crate named in `[workspace.dependencies]`. A row is required when
the pinned version is behind the published one, or when a pin resolves to no
consumer at all.

> **This file never argues for upgrading.** It records what is true and why the
> decision stands. Being a release behind is normal and often correct; being a
> release behind *without knowing* is the defect, and it is the one this file
> and its gate exist to make impossible.

---

## Why the pins cannot move on their own — read this before assuming `cargo update` covers it

`Cargo.toml` says `egui = { version = "0.35", … }`. That is `^0.35` in semver:
**`>=0.35.0, <0.36.0`**. So `cargo update` resolves the newest `0.35.x` and
reports everything current — truthfully, about a question nobody meant to ask.
It **cannot** reach 0.36, ever, and no amount of updating will tell you 0.36
exists.

⇒ That is different in kind from `pdfcer-core`, `pdfcer-render` and
`pdfcer-print`, which are **path** dependencies onto `D:\Dev\pdfcer\` and move
whenever that repository does. Those needed a *drift* gate because they change
under you. egui needed one for the opposite reason: it changes only when
somebody edits this file, and nothing was watching whether anybody should.

---

## The register

| crate | pinned | published | status |
|---|---|---|---|
| `egui` | **0.35.0** | 0.36 | **deliberately behind** — see *The 0.36 measurement* |
| `eframe` | **0.35.0** | 0.36 | **deliberately behind** — moves with `egui`, same row |
| `egui_tiles` | 0.16.0 | 0.17 | ★ **pinned but unused — no consumer, absent from `Cargo.lock`** |

---

## ★ `egui_tiles` — pinned, published, and not in the build at all

`[workspace.dependencies]` carries `egui_tiles = "0.16.0"`. **No crate in this
workspace depends on it**, so Cargo never resolves it and it does not appear in
`Cargo.lock`.

That is not an oversight. `crates/egui-shell/src/dock/mod.rs` states the
decision in its own words — the dock is hand-built, and `float.rs` records the
reason it had to be: *"`egui_tiles`' arena handles are unusable for
persistence"*, so a floating panel that must survive a restart could not be one
of its tiles. The pin is a **reservation**, left in place from the S3 plan.

Two consequences worth stating rather than leaving to be found:

- **`MODES_AND_PANELS.md` sourced its per-capability feasibility verdicts to
  "egui 0.35 / egui_tiles 0.16".** Half of that platform is not in the binary.
  The verdicts that turned on what `egui_tiles` could do are therefore about a
  library this shell does not link — which is exactly why the dock was
  hand-built, so the verdicts led to the right answer by a route their own
  citation does not describe. Corrected 2026-09-08.
- **A published `egui_tiles` 0.17 is of no consequence to this build**, and
  the gate must not be able to make it look like one. Hence this row.

⇒ **The pin stays** rather than being deleted, because deleting it would erase
the record of a considered choice and the next person would reach for
`egui_tiles` again. If it is still unused at fold-in, delete it *then*, with a
line in `PROJECT_PLAN.md` saying so.

---

## The 0.36 measurement — 2026-09-08

Measured rather than estimated, in a scratch copy of the manifest, restored
afterwards. `egui`/`eframe` 0.35 → 0.36, `egui_tiles` 0.16 → 0.17:

| | result |
|---|---|
| total compile errors, `--workspace --all-targets` | **11, across 6 files** |
| distinct causes | **2** |
| the dock, the ribbon, the panels, the canvas, `wgpu` | **compiled untouched** |

The two causes:

1. **`egui::DroppedFile::path` became a method returning `&Path`**, where it
   was a field of type `Option<PathBuf>`. One call site
   (`app/filedrag.rs:165`). ⚠ **Note the semantic change hiding in the type**:
   the option is gone, so a dropped file always has a path. That is true on
   native and was not on web, and this shell is native-only — but a
   `filter_map` that silently becomes a `map` is a behaviour change wearing a
   compile error's clothes, and it deserves a driven check rather than a
   sight-read.

2. **`egui::RawInput::modifiers` was removed** — 10 of the 11 errors, in
   `app/keyboard.rs`, `app/status.rs`, `canvas/textedit/keys.rs`,
   `canvas/textsel/clipboard.rs`, and two test harnesses. Modifiers now travel
   on the events rather than on the frame.

★★★ **The second one is not mechanical, and it is the reason this is not a
same-day bump.** `D:/dev/rag/egui/` already carries
`a_chord_matcher_must_read_the_key_events_own_modifiers_not_the_frames.md` —
this project has *already had a defect* from reading frame-level modifiers
where event-level ones were meant. egui 0.36 removes the frame-level field
outright, which is upstream enforcing the same rule.

⇒ So the migration is small but lands squarely on the code path this project
has already got wrong once, and on the two files that hold the keyboard guard
whose earlier defect (`egui_wants_keyboard_input` vs `text_edit_focused`) is
one of the two founding defects in `DEFECTS.md`. **A green `cargo check` is not
a report of working software here** — R1 applies with unusual force.

### The decision, and what would change it

**Stay on 0.35 until the bump can be verified by driving the binary.** The
operator uses `pdfcer-gui.exe` daily as his PDF reader; an unverified keyboard
change ships a dead Delete key to his working machine, which is the exact
defect this whole rebuild was founded on.

What the bump needs, when the machine is free:

1. The 11 mechanical fixes.
2. The full `tools/ui-verify` sweep — every chord, every modifier-bearing
   gesture (`Ctrl`-drag, `Ctrl`+`Shift`-drag on markup nodes, `Shift`-marquee,
   the clipboard chords).
3. A driven check on **drag-and-drop of a file onto the window**, for cause 1.
4. A screenshot comparison of the dock and the ribbon, because a minor egui
   release moves layout and this project's standing rule is that layout defects
   have exactly one oracle.

Estimated at well under a day of work and rather more than that of verification
— which is the correct ratio for this change, not a complaint about it.
