//! Typeglob assignment (`*foo = \\&bar`, `*foo = *bar`) for subroutine aliasing and stash copy.

use crate::common::*;

#[test]
fn typeglob_assign_coderef_installs_sub_alias() {
    assert_eq!(
        eval_int(
            r#"no strict 'vars';
            sub orig { 41 }
            *alias = \&orig;
            alias() + 1"#,
        ),
        42
    );
}

#[test]
fn typeglob_assign_glob_copies_subroutine_slot() {
    assert_eq!(
        eval_int(
            r#"no strict 'vars';
            sub one { 7 }
            *two = *one;
            two() * 2"#,
        ),
        14
    );
}

#[test]
fn typeglob_qualified_names_parse() {
    let p = perlrs::parse("*Foo::x = *Foo::y;").expect("parse");
    assert!(!p.statements.is_empty());
}
