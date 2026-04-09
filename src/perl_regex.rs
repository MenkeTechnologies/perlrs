//! Dual-engine compiled regex: try Rust [`regex`] first (fast linear-time subset), then
//! [`fancy_regex`] for patterns the linear matcher rejects (e.g. backreferences).

use std::sync::Arc;

use crate::value::PerlValue;

/// Compiled pattern: either the standard [`regex`] crate or [`fancy_regex`] fallback.
#[derive(Debug, Clone)]
pub enum PerlCompiledRegex {
    Rust(Arc<regex::Regex>),
    Fancy(Arc<fancy_regex::Regex>),
}

/// Unified captures for match-variable setup (`$1`, `@-`, `%+`, …).
#[derive(Debug)]
pub enum PerlCaptures<'a> {
    Rust(regex::Captures<'a>),
    Fancy(fancy_regex::Captures<'a>),
}

impl<'a> PerlCaptures<'a> {
    #[inline]
    pub fn len(&self) -> usize {
        match self {
            Self::Rust(c) => c.len(),
            Self::Fancy(c) => c.len(),
        }
    }

    #[inline]
    pub fn get(&self, i: usize) -> Option<RegexMatch<'a>> {
        match self {
            Self::Rust(c) => c.get(i).map(Into::into),
            Self::Fancy(c) => c.get(i).map(Into::into),
        }
    }

    #[inline]
    pub fn name(&self, name: &str) -> Option<RegexMatch<'a>> {
        match self {
            Self::Rust(c) => c.name(name).map(Into::into),
            Self::Fancy(c) => c.name(name).map(Into::into),
        }
    }
}

/// Minimal match view shared by both engines.
#[derive(Clone, Copy, Debug)]
pub struct RegexMatch<'a> {
    pub start: usize,
    pub end: usize,
    pub text: &'a str,
}

impl<'a> From<regex::Match<'a>> for RegexMatch<'a> {
    fn from(m: regex::Match<'a>) -> Self {
        Self {
            start: m.start(),
            end: m.end(),
            text: m.as_str(),
        }
    }
}

impl<'a> From<fancy_regex::Match<'a>> for RegexMatch<'a> {
    fn from(m: fancy_regex::Match<'a>) -> Self {
        Self {
            start: m.start(),
            end: m.end(),
            text: m.as_str(),
        }
    }
}

impl PerlCompiledRegex {
    /// Compile `re_str` (already Perl-expanded: flags as `(?i)` etc. are in the string).
    /// Tries [`regex::Regex`] first; on failure uses [`fancy_regex::Regex`].
    pub fn compile(re_str: &str) -> Result<Arc<Self>, String> {
        if let Ok(r) = regex::Regex::new(re_str) {
            return Ok(Arc::new(Self::Rust(Arc::new(r))));
        }
        match fancy_regex::Regex::new(re_str) {
            Ok(r) => Ok(Arc::new(Self::Fancy(Arc::new(r)))),
            Err(e) => Err(e.to_string()),
        }
    }

    #[inline]
    pub fn is_match(&self, s: &str) -> bool {
        match self {
            Self::Rust(r) => r.is_match(s),
            Self::Fancy(r) => r.is_match(s).unwrap_or(false),
        }
    }

    pub fn captures<'t>(&self, text: &'t str) -> Option<PerlCaptures<'t>> {
        match self {
            Self::Rust(r) => r.captures(text).map(PerlCaptures::Rust),
            Self::Fancy(r) => match r.captures(text) {
                Ok(Some(c)) => Some(PerlCaptures::Fancy(c)),
                _ => None,
            },
        }
    }

    /// Iterator over all non-overlapping capture sets (for `/g` in list context).
    pub fn captures_iter<'r, 't>(
        &'r self,
        text: &'t str,
    ) -> CaptureIter<'r, 't> {
        match self {
            Self::Rust(r) => CaptureIter::Rust(r.captures_iter(text)),
            Self::Fancy(r) => CaptureIter::Fancy(r.captures_iter(text)),
        }
    }

    pub fn capture_names(&self) -> CaptureNames<'_> {
        match self {
            Self::Rust(r) => CaptureNames::Rust(r.capture_names()),
            Self::Fancy(r) => CaptureNames::Fancy(r.capture_names()),
        }
    }

    pub fn replace(&self, s: &str, replacement: &str) -> String {
        match self {
            Self::Rust(r) => r.replace(s, replacement).to_string(),
            Self::Fancy(r) => r.replace(s, replacement).to_string(),
        }
    }

    pub fn replace_all(&self, s: &str, replacement: &str) -> String {
        match self {
            Self::Rust(r) => r.replace_all(s, replacement).to_string(),
            Self::Fancy(r) => r.replace_all(s, replacement).to_string(),
        }
    }

    pub fn find_iter_count(&self, s: &str) -> usize {
        match self {
            Self::Rust(r) => r.find_iter(s).count(),
            Self::Fancy(r) => r
                .find_iter(s)
                .filter(|m| m.is_ok())
                .count(),
        }
    }

    /// `split` / `split EXPR, STR` — same semantics as [`regex::Regex::split`].
    pub fn split_strings(&self, s: &str) -> Vec<String> {
        match self {
            Self::Rust(r) => r.split(s).map(|x| x.to_string()).collect(),
            Self::Fancy(r) => r
                .split(s)
                .filter_map(|x| x.ok())
                .map(|x| x.to_string())
                .collect(),
        }
    }

    pub fn splitn_strings(&self, s: &str, limit: usize) -> Vec<String> {
        match self {
            Self::Rust(r) => r.splitn(s, limit).map(|x| x.to_string()).collect(),
            Self::Fancy(r) => r
                .splitn(s, limit)
                .filter_map(|x| x.ok())
                .map(|x| x.to_string())
                .collect(),
        }
    }
}

pub enum CaptureIter<'r, 't> {
    Rust(regex::CaptureMatches<'r, 't>),
    Fancy(fancy_regex::CaptureMatches<'r, 't>),
}

impl<'r, 't> Iterator for CaptureIter<'r, 't> {
    type Item = PerlCaptures<'t>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Rust(it) => it.next().map(PerlCaptures::Rust),
            Self::Fancy(it) => loop {
                match it.next()? {
                    Ok(c) => return Some(PerlCaptures::Fancy(c)),
                    Err(_) => continue,
                }
            },
        }
    }
}

pub enum CaptureNames<'a> {
    Rust(regex::CaptureNames<'a>),
    Fancy(fancy_regex::CaptureNames<'a>),
}

impl<'a> Iterator for CaptureNames<'a> {
    type Item = Option<&'a str>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Rust(it) => it.next(),
            Self::Fancy(it) => it.next(),
        }
    }
}

/// `$1`… flatten for `@^CAPTURE` / `^CAPTURE_ALL` rows.
pub fn numbered_capture_flat(caps: &PerlCaptures<'_>) -> Vec<PerlValue> {
    let mut cap_flat = Vec::new();
    for i in 1..caps.len() {
        if let Some(m) = caps.get(i) {
            cap_flat.push(PerlValue::string(m.text.to_string()));
        } else {
            cap_flat.push(PerlValue::UNDEF);
        }
    }
    cap_flat
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_engine_used_for_simple_pattern() {
        let r = PerlCompiledRegex::compile("ab+").unwrap();
        assert!(matches!(*r, PerlCompiledRegex::Rust(_)));
        assert!(r.is_match("xabby"));
    }

    #[test]
    fn fancy_fallback_for_backreference() {
        let r = PerlCompiledRegex::compile(r"(.)\1").unwrap();
        assert!(matches!(*r, PerlCompiledRegex::Fancy(_)));
        assert!(r.is_match("aa"));
        assert!(!r.is_match("ab"));
    }
}
