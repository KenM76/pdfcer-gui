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

## ★★★ THE RULE THAT GOVERNS THIS WHOLE MODULE

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
