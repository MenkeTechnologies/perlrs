use crate::common::*;
use perlrs::error::ErrorKind;
use perlrs::value::PerlValue;

#[test]
fn parse_unclosed_brace_is_syntax_error() {
    let err = perlrs::parse("sub f {").unwrap_err();
    assert_eq!(err.kind, ErrorKind::Syntax);
}

#[test]
fn parse_lone_brace_is_syntax_error() {
    assert_eq!(parse_err_kind("}"), ErrorKind::Syntax);
}

#[test]
fn division_by_zero_is_runtime_error() {
    assert_eq!(eval_err_kind("1 / 0"), ErrorKind::Runtime);
}

#[test]
fn modulus_zero_is_runtime_error() {
    assert_eq!(eval_err_kind("1 % 0"), ErrorKind::Runtime);
}

#[test]
fn die_is_die_kind() {
    assert_eq!(eval_err_kind(r#"die "stop""#), ErrorKind::Die);
}

#[test]
fn exit_zero_is_swallowed_as_success_in_execute() {
    // `execute` treats `Exit(0)` like normal completion (break without `Err`).
    let program = perlrs::parse("exit(0)").expect("parse");
    let mut interp = perlrs::interpreter::Interpreter::new();
    let v = interp.execute(&program).expect("execute");
    assert!(matches!(v, PerlValue::Undef));
}

#[test]
fn unterminated_double_quoted_string_is_syntax_error() {
    assert_eq!(parse_err_kind(r#"my $x = "unfinished"#), ErrorKind::Syntax);
}

#[test]
fn unexpected_eof_after_operator_is_syntax_error() {
    assert_eq!(parse_err_kind("++"), ErrorKind::Syntax);
}
