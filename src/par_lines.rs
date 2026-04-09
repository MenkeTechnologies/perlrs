//! Memory-mapped parallel line iteration for `par_lines PATH, sub { ... }`.
//! Splits the file into byte ranges aligned to line starts, then processes each chunk in parallel
//! with rayon (each chunk scans its lines sequentially).

/// Build up to `max_chunks` contiguous byte ranges `[start, end)` covering `data`, where each
/// range starts at a line boundary (byte 0 or immediately after `\n`). Ranges partition the file
/// without splitting lines.
pub fn line_aligned_chunks(data: &[u8], max_chunks: usize) -> Vec<(usize, usize)> {
    let len = data.len();
    if len == 0 {
        return vec![];
    }
    let k = max_chunks.max(1).min(len);
    let mut splits: Vec<usize> = (0..=k).map(|i| i * len / k).collect();
    for split in splits.iter_mut().take(k).skip(1) {
        let mut p = *split;
        while p < len && p > 0 && data[p - 1] != b'\n' {
            p += 1;
        }
        *split = p;
    }
    for i in 1..=k {
        if splits[i] < splits[i - 1] {
            splits[i] = splits[i - 1];
        }
    }
    let mut out = Vec::new();
    for i in 0..k {
        let s = splits[i];
        let e = splits[i + 1];
        if s < e {
            out.push((s, e));
        }
    }
    if out.is_empty() {
        out.push((0, len));
    }
    out
}

/// Convert one line of bytes (no `\n`) to a Perl string; strips trailing `\r` for CRLF.
pub fn line_to_perl_string(line: &[u8]) -> String {
    let mut s = String::from_utf8_lossy(line).into_owned();
    if s.ends_with('\r') {
        s.pop();
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_aligned_chunks_splits_without_breaking_lines() {
        let data = b"a\nbb\nccc\n";
        let chunks = line_aligned_chunks(data, 4);
        let rebuilt: Vec<u8> = chunks
            .iter()
            .flat_map(|(s, e)| data[*s..*e].iter().copied())
            .collect();
        assert_eq!(rebuilt, data);
        for (s, _e) in &chunks {
            if *s > 0 {
                assert_eq!(data[*s - 1], b'\n');
            }
        }
    }

    #[test]
    fn scan_lines_in_slice_three_lines() {
        let data = b"one\ntwo\nthree";
        let mut lines = Vec::new();
        let mut s = 0usize;
        while s < data.len() {
            let e = data[s..]
                .iter()
                .position(|&b| b == b'\n')
                .map(|p| s + p)
                .unwrap_or(data.len());
            lines.push(&data[s..e]);
            if e >= data.len() {
                break;
            }
            s = e + 1;
        }
        assert_eq!(lines, vec![&b"one"[..], &b"two"[..], &b"three"[..]]);
    }
}
