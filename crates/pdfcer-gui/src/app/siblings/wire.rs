//! The transport between pdfcer-gui windows: one discovery file and one pipe
//! per window.
//!
//! Contract:
//! * A window listens on the pipe `pdfcer-gui-window-<pid>` and names it in
//!   `<dir>/<pid>.txt`, where `<dir>` is `windows/` inside the settings
//!   directory. Windows of one installation therefore see each other, and a
//!   copy of the program in another folder — a test sandbox — sees nothing of
//!   them.
//! * A request is one line, `open\t<absolute path>[\t<page index>]\n`, answered
//!   `ok\n` or `no\t<reason>\n`. The page is zero-based; a Windows file name
//!   cannot hold a tab, so the fields split unambiguously. `ok` means the path was handed to the receiving
//!   window's frame; the open itself happens there and reports its own
//!   failures.
//! * `paste\t<x>\t<y>\n` asks the window to paste the clip on the clipboard
//!   at the desktop pixel `(x, y)`, answered the same way.
//! * The pipe admits the current user only (`native_pipe`'s DACL).
//! * The discovery file also carries the window's client area in desktop
//!   pixels, `rect=<left>,<top>,<right>,<bottom>`, so a tab dropped on another
//!   window can be matched to it.

use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;

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
    format!("{PIPE_PREFIX}{}", std::process::id())
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
    inbox: Sender<Request>,
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
                    Some(arrival) => {
                        if inbox.send(arrival).is_err() {
                            return;
                        }
                        wake();
                        format!("{OK}\n")
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
                Some(p) => Some(p.parse().ok()?),
                None => None,
            };
            path.is_absolute()
                .then_some(Request::Open(Arrival { path, page }))?
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

/// Ask the window listening on `pipe` to open `path` at `page`. Blocks until
/// it answers, so callers run it off the frame thread.
pub fn send_open(pipe: &str, path: &Path, page: Option<usize>) -> Result<(), String> {
    let text = path
        .to_str()
        .ok_or_else(|| crate::text::siblings::unsendable_name().to_owned())?;
    exchange(
        pipe,
        &match page {
            Some(p) => format!("{OPEN}\t{text}\t{p}\n"),
            None => format!("{OPEN}\t{text}\n"),
        },
    )
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
