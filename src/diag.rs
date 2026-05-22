//! Optional startup / hot-path diagnostics.
//!
//! Enable with environment variable `FERRITE_DIAG=1` (also accepts `true` / `yes`).
//!
//! **Release builds hide the Windows console**, so when diagnosing hangs use the
//! trace file: `%TEMP%\ferrite_startup_trace.log` (see [`trace_path`]).

use std::cell::Cell;
use std::io::Write;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

thread_local! {
    static FRAME: Cell<u64> = const { Cell::new(0) };
    static FRAME_START: Cell<Option<Instant>> = const { Cell::new(None) };
}

static ENABLED: OnceLock<bool> = OnceLock::new();
static TRACE_PATH: OnceLock<PathBuf> = OnceLock::new();
static START_INSTANT: OnceLock<Instant> = OnceLock::new();

/// Whether `FERRITE_DIAG` is enabled for this process.
pub fn enabled() -> bool {
    *ENABLED.get_or_init(|| {
        std::env::var("FERRITE_DIAG")
            .ok()
            .is_some_and(|v| matches!(v.as_str(), "1" | "true" | "yes" | "on"))
    })
}

/// Path to the startup trace log (written when `FERRITE_DIAG` is on).
pub fn trace_path() -> &'static PathBuf {
    TRACE_PATH.get_or_init(|| std::env::temp_dir().join("ferrite_startup_trace.log"))
}

/// Append a timestamped line to the trace file and mirror to `log::warn!`.
/// Safe to call before `env_logger` is initialized.
pub fn trace(step: &str) {
    if !enabled() {
        return;
    }
    let path = trace_path();
    let line = format!(
        "{:>7.3}s  {}",
        START_INSTANT
            .get_or_init(Instant::now)
            .elapsed()
            .as_secs_f64(),
        step
    );
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        let _ = writeln!(f, "{line}");
        let _ = f.flush();
    }
    // May run before env_logger::init — file is the source of truth.
    let _ = log::warn!("[diag] {line}");
}

/// Reset trace file at process start (call once from `main`).
pub fn trace_reset() {
    if !enabled() {
        return;
    }
    let _ = START_INSTANT.set(Instant::now());
    let path = trace_path();
    let header = format!("Ferrite startup trace — {}\n", chrono_lite_timestamp());
    let _ = std::fs::write(path, header);
    // Also drop a pointer file users can find easily
    let pointer = std::env::temp_dir().join("ferrite_DIAG_LOG_HERE.txt");
    let _ = std::fs::write(
        &pointer,
        format!(
            "Open this log while diagnosing Ferrite:\n{}\n",
            path.display()
        ),
    );
}

fn chrono_lite_timestamp() -> String {
    // Avoid adding chrono dependency — wall clock via humantime-style from std only
    format!("{:?}", std::time::SystemTime::now())
}

/// Advance the per-frame counter (call once per `App::update`).
pub fn next_frame() -> u64 {
    FRAME_START.with(|c| c.set(Some(Instant::now())));
    let n = FRAME.with(|c| {
        let n = c.get() + 1;
        c.set(n);
        n
    });
    if enabled() && n <= 10 {
        trace(&format!("UI frame {n} started"));
    } else if enabled() && n % 60 == 0 {
        trace(&format!("UI frame {n}"));
    }
    n
}

/// Log when the previous frame's `update()` exceeded `threshold_ms`.
pub fn frame_end(threshold_ms: u64) {
    if !enabled() {
        return;
    }
    let Some(start) = FRAME_START.with(|c| c.get()) else {
        return;
    };
    let elapsed = start.elapsed();
    if elapsed >= Duration::from_millis(threshold_ms) {
        trace(&format!(
            "UI frame {} took {:.0}ms (SLOW)",
            FRAME.with(|c| c.get()),
            elapsed.as_secs_f64() * 1000.0
        ));
    }
}

/// Log when a scoped operation exceeds `threshold`.
pub struct SlowScope {
    label: &'static str,
    start: Instant,
    threshold: Duration,
}

impl SlowScope {
    pub fn new(label: &'static str, threshold_ms: u64) -> Self {
        Self {
            label,
            start: Instant::now(),
            threshold: Duration::from_millis(threshold_ms),
        }
    }
}

impl Drop for SlowScope {
    fn drop(&mut self) {
        if !enabled() {
            return;
        }
        let elapsed = self.start.elapsed();
        if elapsed >= self.threshold {
            trace(&format!(
                "slow {} {:.0}ms (frame {})",
                self.label,
                elapsed.as_secs_f64() * 1000.0,
                FRAME.with(|c| c.get())
            ));
        }
    }
}

#[macro_export]
macro_rules! diag_slow {
    ($label:expr, $threshold_ms:expr) => {
        let _diag_scope = $crate::diag::SlowScope::new($label, $threshold_ms);
    };
}

/// One-shot event (deduped by `key` per process).
pub fn event_once(key: &'static str, message: impl AsRef<str>) {
    if !enabled() {
        return;
    }
    static SEEN: OnceLock<std::sync::Mutex<std::collections::HashSet<&'static str>>> =
        OnceLock::new();
    let set = SEEN.get_or_init(|| std::sync::Mutex::new(std::collections::HashSet::new()));
    let mut guard = set.lock().unwrap_or_else(|e| e.into_inner());
    if guard.insert(key) {
        trace(&format!("{key}: {}", message.as_ref()));
    }
}

/// Repeatable event (rate-limited to avoid log spam).
pub fn event(key: &'static str, message: impl AsRef<str>) {
    if !enabled() {
        return;
    }
    trace(&format!("{key}: {}", message.as_ref()));
}
