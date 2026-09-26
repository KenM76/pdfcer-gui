# `pdfcer-gui-base/text/pageclip`

## Item notes

### `fn no_sentence_fakes_its_plural`

A parenthesised plural is the tell of a program that could not be
bothered, and every one of these sentences is read by somebody who did
not expect it — which is the whole reason it exists.

### `fn both_invisible_disclosures_say_what_to_do_about_it`

The two invisible facts are the ones an operator cannot investigate for
themselves — a left-behind field and an orphaned box both look like
nothing at all — so a sentence that stopped at the diagnosis would send
them hunting for a defect.

### `fn fields_dropped`

`PageClip::fields_dropped`. A field whose widgets sit on more than one sheet
cannot travel unless every one of those sheets was picked, so the engine
leaves it out and counts it.

# Why this is said at the COPY and not at the paste

Because it is a fact about **their selection**, and it is still fixable. At
the copy the operator can widen the pick and copy again; by the paste the
clip is made and the sentence would be an autopsy.

⇒ That is the general rule this file follows and it is worth stating once:
**a disclosure belongs at the moment the operator can still act on it.**

# The wording

It says *why* — "on sheets you did not pick" — because the remedy is in the
cause and an operator told only that something was dropped has to guess.

### `fn orphaned_widgets`

`InsertOutcome::orphaned_widgets`, and the engine flagged it by name as
*"the one that produces a document that looks right and is not"*.

# The mechanism, because the sentence has to be trusted

A page's `/Annots` array reaches its form-field boxes, so they travel with
the page. The `/AcroForm` dictionary that **owns** them is a catalog entry
and does not. The boxes therefore arrive drawn, positioned and looking
exactly like working fields, belonging to no field at all — so nothing can
fill them, no form-data export sees them, and no viewer complains.

The engine measured **two** on its own smoke test, which is to say this is
the ordinary case for pasting a page out of a form, not an exotic one.

# Why it offers the Forms panel rather than apologising

Because the remedy exists and is one panel away: the Tab-order section lists
exactly these widgets and offers to register them. A sentence that named the
problem and not the cure would send the operator looking for a bug.

### `fn os_marker`

The clip **is a PDF** — the engine's own choice, because `pageops::assemble`
already does object copying, reference remapping and page-tree construction
on every split and merge, and a private page format would be a second
implementation of the most-exercised code in the crate.

So this sentence is not merely a marker the way the object clipboard's is:
what pdfcer is holding really is a document. The wording says so, because an
operator who learns that can save it, mail it, or open it — and would never
guess it from a program that only offered to paste it back.

### `fn nothing_copied`

It names the **command**, not the chord, because there is no chord: page
copy and paste are named commands on the Pages tab, and `Ctrl+C` belongs to
the canvas. Telling the operator to press a key that does something else is
worse than saying nothing. `app::dispatch::pageclip`'s header carries why.

### `fn copy_refused`

Its sentence, prefixed with what was being attempted rather than replaced.
The engine's refusals name the document's own state — an encryption, a
certification — and are written by the party that knows why; what they cannot
know is which gesture the operator was making when they met one.
