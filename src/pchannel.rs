//! Typed message channels for parallel blocks (`pchannel`, `send`, `recv`).

use std::sync::Arc;

use crossbeam::channel;

use crate::error::{PerlError, PerlResult};
use crate::value::PerlValue;

/// `pchannel()` — two-element list `(tx, rx)` for `my ($tx, $rx) = pchannel`.
pub fn create_pair() -> PerlValue {
    let (tx, rx) = channel::unbounded();
    PerlValue::Array(vec![
        PerlValue::ChannelTx(Arc::new(tx)),
        PerlValue::ChannelRx(Arc::new(rx)),
    ])
}

/// `$tx->send($v)` and `$rx->recv` without package subs.
pub fn dispatch_method(
    receiver: &PerlValue,
    method: &str,
    args: &[PerlValue],
    line: usize,
) -> Option<PerlResult<PerlValue>> {
    match (receiver, method) {
        (PerlValue::ChannelTx(tx), "send") => {
            if args.len() != 1 {
                return Some(Err(PerlError::runtime(
                    "send() on pchannel tx expects exactly one value",
                    line,
                )));
            }
            let ok = tx.send(args[0].clone()).is_ok();
            Some(Ok(PerlValue::Integer(if ok { 1 } else { 0 })))
        }
        (PerlValue::ChannelRx(rx), "recv") => {
            if !args.is_empty() {
                return Some(Err(PerlError::runtime(
                    "recv() on pchannel rx takes no arguments",
                    line,
                )));
            }
            Some(Ok(match rx.recv() {
                Ok(v) => v,
                Err(_) => PerlValue::Undef,
            }))
        }
        _ => None,
    }
}
