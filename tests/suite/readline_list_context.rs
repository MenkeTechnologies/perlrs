//! `<>` / `<STDIN>` in **list** context must read all lines until EOF (Perl `readline` list semantics).

use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn diamond_list_context_slurps_piped_stdin() {
    let exe = env!("CARGO_BIN_EXE_perlrs");
    let mut child = Command::new(exe)
        .args(["-e", r#"my @a = <>; print scalar(@a)"#])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn perlrs");
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(b"a\nb\nc\n")
        .expect("write stdin");
    let out = child.wait_with_output().expect("wait");
    assert!(
        out.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "3");
}

#[test]
fn stdin_angle_bracket_list_context_slurps_piped_stdin() {
    let exe = env!("CARGO_BIN_EXE_perlrs");
    let mut child = Command::new(exe)
        .args(["-e", r#"my @a = <STDIN>; print scalar(@a)"#])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn perlrs");
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(b"a\nb\nc\n")
        .expect("write stdin");
    let out = child.wait_with_output().expect("wait");
    assert!(
        out.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "3");
}
