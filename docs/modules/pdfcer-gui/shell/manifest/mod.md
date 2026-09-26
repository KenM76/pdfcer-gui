# shell::manifest — pdfcer's ribbon, as an `egui_shell::Shell` value

[`built_in`] returns the complete pdfcer shell: eight tabs (seven
ordinary plus the contextual Format tab), thirty-seven groups, three
modes, the quick-access toolbar and the keymap. It is the **built-in
layer** of `SHELL_FRAMEWORK.md` §4's three-layer merge:

1. **Built-in** — this function. Compiled into the binary, always
   valid, and always available as the reset target.
2. **Application override** — an optional file shipped beside the exe.
3. **Operator customization** — `userdata/shell.ron`.

Layers 2 and 3 override this one **per item**, never wholesale. That is
why this layer has to be complete and has to validate: it is the thing
every other layer is a patch against, and it is what an operator gets
back when they reset.

One file per tab. The tab modules are where the *reasoning* lives —
why a command sits where it does, what moved, what was left out and
why — and they are worth reading before changing anything here.

# The no-placeholders rule, and the two registers that keep it honest

`RIBBON_IA.md` P3: *an unavailable capability renders nothing, not a
disabled stub.* Greying is reserved for **temporarily** unavailable —
no document open, undo stack empty — and is always explained on hover.

`RIBBON_IA.md` §5 marks every command it specifies with where it exists
today:

| Mark | Meaning | In this manifest |
|---|---|---|
| **G** | exists in the GUI now | emitted |
| **C** | exists in `pdfcer-core`/`pdfcer`, no GUI surface | **absent**, in [`PLANNED`] |
| **N** | exists nowhere | **absent**, in [`PLANNED`] |

A **C** row is the cheapest kind of missing command — the hard half is
written and tested — and it is still absent, because P3 is about what
the operator can reach and an engine with no caller is not reachable.

Absent is not forgotten. Two registers make the difference visible:

- **[`PLANNED`]** — every specified command this manifest does *not*
  emit, with the reason. Tested in both directions: nothing in it is
  referenced by the manifest, and nothing in it is registered. That is
  the list a later stage reads to find its work.
- **[`DIRECTED`]** — the small set of commands emitted *despite* not
  carrying a **G** mark, each with the instruction that put it there.
  Without this list those seven entries would look like the manifest
  quietly ignoring P3.

# Command ids

Dotted lowercase, and the prefix is **the tab that owns the command**:
`view.zoom_fit_page`, `pages.rotate_left`, `markup.highlight`. That
makes P1 — one command, one tab — legible in the id itself, and it
makes a violation obvious on sight rather than only at validation.

Two deliberate exceptions:

- `edit.undo` and `edit.redo` sit on **no tab**. They live on the QAT
  alone, which `RIBBON_IA.md` §7 keeps unchanged. The `edit.` prefix
  says where they would go if they ever got one.
- `mode.read`, `mode.review` and `mode.edit` are not tab commands at
  all: they are the three positions of the selector at the far right of
  the tab row, reachable from the keymap.

# What this module deliberately does not decide

**Icons, labels, tooltips and enable predicates** — those are the
registry's half of the split, in [`super::commands`]. A manifest
contains command *ids* and nothing else about them, which is what stops
a customized ribbon from inventing a command and what makes an unknown
id a disclosed skip rather than a crash.

**Behaviour.** Nothing here runs.
