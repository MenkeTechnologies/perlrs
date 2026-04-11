//! Framed bincode over stdin/stdout for `pe --remote-worker` (distributed `pmap_on`).

use std::collections::HashMap;
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::ast::Block;
use crate::interpreter::{FlowOrError, Interpreter};
use crate::value::{PerlSub, PerlValue};

/// One unit of work executed on a remote `pe --remote-worker`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteJobV1 {
    pub seq: u64,
    pub subs_prelude: String,
    pub block_src: String,
    pub capture: Vec<(String, serde_json::Value)>,
    pub item: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteRespV1 {
    pub seq: u64,
    pub ok: bool,
    pub result: serde_json::Value,
    pub err_msg: String,
}

const MAX_FRAME: usize = 256 * 1024 * 1024;

pub fn write_framed<W: Write>(w: &mut W, payload: &[u8]) -> std::io::Result<()> {
    w.write_all(&(payload.len() as u64).to_le_bytes())?;
    w.write_all(payload)?;
    w.flush()?;
    Ok(())
}

pub fn read_framed<R: Read>(r: &mut R) -> std::io::Result<Vec<u8>> {
    let mut h = [0u8; 8];
    r.read_exact(&mut h)?;
    let n = u64::from_le_bytes(h) as usize;
    if n > MAX_FRAME {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("remote frame too large: {n}"),
        ));
    }
    let mut v = vec![0u8; n];
    r.read_exact(&mut v)?;
    Ok(v)
}

pub fn encode_job(job: &RemoteJobV1) -> Result<Vec<u8>, String> {
    bincode::serialize(job).map_err(|e| e.to_string())
}

pub fn decode_job(bytes: &[u8]) -> Result<RemoteJobV1, String> {
    bincode::deserialize(bytes).map_err(|e| e.to_string())
}

pub fn encode_resp(resp: &RemoteRespV1) -> Result<Vec<u8>, String> {
    bincode::serialize(resp).map_err(|e| e.to_string())
}

pub fn decode_resp(bytes: &[u8]) -> Result<RemoteRespV1, String> {
    bincode::deserialize(bytes).map_err(|e| e.to_string())
}

pub fn perl_to_json_value(v: &PerlValue) -> Result<serde_json::Value, String> {
    if v.is_undef() {
        return Ok(serde_json::Value::Null);
    }
    if let Some(i) = v.as_integer() {
        return Ok(serde_json::json!(i));
    }
    if let Some(f) = v.as_float() {
        return Ok(serde_json::json!(f));
    }
    if v.is_string_like() {
        return Ok(serde_json::Value::String(v.to_string()));
    }
    if let Some(a) = v.as_array_vec() {
        let mut out = Vec::with_capacity(a.len());
        for x in a {
            out.push(perl_to_json_value(&x)?);
        }
        return Ok(serde_json::Value::Array(out));
    }
    if let Some(h) = v.as_hash_map() {
        let mut m = serde_json::Map::new();
        for (k, val) in h {
            m.insert(k.clone(), perl_to_json_value(&val)?);
        }
        return Ok(serde_json::Value::Object(m));
    }
    Err(format!(
        "value not supported for remote pmap (need null, bool/int/float/string/array/hash): {}",
        v.type_name()
    ))
}

pub fn json_to_perl(v: &serde_json::Value) -> Result<PerlValue, String> {
    Ok(match v {
        serde_json::Value::Null => PerlValue::UNDEF,
        serde_json::Value::Bool(b) => PerlValue::integer(if *b { 1 } else { 0 }),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                PerlValue::integer(i)
            } else if let Some(u) = n.as_u64() {
                PerlValue::integer(u as i64)
            } else {
                PerlValue::float(n.as_f64().unwrap_or(0.0))
            }
        }
        serde_json::Value::String(s) => PerlValue::string(s.clone()),
        serde_json::Value::Array(a) => {
            let mut items = Vec::with_capacity(a.len());
            for x in a {
                items.push(json_to_perl(x)?);
            }
            PerlValue::array(items)
        }
        serde_json::Value::Object(o) => {
            let mut map = indexmap::IndexMap::new();
            for (k, val) in o {
                map.insert(k.clone(), json_to_perl(val)?);
            }
            PerlValue::hash(map)
        }
    })
}

pub fn capture_entries_to_json(
    entries: &[(String, PerlValue)],
) -> Result<Vec<(String, serde_json::Value)>, String> {
    let mut out = Vec::with_capacity(entries.len());
    for (k, v) in entries {
        out.push((k.clone(), perl_to_json_value(v)?));
    }
    Ok(out)
}

pub fn build_subs_prelude(subs: &HashMap<String, Arc<PerlSub>>) -> String {
    let mut names: Vec<_> = subs.keys().cloned().collect();
    names.sort();
    let mut s = String::new();
    for name in names {
        let sub = &subs[&name];
        if sub.closure_env.is_some() {
            continue;
        }
        let sig = if !sub.params.is_empty() {
            format!(
                " ({})",
                sub.params
                    .iter()
                    .map(crate::fmt::format_sub_sig_param)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        } else if let Some(ref p) = sub.prototype {
            format!(" ({})", p)
        } else {
            String::new()
        };
        let body = crate::fmt::format_block(&sub.body);
        s.push_str(&format!("sub {}{} {{\n{}\n}}\n", name, sig, body));
    }
    s
}

/// Run one job in-process (for tests / local debugging).
pub fn run_job_local(job: &RemoteJobV1) -> RemoteRespV1 {
    let mut interp = Interpreter::new();
    let cap: Vec<(String, PerlValue)> = match job
        .capture
        .iter()
        .map(|(k, v)| json_to_perl(v).map(|pv| (k.clone(), pv)))
        .collect()
    {
        Ok(c) => c,
        Err(e) => {
            return RemoteRespV1 {
                seq: job.seq,
                ok: false,
                result: serde_json::Value::Null,
                err_msg: e,
            };
        }
    };
    interp.scope_push_hook();
    interp.scope.restore_capture(&cap);
    let item_pv = match json_to_perl(&job.item) {
        Ok(v) => v,
        Err(e) => {
            interp.scope_pop_hook();
            return RemoteRespV1 {
                seq: job.seq,
                ok: false,
                result: serde_json::Value::Null,
                err_msg: e,
            };
        }
    };
    let _ = interp.scope.set_scalar("_", item_pv);
    let full_src = format!("{}\n{}", job.subs_prelude, job.block_src);
    let prog = match crate::parse(&full_src) {
        Ok(p) => p,
        Err(e) => {
            interp.scope_pop_hook();
            return RemoteRespV1 {
                seq: job.seq,
                ok: false,
                result: serde_json::Value::Null,
                err_msg: e.message,
            };
        }
    };
    let block: Block = prog.statements;
    let r = match interp.exec_block_smart(&block) {
        Ok(v) => v,
        Err(e) => {
            interp.scope_pop_hook();
            let msg = match e {
                FlowOrError::Error(pe) => pe.to_string(),
                FlowOrError::Flow(f) => format!("unexpected control flow: {:?}", f),
            };
            return RemoteRespV1 {
                seq: job.seq,
                ok: false,
                result: serde_json::Value::Null,
                err_msg: msg,
            };
        }
    };
    interp.scope_pop_hook();
    match perl_to_json_value(&r) {
        Ok(j) => RemoteRespV1 {
            seq: job.seq,
            ok: true,
            result: j,
            err_msg: String::new(),
        },
        Err(e) => RemoteRespV1 {
            seq: job.seq,
            ok: false,
            result: serde_json::Value::Null,
            err_msg: e,
        },
    }
}

/// stdin/stdout worker loop: one framed request → one framed response, then exit 0.
pub fn run_remote_worker_stdio() -> i32 {
    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();
    let mut stdout = std::io::stdout();
    let payload = match read_framed(&mut stdin) {
        Ok(p) => p,
        Err(e) => {
            let _ = writeln!(std::io::stderr(), "remote-worker: read frame: {e}");
            return 1;
        }
    };
    let job = match decode_job(&payload) {
        Ok(j) => j,
        Err(e) => {
            let _ = writeln!(std::io::stderr(), "remote-worker: decode job: {e}");
            return 1;
        }
    };
    let resp = run_job_local(&job);
    let out = match encode_resp(&resp) {
        Ok(b) => b,
        Err(e) => {
            let _ = writeln!(std::io::stderr(), "remote-worker: encode resp: {e}");
            return 1;
        }
    };
    if let Err(e) = write_framed(&mut stdout, &out) {
        let _ = writeln!(std::io::stderr(), "remote-worker: write frame: {e}");
        return 1;
    }
    if resp.ok {
        0
    } else {
        let _ = writeln!(std::io::stderr(), "remote-worker: {}", resp.err_msg);
        2
    }
}

pub fn ssh_invoke_remote_worker(host: &str, pe_bin: &str, job: &RemoteJobV1) -> Result<RemoteRespV1, String> {
    let payload = encode_job(job)?;
    let mut child = Command::new("ssh")
        .arg(host)
        .arg(pe_bin)
        .arg("--remote-worker")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("ssh: {e}"))?;
    let mut stdin = child.stdin.take().ok_or_else(|| "ssh: stdin".to_string())?;
    write_framed(&mut stdin, &payload).map_err(|e| format!("ssh stdin: {e}"))?;
    drop(stdin);
       let mut stdout = child.stdout.take().ok_or_else(|| "ssh: stdout".to_string())?;
    let mut stderr = child.stderr.take().ok_or_else(|| "ssh: stderr".to_string())?;
    let stderr_task = std::thread::spawn(move || {
        let mut s = String::new();
        let _ = stderr.read_to_string(&mut s);
        s
    });
    let out_bytes = read_framed(&mut stdout).map_err(|e| format!("ssh read frame: {e}"))?;
    let status = child.wait().map_err(|e| format!("ssh wait: {e}"))?;
    let stderr_text = stderr_task.join().unwrap_or_default();
    if !status.success() {
        return Err(format!(
            "ssh remote pe exited {:?}: {}",
            status.code(),
            stderr_text.trim()
        ));
    }
    decode_resp(&out_bytes).map_err(|e| {
        format!(
            "decode remote response: {e}; stderr: {}",
            stderr_text.trim()
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_roundtrip_doubles() {
        let job = RemoteJobV1 {
            seq: 0,
            subs_prelude: String::new(),
            block_src: "$_ * 2;".to_string(),
            capture: vec![],
            item: serde_json::json!(21),
        };
        let r = run_job_local(&job);
        assert!(r.ok, "{}", r.err_msg);
        assert_eq!(r.result, serde_json::json!(42));
    }
}
