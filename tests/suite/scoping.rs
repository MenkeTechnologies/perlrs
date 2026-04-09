//! Lexical scoping: inner `my` vs outer bindings (integration-level).

use crate::common::*;

#[test]
fn inner_my_shadows_outer_in_do_block() {
    assert_eq!(eval_int("my $x = 1; do { my $x = 2; $x }"), 2);
}

#[test]
fn outer_scalar_preserved_after_inner_declaration() {
    assert_eq!(eval_int("my $x = 1; do { my $y = 99; $y }; $x"), 1);
}

#[test]
fn nested_blocks_each_see_correct_lexicals() {
    assert_eq!(
        eval_int(
            "my $a = 1; \
             my $b = do { \
                 my $a = 10; \
                 $a + $a \
             }; \
             $a + $b",
        ),
        21
    );
}

#[test]
fn our_declares_package_global_scalar() {
    assert_eq!(eval_int("our $pkg = 42; $pkg"), 42);
}

#[test]
fn local_restores_outer_lexical_after_block() {
    // Avoid `do { } + $x` on one line — the parser can bind `+` inside the block.
    assert_eq!(
        eval_int(
            "my $x = 1; \
             my $inner = do { local $x = 9; $x }; \
             $inner + $x",
        ),
        10
    );
}
