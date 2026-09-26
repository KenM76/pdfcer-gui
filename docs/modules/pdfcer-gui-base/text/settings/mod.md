# `text::settings` — every word the Settings window shows

The catalog area for [`crate::dialogs::settings`]. Ported from the old
shell's `ui_text.rs`, where these strings occupied roughly 700 lines in the
middle of a 7,912-line file.

## The one rule this module has that the rest of the catalog does not

Carried across verbatim from the source, because it is the reason the copy
is written the way it is:

> Every string here must be readable by someone who has never opened the
> PDF standard. These settings exist BECAUSE the standard is silent, so the
> operator is being asked to make a judgement — and a judgement cannot be
> made from a clause number. The clause is named for traceability; the
> SENTENCE has to stand on its own.

So `§8.6.4.4` appears in exactly one place per setting, inside a sentence
that would still make sense with the number deleted. An operator who has
never heard of ISO 32000-1 must be able to choose correctly.

## The three obligations, and how they are enforced by shape

`settings_panel.rs`'s header names three things a settings screen must
show that a conventional one omits. Two of them are enforced here by
**function naming**, not by review: every setting has a `*_title`, a
`*_silence` and a `*_radius`, and
[`crate::dialogs::settings::widgets::header`] takes all three as required
arguments. A setting cannot be added without answering all three.

| obligation | where it lives |
|---|---|
| 1. What the default rests on | inside the chosen option's `_note`, and only where it is true |
| 2. That a choice was made at all | `*_silence` — what the standard leaves open |
| 3. Which way costs what | `*_radius` — preview, extraction, or **saved bytes** |

### Obligation 1 is the one the source got wrong, and it is fixed here

The ambiguity register grades each recommended default: **(a)** observed
Acrobat behaviour, **(b)** corpus census, **(c)** other implementations,
**(d)** reasoned inference — *a guess*. Most are (d), and the source's own
header says a guess must say it is a guess.

It said so for five settings and not for five others that `pdfcer-core`
grades (d) just as explicitly: `image_minify`, `unmappable_code`,
`actual_text`, `missing_as` and `trailing_eol` all read as confident
recommendations. **Their notes now carry the disclosure**, in the same
words the settings that had it already use, so the contract the window
states about itself is true of all thirteen rather than of eight.

The one *positively* sourced default — CMYK JPEG polarity, tier (c) —
says so too, because "pdfcer matched every other engine" and "pdfcer
guessed" are different claims and must not read alike.

## Two disclosures the source documented in the engine and showed nowhere

Both are added here, and both are facts rather than directions:

- [`unmappable_omit_note`] now says that a run whose codes are **all**
  unmappable disappears entirely under *Leave it out* — not merely that
  characters go missing. The layout pass drops a run with no characters,
  so a page of `Identity-H` text with no `/ToUnicode` yields *zero runs*.
  That is the surprising half and the source's note omitted it.
- [`actual_text_bound`] is new. No length correspondence exists between
  `/ActualText` and the content it replaces, so character-level mapping
  back to glyph positions is **impossible across such a run whichever
  option is chosen** — which bounds search highlighting, selection and
  redaction-by-text to sequence granularity. `pdfcer-core` calls this *"a
  fact to disclose, not a direction to pick"* and the old window disclosed
  it nowhere.

Both settings' radius lines also now name **redaction**, because R35 is
explicit that a redaction built under one value is not equivalent under
another, and "affects copied and extracted text" does not tell an operator
that.

## Item notes

### `fn triples`

The list held exactly the thirteen `pdfcer_core::settings` entries and
had never grown: the *Drawing the page* group's two preferences were
added on 2026-08-17 and neither reached it, so the window's own stated
contract — *"a setting cannot be added without answering all three,
because the code does not compile otherwise"* — was being checked over a
subset of the window while reading as though it covered all of it.

The `header` helper's required arguments did their job: both settings
**do** answer all three. What was missing was any check that they were
non-empty, and nothing would have caught a `""` passed to satisfy the
signature. That is the whole failure mode the test exists for, and it
had quietly stopped applying to the newest group — which is this
project's most common defect shape wearing a fifth set of clothes.

### `fn every_setting_states_its_silence_and_its_radius`

The mechanical half of the window's stated contract. A setting added
with a title and no silence line would compile — the helper takes
`&str` — and would ship a control that says what it is and never says
why the operator is being asked.

### `const GROUP_SOURCES`

So this counts from the **other** end: it parses the dialog's own source
and counts the [`crate::dialogs::settings::widgets::header`] calls the
application actually makes.

# Why `syn` and not a grep

The same reason `shell::commands::reach` uses it: a substring search
would count the word in a doc comment, in a string, or in
`widgets.rs`'s own `header(ui, title, silence, radius)` sketch — which
is inside a fenced block and is not code. The syntax tree contains no
comments, so a header discussed is not a header called.

# Why the file list is written out

`include_str!` needs a literal path, and that is the useful half rather
than the awkward half: a **moved or deleted module is a compile error**,
so "scanned nothing" cannot pass as "found nothing" — the trap
`reach.rs` names in its own header. A *new* group module is the one case
this cannot see by construction, and it fails in the right direction:
its settings will be in neither list, the counts still agree, and the
catalog test then fails on the missing triples. The cost is that the
author has to add one line here; the alternative is a directory walk at
test time, which is the runtime file read `reach.rs` refused.
The group modules the window is built from, paired with their source.

### `struct Counter`

Matching on the **last segment** rather than the full path, because
the call is written `widgets::header` today and `super::widgets::header`
or a plain `header` after an import would be the same call. Nothing
else in these modules is named `header`, so the loose match costs
nothing and survives an import style change.

### `fn every_settings_module_is_counted`

# The gap this closes, which had already been found once and left open

The list above is hand-written, and the comment beside it says so:
*"a new settings module is invisible to the very test whose job is to
prove the window and the catalog agree."* That was written on
2026-08-28 after `comments.rs` was missed for ten minutes, and it ends
*"if a third module is ever added, this line is the one to remember"*.


⇒ So this test reads `dialogs/settings/mod.rs` and requires every module
it declares to appear above. Two modules are excluded **by name and with
a reason**, which is the part that keeps the exclusion honest:

| module | why it is not a group |
|---|---|
| `widgets` | the helpers the groups are built from; it draws no setting of its own |
| `preset` | the preset row at the top of the window. It DOES call `header`, and that is precisely why it must be excluded rather than forgotten: a preset is not a setting, and counting its header would inflate the total by one forever |

### `fn exactly_the_byte_changing_settings_say_they_change_the_file`

The distinction the window exists to make legible: a setting whose blast
radius is the file on disk is a different kind of decision from one that
only changes the preview, and an operator must be able to tell them
apart from the words. Four of thirteen touch bytes.

Asserted in both directions. A preview setting whose radius grew the
word "file" would be quietly claiming a consequence it does not have —
which trains the operator to stop reading these lines, and the four that
matter are the ones that then get skipped.

### `fn every_guessed_default_says_it_is_a_guess`

Obligation 1, mechanised — and the test that would have failed on the
old shell for five of these. `pdfcer-core` grades `image_minify`,
`unmappable_code`, `actual_text`, `missing_as` and `trailing_eol` tier
(d), reasoned inference, exactly as explicitly as it grades the two that
already said so, and all five read as confident recommendations.

The predicate is deliberately loose about wording and strict about
presence: what matters is that the sentence disclaims external
authority, not that it uses one phrasing.

### `fn the_sourced_default_claims_its_evidence`

The counterpart to the test above, and the reason that one is not
enough. If every note hedged, the operator would have no way to tell
which of thirteen defaults rests on evidence. CMYK JPEG polarity is
tier (c) — every reference engine agrees — and it must not read like a
guess.

### `fn the_superseded_formula_and_its_divergence_note_are_gone`

This replaces `the_acrobat_divergence_names_the_option_that_matches`,
which asserted that a note naming the divergence from Acrobat also named
the option that restores parity. That note is **gone** — the default now
matches Acrobat (`OPERATOR_REQUESTS.md` O52), so a sentence saying pdfcer
deliberately differs is backwards rather than redundant.

The replacement asserts the **absence**, which is the harder half. A
deleted string leaves no test behind it, so nothing would notice a
future edit reinstating either one — and *"the old pdfcer formula"* is
exactly the kind of option somebody restores while looking for
something else.

### `fn an_unknown_theme_is_named_and_kept`

Both halves matter. Quoting is what makes the cause legible; the promise
is what stops the operator "fixing" it by picking one of the three,
which would discard a newer version's setting.

### `fn every_store_location_is_described`

The `None` case is the one worth pinning: an operator whose settings
cannot be written anywhere must be told before they spend a minute
choosing, not after they press Save.

### `fn the_two_engine_facts_the_old_window_hid_are_disclosed`

Both were documented in `pdfcer-core` and shown nowhere in the old
window. A test rather than a comment, because "we should surface that"
is the kind of intention that survives one session.

### `fn intro`

Load-bearing rather than decorative: it is the sentence that tells an
operator why this window is full of questions instead of being full of
answers. Without it, thirteen radio groups read as thirteen things pdfcer
could not decide.

### `fn store_location`

# Why this line is always shown

An operator who does not know which of the two homes is live cannot follow
the update instructions, and those instructions are the one place a wrong
guess costs them their configuration: *"replace the program files, keep
your `userdata` folder"* means nothing if the settings are not in it.

[`StoreKind`] is `#[non_exhaustive]`, so the catch-all arm is required by
the compiler — and it still says something useful rather than falling
silent, because a variant this build does not know about is still a home
the operator's settings are in.

### `fn save_disabled_tooltip`

Greyed rather than absent, which is the one place this window departs from
the no-placeholders rule and is entitled to: Save is *temporarily*
unavailable — one radio click makes it live — and greying with a reason on
hover is exactly what that rule reserves greying for.

### `fn cancel_tooltip`

Not a courtesy. Four of the thirteen settings change **saved bytes**, so an
operator who has been clicking radio buttons for a minute needs to know
that none of it has taken effect — and needs to know it *before* they
decide whether to click Cancel, which is why it is a tooltip on an
always-enabled control rather than a confirmation after the fact.

### `fn restore_defaults_tooltip`

It replaces the **draft** and does not save. Said out loud because the
button's name suggests otherwise: "restore defaults" in most programs is
immediate and irreversible, and this one is neither.

### `fn save_failed`

Loud, and deliberately not softened: the operator asked for something to be
remembered and it was not. The session still honours the choice — see the
dispatch arm — so the sentence has to carry the distinction between "this
did not happen" and "this will not survive a restart".

### `fn group_comments`

*"Comments"*, not *"Annotations"* or *"Markup"*. Every reviewer UI the
operator has used calls them comments; *annotation* is the PDF's word for
the object and *markup* is ours for the tool. The heading is where somebody
looks, so it takes their word.

### `fn group_forms`

*"Forms"*, not *"Tab order"*. The caption names the subject an
operator is looking for; both controls inside it name the property.
It also leaves room for a second forms setting to join without the
caption turning into a list.

### `fn group_fonts`

*"Fonts"*, not *"Font folders"*. A group caption names the subject and the
control inside it names the property — the same call `text::ribbon`'s
`group_format_font` makes, and it leaves room for a second font setting to
join without the caption becoming a list.

### `fn font_folders_hint`

pdfcer does not search the system font directory and will not: embedding
whatever a machine happens to hold into somebody's document is a licensing
decision, and it is not pdfcer's to make silently. So an empty list is not a
default that works — it is embedding switched off, and the sentence says so
before the operator meets it at the far end of a failed embed.

### `fn font_folders_none`

Its wording changed on 2026-08-28 when the OS-fonts checkbox landed:
"no folders" stopped meaning "nothing to embed from", because the box may be
ticked. An empty-state sentence that contradicts a control four rows below it
is worse than none -- an operator who has ticked the box and reads *"nowhere
to take one from"* has been told their setting does not work.

### `fn font_folders_none_at_all`

Two sentences for two states rather than one that hedges. This is the only
configuration in which embedding genuinely cannot take a font from anywhere,
and it is worth saying plainly at the moment it is true -- not at the far end
of an embed, which is where the operator would otherwise meet it.

### `fn use_os_fonts_label`

`OPERATOR_REQUESTS.md` **O50**: *"just a simple checkbox to include fonts
from the OS installed font folders."* "Installed on this computer" rather
than "system fonts" or "OS fonts", because that is what the thing IS to the
person ticking it -- they installed them, or their IT did, and either way
"OS" is a word about implementation.

### `fn use_os_fonts_hint`

It states the **licensing** consequence, and that is not legal throat-
clearing: it is the reason this is a checkbox and not the default. The
operator is being handed a decision, and a control that hands somebody a
decision without saying what the decision is about is a control that took it
for them.

### `fn use_os_fonts_folders`

The folders are DRAWN, greyed, under the tick. A checkbox whose effect is
invisible is one nobody can verify -- and the per-user folder in particular
is somewhere most operators do not know exists, so listing it is the
difference between a setting they trust and one they re-tick to see if it
took.

### `fn use_os_fonts_none_found`

A real state and not a defensive one: `%WINDIR%` and `%LOCALAPPDATA%` are
read from the environment rather than assumed, and a stripped or unusual
image can leave both unset. Saying so beats a tick with nothing under it,
which reads as the list still loading.

### `fn font_folder_remove`

A word rather than a `×`. This list is at most sixteen rows and every row
is a path an operator typed or picked; a glyph that means *delete* on a row
whose other content is a file path is one mis-click from removing the wrong
one, and the word is two characters wider.

### `fn group_measuring`

**New in this port.** In the old shell `parallel_epsilon_degrees` sat
under *Copying and extracting text* — where it has nothing to do with
either — purely because it happened to be a slider like the word-gap one
beside it. The operator symptom is *"my dimension came out as an angle"*,
and nobody with that symptom looks under a heading about copying.

The group headings are the whole navigation model of this window: an
operator arrives with a symptom and the headings are how a symptom finds
its setting. A setting filed under the wrong one is not untidy, it is
unreachable.

### `fn group_display`

**Named for what it is about, not for where its values are stored.** These
two settings live in `preferences.txt` rather than `settings.txt`, which is
an implementation fact the operator has no business meeting: they opened one
window, they press one Save, and one Cancel discards the lot.

*"Drawing"* rather than *"Rendering"* or *"Performance"*. "Rendering" is a
word from our side of the fence; "Performance" promises a tuning panel and
there are two controls. What both settings actually change is how the page
gets drawn, which is what the heading says.
