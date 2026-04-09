//! Public crate API: `perlrs::run` and `parse_and_run_string` with shared `Interpreter`.

use perlrs::interpreter::Interpreter;
use perlrs::{parse_and_run_string, run};

#[test]
fn run_returns_computed_integer() {
    assert_eq!(run("17 - 4").expect("run").to_int(), 13);
}

#[test]
fn run_returns_err_on_invalid_syntax() {
    assert!(run("}").is_err());
}

#[test]
fn parse_and_run_string_preserves_subroutine_definitions() {
    let mut interp = Interpreter::new();
    parse_and_run_string("sub api_t { return 40 + 2; }", &mut interp).expect("define");
    let v = parse_and_run_string("api_t()", &mut interp).expect("call");
    assert_eq!(v.to_int(), 42);
}
