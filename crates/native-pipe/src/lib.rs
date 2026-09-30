//! A named-pipe server only the current user can reach.
//!
//! [`PipeServer::accept`] blocks until one client connects and returns the
//! connection as a [`Connection`] (`Read + Write`). Each accept creates a fresh
//! pipe instance, so a server loop is `loop { let c = server.accept()?; … }`.
//!
//! **Who can connect.** Every instance carries a DACL granting the current
//! user's SID and nobody else, plus `PIPE_REJECT_REMOTE_CLIENTS`: another
//! machine, or another account on this one, is refused by the OS before a byte
//! is read. The first instance takes `FILE_FLAG_FIRST_PIPE_INSTANCE`, so a
//! name already owned by another process fails rather than being shared.
//!
//! **Clients need nothing from here.** A pipe opens as a file:
//! `std::fs::OpenOptions::new().read(true).write(true).open(r"\\.\pipe\name")`.
//!
//! The caller owns the protocol. Off Windows every call returns
//! `ErrorKind::Unsupported`.

use std::io;

#[cfg(windows)]
mod win32;

/// The prefix every local pipe path carries.
pub const PREFIX: &str = r"\\.\pipe\";

/// Whether a server currently holds the pipe `name` ([`PREFIX`] optional).
/// Asks without connecting, so the server sees nothing. Always `false` off
/// Windows.
#[must_use]
pub fn exists(name: &str) -> bool {
    #[cfg(windows)]
    {
        if name.starts_with(PREFIX) {
            win32::exists(name)
        } else {
            win32::exists(&format!("{PREFIX}{name}"))
        }
    }
    #[cfg(not(windows))]
    {
        let _ = name;
        false
    }
}

/// A server for one pipe name. Nothing is created until [`accept`](Self::accept).
#[derive(Debug)]
pub struct PipeServer {
    name: String,
    created_first: bool,
}

/// One connected client. Dropping it flushes, disconnects and closes the instance.
#[derive(Debug)]
pub struct Connection {
    #[cfg(windows)]
    inner: win32::Instance,
}

impl PipeServer {
    /// A server for `name`; [`PREFIX`] is added when absent.
    pub fn new(name: &str) -> Self {
        let name = if name.starts_with(PREFIX) {
            name.to_owned()
        } else {
            format!("{PREFIX}{name}")
        };
        Self {
            name,
            created_first: false,
        }
    }

    /// The full pipe path clients open.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Create a pipe instance and block until a client connects to it.
    ///
    /// # Errors
    /// The OS error when the instance cannot be created (the name is owned by
    /// another process, or the security descriptor cannot be built) or the
    /// connection fails. `Unsupported` off Windows.
    pub fn accept(&mut self) -> io::Result<Connection> {
        #[cfg(windows)]
        {
            let inner = win32::Instance::create(&self.name, !self.created_first)?;
            self.created_first = true;
            inner.connect()?;
            Ok(Connection { inner })
        }
        #[cfg(not(windows))]
        {
            Err(io::Error::from(io::ErrorKind::Unsupported))
        }
    }
}

impl io::Read for Connection {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        #[cfg(windows)]
        {
            self.inner.file().read(buf)
        }
        #[cfg(not(windows))]
        {
            let _ = buf;
            Err(io::Error::from(io::ErrorKind::Unsupported))
        }
    }
}

impl io::Write for Connection {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        #[cfg(windows)]
        {
            self.inner.file().write(buf)
        }
        #[cfg(not(windows))]
        {
            let _ = buf;
            Err(io::Error::from(io::ErrorKind::Unsupported))
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        #[cfg(windows)]
        {
            self.inner.file().flush()
        }
        #[cfg(not(windows))]
        {
            Ok(())
        }
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader, Write};
    use std::time::Duration;

    fn open_client(path: &str) -> std::fs::File {
        for _ in 0..300 {
            if let Ok(f) = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(path)
            {
                return f;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        panic!("no client connection to {path}");
    }

    #[test]
    fn a_client_round_trips_a_line() {
        let mut server = PipeServer::new(&format!("native-pipe-rt-{}", std::process::id()));
        let path = server.name().to_owned();
        let t = std::thread::spawn(move || {
            let mut conn = server.accept().expect("accept");
            let mut line = String::new();
            BufReader::new(&mut conn)
                .read_line(&mut line)
                .expect("read");
            conn.write_all(format!("echo {line}").as_bytes())
                .expect("write");
        });
        let mut client = open_client(&path);
        client.write_all(b"hello\n").expect("client write");
        let mut reply = String::new();
        BufReader::new(&client)
            .read_line(&mut reply)
            .expect("client read");
        assert_eq!(reply, "echo hello\n");
        t.join().expect("server thread");
    }

    #[test]
    fn exists_sees_a_live_server_and_not_a_missing_one() {
        let name = format!("native-pipe-exists-{}", std::process::id());
        assert!(!exists(&name), "no server yet");
        let mut server = PipeServer::new(&name);
        let path = server.name().to_owned();
        let t = std::thread::spawn(move || server.accept().map(|_| ()));
        let mut seen = false;
        for _ in 0..300 {
            if exists(&name) {
                seen = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(seen, "a listening server must be seen");
        drop(open_client(&path));
        t.join().expect("thread").expect("accept");
    }

    #[test]
    fn a_second_owner_of_a_live_name_is_refused() {
        let name = format!("native-pipe-own-{}", std::process::id());
        let mut first = PipeServer::new(&name);
        let path = first.name().to_owned();
        let t = std::thread::spawn(move || first.accept().map(|_| ()));
        // The first instance must exist before the second claim.
        std::thread::sleep(Duration::from_millis(150));
        assert!(
            PipeServer::new(&name).accept().is_err(),
            "a second first-instance must be refused"
        );
        drop(open_client(&path));
        t.join().expect("thread").expect("first accept");
    }
}
