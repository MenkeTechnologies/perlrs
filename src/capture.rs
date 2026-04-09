//! Structured shell output: `capture("cmd")`.

use std::process::Command;
use std::sync::Arc;

use crate::error::{PerlError, PerlResult};
use crate::interpreter::Interpreter;
use crate::value::{CaptureResult, PerlValue};

/// Run `cmd` through `sh -c` and return stdout, stderr, and exit code.
/// Updates [`Interpreter::child_exit_status`] (`$?`) like `system` and backticks.
pub fn run_capture(interp: &mut Interpreter, cmd: &str, line: usize) -> PerlResult<PerlValue> {
    let output = match Command::new("sh").arg("-c").arg(cmd).output() {
        Ok(o) => o,
        Err(e) => {
            interp.errno = e.to_string();
            interp.child_exit_status = -1;
            return Err(PerlError::runtime(format!("capture: {}", e), line));
        }
    };
    interp.record_child_exit_status(output.status);
    let exitcode = output.status.code().unwrap_or(-1) as i64;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    Ok(PerlValue::capture(Arc::new(CaptureResult {
        stdout,
        stderr,
        exitcode,
    })))
}
