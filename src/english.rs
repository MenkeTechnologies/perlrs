//! Minimal `English.pm`-style scalar aliases (`use English`).
//!
//! Stock `English` maps long names to the same globals as short punctuation variables.
//! Only a subset is listed; unknown names are not aliased.
//!
//! Not yet wired into the compiler/interpreter; this module is covered by unit tests and
//! reserved for a future `use English` implementation.

use std::collections::HashMap;
use std::sync::LazyLock;

#[allow(dead_code)] // Only referenced from `#[cfg(test)]` in this file for non-test library builds.
static ENGLISH_ALIASES: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        // $_
        ("ARG", "_"),
        // $.
        ("INPUT_LINE_NUMBER", "."),
        ("NR", "."),
        // $/
        ("INPUT_RECORD_SEPARATOR", "/"),
        ("RS", "/"),
        // $,
        ("OFS", ","),
        ("OUTPUT_FIELD_SEPARATOR", ","),
        // $\
        ("ORS", "\\"),
        ("OUTPUT_RECORD_SEPARATOR", "\\"),
        // $"
        ("LIST_SEPARATOR", "\""),
        // $;
        ("SUBSCRIPT_SEPARATOR", ";"),
        ("SUBSEP", ";"),
        // $|
        ("OUTPUT_AUTOFLUSH", "|"),
        // $!
        ("OS_ERROR", "!"),
        ("ERRNO", "!"),
        // $@
        ("EVAL_ERROR", "@"),
        // $?
        ("CHILD_ERROR", "?"),
        // $$
        ("PROCESS_ID", "$$"),
        ("PID", "$$"),
        // $0
        ("PROGRAM_NAME", "0"),
        // $^O
        ("OSNAME", "^O"),
        // $^T
        ("BASETIME", "^T"),
        // $^V
        ("PERL_VERSION", "^V"),
        // $^E
        ("EXTENDED_OS_ERROR", "^E"),
        // $^W
        ("WARNING", "^W"),
        // $^C
        ("INTERRUPT", "^C"),
        // $*
        ("MULTILINE_MATCHING", "*"),
        // $&
        ("MATCH", "&"),
        // `` $` ``
        ("PREMATCH", "`"),
        // $'
        ("POSTMATCH", "'"),
        // $+
        ("LAST_PAREN_MATCH", "+"),
    ])
});

/// If `name` is a known `English` long name, return the short special name (`_`, `.`, …).
#[inline]
#[allow(dead_code)] // See `ENGLISH_ALIASES`.
pub(crate) fn scalar_alias(name: &str) -> Option<&'static str> {
    ENGLISH_ALIASES.get(name).copied()
}

#[cfg(test)]
mod tests {
    use super::scalar_alias;

    #[test]
    fn alias_arg_maps_to_default_scalar() {
        assert_eq!(scalar_alias("ARG"), Some("_"));
    }

    #[test]
    fn alias_input_line_number_and_nr_map_to_dot() {
        assert_eq!(scalar_alias("INPUT_LINE_NUMBER"), Some("."));
        assert_eq!(scalar_alias("NR"), Some("."));
    }

    #[test]
    fn alias_rs_and_input_record_separator_map_to_slash() {
        assert_eq!(scalar_alias("RS"), Some("/"));
        assert_eq!(scalar_alias("INPUT_RECORD_SEPARATOR"), Some("/"));
    }

    #[test]
    fn alias_process_id_and_pid_map_to_double_dollar() {
        assert_eq!(scalar_alias("PROCESS_ID"), Some("$$"));
        assert_eq!(scalar_alias("PID"), Some("$$"));
    }

    #[test]
    fn alias_program_name_maps_to_zero() {
        assert_eq!(scalar_alias("PROGRAM_NAME"), Some("0"));
    }

    #[test]
    fn unknown_long_name_returns_none() {
        assert_eq!(scalar_alias("NOT_A_REAL_ENGLISH_NAME"), None);
        assert_eq!(scalar_alias(""), None);
    }
}
