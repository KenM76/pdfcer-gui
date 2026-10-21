# `pdfcer-gui-base/prefs/trust`

**What a document may reach outside itself: web links, other files, 3D
content.** Security ▸ Trust's settings and the answers remembered from the
Security Warning. `preferences.txt` lines:

```
web_links = ask
web_site = block example.com
web_site = allow intranet.local
open_other_files = ask
play_3d = true
trusted_3d_document = E:/drawings/a.pdf
```

# Contract

- `web_links` is `ask` (default), `allow` or `block`. `web_decision(host)`
  answers Open for `allow`, Refuse for `block`, and under `ask` the remembered
  answer for that host, else Ask. Nothing is refused without the operator
  having refused it.
- `web_site` repeats: one remembered answer per host, lower-cased; a second
  answer for the same host replaces the first. At most `MAX_REMEMBERED` (256).
- `open_other_files` is `ask` (default) or `allow`. `play_3d` defaults to
  true; `trusted_3d_document` repeats, and a listed document plays 3D even
  when `play_3d` is false (`plays_3d`).
- A malformed value is `BadValue`; an unknown key is `NotMine`.

# Where it is set

The Security Warning's "Remember this action for this site" box
(`dialogs::weblink`, applied in `app::actions::outward`). Design: `DESIGNS.md`
§ O320.
