# `app::actions::stamps` — the apply half of `Save as stamp collection…`

`OPERATOR_REQUESTS.md` **O169**. The dialog collected the plan; this module
asks where the file goes, calls [`crate::stamps::write::build_and_write`],
and says on the status bar what actually happened.

## Why this is its own module rather than an arm in `super::export`

Every other body in `super::export` is an *export* in the ordinary sense —
the same pages, expressed in another format, for a person or a downstream
tool to read. This one **authors a new kind of document**: the bytes it
writes are a PDF whose structure means something specific to a second
application, and the reason it is hard is not the encoding but the naming.

## What this module is responsible for, and nothing else

1. **The suggestion.** Acrobat's own user-stamps folder, discovered rather
   than hard-coded — see [`crate::stamps::folder`] for why `DC` is a version
   that will one day be something else, and why `Preflight Acrobat
   Continuous` sorts first among the folders beside it.
2. **Creating the folder the operator named.** Non-obvious and load
   bearing: `…\Adobe\Acrobat\DC\Stamps` **does not exist** until the day
   somebody makes their first custom stamp in Acrobat. Refusing to write
   there would make the feature fail for precisely the operator who has
   never made one — which is every operator this feature was built for.
   The directory is created only *after* he has accepted a path, so pdfcer
   never drops a folder into another application's preferences uninvited.
3. **The receipt**, including the sentence that says **restart Acrobat**.
   Acrobat scans its stamps folder at startup and nowhere else; finding that
   out by experiment costs ten minutes of believing the feature is broken.

Everything about *what the file contains* lives in [`crate::stamps`], which
is where it can be unit tested without a picker.
