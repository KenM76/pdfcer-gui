# `acrobat` — finding the operator's Acrobat, and handing the file over to
it


> *"also beside our read-review-edit buttons at the top there should be an
> open in acrobat button which will open the active pdf in acrobat reader or
> pro depending on what is installed - we'll have to add a feature to
> automatically locate and open the installed acrobat on the system, and
> have a setting where people can change it. When clicked it will check if
> the file has been changed (forms filled out for example, etc) and ask to
> save changes first, but if it hasn't changed it will note the file will be
> closed when opened in acrobat with and ok button to continue - there will
> be a cancel button as well."*

## 1. What this module is for

pdfcer **gives the file up**. It does not leave a second editor looking at
a document this program is still holding: Acrobat is handed the path and
the document is then closed, so that one program owns the file.

That is the operator's own instruction — point 6 of the request — and the
reason is worth stating rather than merely obeying. **Acrobat takes its own
lock on the file it opens.** Two editors on one PDF is how an afternoon's
work disappears: pdfcer writes its revision, Acrobat writes its own over the
top from a copy it read before pdfcer saved, and neither program ever
reports an error because neither one did anything wrong. The only defence
that actually holds is for exactly one program to have the file at a time,
and the program giving it up is the one the operator just told to hand it
over.

So the confirmation is not ceremony. Closing a document is a thing that
happens to the operator's work, and it is announced before it happens.

## 2. The state of the open document chooses the answer

| State of the open document | What happens | Why |
|---|---|---|
| no Acrobat found, none configured | **the button is not there** | R9: an unavailable capability renders nothing |
| never saved — no file on disk | a refusal in its own words | Acrobat opens *files*; there is no file. This is not "Acrobat is missing" and must not sound like it |
| unsaved edits | **Save and open**, or Cancel | the document is about to be closed, so a third "open anyway" button would be data loss dressed as a choice |
| clean | **OK**, or Cancel | it says the file will be closed, which is the operator's point 6 |

The third row is the one worth arguing. Every close prompt an operator has
ever seen offers *Save · Don't save · Cancel*, and
`pdfcer_gui::dialogs::unsaved` is the application's implementation of that
shape. **This dialog deliberately is not that one.** Its middle button would
read *"open in Acrobat without saving"*, and pressing it would close the
document, discard the edits, and hand Acrobat the **old bytes** — the
operator would be looking at a file that does not contain the form they just
filled in, in a program that is perfectly capable of saving it, and the two
facts together are how a morning of data entry is silently overwritten. The
answer that loses nothing and the answer that loses everything are not two
points on a scale here; only one of them is a coherent request.

## 3. The seam: where the impurity is, and what stays testable

Two things in this module can only be true of a real machine — reading the
Windows registry, and starting a process — and both are behind a trait so
that everything *interesting* is a pure function over values.

| Trait | The impure act | The test double |
|---|---|---|
| [`Registrations`] | `reg query` over `App Paths` and the `.pdf` handler | a table of `&str` readings, plus a set of paths that "exist" |
| [`Launcher`] | `std::process::Command::spawn` | a recorder that keeps the argv it was given |

Everything above those two lines — which candidate wins, whether Pro beats
Reader, whether a configured override beats discovery, whether the button
is drawn at all, which dialog is raised — is decided by
[`resolve`] and [`prompt_for`], which take values and return values. The
test suite never touches a registry and never starts a process, and that is
not a convenience: a test that shelled out to `reg.exe` would pass or fail
according to what Adobe installer last ran on the machine running it, which
makes it a report about the machine rather than about the code.

## 4. Discovery reads Windows' own registration; it never guesses a path

`C:\Program Files\Adobe\…` is wrong the first time somebody installs
anywhere else, and **this operator's own working volume is `D:`**. So the
three sources, in the order [`resolve`] consults them:

1. **The configured override**, from Settings. Beats everything, because a
   person who typed a path has answered the question this module is
   otherwise guessing at.
2. **`App Paths`** — `HKLM`, its `WOW6432Node` mirror, and `HKCU` — for
   `Acrobat.exe` (Pro) and `AcroRd32.exe` (Reader). This is the key Windows
   itself reads when something says "run Acrobat.exe" with no path, so it is
   the registration Adobe's installer is obliged to keep correct.
3. **The registered `.pdf` handler's command**, as a fallback, parsed for
   its executable.

### Why the `.pdf` handler is filtered rather than trusted

Source 3 answers *"what opens PDFs here"*, which is **not** the question.
Verified on this machine, 2026-09-04: `HKLM\SOFTWARE\Classes\.pdf` reads
`OpenPDFStudio.pdf` — a different vendor's product entirely. A fallback that
took whatever the handler named would have put a button labelled *Open in
Acrobat* over a launcher for PDF Studio, which is a lie the operator finds
out about after their document is already closed.

So a handler command is accepted only when the executable it names is
called `Acrobat.exe` or `AcroRd32.exe`. See [`discover::edition_of`].

### Every candidate is checked against the disk before it is offered

A registry key outlives the program it points at: an uninstall that leaves a
stale `App Paths` value is ordinary, and so is a path typed into Settings
with a letter missing. [`Registrations::exists`] is asked about every
candidate before it becomes a [`Viewer`], because the alternative is a
button that is present, enabled, and does nothing when pressed — the exact
shape R9 exists to prevent, arrived at from the other direction.

## 5. Why `std::process::Command` and not `combridge`

`combridge` is this machine's canonical COM-automation bridge and it is the
right tool for *driving* an application that is already running. Nothing
here drives anything: pdfcer starts a program with a file name on its
command line and stops caring. That is a plain process launch, and routing
it through a COM bridge would add a dependency, a running-instance
requirement and an attach step to an operation whose whole content is one
`spawn`.

## 6. Why the registry is read by `reg query` and not by a crate

Three constraints meet here and they only have one intersection:

- This crate's root carries `#![forbid(unsafe_code)]`, as does
  `pdfcer-gui`'s, and `forbid` cannot be relaxed by an inner `allow`.
  Calling `advapi32`'s `RegGetValueW` from here is therefore not available
  at all.
- No crate in this workspace depends on a registry crate, and this work is
  not permitted to edit a `Cargo.toml` to add one. (`winreg` does appear in
  `Cargo.lock`, but only as a build-dependency of `embed-resource`, which
  is not a dependency this crate can call into.)
- `native-window` is the crate that quarantines `unsafe`, but it exists to
  hold the window-manager and clipboard calls a GUI toolkit will not
  express, and its manifest says so. Growing it a registry reader would
  make it "the unsafe crate" rather than a crate named for a capability,
  which is the drift its own documentation was written to prevent.

`reg.exe` ships with Windows, needs no dependency, and its output is a
two-line format that has been stable since NT. It is read at most a handful
of times — once when the shell starts, and again whenever the setting
changes — so the cost of a process spawn is not on any path an operator can
feel. And it is behind [`Registrations`], so the day a registry crate is
permissible the swap is one file.

The one non-obvious part is [`windows::CREATE_NO_WINDOW`]: a GUI process
that spawns a console program on Windows gets a **console window flashed on
screen** unless it says otherwise. Without that flag, discovery would blink
a black box over the operator's document every time the shell started.

## Item notes

### `fn app_path`

Implementations return the value **as the registry holds it**: quoting,
surrounding whitespace and all. Cleaning it up is [`discover`]'s job,
so that the cleaning is tested.

### `fn launch`

# Errors

Whatever the platform reports: the executable has been removed since
discovery, the operator lacks permission, the process table is full.
The caller words it; this trait does not.
