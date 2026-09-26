# `pdfcer-gui/app/prefs/quality`

## Item notes

### `fn every_quality_has_a_distinct_stable_token`

They are what the file holds, so two quality values sharing a token
would make one of them unreachable from a hand-edited file, and a token
that changed with a display name would reset everybody's preference on
upgrade.
