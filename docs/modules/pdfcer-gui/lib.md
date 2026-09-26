# pdfcer-gui — native desktop shell (rebuild), library root

**This crate is a library with a thin binary in front of it.** The
binary (`src/main.rs`) does one thing: read `argv`, call [`run`]. Every
module, every type and every test lives here.

## Why a library at all, when the product is an executable

Three reasons, and the third is the one that bites daily.

1. **`tools/ui-verify` and any integration test can `use pdfcer_gui::…`.**
   Without a library target every assertion has to cross the process
   boundary, even when the question is really a unit-level one — "does
   this manifest validate?" does not need a window.
2. **`cargo doc` documents something.** A binary crate's rustdoc is
   empty, and this project's standing rule is that the documentation is
   the logic. Docs that cannot be browsed are docs that rot.
3. **`main.rs` stops being a contention point.** With the module tree in
   a binary, *every* new module must edit the same file — which is the
   one guaranteed merge conflict when work runs in parallel, and this
   project runs work in parallel by design. Moving the tree here does not
   remove the shared file, but it removes it from the path of the
   `argv`-and-viewport code that has nothing to do with it.

Converted at the S2 → S3 boundary, deliberately: it changes visibility
across every module, so it wants a moment when nothing else is in
flight, and it wants to happen *before* the panel modules multiply.
`PROJECT_PLAN.md` §4.2b records the decision.

## Where everything lives

| module | responsibility | headlessly testable |
|---|---|---|
| [`app`] | the one owner of state; frame composition; actions | partly |
| [`shell`] | the ribbon/mode/keymap definition, **as data** | **yes** |
| [`viewer`] | page index, zoom ladder, fit math, raster ceiling | **yes** |
| [`render`] | off-thread rasterization; pixmap → texture | worker keys only |
| [`canvas`] | drawing the page, wheel/ctrl-wheel/middle-drag input | geometry only |
| [`find`] | the search query, its options, stepping, staleness and the bar | mostly |
| [`panels`] | the dock's panel bodies, and the page object model behind them | mostly |
| [`text`] | every operator-visible string (the ui-text catalog) | n/a |
| [`diag`] | the opt-in `PDFCER_DIAG` trace channel | n/a |

The split is driven by testability: a windowed UI cannot run on a CI
runner, so every piece of *logic* that could be wrong in a way a human
would notice — an off-by-one page step, a fit scale that overflows an
axis, a zoom that blows the rasterizer's allocation guard — is pushed
into a pure function with a unit test. What is left is wiring. Wiring
can be reviewed; arithmetic needs tests.

## Privacy posture, carried across unchanged

This crate makes no network calls of any kind. The only file it opens is
the one it is asked to open.

## Item notes

### `const MIN_WINDOW_SIZE`

Below this the canvas stops being usable rather than merely small;
enforcing it in the viewport builder is cheaper than defending every
layout against a 200×100 window.

### `fn window_icon`

Everything from here down is the event loop. The caller has already
answered anything that must be decided *before* a window exists — a
terminal invocation must not open a window it then has to be told to
close, which is why argument handling belongs to the binary and not to
this function.

# Errors

Propagates whatever `eframe::run_native` reports: a windowing system
that could not be reached, a graphics backend that failed to initialise.
The window's icon — title bar, Alt-Tab, and the taskbar button.

# Why this exists when the executable already carries an icon resource

They are two different mechanisms answering two different questions, and
doing only one of them leaves a visible gap.

The **resource** in `assets/pdfcer-gui.rc` is read by the shell *without
running the program*: Explorer, the Start menu, and the file-association
dialog the operator's request was about. It is the right and only answer
there, and it cannot be the answer here — winit creates its window class
without an icon, so a running window with no `window_icon` shows the
system's default in its title bar however good the executable's resource is.

This is the run-time half. It is the same art, at 64 px, as raw RGBA —
which is what `egui::IconData` takes, and why the bitmap is checked in
separately from the `.ico` rather than decoded out of it at start-up: the
alternative is carrying a PNG decoder in the binary to recover one
64-pixel image. `tools/make-icon.py` writes both from one render, so they
cannot come to disagree.

# Why the failure mode is an empty icon rather than a panic

`include_bytes!` cannot fail — a missing file is a compile error, so the
bytes are always there and always the right length. The length check below
is therefore not defensive against absence; it is defensive against
`make-icon.py`'s `WINDOW_ICON_SIZE` changing without this constant
following. An `IconData` whose buffer does not match its dimensions is
rejected by winit with a log line nobody reads, so a wrong size would
present as "the icon silently stopped working" — the same class of quiet
failure the encoding bug in the `.rc` was.

### `mod pagedrag`

In `egui::Memory` rather than on `PdfcerApp` because switching documents
resets the panels' state, and switching documents is exactly what a
cross-document drag has to do on its way. See the module header.

### `mod protect`

The headless half of the two Security controls: what the document says
today, which jobs it may be offered, which of the engine's three encryption
verbs a choice reaches, and the atomic write at the end. The window is
`crate::dialogs::protect`; the split is `crate::redact`'s, and for the same
reason — every rule on this surface is a rule about the operator's file, and
a rule that can only be exercised by driving a window is one that gets
asserted once, by hand, and then drifts.

### `mod sign`

**THE ONE `#[cfg]` IN THIS FILE, AND IT IS WHERE R8 SAYS IT BELONGS.**
`SHELL_FRAMEWORK.md` §5b's rule is that *a capability's presence is
expressed by registering its command, and by nothing else* — no `#[cfg]` in
the ribbon, no panel asking whether signing exists. A module declaration is
not the ribbon: it is the boundary the capability is compiled in or out at,
and the engine draws the same one (`pdfcer_core::sign` does not exist
without the feature, so nothing here could reference it if it wanted to).
The ribbon item naming `file.sign` is unconditional and is dropped by the
ordinary merge, with a `CapabilityAbsent` skip reason.

### `mod stamps`

A stamp collection is an ordinary PDF — one file per category, one page
per stamp, names in the catalog's `/Names` → `/Pages` tree — which is why
the operator's ask for *"the same import/export"* Acrobat has has no
serialiser anywhere in here: **Acrobat has no interchange format**, and
handing someone the PDF is the export. This module turns a document into a
plan he can correct, and the plan into that file.

Consumed by `crate::dialogs::stamp_collection` and
`crate::panels::docprops`.
