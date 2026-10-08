//! The transport between pdfcer-gui windows: one discovery file and one pipe
//! per window.
//!
//! Contract:
//! * A window listens on the pipe `pdfcer-gui-window-<pid>` and names it in
//!   `<dir>/<pid>.txt`, where `<dir>` is `windows/` inside the settings
//!   directory. Windows of one installation therefore see each other, and a
//!   copy of the program in another folder — a test sandbox — sees nothing of
//!   them.
//! * A request is one line, `open\t<absolute path>\n`, answered `ok\n` or
//!   `no\t<reason>\n`. `ok` means the path was handed to the receiving
//!   window's frame; the open itself happens there and reports its own
//!   failures.
//! * The pipe admits the current user only (`native_pipe`'s DACL).

use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;

/// The pipe-name prefix; the process id follows.
const PIPE_PREFIX: &str = "pdfcer-gui-window-"; // ui-text-exempt: a pipe name, never displayed

/// The discovery directory's name inside the settings directory.
const DIR_NAME: &str = "windows"; // ui-text-exempt: a directory name, never displayed

/// The request verb.
const OPEN: &str = "open"; // ui-text-exempt: a protocol word, never displayed

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
pub fn publish(dir: &Path, pipe: &str, documents: &str) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    let pid = std::process::id();
    let staging = dir.join(format!("{pid}.tmp"));
    // One line per field; a document name cannot hold a newline.
    let body = format!("pipe={pipe}\ndocuments={documents}\n");
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
    for line in text.lines() {
        if let Some(v) = line.strip_prefix("pipe=") {
            pipe = Some(v.to_owned());
        } else if let Some(v) = line.strip_prefix("documents=") {
            v.clone_into(&mut documents);
        }
    }
    let pipe = pipe.filter(|p| p.starts_with(PIPE_PREFIX))?;
    Some(Peer {
        pid,
        pipe,
        documents,
    })
}

/// Listen on `pipe` on a thread of its own, handing every requested path to
/// `inbox` and calling `wake` so the frame that opens it runs.
pub fn serve(
    pipe: String,
    inbox: Sender<PathBuf>,
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
                    Some(path) => {
                        if inbox.send(path).is_err() {
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

/// The path an `open` request names, when the line is one.
fn request(connection: &mut impl Read) -> Option<PathBuf> {
    let mut line = String::new();
    BufReader::new(connection.take(MAX_LINE))
        .read_line(&mut line)
        .ok()?;
    let (verb, path) = line.trim_end_matches(['\r', '\n']).split_once('\t')?;
    let path = PathBuf::from(path);
    (verb == OPEN && path.is_absolute()).then_some(path)
}

/// Ask the window listening on `pipe` to open `path`. Blocks until it
/// answers, so callers run it off the frame thread.
pub fn send_open(pipe: &str, path: &Path) -> Result<(), String> {
    let text = path
        .to_str()
        .ok_or_else(|| crate::text::siblings::unsendable_name().to_owned())?;
    let mut stream = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(format!("{}{pipe}", native_pipe::PREFIX))
        .map_err(|e| e.to_string())?;
    stream
        .write_all(format!("{OPEN}\t{text}\n").as_bytes())
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
            "pipe=pdfcer-gui-window-7\ndocuments=a.pdf \u{b7} b.pdf\n",
        );
        assert_eq!(
            peer,
            Some(Peer {
                pid: 7,
                pipe: "pdfcer-gui-window-7".to_owned(),
                documents: "a.pdf \u{b7} b.pdf".to_owned(),
            })
        );
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
        let mut good = format!("open\t{absolute}\r\n");
        assert_eq!(request(&mut good.as_bytes()), Some(PathBuf::from(absolute)));
        good.clear();
        assert_eq!(request(&mut "open\trelative.pdf\n".as_bytes()), None);
        assert_eq!(
            request(&mut format!("close\t{absolute}\n").as_bytes()),
            None
        );
        assert_eq!(request(&mut "open\n".as_bytes()), None);
    }
}
