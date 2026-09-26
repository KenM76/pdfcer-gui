# `text::assoc` — every word O173 puts on screen

`OPERATOR_REQUESTS.md` **O173**: *"we should have an easy way to make
pdfce-gui our default opener for pdfs. Ask once with a don't show me again (old-name-exempt: HIS words, quoted verbatim from O173 — correcting an operator's own sentence would stop this being a quotation)
check box option. Then it should be in the top of our settings as a button
to execute the changeover."*

Three surfaces, one conversation, so one module: the **ask-once dialog**
that offers it, the **Settings group** that is where the offer lives
afterwards, and the **state line** that says what Windows actually thinks.
Plus the two registry strings Windows itself displays, which belong here for
the same reason every other displayed string does — they are read by the
operator, in the *Open with* menu and in Windows Settings, and a catalog
that held every string except the two with the widest audience would be a
catalog with a hole in it.

## The sentence this module is written around

> **Windows will ask you to confirm.**

Every word here is shaped by a platform fact: **no program can make itself
the default PDF viewer on its own.** Windows 10 and 11 store the choice
behind a hash only Explorer can compute, and anything written there by an
application is detected and discarded. See [`crate::app::assoc`] for the
mechanism and the citation.

So the button cannot promise what its name suggests, and the wording has to
be honest about a two-step act without turning into an essay about the
registry. The shape chosen: **say what pdfcer does, then say what is left
for the operator, in that order, in two short sentences.** The button label
ends in an ellipsis for the same reason every other label that opens
something does — it is a promise that another surface is coming.

⇒ What is deliberately NOT written here: any string claiming the default was
changed. [`is_default`](crate::app::assoc::is_default) is the only thing
allowed to make that claim, and it makes it by reading what Windows says
rather than by remembering what pdfcer did.

## Vocabulary

- **Windows**, named outright. This is one of the few places where the
  operating system is a participant in the conversation rather than the
  floor it happens on, and *"the system"* would read as pdfcer being coy
  about which program it is talking about.
- **Open PDFs with pdfcer**, not *"file association"*, not *"default
  handler"*, not *"ProgID"*. The operator's question is *"does
  double-clicking a drawing open this?"* and the words should be that
  question's words.
- **Ask again**, not *"show this again"*, in the checkbox — the thing being
  suppressed is a question, not a notification.

## Item notes

### `fn title`

Deliberately identical. The dialog is the offer and the group is where the
offer lives afterwards; an operator who ticks *"don't ask again"* and then
changes their mind is looking for the words they dismissed, and finding a
differently-named group is how a feature is concluded not to exist.

### `fn body`

Two sentences: what pdfcer does, then what is left for the operator. The
second is not a caveat tucked underneath — it is half the act, and an
operator who is not expecting a Windows dialog will read that dialog as
something going wrong.

### `fn later`

*"Not now"* rather than *"Cancel"*: cancelling implies the offer is
withdrawn, and it is not — it is in Settings, permanently, which is the
whole point of the operator's *"then it should be in the top of our
settings"*.

### `fn state_other`

The ProgID is shown raw. It is not a friendly name and there is no
reliable way to turn one into a friendly name — `AppXd4nrz…` is what Windows
stores for Edge — but it is *stable and searchable*, and an operator who
wants to know what has the association can paste it somewhere. A prettier
string that guessed would eventually name the wrong program.

### `fn state_registered_elsewhere`

⚠ The state worth calling out, because it is the one that looks like it
works and does not: a portable build gets unzipped somewhere new, and the
registration still names the old folder. Double-clicking then opens a build
the operator thought they had replaced — or nothing, if the old folder is
gone.

### `fn no_exe_path`

Rare enough to be surprising and real enough to need a sentence:
`current_exe` is documented as able to fail. Saying so plainly beats a
button that does nothing.

### `fn settings_page_refused`

⇒ The instruction is the fallback: this names the route through the Settings
app, because an operator who cannot reach the page by link can still reach
it by hand, and a refusal with no way forward is just an apology.
