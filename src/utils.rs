//! Various utility functionality.
use std::io;
use std::fs;
use std::mem;
use std::env;
use std::time;
use std::path::{Path, PathBuf};
use std::io::{Read, Write, Seek};

use term;
use log;
use uuid::Uuid;
use sha1::Sha1;
use clap::{App, AppSettings};

use prelude::*;

#[cfg(not(windows))]
use chan_signal::{notify, Signal};

/// A simple logger
pub struct Logger;

impl log::Log for Logger {
    fn enabled(&self, _metadata: &log::LogMetadata) -> bool {
        true
    }
    fn log(&self, record: &log::LogRecord) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let mut term = term::stderr().unwrap();
        term.fg(match record.level() {
            log::LogLevel::Error | log::LogLevel::Warn => term::color::RED,
            log::LogLevel::Info => term::color::CYAN,
            log::LogLevel::Debug | log::LogLevel::Trace => term::color::YELLOW,
        }).ok();
        writeln!(term, "[{}] {} {}", record.level(),
                 record.target(), record.args()).ok();
        term.reset().ok();
    }
}

/// Helper for temporary file access
pub struct TempFile {
    f: Option<fs::File>,
    path: PathBuf,
}

impl TempFile {
    /// Creates a new tempfile.
    pub fn new() -> io::Result<TempFile> {
        let mut path = env::temp_dir();
        path.push(Uuid::new_v4().to_hyphenated_string());
        let f = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path).unwrap();
        Ok(TempFile {
            f: Some(f),
            path: path.to_path_buf(),
        })
    }

    /// Opens the tempfile
    pub fn open(&self) -> fs::File {
        let mut f = self.f.as_ref().unwrap().try_clone().unwrap();
        let _ = f.seek(io::SeekFrom::Start(0));
        f
    }

    /// Returns the path to the tempfile
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        mem::drop(self.f.take());
        let _ = fs::remove_file(&self.path);
    }
}

/// On non windows platforms this runs the function until it's
/// being interrupted by a signal.
#[cfg(not(windows))]
pub fn run_or_interrupt<F>(f: F) -> Option<Signal>
    where F: FnOnce() -> (), F: Send + 'static
{
    use chan;
    let run = |_sdone: chan::Sender<()>| f();
    let signal = notify(&[Signal::INT, Signal::TERM]);
    let (sdone, rdone) = chan::sync(0);
    ::std::thread::spawn(move || run(sdone));

    let mut rv = None;

    chan_select! {
        signal.recv() -> signal => { rv = signal; },
        rdone.recv() => {}
    }

    rv
}

/// Helper function to create a clap app for subcommands
pub fn make_subcommand<'a, 'b: 'a>(name: &str) -> App<'a, 'b> {
    App::new(name)
        .setting(AppSettings::UnifiedHelpMessage)
        .setting(AppSettings::DisableVersion)
}

/// Given a path returns the SHA1 checksum for it.
pub fn get_sha1_checksum(path: &Path) -> Result<String> {
    let mut sha = Sha1::new();
    let mut f = fs::File::open(path)?;
    let mut buf = [0u8; 16384];
    loop {
        let read = f.read(&mut buf)?;
        if read == 0 {
            break;
        }
        sha.update(&buf[..read]);
    }
    Ok(sha.digest().to_string())
}

/// Checks if a path is writable.
pub fn is_writable<P: AsRef<Path>>(path: P) -> bool {
    fs::OpenOptions::new().write(true).open(&path).map(|_| true).unwrap_or(false)
}

/// Set the mode of a path to 755 if we're on a Unix machine, otherwise
/// don't do anything with the given path.
pub fn set_executable_mode<P: AsRef<Path>>(path: P) -> io::Result<()> {
    #[cfg(not(windows))]
    fn exec<P: AsRef<Path>>(path: P) -> io::Result<()> {
        use std::os::unix::fs::PermissionsExt;
        let mut perm = fs::metadata(&path)?.permissions();
        perm.set_mode(0o755);
        fs::set_permissions(&path, perm)
    }

    #[cfg(windows)]
    fn exec<P: AsRef<Path>>(_path: P) -> io::Result<()> {
        Ok(())
    }

    exec(path)
}

/// Prints a message and loops until yes or no is entered.
pub fn prompt_to_continue(message: &str) -> io::Result<bool> {
    loop {
        print!("{} [y/n] ", message);
        io::stdout().flush()?;

        let mut buf = String::new();
        io::stdin().read_line(&mut buf)?;
        let input = buf.trim();

        if input == "y" {
            return Ok(true);
        } else if input == "n" {
            return Ok(false);
        }
        println!("invalid input!");
    }
}

/// Prompts for input and returns it.
pub fn prompt(message: &str) -> io::Result<String> {
    loop {
        print!("{}: ", message);
        io::stdout().flush()?;
        let mut buf = String::new();
        io::stdin().read_line(&mut buf)?;
        let input = buf.trim();
        if input.len() > 0 {
            return Ok(input.to_owned());
        }
    }
}

/// Given a system time returns the unix timestamp as f64
pub fn to_timestamp(tm: time::SystemTime) -> f64 {
    let duration = tm.duration_since(time::UNIX_EPOCH).unwrap();
    (duration.as_secs() as f64) + (duration.subsec_nanos() as f64 / 1e09)
}

/// Capitalizes a string and returns it.
pub fn capitalize_string(s: &str) -> String {
    use std::ascii::AsciiExt;
    let mut bytes = s.as_bytes().to_vec();
    bytes.make_ascii_lowercase();
    bytes[0] = bytes[0].to_ascii_uppercase();
    String::from_utf8(bytes).unwrap()
}
