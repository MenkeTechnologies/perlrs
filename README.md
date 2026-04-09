# perlrs

A highly parallel Perl 5 interpreter written in Rust by [MenkeTechnologies](https://github.com/MenkeTechnologies).

## Overview

`perlrs` is a Perl 5 compatible interpreter that brings native parallelism to Perl scripting via rayon-powered parallel primitives. It parses and executes Perl 5 scripts with a focus on leveraging all available CPU cores for data-parallel operations.

Built with:
- **rayon** for work-stealing parallel execution
- **regex** for SIMD-accelerated pattern matching
- **clap** for full Perl 5 CLI compatibility
- **parking_lot** and **crossbeam** for low-overhead synchronization
- LTO + single codegen unit release builds for maximum throughput

## Installation

```bash
cargo install --path .
```

This installs two binaries: `perlrs` and `pe` (short alias).

## Usage

```bash
# Execute inline code
perlrs -e 'print "Hello, world!\n"'

# Run a script
perlrs script.pl arg1 arg2

# Process stdin line by line
echo "data" | perlrs -ne 'print uc $_'

# Auto-print mode
cat file.txt | perlrs -pe 's/foo/bar/g'

# Check syntax without executing
perlrs -c script.pl

# Set parallel thread count
perlrs -j 8 -e 'my @r = pmap { heavy_work($_) } @data'
```

## Parallel Extensions

The core differentiator: `pmap`, `pgrep`, `pfor`, and `psort` distribute work across all CPU cores automatically via rayon's work-stealing scheduler.

```perl
# Parallel map — transform elements across all cores
my @doubled = pmap { $_ * 2 } @data;

# Parallel grep — filter elements in parallel
my @evens = pgrep { $_ % 2 == 0 } @data;

# Parallel foreach — execute side effects concurrently
pfor { process($_) } @items;

# Parallel sort — sort using all cores
my @sorted = psort { $a <=> $b } @data;

# Chain them together
my @result = pmap { $_ ** 2 } pgrep { $_ > 100 } @data;
```

Each parallel block receives its own interpreter context with captured lexical scope, so there are no data races. The sequential equivalents (`map`, `grep`, `sort`, `foreach`) work identically for correctness testing.

## Supported Perl Features

### Data Types
- Scalars (`$x`), arrays (`@a`), hashes (`%h`)
- References: `\$x`, `\@a`, `\%h`, `\&sub`
- Array refs `[1,2,3]`, hash refs `{a => 1}`
- Code refs / closures `sub { ... }`
- Regex objects `qr/.../`
- Blessed references (basic OOP)

### Control Flow
- `if`/`elsif`/`else`, `unless`
- `while`, `until`, `do...while`
- `for` (C-style), `foreach`
- `last`, `next`, `redo` with labels
- Postfix modifiers: `expr if COND`, `expr unless COND`, `expr while COND`, `expr for @list`
- Ternary `?:`

### Operators
- Arithmetic: `+`, `-`, `*`, `/`, `%`, `**`
- String: `.` (concat), `x` (repeat)
- Comparison: `==`, `!=`, `<`, `>`, `<=`, `>=`, `<=>`
- String comparison: `eq`, `ne`, `lt`, `gt`, `le`, `ge`, `cmp`
- Logical: `&&`, `||`, `//`, `!`, `and`, `or`, `not`
- Bitwise: `&`, `|`, `^`, `~`, `<<`, `>>`
- Assignment: `=`, `+=`, `-=`, `*=`, `/=`, `.=`, `//=`, etc.
- Regex: `=~`, `!~`
- Range: `..`
- Increment/decrement: `++`, `--`
- Arrow dereference: `->`

### Regex
- Match: `$str =~ /pattern/flags`
- Substitution: `$str =~ s/pattern/replacement/flags`
- Transliterate: `$str =~ tr/from/to/`
- Flags: `g`, `i`, `m`, `s`, `x`
- Capture variables: `$1`, `$2`, etc.
- Quote-like: `m//`, `qr//`

### Subroutines
- Named subs with `sub name { ... }`
- Anonymous subs / closures
- Recursive calls
- `@_` argument passing, `shift`, `return`
- `return EXPR if COND` (postfix modifiers on return)

### Built-in Functions

**Array**: `push`, `pop`, `shift`, `unshift`, `splice`, `reverse`, `sort`, `map`, `grep`, `scalar`

**Hash**: `keys`, `values`, `each`, `delete`, `exists`

**String**: `chomp`, `chop`, `length`, `substr`, `index`, `rindex`, `split`, `join`, `sprintf`, `printf`, `uc`, `lc`, `ucfirst`, `lcfirst`, `chr`, `ord`, `hex`, `oct`

**Numeric**: `abs`, `int`, `sqrt`

**I/O**: `print`, `say`, `printf`, `open`, `close`, `eof`, `readline` (`<>`)

**File tests**: `-e`, `-f`, `-d`, `-l`, `-r`, `-w`, `-s`, `-z`

**System**: `system`, `exec`, `exit`, `chdir`, `mkdir`, `unlink`

**Type**: `defined`, `undef`, `ref`, `bless`

**Control**: `die`, `warn`, `eval`, `do`, `require`, `caller`

### Other Features
- `use strict`, `use warnings` (recognized)
- `package` declarations
- `BEGIN`/`END` blocks
- String interpolation with `$var`, `$hash{key}`, `$array[idx]`
- Heredocs (`<<EOF`)
- `qw()`, `q()`, `qq()`
- POD documentation skipping
- Shebang line handling

## CLI Flags

| Flag | Description |
|------|-------------|
| `-e CODE` | Execute one line of code |
| `-E CODE` | Like `-e` with all features enabled |
| `-n` | Line-by-line processing (wraps in `while(<>){}`) |
| `-p` | Like `-n` but prints `$_` after each line |
| `-i[EXT]` | Edit files in place |
| `-w` | Enable warnings |
| `-W` | Enable all warnings |
| `-c` | Check syntax only |
| `-l` | Automatic line-end processing |
| `-a` | Auto-split mode (populates `@F`) |
| `-F PATTERN` | Field separator for `-a` |
| `-0 DIGITS` | Input record separator |
| `-v` | Print version |
| `-I DIR` | Add to module search path |
| `-M MODULE` | `use MODULE` before running |
| `-m MODULE` | `use MODULE ()` before running |
| `-j N` | Set number of parallel threads |

## Architecture

```
Source Code
    |
    v
 Lexer (src/lexer.rs)
    | Tokens
    v
 Parser (src/parser.rs)
    | AST
    v
 Interpreter (src/interpreter.rs)
    |--- Sequential: map, grep, sort, foreach
    |--- Parallel:   pmap, pgrep, psort, pfor (rayon)
    v
 Output
```

- **Lexer**: Context-sensitive tokenizer handling Perl's ambiguous syntax (regex vs division, hash vs modulo, heredocs, interpolated strings)
- **Parser**: Recursive descent with Pratt precedence climbing for expressions
- **Interpreter**: Tree-walking execution with proper lexical scoping, `Arc<RwLock>` for thread-safe reference types
- **Parallelism**: Each parallel block gets an isolated interpreter with captured scope; rayon handles scheduling

## Examples

See the `examples/` directory:

```bash
perlrs examples/fibonacci.pl
perlrs examples/text_processing.pl
perlrs examples/parallel_demo.pl
```

## License

MIT
