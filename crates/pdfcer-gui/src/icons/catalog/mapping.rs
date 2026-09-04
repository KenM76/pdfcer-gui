//! # icons::catalog::mapping — every [`Icon`] resolved to its asset and its key
//!
//! Three total functions over [`Icon`], separated from the enum next door
//! because of R2 and because they answer a different question. `catalog/mod.rs`
//! says **what a glyph means and why it exists**; this file says **which bytes
//! it draws and what string names it**. A reader arriving to add an icon needs
//! the first; a reader chasing "why did this ribbon button draw nothing" needs
//! the second, and the two searches were previously the same 1,167-line file.
//!
//! ## Why the split fell here and not somewhere else
//!
//! Because the seam is total-ness. [`Icon::ALL`], [`Icon::source`] and
//! [`Icon::name`] are each required to cover every variant, and each one fails
//! LOUDLY when it does not: `source` and `name` are exhaustive `match`es the
//! compiler checks, and `ALL` is checked by `all_is_exhaustive_and_free_of_duplicates`
//! because a missing entry there is the one omission the compiler cannot see.
//! Keeping the three together means the three lists a new variant must join are
//! adjacent, which is the property that stops one of them being forgotten.
//!
//! The doc comments — the rulings about what a glyph may not look like — stay
//! with the enum, because that is where somebody choosing a glyph reads.

use super::super::assets;
use super::Icon;

impl Icon {
    /// Every icon, in catalogue order.
    ///
    /// This is the list the catalogue-wide tests walk, and it is what makes
    /// "every shipped asset is valid" an enforced property rather than a
    /// hope — so a new [`Icon`] variant MUST be added here or it ships
    /// unverified. `all_is_exhaustive` guards the omission that would
    /// otherwise be invisible.
    pub const ALL: &'static [Icon] = &[
        Icon::Open,
        Icon::Save,
        Icon::Sidebar,
        Icon::Comment,
        Icon::ChevronLeft,
        Icon::Back,
        Icon::ChevronRight,
        Icon::ChevronDown,
        Icon::Search,
        Icon::ChevronUp,
        Icon::Close,
        Icon::ZoomOut,
        Icon::ZoomIn,
        Icon::FitPage,
        Icon::FitWidth,
        Icon::FitHeight,
        Icon::RotateCcw,
        Icon::RotateCw,
        Icon::Properties,
        Icon::Markup,
        Icon::Text,
        Icon::EditText,
        Icon::AddText,
        Icon::EditObjects,
        Icon::FormField,
        Icon::Measure,
        Icon::Undo,
        Icon::Redo,
        Icon::Copy,
        Icon::Tools,
        Icon::Keyboard,
        Icon::Info,
        Icon::ShowPoints,
        Icon::Bookmarks,
        Icon::Layers,
        Icon::Signatures,
        Icon::Fonts,
        Icon::Pointer,
        Icon::ShapeRect,
        Icon::ShapeEllipse,
        Icon::ShapeArrow,
        Icon::ShapePolyline,
        Icon::ShapePolygon,
        Icon::ShapeCloud,
        Icon::ShapeInk,
        Icon::ShapeHighlight,
        Icon::TextSelect,
        Icon::TextUnderline,
        Icon::TextStrikeout,
        Icon::TextSquiggly,
        Icon::TextFreeText,
        Icon::TextSticky,
        Icon::Stamp,
        Icon::ImportFormData,
        Icon::Combine,
        Icon::Split,
        Icon::InsertPages,
        Icon::FontFolders,
        Icon::Redact,
        // The 2026-08-14 pass, in the enum's own order.
        Icon::Print,
        Icon::Export,
        Icon::Settings,
        Icon::InsertImage,
        Icon::SetScale,
        Icon::PageSingle,
        Icon::PageContinuous,
        Icon::PageFacing,
        Icon::PageFacingContinuous,
        Icon::ZoomRegion,
        Icon::ZoomSelection,
        Icon::Cut,
        Icon::Paste,
        Icon::Cursor,
        Icon::CursorNode,
        Icon::Hand,
        Icon::Rulers,
        Icon::Grid,
        Icon::Guides,
        Icon::Pages,
        Icon::Forms,
        Icon::ReadMode,
        Icon::Fullscreen,
        Icon::FloatingPanels,
        Icon::ResetLayout,
        Icon::Delete,
        Icon::PageExtract,
        Icon::FormFlatten,
        Icon::ManageList,
        // The 2026-08-21 pass — the selection filter's rows (O17).
        Icon::PickText,
        Icon::PickPath,
        Icon::PickPart,
        Icon::PickFormXObject,
        Icon::PickLink,
        // Orphaned by breaking their aliases (2026-09-04); kept so the
        // catalogue-wide tests still walk the art.
        Icon::Document,
        Icon::Convert,
        // The 2026-09-04 batch — see `super`'s note on the enum.
        Icon::ApplyRedactions,
        Icon::Attachment,
        Icon::Accept,
        Icon::CheckBox,
        Icon::CloseOthers,
        Icon::Collapse,
        Icon::CopyDocumentText,
        Icon::CopyPageText,
        Icon::DimensionGroups,
        Icon::NextDocument,
        Icon::PreviousDocument,
        Icon::DropDown,
        Icon::EmbedFonts,
        Icon::Expand,
        Icon::FinishShape,
        Icon::Locked,
        Icon::MeasureAngle,
        Icon::MeasureLength,
        Icon::MeasurePerimeter,
        Icon::MeasureRadius,
        Icon::MergeInto,
        Icon::New,
        Icon::NewFromTemplate,
        Icon::PushButton,
        Icon::PutDown,
        Icon::RadioButton,
        Icon::Recent,
        Icon::RecogniseText,
        Icon::RedactSelection,
        Icon::Reflow,
        Icon::RenderDiagnostics,
        Icon::SaveAs,
        Icon::SaveCompacted,
        Icon::SaveCopy,
        Icon::UnembedFonts,
        Icon::WheelFlip,
        // The five with no ribbon control yet, 2026-09-04. `ExportImage` is
        // named by `file.export_image`; the other four are art before button,
        // and joining this list is what puts them under the catalogue-wide
        // tests — which is the whole reason a variant exists for each.
        Icon::CopyAsVector,
        Icon::Encrypt,
        Icon::ExportImage,
        Icon::OpenInAcrobat,
        Icon::Permissions,
    ];

    /// The asset's SVG source.
    ///
    /// `include_str!` at compile time rather than a runtime file read,
    /// because pdfcer ships single-folder portable: the executable must not
    /// depend on an `assets/` directory travelling beside it, and an icon
    /// that fails to load at startup is not a failure mode worth having when
    /// the whole set is ~79 KB of text. See [`super::assets`] for why the
    /// `.svg` files live inside `src/icons/`.
    #[must_use]
    pub const fn source(self) -> &'static str {
        match self {
            Icon::ApplyRedactions => assets::APPLY_REDACTIONS,
            Icon::Attachment => assets::ATTACHMENT,
            Icon::Accept => assets::CHECK,
            Icon::CheckBox => assets::CHECK_BOX,
            Icon::CloseOthers => assets::CLOSE_OTHERS,
            Icon::Collapse => assets::COLLAPSE,
            Icon::CopyDocumentText => assets::COPY_DOCUMENT_TEXT,
            Icon::CopyPageText => assets::COPY_PAGE_TEXT,
            Icon::DimensionGroups => assets::DIMENSION_GROUPS,
            Icon::NextDocument => assets::DOCUMENT_NEXT,
            Icon::PreviousDocument => assets::DOCUMENT_PREVIOUS,
            Icon::DropDown => assets::DROP_DOWN,
            Icon::EmbedFonts => assets::EMBED_FONTS,
            Icon::Expand => assets::EXPAND,
            Icon::FinishShape => assets::FINISH_SHAPE,
            Icon::Locked => assets::LOCK,
            Icon::MeasureAngle => assets::MEASURE_ANGLE,
            Icon::MeasureLength => assets::MEASURE_LENGTH,
            Icon::MeasurePerimeter => assets::MEASURE_PERIMETER,
            Icon::MeasureRadius => assets::MEASURE_RADIUS,
            Icon::MergeInto => assets::MERGE,
            Icon::New => assets::NEW_DOCUMENT,
            Icon::NewFromTemplate => assets::NEW_FROM_TEMPLATE,
            Icon::PushButton => assets::PUSH_BUTTON,
            Icon::PutDown => assets::PUT_DOWN,
            Icon::RadioButton => assets::RADIO_BUTTON,
            Icon::Recent => assets::RECENT,
            Icon::RecogniseText => assets::RECOGNISE_TEXT,
            Icon::RedactSelection => assets::REDACT_SELECTION,
            Icon::Reflow => assets::REFLOW,
            Icon::RenderDiagnostics => assets::RENDER_DIAGNOSTICS,
            Icon::SaveAs => assets::SAVE_AS,
            Icon::SaveCompacted => assets::SAVE_COMPACT,
            Icon::SaveCopy => assets::SAVE_COPY,
            Icon::UnembedFonts => assets::UNEMBED_FONTS,
            Icon::WheelFlip => assets::WHEEL_FLIP,
            Icon::CopyAsVector => assets::COPY_AS_VECTOR,
            Icon::Encrypt => assets::ENCRYPT,
            Icon::ExportImage => assets::EXPORT_IMAGE,
            Icon::OpenInAcrobat => assets::OPEN_IN_ACROBAT,
            Icon::Permissions => assets::PERMISSIONS,
            Icon::Open | Icon::FontFolders => assets::FOLDER,
            Icon::Save => assets::SAVE,
            Icon::Sidebar => assets::SIDEBAR,
            Icon::Comment => assets::COMMENT,
            Icon::ChevronLeft => assets::CHEVRON_LEFT,
            Icon::Back => assets::BACK,
            Icon::ChevronRight => assets::CHEVRON_RIGHT,
            Icon::ChevronDown => assets::CHEVRON_DOWN,
            Icon::Search => assets::SEARCH,
            Icon::ChevronUp => assets::CHEVRON_UP,
            Icon::Close => assets::CLOSE,
            Icon::ZoomOut => assets::ZOOM_OUT,
            Icon::ZoomIn => assets::ZOOM_IN,
            Icon::FitPage => assets::FIT_PAGE,
            Icon::FitWidth => assets::FIT_WIDTH,
            Icon::FitHeight => assets::FIT_HEIGHT,
            Icon::RotateCcw => assets::ROTATE_CCW,
            Icon::RotateCw => assets::ROTATE_CW,
            Icon::Properties => assets::PROPERTIES,
            Icon::Document => assets::DOCUMENT,
            Icon::Convert => assets::CONVERT,
            Icon::Markup => assets::MARKUP,
            Icon::Text => assets::TEXT,
            Icon::EditText => assets::EDIT,
            Icon::AddText => assets::ADD_TEXT,
            Icon::FormField => assets::FORM_FIELD,
            Icon::EditObjects => assets::EDIT_OBJECTS,
            Icon::ShowPoints => assets::SHOW_POINTS,
            Icon::Bookmarks => assets::BOOKMARKS,
            Icon::Layers => assets::LAYERS,
            Icon::Signatures => assets::SIGNATURES,
            Icon::Fonts => assets::FONTS,
            Icon::Measure => assets::RULER,
            Icon::Undo => assets::UNDO,
            Icon::Redo => assets::REDO,
            Icon::Copy => assets::COPY,
            Icon::Tools => assets::TOOL,
            Icon::Keyboard => assets::KEYBOARD,
            Icon::Info => assets::INFO,
            Icon::Pointer => assets::POINTER,
            Icon::ShapeRect => assets::SHAPE_RECT,
            Icon::ShapeEllipse => assets::SHAPE_ELLIPSE,
            Icon::ShapeArrow => assets::SHAPE_ARROW,
            Icon::ShapePolyline => assets::SHAPE_POLYLINE,
            Icon::ShapePolygon => assets::SHAPE_POLYGON,
            Icon::ShapeCloud => assets::SHAPE_CLOUD,
            Icon::ShapeInk => assets::SHAPE_INK,
            Icon::ShapeHighlight => assets::SHAPE_HIGHLIGHT,
            Icon::TextSelect => assets::TEXT_SELECT,
            Icon::TextUnderline => assets::TEXT_UNDERLINE,
            Icon::TextStrikeout => assets::TEXT_STRIKEOUT,
            Icon::TextSquiggly => assets::TEXT_SQUIGGLY,
            Icon::TextFreeText => assets::TEXT_FREETEXT,
            Icon::TextSticky => assets::TEXT_STICKY,
            Icon::Stamp => assets::STAMP,
            Icon::Combine => assets::LINK,
            Icon::Split => assets::SCISSORS,
            Icon::InsertPages => assets::INSERT_PAGES,
            Icon::ImportFormData => assets::UPLOAD,
            Icon::Redact => assets::REDACT,
            Icon::Print => assets::PRINTER,
            Icon::Export => assets::DOWNLOAD,
            Icon::Settings => assets::SETTINGS,
            Icon::InsertImage => assets::IMAGE,
            Icon::SetScale => assets::SET_SCALE,
            Icon::PageSingle => assets::PAGE_SINGLE,
            Icon::PageContinuous => assets::PAGE_CONTINUOUS,
            Icon::PageFacing => assets::PAGE_FACING,
            Icon::PageFacingContinuous => assets::PAGE_FACING_CONTINUOUS,
            Icon::ZoomRegion => assets::ZOOM_REGION,
            Icon::ZoomSelection => assets::ZOOM_SELECTION,
            Icon::Cut => assets::CUT,
            Icon::Paste => assets::PASTE,
            Icon::Cursor => assets::CURSOR,
            Icon::CursorNode => assets::CURSOR_NODE,
            Icon::Hand => assets::HAND,
            Icon::Rulers => assets::RULERS,
            Icon::Grid => assets::GRID,
            Icon::Guides => assets::GUIDES,
            Icon::Pages => assets::PAGES,
            Icon::Forms => assets::FORMS,
            Icon::ReadMode => assets::READ_MODE,
            Icon::Fullscreen => assets::FULLSCREEN,
            Icon::FloatingPanels => assets::FLOATING_PANELS,
            Icon::ResetLayout => assets::RESET_LAYOUT,
            Icon::Delete => assets::DELETE,
            Icon::PageExtract => assets::PAGE_EXTRACT,
            Icon::FormFlatten => assets::FORM_FLATTEN,
            Icon::ManageList => assets::LIST,
            Icon::PickText => assets::PICK_TEXT,
            Icon::PickPath => assets::PICK_PATH,
            Icon::PickPart => assets::PICK_PART,
            Icon::PickFormXObject => assets::PICK_FORM_XOBJECT,
            Icon::PickLink => assets::PICK_LINK,
        }
    }

    /// The stable key this icon answers to.
    ///
    /// Two jobs, and they are the same string on purpose:
    ///
    /// 1. **It is the application's icon key**, the thing a command names
    ///    with `.with_icon("…")` and the thing `egui-shell` hands back in
    ///    `IconRequest::key`. The shell never interprets it — an icon set is
    ///    a licensing and rasterization decision, which is the application's
    ///    business — so this is the only place the vocabulary is defined.
    /// 2. **It is the texture's debug name.** egui keys textures by handle,
    ///    not by name, so that part is purely for debuggers and texture
    ///    inspectors — but a texture list full of "icon" tells you nothing,
    ///    and one full of `icon:rotate-ccw@32:Bold` tells you everything.
    ///
    /// Kebab-case throughout, matching the command ids and the asset
    /// filenames it was salvaged from.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Icon::ApplyRedactions => "apply-redactions",
            Icon::Attachment => "attachment",
            Icon::Accept => "check",
            Icon::CheckBox => "check-box",
            Icon::CloseOthers => "close-others",
            Icon::Collapse => "collapse",
            Icon::CopyDocumentText => "copy-document-text",
            Icon::CopyPageText => "copy-page-text",
            Icon::DimensionGroups => "dimension-groups",
            Icon::NextDocument => "document-next",
            Icon::PreviousDocument => "document-previous",
            Icon::DropDown => "drop-down",
            Icon::EmbedFonts => "embed-fonts",
            Icon::Expand => "expand",
            Icon::FinishShape => "finish-shape",
            Icon::Locked => "lock",
            Icon::MeasureAngle => "measure-angle",
            Icon::MeasureLength => "measure-length",
            Icon::MeasurePerimeter => "measure-perimeter",
            Icon::MeasureRadius => "measure-radius",
            Icon::MergeInto => "merge",
            Icon::New => "new-document",
            Icon::NewFromTemplate => "new-from-template",
            Icon::PushButton => "push-button",
            Icon::PutDown => "put-down",
            Icon::RadioButton => "radio-button",
            Icon::Recent => "recent",
            Icon::RecogniseText => "recognise-text",
            Icon::RedactSelection => "redact-selection",
            Icon::Reflow => "reflow",
            Icon::RenderDiagnostics => "render-diagnostics",
            Icon::SaveAs => "save-as",
            Icon::SaveCompacted => "save-compact",
            Icon::SaveCopy => "save-copy",
            Icon::UnembedFonts => "unembed-fonts",
            Icon::WheelFlip => "wheel-flip",
            Icon::CopyAsVector => "copy-as-vector",
            Icon::Encrypt => "encrypt",
            Icon::ExportImage => "export-image",
            // ui-text-exempt: icon key, never displayed. The vendor name is the
            // COMMAND's label, not this string, and the art carries no mark.
            Icon::OpenInAcrobat => "open-in-acrobat",
            Icon::Permissions => "permissions",
            Icon::Open => "open",
            Icon::Save => "save",
            Icon::Sidebar => "sidebar",
            Icon::Comment => "comment",
            Icon::Close => "close",
            Icon::ChevronLeft => "chevron-left",
            Icon::Back => "back",
            Icon::ChevronRight => "chevron-right",
            Icon::ChevronDown => "chevron-down",
            Icon::Search => "search",
            Icon::ChevronUp => "chevron-up",
            Icon::ZoomOut => "zoom-out",
            Icon::ZoomIn => "zoom-in",
            Icon::FitPage => "fit-page",
            Icon::FitWidth => "fit-width",
            // ui-text-exempt: icon key, never displayed
            Icon::FitHeight => "fit-height",
            Icon::RotateCcw => "rotate-ccw",
            Icon::RotateCw => "rotate-cw",
            Icon::Properties => "properties",
            Icon::Document => "document",
            Icon::Convert => "convert",
            Icon::Markup => "markup",
            Icon::Text => "text",
            Icon::EditText => "edit-text",
            Icon::AddText => "add-text",
            Icon::FormField => "form-field",
            Icon::EditObjects => "edit-objects",
            Icon::ShowPoints => "show-points",
            Icon::Bookmarks => "bookmarks",
            Icon::Layers => "layers",
            Icon::Signatures => "signatures",
            Icon::Fonts => "fonts",
            Icon::Measure => "measure",
            Icon::Undo => "undo",
            Icon::Redo => "redo",
            Icon::Copy => "copy",
            Icon::Tools => "tools",
            Icon::Keyboard => "keyboard",
            Icon::Info => "info",
            // ui-text-exempt: icon asset key, never displayed
            Icon::Pointer => "pointer",
            Icon::ShapeRect => "shape-rect",
            Icon::ShapeEllipse => "shape-ellipse",
            Icon::ShapeArrow => "shape-arrow",
            Icon::ShapePolyline => "shape-polyline",
            Icon::ShapePolygon => "shape-polygon",
            // ui-text-exempt: icon asset key, never displayed
            Icon::ShapeCloud => "shape-cloud",
            Icon::ShapeInk => "shape-ink",
            Icon::ShapeHighlight => "shape-highlight",
            Icon::TextSelect => "text-select",
            Icon::TextUnderline => "text-underline",
            Icon::TextStrikeout => "text-strikeout",
            Icon::TextSquiggly => "text-squiggly",
            Icon::TextFreeText => "text-freetext",
            Icon::TextSticky => "text-sticky",
            Icon::Stamp => "stamp",
            Icon::Combine => "combine",
            Icon::Split => "split",
            Icon::InsertPages => "insert-pages",
            Icon::ImportFormData => "import-form-data",
            Icon::FontFolders => "font-folders",
            Icon::Redact => "redact",
            Icon::Print => "print",
            Icon::Export => "export",
            Icon::Settings => "settings",
            Icon::InsertImage => "insert-image",
            Icon::SetScale => "set-scale",
            Icon::PageSingle => "page-single",
            Icon::PageContinuous => "page-continuous",
            Icon::PageFacing => "page-facing",
            Icon::PageFacingContinuous => "page-facing-continuous",
            Icon::ZoomRegion => "zoom-region",
            Icon::ZoomSelection => "zoom-selection",
            Icon::Cut => "cut",
            Icon::Paste => "paste",
            Icon::Cursor => "cursor",
            Icon::CursorNode => "cursor-node",
            Icon::Hand => "hand",
            Icon::Rulers => "rulers",
            Icon::Grid => "grid",
            Icon::Guides => "guides",
            Icon::Pages => "pages",
            Icon::Forms => "forms",
            Icon::ReadMode => "read-mode",
            Icon::Fullscreen => "fullscreen",
            Icon::FloatingPanels => "floating-panels",
            Icon::ResetLayout => "reset-layout",
            Icon::Delete => "delete",
            Icon::PageExtract => "page-extract",
            Icon::FormFlatten => "form-flatten",
            Icon::ManageList => "list",
            // ui-text-exempt: diagnostic/lookup keys, matched by ui-verify and
            // by `from_key`; never rendered.
            Icon::PickText => "pick-text",
            // ui-text-exempt: diagnostic/lookup key, never rendered.
            Icon::PickPath => "pick-path",
            // ui-text-exempt: diagnostic/lookup key, never rendered.
            Icon::PickPart => "pick-part",
            // ui-text-exempt: diagnostic/lookup key, never rendered.
            Icon::PickFormXObject => "pick-form-xobject",
            // ui-text-exempt: diagnostic/lookup key, never rendered.
            Icon::PickLink => "pick-link",
        }
    }

    /// Resolve an application icon key back to an [`Icon`].
    ///
    /// This is the lookup [`super::paint_ribbon_icon`] performs on every
    /// icon-bearing ribbon control, every frame.
    ///
    /// # Why a linear scan and not a `match` or a `HashMap`
    ///
    /// A reverse `match` would be a second copy of the key vocabulary, and
    /// two copies of a mapping is exactly how a rename lands in one of them.
    /// [`Icon::name`] stays the single source of truth and this walks it.
    ///
    /// The cost is one pointer-length comparison per catalogue entry
    /// ([`Icon::ALL`]`.len()`) with an early exit, for the
    /// handful of icons a ribbon draws per frame — comfortably under a
    /// microsecond, against a frame budget of 16 ms. A `HashMap` would need
    /// a lazily-initialised static, would hash the key anyway, and would buy
    /// nothing measurable. If the set ever reaches the hundreds, revisit;
    /// `every_name_round_trips_through_from_key` makes the swap safe.
    #[must_use]
    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|icon| icon.name() == key)
    }
}
