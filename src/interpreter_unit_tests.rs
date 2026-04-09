//! Unit tests for `Interpreter`: defaults, `set_file`, and `execute_tree` behavior.

use crate::interpreter::Interpreter;
use crate::parse;

#[test]
fn new_default_file_is_dash_e() {
    assert_eq!(Interpreter::new().file, "-e");
}

#[test]
fn new_default_program_name() {
    assert_eq!(Interpreter::new().program_name, "perlrs");
}

#[test]
fn new_default_irs_newline() {
    assert_eq!(Interpreter::new().irs, "\n");
}

#[test]
fn new_line_number_starts_zero() {
    assert_eq!(Interpreter::new().line_number, 0);
}

#[test]
fn new_env_populated_from_process() {
    let i = Interpreter::new();
    assert!(
        i.env.contains_key("PATH") || i.env.contains_key("HOME") || !i.env.is_empty(),
        "expected some process env in interpreter env"
    );
}

#[test]
fn set_file_updates_file_field() {
    let mut i = Interpreter::new();
    i.set_file("t/foo.pl");
    assert_eq!(i.file, "t/foo.pl");
}

#[test]
fn execute_tree_computed_expression() {
    let p = parse("7 * 6;").expect("parse");
    let mut i = Interpreter::new();
    let v = i.execute_tree(&p).expect("execute_tree");
    assert_eq!(v.to_int(), 42);
}

#[test]
fn execute_tree_my_scalar_sequence() {
    let p = parse("my $a = 10; my $b = 32; $a + $b;").expect("parse");
    let mut i = Interpreter::new();
    let v = i.execute_tree(&p).expect("execute_tree");
    assert_eq!(v.to_int(), 42);
}

#[test]
fn execute_tree_registers_sub_for_later_call() {
    let p = parse("sub times6 { return $_[0] * 6; } times6(7);").expect("parse");
    let mut i = Interpreter::new();
    let v = i.execute_tree(&p).expect("execute_tree");
    assert_eq!(v.to_int(), 42);
}

#[test]
fn execute_preserves_scope_scalar_across_two_parses() {
    let p1 = parse("my $interp_unit_x = 41;").expect("parse");
    let p2 = parse("$interp_unit_x + 1;").expect("parse");
    let mut i = Interpreter::new();
    i.execute_tree(&p1).expect("first");
    let v = i.execute_tree(&p2).expect("second");
    assert_eq!(v.to_int(), 42);
}

#[test]
fn subs_map_holds_declared_sub() {
    let p = parse("sub interp_named { 1 }").expect("parse");
    let mut i = Interpreter::new();
    i.execute_tree(&p).expect("execute_tree");
    assert!(i.subs.contains_key("interp_named"));
}
