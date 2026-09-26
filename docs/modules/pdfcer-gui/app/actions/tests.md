# `pdfcer-gui/app/actions/tests`

## Item notes

### `fn the_history_commands_raise_actions`

The dispatch link. Written through `PdfcerApp::dispatch_token` with the
token the **ribbon** would raise, so a build that renamed the id or
reassigned the token fails here rather than shipping a control whose press
is traced and discarded. `crate::app::files`'
`the_save_copy_command_raises_the_save_action` is the same assertion for
the save commands.

# What it deliberately does not assert

That the actions *do* anything. Two arms that pushed the wrong variant
would pass a test written as "some action was raised", which is why the
comparison is against the exact vector — and what each variant does when
applied is `crate::app::actions::apply`'s
`an_undo_is_an_edit_and_moves_the_epoch_like_one`, on a real fixture with
a real edit on the log.

# Why an EMPTY log is the state under test here

Because the dispatcher must not consult one. `undo.available` greys the
control and the apply arm declines an empty stack in words — both of
which are somebody else's job — and an arm that checked the session here
would be the second place that question is asked. So the action is raised
with nothing to undo, exactly as it would be for a `Ctrl+Z` fired at a
freshly opened document, and the decline happens downstream.

### `fn the_push_button_arms_its_tool_like_every_other_kind`

`edit.form_push_button` is live: `pdfcer-core`'s
`EditSession::set_button_action` gives a placed button an `/A`, so the
button pdfcer draws runs something. `app::conditions` sets
`forms.push_button_runnable` and the command is `enabled_when` it — one
line, and therefore one careless revert away from re-greying the control.
Without this test that revert is invisible: the ribbon item goes grey and
every other test still passes.

# Why it also asserts that nothing was declined

Because **greying is a hint and a sentence is the answer**, and the two are
not interchangeable. `egui` refuses a click on a disabled widget and that is
the whole of what greying does — a chord, the QAT, a context menu or the
`PDFCER_DIAG_INVOKE` seam all reach `dispatch_command` without passing the
ribbon at all. So a command that is unavailable must say so in words at the
point it is refused, and a blanket refusal at the top of `dispatch_command`
is not that repair: it stops the arming and removes the words in one move.
`the_history_commands_raise_actions`' header states the same rule from the
other side — the dispatcher must not consult the undo log, because the apply
arm is what declines an empty stack in words.

The guard that forces a future author to rebuild a worded decline for a kind
that becomes inert is `canvas::formfield::tests::no_kind_is_authorable_but_inert`,
whose failure message names both halves of the repair.

### `fn the_four_useful_form_commands_still_arm`

The positive control for the test above. Without it, a mistake that declined
every form command would leave that test passing and the whole feature dead
— the standing rule that a check which cannot fail is not evidence, applied
to its own neighbour.

The `is_useful_once_placed` filter is the enumeration's own answer to *"can
pdfcer do anything with this once it is on the page?"*. It currently admits
every kind, so the `continue` is dormant; it is kept so that a kind added
while it is still inert does not turn this control red for the wrong reason.
The name says four and the body says all of them — read the body.
