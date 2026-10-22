# `ui-verify/checks/trust`

`the_trust_switches_and_web_sites_are_kept` — **Security ▸ Trust's switches
flip and are saved at once, and Web sites… forgets a site on OK.**

## What it drives

Seeds the sandbox's preferences with both switches on and one remembered site
(`web_site = allow a.example`), launches off-screen on a copy of the four-page
document, and presses through the scripted pointer, so it runs under
`--no-input`. Each press opens the Security tab first and, when the window is
too narrow, the collapsed Trust group (`ribbon.group.security.trust.collapsed`).

## What it asserts, in order

| # | assertion | what its failure means |
|---|---|---|
| 1 | *Ask before opening other files* traces `trust-switch id=file.trust_ask_files on=false` | the switch does not reach its preference |
| 2 | *Play 3D content* traces `trust-switch id=file.trust_play_3d on=false` | the same, for 3D |
| 3 | Web sites…, Forget on row 0, OK traces `trust-sites-set mode=ask sites=0` | the editor does not carry its edited copy to OK |
| 4 | after exit the preferences say `open_other_files = allow` and `play_3d = false`, and hold no `web_site` line | the change was traced and not written |

The judge of step 4 is a pure function with its own unit test, so a
preferences file the drive left unchanged is shown to fail it.

Falsified with Forget made a no-op: step 3 fails at `sites=1`. A switch that
traces and does not save is not caught on its own, because Web sites…' OK
writes the whole preferences file after it; the step 4 lines then still hold.
