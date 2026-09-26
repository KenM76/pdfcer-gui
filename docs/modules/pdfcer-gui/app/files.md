# `app::files` — how a path gets from an operator (or a harness) to
[`crate::app::actions::Action::Open`]

One question, asked in one place: **which document does the operator want
to open?** Everything downstream of the answer — loading, the three-way
failure split, forgetting the previous document's panel state, recording
the file in the recent list — is [`crate::app::PdfcerApp::open_path`]'s and
is reached through the action funnel, never from here.

## Rule 1: substitute the dialog's ANSWER, never its interaction

`D:\dev\rag\egui\native_file_dialog_is_a_hard_wall_substitute_the_answer_via_env_var.md`
records this as a **pattern in this project**, promoted after its second
independent instance (`diag::font_dirs`, then `PDFCER_DIAG_EXPORT_DIR`):

> A native file/folder picker hands control to the OS shell, outside
> egui's own event loop. Neither `eframe::App::raw_input_hook` synthetic
> events nor OS-level `SendInput`/`PostMessage` automation can drive the
> native dialog's own widget tree — it is a separate top-level window
> owned by the shell, not an `egui::Window`. […] Don't try to script the
> dialog. Check an environment variable BEFORE opening it; if set, use its
> value as the dialog's result and skip opening the dialog at all.

So [`pick_document`] checks [`DIAG_OPEN_PATH`] first, and the seam
replaces exactly **one** call: everything the harness is actually testing
— the action, the load, the failure classification, the recent list, the
panels forgetting the previous document — runs through the identical code
path a real click produces. The RAG's own instruction for anything new is
followed here rather than rediscovered: *"any future `rfd` call added to a
scripted-driven GUI should get the same `PDFCER_DIAG_<PURPOSE>` seam from
the start, not after the harness fails to reach it."*

| `PDFCER_DIAG_OPEN_PATH` | [`pick_document`] returns | For |
|---|---|---|
| unset | whatever the native picker says | the operator |
| a path | [`Picked::Path`] — no dialog opens, and **every** call answers the same path | a harness opening a second document |
| set but **empty** | [`Picked::Cancelled`] — no dialog opens | a harness exercising the *cancel* path, which is the one that must change nothing |
| paths joined by `;` | one entry per call, in order, then [`Picked::Cancelled`] for ever | a harness provisioning **one process with several documents** — see [`queued`] |

## The picker is `rfd`, and the version is not a choice

[`rfd`](https://docs.rs/rfd) 0.17.2 — the **exact** version in
`D:\Dev\pdfce\crates\pdfce-gui\Cargo.toml`. Pinning to it is what makes
this an *adoption* of a dependency already in pdfcer's lockfile and already
licence-vetted (MIT, no C dependency), rather than a new one this workspace
introduced. The workspace root states the rule: a crate this workspace adds
which is not already in pdfcer's lockfile is an operator decision.

It opens the OS's real dialog, so the operator gets the picker they already
know — their own places, recent folders and typing habits — and the dialog
is owned by the pdfcer window, so it travels with it and centres on it.

### What was here first, and why it is worth recording

This module was built while the manifest was not its to edit, and the
obvious response to that — draw the Open button and leave it inert until
the dependency arrives — is defect **D1**'s exact shape, on the command
where it matters most: *a reader that cannot open a second file is not a
reader*. So the first implementation shelled out to `powershell.exe` for
WinForms' `OpenFileDialog`, returning the chosen path as **ASCII hex**
because a redirected PowerShell stdout carries the console code page and
`Übersicht.pdf` would otherwise arrive mangled — naming a different file,
or none, while looking exactly like a real answer.

It worked, and it was verified against a live dialog. It was also
Windows-only, unparented, and cost a process launch before anything
appeared. It is gone now, and the reason to keep the paragraph is the
judgement rather than the code: **an honest interim that works beats a
placeholder that does not**, and the interim was built so that replacing it
touched exactly one function — the seam, the action, the command and the
dirty-document rule were all deliberately on this side of the call.

## Rule 3: no test may dispatch `file.open`

On the machine this is built on, dispatching `file.open` opens a **real
modal dialog** and blocks until a human dismisses it. A `cargo test` that
did that would hang the suite with an invisible window behind the
terminal. So the tests here cover [`from_env`], which is pure, and
[`raise`] — the translation from a [`Picked`] to an action — with all
three variants supplied directly. The only untested millimetre is the
`env::var_os` read itself, and it cannot be tested: `std::env::set_var` is
`unsafe` in edition 2024 and this crate is `#![forbid(unsafe_code)]`.

## The dirty-document rule, stated where it will be needed


* `save_pending` asks *"is a save **in flight**?"* — is there a moment at
  which the bytes on disk are a partial revision and the `EditSession` being
  read from must not be replaced. `crate::app::save::save_copy` is
  **synchronous**: it is entered and finished inside one
  [`crate::app::PdfcerApp::apply`] call, and no frame is ever drawn while it
  is part-way through. So there is still no state in which the predicate can
  be true, and `PROJECT_PLAN.md`'s no-placeholders invariant still forbids
  building a confirmation dialog for a condition that cannot occur.
* It is emphatically **not** *"are there unsaved edits?"*. Save-a-copy does
  not clear that, because a copy went elsewhere and the open document is
  still unsaved **at its own path** — see `crate::app::save`, which carries
  the whole argument and the reason nothing on `OpenDoc` moves.

What exists is one predicate, [`crate::app::PdfcerApp::save_pending`],
consulted by [`crate::app::actions::Action::Open`],
`crate::app::actions::Action::New` and
[`crate::app::actions::Action::Close`], returning `false` with the whole
rule written above it. The day an **asynchronous** save lands — the one
`file.save` in `crate::shell::manifest::PLANNED` is blocked on, behind
autosave and crash recovery — that function reads its state and the three
arms grow a confirmation, in one place, already wired.
