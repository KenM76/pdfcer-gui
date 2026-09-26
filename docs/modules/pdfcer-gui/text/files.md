# `text::files` — the copy the open/close/recent surface owns

The strings [`crate::app::files`] and [`crate::app::recent`] show:
the file dialog's own title and filter names, and everything the Recent
control draws.

## Why these are not in [`crate::text::commands`]

That catalog holds one thing: the **label and tooltip of a registered
command**, paired, because the tooltip's job is to say what the label
cannot fit. Nothing here is that. A dialog title is a string handed to the
operating system; a menu row is a file name the operator chose long ago
and this catalog only frames; "No recent documents" is a *state*, not a
verb. Putting them in `commands` would mean that file no longer answered
one question.

## The dialog strings cross a shell boundary

[`open_dialog_title`], [`filter_pdf`] and [`filter_all`] are interpolated
into a PowerShell script (see [`crate::app::files`] for why that script
exists and what replaces it). They are quoted there with **single quotes**,
which PowerShell does not interpolate, so the only character that could
break out is a single quote itself.

**No string in this module may contain `'`.**
[`tests::the_dialog_strings_cannot_break_out_of_the_script`] enforces it,
so an apostrophe added to "pdfcer's documents" fails the suite rather than
producing a parse error inside a child process nobody is watching. English
copy here has no need of one, and the day it does — a translation, most
likely — the fix is the `rfd` call the module header already carries,
which has no shell in it at all.
