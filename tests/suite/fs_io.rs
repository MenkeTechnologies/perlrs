//! Filesystem builtins with real temp paths (headless-safe, no network).

use crate::common::*;
use std::path::PathBuf;

#[test]
fn mkdir_creates_directory_and_file_test_sees_it() {
    let dir: PathBuf =
        std::env::temp_dir().join(format!("perlrs_itest_mkdir_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let p = dir.to_str().expect("temp path utf-8");
    let code = format!(r#"mkdir("{p}", 0755); (-e "{p}" ? 1 : 0)"#);
    assert_eq!(eval_int(&code), 1);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn unlink_returns_zero_for_missing_file() {
    assert_eq!(
        eval_int(r#"unlink("/nonexistent_path_perlrs_itest_01234")"#),
        0
    );
}
