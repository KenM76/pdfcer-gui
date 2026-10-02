# `pdfcer-gui/app/frame`

## Item notes

### `fn scripted_invoke`

Consumed on the first frame that reads it and `None` for ever after, so a
scripted invocation happens exactly once rather than sixty times a second.

# Why this seam exists, and it is R1 rather than convenience

R1 says a phase is not done until its behaviour is asserted by **driving the
running binary**. `tools/ui-verify` does that by moving the operator's real
mouse and keyboard — which means it cannot run while he is at the machine,
and this project's own memory records him saying *"I'm working on the pc"*
mid-session and everything after it having to be headless.

Two features have already needed a seam of exactly this shape and got one:
`PDFCER_DIAG_OPEN_PATH` (a native file picker is a hard wall for synthetic
input) and `PDFCER_DIAG_DROP_PATH` (a drop originates in Explorer and cannot
be synthesised at all). `app::dropped`'s note is the argument, and it
generalises:

> *"without this, drag-and-drop would be the one feature in this shell that
> R1 could not reach — implemented, unit-tested, and never once exercised in
> a running window, which is exactly the state R1 exists to forbid."*

This one generalises it one step further. **An offscreen window cannot be
driven by OS input at all** — `D:/dev/rag/egui/postmessage_to_offscreen_eframe_window_drops_pointer_button.md`
— so a headless run can launch the application and read its trace and can
press nothing. `PDFCER_DIAG_VIEWPORT` already gives a real, laid-out,
invisible window; this gives it something to do.

It landed with `dialogs::host` on 2026-08-20 because that change had no
other honest oracle: *"a dialog opened in its own OS window"* is a fact
about a second viewport that no unit test can observe and no screenshot of
the main window contains.

# It reaches the same choke point an operator's chord does

Deliberately. `dispatch_command` is where mode gating, the decline
retirement and the command registry all live, and a seam that went round it
would prove that a *different* path works. What this substitutes is the
keystroke, not the dispatch.

# Why a list of ids and not a script

Because a grammar is a language and these are doorbells. The variable takes
a comma-separated list, rung one per frame in order — `mode.edit,
edit.form_text_field` — and that is the whole of it: no arguments, no
conditionals, no state. Each id is one the command registry already
publishes, dispatched through the same choke point a chord reaches. `diag`'s own header
records that the old shell's 800-line `PDFCER_DIAG_SCRIPT` harness was
deliberately not salvaged — *"salvaging a script grammar before there is a
harness to run it would be shipping a language with no speakers"* — and
`tools/ui-verify` is that harness now. What it lacks is a way in on a
machine whose desktop is occupied, and one command id is the whole of that.

### `static RUNG`

A counter rather than a flag, and the header's *"one command and not a
script"* argument survives that intact: **a list of doorbells is not a
grammar.** There is no syntax to learn, no arguments, no conditionals
and no state — the ids are the same ids the registry already publishes,
and each is dispatched through the same `dispatch_command` a keystroke
reaches.

Why a list and not a single id: **a capability can take two commands to
reach.** Arming
a form-field tool needs Edit mode first, because the arm declines
without `edit_content`. With a single-shot variable, every feature gated
behind a mode was unreachable headlessly — implemented, unit-tested and
never once exercised in a running window, which is precisely the state
R1 exists to forbid.

### `fn on_exit`

The operator: *"it should remember my page display preferences from my
last closing of the program."*

# What was wrong, and it is the shape worth naming

[`crate::app::persistence::LayoutStore::flush`] exists, is documented,
is tested — and had **no production caller**. Its own doc comment says
what it is for in as many words: *"For an exit path, which must not
lose the last change to a debounce that had not yet expired."* There
was no exit path. `impl eframe::App for PdfcerApp` implemented `ui` and
nothing else, `run_native` installed no exit callback, and there is no
`Drop`.

So the layout is written 750 ms after it changes, with a 5 s ceiling —
and **a change made in the last 750 ms before the window closes was
silently thrown away.** The debounce was correct, the ceiling was
correct, and the last write of every session was a coin toss.

It reaches the operator through page display, which is why it lands
under O80 rather than as a housekeeping note. The active ribbon **mode**
rides in `layout.ron`, and the mode is what picks
`PageDisplay::default_for_mode` for a document with no remembered
entry. Switch to Edit, close the program within three quarters of a
second, reopen: the mode is Read again, Read defaults to continuous,
and from his chair the program forgot which way it was showing pages.

# Why `on_exit` and not `save`

`save` is only called when eframe's `persistence` feature is enabled,
and this application deliberately does its own persistence — see
[`crate::app::persistence`]'s header on why the location is
`pdfcer-core`'s decision rather than the platform's. `on_exit` is
unconditional and runs after `save`, so it is the hook that is actually
there.

The `glow` form of the signature, because that is the backend this
workspace's `eframe` features select. The parameter is unused: this
flushes a file, it does not touch the GPU.

### `fn ui`

The trait hands a root [`egui::Ui`] rather than a [`egui::Context`]
(`eframe-0.35.0/src/epi.rs:176`), and panels are added *inside* that
`Ui` — `CentralPanel::show(ui, …)`, not `show(ctx, …)`. Anyone
arriving from an older eframe, or from a code sample, will write
`update` and get a "not a member of trait" error whose message does
not say what to write instead; hence this note.

The context is cloned out at the top because the raster bookkeeping
needs it after the panel closure has ended, and `Ui::ctx()` borrows.
