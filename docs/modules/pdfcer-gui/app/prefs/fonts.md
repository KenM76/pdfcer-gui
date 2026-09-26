# `app::prefs::fonts` — where pdfcer looks for a font it has to embed

One preference — an ordered list of folders — and it is the input
`tools.embed_fonts` cannot run without.

## ★★★ Why embedding needs this, and why it is a PREFERENCE

`pdfcer_core::font_embed_missing::EmbedRequest::supplied` is a `/BaseFont` →
donor-file map **the shell resolves**; a selected font with no entry in it
comes back as `EmbedBlocker::NoSourceFont`. The engine never searches a disk
for a donor. So an Embed command has nothing to send until an operator has
said where their fonts live, and this list is that answer.

⇒ The general shape, because it recurs: a verb can exist, be registered and
be drawn, and still be unreachable because its *request struct* requires an
input nothing produces. "Does the verb exist" is a different question from
"can this shell fill in what the verb asks for".

## ★★ It lives in `userdata/preferences.txt`, not in `settings.txt`

`crate::app::prefs`' header states the rule and it decides this cleanly:
`pdfcer_core::settings` is for entries that **cite a clause the standard
leaves silent** — an ambiguity pdfcer has to resolve one way or another.
*Where this operator keeps their font files* cites nothing. It is a fact
about a machine, and filing it there would make the settings window's own
opening paragraph dishonest.

## ★ Why a repeated key rather than one joined line

`font_folder = C:\…` may appear as many times as the operator likes, and
every occurrence is another folder in search order. The alternative — a
separator-joined value — needs a separator that cannot occur in a path, and
on Windows the obvious candidates are all legal in one. A repeated key has
no such question, reads correctly in a file an operator edits by hand, and
makes "search order" visible as line order.

★ **Order is preserved and duplicates are dropped.** Order matters because
two folders may hold the same face and the first one wins; duplicates are
dropped because a folder listed twice is a folder searched twice for the
same answer, and because an operator who adds the same folder from the
picker twice has not asked for anything.
