# Paste-in and drop from other programs (O279) — plan

Ken: "I can't copy from another program such as an image from the snipping tool and paste onto a page. I can't copy paste text such as from word either. Also we should be able to support dragging and dropping an svg file onto a page. this is just the tings I've tried, but we should aim for full support of whatever acrobat can support of these sorts of features."

## Why each attempt fails
- (a) Snipping Tool image + Ctrl+V. egui-winit 0.35 (`src/lib.rs`, `is_paste_command`) only calls arboard `get_text`. If there is no text it emits nothing (no Paste, no Key), so the keystroke vanishes. The ribbon Paste reads only the in-memory clip (`app/dispatch/clipboard.rs paste()` → `canvas::clipboard::read`). No image format is ever read. The engine BMP reader also refuses a raw CF_DIB: it has no "BM" header and uses BI_BITFIELDS.
- (b) Word text + Ctrl+V. `Event::Paste(text)` arrives (`app/keyboard.rs clipboard_chord`), but `paste()` discards the text. The result is either "Nothing has been copied yet" (`text/clipboard.rs`) or the STALE pdfcer clip. The clip is never compared with the OS clipboard, so that is a second bug. External text is used only inside an open caret draft (`canvas/textedit/edits.rs`).
- (c) SVG drop. `app/dropped.rs` turns `.svg` into `Unknown("svg")`, and `text/dropped.rs` says "It does not read .svg files". The engine has no SVG import. Separately, `landing.at` is ignored: an image drop opens the insert dialog and fits the image to 90% of the page. Only the first dropped file is used.
- Off-window OLE drag (text or image dragged from another app) is impossible today: winit's drop target accepts only CF_HDROP (winit-0.30.13 `drop_handler.rs`).

## Acrobat parity targets
- Image on the clipboard + Ctrl+V. Acrobat pastes a Stamp comment. Ours:
  - Review: an image stamp annotation;
  - Edit: a page-content image;
  - placed at the pointer if it is over a page, otherwise the view centre;
  - one undo.
- Text with a caret: already works.
- Text with no caret:
  - Edit: a new wrapped text box (`CommitAddText`);
  - Review: a FreeText annotation;
  - placed at the pointer.
- EMF/SVG/PDF on the clipboard: vector import, which goes beyond Acrobat. Blocked on the engine.
- Create PDF from Clipboard, and Insert Pages from Clipboard.
- Image file dropped on a page: placed at the drop point. PDF dropped on the canvas: ask Open / Insert pages / Place as artwork (default Open). `.txt` dropped: new pages. Office file: an honest refusal.
- OLE drag from another window: deferred; it needs our own IDropTarget.
- Form-field paste already works. Image into a button icon needs the engine (G-5).
- Do NOT copy Acrobat's quirk where paste fails while the thumbnails panel has focus.

## Engine (v0.72.0)
- Has:
  - `image_import::import` (PNG/JPEG/BMP-with-header/TIFF);
  - `EditSession::add_image`, `add_text(AddTextRequest)`, `place_text`, `place_page_artwork`, `paste_objects`, `insert_pages`;
  - `pdfcer_render::export::encode_png`.
- The GUI decodes CF_DIBV5/CF_DIB → RGBA → PNG → `image_import`, with no engine change. Prefer the registered "PNG" format when present.
- G requests to file:
  - G-1 `svg_import`: SVG → Form XObject/ObjectClip, plus `add_svg(page, rect, bytes)`. usvg is already in pdfcer's lockfile as a dev-dep.
  - G-2 `emf_import`.
  - G-3 `add_image_stamp(page, rect, &ImportedImage)`.
  - G-4 GIF decode.
  - G-5 button icon from an image.
  - G-6 FreeText authoring, only if it is missing.
- Dependencies: GUI-side usvg would newly ship in the binary, so it needs Ken's approval; prefer G-1. svg2pdf is not allowed.

## Steps (each: driven ui-verify check + falsification)
- P0 ui-verify `sys/win32.rs` (or a new `sys/win32_clip.rs`): `set_clipboard(&[(fmt, bytes)])`, `register_format`. Self-test round trip. Falsify: assert a different format.
- P1 OS clipboard read + image paste via ribbon Paste (Edit mode).
  - New `native-clipboard/src/read.rs`: `sequence()`, `available()`, `get(fmt)` for PNG, DIBV5, DIB, UNICODETEXT, image/svg+xml, HTML Format, ENHMETAFILE, HDROP. Off Windows it is a stub.
  - New `pdfcer-gui-base/src/clippaste/{mod,dib}.rs`: `enum Incoming { Image, Text, Svg, Emf, Files }`, priority SVG > EMF > PNG > DIBV5 > DIB > text. DIB decode tests cover 24-bit, 32-bit, BITFIELDS and top-down.
  - `paste()`: use the external clipboard when there is no clip or the clip's recorded clipboard sequence number differs. Record the sequence in `Clipped` on store; this fixes the stale clip.
  - In Edit mode, raise `Action::InsertImage` at the pointer at natural size, clamped to the page. Update the `appaction.rs` doc.
  - In Review mode, refuse for now. Add a gate in `modecapability.rs`.
  - Trace `clip-pasted source=os kind=image page= rect=`.
  - Check `paste_os_image`: a BI_BITFIELDS DIB at 64×32 → 48×24 pt centred within 2 pt of the pointer; one undo removes it.
  - Falsify: move the pointer and the rect moves; a text-only clipboard gives no image trace; a pdfcer copy followed by an external set makes the external content win.
- P2 Ctrl+V with no text on the clipboard.
  - In `native-window`, subclass the window procedure (`SetWindowSubclass`). On WM_KEYDOWN 'V'+Ctrl, set an atomic flag and request a repaint; pass the message on.
  - New `native-window/src/pastechord.rs`. `app/keyboard.rs clipboard_chord` emits the chord once per frame from `Event::Paste` or the flag.
  - Update `check-clipboard-chords.sh`. The `composing()` guard is kept.
  - Check: a real Ctrl+V via SendInput with an image-only clipboard gives the P1 trace; with a focused TextEdit there is no trace.
  - Falsify: `PDFCER_DIAG_NO_PASTE_HOOK` disables the hook and the check fails.
- P3 Text paste with no caret.
  - Edit: `CommitAddText` at the pointer with a wrap box of `min(240pt, right margin)`.
  - Review: FreeText (G-6 if missing), else a refusal naming Edit mode.
  - Move the `edits.rs` normalisation to base as a pure fn. Use CF_UNICODETEXT only.
  - Trace `clip-pasted source=os kind=text`.
  - Check: "Hello\r\nWorld" gives a two-line block at the pointer. Falsify: Read mode is refused with no trace.
- P4 Image drop.
  - Land at the drop point at natural size with no dialog (Alt+drop opens the dialog). Multiple files cascade.
  - Refusal sentence for GIF/WebP.
  - Extend `checks/dropped_file.rs` with a scripted `at`. Falsify: change `at`.
- P5 `.txt` drop → `place_text` pages after the current page (pages gate). Falsify: an empty `.txt` is refused.
- P6 PDF drop on the canvas: `dialogs/drop_pdf.rs` offering Open / Insert after this page / Place page 1 as artwork here. The default stays Open.
- P7 Review image paste as a stamp. Interim: a one-page PDF passed to `customstamp::place`. Final: G-3. Add a "Paste clipboard image as stamp" Stamp-menu tool via `canvas/placing.rs`.
- P8 SVG drop + paste. Blocked on G-1 (or Ken approves GUI usvg). Remove the "does not read .svg" clause.
- P9 EMF paste. Blocked on G-2; Office also puts PNG on the clipboard, which P1 covers.
- P10 `file.new_from_clipboard`, `pages.insert_from_clipboard`.
- P11 OLE drag from another window: a new `native-drop` crate with an IDropTarget, and winit `with_drag_and_drop(false)`. Deferred until Ken signs off.

Big files (`frame.rs`, `apply.rs`, `appaction.rs`) get one-line hooks only. Strings go in the catalog, and trace names are registered.
