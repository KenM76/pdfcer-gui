# `app::assoc` — making pdfcer the program Windows opens a PDF with

`OPERATOR_REQUESTS.md` **O173**: *"we should have an easy way to make
pdfce-gui our default opener for pdfs. Ask once with a don't show me again (old-name-exempt: HIS words, quoted verbatim from O173 — correcting an operator's own sentence would stop this being a quotation)
check box option. Then it should be in the top of our settings as a button
to execute the changeover."*

## ★★★ What an application can and cannot do here, because it shapes every
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

★★ The surfaces therefore never claim to have changed the default. They say
what they did and what is left, and [`current_owner`] reports what Windows
actually thinks afterwards — a button that claimed success and had not
succeeded would be worse than no button, because the operator would stop
looking.

## ★★ Why `reg.exe` rather than the registry API

This crate carries `#![forbid(unsafe_code)]` and means it. Writing ten keys
through `advapi32` would need a second quarantine crate beside
`native-window`, for a job the operating system ships a supported
command-line tool for. `reg.exe` is in `System32` on every Windows since
2000, it is the tool Microsoft's own documentation uses for exactly these
keys, and it returns a process exit code — which is a better error channel
than an `LSTATUS` this module would have to translate anyway.

★ Every invocation carries `CREATE_NO_WINDOW`, from `std`'s own
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
