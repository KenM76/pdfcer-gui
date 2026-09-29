# `pdfcer-gui-base/settingspages/redaction`

## Item notes

### `fn residual_reach`

# Why the choice is here and not in the apply dialog

The apply dialog performs the removal the moment it opens — it exists to
show a completed rewrite and ask whether to keep it — so a control inside it
would be answering a question that has already been decided. The value has
to be in force before that window appears, which makes it a preference.
[`crate::app::prefs::redaction`] carries the argument in full.

# What this does not change

It does not change what is **reported**. Every value runs the same search
and names everything it finds; they differ only in what the search is
permitted to edit. A match the narrowest value declines to act on is still
counted and still disclosed off-canvas, which is the only reason offering
the narrowest value is safe at all.
