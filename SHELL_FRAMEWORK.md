# `egui-shell` — the reusable application shell

**Status:** architecture, 2026-08-13. Operator directive: *"all this
customization work should be made so we can reuse it for other projects
that need a GUI. we should even be able to customize the ribbon menus
and layouts if we want to."*

---

## 1. The insight that makes both asks one piece of work

Two requirements arrived together:

- **Reusable across projects.** The ribbon, dock, modes and persistence
  should not be welded to PDF concepts.
- **Customizable at runtime.** The operator should be able to rearrange
  the ribbon itself, not just the panels.

Both are satisfied by the same decision:

> **The shell is data. Tabs, groups, commands, panels, layouts, modes
> and key bindings are a serializable document that the application
> *supplies* and the operator *edits* — not code that has to be
> recompiled to change.**

A ribbon defined in Rust `match` arms can be neither reused nor
customized. A ribbon defined as data can be both, and the same
serializer that lets an operator save a customized ribbon lets a
different application ship a completely different one.

This also retires a deferral. `ribbon.rs:42-52` currently defers ribbon
customization on the grounds that *"a customisable ribbon that also
forgets itself would be worse than none."* That objection was about
persistence, and persistence is now the first thing built.

---

## 2. Crate split

```
D:\Dev\pdfcer-gui\
├── crates\
│   ├── egui-shell\        ← reusable. Knows nothing about PDF.
│   └── pdfcer-gui\         ← the application. Supplies a manifest + panel bodies.
└── tools\ui-verify\       ← harness; drives any egui-shell app.
```

**The hard rule that keeps `egui-shell` reusable:** it may depend on
`egui`, `eframe`, `egui_tiles`, `serde` and small leaf utilities —
**never on `pdfcer-core`, `pdfcer-render`, or anything that knows what a
PDF is.** A CI gate enforces it, the same way pdfcer already gates
`pdfcer-core` against gaining a GUI dependency. If `egui-shell` needs to
know about pages, the abstraction is wrong.

**Licence:** MIT, matching pdfcer, so it can be published separately.

**Fold-in:** `egui-shell` is extracted to its own repository at or before
fold-in and consumed by path or git dependency. It is deliberately *not*
folded into pdfcer as a private module, because the whole point is that
the next project can take it.

---

## 3. What lives in `egui-shell`

| Module | Responsibility |
|---|---|
| `manifest` | The serializable shell definition — tabs, groups, commands, panels, modes, keymap. The single source of truth. |
| `ribbon` | Renders a manifest. Group captions, overflow, contextual tabs, the QAT. |
| `dock` | Panel host over `egui_tiles`: columns, stacks, tabs, splitters, overflow menu, icon rail, tear-out. |
| `modes` | Named workspaces and the N-position selector. Read/Review/Edit is a *configuration*, not a built-in. |
| `layout` | Serialization, persistence, fail-soft loading, reset scopes, named workspace save/load. |
| `theme` | Token palette, presets, and the **rendered-pair contrast gate** that the old GUI lacked. |
| `commands` | Command registry, enable/visibility predicates, keymap, and the action-dispatch choke point. |
| `verify` | Diagnostic channel and hooks `ui-verify` drives. |

### What stays in the application

Panel *bodies*, command *implementations*, domain state, and the
manifest itself. `egui-shell` renders a tab called "Measure" and routes a
command called `measure.linear`; it has no idea what either means.

---

## 4. The manifest

RON, beside the existing `userdata/settings.txt`. Sketch — not final
syntax:

```ron
Shell(
    modes: [
        Mode(id: "read",   label: "Read",   tabs: ["file", "view"]),
        Mode(id: "review", label: "Review", tabs: ["file", "view", "pages", "markup", "measure"]),
        Mode(id: "edit",   label: "Edit",   tabs: ["file", "view", "pages", "edit", "markup", "measure", "tools"]),
    ],
    contextual_tabs: [
        Tab(id: "format", label: "Format", visible_when: "selection.any"),
    ],
    tabs: [
        Tab(id: "view", label: "View", question: "What is on my screen?", groups: [
            Group(id: "page_display", caption: "Page display", items: [
                Command("view.single"), Command("view.continuous"),
                Command("view.facing"), Command("view.facing_continuous"),
            ]),
            Group(id: "render", caption: "Render", items: [
                Command("view.render.strategy"), Command("view.render.quality"),
                Command("view.render.settle"),   Command("view.thin_lines"),
            ]),
            Group(id: "window", caption: "Window", items: [
                Command("view.read_mode"), Command("view.fullscreen"),
                Command("view.floating_panels"),      // Off · Allowed
                Command("view.app_initiative"),       // Never · Ask · Allowed
                Command("view.reset_layout"),
            ]),
        ]),
    ],
    qat: ["file.open", "file.save_copy", "edit.undo", "edit.redo"],
    keymap: { "Ctrl+E": "edit.text", "Ctrl+1": "mode.read", "F11": "view.fullscreen" },
)
```

**Commands are referenced by string id, never defined here.** The
application registers `measure.linear` with its label, icon, tooltip,
enable predicate and handler; the manifest only says where it appears.
That is what stops a customized ribbon from being able to invent a
command that does not exist, and what makes an unknown id a *disclosed
skip* rather than a crash.

### Three layers, merged at load

1. **Built-in** — compiled into the binary. Always valid, always
   available as the reset target.
2. **Application override** — optional file shipped beside the exe.
3. **Operator customization** — `userdata/shell.ron`.

Later layers override earlier ones **per item**, not wholesale, which is
the same per-key fail-soft contract `settings.txt` already uses. A
customization referencing a command that no longer exists loses that one
item and says so in the status surface; it does not discard the layout.

---

## 5. What customization actually permits

| | Allowed | Why |
|---|---|---|
| Reorder tabs, rename them, hide them | ✅ | Presentation. |
| Move a command between groups or tabs | ✅ | Presentation. |
| Create a custom tab or group | ✅ | The main ask. |
| Rebind keys | ✅ | Presentation. |
| Define a new mode with its own tab set | ✅ | Modes are configuration. |
| Save / load / share a named workspace | ✅ | Includes ribbon **and** panel layout. |
| Invent a command | ❌ | Commands come from the registry. |
| Change what a command does | ❌ | Behaviour is code. |
| Bypass a command's enable predicate | ❌ | Predicates are safety, not decoration. |

**One rule preserved from the old ribbon:** a command may appear on
exactly one *tab*; the QAT and status bar may mirror it. The uniqueness
test moves into `egui-shell` and now runs against the merged manifest,
so a customization that puts one command on two tabs is rejected at load
with a message naming the command — which is more than the old
compile-time test could do for a user-supplied layout.

---

## 5b. Modularity — capabilities that can be removed

**Operator directive, 2026-08-13:** *"Everything should be capable of
being modular… even the core components, if not needed by someone they
could just remove them and they would not show up as options in the
GUI."* DLLs are named as the eventual mechanism; the exe stays for now.

**Most of this already works, and it works because the shell is data.**

### What exists today

`pdfcer-core` already has a documented **strippable capability**
convention, with `jpx` (JPEG 2000) as its first instance: an optional
dependency named by a feature, forwarded through every intermediate
crate, with CI building `--no-default-features` to prove the stripped
build is real. Its manifest states the discipline outright — *"a
capability that silently disappears from a default build is a regression
wearing a feature flag."*

The shell side is already built for it too. Commands live in a
**registry populated at runtime**, and a manifest item naming an
unregistered command is dropped with a `SkipReason::UnknownCommand`
rather than failing. So a capability that is not compiled in simply has
no command registered, and its ribbon item **disappears** — no `#[cfg]`
anywhere in the ribbon, no dead button, no disabled stub. That is
already exactly the behaviour asked for.

### The one rule that has to hold

> **A capability's presence is expressed by registering its command, and
> by nothing else. No other code in the GUI may test for a capability.**

If the ribbon renderer ever hard-codes "the OCR button goes here", or a
panel does `if cfg!(feature = "ocr")`, the exe→DLL move stops being a
swap and becomes a rewrite. The registry does not care whether a command
was registered by a statically linked module or by one loaded from a DLL
at start-up — but only if it is the *only* thing that knows.

This also satisfies the existing **no placeholders** rule by
construction rather than by discipline: an unavailable capability
renders nothing because there is nothing to render.

### The gap that must be closed

`merge.rs` deliberately does **not** filter the built-in layer, and the
reasoning is sound *for the case it was written for*: an unknown command
in a compiled-in manifest is a programming error, and quietly repairing
it at start-up would hide the bug on every machine that runs it.

But modularity creates a **second, legitimate** case. The built-in
manifest names `tools.ocr`; OCR is compiled out; that is not a bug, it is
the intended configuration. Under today's rule that is either a
validation failure that blocks start-up, or a live command with no
handler.

The fix is not to weaken the strict rule — it is to make the two cases
distinguishable, which means the manifest must say which items are
conditional:

```ron
Group(id: "recognise", caption: "Recognise", items: [
    Command("tools.ocr", capability: "ocr"),   // conditional
    Command("tools.fonts"),                    // mandatory
])
```

| Item | Command registered? | Result |
|---|---|---|
| mandatory | no | **hard validation failure** — a programming error, unchanged |
| conditional | no | dropped, `SkipReason::CapabilityAbsent { capability }` — informational |
| either | yes | rendered |

`CapabilityAbsent` is a *different* reason from `UnknownCommand`
precisely so the two never get confused in a log: one says "this build
does not include that", the other says "someone made a mistake". A group
left with no items after filtering does not render; a tab left with no
groups does not appear in any mode's tab list.

**Scheduled for S2, immediately after the ribbon renderer lands** —
the `egui-shell` crate is mid-write by another agent as of this note.

### On DLLs specifically

Worth being precise, since it is named as the goal:

- **Cargo features are the cheap 90 %, and they work today.** Removing a
  feature removes the dependency, the code, and — with the rule above —
  the ribbon item. It is compile-time rather than drop-in, but it is real
  modularity at zero new machinery, and it is already the established
  convention in `pdfcer-core`.
- **True drop-in DLLs need an ABI boundary.** Rust has no stable ABI, so
  a `cdylib` cannot expose Rust types across the boundary safely. The
  options are an `extern "C"` interface with a hand-written vocabulary of
  plain-data types, or a crate like `abi_stable`/`stabby` that provides
  one. Either is a real project, and it constrains what a capability may
  exchange with the host to what survives that boundary.
- **What that project would *not* have to include** is any GUI work, if
  the rule above holds. Loading a DLL at start-up and calling its
  `register(&mut CommandRegistry)` is the same act as calling a
  statically linked module's. The shell already cannot tell the
  difference.

So the decision to defer DLLs costs nothing, **provided the registry
stays the single authority on what exists.**

## 6. Why this is not over-engineering

The manifest is roughly the same volume of information the old
`ribbon.rs` + `ribbon_ui.rs` held in code — about 1,850 lines — but as
data it is inspectable, diffable, testable without a GUI, and
serializable. Concretely it buys:

- **Ribbon customization**, previously deferred as impractical.
- **Modes**, which are otherwise a large `if mode == …` running through
  every tab-rendering function.
- **Named workspaces**, since a workspace is a manifest overlay plus a
  dock layout.
- **Testability** — the IA can be asserted (every command reachable,
  every group captioned, no duplicates) with no window open.
- **Reuse**, which was the directive.

The cost is one indirection between a button and its handler, and a
schema to version. Both are cheap; neither is on a hot path.

---

## 7. Build order

`egui-shell` is built **as** pdfcer-gui is built, not before it. A
framework designed without a consumer gets the abstractions wrong.

| Stage | `egui-shell` gains | Driven by |
|---|---|---|
| S0 | crate, theme tokens + contrast gate, diag channel | skeleton |
| S1 | `verify` hooks | ui-verify harness |
| S2 | `manifest`, `ribbon`, `commands`, keymap | the seven-tab ribbon |
| S3 | `dock`, `layout` persistence, overflow menu | the panels |
| S3b | `modes`, named workspaces | Read/Review/Edit |
| S6 | icon rail, fit-zoom cache hooks | viewer conventions |
| post | tear-out, cross-dock drag | — |

**Extraction test, at S3b:** write a throwaway second application — a
few hundred lines, a different domain, three tabs and two panels —
against `egui-shell` alone. If it needs one line of pdfcer, the boundary
is wrong and it gets fixed then, while the cost is a day rather than a
rewrite.
