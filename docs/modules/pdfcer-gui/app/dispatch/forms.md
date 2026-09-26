# `app::dispatch::forms` — what the form-field commands do

One arm of [`super::PdfcerApp::dispatch_command`], in its own file. The
dispatcher is a routing table and this is a route; the **reasoning** below
is the reason it is not an arm, because it is about form fields rather than
about routing and would be unfindable among the dispatcher's other arms.

## The whole of what a form command does: it arms a tool

Nothing is authored here, and — unusually — nothing is authored on release
either. The placing gesture raises `Action::BeginFormField`, which opens
`crate::dialogs::formfield`, and the field exists once the operator presses
Add. `crate::canvas::formfield`'s header argues why that indirection is the
feature rather than an extra step: a stray form field is invisible on a
printed page and swallows every keystroke aimed near it, so a mis-drag must
cost nothing.

## ★★★ Why the push button is refused here, in words

This file exists to hold one finding, and it is worth the space.

`edit.form_push_button` is `enabled_when("forms.push_button_runnable")`, a
condition nothing sets, so its ribbon item is greyed. That much is measured
rather than assumed — `egui_shell::ribbon::ctx::condition_holds` answers
`false` for an unset name.

**But `egui` refusing a click on a disabled widget is the entire mechanism
of greying.** Every other route into the dispatcher — a keyboard chord, the
QAT, a context menu, the `PDFCER_DIAG_INVOKE` harness seam — never touches
the ribbon at all. Driving the release binary with that id arms the tool and
traces `form-tool-armed kind=PushButton`. **An `enabled_when` is a drawing
instruction, not a rule**, and that holds for every command that carries one.

### ★★ Why there is no blanket guard at the top of `dispatch_command`

The obvious repair — refuse any command whose `enable` predicate is false —
is wrong, and the apply layer's own tests assert against it: *"the
dispatcher must not consult one. `undo.available` greys the control and the
apply arm declines an empty stack **in words** — both of which are somebody
else's job."*

**Greying is a hint; the worded decline is the answer.** A choke point that
swallowed the command would make `Ctrl+Z` on an empty stack do nothing at
all *and* say nothing at all — strictly worse than the status line it
produces, and the exact shape of the silent-control defect this project
keeps finding.

So enforcement lives where the words can live, which is the arm. What that
costs is one branch per command that needs it; what it buys is that an
operator who reaches a greyed capability by some other route is told why
rather than left pressing a key that does nothing.
