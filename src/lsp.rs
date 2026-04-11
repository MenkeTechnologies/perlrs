//! Language server protocol (stdio) for editors — `pe --lsp`.

use std::collections::{BTreeSet, HashMap};
use std::sync::OnceLock;

use lsp_server::{Connection, ErrorCode, ExtractError, Message, Notification, ProtocolError, Request, Response};
use lsp_types::notification::Notification as NotificationTrait;
use lsp_types::notification::{
    DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, PublishDiagnostics,
};
use lsp_types::request::Completion;
use lsp_types::request::DocumentSymbolRequest;
use lsp_types::request::Request as RequestTrait;
use lsp_types::request::ResolveCompletionItem;
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionOptions, CompletionParams, CompletionResponse,
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams, Diagnostic,
    DiagnosticSeverity, DocumentSymbolParams, DocumentSymbolResponse, Documentation, Location,
    OneOf, PublishDiagnosticsParams, Range, ServerCapabilities, SymbolInformation, SymbolKind,
    TextDocumentContentChangeEvent, TextDocumentSyncCapability, TextDocumentSyncKind,
    TextDocumentSyncOptions, Uri,
};
use lsp_types::{InsertTextFormat, MarkupContent, MarkupKind};
use percent_encoding::percent_decode_str;

use crate::ast::{Block, Sigil, StmtKind, SubSigParam, Statement, VarDecl};
use crate::ast::MatchArrayElem;
use crate::error::{ErrorKind, PerlError};
use crate::interpreter::Interpreter;

pub(crate) fn run_stdio() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (conn, io_threads) = Connection::stdio();

    let (init_id, init_params) = conn.initialize_start()?;
    let _: lsp_types::InitializeParams = serde_json::from_value(init_params).unwrap_or_default();

    let caps = ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Options(TextDocumentSyncOptions {
            open_close: Some(true),
            change: Some(TextDocumentSyncKind::FULL),
            will_save: None,
            will_save_wait_until: None,
            save: None,
        })),
        completion_provider: Some(CompletionOptions {
            resolve_provider: Some(true),
            trigger_characters: Some(vec![
                "$".to_string(),
                "@".to_string(),
                "%".to_string(),
                ":".to_string(),
            ]),
            ..Default::default()
        }),
        document_symbol_provider: Some(OneOf::Left(true)),
        ..Default::default()
    };

    let init_result = serde_json::json!({
        "capabilities": caps,
        "serverInfo": {
            "name": "perlrs",
            "version": env!("CARGO_PKG_VERSION"),
        }
    });
    conn.initialize_finish(init_id, init_result)?;

    let mut docs: HashMap<Uri, String> = HashMap::new();

    for msg in &conn.receiver {
        match msg {
            Message::Request(req) => {
                if conn.handle_shutdown(&req)? {
                    break;
                }
                dispatch_request(&conn, &mut docs, req)?;
            }
            Message::Notification(not) => {
                dispatch_notification(&conn, &mut docs, not)?;
            }
            Message::Response(_) => {}
        }
    }

    io_threads.join()?;
    Ok(())
}

fn dispatch_request(
    conn: &Connection,
    docs: &mut HashMap<Uri, String>,
    req: Request,
) -> Result<(), ProtocolError> {
    if req.method == DocumentSymbolRequest::METHOD {
        let req_id = req.id.clone();
        let (id, params) = match req.extract(DocumentSymbolRequest::METHOD) {
            Ok(p) => p,
            Err(ExtractError::JsonError { method, error }) => {
                let resp = Response::new_err(
                    req_id,
                    ErrorCode::InvalidParams as i32,
                    format!("{method}: {error}"),
                );
                conn.sender.send(resp.into()).expect("lsp channel");
                return Ok(());
            }
            Err(ExtractError::MethodMismatch(_)) => unreachable!(),
        };
        let result = document_symbols(docs, params);
        let resp = Response::new_ok(id, result);
        conn.sender.send(resp.into()).expect("lsp channel");
        return Ok(());
    }

    if req.method == Completion::METHOD {
        let req_id = req.id.clone();
        let (id, params) = match req.extract::<CompletionParams>(Completion::METHOD) {
            Ok(p) => p,
            Err(ExtractError::JsonError { method, error }) => {
                let resp = Response::new_err(
                    req_id,
                    ErrorCode::InvalidParams as i32,
                    format!("{method}: {error}"),
                );
                conn.sender.send(resp.into()).expect("lsp channel");
                return Ok(());
            }
            Err(ExtractError::MethodMismatch(_)) => unreachable!(),
        };
        let result = completions(docs, params);
        let resp = Response::new_ok(id, result);
        conn.sender.send(resp.into()).expect("lsp channel");
        return Ok(());
    }

    if req.method == ResolveCompletionItem::METHOD {
        let req_id = req.id.clone();
        let (id, item) = match req.extract::<CompletionItem>(ResolveCompletionItem::METHOD) {
            Ok(p) => p,
            Err(ExtractError::JsonError { method, error }) => {
                let resp = Response::new_err(
                    req_id,
                    ErrorCode::InvalidParams as i32,
                    format!("{method}: {error}"),
                );
                conn.sender.send(resp.into()).expect("lsp channel");
                return Ok(());
            }
            Err(ExtractError::MethodMismatch(_)) => unreachable!(),
        };
        let resolved = resolve_completion_item(item);
        let resp = Response::new_ok(id, resolved);
        conn.sender.send(resp.into()).expect("lsp channel");
        return Ok(());
    }

    let resp = Response::new_err(
        req.id,
        ErrorCode::MethodNotFound as i32,
        format!("perlrs LSP: unimplemented request {}", req.method),
    );
    conn.sender.send(resp.into()).expect("lsp channel");
    Ok(())
}

fn dispatch_notification(
    conn: &Connection,
    docs: &mut HashMap<Uri, String>,
    not: Notification,
) -> Result<(), ProtocolError> {
    if let Ok(params) = not.clone().extract::<DidOpenTextDocumentParams>(DidOpenTextDocument::METHOD) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        docs.insert(uri.clone(), text.clone());
        publish_diagnostics(conn, &uri, &text, &uri_to_path(&uri))?;
        return Ok(());
    }
    if let Ok(params) = not
        .clone()
        .extract::<DidChangeTextDocumentParams>(DidChangeTextDocument::METHOD)
    {
        let uri = params.text_document.uri;
        let text = merge_full_change(docs.get(&uri), params.content_changes);
        docs.insert(uri.clone(), text.clone());
        publish_diagnostics(conn, &uri, &text, &uri_to_path(&uri))?;
        return Ok(());
    }
    if let Ok(params) = not.extract::<DidCloseTextDocumentParams>(DidCloseTextDocument::METHOD) {
        let uri = params.text_document.uri;
        docs.remove(&uri);
        let n = Notification::new(
            PublishDiagnostics::METHOD.to_string(),
            PublishDiagnosticsParams::new(uri, Vec::new(), None),
        );
        conn.sender.send(n.into()).expect("lsp channel");
        return Ok(());
    }
    Ok(())
}

fn merge_full_change(prev: Option<&String>, changes: Vec<TextDocumentContentChangeEvent>) -> String {
    if let Some(c) = changes.into_iter().last() {
        return c.text;
    }
    prev.cloned().unwrap_or_default()
}

fn uri_to_path(uri: &Uri) -> String {
    let s = uri.as_str();
    if let Some(rest) = s.strip_prefix("file://") {
        let path_bytes = if rest.starts_with('/') {
            rest.as_bytes()
        } else if let Some(i) = rest.find('/') {
            rest[i..].as_bytes()
        } else {
            rest.as_bytes()
        };
        return percent_decode_str(std::str::from_utf8(path_bytes).unwrap_or(""))
            .decode_utf8_lossy()
            .to_string();
    }
    s.to_string()
}

fn publish_diagnostics(
    conn: &Connection,
    uri: &Uri,
    text: &str,
    path: &str,
) -> Result<(), ProtocolError> {
    let diagnostics = compute_diagnostics(text, path);
    let n = Notification::new(
        PublishDiagnostics::METHOD.to_string(),
        PublishDiagnosticsParams::new(uri.clone(), diagnostics, None),
    );
    conn.sender.send(n.into()).expect("lsp channel");
    Ok(())
}

fn compute_diagnostics(text: &str, path: &str) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    match crate::parse_with_file(text, path) {
        Err(e) => out.push(perror_to_diagnostic(&e, text)),
        Ok(program) => {
            let mut interp = Interpreter::new();
            interp.file = path.to_string();
            if let Err(e) = crate::lint_program(&program, &mut interp) {
                out.push(perror_to_diagnostic(&e, text));
            }
        }
    }
    out
}

fn perror_to_diagnostic(e: &PerlError, source: &str) -> Diagnostic {
    let severity = Some(match e.kind {
        ErrorKind::Syntax | ErrorKind::Runtime | ErrorKind::Type | ErrorKind::UndefinedVariable
        | ErrorKind::UndefinedSubroutine | ErrorKind::Regex | ErrorKind::DivisionByZero
        | ErrorKind::Die => DiagnosticSeverity::ERROR,
        ErrorKind::FileNotFound | ErrorKind::IO => DiagnosticSeverity::WARNING,
        ErrorKind::Exit(_) => DiagnosticSeverity::HINT,
    });
    let range = line_range_utf16(source, e.line.max(1));
    Diagnostic {
        range,
        severity,
        code: None,
        code_description: None,
        source: Some("perlrs".to_string()),
        message: e.message.clone(),
        related_information: None,
        tags: None,
        data: None,
    }
}

/// Map1-based line to a single-line range; `character` is UTF-16 offset per LSP.
fn line_range_utf16(source: &str, line_1based: usize) -> Range {
    let lines: Vec<&str> = source.lines().collect();
    let n = lines.len().max(1);
    let idx = line_1based.saturating_sub(1).min(n.saturating_sub(1));
    let line = lines.get(idx).copied().unwrap_or("");
    let end16 = line.encode_utf16().count() as u32;
    lsp_types::Range {
        start: lsp_types::Position {
            line: idx as u32,
            character: 0,
        },
        end: lsp_types::Position {
            line: idx as u32,
            character: end16,
        },
    }
}

fn document_symbols(
    docs: &HashMap<Uri, String>,
    params: DocumentSymbolParams,
) -> Option<DocumentSymbolResponse> {
    let uri = params.text_document.uri;
    let text = docs.get(&uri)?;
    let path = uri_to_path(&uri);
    let program = crate::parse_with_file(text, &path).ok()?;
    let mut symbols = Vec::new();
    for stmt in &program.statements {
        walk_stmt(stmt, &uri, text, &mut symbols, None);
    }
    Some(DocumentSymbolResponse::Flat(symbols))
}

fn walk_stmt(
    stmt: &Statement,
    uri: &Uri,
    source: &str,
    symbols: &mut Vec<SymbolInformation>,
    container: Option<&str>,
) {
    match &stmt.kind {
        StmtKind::SubDecl { name, body, .. } => {
            symbols.push(sym(
                format!("sub {name}"),
                SymbolKind::FUNCTION,
                uri,
                source,
                stmt.line,
                container,
            ));
            walk_block(body, uri, source, symbols, Some(name.as_str()));
        }
        StmtKind::Package { name } => {
            symbols.push(sym(
                format!("package {name}"),
                SymbolKind::MODULE,
                uri,
                source,
                stmt.line,
                container,
            ));
        }
        StmtKind::StructDecl { def } => {
            symbols.push(sym(
                format!("struct {}", def.name),
                SymbolKind::STRUCT,
                uri,
                source,
                stmt.line,
                container,
            ));
        }
        StmtKind::FormatDecl { name, .. } => {
            symbols.push(sym(
                format!("format {name}"),
                SymbolKind::METHOD,
                uri,
                source,
                stmt.line,
                container,
            ));
        }
        StmtKind::Block(b)
        | StmtKind::StmtGroup(b)
        | StmtKind::Begin(b)
        | StmtKind::End(b)
        | StmtKind::UnitCheck(b)
        | StmtKind::Check(b)
        | StmtKind::Init(b)
        | StmtKind::Continue(b) => walk_block(b, uri, source, symbols, container),
        StmtKind::If {
            body,
            elsifs,
            else_block,
            ..
        } => {
            walk_block(body, uri, source, symbols, container);
            for (_, b) in elsifs {
                walk_block(b, uri, source, symbols, container);
            }
            if let Some(b) = else_block {
                walk_block(b, uri, source, symbols, container);
            }
        }
        StmtKind::Unless {
            body,
            else_block,
            ..
        } => {
            walk_block(body, uri, source, symbols, container);
            if let Some(b) = else_block {
                walk_block(b, uri, source, symbols, container);
            }
        }
        StmtKind::While {
            body,
            continue_block,
            ..
        }
        | StmtKind::Until {
            body,
            continue_block,
            ..
        } => {
            walk_block(body, uri, source, symbols, container);
            if let Some(b) = continue_block {
                walk_block(b, uri, source, symbols, container);
            }
        }
        StmtKind::DoWhile { body, .. } => walk_block(body, uri, source, symbols, container),
        StmtKind::For {
            init,
            body,
            continue_block,
            ..
        } => {
            if let Some(init) = init {
                walk_stmt(init, uri, source, symbols, container);
            }
            walk_block(body, uri, source, symbols, container);
            if let Some(b) = continue_block {
                walk_block(b, uri, source, symbols, container);
            }
        }
        StmtKind::Foreach {
            body,
            continue_block,
            ..
        } => {
            walk_block(body, uri, source, symbols, container);
            if let Some(b) = continue_block {
                walk_block(b, uri, source, symbols, container);
            }
        }
        StmtKind::EvalTimeout { body, .. } => walk_block(body, uri, source, symbols, container),
        StmtKind::TryCatch {
            try_block,
            catch_block,
            finally_block,
            ..
        } => {
            walk_block(try_block, uri, source, symbols, container);
            walk_block(catch_block, uri, source, symbols, container);
            if let Some(b) = finally_block {
                walk_block(b, uri, source, symbols, container);
            }
        }
        StmtKind::Given { body, .. } => walk_block(body, uri, source, symbols, container),
        StmtKind::When { body, .. } | StmtKind::DefaultCase { body } => {
            walk_block(body, uri, source, symbols, container);
        }
        _ => {}
    }
}

fn walk_block(
    block: &Block,
    uri: &Uri,
    source: &str,
    symbols: &mut Vec<SymbolInformation>,
    container: Option<&str>,
) {
    for stmt in block {
        walk_stmt(stmt, uri, source, symbols, container);
    }
}

#[allow(deprecated)]
fn sym(
    name: String,
    kind: SymbolKind,
    uri: &Uri,
    source: &str,
    line: usize,
    container_name: Option<&str>,
) -> SymbolInformation {
    let range = line_range_utf16(source, line.max(1));
    SymbolInformation {
        name,
        kind,
        tags: None,
        deprecated: None,
        location: Location {
            uri: uri.clone(),
            range,
        },
        container_name: container_name.map(|s| s.to_string()),
    }
}

// ── completion: builtins, file subs/vars, sigils, `Foo::`, snippets, resolve ─

static COMPLETION_WORDS: OnceLock<Vec<String>> = OnceLock::new();

#[derive(Default)]
struct CompletionIndex {
    scalars: BTreeSet<String>,
    arrays: BTreeSet<String>,
    hashes: BTreeSet<String>,
    subs_short: BTreeSet<String>,
    subs_qualified: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum LineCompletionMode {
    Bare(String),
    Scalar(String),
    Array(String),
    Hash(String),
}

fn completion_words() -> &'static Vec<String> {
    COMPLETION_WORDS.get_or_init(|| {
        let mut v: Vec<String> = include_str!("lsp_completion_words.txt")
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(|s| s.to_string())
            .collect();
        v.sort_unstable();
        v.dedup();
        v
    })
}

fn perl_keyword_kind(word: &str) -> Option<CompletionItemKind> {
    const KW: &[&str] = &[
        "and", "async", "catch", "continue", "default", "do", "else", "elsif", "eval", "finally",
        "for", "foreach", "given", "if", "last", "local", "my", "next", "no", "not", "or", "our",
        "package", "redo", "return", "state", "sub", "try", "unless", "until", "use", "when",
        "while",
    ];
    KW.binary_search(&word).ok()?;
    Some(CompletionItemKind::KEYWORD)
}

fn doc_markup(text: &'static str) -> Documentation {
    Documentation::MarkupContent(MarkupContent {
        kind: MarkupKind::Markdown,
        value: text.to_string(),
    })
}

/// Short markdown for common symbols; `completionItem/resolve` fills this when missing.
fn doc_for_label(label: &str) -> Option<Documentation> {
    let key = label.strip_suffix(" …").unwrap_or(label);
    let md: &'static str = match key {
        "my" => "Declare a lexical: `my $x;`, `my @a;`, `my %h;`.",
        "our" => "Declare a package-global lexical alias in the current package.",
        "sub" => "Define a subroutine: `sub name { ... }`.",
        "package" => "Set the current package for subsequent declarations.",
        "say" => "Print operands with newline (`-E` / perlrs).",
        "print" => "Print to the selected handle (default `STDOUT`).",
        "map" => "Evaluate a block or expression for each list element.",
        "grep" => "Filter a list by boolean block or expression.",
        "sort" => "Sort a list (optional comparator block).",
        "foreach" | "for" => "`for` / `foreach` loop over a list.",
        "pmap" => "Parallel `map` (rayon); order preserved for scalar results.",
        "cluster" => "Build an SSH worker pool: `cluster([\"host:8\", ...])`.",
        "pmap_on" => "Distributed `pmap` over `cluster` via `pe --remote-worker`.",
        "try" => "`try { } catch ($e) { } [ finally { } ]` exception handling.",
        "given" => "Switch-like `given (EXPR) { when ... default ... }`.",
        _ => return None,
    };
    Some(doc_markup(md))
}

fn resolve_completion_item(mut item: CompletionItem) -> CompletionItem {
    if item.documentation.is_none() {
        if let Some(doc) = doc_for_label(&item.label) {
            item.documentation = Some(doc);
        }
    }
    item
}

fn completions(docs: &HashMap<Uri, String>, params: CompletionParams) -> Option<CompletionResponse> {
    let uri = params.text_document_position.text_document.uri;
    let text = docs.get(&uri)?;
    let pos = params.text_document_position.position;
    let path = uri_to_path(&uri);

    let lines: Vec<&str> = text.lines().collect();
    let line_text = lines.get(pos.line as usize).copied()?;
    let byte_col = utf16_col_to_byte_idx(line_text, pos.character);
    let mode = line_completion_mode(line_text, byte_col);

    let mut idx = CompletionIndex::default();
    if let Ok(program) = crate::parse_with_file(text, &path) {
        let mut pkg = String::from("main");
        for stmt in &program.statements {
            visit_stmt_for_index(stmt, &mut pkg, &mut idx);
        }
        idx.arrays.insert("_".to_string());
    }

    let mut items: Vec<CompletionItem> = match &mode {
        LineCompletionMode::Scalar(f) => sigil_completions(f, '$', &idx.scalars, "scalar"),
        LineCompletionMode::Array(f) => sigil_completions(f, '@', &idx.arrays, "array"),
        LineCompletionMode::Hash(f) => sigil_completions(f, '%', &idx.hashes, "hash"),
        LineCompletionMode::Bare(f) => {
            if let Some((pkg_p, partial)) = split_qualified_prefix(f) {
                qualified_sub_completions(&pkg_p, &partial, &idx)
            } else {
                bare_completions(f, &idx)
            }
        }
    };

    if matches!(mode, LineCompletionMode::Bare(_)) {
        let f = match &mode {
            LineCompletionMode::Bare(s) => s.as_str(),
            _ => "",
        };
        push_snippet_completions(f, &mut items);
    }

    items.sort_by(|a, b| a.label.cmp(&b.label));
    items.truncate(384);
    Some(CompletionResponse::Array(items))
}

fn line_completion_mode(line: &str, byte_col: usize) -> LineCompletionMode {
    let b = byte_col.min(line.len());
    let before = &line[..b];
    let mut start = b;
    for (i, c) in before.char_indices().rev() {
        if c.is_ascii_alphanumeric() || c == '_' || c == ':' {
            start = i;
        } else {
            break;
        }
    }
    let raw = before[start..b].to_string();
    if start > 0 {
        let prev = line[..start].chars().last();
        match prev {
            Some('$') => return LineCompletionMode::Scalar(raw),
            Some('@') => return LineCompletionMode::Array(raw),
            Some('%') => return LineCompletionMode::Hash(raw),
            _ => {}
        }
    }
    LineCompletionMode::Bare(raw)
}

fn split_qualified_prefix(raw: &str) -> Option<(String, String)> {
    if !raw.contains("::") {
        return None;
    }
    let (pkg, tail) = raw.rsplit_once("::")?;
    if pkg.is_empty() {
        return None;
    }
    Some((format!("{pkg}::"), tail.to_string()))
}

fn fqn_matches(pkg_prefix: &str, partial: &str, fqn: &str) -> bool {
    let Some(rest) = fqn.strip_prefix(pkg_prefix) else {
        return false;
    };
    partial.is_empty() || rest.starts_with(partial)
}

fn qualified_sub_completions(
    pkg_prefix: &str,
    partial: &str,
    idx: &CompletionIndex,
) -> Vec<CompletionItem> {
    let mut items = Vec::new();
    for fqn in &idx.subs_qualified {
        if !fqn_matches(pkg_prefix, partial, fqn) {
            continue;
        }
        let doc = doc_for_label(fqn.rsplit("::").next().unwrap_or(fqn.as_str()));
        items.push(CompletionItem {
            label: fqn.clone(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some("sub (qualified)".to_string()),
            documentation: doc,
            ..Default::default()
        });
    }
    items
}

fn sigil_completions(
    filter: &str,
    sigil: char,
    names: &BTreeSet<String>,
    kind: &'static str,
) -> Vec<CompletionItem> {
    let mut items = Vec::new();
    for n in names {
        if !filter.is_empty() && !n.starts_with(filter) {
            continue;
        }
        let insert = format!("{sigil}{n}");
        items.push(CompletionItem {
            label: insert.clone(),
            kind: Some(CompletionItemKind::VARIABLE),
            detail: Some(kind.to_string()),
            filter_text: Some(n.clone()),
            insert_text: Some(insert),
            ..Default::default()
        });
    }
    items
}

fn bare_completions(filter: &str, idx: &CompletionIndex) -> Vec<CompletionItem> {
    let mut items = Vec::new();
    for s in &idx.subs_short {
        if !filter.is_empty() && !s.starts_with(filter) {
            continue;
        }
        let doc = doc_for_label(s.as_str());
        items.push(CompletionItem {
            label: s.clone(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some("sub".to_string()),
            documentation: doc,
            ..Default::default()
        });
    }

    for w in completion_words() {
        if !filter.is_empty() && !w.starts_with(filter) {
            continue;
        }
        if idx.subs_short.contains(w.as_str()) {
            continue;
        }
        let kind = perl_keyword_kind(w).unwrap_or(CompletionItemKind::FUNCTION);
        let detail = if kind == CompletionItemKind::KEYWORD {
            "keyword"
        } else {
            "builtin"
        };
        let doc = doc_for_label(w);
        items.push(CompletionItem {
            label: w.clone(),
            kind: Some(kind),
            detail: Some(detail.to_string()),
            documentation: doc,
            ..Default::default()
        });
    }
    items
}

fn push_snippet_completions(filter: &str, items: &mut Vec<CompletionItem>) {
    const SNIPS: &[(&str, &str, &str)] = &[
        (
            "my",
            "my \\$${1:name} = ${2:undef};",
            "Lexical declaration (snippet)",
        ),
        ("sub", "sub ${1:name} {\n\t${0}\n}\n", "New subroutine (snippet)"),
        ("say", "say ${1:expr};", "say with placeholder (snippet)"),
        (
            "foreach",
            "foreach my \\$${1:var} (@${2:list}) {\n\t${0}\n}\n",
            "foreach loop (snippet)",
        ),
        (
            "if",
            "if (${1:condition}) {\n\t${0}\n}\n",
            "if block (snippet)",
        ),
    ];
    for (kw, body, detail) in SNIPS {
        if !filter.is_empty() && !kw.starts_with(filter) {
            continue;
        }
        items.push(CompletionItem {
            label: format!("{kw} …"),
            kind: Some(CompletionItemKind::SNIPPET),
            detail: Some((*detail).to_string()),
            filter_text: Some((*kw).to_string()),
            insert_text: Some((*body).to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        });
    }
}

fn collect_var_decls(decls: &[VarDecl], idx: &mut CompletionIndex) {
    for d in decls {
        match d.sigil {
            Sigil::Scalar => {
                idx.scalars.insert(d.name.clone());
            }
            Sigil::Array => {
                idx.arrays.insert(d.name.clone());
            }
            Sigil::Hash => {
                idx.hashes.insert(d.name.clone());
            }
            Sigil::Typeglob => {}
        }
    }
}

fn collect_array_pattern(elems: &[MatchArrayElem], idx: &mut CompletionIndex) {
    for e in elems {
        match e {
            MatchArrayElem::CaptureScalar(n) => {
                idx.scalars.insert(n.clone());
            }
            MatchArrayElem::RestBind(n) => {
                idx.arrays.insert(n.clone());
            }
            MatchArrayElem::Expr(_) | MatchArrayElem::Rest => {}
        }
    }
}

fn collect_sub_params(params: &[SubSigParam], idx: &mut CompletionIndex) {
    for p in params {
        match p {
            SubSigParam::Scalar(n) => {
                idx.scalars.insert(n.clone());
            }
            SubSigParam::ArrayDestruct(elems) => collect_array_pattern(elems, idx),
            SubSigParam::HashDestruct(pairs) => {
                for (_key, name) in pairs {
                    idx.scalars.insert(name.clone());
                }
            }
        }
    }
}

fn visit_block_for_index(block: &Block, pkg: &mut String, idx: &mut CompletionIndex) {
    for stmt in block {
        visit_stmt_for_index(stmt, pkg, idx);
    }
}

fn visit_stmt_for_index(stmt: &Statement, pkg: &mut String, idx: &mut CompletionIndex) {
    match &stmt.kind {
        StmtKind::Package { name } => {
            *pkg = name.clone();
        }
        StmtKind::SubDecl {
            name,
            params,
            body,
            ..
        } => {
            let fqn = if name.contains("::") {
                name.clone()
            } else {
                format!("{}::{}", pkg.as_str(), name)
            };
            idx.subs_qualified.insert(fqn);
            idx.subs_short.insert(name.clone());
            idx.subs_short.insert(name.rsplit("::").next().unwrap_or(name.as_str()).to_string());
            collect_sub_params(params, idx);
            visit_block_for_index(body, pkg, idx);
        }
        StmtKind::My(decls)
        | StmtKind::Our(decls)
        | StmtKind::Local(decls)
        | StmtKind::MySync(decls) => {
            collect_var_decls(decls, idx);
        }
        StmtKind::Foreach { var, body, continue_block, .. } => {
            idx.scalars.insert(var.clone());
            visit_block_for_index(body, pkg, idx);
            if let Some(b) = continue_block {
                visit_block_for_index(b, pkg, idx);
            }
        }
        StmtKind::Block(b)
        | StmtKind::StmtGroup(b)
        | StmtKind::Begin(b)
        | StmtKind::End(b)
        | StmtKind::UnitCheck(b)
        | StmtKind::Check(b)
        | StmtKind::Init(b)
        | StmtKind::Continue(b) => visit_block_for_index(b, pkg, idx),
        StmtKind::If {
            body,
            elsifs,
            else_block,
            ..
        } => {
            visit_block_for_index(body, pkg, idx);
            for (_, b) in elsifs {
                visit_block_for_index(b, pkg, idx);
            }
            if let Some(b) = else_block {
                visit_block_for_index(b, pkg, idx);
            }
        }
        StmtKind::Unless {
            body,
            else_block,
            ..
        } => {
            visit_block_for_index(body, pkg, idx);
            if let Some(b) = else_block {
                visit_block_for_index(b, pkg, idx);
            }
        }
        StmtKind::While {
            body,
            continue_block,
            ..
        }
        | StmtKind::Until {
            body,
            continue_block,
            ..
        } => {
            visit_block_for_index(body, pkg, idx);
            if let Some(b) = continue_block {
                visit_block_for_index(b, pkg, idx);
            }
        }
        StmtKind::DoWhile { body, .. } => visit_block_for_index(body, pkg, idx),
        StmtKind::For {
            init,
            body,
            continue_block,
            ..
        } => {
            if let Some(init) = init {
                visit_stmt_for_index(init, pkg, idx);
            }
            visit_block_for_index(body, pkg, idx);
            if let Some(b) = continue_block {
                visit_block_for_index(b, pkg, idx);
            }
        }
        StmtKind::EvalTimeout { body, .. } => visit_block_for_index(body, pkg, idx),
        StmtKind::TryCatch {
            try_block,
            catch_block,
            finally_block,
            ..
        } => {
            visit_block_for_index(try_block, pkg, idx);
            visit_block_for_index(catch_block, pkg, idx);
            if let Some(b) = finally_block {
                visit_block_for_index(b, pkg, idx);
            }
        }
        StmtKind::Given { body, .. } => visit_block_for_index(body, pkg, idx),
        StmtKind::When { body, .. } | StmtKind::DefaultCase { body } => {
            visit_block_for_index(body, pkg, idx);
        }
        _ => {}
    }
}

fn utf16_col_to_byte_idx(line: &str, col16: u32) -> usize {
    let mut acc = 0u32;
    for (b, ch) in line.char_indices() {
        let w = ch.len_utf16() as u32;
        if acc + w > col16 {
            return b;
        }
        acc += w;
    }
    line.len()
}

#[cfg(test)]
mod completion_tests {
    use super::{
        line_completion_mode, split_qualified_prefix, utf16_col_to_byte_idx, LineCompletionMode,
    };

    fn raw_at(line: &str, col16: u32) -> (LineCompletionMode, String) {
        let b = utf16_col_to_byte_idx(line, col16);
        let m = line_completion_mode(line, b);
        let s = match &m {
            LineCompletionMode::Bare(x)
            | LineCompletionMode::Scalar(x)
            | LineCompletionMode::Array(x)
            | LineCompletionMode::Hash(x) => x.clone(),
        };
        (m, s)
    }

    #[test]
    fn bare_prefix_is_word_before_cursor_ascii() {
        let (m, s) = raw_at("ba", 2);
        assert!(matches!(m, LineCompletionMode::Bare(_)));
        assert_eq!(s, "ba");
        let (_, s) = raw_at("foo", 2);
        assert_eq!(s, "fo");
    }

    #[test]
    fn bare_prefix_empty_when_cursor_after_space() {
        let (_, s) = raw_at("print ", 6);
        assert_eq!(s, "");
    }

    #[test]
    fn sigil_modes() {
        let (m, s) = raw_at("$abc", 4);
        assert!(matches!(m, LineCompletionMode::Scalar(_)));
        assert_eq!(s, "abc");
        let (m, s) = raw_at("@things", 7);
        assert!(matches!(m, LineCompletionMode::Array(_)));
        assert_eq!(s, "things");
        let (m, s) = raw_at("%h", 2);
        assert!(matches!(m, LineCompletionMode::Hash(_)));
        assert_eq!(s, "h");
    }

    #[test]
    fn qualified_split() {
        assert_eq!(
            split_qualified_prefix("Foo::ba"),
            Some(("Foo::".to_string(), "ba".to_string()))
        );
        assert_eq!(
            split_qualified_prefix("Foo::"),
            Some(("Foo::".to_string(), "".to_string()))
        );
        assert_eq!(split_qualified_prefix("foo"), None);
        assert_eq!(split_qualified_prefix("::foo"), None);
    }
}
