# `pdfcer-gui/app/dispatch/trust`

**Security ▸ Trust's three commands.** All three are preferences, so none needs
a document open and none is greyed.

# Contract

- `claims(id)` is true for `file.trust_web_sites`, `file.trust_ask_files` and
  `file.trust_play_3d`; `dispatch_command` routes those here.
- The two switches flip `TrustPrefs::ask_before_files` / `TrustPrefs::play_3d`,
  trace `trust-switch id=<command> on=true|false` and save the preferences at
  once. `app::conditions::armed` draws each pressed while it is on.
- `file.trust_web_sites` opens `dialogs::trust_sites` over the current
  `TrustPrefs`; that window's OK is what writes.
- Driven by `ui-verify` check `the_trust_switches_and_web_sites_are_kept`.
