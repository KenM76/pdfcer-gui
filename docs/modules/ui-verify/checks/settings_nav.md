# `ui-verify/checks/settings_nav`

`settings_navigate_by_page` — the Settings window shows one page at a time
from a grouped page list, the search keeps only the pages holding a match, and
Tools ▸ Font folders opens on the Fonts page. Operator request O259.

# What it drives

No `--pdf`. Two launches, each off the desktop (`-4200,-4200,1400,900`) with
the scripted pointer, so it runs under `--no-input`. The Settings window is its
own viewport, also placed off the desktop; clicks go to the viewport each
region was declared in, and keys to the one holding `settings.search`.

1. `PDFCER_DIAG_INVOKE=file.settings`. Every page's list entry
   (`settings.heading.<key>`) must be on screen, top to bottom in the contract
   order of `dialogs::settings::nav::PAGES`, and `settings-page key=general`
   must be the page on show.
2. Click `settings.heading.signatures`: the page on show must become
   `signatures`.
3. Click `settings.search`, type `font`: the Fonts entry must stay, at least
   one entry must go (a `ui-rect-gone` after its last rect), and the page on
   show must be one the list still holds.
4. Ctrl+A, Backspace: all seventeen entries must be back.
5. Second launch with `PDFCER_DIAG_INVOKE=tools.font_folders`: the page on
   show must be `fonts`.

The page list is duplicated here on purpose: a page added to the window without
a decision about where it sits in the list should fail this check.

# Falsified

- Typing into the root viewport instead of the dialog's: fails at step 3 (the
  list does not narrow).
- With the list entry's click not selecting its page: fails at step 2,
  "Signatures was clicked … and the window shows general".
