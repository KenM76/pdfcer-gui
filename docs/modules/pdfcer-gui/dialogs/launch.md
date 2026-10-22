# `pdfcer-gui/dialogs/launch`

**The Launch File prompt a `/Launch` link raises.** Acrobat's warning naming
the file in full, a "Do not show this message again" box, Open and Do Not
Open.

# Contract

- Enter or Open: `OutwardAction::LaunchAnswer { open: true }`.
- Escape, Do Not Open or the window's close button: `open: false`. Dismissing
  the prompt never opens anything.
- The checkbox is carried as `dont_ask`. It is honoured with Open only: it
  turns Security ▸ Trust ▸ *Ask before opening other files* off. With Do Not
  Open it changes nothing, so no file is ever refused without a prompt.
- A PDF gets a line saying it opens here, in a tab of its own.
- Regions: `launch.body`, `launch.dont_ask`, and the dialog host's
  `dialog.buttons.accept` / `dialog.buttons.cancel`. Trace:
  `launch-answer open=… dont_ask=…`.
- Driven by `ui-verify` check `a_file_link_asks_or_opens_here`.
