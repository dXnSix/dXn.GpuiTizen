//! The report transport. The app listens on the TV's loopback interface (default port 47123,
//! reached from the Mac through `sdb forward tcp:<p> tcp:<p>`) on a background thread, never the
//! main loop, and serves one client at a time newline-delimited JSON: the report, then its log
//! lines. A client that connects before the report is complete waits for it, and gets the latest
//! partial report (marked incomplete) only when the wait times out. The client half is used by
//! `cargo xtask diag pull`. Platform-neutral; `transport::tests` runs both over loopback.

use std::io::{self, BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use std::sync::{Arc, Condvar, Mutex, PoisonError};
use std::thread::JoinHandle;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::report::Report;

/// The default port; an `app_control` extra `port` overrides it.
pub const DEFAULT_PORT: u16 = 47123;

/// How long the server waits for the report before closing a client's connection.
pub const DEFAULT_REPORT_WAIT: Duration = Duration::from_secs(120);

/// How long a write to a client may block before the server gives up on that client.
pub const WRITE_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Default)]
struct State {
    report: Option<String>,
    complete: bool,
    log: Vec<String>,
}

/// The report and log lines shared between the app and the server thread.
#[derive(Clone, Default)]
pub struct Shared(Arc<(Mutex<State>, Condvar)>);

impl Shared {
    pub fn new() -> Self {
        Self::default()
    }

    /// Publishes (or replaces) the report; a complete report wakes waiting clients.
    pub fn set_report(&self, report: &Report) {
        let (state, ready) = &*self.0;
        let mut state = state.lock().unwrap_or_else(PoisonError::into_inner);
        state.report = Some(report.to_json());
        state.complete = report.complete;
        drop(state);
        ready.notify_all();
    }

    /// Appends a log line (newlines become spaces, so each line stays one line).
    pub fn log(&self, line: &str) {
        let (state, _) = &*self.0;
        state
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .log
            .push(line.replace(['\n', '\r'], " "));
    }

    /// Waits up to `timeout` for the complete report; returns it (or, on timeout, the latest
    /// partial one) with the log lines so far.
    fn wait(&self, timeout: Duration) -> Option<(String, Vec<String>)> {
        let (state, ready) = &*self.0;
        let guard = state.lock().unwrap_or_else(PoisonError::into_inner);
        let (guard, _) = ready
            .wait_timeout_while(guard, timeout, |s| !s.complete)
            .unwrap_or_else(PoisonError::into_inner);
        guard
            .report
            .clone()
            .map(|report| (report, guard.log.clone()))
    }
}

/// Serves clients from `listener` on a new thread until the process ends. Write errors (a client
/// that dropped the connection) end that client only.
pub fn serve(
    listener: TcpListener,
    shared: Shared,
    report_wait: Duration,
) -> io::Result<JoinHandle<()>> {
    std::thread::Builder::new()
        .name("diag-transport".into())
        .spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                let _ = stream.set_write_timeout(Some(WRITE_TIMEOUT));
                if let Some((report, log)) = shared.wait(report_wait) {
                    let _ = send(&mut stream, &report, &log);
                }
            }
        })
}

fn send(stream: &mut TcpStream, report: &str, log: &[String]) -> io::Result<()> {
    let mut out = io::BufWriter::new(stream);
    writeln!(out, "{report}")?;
    for line in log {
        writeln!(out, "{line}")?;
    }
    out.flush()
}

/// Binds `127.0.0.1:<port>` (only `sdb forward` reaches it, not the TV's network) and starts the
/// server.
pub fn start(port: u16, shared: Shared) -> io::Result<JoinHandle<()>> {
    serve(
        TcpListener::bind(("127.0.0.1", port))?,
        shared,
        DEFAULT_REPORT_WAIT,
    )
}

/// `sdb forward tcp:<p> tcp:<p>`'s arguments, which `cargo xtask diag pull` runs first.
pub fn sdb_forward_args(port: u16) -> [String; 3] {
    [
        "forward".to_owned(),
        format!("tcp:{port}"),
        format!("tcp:{port}"),
    ]
}

/// The `port` extra of the launch request, else [`DEFAULT_PORT`].
pub fn port_from_extra(extra: Option<&str>) -> u16 {
    extra
        .and_then(|value| value.trim().parse::<u16>().ok())
        .filter(|port| *port != 0)
        .unwrap_or(DEFAULT_PORT)
}

/// What a client received.
#[derive(Debug)]
pub struct Pulled {
    /// The report line as sent.
    pub json: String,
    pub report: Report,
    pub log: Vec<String>,
}

/// Connects, reads the report and the log lines until the server closes the connection.
pub fn pull(addr: impl ToSocketAddrs, timeout: Duration) -> io::Result<Pulled> {
    let addr: SocketAddr = addr
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "no address"))?;
    let stream = TcpStream::connect_timeout(&addr, timeout)?;
    stream.set_read_timeout(Some(timeout))?;
    let mut lines = BufReader::new(stream).lines();
    let json = match lines.next() {
        Some(line) => line?,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "the app closed the connection without a report",
            ));
        }
    };
    let report =
        Report::from_json(&json).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let log = lines.collect::<io::Result<Vec<String>>>()?;
    Ok(Pulled { json, report, log })
}

/// `YYYYMMDDTHHMMSSZ` for a time since the Unix epoch (UTC).
pub fn utc_timestamp(time: SystemTime) -> String {
    let secs = time.duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs());
    let (days, rem) = (secs / 86_400, secs % 86_400);
    let (h, m, s) = (rem / 3600, rem % 3600 / 60, rem % 60);
    // Civil date from days since 1970-01-01 (Howard Hinnant's algorithm).
    let z = days as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!("{year:04}{month:02}{day:02}T{h:02}{m:02}{s:02}Z")
}

/// `<utc>-<model>.json`, with the model reduced to `[A-Za-z0-9._-]`.
pub fn report_file_name(report: &Report, time: SystemTime) -> String {
    let model: String = report
        .model
        .as_deref()
        .unwrap_or("unknown")
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') {
                c
            } else {
                '-'
            }
        })
        .collect();
    let model = if model.is_empty() {
        "unknown".to_owned()
    } else {
        model
    };
    format!("{}-{model}.json", utc_timestamp(time))
}

fn or_dash(value: Option<&str>) -> &str {
    value.unwrap_or("-")
}

/// A one-screen summary: GL version, vertex storage blocks, the flagged EGL extensions, the probe
/// result and the launch timings.
pub fn summary(report: &Report) -> String {
    let mut out = String::new();
    let mut line = |text: String| {
        out.push_str(&text);
        out.push('\n');
    };
    line(format!(
        "model {}  platform {}  cpu {} x{} ({}-bit)",
        or_dash(report.model.as_deref()),
        or_dash(report.platform_version.as_deref()),
        or_dash(report.cpu.model_name.as_deref()),
        report.cpu.processors,
        report.cpu.pointer_width
    ));
    match &report.gles {
        Some(gles) => line(format!(
            "GL ES  {}  ({})  vertex storage blocks {}  max texture {}",
            or_dash(gles.version.as_deref()),
            or_dash(gles.renderer.as_deref()),
            gles.max_vertex_shader_storage_blocks
                .map_or("-".into(), |v| v.to_string()),
            gles.max_texture_size.map_or("-".into(), |v| v.to_string()),
        )),
        None => line("GL ES  not queried".into()),
    }
    match &report.egl {
        Some(egl) => {
            line(format!(
                "EGL    {}  ({})",
                or_dash(egl.version.as_deref()),
                or_dash(egl.vendor.as_deref())
            ));
            let flags: Vec<String> = egl
                .flagged
                .iter()
                .map(|(name, present)| format!("{} {name}", if *present { "+" } else { "-" }))
                .collect();
            line(format!("       {}", flags.join("  ")));
        }
        None => line("EGL    not queried".into()),
    }
    let probe = &report.probe;
    let failed = probe.stages.iter().find_map(|s| s.error.clone());
    let state = match (probe.ok, failed) {
        (true, _) => "ok".to_owned(),
        (false, Some(error)) => error,
        (false, None) if !report.complete => {
            "running (partial report: the probe had not finished)".to_owned()
        }
        (false, None) => "not run".to_owned(),
    };
    line(format!(
        "probe  {}  frames {}  interval {}  adapter {}",
        state,
        probe.frames_presented,
        report
            .frame_interval_ms
            .map_or("-".into(), |v| format!("{v:.2} ms")),
        or_dash(probe.adapter.as_deref()),
    ));
    let steps: Vec<String> = report
        .steps
        .iter()
        .map(|s| format!("{} {:.0}", s.name, s.ms))
        .collect();
    line(format!(
        "launch {}",
        if steps.is_empty() {
            "-".into()
        } else {
            steps.join(" > ")
        }
    ));
    if !report.errors.is_empty() {
        line(format!("errors {}", report.errors.join("; ")));
    }
    out
}

#[cfg(test)]
mod tests;
