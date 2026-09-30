//! `pdfcer-remote`: send one request to a running pdfcer-gui and print the
//! reply. Each run connects, says hello (the window may ask the operator), sends
//! the request, and disconnects.
//!
//! Exit status: 0 success, 1 the window answered with an error, 2 no window
//! reachable or bad arguments.

use std::io::{BufReader, Write};
use std::process::ExitCode;

use command_remote::protocol::{quote, read_reply};

const APP: &str = "pdfcer";
const USAGE: &str = "\
usage: pdfcer-remote list
       pdfcer-remote [--pid N] [--client NAME] [--purpose TEXT] <verb> [args...]
       pdfcer-remote help      (the window's own list of verbs)";

struct Options {
    pid: Option<u32>,
    client: String,
    purpose: String,
    request: Vec<String>,
}

fn parse(args: Vec<String>) -> Result<Options, String> {
    let mut options = Options {
        pid: None,
        client: "pdfcer-remote".to_owned(),
        purpose: String::new(),
        request: Vec::new(),
    };
    let mut it = args.into_iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--pid" => {
                let n = it.next().ok_or("--pid needs a number")?;
                options.pid = Some(n.parse().map_err(|_| format!("bad pid {n}"))?);
            }
            "--client" => options.client = it.next().ok_or("--client needs a name")?,
            "--purpose" => options.purpose = it.next().ok_or("--purpose needs text")?,
            _ => {
                options.request.push(arg);
                options.request.extend(it.by_ref());
            }
        }
    }
    if options.request.is_empty() {
        return Err(USAGE.to_owned());
    }
    Ok(options)
}

fn open(pipe: &str) -> std::io::Result<std::fs::File> {
    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(pipe)
}

fn main() -> ExitCode {
    let options = match parse(std::env::args().skip(1).collect()) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(2);
        }
    };
    let Some(dir) = command_remote::default_discovery_dir(APP) else {
        eprintln!("LOCALAPPDATA is not set");
        return ExitCode::from(2);
    };
    let instances: Vec<_> = command_remote::list_instances(&dir)
        .into_iter()
        .filter(|i| options.pid.is_none_or(|p| p == i.pid))
        .collect();

    if options.request == ["list"] {
        for i in &instances {
            println!("{}\t{}", i.pid, i.document);
        }
        return ExitCode::SUCCESS;
    }

    // Newest first; a file left by a killed window has no pipe behind it.
    let Some(mut pipe) = instances.iter().find_map(|i| open(&i.pipe).ok()) else {
        eprintln!(
            "no pdfcer-gui window is listening (is one open, and is Remote control not set to Never?)"
        );
        return ExitCode::from(2);
    };
    match exchange(&mut pipe, &options) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("connection failed: {e}");
            ExitCode::from(2)
        }
    }
}

fn exchange(pipe: &mut std::fs::File, options: &Options) -> std::io::Result<ExitCode> {
    let mut reader = BufReader::new(pipe.try_clone()?);
    writeln!(
        pipe,
        "hello {} {}",
        quote(&options.client),
        quote(&options.purpose)
    )?;
    let hello = read_reply(&mut reader)?;
    if let Some(code) = &hello.error {
        eprintln!("refused ({code}): {}", hello.message);
        return Ok(ExitCode::from(1));
    }
    let line: Vec<String> = options.request.iter().map(|a| quote(a)).collect();
    writeln!(pipe, "{}", line.join(" "))?;
    let reply = read_reply(&mut reader)?;
    let _ = writeln!(pipe, "bye");
    if let Some(code) = &reply.error {
        eprintln!("error {code}: {}", reply.message);
        return Ok(ExitCode::from(1));
    }
    if !reply.message.is_empty() {
        println!("{}", reply.message);
    }
    for line in &reply.body {
        println!("{line}");
    }
    Ok(ExitCode::SUCCESS)
}
