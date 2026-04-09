//! Perl-style filesystem helpers (`stat`, `glob`, etc.).

use crate::value::PerlValue;

/// 13-element `stat` / `lstat` list (empty vector on failure).
pub fn stat_path(path: &str, symlink: bool) -> PerlValue {
    let res = if symlink {
        std::fs::symlink_metadata(path)
    } else {
        std::fs::metadata(path)
    };
    match res {
        Ok(meta) => PerlValue::Array(perl_stat_from_metadata(&meta)),
        Err(_) => PerlValue::Array(vec![]),
    }
}

pub fn perl_stat_from_metadata(meta: &std::fs::Metadata) -> Vec<PerlValue> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        vec![
            PerlValue::Integer(meta.dev() as i64),
            PerlValue::Integer(meta.ino() as i64),
            PerlValue::Integer(meta.mode() as i64),
            PerlValue::Integer(meta.nlink() as i64),
            PerlValue::Integer(meta.uid() as i64),
            PerlValue::Integer(meta.gid() as i64),
            PerlValue::Integer(meta.rdev() as i64),
            PerlValue::Integer(meta.len() as i64),
            PerlValue::Integer(meta.atime()),
            PerlValue::Integer(meta.mtime()),
            PerlValue::Integer(meta.ctime()),
            PerlValue::Integer(meta.blksize() as i64),
            PerlValue::Integer(meta.blocks() as i64),
        ]
    }
    #[cfg(not(unix))]
    {
        let len = meta.len() as i64;
        vec![
            PerlValue::Integer(0),
            PerlValue::Integer(0),
            PerlValue::Integer(0),
            PerlValue::Integer(0),
            PerlValue::Integer(0),
            PerlValue::Integer(0),
            PerlValue::Integer(0),
            PerlValue::Integer(len),
            PerlValue::Integer(0),
            PerlValue::Integer(0),
            PerlValue::Integer(0),
            PerlValue::Integer(0),
            PerlValue::Integer(0),
        ]
    }
}

pub fn link_hard(old: &str, new: &str) -> PerlValue {
    PerlValue::Integer(if std::fs::hard_link(old, new).is_ok() {
        1
    } else {
        0
    })
}

pub fn link_sym(old: &str, new: &str) -> PerlValue {
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        PerlValue::Integer(if symlink(old, new).is_ok() { 1 } else { 0 })
    }
    #[cfg(not(unix))]
    {
        let _ = (old, new);
        PerlValue::Integer(0)
    }
}

pub fn read_link(path: &str) -> PerlValue {
    match std::fs::read_link(path) {
        Ok(p) => PerlValue::String(p.to_string_lossy().into_owned()),
        Err(_) => PerlValue::Undef,
    }
}

pub fn glob_patterns(patterns: &[String]) -> PerlValue {
    let mut paths: Vec<String> = Vec::new();
    for pat in patterns {
        if let Ok(g) = glob::glob(pat) {
            for e in g.flatten() {
                paths.push(e.to_string_lossy().into_owned());
            }
        }
    }
    paths.sort();
    paths.dedup();
    PerlValue::Array(paths.into_iter().map(PerlValue::String).collect())
}
