# `app::keyboard` — the keyboard map, and the guard that must not be wrong

## ★ `DEFECTS.md` D1 — read this before touching the guard

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
