//! Integration test harness for `perlrs`: `tests/suite/` holds grouped cases (phases, control,
//! regex, eval/`$@`, closures, aggregates, parallelism, filesystem builtins, etc.);
//! `tests/common/` provides `eval*` helpers. Library unit tests live in `src/lib.rs`, `src/lexer.rs`,
//! and `src/value.rs`. Run everything with `cargo test`.

mod common;
mod suite;
