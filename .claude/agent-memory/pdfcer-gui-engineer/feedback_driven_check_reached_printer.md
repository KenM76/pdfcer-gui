---
name: feedback-driven-check-reached-printer
description: A ui-verify check pressing Enter in the print dialog spooled 42 real sheets to Ken's ET-16600; pause the queue around any print-dialog drive
metadata:
  type: feedback
---

On 2026-09-23 the poster check typed a scale and pressed Enter. The dialog
host's Enter guard read `text_edit_focused()` after the field had already
surrendered focus, so Print fired and 42 sheets went to the real printer.
The job was cancelled at about 18 printed pages.

**Why:** the isolated ui-verify profile uses the machine's default printer.
Nothing in the harness stops a spool, and no earlier print check pressed a
key that could reach Print.

**How to apply:**
- Before driving any check that types or presses Enter in the print dialog,
  pause the printer (`Invoke-CimMethod` `Pause` on `Win32_Printer`), and
  resume it after.
- Every such check asserts that no `print-spool` line follows its keystrokes.
- If a job queued anyway, delete it with `Remove-PrintJob` before resuming.

Related: [[feedback_ui_verify_competes]].
