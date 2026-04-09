//! Integration test harness for `perlrs`: `tests/suite/` holds grouped cases (phases, control,
//! regex, eval/`$@`, closures, aggregates, parallelism, filesystem builtins, `lib_api` for
//! `run` / `parse_and_run_string`, etc.); `tests/common/` provides `eval*` helpers. Library unit
//! tests cover `parse()`, `run`, lexer (`q{}`, `qr//`, floats, `m//`, strings, `<=>`), `Scope`
//! (arrays, hashes),
//! `keyword_or_ident`, `PerlError` display, and `PerlValue`. Run with `cargo test`.

mod common;
mod suite;
