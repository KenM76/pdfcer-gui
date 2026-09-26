# `text::stamps` — the words for custom stamp collections

Two surfaces, one vocabulary: the **Save as stamp collection** window that
authors one, and the **Document Properties** section that discloses one
when the open file already is one. They share this catalog because they
share the concept, and a feature that calls the same thing a *category*
in one place and a *set* in the other has already lost the operator.


This header used to open with a section called *"the hardest sentence in
here, and what it must not say"*, and its subject was this:

> pdfcer can **make** a stamp collection and cannot **use** one — placing a
> custom stamp needs an engine verb that does not exist (`ENGINE_BACKLOG.md`,
> `Pass 288.0`). So the copy has to explain a feature whose payoff happens in
> *another application*, without either apologising or pretending.

That was true when written and is false now. Engine `Pass 293.0` shipped
`EditSession::place_page_artwork`, the operator's own stamps are read by
[`crate::stamps::library`], and the Markup ▸ Stamp gallery offers them
beside the standard ones. The payoff happens **here**.

⚠ **This is the eighth time a sentence in this project asserting an engine
limitation has gone stale within hours of being written.** The rule that
keeps coming back: *a sentence about what the engine cannot do is a dated
citation, not a property of the world*. It gets a date and a Pass number so
the next reader can check it, and it gets deleted the moment the Pass lands
— a stale limitation sentence in a doc comment is worse than no sentence,
because it stops the next session from looking.

What survives from that section, and still binds: **R9 — an unavailable
capability renders nothing.** There is no greyed control in any of these
surfaces, and no string here explains an absent one.

## ★ What the placement copy must say, and must not

Four disclosures come back from one placement, and they are the rule-4
half of this feature: the artwork was stretched, the stamp is a *dynamic*
one whose words are frozen at design time, the source page carried form
widgets that did not travel, the source page carried annotations that did
not travel. **All four are off-canvas** — the status line — and none of
them marks the placed stamp. R8b rule 4: applied content renders exactly
as saved content will render, so a stretched stamp draws stretched and
says so in words, and never in a badge.

## The vocabulary, fixed here

| word | means | never used for |
|---|---|---|
| **stamp** | one page of artwork with a name | the `/Stamp` annotation the Markup tab places — those are *standard stamps* |
| **collection** | the file: one category, its stamps | "set", "library", "pack" |
| **category** | the collection's name, shown as Acrobat's submenu heading | "title", even though it is stored as `/Info` `/Title` |
| **display name** | what a picker shows for one stamp | "label" |

⚠ The **standard stamps** distinction matters. `crate::text::markup` and
`crate::text::textannot` already own the words for the `/Stamp` annotation
pdfcer *does* place — `Approved`, `Draft`, and the rest of §12.5.6.12's
closed vocabulary. Those are a different feature that happens to share a
noun, and no string in this file may imply that this window produces one.

## Rule 15

No string here says "dimension" in either sense, and none should acquire
one: a stamp collection has nothing to do with **ce dimensions** or with
**pdf dimensions**. The word to reach for when describing a page's extent
here is *size*.
