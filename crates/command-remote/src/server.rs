//! The pipe thread: accepts one client at a time and turns each line it sends
//! into an [`Event`] for the UI thread, blocking until the UI thread answers.
//!
//! The thread never touches application state. Everything it learns crosses
//! the channel; everything it says comes back over a per-request reply
//! channel. A client that stops reading cannot stall the UI thread, only this
//! one.

use std::io::{BufRead, BufReader, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, channel};
use std::time::Duration;

use crate::consent::{ASK_TIMEOUT, Scope};
use crate::protocol::{self, Line, Reply};

/// How long a request may wait for the UI thread before the client is told so.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(300);

pub(crate) type Wake = Arc<dyn Fn() + Send + Sync>;

/// What the pipe thread tells the UI thread.
pub(crate) enum Event {
    Connected {
        id: u64,
    },
    Hello {
        id: u64,
        client: String,
        purpose: String,
        answer: Sender<Result<Scope, &'static str>>,
    },
    Request {
        id: u64,
        line: Line,
        reply: Sender<Reply>,
    },
    Closed {
        id: u64,
    },
    /// The pipe could not be created; the thread has ended.
    Failed(String),
}

/// Start the thread. `revoked` holds the id of the last connection the
/// operator disconnected; that connection's next request ends it.
pub(crate) fn spawn(
    mut server: native_pipe::PipeServer,
    tx: Sender<Event>,
    wake: Wake,
    revoked: Arc<AtomicU64>,
) -> std::io::Result<()> {
    std::thread::Builder::new()
        .name("command-remote".to_owned())
        .spawn(move || {
            let mut id = 0u64;
            loop {
                let conn = match server.accept() {
                    Ok(conn) => conn,
                    Err(e) => {
                        let _ = tx.send(Event::Failed(e.to_string()));
                        wake();
                        return;
                    }
                };
                id += 1;
                if tx.send(Event::Connected { id }).is_err() {
                    return;
                }
                wake();
                serve(conn, id, &tx, &wake, &revoked);
                if tx.send(Event::Closed { id }).is_err() {
                    return;
                }
                wake();
            }
        })
        .map(|_| ())
}

fn serve(
    conn: native_pipe::Connection,
    id: u64,
    tx: &Sender<Event>,
    wake: &Wake,
    revoked: &AtomicU64,
) {
    let mut reader = BufReader::new(conn);
    let mut granted = false;
    loop {
        let mut raw = String::new();
        match reader.read_line(&mut raw) {
            Ok(0) | Err(_) => return,
            Ok(_) => {}
        }
        let (reply, end) = if revoked.load(Ordering::Acquire) >= id {
            (
                Reply::err("disconnected", "the operator ended this connection"),
                true,
            )
        } else {
            match protocol::parse(&raw) {
                None => (
                    Reply::err("bad-request", "empty line or unclosed quote"),
                    false,
                ),
                Some(line) if line.verb == "bye" => (Reply::ok("bye"), true),
                Some(line) if line.verb == "hello" => {
                    let reply = hello(line, id, granted, tx, wake);
                    granted |= reply.is_ok();
                    (reply, false)
                }
                Some(_) if !granted => (
                    Reply::err("not-enabled", "send: hello <client-name> <purpose> first"),
                    false,
                ),
                Some(line) => (request(line, id, tx, wake), false),
            }
        };
        let conn = reader.get_mut();
        if conn.write_all(reply.encode().as_bytes()).is_err() || conn.flush().is_err() || end {
            return;
        }
    }
}

fn hello(line: Line, id: u64, granted: bool, tx: &Sender<Event>, wake: &Wake) -> Reply {
    if granted {
        return Reply::ok("already allowed");
    }
    let mut args = line.args.into_iter();
    let Some(client) = args.next().filter(|c| !c.is_empty()) else {
        return Reply::err("bad-request", "hello needs a client name");
    };
    let purpose = args.collect::<Vec<_>>().join(" ");
    let (answer, rx) = channel();
    if tx
        .send(Event::Hello {
            id,
            client,
            purpose,
            answer,
        })
        .is_err()
    {
        return Reply::err("closing", "the application is closing");
    }
    wake();
    match wait(&rx, ASK_TIMEOUT + Duration::from_secs(5)) {
        Ok(Ok(scope)) => Reply::ok(match scope {
            Scope::Once => "allowed once",
            Scope::Session => "allowed for this session",
            Scope::Always => "allowed always",
        }),
        Ok(Err(reason)) => Reply::err("refused", reason),
        Err(code) => Reply::err("refused", code),
    }
}

fn request(line: Line, id: u64, tx: &Sender<Event>, wake: &Wake) -> Reply {
    let (reply, rx) = channel();
    if tx.send(Event::Request { id, line, reply }).is_err() {
        return Reply::err("closing", "the application is closing");
    }
    wake();
    match wait(&rx, REQUEST_TIMEOUT) {
        Ok(reply) => reply,
        Err(code) => Reply::err(code, "the application did not answer"),
    }
}

fn wait<T>(rx: &Receiver<T>, timeout: Duration) -> Result<T, &'static str> {
    rx.recv_timeout(timeout).map_err(|e| match e {
        RecvTimeoutError::Timeout => "timeout",
        RecvTimeoutError::Disconnected => "dropped",
    })
}
