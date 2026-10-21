# `ui-verify/checks/weblink_prompt`

`a_web_link_asks_before_it_opens` — **a web link raises the Security Warning
instead of being refused; Allow opens it, Block with Remember refuses it on
every later click without asking.**

## What it drives

Opens `fixtures/non-navigation-links.pdf` off-screen with
`PDFCER_DIAG_NO_LAUNCH` set, so an opened address is traced as
`external-open kind=web target=…` and no browser starts. The check runs in
its own sandbox profile, so the remembered answer never outlives it.

1. Click the `/URI` link (rect 36,700–200,730); require
   `weblink-decision decision=ask site=example.invalid` and the
   `weblink.body` region drawn. Press Allow; require
   `weblink-answer allow=true remember=false` and
   `external-open … target=https://example.invalid/spec`.
2. Click it again; Allow was not remembered, so it must ask again. Tick
   `weblink.remember`, press Block; require
   `weblink-answer allow=false remember=true` and no `external-open`.
3. Click it a third time; require `decision=refuse` and no prompt.

## What a failure means

| arm | failure |
|---|---|
| first click | the link is refused outright or opens without asking |
| Allow | the address handed to Windows is not the one in the file |
| second click | Allow was remembered though the box was not ticked |
| Block | Block opened the address, or the checkbox was not carried |
| third click | the remembered Block was not honoured, or it still prompts |
