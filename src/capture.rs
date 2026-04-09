//! Structured shell output: `capture("cmd")`.

use std::process::Command;
use std::sync::Arc;

use crate::error::{PerlError, PerlResult};
use crate::value::{CaptureResult, PerlValue};

/// Run `cmd` through `sh -c` and return stdout, stderr, and exit code.
pub fn run_capture(cmd: &str, line: usize) -> PerlResult<PerlValue> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .map_err(|e| PerlError::runtime(format!("capture: {}", e), line))?;
    let exitcode = output.status.code().unwrap_or(-1) as i64;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    Ok(PerlValue::Capture(Arc::new(CaptureResult {
        stdout,
        stderr,
        exitcode,
    })))
}
