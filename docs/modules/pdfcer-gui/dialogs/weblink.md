# `pdfcer-gui/dialogs/weblink`

**The Security Warning a web link raises.** The address in full, a
"Remember this action for this site for all PDF documents" box, Allow and
Block.

# Contract

- Enter or Allow: `OutwardAction::WebAnswer { allow: true }`.
- Escape, Block or the window's close button: `allow: false`. Dismissing the
  prompt never opens anything.
- The checkbox's state is carried as `remember`; remembering and opening
  happen in the apply phase, not here.
- An address that is not http, https, mailto or ftp gets a line naming its
  scheme under the question.
- Regions: `weblink.body`, `weblink.remember`, and the dialog host's
  `dialog.buttons.accept` / `dialog.buttons.cancel`. Trace:
  `weblink-answer allow=… remember=…`.
- Driven by `ui-verify` check `a_web_link_asks_before_it_opens`.
