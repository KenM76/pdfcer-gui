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
