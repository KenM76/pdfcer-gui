# `ui-verify/sys/mod`

Every `unsafe` line in the crate, behind one safe API.

## Why the platform code is quarantined here

Two reasons.

**Review surface.** Everything else in this crate is ordinary safe Rust
that a reviewer can read quickly. Interleaving `mouse_event` calls with
check logic would mean every future check has to be reviewed as if it
contained `unsafe`, which is how `unsafe` spreads.

**Portability without pretence.** The harness only works on Windows — it
drives a Windows window through the Windows input API. But the *workspace*
must build elsewhere, or the crate gets dropped from the members list the
first time somebody runs `cargo check` on a laptop. So there are two
implementations of one API: [`win32`] on Windows, [`unsupported`]
everywhere else, which compiles and returns a clear error naming the
platform. The harness reports SKIPPED, not a build failure and not a pass.

## The API

| Function | Job |
|---|---|
| [`find_window_for_pid`] | which top-level window belongs to the process we launched |
| [`window_frame`] | where its client area is, and at what DPI scale |
| [`raise_window`] | bring it to the foreground before driving or capturing it |
| [`cursor_position`] / [`set_cursor_position`] | the pointer |
| [`mouse_button`] | primary button down/up |
| [`key_stroke`] | a virtual key press and release |
| [`capture_screen`] | a desktop region as BGRA pixels |

Note what is absent: there is no `send_message`, and there never will be.
See [`crate::input`] for why posting messages to the window was tried in
this project's predecessor and does not work.

## Item notes

### `mod vk`

Deliberately a tiny closed list rather than a binding of the whole
`VIRTUAL_KEY` space: a harness that can press any key is a harness whose
scripts stop being readable.

### `const CONTROL`

Named `CONTROL` rather than `CTRL` because that is what Windows calls
it (`VK_CONTROL`), and a constant that renames a platform's own
vocabulary makes the next person check twice.

### `const LSHIFT`

Not a synonym for [`SHIFT`] where synthesis is concerned. `VK_SHIFT`
is the "either shift" virtual key that Windows reports in keyboard
STATE; a real keyboard never sends it, and a toolkit that derives its
modifier state from key events — winit does — may not recognise it.

### `const F`

Letters are their ASCII uppercase code point on Windows, which is why
this is `0x46` and not something derived. Only the letters the harness
actually presses are listed: the closed-list rule above applies to
letters more than to anything else, because `pub const A..Z` would be
exactly the "can press any key" the doc comment refuses.

### `const F4`

A check that killed the window and then asserted the preference survived
would be asserting that the 750 ms debounce had already expired --
which is true on a slow run and false on a fast one, and is not the
property anybody cares about. The property is *"I changed it and closed
the program straight away"*, and only a real `WM_CLOSE` reproduces it.

### `const OPEN_BRACKET`

A bare character is the class that has to YIELD to typing, so it is the
one worth driving: a build where `[` fires while a caret is in flight
rotates the drawing instead of inserting a bracket.

### `const PAGE_DOWN`

Pressed rather than reached through the ribbon because `Action::NextPage`
is what the operator's own gesture raises, and because the page-number
box would make the run depend on a text field's focus rules. Note the
Windows name is `VK_NEXT`, not `VK_PAGEDOWN`: the platform's own
vocabulary is kept, per the note on `CONTROL` above, and the doc line
is what tells a reader which key it is.

### `const D`

The closed-list rule again, and this is the first entry that exists to
**type a word** rather than to press a chord.
`checks::dimension_groups` names a new dimension group, and the name is
the one thing in that window a check must supply — the Add button is
greyed with an empty field, deliberately, so a check that cannot type
cannot reach the verb at all.

"Detail" is chosen rather than a nonsense string because the check's
failure text quotes it, and an operator reading *"no group called
Detail appeared in the list"* is being told something about a drawing
they recognise. Added 2026-08-18.

### `const SPACE`

A space rather than a tab, although the defect covers both: a tab in a
single-line egui field is a focus-moving key in most toolkits and would
risk measuring the focus handling instead of the search. The space is
also the character the operator actually named.

### `const DIGIT_2`

Present only as a **control probe**: `Ctrl+2` is bound to
`mode.review` and is a chord the application's key table has always
been able to spell, so a check that gets nothing from it learns that
the keystroke never arrived, rather than that the feature under test
is broken.
