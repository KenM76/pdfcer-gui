# `text::buttonaction` — every word the *What this button does* chooser says

One module for one control, because the control is where this project's
rule-4 obligation is heaviest: two of the seven choices write an address
into the document that some other program may act on, and **the operator
cannot see that by looking at the page**. Everything they can learn about it
has to be said here.

## ★★★ The disclosure rule these strings implement


⇒ *"Design it from `SubmitDisclosure`, not from Acrobat"* — a straight copy
of Acrobat's dialog would be a **regression** against what pdfcer can now
say. So these strings state the whole address, and the six facts about the
payload that ISO 32000-1 §12.7.5.2 makes true and nobody can guess.

## ★★ Where the disclosure is NOT

**Not on the canvas.** Rule 4's clause that is most often got backwards:
applied content renders exactly as saved content will. A button carrying a
submit gets no badge, no tint and no dashed outline on the page — it is
drawn as the file will draw it. The disclosure lives in this dialog, and
afterwards in the status line, which is off-canvas by construction.

## ★ Not a warning, and not a refusal

None of these say *"are you sure?"*. No scheme, host or port is refused
anywhere — destination policy is open by operator ruling — and `https`
appears **zero times** in ISO 32000-1, so blocking `http://` would be pdfcer
inventing a conformance requirement. [`submit_unencrypted`] therefore
**states** it and lets the operator decide. Nothing here may be phrased as
*"the standard requires"*, because none of it is.
