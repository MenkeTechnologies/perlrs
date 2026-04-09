//! Anonymous subs and lexical capture.

use crate::common::*;

#[test]
fn anon_sub_captures_outer_lexical() {
    assert_eq!(
        eval_int(
            "my $x = 10; \
             my $c = sub { $x + 5 }; \
             $c->()",
        ),
        15
    );
}

#[test]
fn sub_implicit_return_last_expression() {
    assert_eq!(eval_int("sub foo { 5 } foo()"), 5);
}

#[test]
fn named_sub_captures_outer_lexical_vm_and_tree() {
    assert_eq!(
        eval_int(
            "my $x = 10; \
             sub foo { $x + 5 } \
             foo()",
        ),
        15
    );
}
