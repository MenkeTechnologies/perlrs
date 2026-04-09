//! Unit tests for the crate root API: `parse`, `run`, `parse_and_run_string`, `try_vm_execute`.

use crate::interpreter::Interpreter;
use crate::value::PerlValue;
use crate::{parse, parse_and_run_string, run, try_vm_execute};

fn run_int(code: &str) -> i64 {
    run(code).expect("run").to_int()
}

#[test]
fn run_arithmetic_add_sub_mul_div_mod() {
    assert_eq!(run_int("11 + 4;"), 15);
    assert_eq!(run_int("20 - 7;"), 13);
    assert_eq!(run_int("6 * 9;"), 54);
    assert_eq!(run_int("22 / 4;"), 5);
    assert_eq!(run_int("17 % 5;"), 2);
}

#[test]
fn run_power_and_precedence() {
    assert_eq!(run_int("2 ** 8;"), 256);
    assert_eq!(run_int("2 + 3 * 4;"), 14);
    assert_eq!(run_int("(2 + 3) * 4;"), 20);
}

#[test]
fn run_numeric_comparisons_yield_perl_truth() {
    assert_eq!(run_int("5 == 5;"), 1);
    assert_eq!(run_int("5 != 3;"), 1);
    assert_eq!(run_int("3 < 5;"), 1);
    assert_eq!(run_int("5 > 3;"), 1);
    assert_eq!(run_int("5 <= 5;"), 1);
    assert_eq!(run_int("5 >= 4;"), 1);
}

#[test]
fn run_spaceship_operator() {
    assert_eq!(run_int("5 <=> 3;"), 1);
    assert_eq!(run_int("3 <=> 5;"), -1);
    assert_eq!(run_int("4 <=> 4;"), 0);
}

#[test]
fn run_string_cmp_and_eq() {
    assert_eq!(run_int(r#""a" cmp "b";"#), -1);
    assert_eq!(run_int(r#""b" cmp "a";"#), 1);
    assert_eq!(run_int(r#""a" eq "a";"#), 1);
    assert_eq!(run_int(r#""a" ne "b";"#), 1);
}

#[test]
fn run_logical_short_circuit() {
    assert_eq!(run_int("1 && 7;"), 7);
    assert_eq!(run_int("0 && 7;"), 0);
    assert_eq!(run_int("0 || 8;"), 8);
    assert_eq!(run_int("3 || 8;"), 3);
}

#[test]
fn run_defined_or_operator() {
    assert_eq!(run_int("undef // 99;"), 99);
    assert_eq!(run_int("0 // 5;"), 0);
}

#[test]
fn run_bitwise_ops() {
    assert_eq!(run_int("0x0F & 0x33;"), 0x03);
    assert_eq!(run_int("0x01 | 0x02;"), 0x03);
    assert_eq!(run_int("0x0F ^ 0x33;"), 0x3C);
}

#[test]
fn run_unary_minus_and_not() {
    assert_eq!(run_int("- 42;"), -42);
    assert_eq!(run_int("!0;"), 1);
    assert_eq!(run_int("!1;"), 0);
}

#[test]
fn run_concat_and_repeat() {
    assert_eq!(run(r#""a" . "b" . "c";"#).expect("run").to_string(), "abc");
    assert_eq!(run(r#""x" x 4;"#).expect("run").to_string(), "xxxx");
}

#[test]
fn run_list_and_scalar_context_array() {
    assert_eq!(run_int("scalar (1, 2, 3);"), 3);
}

#[test]
fn run_my_variable_and_assignment() {
    assert_eq!(run_int("my $x = 41; $x + 1;"), 42);
}

#[test]
fn run_conditional_expression() {
    assert_eq!(run_int("1 ? 10 : 20;"), 10);
    assert_eq!(run_int("0 ? 10 : 20;"), 20);
}

#[test]
fn run_simple_subroutine() {
    assert_eq!(
        run_int("sub add2 { return $_[0] + $_[1]; } add2(30, 12);"),
        42
    );
}

#[test]
fn parse_and_run_string_shares_interpreter_state() {
    let mut i = Interpreter::new();
    parse_and_run_string("my $crate_api_z = 100;", &mut i).expect("first");
    let v = parse_and_run_string("$crate_api_z + 1;", &mut i).expect("second");
    assert_eq!(v.to_int(), 101);
}

#[test]
fn try_vm_execute_runs_simple_literal_program() {
    let p = parse("42;").expect("parse");
    let mut i = Interpreter::new();
    let out = try_vm_execute(&p, &mut i);
    assert!(out.is_some());
    assert_eq!(out.unwrap().expect("vm").to_int(), 42);
}

#[test]
fn try_vm_execute_none_when_begin_block() {
    let p = parse("BEGIN { 1; } 2;").expect("parse");
    let mut i = Interpreter::new();
    assert!(try_vm_execute(&p, &mut i).is_none());
}

#[test]
fn run_empty_statement_list_undef_or_zero() {
    let v = run(";;;").expect("run");
    assert!(matches!(v, PerlValue::Undef) || v.to_int() == 0);
}

#[test]
fn parse_returns_empty_program_for_whitespace() {
    let p = parse("   \n  ").expect("parse");
    assert!(p.statements.is_empty());
}
