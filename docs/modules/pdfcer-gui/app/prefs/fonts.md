# `app::prefs::fonts` — where pdfcer looks for a font it has to embed

One preference — an ordered list of folders — and it is the input
`tools.embed_fonts` cannot run without.

## Why embedding needs this, and why it is a PREFERENCE

`pdfcer_core::font_embed_missing::EmbedRequest::supplied` is a `/BaseFont` →
donor-file map **the shell resolves**; a selected font with no entry in it
comes back as `EmbedBlocker::NoSourceFont`. The engine never searches a disk
for a donor. So an Embed command has nothing to send until an operator has
said where their fonts live, and this list is that answer.

⇒ The general shape, because it recurs: a verb can exist, be registered and
be drawn, and still be unreachable because its *request struct* requires an
input nothing produces. "Does the verb exist" is a different question from
"can this shell fill in what the verb asks for".

## It lives in `userdata/preferences.txt`, not in `settings.txt`

`crate::app::prefs`' header states the rule and it decides this cleanly:
`pdfcer_core::settings` is for entries that **cite a clause the standard
leaves silent** — an ambiguity pdfcer has to resolve one way or another.
*Where this operator keeps their font files* cites nothing. It is a fact
about a machine, and filing it there would make the settings window's own
opening paragraph dishonest.

## Why a repeated key rather than one joined line

`font_folder = C:\…` may appear as many times as the operator likes, and
every occurrence is another folder in search order. The alternative — a
separator-joined value — needs a separator that cannot occur in a path, and
on Windows the obvious candidates are all legal in one. A repeated key has
no such question, reads correctly in a file an operator edits by hand, and
makes "search order" visible as line order.

**Order is preserved and duplicates are dropped.** Order matters because
two folders may hold the same face and the first one wins; duplicates are
dropped because a folder listed twice is a folder searched twice for the
same answer, and because an operator who adds the same folder from the
picker twice has not asked for anything.

## Item notes

### `fn a_duplicate_is_refused_and_says_so`

Asserted through the return value as well as the length, because the
caller uses it to decide what to say: *"added"* and *"you already have
that one"* are different sentences and a length comparison cannot tell
them apart when the list is also at its cap.

### `fn the_key_is_documented_even_when_unset`

The failure this guards is the tempting simplification — emit nothing
when the list is empty — which produces a preferences file with no
mention of the one key an operator would want to add by hand.

### `fn the_operators_own_folders_are_searched_first`

Order is search order and the first match wins, so a face the operator
put in a folder they curated for a job beats the same-named face the
machine happens to have. The reverse would make their list decorative on
every name Windows also carries — which is most of them.

### `fn unticked_adds_nothing`

The assertion that guards the licensing argument. A build whose
checkbox did nothing would be caught by the UI; a build that searched
the OS folders regardless of it would not be caught anywhere else.

### `fn ticking_the_box_does_not_duplicate_a_hand_added_folder`

An operator who typed the machine's font folder into the list and then
ticked the box has said one thing twice, and a list holding it twice
would search it twice for the same answer — and would spend one of the
sixteen slots saying nothing.

### `fn a_real_machine_reports_a_real_font_directory`

Every test above would pass on a build whose `os_font_dirs` returned an
empty vector — they assert about ordering and absence. This one asserts
the function finds something, which is the only claim that fails if the
environment lookup is wrong.

SKIPPED where there is no such directory, because that is a fact about
the machine and not about this code.
