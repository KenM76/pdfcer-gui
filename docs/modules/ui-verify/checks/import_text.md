# `ui-verify/checks/import_text`

`a_text_file_becomes_pages` — **File ▸ Import text as pages, driven: a
`.txt` on disk becomes real pages in the open document.**

The other half of the operator's 2026-09-04 ask — *"we should have
export/import for that"* — which was half a feature for two days because
`pdfcer-core` could not create a page, only copy one. `blank_document` and
`place_text` shipped as `Pass 252.0`; this is what says the shell reached
them.

## ★★★ The five links, and four of them have no test anywhere else

| # | link | its own test |
|---|---|---|
| 1 | the ribbon item exists and dispatches | `reach::every_registered_command_is_routed_or_argued` — that an arm EXISTS, not that it runs |
| 2 | the picker's answer reaches the window | **nothing** |
| 3 | **the window's four choices reach the template** | `dialogs::import_text::tests` — the conversion, given a dialog |
| 4 | **Import raises the action and the apply arm reads the file** | **nothing** |
| 5 | **the engine's report becomes a receipt the operator can read** | `actions::importtext::tests` — the sentences, given a report |

**Link 4 is the one that would ship as silence.** The dialog is its own OS
window; a button whose published rect is right and whose click lands on the
main window instead does nothing, says nothing, and looks exactly like a
feature that was never wired.

## ★★ `frame_of`, never `session.frame()` — this is a DIALOG

A dialog is a separate viewport with its own client rect. This project spent
a driven run discovering that: every in-dialog click landed hundreds of
points away and the symptom was *silence*. `driving::frame_of` resolves the
frame the region was published in, and it is safe on main-window regions
too, so there is no reason to reach for the other one.

## ★ The picker is an OS dialog, so it is bypassed by the env seam

`PDFCER_DIAG_TEXT_IMPORT_PATH` — the same seam `pick_form_data_source`,
`pick_document` and `pick_image_source` all carry, and for the same reason:
a native file dialog is a window this harness cannot type into, so without
it the check could press the ribbon item and get no further.

⚠ **The seam is not a shortcut around the feature.** Everything after the
picker — the window, the four choices, the button, the action, the file
read, the engine call and the receipt — is the real thing.
