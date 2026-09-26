# `pdfcer-gui/canvas/textsel/clipboard`

## Item notes

### `fn an_idle_frame_asks_for_no_text_chord`

It is asserted at the level of the **predicate** rather than by counting
extractions, because that is where the property lives: the caller's `if
let` cannot fetch anything when this answers `None`, whatever else it
does.

### `fn a_focused_text_field_keeps_the_text_chords`

These are the two chords an operator presses *inside* the Find field. A
canvas that took them would select and copy the page instead of the text
being typed, which is the same failure D1 produced with Delete and would
be more surprising, because the operator can see the field they are in.

Built against a **real** `TextEdit`, for the reason
`canvas::keys::a_focused_text_field_keeps_delete_for_itself` gives:
`text_edit_focused()` resolves the focused id and looks for a
`TextEditState` under it, so a hand-requested focus on a bare id would
pass vacuously.

### `fn pending_key`

# The defect this split closes, and it was found by driving the binary

The first version of this feature had one function that took a
[`PageContext`], read the two chords, and acted. `canvas::interact` therefore
had to build that context — which means calling
[`crate::app::state::OpenDoc::page_text`] — **on every frame in a reading
mode**, in order to discover that no chord had been pressed.

The cache made that one extraction rather than sixty a second, so no test
noticed and the trace showed exactly one `page-text` line, as designed. What
driving the real binary showed was *when* that line appeared: at **open**,
before the operator had touched anything, at a measured **392 ms** on
`ncored-benchmark-cad-drawing.pdf`. A reader opening a dense drawing paid
four tenths of a second for a gesture they had not made and might never make.

It is the same failure [`crate::app::state::OpenDoc::page_objects`]'s own
gate exists for, in that method's own words: *"asking for it on a frame that
has no hit test to do would decompose the page the first time the operator
merely zoomed"*. The difference is that the decomposition's gate is in its
caller, and this one was missing — because the expensive value was being
fetched in order to answer a question that did not need it.


# The guard

`text_edit_focused()`, the `DEFECTS.md` D1 predicate, and this is the
sharpest instance of D1's own reason in the product: `Ctrl+A` and `Ctrl+C`
are what an operator presses **inside** the Find field or the status bar's
page box, and a canvas that took them would make the two most reflexive
keystrokes in the application select and copy a page instead of the text they
were typing. It is `text_edit_focused()` and never
`egui_wants_keyboard_input()` — see `app::keyboard`'s header for why that
distinction is not a nicety.

# Why Copy wins a frame carrying both

Unreachable from a keyboard, and answered anyway: the **narrower** verb wins,
so a synthetic frame carrying both copies what was selected rather than
copying the whole page it selected a microsecond earlier. A rule that is
stated cannot be got wrong by a later reader who reaches that state from a
script.

### `fn apply_key`

Handled here rather than in [`crate::canvas::keys`] because both verbs need
that extraction, and `canvas_keys` is deliberately a *document-free* function
a headless `egui::Context` can drive end to end. Escape **is** in
`canvas_keys`, because clearing needs nothing but the field, and because it
has a precedence question to answer that these two do not.

Traces its own outcome: the caller cannot see which verb fired, and a
selection is otherwise invisible from outside the process — see
[`crate::canvas::trace::text_selection`] for that argument.

### `fn copy`

The one place this shell writes the clipboard. Three verbs reach it — the
canvas selection's Ctrl+C, `file.copy_page_text` and
`file.copy_document_text` — and routing all of them through one function is
what makes the trace line below a complete record of what pdfcer has copied
rather than one of three partial ones.

It raises no [`crate::app::actions::Action`], and that is the same call
`file.print` makes for the same reason: the funnel exists for work that
touches a document or that must not happen mid-frame, and a clipboard write
is neither. `egui::Context::copy_text` queues an output command that the
backend spends after the frame anyway.

An **empty** string is refused rather than written. Copying nothing would
silently destroy whatever the operator had on their clipboard — which is
their data, from another application, and not pdfcer's to discard — and the
decline is traced so the difference between "copied nothing" and "did not
run" is on the record.
