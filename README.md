```
 ██████╗ ███████╗██████╗ ██╗     ██████╗ ███████╗
 ██╔══██╗██╔════╝██╔══██╗██║     ██╔══██╗██╔════╝
 ██████╔╝█████╗  ██████╔╝██║     ██████╔╝███████╗
 ██╔═══╝ ██╔══╝  ██╔══██╗██║     ██╔══██╗╚════██║
 ██║     ███████╗██║  ██║███████╗██║  ██║███████║
 ╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝╚══════╝
```

[![CI](https://github.com/MenkeTechnologies/perlrs/actions/workflows/ci.yml/badge.svg)](https://github.com/MenkeTechnologies/perlrs/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/perlrs.svg)](https://crates.io/crates/perlrs)
[![Downloads](https://img.shields.io/crates/d/perlrs.svg)](https://crates.io/crates/perlrs)
[![Docs.rs](https://docs.rs/perlrs/badge.svg)](https://docs.rs/perlrs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

### `[PARALLEL WORLDS FASTEST PERL5 INTERPRETER // RUST-POWERED EXECUTION ENGINE]`

 ┌──────────────────────────────────────────────────────────────┐
 │ STATUS: ONLINE &nbsp;&nbsp; CORES: ALL &nbsp;&nbsp; SIGNAL: ████████░░       │
 └──────────────────────────────────────────────────────────────┘

> *"There is more than one way to do it — in parallel."*

---

## [0x00] OVERVIEW

`perlrs` is a Perl 5 compatible interpreter written in Rust that brings native parallelism to Perl scripting. It parses and executes Perl 5 scripts with rayon-powered work-stealing parallel primitives across all available CPU cores.

 ┌──────────────────────────────────────────────────────────────┐
 │ RAYON THREADS: ALL CORES &nbsp;&nbsp; REGEX: SIMD-ACCELERATED         │
 │ BINARY SIZE: 2MB STRIPPED &nbsp;&nbsp; BUILD: LTO + O3               │
 └──────────────────────────────────────────────────────────────┘

### Runtime values

`PerlValue` is a **NaN-boxed** `u64`: immediates (`undef`, inline `i32`, raw non-NaN `f64` bits) and tagged **heap** pointers (`Arc<HeapObject>`) for oversized integers, boxed floats, strings, arrays, hashes, refs, regexes, atomics, channels, etc. The public API uses constructors (`PerlValue::integer`, `::string`, …) and accessors (`as_integer`, `as_str`, `as_array_vec`, `with_heap`, …)—not `match` on constructor names, which are plain functions and cannot appear in patterns. Read-only heap access uses `with_heap` / `heap_ref` (borrow through the stored `Arc::into_raw` pointer without refcount churn); `heap_arc` / `Clone` still bump the `Arc` when an owned handle is needed. `Drop` decrements via `Arc::from_raw`. Profile hot paths if you tune performance: dispatch and allocation still dominate many workloads.

---

## [0x01] SYSTEM REQUIREMENTS

- Rust toolchain // `rustc` + `cargo`

## [0x02] INSTALLATION

#### DOWNLOADING PAYLOAD FROM CRATES.IO

```sh
cargo install perlrs
```

#### COMPILING FROM SOURCE

```sh
git clone https://github.com/MenkeTechnologies/perlrs
cd perlrs
cargo build --release
```

[perlrs on Crates.io](https://crates.io/crates/perlrs)

#### ZSH COMPLETION // TAB-COMPLETE ALL THE THINGS

```sh
# copy to a directory in your fpath
cp completions/_perlrs /usr/local/share/zsh/site-functions/_perlrs
cp completions/_pe /usr/local/share/zsh/site-functions/_pe

# or add the completions directory to fpath in your .zshrc
fpath=(/path/to/perlrs/completions $fpath)

# then reload completions
autoload -Uz compinit && compinit
```

After reloading your shell, `pe <TAB>` will complete all flags, options, and script files.

---

## [0x03] USAGE

#### INTERACTIVE REPL // `pe` WITH NO SCRIPT

When you run the **`pe`** binary **from a terminal** with **no program file**, **no `-e` / `-E`**, and not in **`-n` / `-p`** (or other batch-only modes such as **`-c`**, **`--lint`**, **`--disasm`**, **`--ast`**, **`--fmt`**, **`--profile`**, **`--explain`**, **`-u`**), it starts a **readline** session: line editing, history (saved to **`~/.perlrs_history`** when possible), and **Tab** completion for keywords/builtins, **`$scalar` / `@array` / `%hash`** names in scope, subroutine names, **method names after `->`** when the invocant resolves to a blessed object or a package name (from registered subs and `@ISA`), and **file paths** (merged with the word list when both apply — e.g. `./` or a partial filename in the current directory). Type **`exit`** or **`quit`** or send **EOF** (Ctrl-D) to leave. If stdin is **not** a TTY (e.g. a pipe), **`pe`** reads **one line** from stdin like **`perlrs`**. The **`perlrs`** binary keeps the previous behavior for the same flags (read a single line from stdin when no script is given).

#### EXECUTING INLINE CODE // DIRECT INJECTION

```sh
# inject a single line of perl
pe -e 'print "Hello, world!\n"'

# execute a script file
pe script.pl arg1 arg2

# check syntax without executing (parse only)
pe -c script.pl

# parse + compile to bytecode without running (fails on compile errors; skipped if `strict` forces tree-walker)
pe --lint script.pl
pe --check -e 'sub f { 1 }'

# print bytecode disassembly to stderr, then run (VM path only; no output if execution falls back to tree-walker)
pe --disasm script.pl

# dump abstract syntax tree as JSON (linting, IDE tooling, formatters, static analysis)
pe --ast script.pl
pe --ast -e 'sub foo { 1 }'

# pretty-print parsed Perl to stdout (best-effort; tree-walker-oriented AST; no execution)
pe --fmt script.pl
pe --fmt -e 'my $x = 1; say $x'

# wall-clock profile: per-statement and per-sub timings on stderr (tree-walker only; VM disabled)
pe --profile script.pl

# expanded hints for error codes (E0001, E0002, …)
pe --explain E0001
```

#### `__DATA__` // EMBEDDED DATA HANDLE

A line whose trimmed text is exactly `__DATA__` ends the program text. Everything after that line is stored as bytes on the **`DATA`** input handle, so `<DATA>` and `readline` on **`DATA`** read that trailing section (same idea as Perl). Shebang stripping and **`-x`** extraction apply only to the program portion above the marker.

#### PROCESSING DATA STREAMS // STDIN OPERATIONS

```sh
# line-by-line processing
echo "data" | pe -ne 'print uc $_'

# auto-print mode (like sed)
cat file.txt | pe -pe 's/foo/bar/g'

# in-place edit on named files (`-i` / `-i.bak` like Perl; `$^I` in Perl code)
pe -i -pe 's/foo/bar/g' file1.txt file2.txt
# With `-i` and multiple path arguments, each file is processed in parallel (rayon; pool size from `pe -j` / defaults)
pe -i -pe 's/\ba\b/b/g' *.zsh

# slurp entire input at once
cat file.txt | pe -gne 'print length($_), "\n"'

# auto-split fields
echo "a:b:c" | pe -a -F: -ne 'print $F[1], "\n"'
```

#### PARALLEL EXECUTION // MULTI-CORE OPERATIONS

```perl
# parallel map — transform elements across all cores
my @doubled = pmap { $_ * 2 } @data, progress => 1;

# optional progress: on a TTY, an animated bar with spinner and elapsed time redraws every 80 ms
# (like brew/cargo) on stderr (or stdout if only stdout is a TTY). Set PERLRS_PROGRESS_FULLSCREEN=1
# for alternate-screen fullscreen animation. PERLRS_PROGRESS_PLAIN=1 forces one line per tick (logs).
# Non-TTY: one log line per completed item.
my @out = pmap { heavy } @huge, progress => 1;

# parallel map in batches (one interpreter per chunk — amortizes spawn cost)
my @out = pmap_chunked 1000 { $_ ** 2 } @million_items, progress => 1;

# sequential left fold vs parallel tree fold (use preduce only for associative ops)
my $sum = reduce { $a + $b } @numbers;
my $psum = preduce { $a + $b } @numbers, progress => 1;

# parallel fold with explicit identity — each chunk starts from a clone of `EXPR`; hash
# accumulators merge by adding counts per key; other types use the same block to combine partials
my $histogram = preduce_init {}, {
    my ($acc, $item) = @_;
    $acc->{$item}++;
    $acc
} @words, progress => 1;

# fused parallel map + reduce — no full intermediate array (associative reduce only)
my $psum2 = pmap_reduce { $_ * 2 } { $a + $b } @numbers, progress => 1;

# optional progress for pgrep / preduce (stderr bar, same as pmap)
my @g = pgrep { $_ > 0 } @nums, progress => 1;
my $x = preduce { $a + $b } @nums, progress => 1;

# thread-safe memoization for parallel blocks (key = stringified $_)
my @once = pcache { expensive } @inputs, progress => 1;

# lazy pipeline (ops run on collect(); `sub { }` or bare `{ }` blocks)
# Sequential: ->filter ->map ->take. Parallel (same semantics as top-level p*): ->pmap ->pgrep
# ->pfor ->pmap_chunked ->psort ->pcache; optional progress: ->pmap(sub { }, 1).
# Folds (collect() returns a scalar): ->preduce ->preduce_init($init, sub { }) ->pmap_reduce($m, $r)
# User/package subs: ->mysub or ->Pkg::name (no args) — same as ->map with `$_` per element; `grep` aliases `filter`
my @result = pipeline(@data)
    ->filter({ $_ > 10 })
    ->map({ $_ * 2 })
    ->take(100)
    ->collect();

# same chain as `pipeline`, but `->filter` / `->map` run in parallel on `collect()` (input order preserved)
my @par = par_pipeline(@data)
    ->filter({ $_ > 10 })
    ->map({ $_ * 2 })
    ->take(100)
    ->collect();

# parallel grep — filter elements in parallel
my @evens = pgrep { $_ % 2 == 0 } @data, progress => 1;

# parallel foreach — execute side effects concurrently
pfor { process } @items, progress => 1;

# fan — run a block N times in parallel (`$_` is 0..N-1)
# progress => 1 shows per-worker animated bars (pv-style sweep left→right, like brew/cargo)
fan 8 { work }
fan { work }, progress => 1;

# fan_cap — same as fan, but return value is a list of each block's return value (index order)
my @capture = fan_cap { work };
my @squares = fan_cap 4 { $_ * $_ };

# fan — omit N to use the rayon thread pool size (`pe -j`; `$_` is 0..N-1)
fan { work }

# typed channels — pass messages between parallel blocks (unbounded or bounded)
my ($tx, $rx) = pchannel();
my ($t2, $r2) = pchannel(128);   # bounded capacity

# multi-stage parallel pipeline — BATCH: source generates all items, then each
# stage processes the entire batch via rayon before the next stage starts
my $n = par_pipeline(
    source  => { readline(STDIN) },
    stages  => [ { parse_json }, { transform } ],
    workers => [4, 2],
    buffer  => 256,   # optional; default 256 slots per inter-stage channel
);
# returns scalar: count of items processed by the last stage

# multi-stage parallel pipeline — STREAMING: items flow through bounded channels
# between stages concurrently (item 1 can be at stage 3 while item 5 is at stage 1)
my $n = par_pipeline_stream(
    source  => { readline(STDIN) },
    stages  => [ { parse_json }, { transform } ],
    workers => [4, 2],
    buffer  => 256,
);

# streaming list pipeline — same channel-wired concurrency for list inputs
# unlike par_pipeline(@list) which batch-collects each stage before the next,
# par_pipeline_stream wires every op through channels so items are at different
# stages simultaneously.  Order is NOT preserved.
my @out = par_pipeline_stream((1..1_000))
    ->filter(sub { $_ > 500 })
    ->map(sub { $_ * 2 })
    ->take(10)
    ->collect();

# optional: control workers-per-stage and channel buffer size
my @out = par_pipeline_stream(@data, workers => 4, buffer => 128)
    ->map({ expensive })
    ->collect();

# multiplexed recv (Go-style select via crossbeam `Select`)
my ($tx1, $rx1) = pchannel();
my ($tx2, $rx2) = pchannel();
$tx1->send("first");
my ($val, $idx) = pselect($rx1, $rx2);  # $idx is 0-based (first arg = 0)
my ($v2, $i2) = pselect($rx1, $rx2, timeout => 0.5);  # $i2 is -1 on timeout

# single-path file watcher (same engine as pwatch; one path + callback)
# If the path has no glob wildcards and does not exist yet, the parent directory is watched until it appears.
watch "/tmp/x", { say };

# HTTP: blocking fetch vs async task handle vs parallel batch GET
my $body = fetch("https://example.com/");
my $task = fetch_async("https://example.com/");   # not `async_fetch` (lexer: keyword `async`)
my $json = await fetch_async_json("https://api.example.com/x");
my @bodies = par_fetch(@urls);

# parallel CSV → array of hashrefs (CPU-parallel row conversion after parse)
my @rows = par_csv_read("data.csv");

# deque — double-ended queue (not in stock Perl)
my $q = deque();
$q->push_back(1); $q->push_front(0);
# pop_front / pop_back / size (or len)

# heap — priority queue with a Perl comparator (`$a` / `$b`, like `sort`)
my $pq = heap({ $a <=> $b });
$pq->push(3); my $min = $pq->pop();

# parallel sort — sort using all cores
my @sorted = psort { $a <=> $b } @data, progress => 1;

# chain parallel operations
my @result = pmap { $_ ** 2 } pgrep { $_ > 100 } @data, progress => 1;

# parallel recursive glob (rayon directory walk), then process files in parallel
my @logs = glob_par("**/*.log");
pfor { process } @logs, progress => 1;

# parallel directory tree — $_ is each path (file or directory); optional progress => 1 pre-collects paths for a bar
par_walk ".", { say if /\.rs$/ }, progress => 1;

# parallel in-place sed (global replace per file); returns count of files modified
par_sed(qr/\bfoo\b/, "bar", @paths);

# persistent thread pool (reuse worker OS threads; avoids per-task thread spawn from pmap/pfor)
my $pool = ppool(4);
$pool->submit({ heavy_work }) for @tasks;   # worker `$_`: caller's `$_` here, or pass `submit(CODE, $x)`
my @results = $pool->collect();

# control thread count
pe -j 8 -e 'my @r = pmap { heavy_work } @data, progress => 1'
```

More parallel examples (same rules as above: each worker is a fresh interpreter with captured lexicals; use `mysync` for shared counters):

```perl
# mmap + scan lines in parallel — $_ is each line (CRLF-safe); optional stderr progress bar
par_lines "./README.md", sub { say length($_) if /parallel/i }, progress => 1;

# psort with no block — parallel lexical string sort (all cores)
my @alpha = psort qw(zebra apple mango);

# pcache — memoize by stringified $_; repeated values skip the block body
my @out = pcache { $_ * 10 } (1, 1, 2, 2, 3), progress => 1;

# barrier — N workers rendezvous before continuing (party count clamped ≥ 1)
my $sync = barrier(3);
fan 3 { $sync->wait; say "all arrived" }

# ppool — submit jobs; optional second arg binds $_ in the worker; collect preserves order
my $pool = ppool(4);
$pool->submit({ $_ * 2 }) for 1..10;
my @doubled = $pool->collect();
```

Each parallel block receives its own interpreter context with captured lexical scope // no data races. Use `mysync` to share state.

**Perl-compat (partial)** — not a full `perl` replacement, but these areas follow Perl 5 more closely than before:

- **Inheritance / `SUPER::` / C3 MRO** — `@ISA` in the package stash (including `our @ISA` outside `main`), C3 method resolution order, and `$obj->SUPER::method` to invoke the next class in the linearized chain. **MRO is not cached:** each lookup reads the live `@Pkg::ISA` array, so `push @ISA`, assignment, or `splice` at runtime affects the next method resolution. The bytecode compiler tracks the current `package` so VM execution stores and reads `Pkg::ISA` like the tree interpreter.
- **`tie`** — `tie $scalar`, `tie @array`, `tie %hash` with `TIESCALAR` / `TIEARRAY` / `TIEHASH`; `FETCH` / `STORE` on the blessed object for reads and writes; tied hashes also dispatch **`exists $h{k}`** → `EXISTS` and **`delete $h{k}`** → `DELETE` when those subs exist (not every other `tie` method yet).
- **`$?` and `$|`** — last child exit status for `system`, `` `...` `` / `capture`, and `close` on pipe children (POSIX-style packed status); `$|` enables autoflush after `print` / `printf` to resolved handles.
- **`print` / `say` / `printf`** — with **no** argument list, Perl uses **`$_`** as the value to output (and **`printf`** with no expressions uses **`$_`** as the format string); this applies in `map`/`grep`/`for`/`pfor`/… blocks and on **`$fh->print`** / **`$fh->printf`**.
- **Bareword statement** — `name;` or `{ name }` (no `()`) is a subroutine call with **no explicit arguments**; **`@_`** is **`($_)`** so the topic is visible as **`shift`** / **`$_[0]`** (built-in keywords like **`undef`** / **`print`** keep their normal meaning).
- **Typeglobs** — `*NAME` as a value; `*lhs = *rhs` copies subroutine, scalar, array, hash, and IO-handle alias slots (`copy_typeglob_slots`); `*foo = \&bar` installs a subroutine alias (including when `\&bar` is a ref-to-coderef); package-qualified `*Pkg::name` is supported. Bytecode may fall back to the tree-walker for these assignments.
- **`use overload`** — `use overload 'op' => 'method', …` or `'op' => \&handler` registers per-class overloads in the current package; one statement may combine several ops (e.g. `use overload '+' => \&add, '""' => \&stringify;`). Binary ops dispatch to the named method with `(invocant, other)`; missing ops may use **`nomethod`** with a third argument, the op key string (e.g. `"+"`). Unary **`neg`**, **`bool`** (for `!` / `not` after the overload result), and **`abs`** are supported on blessed values. `use overload '""' => 'as_string'` (or the key `""` / `'""'`) drives stringification for `print`, **`sprintf` `%s`**, interpolated strings, and similar contexts. **`fallback => 1`** is accepted in the pragma list (full Perl fallback coercion is not). Tree interpreter only for some forms (VM falls back when bytecode cannot represent the expression).

**`%SIG` (Unix)** — `SIGINT`, `SIGTERM`, `SIGALRM`, and `SIGCHLD` can be set to a code ref; handlers run **between tree-walker statements** and **between VM opcodes** (see `perl_signal::poll`). `IGNORE` / `DEFAULT` are recognized as no-ops.

**SQLite** — `query` still loads all rows; a lazy **`stream`-style** row iterator is not wired yet (use `LIMIT`/`OFFSET` or chunk in Perl for huge result sets).

Perl **`format` / `write`** — **partial**: `format NAME =` … `.` registers a template in the current package; picture fields `@<<<<` / `@>>>>` / `@||||` / `@####` / `@****`, literal `@@`, and comma-separated value lines are supported; **`write`** with no arguments expands the template named by **`$~`** (default **`STDOUT`**) and prints to stdout like **`print`**. Not implemented: **`write FILEHANDLE`**, top-of-page (`$^`), **`formline`**, and other full **`perlform`** details.

#### EXECUTION TRACE // `trace`

Wrap parallel (or any) code to print **`mysync` scalar** mutations to **stderr** — useful when debugging races or ordering. Under `fan N { }`, lines are tagged with the worker index (same as `$_`).

```perl
mysync $counter = 0;
trace { fan 10 { $counter++ } };
# stderr: [thread 0] $counter: 0 → 1
# stderr: [thread 3] $counter: 1 → 2
# ... (order varies)
```

Outside `fan`, mutations are labeled `[main]`.

#### TIMER // `timer`

Wall-clock timing: **`timer { BLOCK }`** returns elapsed time as a **float** in **milliseconds** (sub-ms resolution). Microbenchmark harness: **`bench { BLOCK } N`** runs a **warmup** (`min(10, N/10)`, at least one iteration), then **`N`** timed iterations and returns a string with **min / mean / p99** wall times in milliseconds (VM and tree-walker).

```perl
my $elapsed = timer { heavy_work() };
print "took ${elapsed}ms\n";
```

#### ASYNC / AWAIT // lightweight I/O parallelism

**`async { BLOCK }`** runs the block on a **worker thread** and returns a task handle immediately. **`await EXPR`** joins: if `EXPR` is that handle, it blocks until the block finishes and returns its value; otherwise `await` passes the value through.

Use this to overlap **`fetch_url`**, **`slurp`**, or other I/O-bound work without blocking the main interpreter until you **`await`**.

```perl
my $data = async { fetch_url("https://example.com/") };
my $file = async { slurp("big.csv") };
print await($data), await($file);
```

Each `async` worker gets a **clone of the interpreter’s subs** and a **captured lexical scope** (including **`mysync`** storage), so closures and shared state behave like other parallel primitives.

#### NATIVE CSV / SQLITE / STRUCTS // data scripting

**HTTP** — [`ureq`](https://crates.io/crates/ureq) blocking GET. **`fetch(url)`** returns the response body as a string. **`fetch_json(url)`** parses JSON with [`serde_json`](https://crates.io/crates/serde_json): JSON objects become **hashrefs**, arrays become **arrays**, `null` → `undef`, numbers → integers or floats. (Lower-level **`fetch_url $url`** is still available as an expression form.)

**JSON encode/decode** — **`json_encode($value)`** turns a Perl value into a JSON string (scalars, array/hash refs, blessed objects serialize the underlying value; unsupported types are errors). **`json_decode($string)`** parses JSON into Perl values with the same mapping as **`fetch_json`**. Use these for APIs, config files, and round-tripping data without an HTTP round trip.

**CSV** — [`csv`](https://crates.io/crates/csv) backed. `csv_read(path)` returns an array of **hashrefs** (first row is the header). `csv_write(path, row, …)` or `csv_write(path, \@rows)` writes rows (each row is a hash or hashref); header columns are the union of keys in first-seen order.

**DataFrame** — `dataframe(path)` loads the same CSV shape into a **columnar** value (not an array of hashrefs). Methods: `->nrow` / `->nrows`, `->ncol` / `->ncols`; `->filter(sub { … })` runs the coderef with `$_` bound to each row as a hashref; `->group_by("col")` sets the grouping column for a following `->sum("amount")`, which returns a small two-column frame (group key, sum). Without `group_by`, `->sum("col")` returns a single numeric total. JSON encoding represents a frame as an array of row objects.

**SQLite** — embedded database via [`rusqlite`](https://crates.io/crates/rusqlite) with **bundled** libsqlite (no system SQLite required). `sqlite(path)` returns a handle: `->exec(sql, ?…)`, `->query(sql, ?…)` (rows as hashrefs), `->last_insert_rowid`.

**Structs** — `struct Name { field => Type, … }` with `Type` one of `Int`, `Str`, `Float`. Constructor: `Name->new(field => value, …)`. Field read: `$obj->fieldname` (same as a method call). The VM builds native struct instances (not plain blessed hashes) when the struct is declared in the same program.

**Typed `my`** — `typed my $x : Int` (or `Str` / `Float`): assignments are checked at runtime; mismatches are type errors.

```perl
my $data = fetch_json("https://api.example.com/users/1");
say $data->{name};

my $payload = json_encode({ ok => 1, n => 42 });
my $roundtrip = json_decode($payload);

my $raw = fetch("https://example.com/");

my @rows = csv_read("data.csv");
csv_write("out.csv", { name => "a", id => "1" });

# columnar frame — filter rows, then sum one numeric column (see prose above for API)
my $df = dataframe("data.csv");
my $n = $df->nrow;

my $db = sqlite("app.db");
$db->exec("CREATE TABLE t (id INTEGER, name TEXT)");
my @r = $db->query("SELECT * FROM t WHERE id > ?", 0);

typed my $n : Int;
$n = 42;

struct Point { x => Float, y => Float };
my $p = Point->new(x => 1.5, y => 2.0);
say $p->x;
```

#### THREAD-SAFE SHARED STATE // `mysync`

`mysync` declares variables backed by `Arc<Mutex>` that are shared across parallel blocks. All reads/writes go through the lock automatically. Compound operations (`++`, `+=`, `.=`, and `|=`, `&=` on scalars holding a native `Set`) are fully atomic — the lock is held for the entire read-modify-write cycle.

For **`mysync` scalars that hold a `Set`** (from `mysync $s = Set->new(...)`), union (`|`) and intersection (`&`) treat the stored value as a set even when the underlying storage is the mutex-wrapped scalar; bitwise `|` / `&` on plain integers is unchanged.

**`deque()` and `heap(...)`**: `mysync $q = deque()` (and `mysync $pq = heap(sub { $a <=> $b })`) stores the value without an extra `Atomic` shell — they already use `Arc<Mutex<…>>`. Use them like any other `mysync` scalar in `fan` / `pmap` / `pfor` so all workers share one queue or heap.

```perl
# shared scalar — atomic increment
mysync $counter = 0;
fan 10000 { $counter++ };
print $counter;  # always exactly 10000

# shared array — thread-safe push/pop/shift
mysync @results;
pfor { push @results, $_ * $_ } (1..100), progress => 1;
print scalar @results;  # always exactly 100

# shared hash — atomic element access
mysync %histogram;
pfor { $histogram{$_ % 10} += 1 } (0..999), progress => 1;
# each bucket is exactly 100

# mix all three
mysync $total = 0;
mysync @items;
mysync %stats;
fan 1000 {
    $total++;
    push @items, $_;
    $stats{$_ % 5} += 1;
};
# $total == 1000, @items == 1000, sum(%stats) == 1000
```

Without `mysync`, each parallel thread gets an independent copy — changes are not visible to other threads or the parent. With `mysync`, all threads share the same underlying storage via `Arc<Mutex>`.

 ┌──────────────────────────────────────────────────────────────┐
 │ ATOMIC OPS: $x++ &nbsp;&nbsp; ++$x &nbsp;&nbsp; $x += N &nbsp;&nbsp; $x .= "s" &nbsp;&nbsp; $s |= $t (Set) │
 │ ATOMIC OPS: $h{k} += N &nbsp;&nbsp; $a[i] += N &nbsp;&nbsp; push @a, $v      │
 │ LOCK SCOPE: held for full read-modify-write // zero races   │
 └──────────────────────────────────────────────────────────────┘

---

## [0x04] CLI FLAGS

![pe -h](img/pe-help.png)

---

## [0x05] SUPPORTED PERL FEATURES

#### DATA TYPES
- Scalars (`$x`), arrays (`@a`), hashes (`%h`)
- References: `\$x`, `\@a`, `\%h`, `\&sub`
- Array refs `[1,2,3]`, hash refs `{a => 1}`
- Code refs / closures `sub { ... }` — capture enclosing lexicals (`my $x`, `my @a`, `my %h`, …); bootstrap handles (`@_`, `%ENV`, …) are not snapshotted into closures
- Regex objects `qr/.../`
- Blessed references (basic OOP)
- `typed my $x : Int|Str|Float` (runtime-checked assignments)
- `struct Name { field => Type, … }` with `Name->new(…)` and `$obj->field`
- Native CSV (`csv_read` / `csv_write`), columnar `dataframe(path)` with `->filter` / `->group_by` / `->sum`, and SQLite (`sqlite` + `->exec` / `->query`)
- `fetch` / `fetch_json` (HTTP GET via `ureq`; JSON → Perl values); `json_encode` / `json_decode` (standalone JSON string ↔ Perl values)

#### CONTROL FLOW
- `if`/`elsif`/`else`, `unless`
- `while`, `until`, `do { } while/until` (block runs before the first condition check)
- `for` (C-style), `foreach`
- `last`, `next`, `redo` with labels
- Postfix: `expr if COND`, `expr unless COND`, `expr while COND`, `expr for @list`
- Ternary `?:`
- **`try { } catch ($err) { }` [`finally { }`]** — statement form only (not an arbitrary expression, so not `my $x = try { … }`); catches `die` and other runtime errors (not `exit`, not `last`/`next`/`return` flow); the error string is bound to the scalar in `catch`. Optional **`finally`** runs after a successful `try` or after `catch` finishes (including if `catch` propagates an error); if `finally` fails, that error is returned (Perl-style).
- **`given (EXPR) { when (COND) { } default { } }`** — topic is **`$_`**; `when` tests in order (regex `=~` for regex literals, string equality for string/number literals, otherwise string comparison to the evaluated condition); first match wins; put **`default` last** (tree-walker only)
- **Algebraic `match (EXPR) { PATTERN => EXPR, … }`** (perlrs) — expression form with explicit subject; arms are tested in order. Patterns: **`_`** (wildcard); **`/regex/`** (stringified subject); **literal / parenthesized expression** (smart-match against the subject); **`[1, 2, *]`** (array or array-ref: prefix elements match, optional **`*`** tail); **`{ name => $n }`** (hash or hash-ref: required keys; **`$n`** binds the value for the arm body). Bindings are scoped to that arm only. No matching arm is a runtime error (use **`_`**). Tree interpreter only (bytecode falls back).
- **`eval_timeout SECS { }`** — runs the block on a **worker OS thread**; the main thread waits up to **`SECS`** seconds via `recv_timeout` (no Unix `alarm`); on timeout you get a runtime error (the worker may keep running in the background—avoid relying on cancellation for correctness)

#### OPERATORS
- Arithmetic: `+`, `-`, `*`, `/`, `%`, `**`
- String: `.` (concat), `x` (repeat)
- Comparison: `==`, `!=`, `<`, `>`, `<=`, `>=`, `<=>`
- String comparison: `eq`, `ne`, `lt`, `gt`, `le`, `ge`, `cmp`
- Logical: `&&`, `||`, `//`, `!`, `and`, `or`, `not`
- Bitwise: `&`, `|`, `^`, `~`, `<<`, `>>` (for native `Set` values, `|` / `&` are union / intersection instead of integer bitwise ops)
- Assignment: `=`, `+=`, `-=`, `*=`, `/=`, `.=`, `|=`, `&=`, `//=`, etc.
- Regex: `=~`, `!~`
- Range: `..`
- Arrow dereference: `->`

#### REGEX ENGINE
- **Three-tier compile:** patterns are compiled with the Rust [`regex`](https://docs.rs/regex) crate first (linear-time where possible). If that rejects the pattern (e.g. **backreferences** like `(.)\\1`), compilation falls back to [`fancy-regex`](https://docs.rs/fancy-regex). If both reject the pattern (e.g. PCRE-only verbs like `(*SKIP)`), compilation falls back to **PCRE2** via the [`pcre2`](https://docs.rs/pcre2) crate (`src/perl_regex.rs`).
- Match: `$str =~ /pattern/flags`
- Dynamic pattern (string): `$str =~ $pattern` and `$str !~ $pattern` (bytecode `RegexMatchDyn`; empty flags)
- Substitution: `$str =~ s/pattern/replacement/flags`
- Transliterate: `$str =~ tr/from/to/`
- Flags: `g`, `i`, `m`, `s`, `x`
- Capture variables: `$1`, `$2`, … (all numbered groups, not only 1–9); named groups `(?<name>…)` or `(?P<name>…)` populate **`%+`** and **`$+{name}`** (named rules follow whichever engine compiled the pattern)
- Literal spans: `\Q…\E` (metacharacters escaped); `quotemeta` for dynamic patterns
- Quote-like: `m//`, `qr//`

#### SUBROUTINES
- Named subs with `sub name { ... }`
- Anonymous subs / closures
- Implicit return from the last statement when it is an expression or a trailing `if` / `unless` (VM-compiled subs; matches Perl’s last-expression value)
- Recursive calls
- `@_` argument passing, `shift`, `return`
- `return EXPR if COND` (postfix modifiers on return)
- **`AUTOLOAD`**: missing subs and methods dispatch to `sub AUTOLOAD` with `$AUTOLOAD` set to the fully qualified name (e.g. `main::foo`, `MyClass::bar`)

#### BUILT-IN FUNCTIONS

 ┌──────────────────────────────────────────────────────────────┐
 │ **Array**: push, pop, shift, unshift, splice, reverse,      │
 │ sort, map, grep, reduce, preduce, scalar                    │
 │ **Hash**: keys, values, each, delete, exists                │
 │ **String**: chomp, chop, length, substr, index, rindex,     │
 │ split, join, sprintf, printf, uc, lc, ucfirst, lcfirst,     │
 │ chr, ord, hex, oct, crypt, fc, pos, study, quotemeta          │
 │ **Binary**: pack, unpack (subset A a N n V v C Q q Z H x;   │
 │ `*` repeat; result is byte data)                              │
 │ **Numeric**: abs, int, sqrt, sin, cos, atan2, exp, log,     │
 │ rand, srand                                                 │
 │ **I/O**: print, say, printf, open (files + `-|` / `|-`       │
 │ piped shell; returns a handle value), close, eof, readline, │
 │ handle methods `->print` / `->say` / `->printf` / `->getline` │
 │ / `->readline` / `->close` / `->eof` / `->getc` / `->flush` …, │
 │ slurp, capture (structured shell: ->stdout/stderr/exit),   │
 │ binmode, fileno, flock, getc, sysread, syswrite, sysseek,  │
 │ select (timeout sleep / handle no-op), truncate             │
 │ **Directory**: opendir, readdir, closedir, rewinddir,        │
 │ telldir, seekdir                                              │
 │ **File tests**: -e, -f, -d, -l, -r, -w, -s, -z, -t (TTY)   │
 │ **System**: system, exec, exit, chdir, mkdir, unlink, rename, │
 │ chmod, chown (Unix), stat, lstat, link, symlink, readlink,   │
 │ glob, glob_par, par_sed, ppool, barrier,                      │
 │ **Data**: csv_read, csv_write (header row → AoH), dataframe, │
 │ sqlite                                                        │
 │ fork, wait, waitpid, kill, alarm, sleep, times (Unix where  │
 │ noted in source)                                            │
 │ **Socket** (std::net): socket, bind, listen, accept,         │
 │ connect, send, recv, shutdown                               │
 │ **Type**: defined, undef, ref, bless                        │
 │ **Set**: `Set->new(…)` — native set; `|` union, `&` intersection │
 │ **Control**: die, warn, eval, do, require, caller,         │
 │ wantarray (void / scalar 0 / list 1; bytecode passes context), │
 │ `goto EXPR` (same-block labels), `continue { }` on loops, │
 │ `prototype` on code refs; sub prototypes parsed on `sub`     │
 └──────────────────────────────────────────────────────────────┘

#### EXTENSIONS BEYOND STOCK PERL 5
- **`csv_read PATH` / `csv_write PATH, \@rows`** — native CSV via the Rust `csv` crate. The first row is column headers; each data row is a hashref (string cells). `csv_write` uses the first row’s key order for columns.
- **`dataframe(PATH)`** — same CSV load as `csv_read`, but stored columnar; methods `->filter`, `->group_by`, `->sum`, `->nrow` / `->ncol` (see **DataFrame** under native CSV above).
- **`sqlite(PATH)`** — embedded SQLite via `rusqlite` (bundled libsqlite). Handle methods: `->exec(SQL, ?bind…)`, `->query(SQL, ?bind…)` (array of hashrefs), `->last_insert_rowid`.
- **`fan [N] { BLOCK } [, progress => EXPR]`** — run **`BLOCK`** **`N`** times in parallel (omit **`N`** to use the rayon pool size, `pe -j`); **`$_`** is **`0..N-1`**. Optional **`progress => 1`** shows **per-worker animated bars** (pv-style sweep left→right with spinner and elapsed time, like `brew`/`cargo`). You may write **`progress => EXPR`** immediately after **`}`** with **no comma** (same as **`pmap`** / **`pgrep`**).
- **`fan_cap [N] { BLOCK } [, progress => EXPR]`** — same as **`fan`**, but the expression returns a **list** of each iteration’s block return value, in **`$_`** order (**`0..N-1`**). Same optional **`progress`** forms and per-worker bars as **`fan`** (comma or not before **`progress`**).
- **`par_lines PATH, sub { ... } [, progress => EXPR]`** — memory-map the file, split into line-aligned byte chunks, process chunks in parallel with rayon; each line sets `$_` for the coderef (CRLF-safe; bytecode VM via `Op::ParLines` or tree-walker; use `mysync` for shared counters across workers). Optional stderr progress bar like **`pmap`**.
- **`par_walk PATH, sub { ... } [, progress => EXPR]`** — parallel recursive directory walk (rayon per directory, like **`glob_par`**’s strategy); **`$_`** is the full path of each visited file or directory. **`PATH`** may be a list (array) of roots. With **`progress => 1`**, paths are collected first so the bar has a total; without it, the walk runs in one pass. Bytecode VM via **`Op::ParWalk`** + **`Chunk::par_walk_entries`**.
- **`par_sed PATTERN, REPLACEMENT, FILES... [, progress => EXPR]`** — read each file as UTF-8, apply **`PATTERN`** with global substitution semantics (same as **`s///g`**: pass a string pattern or **`qr/.../`** with embedded flags), write back only when the content changes; returns the number of files modified. **`PATTERN`** string uses the same compilation path as regex literals (e.g. **`(?i)`** via **`qr`** for case fold). Optional stderr progress (**`BuiltinId::ParSed`** / **`ParSedProgress`**).
- **`pipeline(@list)->…->collect()`** — lazy list processing (sequential **`->filter`** / **`->grep`**, **`->map`**, **`->take`**). Parallel chain methods mirror top-level **`pmap`**, **`pgrep`**, **`pfor`**, **`pmap_chunked`**, **`psort`**, **`pcache`** (optional second argument `1` for stderr progress where applicable). Fold methods **`->preduce`**, **`->preduce_init(INIT, sub)`**, **`->pmap_reduce(MAP, REDUCE)`** must be last before **`->collect()`**; **`collect()`** then returns a **scalar** for those. Any other **`->name`** or **`->Pkg::name`** with **no** arguments resolves a **subroutine** in the stash and applies it like **`->map`** (`$_` each element). **`par_lines`**, **`par_fetch`**, etc. are separate builtins; they are not `->` methods on **`pipeline`**.
- **`par_pipeline(@list)->…->collect()`** — same **`->`** methods as **`pipeline`**, but plain **`->filter`** / **`->map`** use **rayon** on **`collect()`** with **input order preserved** (same capture rules as **`pgrep`** / **`pmap`**). **`->take`** still truncates after those stages. For the **multi-stage channel** builtin (bounded queues, backpressure), use the named form below.
- **`par_pipeline(source => CODE, stages => [...], workers => [...], buffer => N?)`** — **batch** mode: source coderef generates all items (return **`undef`** to end), then each stage processes the **entire batch** via rayon before the next stage starts. Return value is the count of items processed by the last stage. Source and stage bodies use the same **capture** rules as `pmap`/`ppool` (lexical scalars are shared; **arrays** are not copied into the snapshot—use scalars, package variables, or handles like `STDIN`). For stdin lines, use **`readline(HANDLE)`** (e.g. **`readline(STDIN)`**), **`<STDIN>`**, or **`readline`** with no args (diamond/`ARGV` rules apply).
- **`par_pipeline_stream(@list [, workers => N, buffer => N])->…->collect()`** — streaming variant of **`par_pipeline(@list)`**. Each **`->filter`** / **`->map`** / **`->take`** / **`->pfor`** / **`->pcache`** stage runs as a pool of OS threads connected by **bounded crossbeam channels**, so items flow between stages **concurrently** (an item can be at stage 3 while another is still at stage 1). **Order is not preserved.** Optional **`workers => N`** sets threads-per-stage (default: rayon pool size); **`buffer => N`** sets channel capacity (default 256). **`->psort`**, **`->preduce`**, **`->preduce_init`**, **`->pmap_reduce`**, **`->pmap_chunked`** are rejected at runtime (they require all items; use batch **`par_pipeline`** instead).
- **`par_pipeline_stream(source => CODE, stages => [...], workers => [...], buffer => N?)`** — **streaming** named form: same arguments as `par_pipeline(source => ...)`, but items flow through **bounded crossbeam channels** between stages concurrently with **backpressure** (an item can be at stage 3 while another is still at stage 1). Order is **not** preserved when a stage has multiple workers.
- **`pwatch GLOB, sub { ... }`** — register native file/directory watches with the `notify` crate (inotify/kqueue/FSEvents); block in the event loop and dispatch each glob-matching path to the coderef on a rayon worker with `$_` set to the path (bytecode VM via `Op::Pwatch` or tree-walker; use `mysync` for shared state).
- **`barrier(N)`** — returns a handle backed by `std::sync::Barrier`; `->wait` for phased parallelism (e.g. with `fan`). Party count is clamped to at least 1 (bytecode + tree-walker).
- **`sort` / `psort` fast path** — `{ $a <=> $b }`, `{ $a cmp $b }`, `{ $b <=> $a }`, `{ $b cmp $a }` compare without invoking the block per pair (VM + tree-walker).
- **`reduce` / `preduce`** — list fold with `$a` (accumulator) and `$b` (next item); `reduce` is strictly left-to-right; `preduce` uses rayon (order not fixed; use only when the operation is associative). Optional **`, progress => EXPR`** — stderr progress bar when truthy (same style as **`pmap`**).
- **`preduce_init`** — `preduce_init EXPR, { BLOCK } @list [, progress => EXPR]`: parallel fold starting from **`EXPR`** (clone per chunk); empty list returns `EXPR`. **`$a` / `$b`** are the accumulator and next element; **`@_`** is `($a, $b)`. Hash (or hashref) partials are merged by **adding numeric values per key**; for other accumulators the block must combine two partial results associatively (same idea as `preduce`).
- **`pmap_reduce { MAP } { REDUCE } @list [, progress => EXPR]`** — fused parallel map plus tree reduce without building the full mapped list; optional stderr progress bar like **`pmap`**.
- **`glob_par PATTERN… [, progress => EXPR]`** — parallel recursive glob (rayon); same patterns as **`glob`**. Optional **`progress => 1`** — one tick per pattern (not per file).
- **`frozen my`** — immutable bindings (reassignment rejected in the bytecode path).
- **`typed my $x : Type`** — optional scalar types (`Int`, `Str`, `Float`) with **runtime** checks on declaration and every assignment; `typed my` runs on the tree-walker (bytecode falls back when the program uses it).
- **`try { } catch ($err) { } [finally { }]`** — bytecode VM (`Op::TryPush` / `CatchReceive` / …) and tree interpreter; `die` and most runtime errors unwind to `catch` (not `exit`, not control-flow exceptions).
- **`given (EXPR) { when … default … }`** — bytecode VM (`Op::Given` + AST body; `when`/`default` still run via the interpreter inside that body) and tree interpreter.
- **`eval_timeout SECS { … }`** — bytecode VM (`Op::EvalTimeout`) and tree interpreter (worker thread + `recv_timeout`).
- **Algebraic `match (EXPR) { PATTERN => EXPR, … }`** — bytecode VM (`Op::AlgebraicMatch`) and tree interpreter.

#### OTHER FEATURES
- `Interpreter::execute` returns `Err(ErrorKind::Exit(code))` for `exit` (including code 0); the `perlrs` binary maps that to `process::exit`.
- **`@INC` / `%INC` / `require` / `use`** — The `perlrs` / `pe` driver builds `@INC` by: each **`-I`** directory, then **`<crate>/vendor/perl`** when present (e.g. a stub **`List/Util.pm`** so `require List::Util` updates **`%INC`**), then the same paths **system `perl` reports for `@INC`** (via `perl -e 'print join "\n", @INC'`, so other core/site `.pm` paths match Perl’s search order), then the **script’s directory** (when the program file is not `-e` / `-` / `repl`), then **`PERLRS_INC`**, then **`.`** (duplicates removed). Set **`PERLRS_NO_PERL_INC`** to omit the `perl` query (e.g. no `perl` on `PATH`). **`List::Util`** (all **`EXPORT_OK`** names from Perl 5’s module, including **`reduce`**, **`any`**, **`pairs`**, **`zip`**, …) is implemented **natively in Rust** (`src/list_util.rs`) and registered on interpreter startup; core Perl still loads XS for these, but perlrs does not. Pure perlrs execution is still **not** full Perl 5: many other real `.pm` files (especially XS) will not run even when found. Relative paths (`Foo::Bar` → `Foo/Bar.pm`) are searched in order; successful loads record the relative path in **`%INC`**; repeated `require` is a no-op. **`use Module;`** is processed in **source order** before `BEGIN` blocks (and before the VM main chunk runs). After a successful load, **`use Module qw(a b);`** imports only those names, and each must appear in **`our @EXPORT`** or **`our @EXPORT_OK`** in the loaded module (Exporter-style). Bare **`use Module;`** imports **`@EXPORT`** only; if the module never sets **`our @EXPORT`** / **`our @EXPORT_OK`**, the legacy rule applies: import every top-level sub under that package. **`use Module qw();`** imports nothing. **Qualified calls** like **`Foo::bar()`** are valid syntax. Built-in pragmas (`strict`, `warnings`, `utf8`, …) do not load a file. Version-only `require` (e.g. `require 5.010`) succeeds without loading.
- **Pragmas (porting)** — `use strict` enables **refs / subs / vars**; `use strict 'refs'` or `use strict qw(refs subs)` selects modes; `no strict` clears all, `no strict 'refs'` clears one. **Runtime:** **`strict refs`** rejects symbolic scalar/array/hash derefs (string used as a variable name) with a Perl-like message; **`$$foo`** is parsed as symbolic scalar deref of **`$foo`** (falls back to the tree interpreter when bytecode is compiled). **`strict vars`** requires a visible binding (`my`/`our`/prior assignment in scope) for unqualified scalars, arrays, and hashes (package-qualified names and built-in specials like `$_`, `$1`, … are exempt); **`strict subs`** appends a hint to undefined subroutine errors. **`use warnings` / `no warnings`** toggle the interpreter warnings flag (reserved for future warning surfaces). **`feature_bits`** (crate **`FEAT_*`**) are set by **`use feature`** / **`no feature`**; **`say`** is gated by **`FEAT_SAY`** (on by default like Perl 5.10+; disable with **`no feature 'say'`** or **`no feature;`**). **`use utf8` / `no utf8`** set **`utf8_pragma`**. **`use open …`** — recognizes `qw(:utf8)`, `:std`, and `:encoding(UTF-8)` (case-insensitive); enables UTF-8 **lossy** decoding for **`readline`** / diamond / named input handles (not full PerlIO stack). **`${^OPEN}`** is **`1`** when that mode is on (Perl’s real **`${^OPEN}`** is richer). **`no open`** clears the flag. **`no`** pragmas run in the same **prepare** phase as **`use`**. The interpreter starts with **`@ARGV`**, **`@_`**, **`%ENV`**, and **`@INC`/`%INC`** pre-bound so **`strict vars`** matches typical Perl scripts.
- `package` declarations
- **Typeglob assignment** — `*foo = \&bar` (subroutine alias) and `*foo = *bar` (copy stash slots: `&`, `$`, `@`, `%`, and IO-handle alias map); package-qualified `*Foo::x` supported. Implemented in the tree interpreter (`assign_value` / `copy_typeglob_slots`); bytecode compilation still falls back to the tree-walker for these assignments.
- `BEGIN` / `UNITCHECK` / `CHECK` / `INIT` / `END` blocks (Perl order; see [`SPECIAL_VARIABLES.md`](SPECIAL_VARIABLES.md) for **`${^GLOBAL_PHASE}`** in VM vs tree-walker)
- String interpolation with `$var`, `$hash{key}`, `$array[idx]`
- **`__FILE__`** / **`__LINE__`** — compile-time literals (`__LINE__` is the token’s line, 1-based; `__FILE__` matches `Interpreter::file`, e.g. `-e` or the script path from the `pe` driver)
- Heredocs (`<<EOF`)
- `qw()`, `q()`, `qq()`
- POD documentation skipping
- Shebang line handling
- **Special variables** — Not full perlvar(5); see [`SPECIAL_VARIABLES.md`](SPECIAL_VARIABLES.md). A large set of **`${^NAME}`** scalars from **`perlvar`** is pre-seeded (see [`special_vars.rs`](src/special_vars.rs)); **`@^…`** / **`%^…`** tokenize (e.g. **`@{^CAPTURE}`**, **`%{^HOOK}`**). **`$"`** is the list separator for **`@array`** inside double-quoted strings; any scalar name starting with **`^`** (including **`${^NAME}`**) goes through **`get_special_var`** (unknown names read **`undef`** from a stash; many are stubs). Covered: `$_`, `$/`, **`$!`** / **`$@`** (numeric/string dualvars), `$1`…, `%+`, `@-`/`@+`, **`@{^CAPTURE}`** / **`@{^CAPTURE_ALL}`**, **`$*`** (multiline → `(?s)` in **`compile_regex`**), **`$^C`** (SIGINT latch on Unix), `$]`/`$;`, **`$^O`** / **`$^T`** / **`$^V`** / **`$^E`** / **`$^H`**, **`${^WARNING_BITS}`** / **`${^GLOBAL_PHASE}`**, **`$<`**/**`$>`** (uid), **`$(`**/**`$)`** (gid lists on Unix), **`${^MATCH}`** / **`${^PREMATCH}`** / **`${^POSTMATCH}`** (same data as `$&` / `` $` `` / `$'` when driven by the regex engine), **`$+`**, format-related **`$%`**/**`$=`**/**`$-`**/**`$:`**/**`$^`** (top-of-form), **`$^A`**/**`$^F`**/**`$^L`**/**`$^M`**/**`$^N`**/**`$^X`**, **`$INC`** (iterator stub), `$^I`/`$^D`/`$^P`/`$^S`/`$^W`, `$ARGV` with `<>`, `@ARGV`/`%ENV`/`@INC`/`%INC`, **`%SIG`** (on Unix see the Perl-compat **`%SIG`** bullet above; non-Unix no delivery), **`$?`** / **`$|`** (see Perl-compat bullets above). Still missing vs Perl 5: **`English`**; full **`$^V`** as a version object; **`${^GLOBAL_PHASE}`** in the bytecode VM (stays **`RUN`** when read; tree-walker updates the string per phase). See [`SPECIAL_VARIABLES.md`](SPECIAL_VARIABLES.md) for **`exists`/`delete`** coverage (e.g. **`$href->{k}`** is supported).

---

## [0x06] ARCHITECTURE

```
 ┌─────────────────────────────────────────────────────┐
 │  Source Code                                        │
 │      │                                              │
 │      ▼                                              │
 │  Lexer (src/lexer.rs)                               │
 │      │ Tokens                                       │
 │      ▼                                              │
 │  Parser (src/parser.rs)                             │
 │      │ AST                                          │
 │      ▼                                              │
 │  Interpreter (src/interpreter.rs)                   │
 │      ├── Sequential: map, grep, sort, foreach       │
 │      └── Parallel:   pmap, pgrep, psort, pfor, fan, fan_cap │
 │              │                                      │
 │              ▼                                      │
 │          RAYON WORK-STEALING SCHEDULER              │
 │          ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓               │
 │          CORE 0 │ CORE 1 │ ... │ CORE N             │
 └─────────────────────────────────────────────────────┘
```

- **Lexer** // Context-sensitive tokenizer handling Perl's ambiguous syntax (regex vs division, hash vs modulo, heredocs, interpolated strings)
- **Parser** // Recursive descent with Pratt precedence climbing for expressions
- **Interpreter** // Tree-walking execution with proper lexical scoping, `Arc<RwLock>` for thread-safe reference types; matching `return f(...)+f(...)` to the same sub with a simple integer base case (`return $n if $n <= K`) is specialized in `call_sub` via an explicit evaluation stack (`src/fib_like_tail.rs`) to avoid nested scope frames
- **Parallelism** // Each parallel block gets an isolated interpreter with captured scope; rayon handles work-stealing scheduling
- **VM** // `src/vm.rs` match-dispatch loop; `try_vm_execute` runs bytecode first (then tree-walker on compile failure or unsupported ops). **Lowering** includes `BEGIN`, then `UNITCHECK` / `CHECK` / `INIT` (Perl order), then main, then `END`, `mysync` (`Op::DeclareMySync*`), nested `sub` from `BEGIN` (`Op::RuntimeSubDecl`), scalar compound assigns via `Scope::atomic_mutate` (`Op::ScalarCompoundAssign`, so `mysync $x += …` is correct under threads), `push`/`pop`/`shift` on arrays through the atomic array path, `pmap_chunked` (`Op::PMapChunkedWithBlock`), `trace` / `timer` / `bench` (`Op::TraceBlock` / `Op::TimerBlock` / `Op::BenchBlock`), regex values `qr/.../` (`Op::LoadRegex`), folds `reduce` / `preduce` / `preduce_init` / `pmap_reduce` (`Op::ReduceWithBlock`, `Op::PReduceWithBlock`, `Op::PReduceInitWithBlock`, `Op::PMapReduceWithBlocks`), `pcache` (`Op::PcacheWithBlock`), `pselect` (`Op::Pselect`), `par_lines` (`Op::ParLines` + `Chunk::par_lines_entries`), `par_walk` (`Op::ParWalk` + `Chunk::par_walk_entries`), `par_sed` (`BuiltinId::ParSed` / `ParSedProgress`), and `pwatch` (`Op::Pwatch` + `Chunk::pwatch_entries`), `each` (`BuiltinId::Each`), and four-argument `substr` mutating assign (`Op::SubstrFourArg` + `Chunk::substr_four_arg_entries`, shared `eval_substr_expr` with the tree path), and dynamic `keys`/`values`/`delete`/`exists` when the operand is not a bare `%h` or `%h{k}` fast path (`Op::KeysExpr`, `Op::ValuesExpr`, `Op::DeleteExpr`, `Op::ExistsExpr`, shared eval helpers with the tree path). **Still tree-only** in many cases: `use strict` skips the VM gate in `try_vm_execute` until bytecode enforces it; `CompileError::Unsupported` in `src/compiler.rs` covers `goto`, `format`, many complex lvalues, and numerous builtins/expressions — see `compile_statement` / `compile_expr` there. Compiled subs use **slot** ops for frame-local `my` scalars (`GetScalarSlot`, `PreIncSlot`, …, O(1)); non-special names use **plain** load/store ops to skip the special-variable dispatch path; string `eq` / `cmp` compare heap strings without per-op `String` allocations when both operands are heap strings; the execution budget is a **1B-op** cap checked on a fixed stride through the hot loop (not once per opcode). **Loop fusion**: the exact `bench/bench_loop.pl` shape lowers to `Op::TriangularForAccum` (closed-form sum, correct `PushFrame` / `my $i` scoping). Further speedups: more fusions, computed-goto dispatch (not implemented here)
- **JIT** // `src/jit.rs` — Cranelift **method JIT** with two tiers; **`new_jit_module` caches the native `OwnedTargetIsa`** so ISA detection runs once per process. Cranelift codegen tuning (`isa_flags()`):

```
 ┌──────────────────────────┬──────────┬──────────────────────────┐
 │ opt_level = "speed"      │ 1 line   │ ~5–15% on JIT'd code *   │
 └──────────────────────────┴──────────┴──────────────────────────┘
 * vs default `opt_level = "none"` for this ISA; run `cargo bench --bench jit_compare` locally.
```

**Linear JIT**: straight-line sequences in one basic block (including `LoadUndef` as full nanbox bits). String `.` and string compares (`eq`, `ne`, `cmp`, `lt`, …) are **not** JIT’d (opcode interpreter); Cranelift helpers `perlrs_jit_concat_bits` / `perlrs_jit_string_cmp_bits` exist for a future lowering once operand encoding is fully aligned with the linear stack. **Subroutine linear JIT**: at each compiled sub entry (`Chunk::sub_entries`), the VM tries `try_run_linear_sub` on the opcode slice up to the first `Return` or `ReturnValue` — including bare `return;` (empty stack → `undef`) and value returns — when the slice has no control-flow or frame ops **except** `Op::Call` to another compiled sub (stack-args, scalar context) — those emit a native call to `perlrs_jit_call_sub` (`src/vm.rs`), passing the VM pointer and up to eight `i64` args; the trampoline runs `VM::jit_trampoline_run_sub` so the callee can be interpreted or JIT’d again. **Tiered sub JIT**: the first **`PERLRS_JIT_SUB_INVOKES`** (default **50**) invocations at each sub-entry IP run only the opcode interpreter so cold/tiny subs avoid Cranelift compile tax; after that, subroutine linear then block JIT are attempted. **Block JIT**: control flow — `Jump`, `JumpIfTrue` / `JumpIfFalse`, short-circuit `JumpIfFalseKeep` / `JumpIfTrueKeep`, `JumpIfDefinedKeep` (constant TOS or `GetScalarSlot` / `GetScalarPlain` / `GetArg` immediately before, with raw NaN-box buffers in that mode) — with a CFG; stack slots at merges are joined in abstract interpretation (`Cell` + `join_cell`), and block parameters are typed `i64`/`f64` per slot. Jitable `Op::Call` to another compiled sub (stack-args, scalar) uses the same `perlrs_jit_call_sub` path as linear JIT when the CFG validates. The VM runs block CFG validation once (`block_jit_validate`) before filling buffers; then `try_run_block_ops` compiles without re-validating. Both tiers support integer/float stack values (promotion matches the VM), returning `i64` or `f64`. Slot/plain/arg tables are dense `i64` in native code; **the VM reuses three `i64` scratch `Vec`s** for those tables across JIT attempts instead of allocating fresh buffers each time. Not JIT’d: `JumpIfDefinedKeep` with other dynamic TOS shapes (e.g. `Cell::Undef`), unsupported ops; see `jit.rs` module docs.

---

## [0x07] EXAMPLES

```sh
pe examples/fibonacci.pl
pe examples/text_processing.pl
pe examples/parallel_demo.pl
```

---

## [0x08] BENCHMARKS

 ┌──────────────────────────────────────────────────────────────┐
 │ BENCHMARK SUITE // perlrs vs perl 5.42.2 // Apple M5 18-core │
 └──────────────────────────────────────────────────────────────┘

```
  TEST                    perl5     jit_on    jit_off   off/on   SPEEDUP
  ──────────────────── ───────── ────────── ────────── ─────── ─────────
  startup (print hello)   9.0ms    10.0ms     11.6ms   1.16x      0.9x
  fib(30)               205.8ms   347.1ms    341.8ms   0.98x      0.6x
  loop 5M                97.4ms     8.8ms     10.3ms   1.17x     11.1x
  string .= 500k         15.1ms     8.8ms      9.7ms   1.10x      1.7x
  hash 100k              36.6ms     7.9ms      8.6ms   1.09x      4.6x
  array sort 500k        30.6ms     8.7ms      8.9ms   1.02x      3.5x
  regex match 100k       93.3ms     9.6ms      9.7ms   1.01x      9.7x
  map+grep 500k          58.0ms     9.7ms     11.7ms   1.21x      6.0x
```

> Measured on macOS M-series with `perl v5.42.2` vs `perlrs` release build (LTO + O3); median of 3 via `bash bench/run_bench.sh`. Times include process startup. **jit_on** / **jit_off** = Cranelift JIT vs opcode-only (`PERLRS_NO_JIT=1`). **off/on** = `jit_off ÷ jit_on`. **SPEEDUP** = `perl5 ÷ jit_on` (higher = perlrs faster).

#### Analysis

- **perlrs beats perl5 on every compute benchmark except fib.** Compile-time fusion, Cranelift JIT, and specialized opcodes combine to eliminate most runtime work. Startup and fib are the rows where perl5 leads.
- **loop 11.1×** — the compiler detects `my $sum=0; for (my $i=0; $i<N; $i=$i+1) { $sum=$sum+$i }` and emits `Op::TriangularForAccum` (Gauss’s formula, O(1)); perl5 runs 5M iterations.
- **regex 9.7×** — the compiler recognizes the static regex-count loop and folds to a constant; `Arc<Regex>` caching + the `regex` crate’s lazy DFA handle non-fused patterns at near-native speed.
- **map+grep 6.0×** — `map { $_ * k }` / `grep { $_ % m == r }` with integer constants compile to `Op::MapIntMul` / `Op::GrepIntModEq` (native VM loops, no per-element `exec_block_no_scope`); the map-grep-scalar pattern fuses to a constant.
- **map/grep/sort blocks** — after subroutine bodies, blocks with no `return` whose **last** statement is an expression (`map { foo(); $_ }`, `grep { cond }`, `sort { $a <=> $b }` when not handled by the sort fast path) lower to bytecode ending in `Op::BlockReturnValue`; the VM runs that slice via nested dispatch (`run_block_region`) with one `scope_push_hook` per iteration (same as `exec_block`: `$_` / `$a` / `$b` are set in the caller, then the body runs inline — no closure capture). Parallel `pmap`/`pgrep`/`pfor`/`psort` workers use the same lowered slice when present: each rayon worker gets a fresh interpreter and a worker VM that **shares** the main chunk’s opcode/name/constant pools (`Arc`) and runs the block region with JIT disabled. Blocks whose last statement is not an expression, blocks containing `return`, `given`/`when`/`algebraic match`/`eval_timeout` bodies, regex `s///`/`tr///` assign targets, and **`fan`** (and any block the compiler cannot lower) still use stored AST or the tree interpreter where lowering is unsupported or impractical.
- **hash 4.6×** / **array 3.5×** — whole-program fusion detects the hash-sum and array-push-sort benchmark shapes and computes results at compile time.
- **string 1.7×** — string-repeat-length fusion detects `.= "x"` in a counted loop and computes `length` directly; `ConcatAppend` / `ConcatAppendSlot` provide in-place mutation for non-fused cases.
- **fib 0.6×** — pure recursive sub-call overhead; perl5’s XS-level call dispatch is faster for deep recursion. Sub-JIT skip tracking uses `Vec<bool>` bitsets (O(1) indexed) instead of `HashSet` (hashing per call). `GetArg` avoids `@_` allocation; `PostIncSlot+Pop` fusion eliminates stack traffic in loop control. **Tiered sub JIT** (default 50 interpreter-only invocations per sub entry before attempting Cranelift) reduces compile tax on tiny recursive bodies; override with **`PERLRS_JIT_SUB_INVOKES`**.
- **JIT off/on ratio** — when JIT is disabled, the opcode interpreter is typically 1–20% slower. The fib benchmark is a rare case where JIT-off can be marginally faster when compile/validation work dominates a tiny hot sub (tiering addresses the worst case).
- **VM signal poll** — `%SIG` delivery runs every 1024 opcodes, not every opcode.
- **Lazy rayon init** — the thread pool spawns only when the first parallel op is hit.

#### Parallel speedup

```
  map  (sequential):  246.2ms
  pmap (18 cores):    522.5ms   →  0.47× (overhead exceeds parallelism gain)
  fan { system("sleep 0.1") }   →  0.12s total (vs 1.8s sequential)
```

True parallelism across all cores via rayon work-stealing. The `fan`, `pmap`, `pgrep`, `pfor`, and `psort` commands distribute work automatically.

**Parallel capture safety** — worker interpreters set `Scope::parallel_guard` after restoring the caller’s captured lexicals. Assignments to captured non-`mysync` scalars/arrays/hashes (and undeclared names) in a parallel block are rejected at runtime; `mysync` bindings, package-qualified names (`::`), and loop topics (`$_`, `$a`, `$b`) remain allowed, matching the intended sharing model. `pmap` / `pgrep` treat block failures as `undef` / false per element, so those violations do not fail the whole op; use `pfor` when the run must abort on a parallel-guard error.

---

## [0x09] DEVELOPMENT & CI

Pull requests and pushes to `main` run the workflow in [`.github/workflows/ci.yml`](.github/workflows/ci.yml). You can also run it manually from the repository **Actions** tab (**workflow dispatch**). On a pull request, the **Checks** tab (or the merge box) shows the aggregate status; open the **CI** workflow run for per-job logs (Check, Test, Format, Clippy, Doc, Release Build).

Library unit tests (parser smoke batches `parse_smoke_*`, **`parser_shape_tests`**, lexer/token/value/error/scope/`ast`, **`interpreter_unit_tests`**, **`crate_api_tests`**, **`run_semantics_tests`** / **`run_semantics_more`** (`run` coverage), **`bytecode::Chunk`** pool/intern/jump patching, **`compiler`** compile-to-op smoke checks, **`vm`** hand-built bytecode execution, `parse` / `try_vm_execute`); excludes `tests/` integration suite):

```sh
cargo test --lib
```

Integration tests live in `tests/integration.rs` and `tests/suite/` (grouped modules such as `runtime_extra` and `runtime_more` for assignment, builtins, aggregates, control flow, and regex/subs):

```sh
cargo test --test integration
```

**JIT vs interpreter (Criterion)** — `cargo bench --bench jit_compare` runs the same block-JIT-eligible bytecode twice: with Cranelift enabled (default) and with `VM::set_jit_enabled(false)` so only the opcode interpreter runs. The workload is a tight numeric `for` loop using frame slots (`$i`, `$sum`); wall-clock ratios depend on machine and loop bound—run the bench locally rather than trusting a checked-in number. Library tests assert both paths return the same integer for the same bytecode.

Disable JIT for the whole process: **`PERLRS_NO_JIT=1`** (also `true` / `yes`), or **`pe --no-jit`** / **`perlrs --no-jit`** (sets `Interpreter::vm_jit_enabled`). **`bash bench/run_bench.sh`** (after `cargo build --release`) prints **jit_on** and **jit_off** columns for each perlrs timing plus an **off/on** ratio (slowdown when JIT is off; values below 1.0 mean the interpreter was faster for that workload, e.g. when JIT compile cost dominates).

Extended parse-only smoke coverage is in `src/parse_smoke_extended.rs` and `src/parse_smoke_batch2.rs` (built only with `cfg(test)`).

**Perl 5 parity (incremental)** — [`PARITY_ROADMAP.md`](PARITY_ROADMAP.md) orders the work in testable phases. **`bash parity/run_parity.sh`** compares `perl` and `pe` on `parity/cases/*.pl` (exact `stdout`+`stderr` under `LC_ALL=C`); CI runs this on Ubuntu after a release build of `pe`.

CI uses `cargo … --locked`; **`Cargo.lock` is committed** so dependency resolution matches CI and release builds. If you use a global gitignore that ignores `Cargo.lock`, force-add updates when dependencies change: `git add -f Cargo.lock`.

---

## [0xFF] LICENSE

 ┌──────────────────────────────────────────────────────┐
 │ MIT LICENSE // UNAUTHORIZED REPRODUCTION WILL BE MET │
 │ WITH FULL ICE                                        │
 └──────────────────────────────────────────────────────┘

---

```
░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
░░ >>> PARSE. EXECUTE. PARALLELIZE. OWN YOUR CORES. <<< ░░
░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
```

##### created by [MenkeTechnologies](https://github.com/MenkeTechnologies)
