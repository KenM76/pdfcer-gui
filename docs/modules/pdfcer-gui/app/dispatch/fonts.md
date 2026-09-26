# `pdfcer-gui/app/dispatch/fonts`

## Item notes

### `fn handles`

`pub(crate)` for [`super::routes::handles`]' reason: `shell::commands::reach`'s
reachability checker must be able to evaluate every guard arm it finds, and
a guard it cannot evaluate is a place commands could hide from the check
that exists to find them.

### `fn folders`

The operator's list first, then anything the harness named. Order is search
order and the first match wins — see [`crate::app::prefs::fonts`] — so the
operator's own folders take precedence over a harness's, which is the only
ordering that keeps a driven run honest about what a real one would do.

### `fn dispatch`

**`tools.embed_fonts` depends on the font-folder preference, not on the
engine verb.** The verb exists; what it will not do is find a donor. Read
[`folders`] before concluding this command is blocked on `pdfcer-core`.

⇒ The general rule, because it costs a re-derivation every time it is
forgotten: **ask what a verb's own request struct requires, not whether the
verb exists.** A command can be unreachable because an operand has no
source in this shell, and nothing about the verb's signature says so.

## It can decline with a sentence, and the sentence is recorded

A document whose fonts are all embedded is the **normal** case, not an
error, and opening a window to say so would be a modal an operator has to
dismiss to learn they did not need it. So the construction declines, and the
decline goes to `record_note` — the same channel a refused clipboard cut
uses, for the same reason: the operator still believes the gesture worked,
and silence is what would leave them believing it.
