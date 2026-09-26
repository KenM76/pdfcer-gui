# `ui-verify/checks/copy_as_vector`

`copy_as_vector_places_the_measured_order` — the Clipboard group's copy-OUT
is reachable, the press places something, and **the SVG is first**.

# The gap this closes — `OPERATOR_REQUESTS.md` **O120**

The operator, 2026-09-03: *"I'd like to be able to copy and paste anything
to other software - like copy and paste vector graphics into word or
inkscape for example if possible."*

O120's own Status line sets the bar and it is the engine's:

> *"they get ticked when the GUI half is **driven**, not when it compiles."*


# ⚠ This check REPLACES the operator's clipboard, and it must

It clears the clipboard, presses a button whose whole job is to write to the
clipboard, and reads back what landed. There is no version of driving this
feature that leaves the machine's clipboard alone, and pretending otherwise
would mean not checking the thing the operator asked for. Said here so it is
a known cost of running `ui-verify` rather than a surprise.

★ It is also why **no unit test anywhere in this project touches the real
clipboard**: `crate::clipboard` and `crates/native-clipboard` assert on the
bytes that *would* be placed, so a `cargo test` run cannot destroy anything.
The destructive act is confined to a harness the operator starts on purpose.

# ★★★ Why the ORDER is the assertion, and availability is not enough

A pasting application "typically retrieves … the first format it
recognizes". So the design of this feature *is* an order, measured by the
engine against a real Word paste through combridge: `image/svg+xml`, then
`CF_ENHMETAFILE`, then `PNG`, then `CF_DIBV5`.

⇒ A check that asked *"is `image/svg+xml` on the clipboard?"* would **pass
on the build that fails in Word** — the one that placed the raster formats
first. Word takes the first thing it recognises, stores it as a picture, and
nothing anywhere says so. `sys::clipboard_formats` walks
`EnumClipboardFormats`, which enumerates in placement order, precisely so
this check can assert on the *prefix* rather than on the *set*.

★★ **Windows synthesises formats, and they come after.** A placed
`CF_DIBV5` makes `CF_DIB` and `CF_BITMAP` appear too. That is why this
asserts a prefix and ignores the tail: a build that placed all four
correctly will show more than four entries, and demanding exactly four would
fail a correct build on a Windows behaviour nobody controls.

# The five links this needs driving for

1. **the control exists on the Clipboard group.** `shell::manifest::edit`
   lists five members in a unit test whatever the ribbon draws; whether a
   fifth icon reached the screen, inside the band's height and not pushed
   into the overflow, is a question about a laid-out frame.
2. **pressing it reaches the handler.** `shell::commands::reach` proves an
   arm exists by reading the source; it cannot prove the click lands on it.
3. **the renderers run against the live session** — the `DocumentView` with
   its overlay and staging buffer, so a copy carries unsaved edits.
4. **the Win32 transaction actually places anything.** Nothing in a unit
   test can reach `SetClipboardData`; `crates/native-clipboard`'s `unsafe`
   is verified by construction and by review, and **this is the only thing
   in the project that observes its effect.**
5. **the order survives the transaction.** The order is decided in
   `crate::clipboard::ORDER` and asserted there — but a staging bug, a
   `HashMap` somewhere, or a future "tidy" that sorts the entries would
   reorder them between the assertion and the clipboard.
