# Keyboard shortcuts

In the program: File ▸ Keyboard shortcuts (`file.shortcuts`), generated from
the keymap. Chords use the keymap grammar, which `PDFCER_DIAG_KEYS` accepts.

## Keymap (chord → command id)

```
Ctrl+N file.new            Ctrl+Alt+N file.new_from_template
Ctrl+O file.open           Ctrl+W file.close
Ctrl+S file.save           Ctrl+Shift+S file.save_copy
Ctrl+P file.print          Ctrl+Shift+C file.copy_page_text
Ctrl+Z edit.undo           Ctrl+Y / Ctrl+Shift+Z edit.redo
Ctrl+X edit.cut            Ctrl+C edit.copy
Ctrl+V edit.paste          Ctrl+Shift+V edit.paste_duplicate
Ctrl+D edit.duplicate      Ctrl+A edit.select_all
Ctrl+F edit.find           Ctrl+E edit.text        Ctrl+Shift+E edit.add_text
Ctrl+Shift+A edit.align
Ctrl+Alt+4/5/6 edit.align_left / align_centre / align_right
Ctrl+Alt+8/2   edit.align_top / align_bottom
Ctrl+Alt+7/1   edit.align_centre_x / align_centre_y
Ctrl+] / Ctrl+[            markup.bring_forward / send_backward
Ctrl+Shift+] / Ctrl+Shift+[ markup.bring_to_front / send_to_back
Ctrl+1/2/3 mode.read / mode.review / mode.edit
Ctrl+0 view.zoom_actual    Ctrl+H view.read_mode   F11 view.fullscreen
Ctrl+Tab / Ctrl+Shift+Tab  view.next_document / previous_document
V view.tool_select   A view.tool_node   T view.tool_text   H view.tool_hand
[ / ]  pages.rotate_left / rotate_right
Alt+Up / Alt+Down  pages.move_up / move_down
```

Setting `paste_chords` (preferences.txt) can swap Ctrl+V and Ctrl+Shift+V
(`new_field_first` | `acrobat`).

## Keys handled outside the keymap

- Delete / Backspace: delete the selection.
- Arrows: nudge the selection 1 pt; with Ctrl, 1/4 pt.
- Ctrl+= or Ctrl++ / Ctrl+-: zoom one rung. Ctrl+wheel zooms; the wheel turns pages.
- PageUp/PageDown/Home/End: move between pages.
- Shift while clicking or boxing adds to the selection; Ctrl takes away.
  Shift while dragging locks the axis.
- Alt held suspends snapping. Tab cycles snap candidates while a measure tool is armed.
- Forms: Tab / Shift+Tab next / previous field; Space ticks a box.
- Escape steps back one thing per press: leave the field, abandon the drag,
  the guide, the measurement or vertex run, put the tool away, clear the selection.
