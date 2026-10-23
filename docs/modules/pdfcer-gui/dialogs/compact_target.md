# `dialogs::compact_target` — the compact window's starting point and size target

Two rows at the top of *Save a compacted copy*'s recompression section.

## Start from

A list of the engine's optimize presets (`pdfcer_core::optimize::Preset::ALL`):
Lossless, Standard, Screen, Print, Archive. The chosen preset is the base;
the window's own switches are laid over it by `dialogs::recompress::settings_for`:

- **Clean-up switches replace the preset's.** Every preset's clean-up is a
  page-invisible tidy, and the window shows those switches, so what they
  show is what runs.
- **Discards are the union.** A preset's discards (thumbnails, private
  application data, print-only pictures) are kept and the window's added.
  Unticking a discard in the window does not remove one the preset carries;
  the preset's hover says which it carries.
- **Pictures, when switched on, replace the preset's picture settings and
  its per-class rules.** The window's picture fold is one rule for every
  picture; mixing it with the preset's colour/grey/mono rules would apply
  two rules to one picture.

Lossless is the default, so a window opened and left alone changes nothing on
any page.

## Fit under

A switch and a size in MB. Switched on, the copy goes through
`EditSession::optimize_to_budget`: the settings as given first, then colour and
grey pictures as JPEG at lower quality, then lower resolution, until the copy
fits; the least change that fits is kept. Black-and-white pictures are never
changed by the search. The result line (`compact.target.said`) says whether it
fits and which step was taken, and one small line per try follows.

- **The size opens at half the plain compacted copy**, to a tenth of a MB, at
  least 0.1 MB, so the first search has something to do.
- **A dragged size is acted on when the drag ends.** Each change restarts a
  search that writes the whole file up to a dozen times.
- MB here is MiB (1,048,576 bytes), the same unit as every other size in the
  window.

## Regions

`compact.preset` (the list), `compact.preset.<as_str>` (each entry),
`compact.target`, `compact.target.size`, `compact.target.said`.

## Trace

`compact-recompressed` carries `target=none|met|missed`, `max=` (bytes, 0
when off) and `step=` (the kept try's ladder step, 0 for the settings as
given).

Driven by `ui-verify` check `a_compacted_copy_can_be_made_to_fit_a_size`.
