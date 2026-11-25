# `the_prepress_commands_change_the_page_and_say_what_they_did`

**Owns:** File ▸ Print's Fix hairlines, Flatten transparency and Convert
colours (O458): the window's Apply, the engine verb's counts, the window's
report.

**Drive.** `prepress-mix.pdf` (`fixtures/prepress-mix.PROVENANCE.py`): one
Letter page with a 0.05 pt stroke, one `/ca 0.5` fill over an opaque one, one
DeviceRGB fill, and a CMYK output intent. Three launches, each on a fresh
copy, with `mode.edit,<command>`; each presses `prepress.apply.<kind>` twice.
Convert colours runs with its default destination, the document's output
intent.

**Oracle**, per command, on each press:

- `prepress-asked verb=<token> run=N`, then `prepress-applied` and
  `prepress-shown run=N refused=0 lines>0`: the window that asked shows the
  report of its own run.
- First press: `widened=1` / `flattened=1` / `converted=1`, and `changed=1`.
- Second press: the same count `=0` and `changed=0`. The first run removed
  the case, so the engine's own scan of the edited page finds none.

**Not covered.** Convert to PDF/X (it needs an installed output-class profile,
which a build machine may lack); the report's pixels; undo, which the edit
funnel's own checks own.

# `the_preflight_panel_gives_a_standards_verdict`

**Owns:** the Preflight panel's standard verdict and its Convert to PDF/X
(O458).

**Drive.** The same fixture, launched with `mode.edit,view.preflight`. Pick
`pdfx-4` in `preflight.profile` and Run; press `preflight.convert_pdfx`, then
close the window it opens; pick `pdfua-1` and Run.

**Oracle.**

- `pdfx-4`: `preflight-verdict verdict=fails failed>0`. The fixture has an
  RGB paint and no TrimBox, both of which PDF/X-4 forbids.
- `prepress.apply.pdfx` is drawn after `preflight.convert_pdfx`: the button
  reaches File ▸ Print's window through its command.
- `pdfua-1`: `verdict=none` with a numeric `unchecked=`. The engine gives no
  PDF/UA verdict, and the panel must not make one up.

**Not covered.** PDF/A; *conforms*, which needs a conforming fixture; the
verdict's words and pixels.
