# `app::keyboard` — the keyboard map, and the guard that must not be wrong

## `DEFECTS.md` D1 — read this before touching the guard

The old GUI's keyboard map guarded its unmodified-key bindings with:

```ignore
let typing = ctx.egui_wants_keyboard_input();
```

**That predicate does not mean what its name says.** Verified in the
vendored source at `egui-0.35.0/src/context.rs:2884-2886`:

```ignore
pub fn egui_wants_keyboard_input(&self) -> bool {
    self.memory(|m| m.focused().is_some())
}
```

— *any* focused widget, including the canvas. Its own doc comment
immediately above says *"egui is currently listening on text input (e.g.
typing text in a `TextEdit`)"*, which is what the name and the comment
both promise and what the implementation does not deliver. This is an
egui API footgun, not a careless read.

The consequence was the defect the operator reported as *"I can't even
click on an object and delete it by hitting the delete key."* The canvas
calls `request_focus()` on click, and because the widget is recreated
every frame its id stays live — so from the **first canvas click
onward**, `typing` was permanently `true` and every unmodified binding
was suppressed. Delete, Backspace, PageUp, PageDown, Home, End and the
rotate keys all died from one click, and the deletion logic downstream
was correct and simply unreachable.

**The fix, applied here from the first line of code:**

```ignore
let typing = ctx.text_edit_focused();
```

`text_edit_focused()` (`egui-0.35.0/src/context.rs:2889-2895`) resolves
the focused id and checks whether a `TextEditState` exists *for that id*.
It therefore preserves the guard's real intent exactly — a focused text
field keeps its unmodified keys — while a focused canvas, button or tab
does not steal them. A `DragValue` in keyboard-edit mode registers its
`TextEdit` under the *same* id it focuses, so numeric property fields
still count as typing, which is the case the original guard was written
for.

### Why the old test did not catch it, and what replaced it

The original had exactly one test, and it built a bare
`egui::Context::default()` with **no widgets** — so `memory.focused()`
was always `None`, `typing` was always `false`, and the single property
that breaks in the real application was structurally absent from the
only harness that exercised the function.

[`tests::a_focused_non_text_widget_does_not_suppress_unmodified_keys`]
is the test that would have caught it: it drives a real `Context`
through two frames, takes focus on a plain widget id in the first,
asserts in the second that `egui_wants_keyboard_input()` really is
`true` (so the test is known to be exercising the failing condition,
not a vacuous one), and then asserts the unmodified bindings still fire.
Swapping the guard back to `egui_wants_keyboard_input()` fails it.

## Why this is a pure-ish function taking a page count

[`collect`] takes `page_count: Option<usize>` rather than the app's
`Status`, so it can be tested against a real `egui::Context` without
constructing a `Document`. `None` means "no document open", and the
whole map is then not installed — a binding that fires with nothing open
is a binding whose action has to defend itself.



- The manifest keymap is what `egui_shell::menu::Shortcuts` inverts to
  draw a context menu's right-aligned chord hint, so a right-click on
  blank paper offered *Actual size — Ctrl+0*.
- `crate::text::commands::view_zoom_actual`'s ribbon tooltip named
  `Ctrl+0` too, and `crate::text::commands::mode_review`'s named `Ctrl+2`.
- This module got there first and did neither, because nothing dispatches
  the manifest keymap: `egui-shell` deliberately does not own key
  handling (`egui_shell::ribbon`'s header: *"the application owns the
  question of what has focus and what a chord means"*).

The visible cost was `crate::text::status::fit_actual_size_tooltip`,
which had to advertise **no chord at all** — with a test pinning the
omission — because it could not honestly name one.

### The fix: the manifest binds, this module enacts

There is now exactly one place a chord is *bound to a meaning*, and it is
the manifest keymap. [`commands`] spells the key it saw the way a
manifest writes it, looks the spelling up in the keymap, and returns the
**command id** — which `crate::app::PdfcerApp::dispatch_command` then
dispatches through the same arm a ribbon click reaches. Rebind `Ctrl+0`
in a customization layer and the keyboard follows, with nothing here to
edit. [`tests::the_derived_chords_follow_the_keymap_rather_than_this_module`]
is the proof: it hands `commands` an invented keymap and watches the
chord change meaning.

### And the chords this module still owns outright

[`collect`] keeps the *viewer* chords, which the manifest documents as
deliberately absent from its keymap — *"Viewer navigation, handled in the
app's own keyboard layer against the view state. They are not ribbon
commands and putting them here would give them a second owner."*
[`OWNED`] names every one of them, in every spelling a manifest might
write, and [`tests::no_chord_has_two_owners`] fails — naming the chord
and both claimants — the moment the keymap claims one back.

## The bindings, and why these

| keys | action | owner |
|---|---|---|
| Ctrl+`+` / Ctrl+`=` | zoom in one rung | here ([`OWNED`]) — what browsers, Acrobat and every PDF reader do |
| Ctrl+`-` | zoom out one rung | here ([`OWNED`]) |
| PageDown / PageUp | next / previous page | here ([`OWNED`]) — the unmodified keys D1 killed |
| Home / End | first / last page | here ([`OWNED`]) |
| Ctrl+`N` | `file.new` → a blank document | the manifest keymap |
| Ctrl+`O` | `file.open` → the file picker | the manifest keymap |
| Ctrl+`F` | `edit.find` → open or close the Find bar | the manifest keymap |
| Ctrl+`0` | `view.zoom_actual` → actual size | the manifest keymap |
| Ctrl+`1` / Ctrl+`2` / Ctrl+`3` | `mode.read` / `mode.review` / `mode.edit` | the manifest keymap |

Ctrl+`=` is bound alongside Ctrl+`+` because `+` is a shifted key on
most layouts and requiring the shift makes "zoom in" a three-finger
chord. Every browser accepts both; so does this.

### Why `Ctrl+0` is actual size and not fit page


What is left is the browser convention — `Ctrl+0` returns to 100 % — and
it is also what two operator-visible strings already claimed before
anything reached them. So the manifest's reading wins, and the two fit
modes keep their status-bar buttons, their View ▸ Zoom controls and their
`canvas.empty` context-menu entries as the routes in. **`FitMode::Width`
is still reachable**, which was the structural reason this module bound
`Ctrl+2` at S0, back when no ribbon and no status bar existed.

Note that these chords require egui's own `zoom_with_keyboard` to be
switched **off**, or it consumes them to rescale the entire user
interface — see [`crate::app::configure_context`]. Without that, the
chords would silently do the wrong thing.

## The scripted keystroke

The viewer chords above are the *only* route to their verbs, so a window
placed off the desktop — which takes no OS input at all — could not be
made to zoom or page at all. [`scripted`] is the seam that closes that,
and it carries why a seam is legitimate there where registering a command
would not be.

## Item notes

### `fn parse_chord`

A table that must be kept in step with a manifest by hand will fall out of
step with it, and the failure is silent in both directions: the keymap
entry looks bound, the menu hint looks true, and the key does nothing. So
nothing is kept in step any more — the manifest is parsed, and a chord
fires if and only if it can be spelled.

# The grammar

`Modifier+Modifier+Key`, modifiers in any order, matching what
`built_in.ron` already writes: `Ctrl`, `Shift`, `Alt` (and `Cmd`/`Command`
as aliases for `Ctrl`, since egui's `command` is Ctrl everywhere and Cmd on
macOS). The key is whatever [`Key::from_name`] accepts, which is egui's own
parser — so `[`, `Down`, `F11`, `0` and `E` all resolve, and the manifest
cannot invent a spelling this shell then fails to honour.

Returns `None` for a chord that cannot be spelled, which
[`tests::every_chord_the_manifest_binds_actually_fires`] turns into a build
failure rather than a dead key.

### `fn actions_for`

`Context::run_ui` (egui 0.35 renamed `run`) hands the closure a root
[`egui::Ui`] rather than the context, and returns a `#[must_use]`
`FullOutput` this harness has no use for — hence the `let _`.

### `fn a_focused_non_text_widget_does_not_suppress_unmodified_keys`

Drives a real `Context` through two frames. The first takes focus on
a plain (non-text) widget id — which is exactly what the canvas does
on click. The second asserts two things in order:

1. `egui_wants_keyboard_input()` is genuinely `true`, so the test is
   known to be exercising the failing condition rather than passing
   vacuously. This is the assertion the original test was missing,
   and its absence is why the defect shipped.
2. The unmodified bindings still fire.

Swap `text_edit_focused()` back to `egui_wants_keyboard_input()` in
[`collect`] and this test fails.

### `fn a_zoom_chord_is_matched_on_the_events_own_modifiers`

One long frame — a dense sheet rasterizing — can deliver the press and
the modifier's release together. `InputState::modifiers` then reports
`Ctrl` up while the event that arrived still carries it down, and a
chord read from the frame's snapshot is dropped silently. It is dropped
precisely when the operator wants zoom most, because a long frame is
what makes him reach for it.

⚠ [`key_press`] sets both places to the same `Modifiers`, so every
other test in this module is blind to the distinction by construction.
This one sets them apart, in both directions:

1. event carries `Ctrl`, frame does not → the chord must fire;
2. frame carries `Ctrl`, event does not → the chord must not fire.

Assertion 1 alone would pass on code that read neither clock and fired
on the bare key.

### `fn every_viewer_chord_has_a_spelling_the_scripted_seam_accepts`

This is the property the document-count ladder rests on: the verbs in
[`OWNED`] have no registered command id, so the seam is their *only*
headless route, and a viewer chord the seam cannot spell is a verb no
driven check can exercise at all.

⚠ **At least one spelling per chord, not every spelling.** [`OWNED`]
lists the alternatives a *manifest author* might write so that
[`tests::no_chord_has_two_owners`] catches a conflict however it was
spelled — `"Page Down"` with a space is in there for that reason alone,
and [`Key::from_name`] does not accept it. Asserting that every entry
parses would be asserting something that table never promised; a
manifest that actually bound the spaced spelling is caught by
[`tests::every_chord_the_manifest_binds_actually_fires`] instead.

The seam consumes a rung whether or not a spelling resolves, reporting
`spelled=no`, so an unspellable entry costs one step of a ladder rather
than wedging it — the right failure, and still a failure.

### `fn a_digit_reaches_no_command_without_its_modifier`

Same rule as the zoom chords, checked on the derived path: a bare `2`
belongs to the page-number box, and a keymap entry spelled `Ctrl+2`
must not be satisfied by a `2` with nothing held.

### `fn built_in_keymap`

Read from [`crate::shell::manifest::built_in`] rather than hand-built,
because a guard that checks an invented keymap guards an invented
defect.

### `fn no_chord_has_two_owners`

This is the regression test for the defect in the module header. It
walks every chord [`collect`] binds outright, in every spelling a
manifest might write it, and asserts the keymap claims none of them.
Reintroduce `Ctrl+0` — or `Ctrl+Plus`, or `PageDown` — on either side
and this fails **naming the chord and both claimants**, which is the
property that makes it useful: a failure that said only "keymap
mismatch" would send the next person looking in the wrong file.

### `fn every_chord_the_manifest_binds_actually_fires`

The test whose absence let fourteen shortcuts ship dead. Its
predecessor asserted the right property and then swept a *third* of the
keymap - `if !is_digit_chord { continue; }` - while its own doc comment
stated the general rule: *"a chord this module cannot see would then be
a keymap entry, a menu hint and a tooltip promising something no
keypress delivers."* The reasoning was right and the enforcement was
narrowed, so `Ctrl+Z`, `Ctrl+Y`, `Ctrl+S`, `Ctrl+E`, `Ctrl+Shift+E`,
`F11`, `[`, `]`, `Alt+Up`, `Alt+Down` and four more sat in the manifest,
printed themselves in menus, and did nothing.

So this sweeps **every** entry, and it does not check that a chord can
be *spelled* - it presses it and checks the command comes back. A
spelling test would have passed on a dispatcher that spelled the chord
correctly and then filtered it out for holding Shift, which is exactly
how `Ctrl+Shift+E` died.

### `fn a_chord_survives_its_modifier_being_released_in_the_same_frame`

The regression test for a defect that only appears under load and reads,
from outside, as harness flakiness.


This frame is exactly that: a `Ctrl+Z` key event carrying its own
modifiers, followed by the modifiers going empty before the frame ends.
Revert [`commands`] to the frame snapshot and it fails.

### `fn a_longer_chord_does_not_also_fire_the_shorter_one`

`Ctrl+Z` is undo and `Ctrl+Shift+Z` is redo. egui's own
`Modifiers::matches_logically` is permissive - it asks only that the
pattern's modifiers are present - so a dispatcher built on it fires
**both** on one keypress, and which of undo and redo wins is iteration
order. This is why [`commands`] compares the three flags exactly.

### `fn a_focused_text_field_silences_the_chords`

Both halves of the guard matter, and this is the `egui::TextEdit` one:
`Ctrl+Z` inside a field is the field's undo, and a document-level undo
firing underneath it would revert an edit the operator never touched.

### `fn a_canvas_text_draft_silences_the_bare_chords`

The half `text_edit_focused()` cannot see. The caret this shell paints
on the page is not an `egui::TextEdit`, so egui reports no focused text
field for an operator who is mid-word - and `[` is bound to
`pages.rotate_left`. Without the draft term, typing a bracket rotates
the drawing and inserts nothing.

### `fn ctrl_0_names_the_actual_size_command`

Both halves matter. The first is the decision — the browser
convention, and what `view_zoom_actual`'s tooltip and the
`canvas.empty` menu hint have claimed all along. The second is the
structure: this asserts the id, not an [`Action`], because this module
no longer knows what `view.zoom_actual` *does*.

### `fn ctrl_o_names_the_open_command`

The chord was in the keymap and printed in `file_open`'s tooltip from
the day the ribbon landed, and pressing it did **nothing**: the shell
carried a hand-written spelling table that held only digits, so the key
could not be spelled, so nothing was looked up. Two operator-visible
surfaces named a chord that did not exist.

It was fixed by adding a row to that table, which fixed exactly this
chord and left thirteen others dead — the table is now gone and
[`parse_chord`] reads the manifest, so the class is closed rather than
the instance. See `every_chord_the_manifest_binds_actually_fires`.

Asserted through the real keymap rather than an invented one, so the
test fails if the binding is ever removed from the manifest as well as
if the spelling is removed from here.

### `fn the_mode_chords_name_the_mode_commands`

`MODES_AND_PANELS.md` Part 1 §6 specifies these three, and all three
tooltips in `crate::text::commands` name them. This is what makes
those three sentences true.

### `fn the_derived_chords_follow_the_keymap_rather_than_this_module`

Hands `commands` a keymap that binds `Ctrl+0` to something else
entirely and watches the chord follow it. This is the test that would
fail if someone "simplified" the lookup back into a `match` here —
which is exactly how the two-owner defect was written the first time.

### `fn every_chord_the_manifest_binds_is_written_down_in_the_manual`

The manual's *"Every keyboard shortcut"* section documented **33** of
the **40** chords `built_in.ron` binds. The seven it did not mention
were `Ctrl+A`, `Ctrl+D`, `Ctrl+Shift+V`, `Ctrl+[`, `Ctrl+]`,
`Ctrl+Shift+[` and `Ctrl+Shift+]` — and `Ctrl+D` and the four arrange
chords were absent from the **whole document**, not merely from the
table.

The operator's report that started this was about a different feature
he could not find; the shape is the same one either way.

Both numbers are measurements of that day's tree and are *not* what
this test asserts — it reads the manifest every run, so it stays right
as the keymap grows. Quoted here only to record the size of the gap an
unchecked table had been carrying.

Nothing could have caught it. Grepping the repository for `MANUAL.md`
on that date returned no gate, no test and no tool: a section whose
entire job is completeness had no instrument, and a hand count of a
forty-row table read as done every time anybody looked.

# Why the assertion runs in this direction

The list of chords is **read out of the manifest**, never written here.
A test that carried its own list would go stale in exactly the way the
manual did, and it would do it silently, because the count would still
add up against itself. Bind a new chord and this fails until the manual
says what it does. That is the only property worth having.

The reverse direction — a chord in the manual that nothing binds — is
**not** asserted, and deliberately. The manual writes `Ctrl + mouse
wheel`, `Escape`, `Alt+F4` and a handful of drag gestures that are not
keymap entries at all, so a reverse sweep would be a list of exceptions
rather than a measurement. What a stale manual entry costs is a reader
pressing a key that does nothing; what a missing one costs is a
capability that ships unreachable. The expensive direction is the one
under guard.

# How a chord is recognised in the prose

The manual bolds every chord, so the haystack is the set of `**…**`
runs. Two rewrites are applied to each run, and **both are shapes of
the prose, not names of commands** — no command id and no chord
spelling is hard-coded below:

1. **Arrow glyphs become egui key names.** The manual writes
   `**Alt+↑**` because that is what is on the keycap; the manifest
   writes `Alt+Up` because that is what [`Key::from_name`] accepts.
2. **A slash list distributes its modifier prefix.** `Ctrl+X / C / V`
   is three chords in one row, and spelling it out as three rows would
   make the table worse to read in order to make this test simpler to
   write. The prefix is whatever precedes the last `+` of the first
   element, applied only to later elements that carry no `+` of their
   own.

A rewrite can only ever **add** spellings to the haystack, so the worst
it can do is let a genuinely missing chord pass by coincidence — and
for that it would have to coincide with a chord the manual does
document. It cannot produce a false failure.

### `fn an_unbound_chord_and_an_absent_keymap_both_produce_nothing`

`Ctrl+1` in a keymap that does not mention it must not fall back to
some default this module remembers — there is no such memory, and the
test is what keeps it that way.
