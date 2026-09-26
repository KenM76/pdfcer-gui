# `dialogs::print::autopaper` — pick the sheet from the pages

Operator request **O167**, 2026-09-10: *"we also need the option to auto
select paper size based on the page sizes in the pdf."*

## What this module is, and what it deliberately is not

It is **arithmetic over two lists**: every sheet the driver enumerated
([`super::spooler::PaperForm`]) and every page of the job measured at its
rotated extent. It answers one question — *which enumerated sheet should
this job be asked for?* — and returns that answer together with everything
the disclosure line needs in order to say what happened and why.

It is **not** a policy about mixed page sizes, not a call into the spooler,
and not a `Ui`. It opens no device context, reads no `DEVMODE` and cannot
print. That is on purpose: this is the half of the feature a unit test can
drive, and this project's standing lesson is that a verb's unit test cannot
see the chain in front of it — so the arithmetic is asserted here and the
chain from the combo entry to the spooled `dmPaperSize` is asserted by
driving the binary (`tools/ui-verify/src/checks/print_paper.rs`).

## The rule, stated once

> **The chosen sheet is the smallest enumerated sheet that contains every
> page in the job, with each page free to lie either way round on it. If no
> enumerated sheet contains them all, the largest enumerated sheet is
> chosen and the shortfall is reported.**

Three parts of that sentence are load-bearing and each is argued below.

### "every page", not "the first page"

A `DEVMODE` names **one** sheet and a job has many pages. Choosing from
page 1 would put a 40-page set of A3 details onto A4 because the cover
sheet happened to be A4, and nothing on screen would say why every drawing
came out at 71 %. Choosing the sheet that holds the *largest* page means
the biggest drawing is right and the smaller ones are scaled down onto a
bigger sheet — visibly generous rather than invisibly cropped.

Windows' own answer for a genuinely mixed set is the **choose tray by sheet
size** flag, which this dialog already exposes beside the paper control
(`spooler::DeviceSettings::pick_tray_by_page_size`). A device with more
than one roll or tray will then feed each page its own sheet, and the
`dmPaperSize` this module picks is what the rest lands on. The disclosure
says so when the job is mixed, because an operator with a mixed set needs
to know that one flag is the difference.

### "either way round"

`dmPaperSize` names a **physical piece of paper**; which way the image is
laid on it is `dmOrientation`, a separate field with its own control in
this dialog. A3 and "A3 landscape" are not two sheets. So the fit test
tries the page both ways round, and it is not a compromise — a fit test
that respected page orientation would refuse an A3 sheet for a landscape
A3 drawing, which is the commonest CAD export there is.

### "the largest, and reported"

When nothing fits — an A0 site plan on an office printer whose largest
sheet is A3 — there is no honest choice that makes the drawing come out
right. The alternatives were to fall back to saying nothing about paper
(which prints on whatever the device is standing on, chosen by nobody), or
to pick the biggest sheet the device has and **say that the page is bigger
than it**. The second is chosen: the operator asked pdfcer to match the
pages, and the closest available match plus a sentence naming the shortfall
is a better answer than a silent no-op. See
[`crate::text::print::paper_auto_too_big`].

## Tolerance, and why it is 2 pt

Producers do not emit exact ISO sizes. A4 is 595.276 pt and is written
`595.32`, `595.3`, `595` and `595.2756` by four different exporters; a
SolidWorks sheet set carries the drawing frame's size rather than the
standard's. A fit test with no tolerance would refuse an A4 sheet for a
595.4 pt page and step up to A3, which is a whole size wrong for two
hundredths of a millimetre.

2 pt is 0.7 mm. It is comfortably larger than every rounding divergence
measured in the producer-quirk notes, and comfortably smaller than the gap
between any two ISO or ANSI sizes (the closest pair in ordinary use is
Letter at 612 pt wide and A4 at 595.3 — 16.7 pt apart, eight times the
tolerance). It cannot promote a page to the next size up and it cannot let
a genuinely oversized page pass.

## What "smallest" means when two sheets are the same size

Drivers routinely enumerate the same physical sheet twice under different
names — `"A4"` and `"A4 210 x 297 mm"`, or a borderless twin. The tie is
broken by **the driver's own enumeration order**, first wins, which makes
the choice deterministic for a given device and matches what the operator
sees at the top of the list. Nothing better is available: pdfcer has no way
to know which of two identically-sized forms the device would rather have.
