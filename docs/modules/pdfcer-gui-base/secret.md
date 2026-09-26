# `secret` — a string the operator typed that must never reach a log

One type, [`Secret`], and its whole reason for existing is its [`Debug`]
implementation.

## The hazard, stated before the type

A document password travels from a text field, through
`pdfcer_gui::app::actions::Action`, into the action queue, and out again in
`pdfcer_gui::app::lifecycle`. Every step of that is ordinary — and
`Action` derives `Debug`, the program traces liberally to stderr under
`PDFCER_DIAG`, and **`tools/ui-verify` captures that stderr to a file it
keeps as evidence**.

So a single `format!("{action:?}")` anywhere on that path writes the
operator's password into `target/ui-verify/*.trace.txt`, in plain text, on
disk, in a directory whose whole purpose is to be kept and read. It would
not fail anything. It would not look wrong in review — a `{:?}` on an action
is exactly what a diagnostic line is made of.

⇒ **The fix is not a rule saying "do not print it".** This project has spent
several corrections learning that a rule written beside the code it governs
is not a mechanism: the rotation-button gate had its rule in its own module
header sixty lines above the code that broke it. The fix is a type whose
`Debug` cannot print the thing.

## What it deliberately does NOT do

**It does not zero its buffer on drop.** A `String`'s allocation can be
moved by the allocator and copied by `realloc` before any `Drop` runs, so
zeroing the final buffer is a gesture rather than a guarantee, and shipping
a gesture named `Zeroizing` would tell a reader they had a property they do
not. If in-memory scrubbing is ever wanted it needs a fixed buffer that is
never reallocated, and that is a different design with its own argument.

What this type guarantees is exactly one thing, and it is the thing that was
actually going to go wrong: **the value cannot be formatted.**

## Why `PartialEq` is here

`pdfcer_gui::app::actions::Action` derives it, and every variant must. The
comparison is the ordinary string one — deliberately **not** constant-time,
because nothing here compares a secret against a stored secret. The only
comparison that matters is `pdfcer-core`'s, inside the document's own
authentication.
