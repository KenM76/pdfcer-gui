# `canvas::textedit::ime`

Composed input — Chinese, Japanese and Korean input methods, dead-key
accents, the Windows emoji panel — typed into a text edit on the canvas.

## Why the draft must ask

egui asks the platform for an input method only when a frame's
`PlatformOutput::ime` is `Some`; egui-winit then calls `set_ime_allowed(true)`
and places the candidate window at `cursor_rect`. A focused `TextEdit` sets it
for itself. The text-edit draft is painted into the canvas and is not an egui
widget, so without [`show`] nothing sets it and Windows never starts a
composition: an input method types nothing.

`show` runs from `paint::preview`, once a frame while the draft's page is
drawn. It yields when an egui text field holds the keyboard (the Find bar, a
properties field), because that field asks for its own. The area is the
draft's line box in screen space; the candidate window opens at the line's
start. A move of the area is traced as `text-ime-area x= y= w= h=`.

## Composition and commit

- `ImeEvent::Preedit` is held in context memory, never in the draft, and is
  drawn on the editor's surface just under the line box with an accent
  underline (region `text-ime-preedit`). That is a pre-commit affordance, not
  content: the page's text is untouched until the commit. Each preedit is
  traced as `text-ime-preedit chars=N`; an empty one ends the composition.
- `ImeEvent::Commit` ends the composition and is typed at the caret through
  `Keys::type_text`, exactly as a keystroke: the run's font sieves it, a key
  the font lacks is named by the refused-keys notice, and it joins the typing
  undo run.

egui-winit already drops the `VK_PROCESSKEY` key events Windows emits for keys
the input method consumed, so a composed key is never also typed as a raw key.

## Driven by

`an_ime_composition_types_once_committed` (`docs/modules/ui-verify/checks/ime.md`).
The scripted pointer's `preedit` and `commit` steps post the two `ImeEvent`s.
