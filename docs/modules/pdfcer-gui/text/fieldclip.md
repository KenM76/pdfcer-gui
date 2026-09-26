# `text::fieldclip` - the sentences the FORM-FIELD clipboard can say



`Pass 167.0` shipped `pdfcer_core::formclip` and every one of those
properties now travels. The loss note is **deleted**, not softened, on the
engine's own instruction: *"you should not be maintaining a hand-written map
of which properties survive, because it rots silently every time we add an
authoring key."*

What replaced it is `FieldPasteOutcome::disclosures` - a `Vec<String>` the
**engine** writes, covering a dropped value, a carried calculation and its
`/CO` registration, a renamed font resource, an ignored rectangle size, the
tab-order position, a dropped structure-tree link and a reused accessibility
name. It reaches the status row through `vector_edit` like every other verb's
disclosures, and **not one word of it is written here**.

⇒ The rule that decided it is this shell's own and it has now removed two
sentences from this file in one day: **one fact, one wording.** The engine's
version is authoritative - it reports what the operation *did*, not what the
shell *intended* - and a second phrasing is a divergence waiting to happen.

## What remains

[`refusal`] - why nothing happened. Same posture as `text::clipboard`: a
keystroke that does nothing and says nothing is indistinguishable from a
broken keyboard.

[`os_marker`] - the sentence a copy leaves on the *operating system's*
clipboard, which is not a courtesy but a **requirement**; see its own header.

[`candidate_name`] - the spelling of a pasted field's name.

## Rule 4, in one line, because it still governs

A pasted field renders exactly as a saved-and-reopened one would - no badge,
no tint, nothing drawn on the page. The disclosure lives off-canvas, on the
status row. *Render normally; report separately.* **Both.**
