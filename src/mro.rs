//! C3 method resolution order (Perl `mro` / `Algorithm::C3` style).

fn in_any_tail(seqs: &[Vec<String>], x: &str) -> bool {
    for s in seqs {
        if s.len() > 1 && s[1..].iter().any(|e| e == x) {
            return true;
        }
    }
    false
}

/// Merge C3 predecessor lists; `None` if inconsistent.
pub fn merge_c3(seqs: &[Vec<String>]) -> Option<Vec<String>> {
    let mut seqs: Vec<Vec<String>> = seqs.iter().cloned().collect();
    let mut out = Vec::new();
    loop {
        if seqs.iter().all(|s| s.is_empty()) {
            return Some(out);
        }
        let mut candidate: Option<String> = None;
        for s in &seqs {
            if let Some(h) = s.first() {
                if !in_any_tail(&seqs, h) {
                    candidate = Some(h.clone());
                    break;
                }
            }
        }
        let cand = candidate?;
        out.push(cand.clone());
        for s in &mut seqs {
            if s.first() == Some(&cand) {
                s.remove(0);
            }
        }
    }
}

/// Linearize `class` with C3 using `parents(class)` as immediate `@ISA`.
pub fn linearize_c3(
    class: &str,
    parents: &impl Fn(&str) -> Vec<String>,
    depth: usize,
) -> Vec<String> {
    if depth > 256 {
        return vec![class.to_string()];
    }
    if class == "UNIVERSAL" {
        return vec!["UNIVERSAL".to_string()];
    }
    let ps = parents(class);
    if ps.is_empty() {
        return vec![class.to_string(), "UNIVERSAL".to_string()];
    }
    let mut seqs: Vec<Vec<String>> = Vec::new();
    for p in &ps {
        seqs.push(linearize_c3(p, parents, depth + 1));
    }
    seqs.push(ps);
    let merged = merge_c3(&seqs).unwrap_or_default();
    let mut out = vec![class.to_string()];
    out.extend(merged);
    if !out.iter().any(|c| c == "UNIVERSAL") {
        out.push("UNIVERSAL".to_string());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c3_diamond() {
        let parents = |c: &str| -> Vec<String> {
            match c {
                "D" => vec!["B".into(), "C".into()],
                "B" => vec!["A".into()],
                "C" => vec!["A".into()],
                "A" => vec![],
                _ => vec![],
            }
        };
        let m = linearize_c3("D", &parents, 0);
        assert_eq!(m, vec!["D", "B", "C", "A", "UNIVERSAL"]);
    }
}
