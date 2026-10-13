//! The transport between pdfcer-gui windows: one discovery file and one pipe
//! per window.
//!
//! Contract:
//! * A window listens on the pipe `pdfcer-gui-window-<pid>` and names it in
//!   `<dir>/<pid>.txt`, where `<dir>` is `windows/` inside the settings
//!   directory. Windows of one installation therefore see each other, and a
//!   copy of the program in another folder — a test sandbox — sees nothing of
//!   them.
//! * A request is one line,
//!   `open\t<absolute path>[\t<page index>|-[\t<name>=<value>]...]\n`,
//!   answered `ok\n` or `no\t<reason>\n`. The page is zero-based; a Windows
//!   file name cannot hold a tab, so the fields split unambiguously. The
//!   named fields are `carry=<absolute path>` (a copy holding unsaved edits,
//!   read in place of the file and deleted by the receiver), `edits=<n>` (how
//!   many it holds) and `key=<hex>` (the password the document was opened
//!   with). `ok` is answered only once the receiving frame has the document
//!   open; a refusal, or no answer within [`ANSWER_WAIT`], leaves the
//!   sender's copy where it is.
//! * `paste\t<x>\t<y>\n` asks the window to paste the clip on the clipboard
//!   at the desktop pixel `(x, y)`, answered the same way.
//! * The pipe admits the current user only (`native_pipe`'s DACL), which is
//!   what makes it a place a password may travel. A key is never traced.
//! * The discovery file also carries the window's client area in desktop
//!   pixels, `rect=<left>,<top>,<right>,<bottom>`, so a tab dropped on another
//!   window can be matched to it.

use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Sender, SyncSender, sync_channel};
use std::time::Duration;

use crate::secret::Secret;

/// The pipe-name prefix; the process id follows.
const PIPE_PREFIX: &str = "pdfcer-gui-window-"; // ui-text-exempt: a pipe name, never displayed

/// The discovery directory's name inside the settings directory.
const DIR_NAME: &str = "windows"; // ui-text-exempt: a directory name, never displayed

/// The request verb that opens a file.
const OPEN: &str = "open"; // ui-text-exempt: a protocol word, never displayed

/// The request verb that pastes the clipboard at a desktop point.
const PASTE: &str = "paste"; // ui-text-exempt: a protocol word, never displayed

/// The positive reply.
const OK: &str = "ok"; // ui-text-exempt: a protocol word, never displayed

/// The refusal reply's verb.
const NO: &str = "no"; // ui-text-exempt: a protocol word, never displayed

/// How long an open waits for the receiving frame. A large drawing takes
/// seconds to load; a window that has not answered by then is treated as
/// having refused, so the sender keeps its copy.
const ANSWER_WAIT: Duration = Duration::from_secs(30);

/// The longest request line accepted. A path is far shorter; this bounds what
/// a misbehaving client can make the server hold.
const MAX_LINE: u64 = 64 * 1024;

/// A leftover discovery file younger than this is never pruned, so a window
/// between writing its file and creating its first pipe instance is not
/// mistaken for a dead one.
const PRUNE_AGE: std::time::Duration = std::time::Duration::from_secs(30);

/// Another window of this installation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Peer {
    /// Its process id.
    pub pid: u32,
    /// Its pipe name, without the `\\.\pipe\` prefix.
    pub pipe: String,
    /// The file names of its open documents, as it published them.
    pub documents: String,
    /// Its client area in desktop pixels, `[left, top, right, bottom]`, when
    /// it published one.
    pub rect: Option<[i32; 4]>,
}

impl Peer {
    /// Whether the desktop pixel `(x, y)` lies in this window's client area.
    #[must_use]
    pub fn contains(&self, x: f32, y: f32) -> bool {
        self.rect.is_some_and(|[l, t, r, b]| {
            x >= l as f32 && x < r as f32 && y >= t as f32 && y < b as f32
        })
    }
}

/// A document another window sent: its path, and the page it was showing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Arrival {
    /// The file, absolute.
    pub path: PathBuf,
    /// The zero-based page index to show, when the sender named one.
    pub page: Option<usize>,
    /// The copy holding unsaved edits, when the document had any.
    pub carry: Option<Carry>,
    /// The password the sender opened the document with.
    pub key: Option<Secret>,
}

/// A written copy of a document with unsaved edits, read in place of
/// [`Arrival::path`] and bound to it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Carry {
    /// The copy, absolute; the receiver deletes it once read.
    pub file: PathBuf,
    /// How many unsaved edits it holds, for the receiver's unsaved count.
    pub edits: u64,
}

/// A request, and for an open the channel the frame answers on.
#[derive(Debug)]
pub struct Incoming {
    /// What was asked.
    pub request: Request,
    /// Where an open's outcome goes; `None` for a paste, answered at once.
    pub answer: Option<SyncSender<Result<(), String>>>,
}

/// One request another window sent.
#[derive(Clone, Debug, PartialEq)]
pub enum Request {
    /// Open a document.
    Open(Arrival),
    /// Paste the clipboard at this desktop pixel.
    Paste {
        /// Desktop pixels from the left.
        x: f32,
        /// Desktop pixels from the top.
        y: f32,
    },
}

/// This window's pipe name.
#[must_use]
pub fn own_pipe() -> String {
    pipe_of(std::process::id())
}

/// The pipe the window with process id `pid` listens on.
#[must_use]
pub fn pipe_of(pid: u32) -> String {
    format!("{PIPE_PREFIX}{pid}")
}

/// The discovery directory, or `None` when there is no writable settings
/// directory.
#[must_use]
pub fn discovery_dir() -> Option<PathBuf> {
    pdfcer_core::settings::resolve_store()
        .directory()
        .map(|d| d.join(DIR_NAME))
}

fn file_of(dir: &Path, pid: u32) -> PathBuf {
    dir.join(format!("{pid}.txt"))
}

/// Write this window's discovery file. Written whole and renamed into place,
/// so a reader never sees half of one.
pub fn publish(
    dir: &Path,
    pipe: &str,
    documents: &str,
    rect: Option<[i32; 4]>,
) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    let pid = std::process::id();
    let staging = dir.join(format!("{pid}.tmp"));
    // One line per field; a document name cannot hold a newline.
    let mut body = format!("pipe={pipe}\ndocuments={documents}\n");
    if let Some([l, t, r, b]) = rect {
        body.push_str(&format!("rect={l},{t},{r},{b}\n"));
    }
    std::fs::write(&staging, body)?;
    std::fs::rename(&staging, file_of(dir, pid))
}

/// Remove this window's discovery file.
pub fn withdraw(dir: &Path) {
    let _ = std::fs::remove_file(file_of(dir, std::process::id()));
}

/// Every other window whose pipe is up, in process-id order.
#[must_use]
pub fn peers(dir: &Path) -> Vec<Peer> {
    let own = std::process::id();
    let mut found: Vec<Peer> = listed(dir)
        .filter(|(pid, _, _)| *pid != own)
        .filter_map(|(pid, _, text)| parse(pid, &text))
        .filter(|p| native_pipe::exists(&p.pipe))
        .collect();
    found.sort_by_key(|p| p.pid);
    found
}

/// Remove discovery files whose pipe is gone and which are old enough that
/// their window cannot still be starting. A window killed rather than closed
/// leaves one behind.
pub fn prune(dir: &Path) {
    let own = std::process::id();
    for (pid, path, text) in listed(dir) {
        if pid == own {
            continue;
        }
        let old = std::fs::metadata(&path)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.elapsed().ok())
            .is_some_and(|age| age > PRUNE_AGE);
        let dead = parse(pid, &text).is_none_or(|p| !native_pipe::exists(&p.pipe));
        if old && dead {
            let _ = std::fs::remove_file(path);
        }
    }
}

/// `(pid, path, contents)` for every `<pid>.txt` in `dir`.
fn listed(dir: &Path) -> impl Iterator<Item = (u32, PathBuf, String)> {
    std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(std::result::Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            let pid = path
                .file_name()?
                .to_str()?
                .strip_suffix(".txt")?
                .parse::<u32>()
                .ok()?;
            let text = std::fs::read_to_string(&path).ok()?;
            Some((pid, path, text))
        })
}

fn parse(pid: u32, text: &str) -> Option<Peer> {
    let mut pipe = None;
    let mut documents = String::new();
    let mut rect = None;
    for line in text.lines() {
        if let Some(v) = line.strip_prefix("pipe=") {
            pipe = Some(v.to_owned());
        } else if let Some(v) = line.strip_prefix("documents=") {
            v.clone_into(&mut documents);
        } else if let Some(v) = line.strip_prefix("rect=") {
            rect = parse_rect(v);
        }
    }
    let pipe = pipe.filter(|p| p.starts_with(PIPE_PREFIX))?;
    Some(Peer {
        pid,
        pipe,
        documents,
        rect,
    })
}

fn parse_rect(v: &str) -> Option<[i32; 4]> {
    let n: Vec<i32> = v.split(',').filter_map(|s| s.trim().parse().ok()).collect();
    <[i32; 4]>::try_from(n).ok()
}

/// Listen on `pipe` on a thread of its own, handing every request to `inbox`
/// and calling `wake` so the frame that acts on it runs.
pub fn serve(
    pipe: String,
    inbox: Sender<Incoming>,
    wake: impl Fn() + Send + 'static,
) -> std::io::Result<()> {
    std::thread::Builder::new()
        .name("sibling-windows".to_owned())
        .spawn(move || {
            let mut server = native_pipe::PipeServer::new(&pipe);
            loop {
                let mut connection = match server.accept() {
                    Ok(c) => c,
                    Err(e) => {
                        // ui-text-exempt: diagnostic trace, never displayed
                        crate::diag::trace(|| format!("window-pipe-stopped error={e:?}"));
                        return;
                    }
                };
                let reply = match request(&mut connection) {
                    Some(request) => {
                        let opens = matches!(request, Request::Open(_));
                        let (tx, rx) = sync_channel(1);
                        let answer = opens.then_some(tx);
                        if inbox.send(Incoming { request, answer }).is_err() {
                            return;
                        }
                        wake();
                        match opens.then(|| rx.recv_timeout(ANSWER_WAIT)) {
                            None | Some(Ok(Ok(()))) => format!("{OK}\n"),
                            Some(Ok(Err(why))) => format!("{NO}\t{why}\n"),
                            Some(Err(_)) => {
                                format!("{NO}\t{}\n", crate::text::siblings::no_answer())
                            }
                        }
                    }
                    None => format!("{NO}\t{}\n", crate::text::siblings::unreadable_request()),
                };
                let _ = connection.write_all(reply.as_bytes());
            }
        })
        .map(|_| ())
}

/// The request on the line, when it is one. A page or coordinate field that
/// is not a number refuses the whole request rather than dropping the field.
fn request(connection: &mut impl Read) -> Option<Request> {
    let mut line = String::new();
    BufReader::new(connection.take(MAX_LINE))
        .read_line(&mut line)
        .ok()?;
    let mut fields = line.trim_end_matches(['\r', '\n']).split('\t');
    let request = match fields.next()? {
        OPEN => {
            let path = PathBuf::from(fields.next()?);
            let page = match fields.next() {
                Some("-") | None => None,
                Some(p) => Some(p.parse().ok()?),
            };
            let (mut carry, mut edits, mut key) = (None, 0, None);
            for field in fields.by_ref() {
                match field.split_once('=')? {
                    ("carry", v) => carry = Some(PathBuf::from(v)).filter(|p| p.is_absolute()),
                    ("edits", v) => edits = v.parse().ok()?,
                    ("key", v) => key = Some(Secret::new(unhex(v)?)),
                    _ => return None,
                }
                if field.starts_with("carry=") && carry.is_none() {
                    return None;
                }
            }
            let carry = carry.map(|file| Carry { file, edits });
            path.is_absolute().then_some(Request::Open(Arrival {
                path,
                page,
                carry,
                key,
            }))?
        }
        PASTE => {
            let mut coordinate = || fields.next()?.parse::<f32>().ok().filter(|v| v.is_finite());
            let (x, y) = (coordinate()?, coordinate()?);
            Request::Paste { x, y }
        }
        _ => return None,
    };
    fields.next().is_none().then_some(request)
}

/// Ask the window listening on `pipe` to open `arrival`. Blocks until it
/// answers, so callers run it off the frame thread.
pub fn send_open(pipe: &str, arrival: &Arrival) -> Result<(), String> {
    exchange(pipe, &open_line(arrival)?)
}

/// The request line for `arrival`.
fn open_line(arrival: &Arrival) -> Result<String, String> {
    let name = |p: &Path| {
        p.to_str()
            .map(str::to_owned)
            .ok_or_else(|| crate::text::siblings::unsendable_name().to_owned())
    };
    let mut line = format!("{OPEN}\t{}", name(&arrival.path)?);
    match arrival.page {
        Some(p) => line.push_str(&format!("\t{p}")),
        None if arrival.carry.is_some() || arrival.key.is_some() => line.push_str("\t-"),
        None => {}
    }
    if let Some(carry) = &arrival.carry {
        line.push_str(&format!(
            "\tcarry={}\tedits={}",
            name(&carry.file)?,
            carry.edits
        ));
    }
    if let Some(key) = &arrival.key {
        line.push_str(&format!("\tkey={}", hex(key.expose())));
    }
    line.push('\n');
    Ok(line)
}

/// `bytes` as lower-case hexadecimal, so a password holding a tab or a
/// newline cannot split the request line.
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// The text [`hex`] encoded, or `None` when `text` is not such an encoding.
fn unhex(text: &str) -> Option<String> {
    if !text.len().is_multiple_of(2) {
        return None;
    }
    let bytes = (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(text.get(i..i + 2)?, 16).ok())
        .collect::<Option<Vec<u8>>>()?;
    String::from_utf8(bytes).ok()
}

/// Ask the window listening on `pipe` to paste its clipboard at the desktop
/// pixel `(x, y)`. Blocks until it answers.
pub fn send_paste(pipe: &str, x: f32, y: f32) -> Result<(), String> {
    exchange(pipe, &format!("{PASTE}\t{x}\t{y}\n"))
}

/// Send one request line and read the answer.
fn exchange(pipe: &str, line: &str) -> Result<(), String> {
    let mut stream = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(format!("{}{pipe}", native_pipe::PREFIX))
        .map_err(|e| e.to_string())?;
    stream
        .write_all(line.as_bytes())
        .map_err(|e| e.to_string())?;
    let mut reply = String::new();
    BufReader::new((&mut stream).take(MAX_LINE))
        .read_line(&mut reply)
        .map_err(|e| e.to_string())?;
    let reply = reply.trim_end_matches(['\r', '\n']);
    if reply == OK {
        Ok(())
    } else {
        Err(reply
            .strip_prefix(NO)
            .map_or(reply, |r| r.trim_start_matches('\t'))
            .to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_discovery_file_parses_back_to_its_peer() {
        let peer = parse(
            7,
            "pipe=pdfcer-gui-window-7\ndocuments=a.pdf \u{b7} b.pdf\nrect=-10,20,1390,920\n",
        );
        assert_eq!(
            peer,
            Some(Peer {
                pid: 7,
                pipe: "pdfcer-gui-window-7".to_owned(),
                documents: "a.pdf \u{b7} b.pdf".to_owned(),
                rect: Some([-10, 20, 1390, 920]),
            })
        );
        let peer = peer.unwrap();
        assert!(peer.contains(-10.0, 20.0));
        assert!(!peer.contains(1390.0, 500.0));
    }

    /// A file naming any other pipe is not a window of this program, and a
    /// request to it would go somewhere else entirely.
    #[test]
    fn a_file_naming_a_foreign_pipe_is_no_peer() {
        assert_eq!(parse(7, "pipe=pdfcer-remote-7\ndocuments=\n"), None);
        assert_eq!(parse(7, "documents=a.pdf\n"), None);
    }

    #[test]
    fn only_an_open_of_an_absolute_path_is_a_request() {
        let absolute = if cfg!(windows) {
            "C:\\a b.pdf"
        } else {
            "/a b.pdf"
        };
        let arrival = |page| {
            Some(Request::Open(Arrival {
                path: PathBuf::from(absolute),
                page,
                carry: None,
                key: None,
            }))
        };
        let good = format!("open\t{absolute}\r\n");
        assert_eq!(request(&mut good.as_bytes()), arrival(None));
        let paged = format!("open\t{absolute}\t3\n");
        assert_eq!(request(&mut paged.as_bytes()), arrival(Some(3)));
        let bad_page = format!("open\t{absolute}\tthree\n");
        assert_eq!(request(&mut bad_page.as_bytes()), None);
        assert_eq!(request(&mut "open\trelative.pdf\n".as_bytes()), None);
        assert_eq!(
            request(&mut format!("close\t{absolute}\n").as_bytes()),
            None
        );
        assert_eq!(request(&mut "open\n".as_bytes()), None);
    }

    /// What a carried open sends is what the receiver reads back, including
    /// a password holding the line's own separators; a relative carry, an
    /// unknown field and a malformed key each refuse the request.
    #[test]
    fn a_carried_open_round_trips() {
        let root = if cfg!(windows) { "C:\\" } else { "/" };
        let sent = Arrival {
            path: PathBuf::from(format!("{root}a b.pdf")),
            page: None,
            carry: Some(Carry {
                file: PathBuf::from(format!("{root}carry 1.pdf")),
                edits: 4,
            }),
            key: Some(Secret::new("p\tw\n\u{e9}".to_owned())),
        };
        let line = open_line(&sent).unwrap();
        assert!(!line.contains("p\tw"), "the key travels encoded");
        assert_eq!(request(&mut line.as_bytes()), Some(Request::Open(sent)));
        let relative = line.replace(&format!("carry={root}"), "carry=");
        assert_eq!(request(&mut relative.as_bytes()), None);
        let unknown = line.replace("edits=", "colour=");
        assert_eq!(request(&mut unknown.as_bytes()), None);
        let odd = line.replace("\tkey=", "\tkey=0");
        assert_eq!(request(&mut odd.as_bytes()), None);
    }

    #[test]
    fn a_paste_names_a_finite_desktop_point() {
        assert_eq!(
            request(&mut "paste\t-3492.5\t-2319\n".as_bytes()),
            Some(Request::Paste {
                x: -3492.5,
                y: -2319.0
            })
        );
        assert_eq!(request(&mut "paste\t1\n".as_bytes()), None);
        assert_eq!(request(&mut "paste\t1\tNaN\n".as_bytes()), None);
        assert_eq!(request(&mut "paste\t1\t2\t3\n".as_bytes()), None);
    }
}
