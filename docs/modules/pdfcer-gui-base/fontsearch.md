# `fontsearch` — where pdfcer looks for a font it has to embed

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

### `const MAX_FOLDERS`

Sixteen, and the cap exists for the same reason every cap in this project
does — a bound is a decision and an unbounded list is a decision nobody
made. It is not a performance limit: an embed searches folders once per
missing face. It is a **legibility** limit, because a settings pane listing
forty directories has stopped being a setting and become a file manager,
and because a preferences file that has accumulated forty entries is one
nobody has pruned.

### `fn os_font_dirs`

# Why searching them is allowed, when pdfcer must not go looking

The rule the module header states is against pdfcer **deciding silently** —
a program that searched `C:\Windows\Fonts` on its own would be answering a
licensing question on the operator's behalf, in a file that outlives the
decision. An explicit, persistent, **off-by-default** switch — the checkbox
`OPERATOR_REQUESTS.md` **O50** asks for — is not a loophole in that rule: it
is the operator making the decision once, visibly, somewhere they can find
it again.

⇒ The shape recurs: **when a capability is refused on the grounds that the
program must not decide, the answer is usually a visible setting rather than
a permanent no.**

# TWO folders, and the second is the one that matters

| | |
|---|---|
| `%WINDIR%\Fonts` | the machine's fonts, installed for everybody |
| `%LOCALAPPDATA%\Microsoft\Windows\Fonts` | installed for **this user only** |

The per-user location is where a plain double-click on a `.ttf` installs by
default — **without** an administrator prompt, which is exactly why it is the
common case. A checkbox that searched only the machine folder would miss the
font the operator installed themselves for this drawing, which is the font
they are most likely to have ticked the box for.

# Read from the environment rather than hard-coded

`%WINDIR%` is `C:\Windows` on essentially every machine and is not guaranteed
to be; a domain image can put it elsewhere. The cost of asking is one
environment lookup, and the cost of assuming is a checkbox that silently
finds nothing on somebody's machine.

Returns only directories that **exist**, unlike [`add`] -- and the two
differ on purpose. A folder the operator typed may be an unmounted drive and
is kept; these are derived, not typed, so a path that is not there is not a
promise anybody made and listing it would put a dead row under the checkbox.

### `fn search_path`

The operator's own folders **first**, then the OS ones. Order is search
order and the first match wins ([`add`]), so a face the operator put in a
folder of their own beats the same-named face the machine happens to have --
which is the only ordering that makes their list mean anything. A folder
they curated for a job is a decision; `C:\Windows\Fonts` is whatever has
accumulated.

### `fn add`

Returns whether the list changed, so a caller can tell "added" from "you
already have that one" without comparing lengths.

It does **not** check that the folder exists. A removable drive that is
not mounted right now is still where the operator's fonts live, and a
preference that silently dropped it on the day the drive was unplugged
would be worse than one that keeps a path that occasionally resolves to
nothing. The *embed* is where a missing folder is reported, because that is
where it matters.

### `fn parse_one`

Trims, and rejects only the empty result. A path is otherwise taken
verbatim — no canonicalisation, no separator normalisation — because
`Path` comparison on Windows is case-insensitive in the filesystem and
case-sensitive in `PathBuf`, and a preference that rewrote what the
operator typed would make their own file unrecognisable to them.

### `fn write_block`

The comment is emitted **even when the list is empty**, which is the
convention every other block in that file follows and is the reason the
file is editable by hand: an operator who wants to add a folder without
opening pdfcer needs to see the key name and its rules, and a key that only
appears once it is already set cannot teach anybody anything.

### `fn write_os_flag`

Written **always**, both values, for [`write_block`]'s reason: the file is
editable by hand, and a key that only appears once it is already set cannot
teach anybody it exists. This one has a second reason of its own — it is the
switch with a licensing consequence, so the file states that consequence
where somebody editing it will read it.
