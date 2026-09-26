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

## Item notes

### `const DIAG_INSERT_PATH`

`PDFCER_DIAG_*` is this project's established prefix for a
diagnostics-only seam — `PDFCER_DIAG`, `PDFCER_DIAG_VIEWPORT`,
`PDFCER_DIAG_EXPORT_DIR` — and the naming is part of the pattern rather
than decoration: a reader who finds one of them knows what kind of thing
the others are.
The environment variable `pages.insert_from_file`'s picker reads.

Deliberately NOT `DIAG_OPEN_PATH` — see [`pick_insert_source`] for why
sharing one seam between two verbs would make a check that drives both
impossible to write.

### `const DIAG_IMAGE_PATH`

A third variable rather than a shared one, on the argument
[`pick_insert_source`] spells out for the second: one seam answering two
pickers makes a run that opens a PDF and inserts a picture unwritable, and
a run meant to test one quietly test both. Three verbs, three seams.

### `static OPEN_QUEUE_POS`

A property of the process, not of a document: the queue exists to
provision a process with several documents, so its position must
survive every one of them being opened and closed.

### `fn queued`

# Why a queue, when every other seam here is single-valued

A shell that holds several documents at once has a ceiling question that
only a several-document process can answer — `OPERATOR_REQUESTS.md` **O221**
— and until this existed there was **no headless route to one at all**.
`main` reads a single `argv[1]`, the synthetic file drop fires once per
process, and a single-valued seam answers every picker with the same path,
which opens one document however many times it is rung.

⇒ *N* separate processes could be provisioned and *N* tabs could not, so the
configuration the per-document strip cache would actually multiply in was
the one that could not be measured.

# The single-valued seam must behave identically, and that is what the
separator test buys

`None` here means *"this is not a queue"*, and the caller then runs the
original path untouched. A value with no `;` in it therefore answers every
call with the same path exactly as before — so every check written against
the old seam keeps its meaning, and this cannot be a silent change to a
seam six checks already depend on.

A `;` cannot occur in a Windows filename, which is what makes the
separator test safe rather than a heuristic. It is also the separator
[`pick_merge_sources`] already uses, so the two list-valued seams in this
module are spelled one way.

# An exhausted queue says the operator declined

Past the last entry — and for an empty entry between two semicolons — the
answer is [`Picked::Cancelled`], which is the same answer the empty
single value gives and for the same reason: a complete, correct outcome
that opens no dialog. The alternative, falling through to the native
picker, would put a real modal dialog in front of a harness that has no
hand to dismiss it.

### `fn native_pick`

`rfd` opens the OS's real dialog — Windows' `IFileOpenDialog`, GTK/portal
on Linux, `NSOpenPanel` on macOS — so the operator gets the picker they
already know, with their own places, recent folders and typing habits.

# Errors have exactly two shapes, and only one of them is an error

`pick_file` returns `Option<PathBuf>`: `Some` is a chosen file, `None` is
a dismissed dialog. There is no third case, so [`Picked::Unavailable`]
cannot arise here at all — it survives as a variant because
[`Picked::from_env`] can still answer it, and because collapsing "the
operator said no" into "this build cannot ask" is the distinction the
type exists to keep.

# It blocks

The UI thread stops while the dialog is open. That is what a modal file
dialog is, and it is what the previous implementation did too; nothing
repaints behind it. `pick_file` is the blocking call deliberately rather
than `pick_file().await` — an async picker would need the frame loop to
keep running with an open document half-replaced, which is a larger
change than opening a file should be.

### `fn native_save`

The directory and the file name are set separately because `rfd` treats
them as separate: handing the whole path as a name would produce a dialog
offering to create a file called `D:\scans\survey-recognised.pdf` inside
whatever folder it happened to open in.

`title` is a parameter rather than a constant because the two callers are
asking about different things and the window's heading is the only place
the OS lets pdfcer say which — see [`pick_save_path`]. It is still catalog
copy: both call sites pass a `crate::text::*` function, and neither builds
a sentence.

### `fn a_single_path_is_not_a_queue_and_never_runs_out`

The arm that matters most: six checks answer their pickers through the
single-valued form, and every one of them calls `pick_document` more
than once. `None` here is what keeps them meaning what they meant.

### `fn an_exhausted_queue_declines_rather_than_opening_a_dialog`

This is the arm that decides whether the seam is safe to leave set
for the whole of a run. Returning `None` at the end would fall through
to `rfd` and put a real modal dialog in front of a harness with no hand
to dismiss it — the exact failure the seam exists to prevent.

### `fn only_a_picked_path_becomes_an_action`

The `file.open` arm reduced to the part a test may run — see rule 3 in
this module's header for why dispatching the command itself is
forbidden here. All three answers are checked, because the interesting
failure is not "a path did nothing" but "a *cancel* opened something":
`Picked` exists as three variants precisely so a dismissed dialog
cannot be mistaken for a path, and an `Option<PathBuf>` collapsed with
`unwrap_or_default` would open `""`.

### `fn the_close_command_empties_the_shell`

`file.close` was registered, drawn on the File tab, gated on
`doc.open` — and had no dispatch arm, exactly as `file.open` had none.
Driven through the real token lookup rather than by calling the arm, so
a command that stopped being registered fails here rather than silently
taking the `command-unimplemented` path.

### `fn the_open_action_opens_whether_or_not_something_is_already_open`

Including the one an operator meets most: nothing open at all.
[`crate::app::PdfcerApp::apply`] refuses every other action when
`Status` is not `Open`, which is right for actions about the open
document and would be fatal here — so Open and Close are matched
*before* that guard, and this is the assertion that says so.

### `fn the_save_copy_command_raises_the_save_action`

The regression guard for the defect this command shipped with for the
whole life of the project: it was registered, drawn on the File tab,
drawn on the quick-access toolbar, bound to `Ctrl+S`, printed "(Ctrl+S)"
in its own tooltip — and had **no dispatch arm**, so every press traced
`command-unimplemented` and nothing this shell could author could be
written to disk.

Driven through `commands.get(id).handler` rather than by calling the arm,
exactly as `the_close_command_empties_the_shell` and
`the_new_command_makes_a_blank_document_from_nothing` are, and for the
reason those two record: a test that called the function directly would
pass against a build in which the command was never registered, or in
which the token-to-id lookup had stopped resolving — which is precisely
the state that produced the fall-through in the first place.

# Why it stops at the action, and must

It raises and does **not** apply. Applying `Action::SaveCopy` reaches
`crate::app::save::save_copy`, which opens a **real modal save dialog**
unless `PDFCER_DIAG_SAVE_PATH` is set — and this crate is
`#![forbid(unsafe_code)]` while `std::env::set_var` is `unsafe` in
edition 2024, so a test cannot set it. That is rule 3 in this module's
header, moved one phase along with the picker: the *dispatch* of
`file.save_copy` is safe to drive and its *apply* is not.

What is therefore untested here and tested elsewhere, stated rather than
implied by a green run: the write itself is covered by
`crate::app::save`'s own tests, which call the picker-free half directly
and re-open the file that comes out; and the whole chain — ribbon click,
dispatch, apply, picker, write, re-open — is covered by
`tools/ui-verify`'s `save_copy_round_trip`, which answers the dialog
through [`DIAG_SAVE_PATH`] because that is the only way anything can.

### `fn the_dirty_document_gate_blocks_nothing_in_a_build_with_no_save`

The dirty-document rule has one home,
[`crate::app::PdfcerApp::save_pending`], consulted by both arms. This
build has no save, so it answers `false`, and the assertion is that the
two arms therefore proceed. It is not a tautology: it pins the
direction of the gate, so a future save subsystem that wired it
backwards — blocking an Open whenever a document is merely *dirty*,
which is not what the rule says — fails here rather than in an
operator's hands.

### `fn the_diagnostic_seam_answers_the_dialog`

This is the whole harness contract, and every row of the table in the
module header is asserted: unset defers to the picker, a value is a
path, and an *empty* value is a cancel — the third being the one a
reader would otherwise assume was an accident.

### `fn the_seam_does_not_mangle_a_real_path`

`OsString` rather than `String` throughout for the same reason
`main.rs` reads `args_os`: a path is not required to be valid Unicode,
and a non-Unicode path is the operator's business rather than ours to
reject.
