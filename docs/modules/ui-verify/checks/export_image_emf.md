# `ui-verify/checks/export_image_emf`

`export_image_writes_a_metafile` — the EMF radio is reachable, the press
writes a file, and the bytes are a metafile that agrees with itself.

# The gap this closes — `OPERATOR_REQUESTS.md` **O120**

The operator, 2026-09-03: *"copy and paste vector graphics into word or
inkscape"*. The engine shipped `emf::export_emf` for exactly that, because
**LibreOffice 24.x cannot read a foreign SVG clipboard entry before 25.2**
and Office's *Paste Special ▸ Picture (Enhanced Metafile)* wants a metafile
too. This shell offers it as the fourth radio in the Export-image window.

O120's own Status line sets the bar and it is the engine's:

> *"they get ticked when the GUI half is **driven**, not when it compiles."*


This header carried a banner from 2026-09-04 to 2026-09-14 saying the check
had never been run, that it was committed unrun deliberately, and that
*"the first person to run it should expect to fix it rather than to read a
verdict from it."* The banner is replaced rather than kept, because a
warning whose subject is gone stops being a warning and starts being a
false statement — and because the prediction in it was exactly right, in a
way worth recording:

1. **The check was wrong; the program was not.** Step 5 compared the
   `format=` field against the enum's `Debug` spelling, `Emf`. On
   2026-09-13, commit `28f5389` (O196) had changed the emitter to the file
   token, `emf` — correctly, and for this project's own standing reason,
   *never `Debug`-format a field a machine reads*. Nothing went red,
   because the only machine reading that field was this check and it had
   never been run. The first sweep to run it duly reported *"the radio
   drew and did not bind"* while quoting `format=emf` in the same sentence.
   **Changing a trace field's spelling is an edit to every reader of
   that field, and an unrun check is a reader that cannot object.** See
   [`EMF_KEY`], which now carries that history where the comparison is.
2. **With that corrected, the pipeline is whole.** 143,132 bytes on disk,
   `iType == 1`, the `' EMF'` signature, a header whose `nBytes` equals the
   file's length, **4,547 records**, and the shell's `bytes=` matching the
   disk exactly. Driven against `fixtures/a1-titleblock.pdf`.

—> So O120 is **driven**, which is the bar its own Status line set, and the
operator's *"copy and paste vector graphics into word or inkscape"* has a
measured answer rather than a compiled one.

⚠ One line of the apply arm was changed in the same commit and it is not
cosmetic: `export-image page=N format=` was still `{:?}`, so one export
printed `format=emf` and `format=Emf` forty lines apart. The next check
anyone writes here is the obvious one — does the file match the plan? —
and it would have compared those two and reported a disagreement that does
not exist. Both lines, and `preferences.txt`, now share one vocabulary.

# Why this needs driving rather than a unit test

Four of the five links between the radio and the file are outside anything a
`cargo test` can reach:

1. **the radio exists and is pressable.** `ImageFormat::ALL` is four long in
   a unit test whatever the window draws; whether a fourth radio is on
   screen, inside the window's height, and not clipped by the scroll area is
   a question about a laid-out frame.
2. **pressing it changes the plan.** The window keeps `format` in its own
   state and the plan is built on the press, so a build whose radio drew and
   did not bind would export a PNG under an `.emf` name and look correct
   everywhere else.
3. **the save dialog is answered.** A modal OS window, which is the whole
   reason the export is an `Action` rather than something a widget does.
4. **the writer runs against the live `DocumentView`** — the session's view,
   with its overlay and staging buffer, not a freshly-loaded `Document`.

# The assertions that make this more than a smoke test

**The file is parsed as an [MS-EMF] metafile and cross-checked against
itself and against the trace.** Three independent claims:

| claim | where it is checked | what a wrong build looks like |
|---|---|---|
| the bytes are a metafile at all | `iType == 1` at offset 0 and the `" EMF"` signature at offset 40 | a PNG written under an `.emf` name — link 2 above, and it opens in nothing |
| the metafile agrees with itself | the header's `nBytes` at offset 48 equals the file's length | a truncated write, or a header back-patched from the wrong buffer; GDI refuses such a file and the operator gets an empty paste |
| the shell's story matches the disk | the trace's `bytes=` equals the file's length | the disclosure describes one export and the disk holds another |

The middle one is the one worth having. `nBytes` is back-patched into a
placeholder after every record is written (`pdfcer_render::emf`'s writer
resizes `out` to 108 zero bytes, writes the body, then copies the header
over the front) — so a header whose `nBytes` disagrees with the file length
is a metafile that was assembled from two different runs. Nothing else in
this pipeline would notice: the file has the right extension, the right
signature, and a plausible size.

⚠ **Do NOT "improve" this by playing the metafile with
`System.Drawing.Imaging.Metafile`.** GDI+'s player mis-plays
`EMR_ALPHABLEND`, which is the record every see-through part of the page
becomes — so a GDI+ rendering of a *correct* metafile looks wrong, and the
obvious next move is to change a writer that was right. Real GDI
(`PlayEnhMetaFile`) plays it correctly and is what Office and Win32 use; the
engine's `docs/core-api` §7.10 has the numbers.
