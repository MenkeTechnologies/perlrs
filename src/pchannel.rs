//! Typed message channels for parallel blocks (`pchannel`, `send`, `recv`, `pselect`).

use std::sync::Arc;
use std::time::Duration;

use crossbeam::channel::{self, Receiver, Select};

use crate::error::{PerlError, PerlResult};
use crate::value::PerlValue;

/// `pchannel()` — two-element list `(tx, rx)` for `my ($tx, $rx) = pchannel`.
pub fn create_pair() -> PerlValue {
    let (tx, rx) = channel::unbounded();
    PerlValue::array(vec![
        PerlValue::channel_tx(Arc::new(tx)),
        PerlValue::channel_rx(Arc::new(rx)),
    ])
}

/// `pchannel(N)` — bounded channel capacity `N`.
pub fn create_bounded_pair(capacity: usize) -> PerlValue {
    let (tx, rx) = channel::bounded(capacity);
    PerlValue::array(vec![
        PerlValue::channel_tx(Arc::new(tx)),
        PerlValue::channel_rx(Arc::new(rx)),
    ])
}

/// Multiplexed receive — [`crossbeam_channel::Select`] over several `pchannel` receivers.
/// Returns `(value, index)` where `index` is **0-based** (first argument is `0`), like Go's `select`.
pub fn pselect_recv(args: &[PerlValue], line: usize) -> PerlResult<PerlValue> {
    if args.is_empty() {
        return Err(PerlError::runtime(
            "pselect() expects at least one pchannel receiver",
            line,
        ));
    }
    let mut rx_owned: Vec<Arc<Receiver<PerlValue>>> = Vec::with_capacity(args.len());
    for v in args {
        if let Some(rx) = v.as_channel_rx() {
            rx_owned.push(rx);
        } else {
            return Err(PerlError::runtime(
                "pselect() arguments must be pchannel receivers",
                line,
            ));
        }
    }
    let rx_refs: Vec<&Receiver<PerlValue>> = rx_owned.iter().map(|a| a.as_ref()).collect();
    let mut sel = Select::new();
    for r in &rx_refs {
        sel.recv(r);
    }
    let oper = sel.select();
    let idx = oper.index();
    let val = match oper.recv(rx_refs[idx]) {
        Ok(v) => v,
        Err(_) => PerlValue::UNDEF,
    };
    Ok(PerlValue::array(vec![val, PerlValue::integer(idx as i64)]))
}

/// Like [`pselect_recv`], with optional overall timeout. On timeout returns `(undef, -1)`.
pub fn pselect_recv_with_optional_timeout(
    args: &[PerlValue],
    timeout: Option<Duration>,
    line: usize,
) -> PerlResult<PerlValue> {
    if args.is_empty() {
        return Err(PerlError::runtime(
            "pselect() expects at least one pchannel receiver",
            line,
        ));
    }
    if timeout.is_none() {
        return pselect_recv(args, line);
    }
    let duration = timeout.unwrap();
    let mut rx_owned: Vec<Arc<Receiver<PerlValue>>> = Vec::with_capacity(args.len());
    for v in args {
        if let Some(rx) = v.as_channel_rx() {
            rx_owned.push(rx);
        } else {
            return Err(PerlError::runtime(
                "pselect() arguments must be pchannel receivers",
                line,
            ));
        }
    }
    let rx_refs: Vec<&Receiver<PerlValue>> = rx_owned.iter().map(|a| a.as_ref()).collect();
    let mut sel = Select::new();
    for r in &rx_refs {
        sel.recv(r);
    }
    let oper = sel.select_timeout(duration);
    let Ok(oper) = oper else {
        return Ok(PerlValue::array(vec![
            PerlValue::UNDEF,
            PerlValue::integer(-1),
        ]));
    };
    let idx = oper.index();
    let val = match oper.recv(rx_refs[idx]) {
        Ok(v) => v,
        Err(_) => PerlValue::UNDEF,
    };
    Ok(PerlValue::array(vec![val, PerlValue::integer(idx as i64)]))
}

/// `$tx->send($v)` and `$rx->recv` without package subs.
pub fn dispatch_method(
    receiver: &PerlValue,
    method: &str,
    args: &[PerlValue],
    line: usize,
) -> Option<PerlResult<PerlValue>> {
    if method == "send" {
        if let Some(tx) = receiver.as_channel_tx() {
            if args.len() != 1 {
                return Some(Err(PerlError::runtime(
                    "send() on pchannel tx expects exactly one value",
                    line,
                )));
            }
            let ok = tx.send(args[0].clone()).is_ok();
            return Some(Ok(PerlValue::integer(if ok { 1 } else { 0 })));
        }
    }
    if method == "recv" {
        if let Some(rx) = receiver.as_channel_rx() {
            if !args.is_empty() {
                return Some(Err(PerlError::runtime(
                    "recv() on pchannel rx takes no arguments",
                    line,
                )));
            }
            return Some(Ok(match rx.recv() {
                Ok(v) => v,
                Err(_) => PerlValue::UNDEF,
            }));
        }
    }
    None
}
