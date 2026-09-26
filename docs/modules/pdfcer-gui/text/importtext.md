# `text::importtext` — what the import tells the operator afterwards

The receipt for `file.import_text`, and its refusals.
`dialogs::import_text` holds the words the *window* says before the press;
this module holds the words that arrive after it.

## ★★★ THE ENGINE COMPOSES ITS OWN DISCLOSURES AND THIS SHELL DOES NOT PRINT
THEM

`PlaceTextReport::disclosures` is a `Vec<String>` documented as *"every
operator-facing disclosure, verbatim, ready to print"*. It is not printed
here, and the reason is not pride — it is that they are written in the
implementer's voice. Verbatim, the first one reads:

> *"imported text was PAGINATED by pdfcer: 47 line(s) at 13.20pt leading
> (derived default 1.2 x size), 44 line(s) per page"*

and the tab one:

> *"12 tab(s) were collapsed into ordinary word spacing. INDENTATION IS
> LOST: showing has no tab stops, so there is nothing to preserve a tab as"*

Both are **correct and useful to a developer**, and neither is a sentence
this operator would read. *"Showing has no tab stops"* is about the `Tj`
operator; leading in points is a typesetting unit he has no control over in
this window; and the count of lines per page is arithmetic rather than
information.

⇒ So this module writes the same facts from the **structured fields**, in
his terms — which is the split this program makes everywhere: the engine
owns the measurement, the shell owns the sentence. The one place the
engine's own words are printed is the catch-all refusal, where a message we
have not anticipated is better than a shrug.

## ★★ Every sentence here is CONDITIONAL except the first

The page count always shows, because it is the answer to *"did that work?"*.
Everything else appears only when its count is non-zero. A receipt that
listed six disclosures reading *"0 tabs collapsed"* would be a form, and by
the third import nobody would read the line that mattered.
