# `assoc` — making pdfcer the program Windows opens a PDF with

`OPERATOR_REQUESTS.md` **O173**: *"we should have an easy way to make
pdfce-gui our default opener for pdfs. Ask once with a don't show me again (old-name-exempt: HIS words, quoted verbatim from O173 — correcting an operator's own sentence would stop this being a quotation)
check box option. Then it should be in the top of our settings as a button
to execute the changeover."*

## What an application can and cannot do here, because it shapes every
## word on the two surfaces above this module

**No program can silently take the `.pdf` association on Windows 10 or 11.**
It could once, through `IApplicationAssociationRegistration::SetAppAsDefault`,
and Microsoft removed that ability on purpose — an unattended "make me the
default" was the single most abused call on the platform. What replaced it
is a **hash** stored under `…\Explorer\FileExts\.pdf\UserChoice`, computed
from the extension, the ProgID, the user's SID and a salt Microsoft does not
publish, and refreshed by Explorer. A value written there by anything but
the shell is detected and thrown away, silently, and on some builds it also
resets the association to Edge.

⇒ So the honest decomposition is **two halves, and this module owns the
first one entirely**:

1. **Become a candidate.** Register a ProgID, an `OpenWithProgids` entry
   against `.pdf`, an `Applications\…` block and a
   `RegisteredApplications` capability set. After this pdfcer appears in
   *Open with*, in *Choose a default app*, and as an application Windows
   Settings can be deep-linked to. **Today it appears in none of them** —
   this is a portable build that no installer ever registered — which is why
   the changeover cannot currently even be attempted by hand.
2. **Let the operator confirm**, in the OS's own dialog, which is the only
   place the confirmation is legal. [`open_settings_page`] opens that page
   deep-linked to pdfcer, so it is one click rather than four levels of
   navigation.

The surfaces therefore never claim to have changed the default. They say
what they did and what is left, and [`current_owner`] reports what Windows
actually thinks afterwards — a button that claimed success and had not
succeeded would be worse than no button, because the operator would stop
looking.

## Why `reg.exe` rather than the registry API

This crate carries `#![forbid(unsafe_code)]` and means it. Writing ten keys
through `advapi32` would need a second quarantine crate beside
`native-window`, for a job the operating system ships a supported
command-line tool for. `reg.exe` is in `System32` on every Windows since
2000, it is the tool Microsoft's own documentation uses for exactly these
keys, and it returns a process exit code — which is a better error channel
than an `LSTATUS` this module would have to translate anyway.

Every invocation carries `CREATE_NO_WINDOW`, from `std`'s own
`CommandExt` — safe, no dependency — because a console flashing up ten
times is how a considered action looks like a virus.

## Rule 4 — fuzzy, never sneaky

Nothing here touches a document, so the canvas rule is not in play. The
disclosure half is: this module **reports what it did and what Windows now
says**, and the surfaces put that off-canvas in the Settings group and in
the dialog. An operator who presses the button and is not asked by Windows
must be able to find out why, and [`current_owner`] is how.

## Rule 15

No dimension of either kind appears in this module.

## Item notes

### `fn set`

`value` is `None` for a key's default (`/ve`) and `Some` for a named one.

`/f` on every write: these are idempotent declarations, not a
conversation. A prompt from a subprocess with no console is a hang with no
symptom.

### `fn run`

# Errors

`reg.exe`'s own stderr, or the reason it could not be started.

### `fn reg`

`None` rather than an error, because every caller is asking a question
about the machine that has a legitimate *"cannot tell"* answer — a key that
does not exist makes `reg query` exit non-zero, and on a fresh machine that
is the normal case rather than a fault worth a sentence.

### `fn open_url`

# Errors

Whatever the shell said.

### `fn open_with_progids_names_the_progid_rather_than_storing_it`

Writing the ProgID as the *data* of a default value is the plausible
mistake, and it produces a key that exists and an *Open with* menu that
does not list pdfcer.

### `fn current_owner`

`None` when the key is absent, which is the ordinary state on a machine
where nobody has ever chosen — Windows then falls back to the machine-wide
association, and there is no honest single answer to report.

Read from `UserChoice` rather than from `HKCR\.pdf`, because `UserChoice`
is what Explorer actually consults and the two disagree routinely.
Reporting the wrong one would produce a state line saying pdfcer is the
default while double-clicking still opened Edge — the exact confusion this
line exists to end.

### `enum Registration`

Three states rather than a bool, and the third is the load-bearing one. A
portable build gets unzipped somewhere new; the registration then still
names the *old* folder, so Windows opens a build the operator thought they
had replaced — or nothing at all, if the old folder is gone. That failure
looks exactly like success from the inside: the key exists, the ProgID is
pdfcer's, and everything reports fine. Only the **path** tells them apart.

### `struct Status`

Why this is a struct probed on demand rather than three functions the
UI calls: a Settings pane redraws on **every frame**, and every one of these
answers costs a `reg.exe` process. `dialogs::settings::acrobat`'s header
states the same rule for the same reason — *"this module must not resolve,
because resolving spawns processes and a Settings pane redraws on every
frame"*. So the probe happens when the window opens and when the button is
pressed, and never in a paint.

### `fn is_default`

Derived from [`Self::owner`] — a reading of what Windows recorded —
and never from *"we pressed the button"*. Only the operator can make
this true, in a dialog pdfcer does not own, so pdfcer's memory of its
own actions is not evidence.

### `fn register`

Returns `Ok(())` when every key was written, or the first failure as a
sentence — `reg.exe`'s own message, which names the key it could not write
and is more use than any wording this module could invent.

## What is written, and why each one is needed

| Key | Without it |
|---|---|
| `Classes\pdfcer.pdf` and its `shell\open\command` | there is nothing to associate; the ProgID does not exist |
| `Classes\pdfcer.pdf\DefaultIcon` | pdfcer's entry in every chooser is the blank generic icon |
| `Classes\.pdf\OpenWithProgids` | pdfcer is missing from the **Open with** submenu |
| `Classes\Applications\<exe>` + `SupportedTypes` | *Open with ▸ Choose another app* does not offer it either |
| `Software\pdfcer\Capabilities` + `RegisteredApplications` | Windows Settings' *Default apps* list does not contain pdfcer, so the deep link has nothing to land on |

All under `HKCU`. Nothing here needs administrator rights, nothing affects
another user of the machine, and an operator who changes their mind can
delete one key. A portable build that wrote to `HKLM` would be a portable
build that needed elevation and left something behind — the opposite of what
the package promises on its front page.

# Errors

The first `reg.exe` invocation that failed, with its own output.

### `fn open_settings_page`

`registeredAppUser=` and not `registeredAUMID=`. The first names an entry
in `HKCU\Software\RegisteredApplications` — which [`register`] has just
written — and the second names a packaged (Store) app identity that a
portable exe does not have and cannot get. Passing the wrong one lands on
the unfiltered list, which is a page with three hundred entries on it.

⚠ Older Windows 10 builds ignore the query entirely and open the top of the
Default-apps page. That is a degraded outcome rather than a failure, and it
is why the wording on both surfaces says *"Windows will ask you to
confirm"* rather than describing a specific dialog: the dialog differs
between builds, and a sentence describing one of them would be wrong on the
others.

# Errors

The shell's own refusal when the page could not be opened.
