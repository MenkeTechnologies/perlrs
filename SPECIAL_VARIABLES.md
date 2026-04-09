# Perl special variables vs perlrs

This document audits **Perl 5’s “special” globals** against **perlrs** as implemented in the tree-walker / VM (`src/interpreter.rs`, `src/lexer.rs`, `src/vm.rs`, `src/scope.rs`). It is **not** an exhaustive perlvar(1) list; it groups the usual categories and states what is wired, partial, or absent.

Legend: **Yes** = behavior matches intent for typical use; **Partial** = exists but semantics differ; **No** = not implemented or wrong tokenization.

---

## Implemented with dedicated handling

| Perl | Role | perlrs |
|------|------|--------|
| `$_` | Default topic | Ordinary scalar `$_` in scope; set by `map`/`grep`/many iterators, `given`, `readline`, etc. |
| `$.` | Input line number | `Interpreter.line_number` via `get_special_var(".")` (`src/interpreter.rs`); incremented on `readline` paths. |
| `$/` | Input record separator | `irs` field; get/set via `get_special_var` / `set_special_var` for `"/"`. |
| `$,` | Output field separator | `ofs` field; `","` in special get/set. |
| `$\` | Output record separator | `ors` field; `"\\"` in special get/set. |
| `$!` | OS error (errno string) | Reads use `Interpreter.errno` (`get_special_var("!")`). Writes go to the scalar stash and **do not** update `errno`, so they are not read back — prefer treating `$!` as read-only. |
| `$@` | Eval error | Reads use `eval_error` (`get_special_var("@")`). Writes store a scalar `"@"` that is **not** read back — same read/write split as `$!`. |
| `$0` | Program name | `program_name`; `"0"` in special get/set. |
| `$$` | Process ID | `get_special_var("$$")` → `std::process::id()`. |
| `$1`…`$n` | Capture groups | After a successful match, `apply_regex_captures` sets `scope` scalars `"1"`…`"n"` (`src/interpreter.rs`). |
| `%+` | Named captures | `scope.set_hash("+", …)` from regex named groups. |
| `@ARGV` | Script arguments | Declared in `Interpreter::new`; populated by `main` driver (`src/main.rs`). |
| `@INC` | Library path | Array of search dirs; `%INC` used for loaded paths in `require`. |
| `%ENV` | Environment | Hash in scope, initialized from `std::env::vars()`. |
| `__PACKAGE__` | Current package | Scalar in scope; `package` statements update it. |
| `wantarray` | List/scalar/void context | `WantarrayCtx` on interpreter; `ExprKind::Wantarray` / `BuiltinId::Wantarray`. |

---

## Partially implemented or different from Perl 5

| Perl | Issue |
|------|--------|
| `$!` / `$@` | **String** errno / eval error only; not dual-var. Assignments do not feed back into reads (see table above). |
| `$.` | Updated on **readline-style** I/O; not a full per-handle line counter as in Perl. |
| `$1`…`$n`, `%+` | Driven by the **Rust `regex` crate**; Perl’s regexp engine differs (lookbehind, backtracking, etc.). |
| `@_` | Works as the **subroutine argument array** in user subs; not fully identical to Perl’s XS calling conventions. |
| `pos $_` | Supported with `regex_pos` map; edge cases may differ from Perl. |

---

## Lexer may tokenize but no Perl semantics

Single-character names after `$` are accepted (`src/lexer.rs` `read_variable_name`), including `&` `` ` `` `'` `+` `*` `?` `|` etc. **Only** the subset handled in `get_special_var` / `set_special_var` and regex capture logic has meaning. The rest resolve as **ordinary scalars** in scope (usually undef), **not** Perl’s `$&`, `` $` ``, `$'`, `$+`, `$|`, etc.

**`$^X` / `$^O` / other `$^A` control variables:** The lexer reads **one** character after `$` for this class, so `$^` becomes the scalar named `"^"`, not Perl’s `$^O` (caret + letter). **Not supported** as in Perl.

---

## Not implemented (common Perl specials)

| Category | Examples |
|----------|----------|
| **Match / regexp** | `$&`, `` $` ``, `$'`, `$+` (last bracket), `${^MATCH}` etc. — not set from engine. |
| **Output** | `$|` output autoflush — not wired to stdio flush behavior. |
| **Process / status** | `$?` child exit status, `$^E` extended OS error, `$PROCESS_ID` aliases. |
| **Ids / groups** | `$<` `$>` `$(` `$)` real/effective uid/gid. |
| **Perlio / globs** | Many handle-related specials beyond what IO builtins use. |
| **Signals** | `%SIG` — not implemented. |
| **Compiler / phase** | `$^H`, `${^WARNING_BITS}`, `${^GLOBAL_PHASE}`, etc. |
| **Debugging** | `$^D`, `$^P`, … |
| **Time** | `$^T` base time, `$^V` version object. |
| **Warnings** | `$^W`; interpreter uses `warnings` boolean + `feature_bits`, not `$^W` scalar. |
| **List formatting** | `$"` (`$LIST_SEPARATOR`) for array stringification — not present; `join` takes an explicit separator. |
| **English.pm** | No `English` module tying long names to these variables. |

---

## Maintenance

When adding I/O, regex, or `eval` behavior, update this file if new globals become meaningful or if `get_special_var` / `set_special_var` change.
