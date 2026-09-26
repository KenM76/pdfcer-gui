# `text::files` — the copy the open/close/recent surface owns

The strings [`crate::app::files`] and [`crate::app::recent`] show:
the file dialog's own title and filter names, and everything the Recent
control draws.

## Why these are not in [`crate::text::commands`]

That catalog holds one thing: the **label and tooltip of a registered
command**, paired, because the tooltip's job is to say what the label
cannot fit. Nothing here is that. A dialog title is a string handed to the
operating system; a menu row is a file name the operator chose long ago
and this catalog only frames; "No recent documents" is a *state*, not a
verb. Putting them in `commands` would mean that file no longer answered
one question.

## The dialog strings cross a shell boundary

[`open_dialog_title`], [`filter_pdf`] and [`filter_all`] are interpolated
into a PowerShell script (see [`crate::app::files`] for why that script
exists and what replaces it). They are quoted there with **single quotes**,
which PowerShell does not interpolate, so the only character that could
break out is a single quote itself.

**No string in this module may contain `'`.**
[`tests::the_dialog_strings_cannot_break_out_of_the_script`] enforces it,
so an apostrophe added to "pdfcer's documents" fails the suite rather than
producing a parse error inside a child process nobody is watching. English
copy here has no need of one, and the day it does — a translation, most
likely — the fix is the `rfd` call the module header already carries,
which has no shell in it at all.

## Item notes

### `fn the_dialog_strings_cannot_break_out_of_the_script`

See the module header. The interim picker single-quotes these into a
script; a `'` inside one would end the literal and the child process
would fail to parse a program nobody can see. This is the mechanical
half of that rule, and it fails at `cargo test` rather than at the
operator's next click.

### `fn each_created_document_gets_its_own_name`

The property the ordinal exists for, asserted rather than assumed. An
`untitled` that ignored its argument would satisfy every other test in
this module, and would make the `new-document` trace line unable to
distinguish a second New from a New that did nothing — a class of
failure findable only by reading a driven run's trace.

### `fn open_dialog_title`

Names what is being asked for rather than the verb, because the verb is
already on the dialog's own accept button ("Open") and repeating it says
nothing. "PDF" appears because the filter defaults to PDFs and an operator
looking for a DWG should learn that here rather than from an empty file
list.

### `fn filter_image`

Names the four formats rather than saying "images", because those four are
what `pdfcer-core` actually places and the picker is the last cheap place to
say so. A filter reading *"Images"* that then refuses a GIF has moved the
refusal from a dialog the operator can dismiss to one they have to read —
and pdfcer refuses GIF, WebP, HEIC and BigTIFF **by name**, which is a good
message arriving at a bad moment.

### `fn filter_text`

*"Text files"* rather than *"Text (TXT)"*, which departs from
[`filter_image`] and [`filter_form_data`] and does so deliberately. Those two
name their extensions because the formats are several and the operator is
matching a suffix he can see. Here the filter's job is the opposite: the
picker offers `.txt` **and** an everything filter precisely because a text
export from another system is very often `.log`, `.csv`, `.dat` or has no
extension at all — so a label reading *"Text (TXT)"* would tell him his file
does not qualify, which is exactly wrong.

### `fn filter_form_data`

It names the three extensions rather than calling them "form data",
because the operator arriving at this dialog has a file with one of those
suffixes in front of them and is matching on what they can see. `filter_image`
makes the same call for the same reason.

### `fn filter_all`

Offered because a PDF with the wrong extension is a real thing an operator
hits — a file saved as `.pdf.txt` by a mail client, a drawing exported
without an extension at all — and pdfcer reads a file by its bytes, not by
its name. A picker that could only offer `*.pdf` would make those files
unopenable through the only surface that opens files.

### `fn save_copy_dialog_title`

Names the thing being produced — *a copy* — rather than the verb, on
[`open_dialog_title`]'s reasoning: the verb is already on the dialog's own
accept button ("Save"). The word `copy` is the load-bearing one and it is
the same word the command's label carries, so an operator who pressed
`Save a copy…` sees the phrase they pressed at the top of the window the
OS put in front of them.

Deliberately **not** [`crate::text::ocr::save_dialog_title`]. Both surfaces
ask the same operating-system question through the same
`crate::app::files::pick_save_path`, and they are asking it about different
things — a recognised copy of one page's text, and the document itself with
its edits. A dialog headed "Save recognised copy" over a save-a-copy would
be a true sentence about the wrong operation, which is the shape of error
this catalog exists to make impossible.

### `fn save_as_dialog_title`

*"Save this document as"* rather than *"Save a copy"*, because the two
commands do different things and this heading is the last place the operator
sees before bytes are written. A copy leaves them editing the original; this
**moves the document** — the next `Ctrl+S` goes to the file they are about to
name. A heading that said "copy" over a command that rebinds would be the
program describing the safer of the two acts while performing the other.

The receipt afterwards says which file they are now editing, for the same
reason: the rebinding is invisible until the next save, and by then it is
too late to be surprised by it.

### `fn save_as_receipt`

It says **"you are now editing"**, not "saved". That is the whole
difference between this command and Save a copy, it is the half the operator
asked for by name, and it is not visible anywhere else on screen until the
next save goes somewhere he did not expect.

### `fn saved_in_place`

# Why the file name is in it

Because with several documents open, *"Saved"* alone leaves the operator to
infer which one - and the answer is "the active tab", which is a fact they
have to reconstruct rather than read. The name is the whole of what makes
this a receipt instead of an acknowledgement.

# Why there is a sentence at all

The only other observable effect of a successful save is that the unsaved
marker disappears from the tab, which is a change you have to already be
watching for to notice. A control whose success is invisible is
indistinguishable from a control that does nothing - which is this project's
defining defect class, and which the operator has now reported under four
different names.

Just the file name, not the whole path: the path is on the tab's tooltip and
in the Properties panel, and a status line that reads as a directory listing
is one nobody reads.

### `fn save_copy_suffix`

# Why the suggestion is never the file that was opened

`crate::text::commands::file_save_copy`'s shipped tooltip promises *"The
original is never overwritten unless you pick it"*, and a **default** is
what makes that promise mechanical rather than aspirational: an operator
who accepts the suggestion without reading it must not overwrite the
drawing they were working on. That is exactly the rule
`crate::dialogs::ocr::suggested_path` already enforces with `-recognised`,
and it is asserted the same way — see
`crate::app::save::tests::the_suggested_name_is_never_the_source_file`.

# Why `-copy`, and which reference application decided it

**Inkscape**, and it is the only one of the three that has this surface at
all. Acrobat and SolidWorks offer *Save As*, which changes which file the
open document *is*; Inkscape offers *Save a Copy*, which does not — and
that is the verb pdfcer registered. Standing instruction 4's head-count is
therefore empty and its sharpened form decides: *ask which of them actually
has the surface you are deciding about*. Inkscape suggests `<name>_copy`;
the separator here is a hyphen rather than an underscore only to match
`-recognised`, the one suffix this shell already ships.

### `fn extract_pages_dialog_title`

Names the thing being produced, on [`save_copy_dialog_title`]'s reasoning,
and the word that has to be there is **new**: an extraction does not modify
the document the operator is looking at, and the one place the operating
system lets pdfcer say so before any bytes are written is this heading. The
third caller of `crate::app::files::pick_save_path`, and the third distinct
heading, for the reason that function's own docs give — a harness or an
operator must be able to tell which of the three questions is being asked.

### `fn merge_dialog_title`

Two things it has to say and neither is optional. **"Combine"** rather than
"Merge", which is Acrobat's word for this operation and therefore the one an
operator arrives with; the ribbon control keeps *Merge files* because that
is where `RIBBON_IA.md` put it, and the two agreeing on the verb matters
less than the dialog being recognised for what it is. And **"several"**,
because this is the only picker in pdfcer that takes more than one file and
nothing else on screen says so — an operator who picks one and presses Open
has to be able to tell it was their mistake rather than the program's limit.

### `fn merge_target_dialog_title`

The fourth caller of `crate::app::files::pick_save_path`, and the fourth
distinct heading, for the reason that function's docs give: a harness or an
operator must be able to tell which of the questions is being asked. Says
**new** for `extract_pages_dialog_title`'s reason — a combine does not
modify any of its sources, and this is the one place the operating system
lets pdfcer say so before any bytes exist.

### `fn merge_target_name`

A bare name rather than a suffix, because unlike an extract there is no ONE
source document to derive one from — that is the whole difference between
the two verbs. `Combined.pdf` names the result rather than the verb, which
is `save_copy_suffix`'s rule, and it lands in whatever folder the first
source came from, which is the only folder pdfcer has any evidence about.

### `fn extract_pages_suffix`

`-pages` rather than `-extract`, on the same *name the result, not the verb*
rule [`save_copy_dialog_title`] follows: the file that lands beside the
drawing is some of its pages, and `SHEET-pages.pdf` says that at a glance in
a folder listing.

It carries [`save_copy_suffix`]'s guarantee for [`save_copy_suffix`]'s
reason: the suggestion is **never** the file that was opened, so an operator
who accepts it without reading it cannot overwrite the document they are
extracting from. That is a promise about a default rather than a warning,
and it is asserted in `crate::app::actions::pages`' tests.

### `fn recent_empty`

Two states share this sentence deliberately: nothing has ever been opened,
and everything that was is on a drive that cannot be reached right now.
The distinction is real but it is not one the operator can act on
differently — in both cases the answer is `Open…` — and a menu that
explained its own bookkeeping would be talking about itself.

### `fn recent_entry_label`

The name alone, because a ribbon menu holding ten full paths is a menu as
wide as the window. The path is on hover — see [`recent_entry_tooltip`] —
which is where two files that share a name are told apart.

Falls back to the whole path when there is no file name to take, which
`Path::file_name` reports for a bare root or a path ending in `..`. A row
that rendered as an empty string would be a live control the operator
cannot see.

### `fn untitled`

`Untitled 1.pdf`, `Untitled 2.pdf`, … — the ordinal counting the documents
this *session* has created, which is what
`crate::app::PdfcerApp::created_documents` holds.

# Why it is numbered

Standing instruction 4, and the head-count is two to one. **Inkscape**
names a new document `New document 1` and increments; **SolidWorks** names
one `Part1` / `Draw1` and increments; **Acrobat** calls its blank page
`Untitled` with no ordinal. Two of the three number them.

The tie-break is a reason of this project's own, and it is the stronger
half. **A defect in this area is found by reading the trace of a driven
run**, and `new-document name="Untitled 1.pdf"` twice in a row is a trace
that cannot distinguish "New was pressed twice" from "New was pressed once
and the second press did nothing". The ordinal is what makes the second
press observable.

# Why the extension is on it

Because this string becomes `crate::app::state::OpenDoc::path`, and three
things downstream build a **file name** from it — a save suggestion, the
Pages panel caption, the recent-menu label shape. `Untitled 1` with no
suffix would produce an extensionless save suggestion, which is the one
place the difference is not cosmetic. Acrobat is also the reference of the
three that shows an extension, and it is the one of the three that makes
PDFs.

The word is `Untitled` rather than `Drawing` or `Sheet`: it says the
document has no name yet, which is true, rather than guessing what the
operator is about to make.

### `fn recent_entry_tooltip`

The full path, unedited. Two drawings called `Sheet 1.pdf` in two job
folders are the ordinary case in this trade, and the folder is the only
thing that distinguishes them.
