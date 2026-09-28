# `ui-verify/checks/signing/timestamp`

`a_timestamped_signature_is_timestamped_or_not_written` — the Sign window
offers a timestamp server, and a signing that asks one either carries its
timestamp or writes nothing.

# What it drives

Per run: open `fixtures/four-pages.pdf`, File ▸ Sign, choose and open the
engine corpus's `signing/rsa2048-modern.pfx`, scroll to `sign-timestamp`,
select-all and type the server with `Driver::type_text` (a URL's `:` and `/`
are punctuation `type_ascii` refuses), press `sign-confirm`.

1. **dead** — `http://127.0.0.1:9/`, which nothing listens on. Must trace
   `sign-requested … tsa=1` and `sign-applied written=0`, emit no
   `sign-prepared`, and leave no output file.
2. **live** — only when `UI_VERIFY_TSA` names a server (network contact is
   opt-in). Must trace `tsa=1`, `written=1` and `sign-prepared … timestamped=1`.

A screenshot after typing is kept as an artefact.

# Why the field is always reached by scrolling

On the harness's window the field sits under the signing time, below the
body's fold, and its declared rect is clipped rather than retired. Clicking
it where declared lands on the separator; `click_scrolled` is the only
reliable route.
