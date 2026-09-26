# `ui-verify/checks/page_display_pref`

`a_page_display_choice_survives_a_close_and_reaches_a_new_document` — **the
preference he said the program forgets.**

# The report


> *"Also it should remember my page display preferences from my last closing
> of the program. Example if I press show one page at a time and enable flip
> pages."*

It **was** remembered — per document, written the moment the control is
pressed. What there was no answer for is a document the program has never
seen, so a choice made on one drawing meant nothing on the next. From his
chair that is forgetting, and he was right.

Three tiers now resolve it: **this document's own record**, then **his
standing preference**, then the mode's rule.

# ★★★ Why this is a TWO-PROCESS check, and why the close must be graceful

Two separate defects sit behind that sentence and only a second launch can
tell them apart.

**The first** is the missing middle tier — it is about a *different
document*, so a check that never opens one cannot see it.

**The second is sharper.** `LayoutStore::flush` says in its own words that it
exists *"for an exit path, which must not lose the last change to a debounce
that had not yet expired"* — and for a long time it had **no production
caller**. The layout is written **750 ms** after it changes, so anything
changed in the last three quarters of a second before the window closed was
silently thrown away.

⇒ **Dropping a `Session` kills the process, and a killed process runs no exit
hook.** A check that killed the window and then found the preference intact
would only be asserting that the debounce had already expired — true on a
slow run, false on a fast one, and not the property anybody cares about. So
this check closes with **`Alt+F4`**, a real `WM_CLOSE`, and closes
**immediately** after the click so the debounce is still holding.

# ★★ The oracle, and the trace line that could not carry it until today


That is precisely the pair this check has to separate, so the disclosure was
fixed first. Second time in three days that writing a driven check found a
trace which could not tell apart the two states the check existed for — the
OCR tally and the marquee census were the others.

# ★ Why the second document must be one the program has never seen

Because the per-document record would answer for any document that had been
opened before, and it would answer *correctly* — hiding the missing tier
completely. The check therefore opens `fixtures/four-pages.pdf` first and
`fixtures/paragraph.pdf` second, and **normalises the remembered-documents
file** before it starts so a previous run cannot have seeded either.

# What is normalised, and why that is safe

`userdata/` sits **beside the executable**, so the state this check reads and
writes belongs to the binary under test. It deletes `page-display.txt`,
`preferences.txt` and `layout.ron` before the first launch, so every run
starts from the shipped defaults rather than from whatever the last run left.

★★ Safe because the suite is **never** pointed at a published build — that is
the standing rule, and this check is one of the reasons for it. Pointed at
`OneDrive\pdfcer-gui1`, it would delete the operator's own saved preferences.

# Every way this reports SKIP

No binary, `--no-input`, no diagnostic channel, the page-display control not
declared on the ribbon, or the window refusing to close within the grace
period — the last being a property of the machine on the day, reported as a
skip that says so rather than as a pass.
