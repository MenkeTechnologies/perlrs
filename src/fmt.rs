//! Pretty-print parsed Perl back to source (`pe --fmt`).
//! Regenerate with `python3 tools/gen_fmt.py` after `ast.rs` changes.

use crate::ast::*;

/// Format a whole program as Perl-like source.
pub fn format_program(p: &Program) -> String {
    p.statements
        .iter()
        .map(|s| format_statement(s))
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_statement(s: &Statement) -> String {
    let lab = s.label.as_ref().map(|l| format!("{}: ", l)).unwrap_or_default();
    let body = match &s.kind {
        StmtKind::Expression(e) => format!("{};", format_expr(e)),
        StmtKind::If {
                condition,
                body,
                elsifs,
                else_block,
            } => {
                let mut s = format!("if ({}) {{\n{}\n}}", format_expr(condition), format_block(body));
                for (c, b) in elsifs {
                    s.push_str(&format!(" elsif ({}) {{\n{}\n}}", format_expr(c), format_block(b)));
                }
                if let Some(eb) = else_block {
                    s.push_str(&format!(" else {{\n{}\n}}", format_block(eb)));
                }
                s
            },
        StmtKind::Unless {
                condition,
                body,
                else_block,
            } => {
                let mut s = format!("unless ({}) {{\n{}\n}}", format_expr(condition), format_block(body));
                if let Some(eb) = else_block {
                    s.push_str(&format!(" else {{\n{}\n}}", format_block(eb)));
                }
                s
            },
        StmtKind::While {
                condition,
                body,
                label,
                continue_block,
            } => {
                let lb = label.as_ref().map(|l| format!("{}: ", l)).unwrap_or_default();
                let mut s = format!("{}while ({}) {{\n{}\n}}", lb, format_expr(condition), format_block(body));
                if let Some(cb) = continue_block {
                    s.push_str(&format!(" continue {{\n{}\n}}", format_block(cb)));
                }
                s
            },
        StmtKind::Until {
                condition,
                body,
                label,
                continue_block,
            } => {
                let lb = label.as_ref().map(|l| format!("{}: ", l)).unwrap_or_default();
                let mut s = format!("{}until ({}) {{\n{}\n}}", lb, format_expr(condition), format_block(body));
                if let Some(cb) = continue_block {
                    s.push_str(&format!(" continue {{\n{}\n}}", format_block(cb)));
                }
                s
            },
        StmtKind::DoWhile { body, condition } => {
                format!("do {{\n{}\n}} while ({})", format_block(body), format_expr(condition))
            },
        StmtKind::For {
                init,
                condition,
                step,
                body,
                label,
                continue_block,
            } => {
                let lb = label.as_ref().map(|l| format!("{}: ", l)).unwrap_or_default();
                let ini = init.as_ref().map(|s| format_statement(s)).unwrap_or_default();
                let cond = condition.as_ref().map(|e| format_expr(e)).unwrap_or_default();
                let st = step.as_ref().map(|e| format_expr(e)).unwrap_or_default();
                let mut s = format!(
                    "{}for ({}; {}; {}) {{\n{}\n}}",
                    lb, ini, cond, st, format_block(body)
                );
                if let Some(cb) = continue_block {
                    s.push_str(&format!(" continue {{\n{}\n}}", format_block(cb)));
                }
                s
            },
        StmtKind::Foreach {
                var,
                list,
                body,
                label,
                continue_block,
            } => {
                let lb = label.as_ref().map(|l| format!("{}: ", l)).unwrap_or_default();
                let mut s = format!(
                    "{}foreach \${} ({}) {{\n{}\n}}",
                    lb, var, format_expr(list), format_block(body)
                );
                if let Some(cb) = continue_block {
                    s.push_str(&format!(" continue {{\n{}\n}}", format_block(cb)));
                }
                s
            },
        StmtKind::SubDecl {
                name,
                params: _params,
                body,
                prototype,
            } => {
                let proto = prototype.as_ref().map(|p| format!(" ({})", p)).unwrap_or_default();
                format!("sub {}{} {{\n{}\n}}", name, proto, format_block(body))
            },
        StmtKind::Package { name } => format!("package {};", name),
        StmtKind::Use { module, imports } => {
                if imports.is_empty() {
                    format!("use {};", module)
                } else {
                    format!("use {} {};", module, format_expr_list(imports))
                }
            },
        StmtKind::No { module, imports } => {
                if imports.is_empty() {
                    format!("no {};", module)
                } else {
                    format!("no {} {};", module, format_expr_list(imports))
                }
            },
        StmtKind::Return(e) => e
                .as_ref()
                .map(|x| format!("return {};", format_expr(x)))
                .unwrap_or_else(|| "return;".to_string()),
        StmtKind::Last(_) => format!("/* unsupported StmtKind::Last */"),
        StmtKind::Next(_) => format!("/* unsupported StmtKind::Next */"),
        StmtKind::Redo(_) => format!("/* unsupported StmtKind::Redo */"),
        StmtKind::My(_) => format!("/* unsupported StmtKind::My */"),
        StmtKind::Our(_) => format!("/* unsupported StmtKind::Our */"),
        StmtKind::Local(_) => format!("/* unsupported StmtKind::Local */"),
        StmtKind::MySync(_) => format!("/* unsupported StmtKind::MySync */"),
        StmtKind::Block(_) => format!("/* unsupported StmtKind::Block */"),
        StmtKind::Begin(b) => format!("BEGIN {{\n{}\n}}", format_block(b)),
        StmtKind::End(b) => format!("END {{\n{}\n}}", format_block(b)),
        StmtKind::Empty => ";".to_string(),
        StmtKind::Goto { target } => format!("goto {};", format_expr(target)),
        StmtKind::Continue(b) => format!("continue {{\n{}\n}}", format_block(b)),
        StmtKind::StructDecl { def } => {
                let fields = def
                    .fields
                    .iter()
                    .map(|(n, t)| format!("{} => {:?}", n, t))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("struct {} {{ {} }}", def.name, fields)
            },
        StmtKind::EvalTimeout { timeout, body } => {
                format!("eval_timeout {} {{\n{}\n}}", format_expr(timeout), format_block(body))
            },
        StmtKind::TryCatch {
                try_block,
                catch_var,
                catch_block,
            } => {
                format!(
                    "try {{\n{}\n}} catch (\${}) {{\n{}\n}}",
                    format_block(try_block),
                    catch_var,
                    format_block(catch_block)
                )
            },
        StmtKind::Given { topic, body } => {
                format!("given ({}) {{\n{}\n}}", format_expr(topic), format_block(body))
            },
        StmtKind::When { cond, body } => {
                format!("when ({}) {{\n{}\n}}", format_expr(cond), format_block(body))
            },
        StmtKind::DefaultCase { body } => format!("default {{\n{}\n}}", format_block(body)),
    };
    format!("{}{}", lab, body)
}

fn format_block(b: &Block) -> String {
    b.iter().map(format_statement).collect::<Vec<_>>().join("\n")
}

fn format_var_decls(decls: &[VarDecl]) -> String {
    decls
        .iter()
        .map(|d| {
            let sig = match d.sigil {
                Sigil::Scalar => "$",
                Sigil::Array => "@",
                Sigil::Hash => "%",
            };
            let mut s = format!("{}{}", sig, d.name);
            if let Some(t) = d.type_annotation {
                s.push_str(&format!(" : {:?}", t));
            }
            if let Some(ref init) = d.initializer {
                s.push_str(&format!(" = {}", format_expr(init)));
            }
            s
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_expr_list(es: &[Expr]) -> String {
    es.iter().map(format_expr).collect::<Vec<_>>().join(", ")
}

fn format_binop(op: BinOp) -> &'static str {
    match op {
        BinOp::Add => "+",
        BinOp::Sub => "-",
        BinOp::Mul => "*",
        BinOp::Div => "/",
        BinOp::Mod => "%",
        BinOp::Pow => "**",
        BinOp::Concat => ".",
        BinOp::NumEq => "==",
        BinOp::NumNe => "!=",
        BinOp::NumLt => "<",
        BinOp::NumGt => ">",
        BinOp::NumLe => "<=",
        BinOp::NumGe => ">=",
        BinOp::Spaceship => "<=>",
        BinOp::StrEq => "eq",
        BinOp::StrNe => "ne",
        BinOp::StrLt => "lt",
        BinOp::StrGt => "gt",
        BinOp::StrLe => "le",
        BinOp::StrGe => "ge",
        BinOp::StrCmp => "cmp",
        BinOp::LogAnd => "&&",
        BinOp::LogOr => "||",
        BinOp::DefinedOr => "//",
        BinOp::BitAnd => "&",
        BinOp::BitOr => "|",
        BinOp::BitXor => "^",
        BinOp::ShiftLeft => "<<",
        BinOp::ShiftRight => ">>",
        BinOp::LogAndWord => "and",
        BinOp::LogOrWord => "or",
        BinOp::BindMatch => "=~",
        BinOp::BindNotMatch => "!~",
    }
}

fn format_unary(op: UnaryOp) -> &'static str {
    match op {
        UnaryOp::Negate => "-",
        UnaryOp::LogNot => "!",
        UnaryOp::BitNot => "~",
        UnaryOp::LogNotWord => "not",
        UnaryOp::PreIncrement => "++",
        UnaryOp::PreDecrement => "--",
        UnaryOp::Ref => "\",
    }
}

fn format_postfix(op: PostfixOp) -> &'static str {
    match op {
        PostfixOp::Increment => "++",
        PostfixOp::Decrement => "--",
    }
}

fn format_string_part(p: &StringPart) -> String {
    match p {
        StringPart::Literal(s) => s.clone(),
        StringPart::ScalarVar(n) => format!("\${{{}}}", n),
        StringPart::ArrayVar(n) => format!("\@{{{}}}", n),
        StringPart::Expr(e) => format_expr(e),
    }
}

fn format_string_literal(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Format an expression; aims for readable Perl-like output.
pub fn format_expr(e: &Expr) -> String {
    match &e.kind {
        ExprKind::Integer(n) => n.to_string(),
        ExprKind::Float(f) => format!("{}", f),
        ExprKind::String(s) => format_string_literal(s),
        ExprKind::Regex(p, fl) => format!("/{}/{}/", p, fl),
        ExprKind::QW(ws) => format!("qw({})", ws.join(" ")),
        ExprKind::Undef => "undef".to_string(),
        ExprKind::InterpolatedString(_) => format!("/* ExprKind::InterpolatedString */"),
        ExprKind::ScalarVar(_) => format!("/* ExprKind::ScalarVar */"),
        ExprKind::ArrayVar(_) => format!("/* ExprKind::ArrayVar */"),
        ExprKind::HashVar(_) => format!("/* ExprKind::HashVar */"),
        ExprKind::ArrayElement {
            array: String,
        index: Box<Expr>,
        } => format!("${{ {}[{}] }}", array, format_expr(index)),
        ExprKind::HashElement {
            hash: String,
        key: Box<Expr>,
        } => format!("${{ {}\{{{}\}} }}", hash, format_expr(key)),
        ExprKind::ArraySlice {
            array: String,
        indices: Vec<Expr>,
        } => format!("@{{{}[{}]}}", array, indices.iter().map(format_expr).collect::<Vec<_>>().join(", ")),
        ExprKind::HashSlice {
            hash: String,
        keys: Vec<Expr>,
        } => format!("@{{{}\{{{}\}}}}", hash, keys.iter().map(format_expr).collect::<Vec<_>>().join(", ")),
        ExprKind::ScalarRef(_) => format!("/* ExprKind::ScalarRef */"),
        ExprKind::ArrayRef(_) => format!("/* ExprKind::ArrayRef */"),
        ExprKind::HashRef(_) => format!("/* ExprKind::HashRef */"),
        ExprKind::CodeRef {
            params: Vec<String>,
        body: Block,
        } => format!("sub {{\n{}\n}}", format_block(body)),
        ExprKind::Deref {
            expr: Box<Expr>,
        kind: Sigil,
        } => match kind {
                Sigil::Scalar => format!("${{${}}}", format_expr(expr)),
                Sigil::Array => format!("@{{${}}}", format_expr(expr)),
                Sigil::Hash => format!("%{{${}}}", format_expr(expr)),
            },
        ExprKind::ArrowDeref {
            expr: Box<Expr>,
        index: Box<Expr>,
        kind: DerefKind,
        } => match kind {
                DerefKind::Array => format!("({})->[{}]", format_expr(expr), format_expr(index)),
                DerefKind::Hash => format!("({})->{{}}", format_expr(expr), format_expr(index)),
                DerefKind::Call => format!("({})->({})", format_expr(expr), format_expr(index)),
            },
        ExprKind::BinOp {
            left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
        } => format!("({} {} {})", format_expr(left), format_binop(*op), format_expr(right)),
        ExprKind::UnaryOp {
            op: UnaryOp,
        expr: Box<Expr>,
        } => format!("({}{})", format_unary(*op), format_expr(expr)),
        ExprKind::PostfixOp {
            expr: Box<Expr>,
        op: PostfixOp,
        } => format!("({}{})", format_expr(expr), format_postfix(*op)),
        ExprKind::Assign {
            target: Box<Expr>,
        value: Box<Expr>,
        } => format!("{} = {}", format_expr(target), format_expr(value)),
        ExprKind::CompoundAssign {
            target: Box<Expr>,
        op: BinOp,
        value: Box<Expr>,
        } => format!("{} {}= {}", format_expr(target), format_binop(*op), format_expr(value)),
        ExprKind::Ternary {
            condition: Box<Expr>,
        then_expr: Box<Expr>,
        else_expr: Box<Expr>,
        } => format!("({} ? {} : {})", format_expr(condition), format_expr(then_expr), format_expr(else_expr)),
        ExprKind::Repeat {
            expr: Box<Expr>,
        count: Box<Expr>,
        } => format!("({} x {})", format_expr(expr), format_expr(count)),
        ExprKind::Range {
            from: Box<Expr>,
        to: Box<Expr>,
        } => format!("({} .. {})", format_expr(from), format_expr(to)),
        ExprKind::FuncCall {
            name: String,
        args: Vec<Expr>,
        } => format!("{}({})", name, args.iter().map(format_expr).collect::<Vec<_>>().join(", ")),
        ExprKind::MethodCall {
            object: Box<Expr>,
        method: String,
        args: Vec<Expr>,
        } => format!("{}->{}({})", format_expr(object), method, args.iter().map(format_expr).collect::<Vec<_>>().join(", ")),
        ExprKind::Print {
            handle: Option<String>,
        args: Vec<Expr>,
        } => {
                let mut s = String::new();
                if let Some(h) = handle {
                    s.push_str(h);
                    s.push_str(": ");
                }
                s.push_str("print ");
                s.push_str(&format_expr_list(args));
                s
            },
        ExprKind::Say {
            handle: Option<String>,
        args: Vec<Expr>,
        } => {
                let mut s = String::new();
                if let Some(h) = handle {
                    s.push_str(h);
                    s.push_str(": ");
                }
                s.push_str("say ");
                s.push_str(&format_expr_list(args));
                s
            },
        ExprKind::Printf {
            handle: Option<String>,
        args: Vec<Expr>,
        } => {
                let mut s = String::new();
                if let Some(h) = handle {
                    s.push_str(h);
                    s.push_str(": ");
                }
                s.push_str("printf ");
                s.push_str(&format_expr_list(args));
                s
            },
        ExprKind::Die(_) => format!("/* ExprKind::Die */"),
        ExprKind::Warn(_) => format!("/* ExprKind::Warn */"),
        ExprKind::Match {
            expr: Box<Expr>,
        pattern: String,
        flags: String,
        scalar_g: bool,
        } => format!("({} =~ /{}/{})", format_expr(expr), pattern, flags),
        ExprKind::Substitution {
            expr: Box<Expr>,
        pattern: String,
        replacement: String,
        flags: String,
        } => format!("({} =~ s/{}/{}/{})", format_expr(expr), pattern, replacement, flags),
        ExprKind::Transliterate {
            expr: Box<Expr>,
        from: String,
        to: String,
        flags: String,
        } => format!("({} =~ tr/{}/{}/{})", format_expr(expr), from, to, flags),
        ExprKind::MapExpr {
            block: Block,
        list: Box<Expr>,
        } => format!("map {{\n{}\n}} {}", format_block(block), format_expr(list)),
        ExprKind::GrepExpr {
            block: Block,
        list: Box<Expr>,
        } => format!("grep {{\n{}\n}} {}", format_block(block), format_expr(list)),
        ExprKind::SortExpr {
            cmp: Option<Block>,
        list: Box<Expr>,
        } => match cmp {
                Some(b) => format!("sort {{\n{}\n}} {}", format_block(b), format_expr(list)),
                None => format!("sort {}", format_expr(list)),
            },
        ExprKind::ReverseExpr(e) => format!("reverse {}", format_expr(e)),
        ExprKind::JoinExpr {
            separator: Box<Expr>,
        list: Box<Expr>,
        } => format!("join({}, {})", format_expr(separator), format_expr(list)),
        ExprKind::SplitExpr {
            pattern: Box<Expr>,
        string: Box<Expr>,
        limit: Option<Box<Expr>>,
        } => match limit {
                Some(l) => format!("split({}, {}, {})", format_expr(pattern), format_expr(string), format_expr(l)),
                None => format!("split({}, {})", format_expr(pattern), format_expr(string)),
            },
        ExprKind::PMapExpr {
            block: Block,
        list: Box<Expr>,
        } => format!("pmap {{\n{}\n}} {}", format_block(block), format_expr(list)),
        ExprKind::PMapChunkedExpr {
            chunk_size: Box<Expr>,
        block: Block,
        list: Box<Expr>,
        } => format!("pmap_chunked {} {{\n{}\n}} {}", format_expr(chunk_size), format_block(block), format_expr(list)),
        ExprKind::PGrepExpr {
            block: Block,
        list: Box<Expr>,
        } => format!("pgrep {{\n{}\n}} {}", format_block(block), format_expr(list)),
        ExprKind::PForExpr {
            block: Block,
        list: Box<Expr>,
        } => format!("pfor {{\n{}\n}} {}", format_block(block), format_expr(list)),
        ExprKind::ParLinesExpr {
            path: Box<Expr>,
        callback: Box<Expr>,
        } => format!("par_lines({}, {})", format_expr(path), format_expr(callback)),
        ExprKind::PwatchExpr {
            path: Box<Expr>,
        callback: Box<Expr>,
        } => format!("pwatch({}, {})", format_expr(path), format_expr(callback)),
        ExprKind::PSortExpr {
            cmp: Option<Block>,
        list: Box<Expr>,
        } => match cmp {
                Some(b) => format!("psort {{\n{}\n}} {}", format_block(b), format_expr(list)),
                None => format!("psort {}", format_expr(list)),
            },
        ExprKind::ReduceExpr {
            block: Block,
        list: Box<Expr>,
        } => format!("reduce {{\n{}\n}} {}", format_block(block), format_expr(list)),
        ExprKind::PReduceExpr {
            block: Block,
        list: Box<Expr>,
        } => format!("preduce {{\n{}\n}} {}", format_block(block), format_expr(list)),
        ExprKind::FanExpr {
            count: Box<Expr>,
        block: Block,
        } => format!("fan {} {{\n{}\n}}", format_expr(count), format_block(block)),
        ExprKind::AsyncBlock {
            body: Block,
        } => format!("async {{\n{}\n}}", format_block(body)),
        ExprKind::Trace {
            body: Block,
        } => format!("trace {{\n{}\n}}", format_block(body)),
        ExprKind::Timer {
            body: Block,
        } => format!("timer {{\n{}\n}}", format_block(body)),
        ExprKind::Await(e) => format!("await {}", format_expr(e)),
        ExprKind::Slurp(e) => format!("slurp {}", format_expr(e)),
        ExprKind::Capture(e) => format!("capture {}", format_expr(e)),
        ExprKind::FetchUrl(e) => format!("fetch_url {}", format_expr(e)),
        ExprKind::Pchannel => "pchannel()".to_string(),
        ExprKind::Push {
            array: Box<Expr>,
        values: Vec<Expr>,
        } => format!("push({}, {})", format_expr(array), format_expr_list(values)),
        ExprKind::Pop(e) => format!("pop {}", format_expr(e)),
        ExprKind::Shift(e) => format!("shift {}", format_expr(e)),
        ExprKind::Unshift {
            array: Box<Expr>,
        values: Vec<Expr>,
        } => format!("unshift({}, {})", format_expr(array), format_expr_list(values)),
        ExprKind::Splice {
            array: Box<Expr>,
        offset: Option<Box<Expr>>,
        length: Option<Box<Expr>>,
        replacement: Vec<Expr>,
        } => format!("splice({}, ...)", format_expr(array)),
        ExprKind::Delete(e) => format!("delete {}", format_expr(e)),
        ExprKind::Exists(e) => format!("exists {}", format_expr(e)),
        ExprKind::Keys(e) => format!("keys {}", format_expr(e)),
        ExprKind::Values(e) => format!("values {}", format_expr(e)),
        ExprKind::Each(e) => format!("each {}", format_expr(e)),
        ExprKind::Chomp(e) => format!("chomp {}", format_expr(e)),
        ExprKind::Chop(e) => format!("chop {}", format_expr(e)),
        ExprKind::Length(e) => format!("length {}", format_expr(e)),
        ExprKind::Substr {
            string: Box<Expr>,
        offset: Box<Expr>,
        length: Option<Box<Expr>>,
        replacement: Option<Box<Expr>>,
        } => format!("substr({}, ...)", format_expr(string)),
        ExprKind::Index {
            string: Box<Expr>,
        substr: Box<Expr>,
        position: Option<Box<Expr>>,
        } => format!("index({}, {})", format_expr(string), format_expr(substr)),
        ExprKind::Rindex {
            string: Box<Expr>,
        substr: Box<Expr>,
        position: Option<Box<Expr>>,
        } => format!("rindex({}, {})", format_expr(string), format_expr(substr)),
        ExprKind::Sprintf {
            format: Box<Expr>,
        args: Vec<Expr>,
        } => format!("sprintf({}, {})", format_expr(format), format_expr_list(args)),
        ExprKind::Abs(e) => format!("abs {}", format_expr(e)),
        ExprKind::Int(e) => format!("int {}", format_expr(e)),
        ExprKind::Sqrt(e) => format!("sqrt {}", format_expr(e)),
        ExprKind::Sin(e) => format!("sin {}", format_expr(e)),
        ExprKind::Cos(e) => format!("cos {}", format_expr(e)),
        ExprKind::Atan2 {
            y: Box<Expr>,
        x: Box<Expr>,
        } => format!("atan2({}, {})", format_expr(y), format_expr(x)),
        ExprKind::Exp(e) => format!("exp {}", format_expr(e)),
        ExprKind::Log(e) => format!("log {}", format_expr(e)),
        ExprKind::Rand(_) => format!("/* ExprKind::Rand */"),
        ExprKind::Srand(_) => format!("/* ExprKind::Srand */"),
        ExprKind::Hex(e) => format!("hex {}", format_expr(e)),
        ExprKind::Oct(e) => format!("oct {}", format_expr(e)),
        ExprKind::Lc(e) => format!("lc {}", format_expr(e)),
        ExprKind::Uc(e) => format!("uc {}", format_expr(e)),
        ExprKind::Lcfirst(e) => format!("lcfirst {}", format_expr(e)),
        ExprKind::Ucfirst(e) => format!("ucfirst {}", format_expr(e)),
        ExprKind::Fc(e) => format!("fc {}", format_expr(e)),
        ExprKind::Crypt {
            plaintext: Box<Expr>,
        salt: Box<Expr>,
        } => format!("crypt({}, {})", format_expr(plaintext), format_expr(salt)),
        ExprKind::Pos(_) => format!("/* ExprKind::Pos */"),
        ExprKind::Study(e) => format!("study {}", format_expr(e)),
        ExprKind::Defined(e) => format!("defined {}", format_expr(e)),
        ExprKind::Ref(e) => format!("ref {}", format_expr(e)),
        ExprKind::ScalarContext(e) => format!("scalar {}", format_expr(e)),
        ExprKind::Chr(e) => format!("chr {}", format_expr(e)),
        ExprKind::Ord(e) => format!("ord {}", format_expr(e)),
        ExprKind::Open {
            handle: Box<Expr>,
        mode: Box<Expr>,
        file: Option<Box<Expr>>,
        } => format!("open({}, {}, ...)", format_expr(handle), format_expr(mode)),
        ExprKind::Close(e) => format!("close {}", format_expr(e)),
        ExprKind::ReadLine(_) => format!("/* ExprKind::ReadLine */"),
        ExprKind::Eof(_) => format!("/* ExprKind::Eof */"),
        ExprKind::Opendir {
            handle: Box<Expr>,
        path: Box<Expr>,
        } => format!("opendir({}, {})", format_expr(handle), format_expr(path)),
        ExprKind::Readdir(e) => format!("readdir {}", format_expr(e)),
        ExprKind::Closedir(e) => format!("closedir {}", format_expr(e)),
        ExprKind::Rewinddir(e) => format!("rewinddir {}", format_expr(e)),
        ExprKind::Telldir(e) => format!("telldir {}", format_expr(e)),
        ExprKind::Seekdir {
            handle: Box<Expr>,
        position: Box<Expr>,
        } => format!("seekdir({}, {})", format_expr(handle), format_expr(position)),
        ExprKind::FileTest {
            op: char,
        expr: Box<Expr>,
        } => format!("-{}{}", op, format_expr(expr)),
        ExprKind::System(_) => format!("/* ExprKind::System */"),
        ExprKind::Exec(_) => format!("/* ExprKind::Exec */"),
        ExprKind::Eval(_) => format!("/* ExprKind::Eval */"),
        ExprKind::Do(_) => format!("/* ExprKind::Do */"),
        ExprKind::Require(_) => format!("/* ExprKind::Require */"),
        ExprKind::Exit(_) => format!("/* ExprKind::Exit */"),
        ExprKind::Chdir(_) => format!("/* ExprKind::Chdir */"),
        ExprKind::Mkdir {
            path: Box<Expr>,
        mode: Option<Box<Expr>>,
        } => format!("mkdir({}, ...)", format_expr(path)),
        ExprKind::Unlink(_) => format!("/* ExprKind::Unlink */"),
        ExprKind::Rename {
            old: Box<Expr>,
        new: Box<Expr>,
        } => format!("rename({}, {})", format_expr(old), format_expr(new)),
        ExprKind::Chmod(_) => format!("/* ExprKind::Chmod */"),
        ExprKind::Chown(_) => format!("/* ExprKind::Chown */"),
        ExprKind::Stat(e) => format!("stat {}", format_expr(e)),
        ExprKind::Lstat(e) => format!("lstat {}", format_expr(e)),
        ExprKind::Link {
            old: Box<Expr>,
        new: Box<Expr>,
        } => format!("link({}, {})", format_expr(old), format_expr(new)),
        ExprKind::Symlink {
            old: Box<Expr>,
        new: Box<Expr>,
        } => format!("symlink({}, {})", format_expr(old), format_expr(new)),
        ExprKind::Readlink(e) => format!("readlink {}", format_expr(e)),
        ExprKind::Glob(_) => format!("/* ExprKind::Glob */"),
        ExprKind::GlobPar(_) => format!("/* ExprKind::GlobPar */"),
        ExprKind::Bless {
            ref_expr: Box<Expr>,
        class: Option<Box<Expr>>,
        } => match class {
                Some(c) => format!("bless({}, {})", format_expr(ref_expr), format_expr(c)),
                None => format!("bless({})", format_expr(ref_expr)),
            },
        ExprKind::Caller(_) => format!("/* ExprKind::Caller */"),
        ExprKind::Wantarray => "wantarray".to_string(),
        ExprKind::List(_) => format!("/* ExprKind::List */"),
        ExprKind::PostfixIf {
            expr: Box<Expr>,
        condition: Box<Expr>,
        } => format!("{} if {}", format_expr(expr), format_expr(condition)),
        ExprKind::PostfixUnless {
            expr: Box<Expr>,
        condition: Box<Expr>,
        } => format!("{} unless {}", format_expr(expr), format_expr(condition)),
        ExprKind::PostfixWhile {
            expr: Box<Expr>,
        condition: Box<Expr>,
        } => format!("{} while {}", format_expr(expr), format_expr(condition)),
        ExprKind::PostfixUntil {
            expr: Box<Expr>,
        condition: Box<Expr>,
        } => format!("{} until {}", format_expr(expr), format_expr(condition)),
        ExprKind::PostfixForeach {
            expr: Box<Expr>,
        list: Box<Expr>,
        } => format!("{} foreach {}", format_expr(expr), format_expr(list)),
    }
}
