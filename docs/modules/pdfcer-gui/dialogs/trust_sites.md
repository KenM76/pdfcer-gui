# `pdfcer-gui/dialogs/trust_sites`

**Security ▸ Trust ▸ Web sites….** Acrobat's Trust Manager ▸ Internet Access as
a window: the web-link mode (ask, allow all, block all) and every remembered
site, each Allow or Block, each with Forget.

# Contract

- Opened by `app::dispatch::trust` over a copy of `TrustPrefs`; edits touch
  only the copy.
- OK raises `OutwardAction::SetWebTrust { mode, sites }`, which replaces the
  mode and the whole list (`TrustPrefs::set_sites`) and saves. It traces
  `trust-sites-set mode=ask|allow|block sites=<count>`.
- Cancel, Escape and the close button change nothing.
- While the mode is not Ask, the list stays editable and the window says its
  answers are kept but unused: they come back into force when Ask is chosen.
- Preferences, not a document: the window outlives a document closing.
- Regions: `trust_sites.body`, `trust_sites.forget.<row>`, and the dialog
  host's `dialog.buttons.accept` / `dialog.buttons.cancel`.
- Driven by `ui-verify` check `the_trust_switches_and_web_sites_are_kept`.
