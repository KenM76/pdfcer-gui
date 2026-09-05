//! # `native-clipboard` — **one ordered clipboard transaction, or none of it**
//!
//! ## What this is for
//!
//! Putting *one* thing on the Windows clipboard is a solved problem and
//! `arboard` — which every `eframe` application already links — solves it.
//! This crate exists for the case that library cannot express at all:
//!
//! > **several formats, in a caller-chosen order, at least two of them under
//! > names that do not exist until they are registered at run time, one of them
//! > a GDI handle rather than a block of memory, and the whole set landing
//! > together or not at all.**
//!
//! Every clause of that sentence is load-bearing, and each one is why a
//! smaller answer was rejected:
//!
//! | clause | why the obvious answer fails |
//! |---|---|
//! | **several formats** | `arboard`'s API sets one payload and clears what was there; a second call erases the first, because `EmptyClipboard` is per-open |
//! | **in a caller-chosen order** | a pasting application "typically retrieves … the first format it recognizes", so the ORDER is the design and it must be the caller's to state |
//! | **registered names** | `arboard` has no registered-format API whatsoever. `RegisterClipboardFormatW` is the only route to a name like `"image/svg+xml"`, and such a format's numeric id does not exist until that call |
//! | **a GDI handle** | `CF_ENHMETAFILE` is an `HENHMETAFILE`, not an `HGLOBAL`. Handing the clipboard a byte block under that format id gives GDI the wrong handle type; the bytes must go through `SetEnhMetaFileBits` first |
//! | **together or not at all** | see [`place`]'s staging argument — a half-placed set is not a smaller success, it is a *different and silently wrong* paste |
//!
//! ## ★★★ It knows nothing about documents, and that is enforced by shape
//!
//! This crate takes **bytes** and **format names**. It does not know what an
//! SVG is, does not know why one entry should precede another, and has never
//! heard of a page. The rule that a raster placed with no vector in front of it
//! degrades a Microsoft Word paste to a flat picture is a fact about *Word* and
//! about the *caller's* intent — it lives with the caller, in
//! `crates/pdfcer-gui/src/clipboard.rs`, as a predicate on the payload.
//!
//! ⇒ The property that matters: nothing under `src/` names a PDF concept, so a
//! crate that must never learn what a document is — `egui-shell` — could take
//! this dependency unchanged. `crates/native-window`'s manifest calls that
//! being R7-clean, and it is the same claim for the same reason.
//!
//! ## ★★ Why `unsafe` is here and not at the call site
//!
//! `crates/pdfcer-gui/src/lib.rs` and `main.rs` both open with
//! `#![forbid(unsafe_code)]`. `forbid` cannot be relaxed by an inner `allow` —
//! that is precisely why it was chosen over `deny` — so an `unsafe` block in
//! the shell is not a lint to be quietened, it is a claim to be given up.
//!
//! `crates/native-window` already answered this question once, for four
//! `user32` calls that make a dialog owned by its parent window. This crate is
//! that answer applied one question further on, and it is deliberately built to
//! the same shape: no dependencies, hand-written `extern` declarations copied
//! from the SDK signatures, one `unsafe` block per call, and a `SAFETY` comment
//! on each one naming the invariant it upholds.
//!
//! ## The ownership rule that is the whole of the risk
//!
//! **`SetClipboardData` takes ownership of the handle it accepts.** After it
//! returns non-null the handle belongs to the clipboard; freeing it is a double
//! free. After it returns null the handle is still ours; *not* freeing it is a
//! leak. Both directions are live in [`place`] and both are covered by
//! [`win32::Staged`], which owns every handle it has created and frees it on
//! `Drop` **unless** the clipboard took it — so the correct thing happens on
//! the success path, the error path, and the panic path, without any of them
//! being written out separately.
//!
//! The clipboard itself is closed the same way, by [`win32::OpenGuard`]'s
//! `Drop`, so a panic between `OpenClipboard` and `CloseClipboard` cannot leave
//! the system-wide clipboard lock held by a process that has stopped running
//! the code that would release it.
//!
//! ## ⚠ Nothing in this crate is unit-tested against a real clipboard
//!
//! **Deliberately, and it must stay that way.** The clipboard is global state
//! on the operator's machine: a test that placed bytes would silently destroy
//! whatever they had copied, from a `cargo test` run they never connected to
//! their clipboard. The tests below assert the pure parts — the UTF-16
//! encoding of a registered name, the refusal of an empty transaction — and the
//! `unsafe` placement is verified **by construction and by review**, which is
//! stated plainly here rather than dressed up as coverage.
//!
//! ## Off Windows
//!
//! [`place`] answers [`PlaceError::Unsupported`] and does nothing. A stub
//! function rather than a `cfg` at every call site, matching how
//! `native_window::clipboard` is handled: a caller writes one line and reads
//! one `Result` on every platform.

#![cfg_attr(not(windows), allow(unused))]

#[cfg(windows)]
mod win32;

/// **`CF_DIBV5`** — a `BITMAPV5HEADER` followed by pixel data.
///
/// Exported so a caller can name the format without hard-coding `17`. It is a
/// Win32 fact rather than a document concept, so it does not breach this
/// crate's "no caller vocabulary" rule — the caller still decides *whether* a
/// DIB is worth placing and *what* is in it.
pub const CF_DIBV5: u32 = 17;

/// **`CF_ENHMETAFILE`** — the predefined id an [MS-EMF] metafile lands under.
///
/// Present for symmetry and for a caller that wants to name it in a
/// disclosure. It is **not** how a caller asks for a metafile to be placed:
/// that is [`Slot::EnhMetaFile`], because the handle conversion — not the
/// number — is what makes this format different from every other.
pub const CF_ENHMETAFILE: u32 = 14;

/// How one entry's bytes become a clipboard handle.
///
/// ★ Three variants rather than a bare `u32` format id, because the three
/// take genuinely different code paths through Win32 and a `u32` cannot say
/// which. A registered name has no id until run time; a metafile is not an
/// `HGLOBAL` at all. Collapsing them would push that decision into the caller,
/// which is the one place it must not be — the caller is the crate that
/// forbids `unsafe`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Slot {
    /// The format's name is registered at run time with
    /// `RegisterClipboardFormatW`, and [`Entry::name`] is the string handed to
    /// it **verbatim**.
    ///
    /// ⚠ Registration is case- and byte-sensitive. `"PNG"` is the name Office,
    /// Chromium, Firefox and every screenshot tool on the machine agree on;
    /// `"png"` registers a *different, private* format that nothing reads —
    /// and the placement succeeds, so nothing anywhere reports the mistake.
    Registered,
    /// A predefined `CF_*` id. The bytes are copied into moveable global
    /// memory and handed over as an `HGLOBAL`.
    Predefined(u32),
    /// `CF_ENHMETAFILE`. The bytes are [MS-EMF] and become a **GDI handle**
    /// through `SetEnhMetaFileBits` before `SetClipboardData` sees them.
    ///
    /// ★★ Its own variant rather than `Predefined(CF_ENHMETAFILE)`, because
    /// the difference is not the number — it is that the byte block must be
    /// converted into a different kind of handle first, and giving GDI an
    /// `HGLOBAL` under this id is undefined rather than refused.
    EnhMetaFile,
}

/// One thing to put on the clipboard.
///
/// Borrowed bytes rather than owned: every payload a caller has already built
/// lives somewhere for the duration of the call, and a `Vec` per entry would
/// copy a multi-megabyte raster twice — once into the `Vec` and once into the
/// `HGLOBAL` that is the only copy Windows keeps.
#[derive(Debug, Clone, Copy)]
pub struct Entry<'a> {
    /// The name Windows knows this format by, and — for [`Slot::Registered`] —
    /// the exact string passed to `RegisterClipboardFormatW`.
    ///
    /// `&'static str` rather than `&'a str` so that [`PlaceError`] can name the
    /// format that failed without allocating, which matters on a path whose
    /// commonest failure is *"another process is holding the clipboard"* and
    /// which may be retried.
    pub name: &'static str,
    /// How the bytes become a handle.
    pub slot: Slot,
    /// The payload, exactly as it should reach the clipboard. Nothing here
    /// reframes, terminates, pads or re-encodes it — a NUL terminator or a
    /// bitmap header is the caller's to add, because only the caller knows
    /// which framing the receiving applications were validated against.
    pub bytes: &'a [u8],
}

/// Why a placement did not happen.
///
/// ★ Every variant names the format where one is implicated, because the
/// operator-facing sentence a caller writes is different for *"another program
/// is holding the clipboard"* (try again) and *"the metafile was refused"*
/// (this document's vector form is the problem, the picture would still work).
/// An undifferentiated "clipboard error" collapses those into one useless line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlaceError {
    /// Nothing was asked for. Refused rather than treated as a success,
    /// because "I placed all zero of your entries" and "the clipboard now holds
    /// what you wanted" are different claims and a caller that cannot tell them
    /// apart will report the second.
    ///
    /// ★ It is refused **before** the clipboard is opened, so an empty
    /// transaction cannot destroy what the operator had copied.
    Nothing,
    /// A format name could not be registered. Not expected for any name in
    /// ordinary use — the registration table is large and process-independent —
    /// but reported rather than assumed.
    Register(&'static str),
    /// Global memory for a payload could not be allocated, or a metafile
    /// handle could not be created from the bytes. Names the format.
    ///
    /// ★★ `SetEnhMetaFileBits` returning null is the interesting member of
    /// this variant: it means Windows itself rejected the metafile's record
    /// structure. Because staging happens **before** the clipboard is opened
    /// (see [`place`]), that refusal costs the operator nothing — the clipboard
    /// still holds whatever it held.
    Stage(&'static str),
    /// The clipboard could not be opened. Windows serialises clipboard access
    /// across processes, so this is transient by nature: another application
    /// held it for longer than the retry budget.
    Open,
    /// `SetClipboardData` refused a format after the clipboard had been
    /// emptied. Entries before this one are on the clipboard and entries after
    /// it are not — the one failure mode this crate cannot make atomic, and the
    /// reason everything that *can* fail is done before the emptying.
    Set(&'static str),
    /// Not Windows. Constructed only by the non-Windows [`place`], so on
    /// Windows this variant is deliberately unreachable — the error type is the
    /// same on every target so a caller's `match` is too.
    Unsupported,
}

impl std::fmt::Display for PlaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // ui-text-exempt: developer-facing diagnostics. An operator-visible
        // sentence about a failed copy is the CALLER's to write, from its own
        // string catalogue, because only the caller knows what was being
        // copied and what the operator should do instead.
        match self {
            Self::Nothing => write!(f, "nothing to place on the clipboard"),
            Self::Register(name) => write!(f, "could not register clipboard format {name:?}"),
            Self::Stage(name) => write!(f, "could not stage a handle for {name:?}"),
            Self::Open => write!(f, "could not open the clipboard"),
            Self::Set(name) => write!(f, "could not place {name:?} on the clipboard"),
            Self::Unsupported => write!(f, "the clipboard is reachable on Windows only"),
        }
    }
}

impl std::error::Error for PlaceError {}

/// ★★★ **Place every entry, in the given order, in one transaction.**
///
/// Returns the names that landed, in placement order, so a caller can disclose
/// exactly what a pasting application will be offered.
///
/// # ★★★ The staging rule, which is what makes this all-or-nothing
///
/// The clipboard has one destructive step — `EmptyClipboard` — and everything
/// after it is visible to the rest of the machine. So **every operation that
/// can fail is performed before the clipboard is opened**:
///
/// 1. every [`Slot::Registered`] name is registered and its id resolved;
/// 2. every payload is copied into moveable global memory;
/// 3. every [`Slot::EnhMetaFile`] payload is handed to `SetEnhMetaFileBits`,
///    which validates the metafile's record structure and refuses a malformed
///    one.
///
/// Only when all three have succeeded for **all** entries does the clipboard
/// get opened and emptied. A malformed metafile, an allocation failure or an
/// unregisterable name therefore leaves the operator's clipboard exactly as it
/// was, rather than half-replaced.
///
/// ⇒ This is a deliberate departure from the worked example this shell's
/// payload was derived from (`crates/pdfcer-cli/src/clipboard.rs` in the engine
/// tree), which creates the metafile handle *inside* the open guard. Both are
/// correct with respect to Win32 — `SetEnhMetaFileBits` does not require an
/// open clipboard — but only this order can honestly claim the transaction is
/// atomic in the direction that matters. It is also the order
/// `native_window::clipboard` already argues for, in its own words:
/// *"holding it across two allocations is holding it longer than necessary,
/// and an allocation failure inside the open would mean deciding whether to
/// publish half a transaction."*
///
/// ★ It also discharges a check `pdfcer-gui`'s exporter explicitly logged as
/// owed. `app::actions::export`'s `emf_bytes` notes that a *file* export need
/// not validate its metafile but *"the clipboard path is the one that owes this
/// check, because there `SetEnhMetaFileBits` is handed a raw buffer and a bad
/// one is a GDI failure rather than a refusal."* Staging turns that GDI failure
/// into exactly a refusal, using the operating system's own validator, before
/// anything is destroyed.
///
/// # The one thing that is still not atomic, stated rather than hidden
///
/// `SetClipboardData` itself can fail, and there is no Win32 rollback for a
/// clipboard that has already been emptied and partly written. If it does,
/// [`PlaceError::Set`] names the format and the caller must assume the
/// clipboard holds a *prefix* of what was asked for. Nothing here pretends
/// otherwise; what the staging buys is that this is the only remaining way to
/// get there, and it is the one Win32 does not let anybody avoid.
///
/// # Errors
///
/// [`PlaceError`] — see the type. Every variant except [`PlaceError::Set`]
/// leaves the previous clipboard contents intact.
#[cfg(windows)]
pub fn place(entries: &[Entry<'_>]) -> Result<Vec<&'static str>, PlaceError> {
    win32::place(entries)
}

/// Off Windows: nothing is placed, and the caller is told so by name.
///
/// Answering an error rather than `Ok(vec![])` is the same choice
/// [`PlaceError::Nothing`] makes: a caller that cannot distinguish *"placed
/// nothing"* from *"placed everything"* will tell the operator the second.
#[cfg(not(windows))]
#[allow(clippy::missing_errors_doc, reason = "the one error is in the summary")]
pub fn place(_entries: &[Entry<'_>]) -> Result<Vec<&'static str>, PlaceError> {
    Err(PlaceError::Unsupported)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★★★ **An empty transaction is refused, and refusing is what keeps
    /// whatever the operator had copied.**
    ///
    /// The check happens before any Win32 call — which is the point, and is
    /// also why this test can exist at all. See the crate header: **no test
    /// here touches the real clipboard**, ever.
    #[test]
    fn an_empty_transaction_is_refused_before_anything_is_opened() {
        assert_eq!(place(&[]), Err(PlaceError::Nothing));
    }

    /// ★ The error type says which format failed, in words a caller can put in
    /// a trace.
    #[test]
    fn every_failure_names_the_format_where_one_is_implicated() {
        assert!(
            PlaceError::Register("image/svg+xml")
                .to_string()
                .contains("image/svg+xml")
        );
        assert!(
            PlaceError::Stage("CF_ENHMETAFILE")
                .to_string()
                .contains("CF_ENHMETAFILE")
        );
        assert!(PlaceError::Set("PNG").to_string().contains("PNG"));
    }
}
