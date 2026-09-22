# UI toolkit pins

Which egui this shell is built against, why each pin stands, and what moving one
costs. `tools/gates/check-ui-toolkit-drift.sh` reads the register table; the
prose is for whoever is deciding whether to bump.

This file never argues for upgrading. Being a release behind is normal and often
correct; being a release behind *without knowing* is the defect, and it is the
one this file and its gate make impossible.

## Why the pins cannot move on their own

`egui = { version = "0.35", … }` is `^0.35` in semver: `>=0.35.0, <0.36.0`. So
`cargo update` resolves the newest `0.35.x` and reports everything current —
truthfully, about a question nobody meant to ask. It cannot reach 0.36, ever,
and no amount of updating will report that 0.36 exists.

`pdfcer-core`, `pdfcer-render` and `pdfcer-print` are the opposite case: **path**
dependencies onto `D:\Dev\pdfcer\`, which move whenever that repository does and
need a drift gate because they change under you. egui needs one because it
changes only when somebody edits this file, and otherwise nothing watches
whether anybody should.

Ask of every dependency: *what would tell me it moved?* For a path dependency, a
diff. For a git pin, a hash. For a caret version requirement, nothing at all —
which is the one that needs an instrument.

## The register

Pinned is what `Cargo.lock` resolved; published is `MAJOR.MINOR` from
`cargo search <crate> --limit 1`.

| crate | pinned | published | status |
|---|---|---|---|
| `egui` | 0.35.0 | 0.36 | deliberately behind — see the 0.36 migration below |
| `eframe` | 0.35.0 | 0.36 | deliberately behind — one decision with `egui`, never bumped apart |
| `egui_tiles` | 0.16.0 | 0.17 | pinned but unused — no consumer, absent from `Cargo.lock` |

### The row shape is the contract

The gate greps these rows, not the prose around them: an argument in a paragraph
that happens to contain the right version number is a gate discharged by
narrative.

- **Cell 1** is the crate, bare or in backticks.
- **Cell 3** carries the published `MAJOR.MINOR` whenever crates.io is ahead of
  the lock. That is the assertion.
- **A pin that resolves to nothing** must carry `unused`, `not used` or
  `no consumer` somewhere in its row.
- Never widen a row past the header's four cells. `check-doc-markup.py` fails
  the build on that, because an unescaped `|` truncates a row silently.

### What the gate checks, and what it refuses to decide

```sh
bash tools/gates/check-ui-toolkit-drift.sh
```

It prints the versions `Cargo.lock` actually resolved, so read its output rather
than trusting the `pinned` column: the manifest states a **requirement**, the
lock states a **fact**, and they answer different questions.

| Part | Network | Asserts |
|---|---|---|
| A0 | no | every egui-family pin in `[workspace.dependencies]` resolves in `Cargo.lock`, or its row declares it unused |
| A | no | a document that names a crate version names the one the lock resolved |
| B | yes | crates.io's newest release is either the pinned one or has a register row |

The crate list is derived by scanning `[workspace.dependencies]` for names
beginning `egui` or `eframe` without a `path =`, never hard-coded — so adopting
`egui_extras` or `egui_plot` brings it under the gate with no edit to the gate,
and it will then demand a row here.

Three behaviours worth knowing before the gate surprises you:

- **It never fails for being behind.** It fails for being behind *unrecorded*.
- **It matches on `MAJOR.MINOR`.** egui publishes patches often, and a gate that
  went red on every patch would train everybody to edit this file without
  reading it. A new minor is the event that deserves a decision.
- **crates.io unreachable prints SKIP and exits 2, not 0.** A check that
  quietly downgrades to the half it could do is how a gate stops running
  unnoticed — and `run-all.sh` classifies on the exit code alone, so the word
  alone bought nothing. An offline run therefore makes the whole roster exit 3,
  which is what "an incomplete run" is supposed to look like.

Falsify it with `bash tools/gates/check-ui-toolkit-drift.sh --self-test`:
twelve arms against synthetic roots and a stub crates.io, needing no network
and reading none of this repository. Two of them assert the behaviours above
by demanding a **green** — a document naming no version at all, and a new
patch release — because a rule that only ever produces red is a rule nobody
has measured the cost of.

Part A binds one document today: `MODES_AND_PANELS.md` names the toolkit its
capability verdicts were measured against, and a bump that leaves that heading
behind turns a measured verdict into an unsourced one. Moving a pin means moving
that heading in the same commit.

## `egui_tiles` — pinned, and not in the build at all

No crate in this workspace depends on it, so Cargo never resolves it and it does
not reach `Cargo.lock`. A published `egui_tiles` 0.17 is therefore of no
consequence to this binary, and its register row is what stops the gate making
it look like one.

The dock is hand-built on `egui` directly, in `crates/egui-shell/src/dock/`. The
reason it had to be: a dock layout here is a plain value that round-trips to a
file, and `egui_tiles`' arena handles are not stable identities across a
restart, so a floating panel that must survive one could not be one of its tiles.
`dock/float.rs` carries the consequence — a float's home is an **address**, four
`usize`s, rebuilt rather than resolved when it goes stale.

Any feasibility verdict elsewhere in the documentation that turns on what
`egui_tiles` can do is a verdict about a library this shell never links.

**The pin stays**: deleting it erases the record of a considered choice, and the
next person reaches for `egui_tiles` again. If it is still unused at fold-in,
delete it then, with a line in `PROJECT_PLAN.md` saying so.

## The 0.36 migration

Re-derive the cost rather than trusting a figure: bump `egui`/`eframe` to 0.36
and `egui_tiles` to 0.17 in `Cargo.toml`, then restore.

```sh
cargo check --workspace --all-targets
git checkout Cargo.toml Cargo.lock
```

Two API changes reach this tree. The dock, the ribbon, the panels, the canvas
and `wgpu` compile untouched, and `egui-shell` is untouched entirely.

### 1. `DroppedFile::path` is a method, not a field

It returns `&Path` in 0.36, where in 0.35 it is a field of type
`Option<PathBuf>`. One call site: `crates/pdfcer-gui/src/app/filedrag.rs`, which
collects `i.raw.dropped_files` through `.filter_map(|f| f.path.clone())`.

A semantic change hides in the type. The option is gone, so a dropped file always
has a path — true on native, false on web, and this shell is native-only, which
makes the new shape correct here. But a `filter_map` that silently becomes a
`map` is a behaviour change wearing a compile error's clothes, and it deserves a
driven check rather than a sight-read. Two already exist: `dropped_file` and
`drop_onto_thumbnails`.

### 2. `RawInput::modifiers` is removed

Modifiers travel on the events instead of on the frame. Every site in this tree
that sets the field is `#[cfg(test)]` code building a synthetic frame:

| File | What it builds |
|---|---|
| `crates/pdfcer-gui/src/app/keyboard.rs` | a `key_press` builder, and one test that mutates the field directly |
| `crates/pdfcer-gui/src/app/status.rs` | a `key_press` builder |
| `crates/pdfcer-gui/src/canvas/textedit/keys.rs` | a clipboard-frame builder and an arrow-key driver |
| `crates/pdfcer-gui/src/canvas/textsel/clipboard.rs` | three frames carrying `Event::Copy` under `Modifiers::COMMAND` |
| `crates/pdfcer-gui/src/canvas/dimdrag/tests.rs` | a drag driver |
| `crates/pdfcer-gui/src/canvas/moving/nudge/tests.rs` | a nudge driver |

### Why the small change is the risky one

Nothing the compiler flags is shipped code, and that is the hazard rather than
the reassurance.

`app::keyboard::commands` matches each chord against the modifiers carried by its
own `Event::Key`, never against `InputState::modifiers`. Those are different
clocks: the frame-level state is as of the **end of the frame**, an event carries
the state as of the **keystroke**, and they disagree whenever the modifier is
released inside a long frame — the operator taps `Ctrl+Z` in fifty milliseconds
while the application is rasterizing a dense CAD sheet, and the chord matches
nothing. Reading the frame snapshot there is a defect this project has already
shipped, and it presented as harness flakiness rather than as a bug.

The regression test that pins the fix constructs exactly that disagreeing frame:
a `Ctrl+Z` key event carrying `Modifiers::COMMAND`, then `RawInput::modifiers`
set empty before the frame ends. **0.36 removes the field that test needs**,
which is upstream enforcing the same rule — and it means the bump rewrites the
harnesses that prove the keyboard path right at the same moment it changes the
platform beneath them. A green `cargo check` is not a report of working software
here; R1 applies with unusual force.

### The decision, and what would change it

**Stay on 0.35 until the bump can be verified by driving the binary.** The
operator uses `pdfcer-gui.exe` daily as his PDF reader; an unverified keyboard
change ships a dead Delete key to his working machine.

What the bump needs, when the machine is free:

1. The mechanical fixes the scratch `cargo check` above enumerates.
2. The full `tools/ui-verify` sweep — `chords`, and every modifier-bearing
   gesture: `Ctrl`-drag and `Ctrl`+`Shift`-drag on markup nodes, `Shift`-extended
   selection and marquee, `Alt` on measure and vertex routing, `Shift` on page
   drag, and the clipboard chords.
3. A driven check on dropping a file onto the window, for change 1.
4. A screenshot comparison of the dock and the ribbon, because a minor egui
   release moves layout and a layout defect has exactly one oracle.

The work is well under a day and the verification rather more, which is the
correct ratio for this change.
