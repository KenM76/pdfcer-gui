# `pdfcer-gui-base/outward`

**The one place the program asks Windows to open something a document
names.** `open(kind, target)` runs `rundll32 url.dll,FileProtocolHandler
<target>`, never `cmd`, so no character of the target reaches a shell.

# Contract

- `handable`: a target that is empty, or carries a quote or a control
  character, is refused before anything runs; `open` returns the sentence.
- `site(uri)`: the key a remembered answer is stored under. The host, without
  user, password or port; an address with no `//` authority (`mailto:`) is
  its own site up to any `?` or `#`.
- `is_web`: http, https, mailto and ftp. The prompt says so when an address is
  anything else.
- `PDFCER_DIAG_NO_LAUNCH` set: `open` traces
  `external-open kind=… target=…` and returns `Opened::Suppressed`. The
  harness follows a link without a browser appearing.
- Off Windows `open` returns `text::outward::no_launcher`.
