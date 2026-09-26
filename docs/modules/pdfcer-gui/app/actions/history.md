# `app::actions::history` — stepping the command log, in both directions

`Direction`, its four per-direction answers, and [`history_step`] — a
separate file from [`super::apply`] under **R2**'s 1,500-line ceiling.

## Why this is the seam

`tools/gates/check-file-size.sh` refuses a split made to fit a number:
*"Split the module along its seams — one subject per file."*

[`super::apply`] answers *what does this verb do to the document*. This
answers a different question — *what does moving along the command log do* —
and it is different in a way that shows in the code: every other arm in
`apply` **describes an edit** and hands it to `vector_edit`, while undo and
redo describe **no edit at all**. They ask the session to replay one it has
already recorded, and everything interesting about them is the four things
that differ per direction and the one thing that does not.

## ★ The four per-direction answers, and why they are methods on the enum

`peek`, `step`, `event`, `applied` and `declined` are written as methods on
[`Direction`] rather than as `if undo { … } else { … }` inside
[`history_step`], and that is the same rule
`canvas::textedit::disposition::Reason::disposition` follows: **the direction
is the whole of the input to each answer**, so a third direction — there
will never be one, but the shape is the argument — is a compile error at
five sites rather than a silent fall-through at one.

It also keeps `history_step` readable as the sequence it is: peek, step,
trace, disclose or decline.

## ★★ Why an undo is an EDIT

It bumps `edit_epoch`, drops the page texture and invalidates the strip,
exactly as a forward edit does — because it changes what the document says,
and every cache in this shell is keyed on that. A build that treated an undo
as *"putting things back"* would leave the operator looking at a raster of a
page that no longer exists, which is the same class of defect as an edit
that forgot to invalidate.

`tests::an_undo_is_an_edit_and_moves_the_epoch_like_one` in [`super::apply`]
is the assertion, and it lives there deliberately: it is about what an
`Action` does, which is that module's subject.
