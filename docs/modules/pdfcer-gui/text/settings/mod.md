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
