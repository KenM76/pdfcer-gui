//! # pdfcer-gui-base — the floor of the pdfcer-gui crate stack
//!
//! Modules that reference no other module of `pdfcer-gui`, lifted out of
//! it so that the compiler, rather than a review, is what keeps them beneath
//! the application.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/lib.md`.

#![forbid(unsafe_code)]

// Finding the operator's installed Acrobat and handing the open document over
// to it. Everything that can only be true of a real machine — reading the
// registry, starting a process — is behind a trait in there, so the decisions
// are testable without either.
pub mod acrobat;
/// The blank-A4 template a new document is parsed from.
pub mod blank;

/// The one place the program reads a wall clock: PDF and ISO dates in UTC.
/// `pdfcer-core` refuses to supply a timestamp, for determinism and because a
/// date is a claim; its header carries why the answer is UTC and never a local
/// time labelled as one.
pub mod clock;

/// Where the view is, when the scroll offset can no longer say.
pub mod deepanchor;

/// The opt-in trace of what the shell actually received.
pub mod diag;

/// Which file a document came from, and which form field is selected.
pub mod docidentity;

/// The text editor's window-free arithmetic: caret, lines, disposition, pen.
pub mod editmodel;

/// OCR: what image the recogniser is shown, the thread it runs on, and the
/// named refusals it can come back with. It authors no PDF — `pdfcer-core`'s
/// `ocr::layer` writes the invisible mode-3 sandwich. See its header for why
/// recognition reads the document as it was OPENED, and for the y-flip it
/// deliberately does not perform.
pub mod ocr;

pub mod pagedrag;

/// The pixel proof that "line weights off" thins a drawing, in the right
/// direction. Tests only.
mod hairline;

/// A revision number per page, so a change to one page repaints only it.
pub mod pageepoch;

/// Does a document's page tree still agree with itself: the raw `/Count`
/// against the leaves actually reachable, audited on every save, and which
/// refusal a disagreement owes. The wording lives in `pdfcer-gui`'s `text`.
pub mod pagetree;

/// The per-page raster scale this document has been measured unable to reach,
/// so zoom stops there instead of showing an error.
pub mod rasterceiling;

/// The ground outside the sheet: the box to rasterize so an object past the
/// page edge is painted.
pub mod rasterhalo;

/// The window's rectangle in PDF user space: the region tier's one conversion.
pub mod rasterregion;

/// Whole page, or just the window: the tier decision, made from numbers.
pub mod rasterstrategy;

/// Rendered pixmaps into egui textures, and what each upload is a picture of.
pub mod raster;

pub mod ribbontabs;

/// A string the operator typed that must never reach a log — one type, and its
/// whole reason for existing is its `Debug`. See its header: a `{:?}` on an
/// action carrying a password writes it into the trace file `tools/ui-verify`
/// keeps as evidence.
pub mod secret;

pub mod settingspages;

pub mod settings;

/// **Acrobat-compatible custom stamp collections** — the shell half of
/// engine `Pass 288.0`, and the answer to `OPERATOR_REQUESTS.md` **O169**.
pub mod stamps;

/// The texture cache behind a continuous strip of pages, under a texel budget.
pub mod stripcache;

/// The operator-facing copy: every string the shell shows.
pub mod text;

/// Poster printing: one page across many sheets, with a band along each
/// sheet's top and left for cut marks and the assembly label, and the drawing
/// of both. The tiling is `pdfcer_print::imposition::plan_poster`'s.
pub mod poster;

pub mod pressure;

/// The icon set: SVG path data, a subset parser, a tiny-skia rasterizer and
/// the painter `egui-shell`'s ribbon calls back into. Supplying that painter
/// is what stops the ribbon falling back to text labels — see `icons::paint`.
pub mod icons;

/// Which optional-content groups the operator has hidden.
pub mod layeroverride;

pub mod redact;

/// How sharply a page is drawn, and how long zoom waits before drawing it.
pub mod renderquality;

/// The background render thread: one page raster at a time, cancellable, and
/// a typed refusal when there are no pixels.
pub mod renderworker;

/// Putting the operator's own digital signature on a document.
#[cfg(feature = "signing")]
pub mod sign;

/// Where signature trust ANCHORS come from, and the three facts they let
/// this shell state.
pub mod trust;

/// **The one length-conversion table for this program.**
///
/// Points to millimetres, inches, metres or any other engine [`Unit`], and
/// back. Every operator-facing length goes through it, and the gate
/// `tools/gates/check-unit-conversion.sh` fails the build when a second copy
/// of the constant appears anywhere under either crate's `src/`.
///
/// It exists because of a measured defect, not for tidiness: a sheet of
/// exactly 210.5 mm rendered `210` in the page-thumbnail tooltip and `211`
/// in the print dialogue. Both surfaces computed the same value; they disagreed
/// on the ROUNDING RULE, because half the program wrote `.round()` (half away
/// from zero, the CAD convention) and the other half wrote `{:.0}` in a format
/// string, which is Rust's default and rounds half to even. The module's header
/// argues that choice rather than leaving it to whichever spelling a caller
/// reached for.
///
/// [`Unit`]: pdfcer_core::dimension::Unit
pub mod units;

/// What one canvas remembers between frames.
pub mod viewframe;

/// Which page is shown, at what zoom, in what arrangement, and where.
pub mod viewer;

/// What a page verb acts on, and what a page move means.
pub mod pageops;

/// What a push button does when it is pressed, and its translation to the engine type.
pub mod pushbutton;

/// The three markup kinds that carry words: text box, sticky note, stamp.
pub mod wordmarkup;

/// Where and when the dock layout is written to disk.
pub mod dockpersist;

/// A dialog is an OS window: the viewport host, its fit and its placement.
pub mod dialoghost;

/// Which parts of a rendered print sheet actually carry ink.
pub mod printink;

/// pdfcer's own crosshair and I-beam cursors, supplied to the OS.
pub mod cursor;

/// The measure tools' pick state machines and scale entry.
pub mod measure;

/// The eight resize grips plus move, their hit test and their cursors.
pub mod handles;

/// Where pdfcer looks for a font it has to embed.
pub mod fontsearch;

/// The planned, directed and tab-scoped command lists the manifest checks against.
pub mod commandregisters;

/// One colour control with three honest states: set, mixed, and none.
pub mod swatch;

/// The seven tolerance forms of a ce dimension and their editors.
pub mod tolerance;

/// The two questions the bookmarks panel asks of an outline tree.
pub mod bookmarktree;

/// Drawing the left rail.
pub mod rail;

/// Where on the window a file was dropped.
pub mod filedrag;

/// Tab moves through what the operator clicked on.
pub mod tabnav;

/// The verbs that re-shape a page's own text.
pub mod textverbs;

/// How long the engine takes to accept one edit; a `#[cfg(test)]`, `#[ignore]`d instrument.
pub mod editlatency;

/// What a save produced, or why it produced nothing.
pub mod saveoutcome;

/// The copied selection, as a picture other programs can paste.
pub mod clipimage;

/// How a placed stamp is sized to its box.
pub mod stampfit;

/// Keeping a spinner's value alive between frames.
pub mod spinnerdraft;

/// The forms panel-to-canvas channel: which field to spotlight.
pub mod formspotlight;

/// How big the program's own controls are drawn.
pub mod chromescale;

/// The sentences one edit owed, kept until they are said.
pub mod editdisclosure;

/// Preparing a typed query for the engine.
pub mod findquery;

/// The region names the print dialog publishes for the driving harness.
pub mod printregions;

/// The reach allow-list.
pub mod reachregister;

/// Which piece of View > Display an action is about.
pub mod displaypiece;

/// How much memory the page cache may spend.
pub mod prefscache;

/// Which chord means which paste.
pub mod pastechords;

pub mod remotecontrol;
/// What the mouse wheel does when the document is not a scroll.
pub mod wheelpaging;

/// What an operator is shown when a page first appears.
pub mod openingfit;

/// Does this document reach outside itself?
pub mod reachout;

/// Making pdfcer the program Windows opens a PDF with.
pub mod assoc;

/// Where an annotation's artwork actually sits.
pub mod annotquad;

/// Which surface this frame's gesture belongs to.
pub mod pasteboard;

/// What rides along when a resize scales something.
pub mod scaling;

/// From glyph cells to the boxes a text selection shows.
pub mod textselbands;

/// Which note pop-ups are showing, and who decided.
pub mod notepopupopen;

/// The pure arithmetic behind panning and zooming.
pub mod viewgeometry;

/// Solid or dashed, on every surface that draws a markup line.
pub mod linestyle;

/// Where the redacted document goes.
pub mod redactdestination;

/// What the file states about tab order.
pub mod taborderstated;

/// Narrowing the Layers list as you type.
pub mod layersearch;

/// Several pages at once, and what an undrawn one says.
pub mod renderstrip;

/// Which strip pages may be ordered, what each holds, and render-ahead.
pub mod stripschedule;

/// The colour recognised text is drawn in.
pub mod ocrlayerpref;

/// One reading of the load anomalies, for the status bar.
pub mod anomalycensus;

/// Which pages the operator has picked.
pub mod pageselection;

/// Acrobat's own markup colours.
pub mod markuppalette;

/// What a mode lets the canvas do.
pub mod modecapability;

/// The documents this operator had open, on disk.
pub mod recentfiles;

/// The View > Window verbs that change the shape of the window.
pub mod windowshape;

/// Which ribbon groups give up their rows first.
pub mod ribbonladder;

/// What is in the left rail.
pub mod railmanifest;

/// What the status bar can afford to show when the window is narrow.
pub mod statusfitting;

/// The one description of a page object.
pub mod objectsummary;

/// Turning a document into a comment list.
pub mod commentmodel;

/// Turning the operator's font folders into donors an embed can use.
pub mod fontlibrary;

/// What Shift does to a drag, written down once.
pub mod constrain;

/// Which family an annotation belongs to.
pub mod annotkind;

/// What a click is allowed to land on.
pub mod pick;

/// Where the pick filter is kept between runs.
pub mod pickstore;

/// The worded decline: a refused command's sentence and its lifetime.
pub mod declined;
/// Why a canvas verb declined.
pub mod refusals;

/// Exporting pages as images: formats, resolution and page scope.
pub mod imageexport;

/// Exporting a document as plain text.
pub mod exporttext;

/// A tagged PDF's own structure for the Word and table exports, with its disclosure.
pub mod taggedexport;

/// What the export windows remember between jobs.
pub mod exportremembered;

/// A panel's controls pinned under its list, so no row moves when they appear.
pub mod panelfooter;

/// Which dockable panels exist, and the ribbon command that opens each.
pub mod panelid;

/// Exporting detected tables as spreadsheet cells.
pub mod tableexport;

/// Attaching the context menus to the command registry.
pub mod menus_wiring;

/// The picker for what a push button does.
pub mod buttonactionpicker;

/// Which layer a selection belongs to.
pub mod layermembership;

/// The keyboard-shortcuts reference window.
pub mod shortcutsdialog;

/// The prompt before handing the document to Acrobat.
pub mod acrobatprompt;

/// The Settings row for the default PDF application.
pub mod defaultappsetting;

/// The verbs that exist only to move a native file picker out of the layout pass.
pub mod writeaction;

/// Page space to screen space and back, for one page at one zoom.
pub mod canvasmapping;

/// The snap indicator and its screen tolerance.
pub mod snapmark;

/// The overlay colour roles the canvas registers with the theme.
pub mod overlayroles;

/// Where a note popup sits and what it holds, as values.
pub mod notepopupmodel;

/// The node marks drawn over a selected path.
pub mod anchormarks;

/// One page decomposed into selectable objects, parts and nodes.
pub mod objectprovider;

/// The seam a hit-testable content model plugs into.
pub mod canvastarget;

/// How deep the last click reached, and how deep it could have.
pub mod pickdepth;

/// A selection as four integers, never a position.
pub mod selectionidentity;

/// Click selects the whole thing; double-click goes inside it.
pub mod smartselect;

/// The right-click route to one line of a text block.
pub mod runmenu;

/// Showing what a measuring click will pick, before it picks it.
pub mod measurehover;

/// The one module that knows `pdfcer-print` exists.
pub mod printspooler;

/// What a print-preview bitmap is a picture of.
pub mod printpreviewkey;

/// What the operator has actually LOOKED at in the print preview.
pub mod printverdicts;

/// The print preview's arithmetic, with no dialog in it.
pub mod printgeometry;

/// Pick the print sheet from the pages.
pub mod printautopaper;

/// Where the page sits on the print paper.
pub mod printposition;

/// Narrowing and ordering the Comments panel's work list.
pub mod commentfilter;

/// A comment's review status.
pub mod commentreviewstate;

/// The note being typed, and the stamp that keeps it honest.
pub mod commentnote;

/// Fixtures for this crate's tests.
#[cfg(test)]
pub mod testsupport;

/// The form tab-order working list and its reorder arithmetic.
pub mod tabordermodel;

/// A form field's appearance colours, as a picker.
pub mod mkcolour;

/// A ce dimension's text and tolerance overrides.
pub mod dimensionoverrides;

/// Which markup annotation the markup tool draws.
pub mod markupkind;

/// The markup pen: colour, width, opacity and dash, per kind.
pub mod markuppen;

/// The Markup ▸ Style group's pen control.
pub mod markupswatch;

/// Spelling a manifest chord into modifiers and a key.
pub mod keychord;

/// Scripted keystrokes for a window OS input cannot reach.
pub mod keyscripted;
/// Scripted clicks, drags and wheel turns for a window OS input cannot reach.
pub mod pointerscripted;

/// How a form field's in-place editor sets its text.
pub mod formeditortext;

/// The in-place editor drawn over a text form field.
pub mod formtextbox;

/// The per-domain verbs an Action carries, as data.
pub mod subactions;

/// What a Find asks for and which way it steps.
pub mod findrequest;

/// A place on a page a bookmark asked the view to arrive at.
pub mod navdest;

/// The verbs whose subject is an annotation.
pub mod annotaction;

/// The verbs whose subject is text style, a ce dimension, a redaction or an attachment.
pub mod editactions;

/// The kind of form field a placement authors.
pub mod formfieldkind;

/// A form-filling edit the Forms panel raises.
pub mod formedit;

/// The verbs whose subject is a form field.
pub mod fieldaction;

/// The verbs that author a layer.
pub mod layeraction;

/// What the form-field placement dialog collects and remembers.
pub mod formdraft;

/// The ruler guides a document carries, and their on-disk spelling.
pub mod guidemodel;

/// The action vocabulary: one variant per operator intent.
pub mod appaction;

/// The live geometry preview a gesture draws, in page space, and the hold that keeps it on screen until the raster catches up..
pub mod shapepreview;

/// A scroll the view still owes: a destination point, a minimum region to reveal, or a find hit, each waiting for a frame that shows its page..
pub mod scrolltarget;

/// An annotation the operator has selected on the canvas: which one, and where its outline sits..
pub mod annotselection;

/// The canvas selection: the chosen objects, the rung they were chosen at, their cached outlines, and the selected annotation..
pub mod selectionstate;

/// A run of selected page text: its two ends in the text model, the edit epoch it was resolved at, and its highlight geometry..
pub mod textselection;

/// The per-document caches an open document carries: each a cell keyed by page and edit epoch, filled on first read..
pub mod doccache;

/// The shell's own preferences and their on-disk format.
pub mod prefs;

/// The page-thumbnail render budget.
pub mod pagebudget;

/// Where a document's guides live between sessions.
pub mod guidestore;

/// One open document and everything the shell derives from it.
pub mod opendoc;

/// The bytes a copy-out places, and the order they go in.
pub mod clipboard;

/// Putting a password on a document, changing what it allows, and taking the
/// protection off.
pub mod protect;

/// **Tests only** — the engine properties a render past the page edge stands on.
pub mod rasteroffpage;

/// Searching the page text, and bringing a hit onto the screen.
pub mod find;

/// What a redaction looks like once applied: its fill and caption.
pub mod redactlook;

/// One reader for everything typed into a value box: units, arithmetic, relative entry.
pub mod entry;

/// Whether a page's crop box hides part of its sheet.
pub mod pagebox;

/// What a corner drag on a ce dimension or ink mark asks for.
pub mod vertexintent;

/// Align and Distribute's arithmetic: boxes in, one delta per box out.
pub mod alignlayout;

/// A freehand mark's points, addressed per stroke and flat at once.
pub mod inkaddress;
