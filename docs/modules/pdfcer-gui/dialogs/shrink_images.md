# `pdfcer-gui/dialogs/shrink_images`

**Format ▸ Make pictures smaller** — the picture options of the compact window,
applied in place to the pictures selected in the open document.

## Routes

One command, `format.shrink_images`, reached three ways, all decided by one
test (`app::dispatch::shrinkimages::shrinkable_images`):

- the Format tab's image group;
- the canvas object menu (right-click a picture, or any object of a selection
  that holds one);
- the Objects panel row menu, which selects the row it was opened on.

A selection is gathered the usual way: Ctrl+click or Shift+click adds or
removes an object, a band selects many. The command is offered when at least
one selected object is a picture drawn through an image XObject; the others in
a mixed selection are ignored. Inline images have no stream of their own to
rewrite and are never counted.

## The window

The resolution, JPEG and colour rows are `dialogs::compact_images::rows`, so a
pairing the engine refuses is greyed here exactly as in the compact window.
Before Apply it says that a picture drawn in several places changes in all of
them: an image XObject is one stored object however often it is drawn. The
window closes itself when another document comes on screen.

## The edit

`app::actions::vector::shrinkimages` builds `OptimizeSettings::new()` (which
leaves every stream) plus one `Rule` selecting the chosen ids with
`StreamAction::Image(patch)`, plans it, and applies it as one undoable
optimize. The status line reports what changed, whether detail was lost, and
each reason a picture was left as stored. When the plan changes nothing the
document is not touched.

## Trace

- `shrink-images-opened page= pictures=` — the window opened on that many
  pictures.
- `shrink-images-requested pictures= ppi=` — Apply pressed.
- `images-shrunk page= asked= changed= resized= jpeg= converted= lossy= skipped=`
  — the edit ran.
- `command-declined id=format.shrink_images reason=no-picture-selected`.

Driven by `ui-verify` check `shrink_pictures_reaches_every_selected_picture`.
