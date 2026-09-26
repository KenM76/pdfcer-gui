# `dialogs::export_text` — the words on the page, in a file anything can
read


> *"also the engine can export PDFs as text. we should have export/import
> for that."*


It read:

> *"There is no import, and this window says nothing about one. The ask
> names two halves and only one of them exists. `pdfcer-core` offers no
> route from a text file back into a PDF — not a document builder, not a
> 'replace this page's text' verb, and not a way to hand `add_ocr_layer` a
> file instead of a recogniser's positioned words. … **No control was drawn
> that declines when pressed**, and nothing in this window implies a round
> trip. R9: a placeholder is worse than an absence, because an absence is
> honest and a placeholder is a promise."*

Every word of that was true for two days, and **the absence was filed rather
than shrugged at**. `pdfcer-core` `Pass 252.0` shipped `place_text` and the
`blank_document` primitive underneath it on 2026-09-06 — *"nothing in the
crate could create a page before, only copy one"*, which is why the gap was
two days wide and not one afternoon.

⇒ [`crate::dialogs::import_text`] is the window R9 forbade drawing until
there was something behind it. **This window still says nothing about the
round trip**, and that is now a wording decision rather than an honesty one:
a window's job is its own act, and the pair is expressed by the two commands
sitting next to each other on File ▸ Export — which is how
`export_form_data` and `import_form_data` already say it.

## The default writes the CLIPBOARD's own bytes


Every control that departs from it — the page-marker separator, Windows line
endings, the byte-order mark — is **opt-in**, and every one of them is named
in the receipt afterwards. Two answers to *"what is the text of this
document"* in one program is worse than either, because both look like text.

## What this window has to say before the press, and what it cannot

The image export's rule: *"everything that could make this export empty has
already been said in the window."* That rule **cannot be met here**, and
saying so is the design.

The thing that makes a text export empty is that the page is a *picture* of
its words — a scan, or a plot from a system that outlined its text. Nothing
about that is knowable from a page count, a page size or anything else this
window holds, and computing it would mean extracting the document to draw a
window that offers to extract the document.

⇒ So this window states the **standing** losses (layout, derived breaks,
style) where an operator can weigh them before pressing, and the *counted*
ones — the empty pages, the unreadable fonts, the scan refusal — arrive
afterwards, off-canvas, from `app::actions::export::text`. That is the same
two-part shape [`crate::text::export_text`]'s header sets out, and it is why
the losses are said twice in two different registers rather than once.

## The page scope is `imageexport`'s, called rather than copied

[`crate::app::actions::imageexport::PageScope`] and `resolve_pages`, which in
turn call `crate::dialogs::print::tabs::parse_page_range` — the print
dialog's parser, which the OCR window and the Insert-pages window already
share. **Five surfaces, one answer** to *"is `1,1` two exports of page one?"*
and *"does `5-3` mean anything?"*, and an operator who learned the syntax on
Print is entitled to it here.

⇒ The module it lives in is now misnamed — `imageexport` owns the page-scope
type for a text export — and that is **stated rather than fixed**. Moving it
is a rename across a file another track is editing, made in the same pass as
a new feature, which is exactly the diff `RIBBON_IA.md` records as
unreviewable. The trigger is armed: the next surface to need a page scope is
the one that moves it to a module named for what it is.

## Item notes

### `fn habits`

The membership rule and the argument for every inclusion and every
omission live on [`crate::app::prefs::ExportTextPrefs`], which is the
type this returns; this function is only the projection. It is one
struct literal with **no `..Default::default()`**, so a field added to
`ExportTextPrefs` is a compile error here rather than a preference
written to disk as its own default and never actually remembered.

### `fn pages`

Called twice per frame — once to grey the Export button, once on the
press — and it is cheap: a parse of a short string. Computing it live is
what keeps the button and the sentence beside the box from ever
disagreeing.

### `fn separator_group`

Two radios rather than a checkbox, because the operator is choosing
between **two things that go in the file**, one of which is the standard
character and one of which is prose pdfcer wrote. A checkbox labelled
"add page markers" would make the second look like a formatting
preference rather than like added content, which is precisely the
distinction rule 4 exists to keep visible.

### `fn losses_group`

Drawn as ordinary labels rather than `weak`, and above the buttons
rather than below them. This is the paragraph that decides whether the
operator should be doing this at all — a drafter who needs the title
block's *layout* wants the DXF export or the image export, and this is
where they find that out.

### `fn region_for_scope`

These exist for `OPERATOR_REQUESTS.md` **O196**. Until the export windows
remembered anything, a driven check had nothing to assert about a radio
beyond *"it is drawn"*; now the question is which one is **selected on
open**, and that cannot be asked of a group rectangle.

The alternative — pressing the group's rectangle plus an offset — is the
check that presses the wrong control the day a hint gains a line, which
`dialogs::export_image::region_for_format` states in full.

### `fn open`

> *"the export windows forget everything. every time I export a dxf I
> have to set it up again."*

All four of this window's answers were literals until that day. The
membership rule — **a setting is remembered only if it would still be
the right answer for a different document** — and the argument for the
shipped value of each one live on
[`crate::app::prefs::ExportTextPrefs`]. Read that first; this is only
the seeding.

⚠ `range_text` is NOT seeded and is not a preference. A typed range is
a statement about *this document's* page numbering, and restoring
`12-40` onto a nine-page file would open the window in a state whose
Export button is already dead for a reason the operator did not cause.

### `fn show`

Takes `&mut Prefs` for O196 alone: the Export press writes this window's
habits to the preferences file before the action is pushed. See
[`crate::dialogs::export_remembered`] for why it happens at the press
and not at the close.

### `fn open_for`

`doc.pages` non-empty is the same guard the command's own predicate applies,
asserted twice for the reason the DXF window asserts it twice: the predicate
greys the control and this refuses the open, and a keymap or a restored
layout can reach a command without going through the ribbon at all.
