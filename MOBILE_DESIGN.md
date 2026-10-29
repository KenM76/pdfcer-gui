# pdfcer on Android: proposed interface

Status: **approved, phone first** (O337). The picture is `mockups/mobile.html`.
This document is the spec for the mobile layouts; where the mockup and this
document disagree, this document wins. Building starts after engine v0.103.0.

**Glyphs.** Every command draws its shipped glyph from
`crates/pdfcer-gui-base/src/icons/assets/`, resolved through the same role keys
as the desktop, in the bottom bar, the tool sheet, the context menu and the
app bar alike. The mockup's icons are hand-drawn stand-ins. The icon-adoption
rule in `RIBBON_IA.md` applies unchanged: a phone surface adopts no glyph the
desktop does not ship.

## 1. The principle: same program, a different layout

Mobile is **not a fork**. The command registry, the Read / Review / Edit modes,
the manifest's tabs, groups and menus, and the disclosure rule (R8b) are the
desktop's, unchanged. What a mobile build changes:

- **Which shell regions draw.** Placement is a property of the layout, not of a command.
- **How gestures turn into the existing pointer and selection events.**

Consequences:

- R8 holds as is. A command absent from the build draws nothing on any form factor.
- The work lives in `egui-shell`, as a layout profile with gesture input, and
  needs no new domain knowledge, so R7 holds.
- The manifest gains nothing for mobile beyond one optional, per-mode list
  `bottom_bar`, which names the phone's primary commands for each mode. Without
  it, the first group of the mode's first tab is used.

## 2. Two layouts, chosen by width

| Window width | Layout | What it is |
|---|---|---|
| < 600 dp | **Phone** | top app bar, full-bleed canvas, chip strip, bottom bar, tool sheet; panels as full screens |
| 600–839 dp | **Phone**, two-column sheets | a foldable or small tablet in portrait |
| ≥ 840 dp | **Tablet** | the desktop ribbon, rail and docks, at touch sizing |

The width breakpoints are Material 3's window-size classes. The layout is chosen
again on every resize, so rotating a tablet or unfolding a phone switches it
live. Document state does not depend on the layout.

## 3. Phone layout

- **Top app bar:**
  - back;
  - the document title, with the mode and page underneath;
  - Undo (Review and Edit only);
  - the mode switch, always visible;
  - ⋮, which holds the File tab's commands (share, save, save a copy, print,
    properties).
- **Canvas:** full-bleed. One tap on an empty part of the page hides the bars
  (immersive reading); another brings them back.
- **Chip strip:** the status bar's disclosures as chips. Tapping a chip opens
  the full text in a sheet. It never blocks and never draws on the page, which
  is R8b unchanged.
- **Bottom bar:** four or five commands from the mode's `bottom_bar` list.
  - **Read:** Pages, Find, Comments, Fill & sign.
  - **Review and Edit:** Pan, then the three most recently used tools, then All tools.
- **Tool sheet:** the ribbon, opened by dragging up the bottom bar or tapping
  All tools. Each of the mode's tabs is a chip, each group a heading, and each
  command a cell in a four-column grid. It reads the same manifest as the ribbon.
- **Selection:**
  - Tap selects; long press opens the context menu from the manifest's `menus`.
  - A selection bar offers *Select several*, which replaces Ctrl-click.
  - Format and Properties share one half-height sheet, so the selection stays
    visible above it.
- **Panels:** each of the sixteen dock panels opens as a full screen with a back
  arrow. Tapping an item, such as a comment or a bookmark, returns to the page
  and acts on it.
- **Several documents:** one Android task per document, switched through Recents.

**Read is the default mode on a phone.** The common uses are an emailed
drawing, a form to fill and a document to sign. Review covers markup. Edit
works on a phone but is designed for a tablet.

## 4. Tablet layout

- The desktop ribbon, rail and docks.
- Touch sizing: items at least 48 dp tall, groups folding to their collapsed
  form sooner (`RIBBON_SCALING.md`), and the Quick Access Toolbar in the tab row.
- One tabbed dock column per side, rather than a split.
- A hardware keyboard enables the desktop keymap.

## 5. Gestures

| Gesture | Meaning |
|---|---|
| one finger drag | pan in Read and with the Pan tool; otherwise the active tool's drag |
| two-finger pinch / drag | zoom / pan, in every mode and with every tool |
| double tap | fit width, or 2× at the tap when already at fit width |
| tap | select (Review, Edit); follow a link (Read) |
| long press | the context menu, i.e. right-click |
| stylus | always the active tool; a finger pans while a stylus is in use |
| stylus barrel button | right-click |
| second finger held during a drag | the constraint modifier (Shift) |

Precision comes from a **magnifier**. While a finger drags a point or a handle,
a magnifier above the finger shows what is under it and where it will snap. The
point is committed when the finger lifts. The magnifier is a pre-commit
affordance, part of the cursor in R8b's terms, and is never stored in the file.

Handles are drawn at their desktop size, but each one responds to a touch
48 dp across, Android's recommended minimum touch target.

## 6. Files

- **Open:**
  - the Storage Access Framework picker (`ACTION_OPEN_DOCUMENT`);
  - *open with* from email, Files and Drive (a `VIEW` intent for `application/pdf`).
- **Save:** writes back to the same content URI.
- **Save a copy:** `ACTION_CREATE_DOCUMENT`.
- **Share:** a `SEND` intent.
- **Print:** Android's print service, in place of the Windows print crates.

## 7. What a phone build leaves out at first

Each is a command not registered in the Android build, so it renders nothing (R8, R9):

- **Batch Tools** (folders of files), until SAF directory access is designed.
- **Windows printing.** Android's print service replaces it.
- **3D models.**
- **OCR models** move to an optional download. OCR commands register only once
  the download is present.

## 8. What stands between today and a launching APK

From the engine's Android feasibility measurement (pdfcer `ROADMAP.md`,
"Android"): the engine crates compile untouched for `aarch64-linux-android`.
The GUI side:

1. **`rfd` has no Android backend.**
   - Open and save go through a file-access trait in `pdfcer-gui`.
   - On Windows, that trait is implemented by `rfd`; on Android, by SAF calls over JNI.
2. **The eframe activity type.**
   - eframe's `accesskit` feature does not build with `android-native-activity`
     (`D:/dev/rag/egui/eframe_035_android_support_accesskit_game_activity_conflict.md`).
   - So the Android build uses `android-game-activity`, which keeps accessibility.
3. **Windows-only crates** (native window, native GL, printing) sit behind
   `cfg(windows)` at the dependency level. Their commands are registered only
   there, following R8 rather than per-button `cfg`.
4. **Fonts.** Installed-font discovery reads `/system/fonts` on Android.
5. **The patched `winit` and `egui` checkouts** need their Android paths built
   once to confirm the patches do not touch them.
6. **ui-verify** drives an emulator through `adb` (touch injection, screencap).
   Until then, a phone phase is NOT VERIFIED (R1).

## 9. Staging: phone first

1. **A. Phone launch.**
   - An Android phone build opens a PDF through SAF and open-with.
   - It uses the phone layout's app bar, canvas and chip strip, with pinch, pan and long press.
   - The Read bottom bar is in place.
   - It is driven by hand on the Pixel 3a (API 34, x86_64) emulator already installed.
2. **B. The rest of the phone layout.**
   - Review and Edit bottom bars, the tool sheet, panels as full screens, the
     Format and Properties sheet, and the magnifier.
3. **C. Driven checks.**
   - ui-verify's `adb` backend, with the phone checks asserted on screenshots
     and the `PDFCER_DIAG` trace.
4. **D. Tablet layout** (section 4), after the phone ships.

**Toolchain.**
- The Android SDK (platform 34, build-tools 34.0.0, `adb`, emulator) and the
  `aarch64-linux-android` Rust target are installed.
- Still missing: the NDK, the `x86_64-linux-android` target that the emulator
  needs, and `cargo-ndk`.

Each stage keeps the desktop build unchanged and green.

## 10. Rulings and open questions

- Phone first, tablet afterwards. (Ken)
- Glyphs everywhere the desktop has one. (Ken)
- Open: should Edit be available on a phone, or only Read and Review?
- Open: distribution, a sideloaded APK on GitHub releases or the Play Store
  (which needs a developer account and a review)?
