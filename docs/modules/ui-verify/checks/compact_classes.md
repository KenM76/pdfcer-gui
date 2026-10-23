# `ui-verify/checks/compact_classes`

`a_compacted_copy_treats_each_kind_of_picture_separately` — **with the compact
window's pictures split by kind, a picture follows its own kind's settings and
no other.**

## What it drives

Opens `fixtures/noisy-photo.pdf` (one 400×400 greyscale noise picture drawn at
300 ppi) with `PDFCER_DIAG_INVOKE=mode.edit,file.save_compacted`, through the
scripted pointer, so it runs under `--no-input`. After the first
`compact-recompressed` line it opens the Pictures fold, then presses in turn
the switch (`compact.pictures.on`), the split (`compact.pictures.split`), the
colour kind's switch (`compact.pictures.colour.on`) and the grey kind's
(`compact.pictures.grey.on`), waiting for a fresh `compact-recompressed` line
after each.

## What it asserts

| after | `resized` | what a wrong count means |
|---|---|---|
| pictures on | 1 | the single set of choices never reached the plan |
| split on | 1 | the split dropped the picture settings, or grey does not open on 150 ppi |
| colour off | 1 | colour's rule reached a grey picture: the class selector is wrong |
| grey off | 0 | grey's rule did not govern the grey picture, or the single set still applies under the split |

The last two together are the discriminating pair: a build that ignored the
split resizes the picture at every step, and one that applied the first
kind's settings to every picture resizes nothing at step 3.

What the rules carry (palette pictures with colour, JPEG kept off palettes,
black and white at 300 ppi stored the smallest exact way) is asserted by
`dialogs::compact_classes`'s unit tests and
`recompress::tests::a_split_picture_follows_its_own_kind`.
