# `clipboard` — the bytes a copy-out places, and the order they go in


> *"Also I'd like to be able to copy and paste anything to other software -
> like copy and paste vector graphics into word or inkscape for example if
> possible."*

## Where the two halves live

| half | where | what it decides |
|---|---|---|
| **the bytes and the order** | this file | [`ORDER`], [`svg_payload`]'s trailing NUL, [`dib_v5`]'s premultiplied top-down `BI_BITFIELDS` framing, [`pixels_per_metre`], and [`CopyPayload::degrades_word_to_a_picture`] |
| **the placement** | [`place`], over `crates/native-clipboard` | producing the payload from a real page or selection, refusing a set that would degrade, and the one ordered transaction |

The seam is not tidiness. **What decides whether a paste into Word
arrives as an editable graphic or as a flat picture is entirely in this
file**, and none of it needs a syscall:

* **the order** the formats are placed in ([`ORDER`]) — measured by the
  engine against a real Word paste, not chosen;
* **the trailing NUL** on the SVG ([`svg_payload`]) — Chromium's exact
  byte shape, which is what Microsoft validated Office against;
* **the DIB's premultiplied, top-down, `BI_BITFIELDS` framing**
  ([`dib_v5`]) — a straight-alpha DIB looks wrong in exactly the readers
  that fall back to it.

Every one of those is pure and every one is asserted below, against bytes
rather than against a clipboard.

## The property the whole feature rests on: half is worse than none

A copy-out that places only the raster formats is *worse than no copy-out at
all*, because Word silently degrades such a paste to a flat picture and the
operator has no way to tell that from a feature that works. The engine's
note is unambiguous: *"place only the raster formats and it degrades to a
plain picture."*

⇒ [`CopyPayload::degrades_word_to_a_picture`] is the predicate that answers
it, and [`place`] asks it **before framing a single byte** — see that
module's `staged`. `native-clipboard` completes the property from the other
end by creating every OS handle *before* the clipboard is opened, so a
refusal anywhere leaves the operator's clipboard exactly as it was.

## Why `native-clipboard` is a crate and not a module here

`CF_ENHMETAFILE` is a **GDI handle**, not an `HGLOBAL`, so the metafile must
go on through `SetEnhMetaFileBits` followed by `SetClipboardData` — two
`unsafe` calls with a real ownership contract between them (the handle
becomes the clipboard's on success and must be deleted by the caller on
failure). `crates/pdfcer-gui/src/lib.rs` and `main.rs` both open with
`#![forbid(unsafe_code)]`, and `forbid` cannot be relaxed from the inside —
that is the whole point of choosing it over `deny`.

**This project had already answered the same question once.**
`crates/native-window` exists for four `user32` calls, and its manifest says
why in one sentence: *"Its own crate rather than a module here, because this
crate's `#![forbid(unsafe_code)]` is a claim worth keeping and `forbid`
cannot be relaxed from the inside."* `crates/native-clipboard` is that
answer applied one question further on, built to the same shape: no
dependencies, hand-written `extern` declarations, a `SAFETY` comment per
call, and RAII types rather than careful sequencing.

⇒ Neither `clipboard-win` nor `arboard` was adopted, and the second could
not have been: `arboard` has **no registered-format API at all**, so it
cannot place entries 1 and 3 of [`ORDER`] — the two that make a Word paste
land as an editable graphic. The engine's note says so in one line:
*"`arboard` cannot do 1 or 4 (no registered-format API); use
`clipboard-win` directly as the CLI does."*

## Why this file stays pure

Because the part that is easy to get wrong is the part that needs no
dependency, and keeping the two apart is what makes the hard half testable
at all. Everything above is decided here, in code that crosses no syscall
and can therefore be asserted byte for byte. [`place`] is the plumbing:
produce a payload, ask the predicate, hand the ordered set over.

⚠ **No test in this file — or in [`place`] — touches the real clipboard,
and none ever should.** The clipboard is global state on the operator's
machine: a test that placed bytes would silently destroy whatever he had
copied, from a `cargo test` run he did not connect to his clipboard. Every
assertion here is on the bytes that *would* be placed, and the `unsafe`
placement inside `native-clipboard` is verified **by construction and by
review** — stated plainly rather than dressed up as coverage.

## Sources

* `D:\Dev\FeatureRequests\pdfce_FeatureRequests\open\note_export_to_png_jpeg_svg_and_copy_out_ship_here_is_what_to_wire.md`
  and its addendum — the measured order, the Word paste, the EMF placement.
* `crates/pdfcer-cli/src/clipboard.rs` in the engine tree — the worked
  ~60-line placement [`place`] mirrors, with one deliberate departure
  documented on `native_clipboard::place`: the metafile handle is created
  *before* the clipboard is opened, not inside the guard.
* The engine's `docs/clipboard-interop-survey.md` §7 — application source
  at pinned revisions, which is where the reader preferences come from.

## Item notes

### `fn the_placement_order_is_the_measured_one`

The single most important assertion in this module. The order was
*measured* by the engine against a real Word paste through combridge,
not chosen for tidiness, and a reader takes the first format it
recognises — so reordering these four silently changes what Word,
LibreOffice and Inkscape each receive, with no error anywhere.

Asserted as the whole array in one comparison rather than as four
index checks, so a *swap* (the likeliest edit) fails as loudly as a
replacement.

### `fn every_vector_format_precedes_every_raster_one`

Stated as a property rather than as the literal array above, because
it is the property the whole feature rests on and it should survive a
deliberate, considered change to the order of two entries within a
half. If a raster format ever precedes a vector one, Word's paste is a
flat picture and the feature has quietly stopped working.

### `fn the_svg_payload_is_utf8_with_exactly_one_trailing_nul`

Chromium's exact byte shape, which is what Office was validated
against. Both halves are asserted: the terminator is present, and it is
*one* byte rather than being doubled by a caller that had already added
one.

### `fn the_stored_svg_has_no_terminator_of_its_own`

XML 1.0 §2.2 does not permit a NUL anywhere in a document, so a file
carrying one is refused by strict parsers. This asserts the seam: the
terminator belongs to the clipboard boundary and nowhere else.

### `fn a_raster_only_payload_reports_that_it_would_degrade_words_paste`

The engine's note: *"place only the raster formats and it degrades to
a plain picture."* This is the predicate a caller asks before placing,
and the reason a half-built copy-out is worse than none.

### `fn the_registered_names_are_byte_exact`

Byte-for-byte, because `RegisterClipboardFormat` is case-sensitive:
`"png"` registers a different, private format that nothing on the
machine reads, and the copy would appear to succeed.

### `fn the_dib_header_is_top_down_bitfields_bgra`

Every one of those four is a silent-corruption failure if it is wrong:
a wrong header length reads pixels from the wrong offset, a positive
height pastes the picture upside down, `BI_RGB` leaves the alpha byte
formally undefined, and a channel-order slip turns red into blue.

### `fn the_dib_pixels_are_premultiplied_and_not_unpremultiplied_on_the_way_out`

The convention Chromium writes and Mozilla reads. A straight-alpha DIB
produces dark haloes around soft edges in exactly the readers that fall
back to `CF_DIBV5`, and it looks correct in every reader that does not.

`tiny_skia` stores premultiplied natively, so what this really asserts
is that nothing on the way out **un**-premultiplies — which is the
tempting "fix" for a channel that looks too dark.

### `fn the_dib_resolution_is_the_exact_inch_rounded_to_nearest`

300 DPI / 0.0254 is 11811.02…, so truncation gives 11810 and a paste
lands very slightly wrong. A nonsense resolution yields 0, which is
`CF_DIBV5`'s own "unspecified" and is better than a garbage number a
reader would honour.

### `enum ClipFormat`

A shell enum rather than a `u32` format id, because a format id cannot be
matched on, cannot be printed in a disclosure, and — for the three
registered names — does not exist until `RegisterClipboardFormat` has been
called at run time. What is *stable* about a clipboard format is its name
and its position in [`ORDER`], and those are what this type carries.

### `fn name`

For [`Self::Svg`] and [`Self::Png`] this is the string handed to
`RegisterClipboardFormat` **verbatim**, and it is case- and
byte-sensitive: `"PNG"` is the registered name every browser and Office
itself uses, and `"png"` would register a different, private format
that nothing reads.

For the two predefined formats it is the Win32 constant's name, which
is not passed to any API and exists so a disclosure can say which
formats went on in words an operator can search for.

### `fn is_registered`

This is the whole of why `arboard` cannot do this job: it offers no
API for a registered format, and the two formats that return `true`
here are the two that make a Word paste editable and an Inkscape paste
vector.

### `const ORDER`

# Why an order exists at all

A pasting application "typically retrieves … the first format it
recognizes". So the order is not a preference — it *is* the design. Every
application that can read two of these will take whichever pdfcer placed
first, and there is no second chance to influence that at paste time.

# What each position buys, and who it buys it from

| # | format | the reader it is there for |
|---|---|---|
| 1 | [`ClipFormat::Svg`] | Word / PowerPoint / Excel, which store it as `svgBlip` and place the shape at the page's physical size; Inkscape, whose own `clipboard.cpp` ranks SVG above EMF above PDF; LibreOffice ≥ 25.2 |
| 2 | [`ClipFormat::Emf`] | LibreOffice 24.x, which has no other vector route on Windows; Office *Paste Special ▸ Picture (Enhanced Metafile)*; Visio, CorelDRAW, CAD importers |
| 3 | [`ClipFormat::Png`] | Paint.NET, GIMP, browsers, Snip & Sketch — and Office, when the operator deliberately pastes as a picture |
| 4 | [`ClipFormat::DibV5`] | everything older than the `"PNG"` convention; Windows synthesises `CF_DIB` and `CF_BITMAP` from it |
| 5 | [`ClipFormat::Pdf`] | pdfcer's own paste, which places it as a drawing; a snapshot copy alone carries it |

# The property that makes a partial implementation harmful

**The two vector entries come first, and if they are absent Word's paste
silently becomes a flat picture.** Not an error, not a warning — a picture
that looks correct at 100% and cannot be scaled, recoloured or ungrouped.
An operator would report that as *"pdfcer's copy doesn't paste as
vectors"*, which is indistinguishable from the feature not existing, except
that it costs them the time to discover it.


# Why `application/pdf` is last, and only on a snapshot copy

The engine's note offers it as an optional fifth entry and says only
Inkscape reads it — and Inkscape already takes the SVG from position 1. Last
in the order, it changes no other program's pick.

A page or selection copy does not carry it: there it would cost a one-page
PDF through `ObjectClip::to_pdf` on a copy the operator expects to be
instant, for no reader. A snapshot copy has already made that PDF (the
engine's region cut, which every other format is drawn from), and pdfcer's
own paste reads it back as a drawing, so there it is a copy of bytes in hand
for a reader that exists.

### `struct CopyPayload`

Every field is optional so that a caller which could not produce one
payload still places the rest — but see [`ORDER`] on why "the rest" is a
dangerous thing to place when the missing one is [`ClipFormat::Svg`] or
[`ClipFormat::Emf`]. [`CopyPayload::degrades_word_to_a_picture`] is the
predicate that answers it, and a caller is expected to refuse rather than
place a payload for which it returns `true`.

### `fn formats`

Derived from `ORDER` by filtering rather than by a hand-written list,
so the order cannot be stated correctly in one place and wrongly in
another. A second list is a second answer, and the one that would go
stale is whichever is not the one being read at the time.

### `fn degrades_word_to_a_picture`

True when there is a raster to place and no vector to place before it.
A caller must refuse rather than place such a payload: the paste
succeeds, looks right, and is not what was asked for, and nothing in
the receiving application says so.

⇒ Stated as a predicate on the payload rather than as a comment on the
placement function, because it is a property of *what was produced* and
the producer is where it can still be fixed — by rendering the SVG
again, or by declining the copy with a sentence.

### `fn svg_payload`

# Why a trailing NUL on a format whose length is already known

Because that is the byte shape Microsoft validated Office against. Chromium
(≥ M127) writes the SVG to `"image/svg+xml"` NUL-terminated, and Office's
importer was tested against Chromium's clipboard rather than against a
specification — so the NUL is a compatibility fact, not a framing
requirement. `HGLOBAL` clipboard entries carry their own size; nothing
*needs* the terminator.

⚠ The failure mode if it is omitted is the worst kind: it very likely works
in most readers, and the one that reads past the end or refuses the entry
does so on somebody else's machine, in a version of Office nobody here has.
It costs one byte. It goes on.

# Why the NUL is added here and is not part of `CopyPayload::svg`

So the same `String` can be written to a `.svg` file, which must **not**
have one. A NUL inside an XML document is not permitted by XML 1.0 §2.2 at
all, and a file carrying one is refused by strict parsers. Keeping the
terminator at the placement boundary means the file path and the clipboard
path cannot accidentally share it in either direction.

### `fn dib_v5`

A `BITMAPV5HEADER` (124 bytes) followed by 32-bit-per-pixel BGRA rows,
**top-down** (a negative height), `BI_BITFIELDS` with explicit channel
masks, and the sRGB colour space.

# Premultiplied, and why the format below it is not

`CF_DIBV5`'s alpha convention is not written down anywhere normative — it
is whatever the ecosystem settled on. Chromium writes premultiplied
(`CreateDIBV5ImageDataFromN32SkBitmap`) and Mozilla settled on reading
premultiplied, so a straight-alpha DIB looks wrong — dark haloes around
anything soft-edged — in precisely the readers that fall back to this
format at all.

⇒ Which is why `"PNG"` is placed **before** it. A PNG's alpha is straight
and unambiguous (ISO 15948 §6.1), so every reader that understands the
registered `"PNG"` name gets the unambiguous answer, and only readers old
enough to need `CF_DIBV5` are exposed to the convention.

`tiny_skia` stores premultiplied RGBA natively, so the per-pixel work here
is a channel reorder and nothing else — no multiply, no divide, no
rounding, and therefore no place for the conversion to lose a value.

# The header fields that are not obvious

* `bV5Height` is **negative**. A positive height means bottom-up, which is
  the DIB default and would paste every copy upside down.
* `bV5Compression` is `BI_BITFIELDS` (3) rather than `BI_RGB` (0), because
  `BI_RGB` at 32 bpp leaves the fourth byte formally undefined and readers
  disagree about whether it is alpha or padding. The explicit masks remove
  the question.
* `bV5CSType` is `LCS_sRGB` — the four bytes `'sRGB'` as a little-endian
  `u32`, which is `0x7352_4742`. The endpoint and gamma fields that follow
  are unused for a named colour space and are written as zero.

### `fn pixels_per_metre`

One inch is exactly 0.0254 m (the international inch, fixed by definition
since 1959), so this is a conversion rather than an approximation. Rounded
to nearest because the DIB field is an integer and a truncation would put
a 300 DPI copy at 11,810 rather than 11,811 pixels per metre — which is
how a paste ends up a hair's breadth off the page size it should have had.

The 0.0254 itself now lives in [`crate::units`], beside every other
length conversion in the program. What stays here is the part that is about
the DIB and not about the inch: the guard against a nonsense DPI, the zero
returned instead, and the rounding argument above. A conversions table
should not know what a bitmap header does with a bad number.
