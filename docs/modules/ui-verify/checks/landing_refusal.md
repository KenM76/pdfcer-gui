# `ui-verify/checks/landing_refusal`

`the_caret_says_a_line_cannot_be_edited_in_place`: a caret landing on a line
the engine will not edit in place says so before a key is typed, and the
commit's refusal is said by its cause.

`canvas::textedit::landing::refused` asks `EditSession::edit_capability_for`
for the run's show operator and `EditTarget` when the caret lands. `EditError::Unsupported(cause)`
with a workaround opens the draft with
`DraftNote::CannotEditInPlace(EditCause)` on the status bar; without one the
caret is refused as `Refusal::CannotEdit(EditCause)`. Every answer is traced
as `text-edit-capability ... target=page|form ... cause=<EditCause::name>
offered=0|1`. At the
commit, `app::status::decline::textedit::cannot_edit` turns the same
`Unsupported(cause)` into `EditRefusal::CannotEdit`.

# What it drives

`fixtures/quote-operator.pdf` (ignores `--pdf`): `Plain line` drawn with
`Tj` at baseline 700 over `Quoted line` drawn with `'` at baseline 680,
12 pt. Paragraphs are switched off so each click opens its own line.

1. Caret into the plain line at (100, 704). Owed: `text-edit-capability`
   with `cause=none`, and no `status-group:draft-note` region. Escape.
2. Caret into the quoted line at (100, 684), no key typed. Owed:
   `cause=quote-operator offered=1`, `text-edit-note
   note=cannot-edit-in-place`, and the note region drawn.
3. End, type `s`, Escape. Owed: `edit-text-classified said=CannotEdit`.
4. A second session on `fixtures/form-text-runs.pdf`: caret into `Two runs`,
   drawn from a form XObject, at (70, 104). Owed: `asked=1 target=form
   cause=none`.

The plain line is the control: a landing that raised the note for every run
would pass step 2 alone.

# Falsified

- `landing::capability` answering `Ok(None)` without asking → FAIL: the
  quoted line traces no `cause=quote-operator`, and no note is raised.
- `decline::textedit::cannot_edit` answering `None` → FAIL:
  `said=Unstated`.
- The form arm traced as `target=page` → FAIL at step 4.

# What it does not prove

The caret refused outright (`Refusal::CannotEdit`) needs a cause with no
workaround on a page's own contents, and no fixture draws one; its sentences
are proved by `text::editcause`'s unit tests.

# Driven off-screen

Scripted pointer and keys, window off the desktop; it runs under
`--no-input`.
