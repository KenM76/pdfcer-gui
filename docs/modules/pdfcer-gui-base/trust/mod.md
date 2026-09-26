# `trust` — where the anchors come from, and the three facts they let this
shell state about a signature

The shell half of `pdfcer-core`'s `Pass 10.2` / `10.3` / `10.4` / `10.5`.
`ENGINE_BACKLOG.md` carried four rows for this — importing an installed
Acrobat/Reader trust store, evaluating a signature's trust against those
anchors, keeping the choice across sessions, and the deterministic
no-network parts of RFC 5280 path validation — and they are one feature.
This module is its model: locating a store, reading it, and turning
`signature::verify_all_with_trust` into something a panel can render without
ever saying more than the engine said.

## THE RULE THAT GOVERNS THIS WHOLE MODULE

**This is the one place in the product where a wrong answer is worse than no
answer.**

A UI that says *"trusted"* about a chain it did not really validate is
precisely the failure mode `pdfcer-core`'s design exists to prevent. Its
`SignatureVerdict` carries **three facts that never collapse into one
bool** — `integrity`, `coverage` and `trust` — and every surface built on
this module reports them **separately**. There is no composite badge, no
green tick, and no arithmetic that turns three answers into one.

In particular [`pdfcer_core::signature::Trust::NotChecked`] renders **as
itself**: *"not checked"*, never as a soft "no", never as a grey tick, and
never omitted. A shell that hid `NotChecked` would be indistinguishable, on
screen, from a shell that had checked and found nothing wrong — which is the
exact inversion this feature exists to prevent.

## 1. What "importing" means here, and why nothing is copied

Adobe's own downloaded trust list lives in `addressbook.acrodata`, a
`%PPKLITE-` COS file that `pdfcer_core::trust_store` parses with pdfcer's own
COS + X.509 code. No Acrobat automation, no network, read-only.

**pdfcer never copies it.** There is no pdfcer-side anchor file, no snapshot,
no cached DER on disk. Every evaluation reads the operator's own file as it
is at that moment. That is a decision and the argument is
`ENGINE_BACKLOG.md`'s own:

> an anchor set that silently went stale is worse than one that was never
> imported.


## 2. Off by default, and the opt-in is the engine's own setting

[`pdfcer_core::settings::AcrobatTrustStore`] is `Off` by default and this
shell does not second-guess it. `AtOwnRisk` is the operator's explicit,
disclosed opt-in, and the engine's own reasoning is why it is a setting
rather than a default: reading Adobe's downloaded file is a local read, and
*whether relying on it fits the Adobe Reader licence is the operator's call,
not a pdfcer legal determination.*

Because it is the **engine's** setting rather than one of this shell's
preferences, the same choice governs `pdfcer verify-signatures` at the
command line. That is the point: one answer to *"may pdfcer read Acrobat's
trust list?"*, in one file, for both front ends.

## 3. Where the store is, and why the path is a preference of ours

The engine deliberately does not locate the file — its own module header:
*"Locating the file is the shell's job."* [`candidate_paths`] is this
shell's list and mirrors the CLI's exactly, so the two front ends look in
the same places in the same order.

`pdfcer-gui`'s `app::prefs::Prefs::acrobat_trust_store_path` overrides it, and it
exists for the same reason `Prefs::acrobat_path` does
(O122): discovery is a list of conventional locations, and a conventional
location is wrong the first time somebody's profile is redirected, or their
Acrobat is a track this build's list does not name, or their store was
handed to them by an administrator. **R9's escape hatch applies**: the
*inspect* control is absent when there is no store to inspect, but the path
field is visible in Settings whether or not discovery succeeded, because it
is the only thing that can fix the case where discovery failed.

## 4. The four states of "what anchors were used", and why four

[`Anchors`] has four variants because collapsing any two of them makes this
shell say something untrue:

| variant | the operator's real situation | what collapsing it would claim |
|---|---|---|
| [`Anchors::OptedOut`] | pdfcer was told not to look | that it looked and found nothing |
| [`Anchors::NoStore`] | it looked and this machine has no store | that the operator declined |
| [`Anchors::Unreadable`] | it found one and could not read it | that there was none — hiding a fixable fault |
| [`Anchors::Used`] | these anchors, from this file, of this date | — |

Only the last one can produce a `Trusted` or an `Untrusted` verdict. The
other three all produce `NotChecked`, and each of them words *why* rather
than sharing one sentence, because *"you turned it off"* and *"your Acrobat
store is corrupt"* are opposite calls to action.

## 5. What is deliberately NOT here

**Revocation.** `PathChecks::revocation_checked` is `false` on every verdict
this build can produce, because CRL/OCSP need the network `pdfcer-core`
never touches (its decision 135). This shell does not fetch either, and the
copy in `pdfcer-gui`'s `text::trust` says so on every `Trusted` verdict rather than
once in a footnote — a disclosure attached to the claim it qualifies is one
an operator reads.

**A trust decision of our own.** Nothing here interprets the `/Trust`
bitfield, promotes an anchor, or has any notion of "trusted enough". The
anchors go in, the engine's verdict comes out, and this shell's entire
contribution is *which file the anchors came from* and *saying what happened*.

## Item notes

### `const TRACKS`

**The same four, in the same order, as `pdfcer-cli`'s
`acrobat_trust_store_paths`.** Deliberately mirrored rather than reasoned
out again: two front ends of one program that look in different places
produce the single most confusing support conversation available — *"the
command line finds my store and the window does not"* — and neither answer
is wrong on its own terms.

`DC` first because it is the only track Adobe still ships to; the three
older ones are there because an install that stopped being updated still has
a store, and a stale store is a thing this module can disclose rather than a
thing it must refuse.

### `fn resolve_anchors`

Returns the [`Located`] alongside the [`Store`] so the caller does not have
to locate twice; the absence path re-locates because it needs the *failed*
state, which by definition produced no store.

### `fn describe_absence`

Separated from [`resolve_anchors`] because the happy path must not pay for
building an explanation, and because an explanation assembled from the same
inputs twice is one that can be tested on its own.

### `struct CacheKey`

**Every input is in the key, and that is the whole safety property.**

A panel redraws sixty times a second and verification is a SHA-256 over the
whole file plus an RSA or ECDSA verify per signature, on top of a 3 MB COS
parse of the anchor store. Doing that per frame is not an option; caching it
against a key that misses an input is worse than not caching, because the
panel then shows a verdict about a file, a setting or an anchor set that is
no longer the one in force — and it looks exactly like a correct answer.

So the key is the file's identity **and** its length **and** its
modification time (the two cheap facts that change when a file is written
to), plus both halves of the trust configuration. Change any of them and the
verdict is recomputed.

`len` and `modified` together rather than either alone: an incremental
save that appends always changes the length, and a same-length rewrite
always changes the time. Neither is sufficient on its own and both are one
`stat`.

### `fn slot`

# Why `egui::Memory` rather than a field on `PanelsState`

Because the cache is an implementation detail of **one panel** and of the
Settings group beside it, and a field on the shared panel state would make
it part of every panel's contract. `dialogs::settings::widgets::text_value`
already keeps its per-control edit buffer here for the same reason and says
so.

It is `insert_temp`, so it is never serialised into the layout file. A
signature verdict is a measurement of a file at a moment; persisting one
across restarts would produce a verdict about a file that may have been
replaced while pdfcer was not running, which is precisely the failure the
key above exists to prevent — reintroduced by a different door.

### `fn anchor_trace`

Not an operator string and not in the catalog: it is a diagnostic word a
driven check matches on. `ui-verify` asserts on `anchors=off` /
`anchors=none` / `anchors=used:N`, and a check that matched translated prose
would break the day the prose improved.

### `fn candidate_paths`

**Platform-neutral by construction, with no `cfg(windows)`.** `%APPDATA%` is
a Windows variable, so on any other target `env::var` simply returns `Err`
and this returns an empty list — which every caller already handles, because
"no store on this machine" is a real state on Windows too. The CLI takes the
identical approach and states the identical reason.

It reports **candidates**, not findings: nothing here touches the disk.
[`locate`] is what asks whether any of them exists, and keeping the two
apart is what lets [`Located::None`] report *what was looked at*, which is
the only actionable half of "nothing was found".

### `enum Located`

Five states rather than `Option<PathBuf>`, and every extra one exists
because it is a different sentence to an operator. In particular
[`Self::ConfiguredMissing`] must never be rendered as [`Self::None`]: a
person who typed a path and got *"no trust store was found"* is being told
their machine has no store, when what actually happened is that they made a
typo — and the field they would fix is the one they are looking at.

### `fn named`

Distinct from [`Self::usable`] on purpose: a missing configured path is
still the path the operator needs to see printed back, and a control
that showed nothing there would be answering a typo with silence.

### `fn locate`

`configured` is `pdfcer-gui`'s `app::prefs::Prefs::acrobat_trust_store_path` as
typed. It is trimmed here as well as on the way in, because this file is not
the only route a value takes — the same argument `prefs` makes about
`acrobat_path`, and the same reason: a trailing space is a path that does
not exist and the failure presents as *"the setting does nothing"*.

**An empty field means "ask this machine", not "no store".** Clearing a text
box is how a person un-sets it, and reading a cleared box as a positive
choice would suppress the feature with no way back except editing a file by
hand.

### `struct Store`

[`Self::modified`] is carried in the same struct as the anchors rather
than fetched where it is displayed. That is deliberate: the count and the
date are one fact — *"1,780 anchors, as Adobe last downloaded them on this
date"* — and a surface that could obtain one without the other would
eventually show the count alone. The whole argument for reading the store
live rather than snapshotting it is that its age stays visible.

### `fn load`

# Errors

Returns the engine's own error text. It is not re-worded here: the engine
names its refusals precisely (`NotAnAddressBook` explains that
`directories.acrodata` and `security-policy.acrodata` carry no anchors), and
a shell that paraphrased would produce a second, vaguer vocabulary for the
same faults.

### `fn evaluated`

Used only to decide which sentence to draw, **never** to decide what a
verdict means. The verdict is the engine's; this predicate says which
explanation of `NotChecked` belongs beside it.

### `struct Report`

Deliberately a value with no methods that judge. It carries the engine's
verdicts verbatim and the provenance of the anchors, and every reading of it
happens in `pdfcer-gui`'s `text::trust` where the words live.

### `fn examine`

# Why this takes bytes AND a graph

Because `/ByteRange` is a claim about **bytes**, and the object model cannot
check a claim about bytes against itself — the engine's own reason for
`byte_range_coverage` taking a length rather than deriving one. Verification
needs the real file, digested; the graph is only how the signature
dictionaries are found.

⚠ The bytes must be **the file on disk**, not the session's rendering of it.
A signature covers what was written, and an unsaved edit is not in the file.
[`examine_path`] is the route that guarantees this; this function is split
out so the whole decision table is testable without a filesystem.

### `fn examine_path`

# Errors

The `std::io::Error` text, when the file cannot be read. There is no
verdict in that case and none is invented: a document whose file pdfcer
cannot read is not a document whose signatures failed.

### `fn modified_date`

Date only, no clock time. The question an operator is answering is *"is
this anchor set current?"*, which is a question about weeks and months —
AATL refreshes are not a daily event — and a timestamp to the second would
invite the reading that the number is precise about something it is not.

Returns `None` for a time the filesystem could not give, or one before the
Unix epoch, rather than substituting today. A store with no readable date is
a store whose staleness is unknown, and saying so is the whole point.

### `fn cached_report`

Returns `Err` with the reason the file could not be read — which is a
different statement from any verdict and must not be rendered as one.

**It computes on the first frame it is called on, without being asked.**
The alternative considered was a *Check signatures* button. It was refused:
an operator who has opened a panel called Signatures has already asked, and
a button would leave the panel's default state showing coverage numbers with
no integrity beside them — which is the state this whole feature exists to
end. The cost is one verification the first time the panel is drawn for a
given file; the cache above is what stops it being sixty.
