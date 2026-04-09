//! Progress reporting on stderr for parallel **`p*`** builtins when `progress => EXPR` is truthy
//! (`pmap`, `pgrep`, `pfor`, `preduce`, `fan`, …).
//!
//! On a **TTY** (stderr), progress uses the **alternate screen buffer** (`\x1b[?1049h`) and redraws the
//! **full terminal** each tick (`\x1b[2J` + home) so the bar animates in place without scrolling the
//! main scrollback. The cursor is hidden during the run and restored on [`PmapProgress::finish`].
//! If the process panics after entering the alternate screen, [`Drop`] restores the main buffer.
//!
//! Set **`PERLRS_PROGRESS_PLAIN=1`** to force the older single-line `\r` mode (keeps scrollback).
//! **Non-TTY** stderr prints one line per completion (e.g. CI logs).
//!
//! Rayon workers call [`PmapProgress::tick`] concurrently; a mutex serializes redraws.

use std::io::{self, IsTerminal, Write};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use parking_lot::Mutex;

/// Renders fullscreen (TTY) or line-based progress on stderr while parallel work runs.
pub(crate) struct PmapProgress {
    total: usize,
    done: AtomicUsize,
    render: Mutex<()>,
    enabled: bool,
    tty: bool,
    /// True after we emit `ESC [ ? 1 0 4 9 h` (alternate screen).
    alt_active: AtomicBool,
    /// True after [`Self::finish`] or [`Drop`] left alternate screen.
    finished: AtomicBool,
    /// When set via env, use `\r` line mode instead of alternate screen.
    plain_line_mode: bool,
}

impl PmapProgress {
    pub fn new(enabled: bool, total: usize) -> Self {
        let tty = io::stderr().is_terminal();
        let plain_line_mode = env_plain_line_mode();
        Self {
            total,
            done: AtomicUsize::new(0),
            render: Mutex::new(()),
            enabled: enabled && total > 0,
            tty,
            alt_active: AtomicBool::new(false),
            finished: AtomicBool::new(false),
            plain_line_mode,
        }
    }

    #[inline]
    pub fn tick(&self) {
        if !self.enabled {
            return;
        }
        let d = self.done.fetch_add(1, Ordering::Relaxed) + 1;
        let _guard = self.render.lock();
        let _ = io::stdout().flush();
        let mut stderr = io::stderr().lock();
        if self.tty && !self.plain_line_mode {
            if d == 1 {
                write!(stderr, "\x1b[?1049h\x1b[?25l").ok();
                self.alt_active.store(true, Ordering::SeqCst);
            }
            write_fullscreen_frame(&mut stderr, d, self.total);
        } else if self.tty {
            write_line_mode_bar(&mut stderr, d, self.total);
        } else {
            write_piped_lines(&mut stderr, d, self.total);
        }
    }

    pub fn finish(&self) {
        if !self.enabled {
            return;
        }
        let _guard = self.render.lock();
        let _ = io::stdout().flush();
        let mut stderr = io::stderr().lock();
        if self.tty && self.alt_active.load(Ordering::SeqCst) {
            if !self.finished.swap(true, Ordering::SeqCst) {
                write!(stderr, "\x1b[?25h\x1b[?1049l\n").ok();
            }
        } else {
            // Plain `\r` line or piped stderr: ensure a trailing newline after the bar.
            let _ = writeln!(stderr);
        }
        stderr.flush().ok();
    }
}

impl Drop for PmapProgress {
    fn drop(&mut self) {
        if !self.enabled {
            return;
        }
        // If finish() already left alternate screen, skip. Otherwise restore (e.g. panic path).
        if self.finished.swap(true, Ordering::SeqCst) {
            return;
        }
        if self.tty && self.alt_active.load(Ordering::SeqCst) {
            // Do not lock `render`: another thread may have panicked while holding it.
            let _ = write!(io::stderr(), "\x1b[?25h\x1b[?1049l\n");
        }
    }
}

/// `PERLRS_PROGRESS_PLAIN=1` (or any non-`0` / non-`false` value) disables alternate-screen mode.
fn env_plain_line_mode() -> bool {
    match std::env::var("PERLRS_PROGRESS_PLAIN") {
        Ok(s) if s == "0" || s.eq_ignore_ascii_case("false") => false,
        Ok(s) if s.is_empty() => false,
        Ok(_) => true,
        Err(_) => false,
    }
}

fn terminal_width() -> usize {
    std::env::var("COLUMNS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(80)
        .clamp(40, 200)
}

fn terminal_height() -> usize {
    std::env::var("LINES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(24)
        .max(1)
}

fn write_fullscreen_frame(stderr: &mut dyn Write, done: usize, total: usize) {
    let cols = terminal_width();
    let rows = terminal_height();
    let bar_w = (cols.saturating_sub(4)).min(96).max(16);
    let filled = (done * bar_w) / total.max(1);
    let pct = (done * 100) / total.max(1);
    let bar: String = (0..bar_w).map(|i| if i < filled { '█' } else { '░' }).collect();

    // Full clear + redraw = "animation" frame; scrollback on main buffer is untouched.
    write!(stderr, "\x1b[2J\x1b[H").ok();

    let pad_top = (rows / 2).saturating_sub(3);
    for _ in 0..pad_top {
        writeln!(stderr).ok();
    }

    let inner = bar_w + 14;
    let pad_l = (cols.saturating_sub(inner)) / 2;
    let pad = " ".repeat(pad_l);

    writeln!(stderr, "{}parallel", pad).ok();
    writeln!(stderr).ok();
    writeln!(stderr, "{}[{}]", pad, bar).ok();
    writeln!(
        stderr,
        "{}  {:3}%     {}/{}",
        pad, pct, done, total
    )
    .ok();
    stderr.flush().ok();
}

fn write_line_mode_bar(stderr: &mut dyn Write, done: usize, total: usize) {
    const W: usize = 48;
    let filled = (done * W) / total.max(1);
    let pct = (done * 100) / total.max(1);
    let bar: String = (0..W).map(|i| if i < filled { '█' } else { '░' }).collect();
    write!(
        stderr,
        "\r\x1b[K[parallel] [{}] {:3}% ({}/{})",
        bar, pct, done, total
    )
    .ok();
    stderr.flush().ok();
}

fn write_piped_lines(stderr: &mut dyn Write, done: usize, total: usize) {
    const W: usize = 48;
    let filled = (done * W) / total.max(1);
    let pct = (done * 100) / total.max(1);
    let bar: String = (0..W).map(|i| if i < filled { '█' } else { '░' }).collect();
    writeln!(
        stderr,
        "[parallel] [{}] {:3}% ({}/{})",
        bar, pct, done, total
    )
    .ok();
    stderr.flush().ok();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_progress_tick_is_noop() {
        let p = PmapProgress::new(false, 10);
        for _ in 0..5 {
            p.tick();
        }
        p.finish();
    }

    #[test]
    fn zero_total_disables_progress() {
        let p = PmapProgress::new(true, 0);
        p.tick();
        p.finish();
    }
}
