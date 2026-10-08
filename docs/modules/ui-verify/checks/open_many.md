# `several_documents_open_from_one_open`

**Defect it guards.** File ▸ Open takes a multi-selection (Shift, Ctrl,
Ctrl+Shift in the native dialog) and every selected file opens in a tab of
its own. A single-file picker opens only the first of the selection and
drops the rest with nothing said.

**Fixture.** Launched on `fixtures/four-pages.pdf`; the selection is
`esign-three-pages.pdf` and `labelled-pages.pdf`, answered through
`PDFCER_DIAG_OPEN_PATHS` (one Open, every `;`-separated path at once — not
`PDFCER_DIAG_OPEN_PATH`'s one-path-per-call queue).

**Steps.** Invoke `file.open` on opening. An `open-picked` line must follow,
then an `open ok` line naming each selected file, and the tab strip must
publish at least three `doc-tab.` regions.

**Falsified** by making `files::raise_all` raise only the first answer, which
is the single-file picker's behaviour: the check fails naming
`labelled-pages.pdf` as never opened.

**What it does not prove.** That the native dialog allows multi-selection;
that is `rfd::FileDialog::pick_files`, which no off-screen drive can open.
