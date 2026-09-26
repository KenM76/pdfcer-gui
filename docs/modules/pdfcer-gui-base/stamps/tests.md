# `pdfcer-gui-base/stamps/tests`

## Item notes

### `fn existing`

Deliberately built from `(page, display)` rather than from a
`StampCollection`: `super::existing_names` is the one place the engine's
`#[non_exhaustive]` type is converted, and the plan's contract is about
this narrower shape. `stamp_collection_reaches_the_engine` in
`tools/ui-verify` is what exercises the real reader, on a real file.

### `fn the_driven_checks_fixtures_are_what_the_check_believes_they_are`

⚠ A tripwire for the harness's INPUT, not for this module's logic — the
same shape, and for the same reason, as
`app::status::anomalies`'s `the_control_fixtures_a_driven_run_uses_are_genuinely_clean`.
`ui-verify`'s `a_stamp_collection_discloses_itself` launches the release
binary on this file and asserts the Document-properties section is there,
then launches again on `four-pages.pdf` and asserts it is not. If this
fixture were quietly regenerated without its name tree, the first launch
would go red and the report would blame the panel for a defect that is
entirely in the file it was pointed at — the harness-input failure this
project has already paid for twice, once at the cost of four filed defects
against code that was correct.

And the reverse tripwire in the same test: `four-pages.pdf` is asserted
NOT to be a collection. An absence assertion whose control had grown a name
tree would be a check that cannot fail.
