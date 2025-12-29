//! Matchers for flexible string and value comparisons in test DSL
//!
//! Provides `StringMatcher` for flexible string comparisons and
//! `NagMatcher` for NAG (Numeric Annotation Glyph) matching.

use pgnq::nag::Nag;

/// Flexible string matching for headers, comments, etc.
#[derive(Debug, Clone)]
pub enum StringMatcher {
    /// Exact string match
    Exact(String),
    /// String contains substring
    Contains(String),
    /// String starts with prefix
    StartsWith(String),
    /// String ends with suffix
    EndsWith(String),
    /// Matches any non-empty string
    NonEmpty,
    /// Matches any string including empty
    Any,
}

impl StringMatcher {
    /// Create an exact match
    pub fn exact(s: impl Into<String>) -> Self {
        StringMatcher::Exact(s.into())
    }

    /// Create a contains match
    pub fn contains(s: impl Into<String>) -> Self {
        StringMatcher::Contains(s.into())
    }

    /// Create a starts-with match
    pub fn starts_with(s: impl Into<String>) -> Self {
        StringMatcher::StartsWith(s.into())
    }

    /// Create an ends-with match
    pub fn ends_with(s: impl Into<String>) -> Self {
        StringMatcher::EndsWith(s.into())
    }

    /// Match any non-empty string
    pub fn non_empty() -> Self {
        StringMatcher::NonEmpty
    }

    /// Match any string (always succeeds)
    pub fn any() -> Self {
        StringMatcher::Any
    }

    /// Check if the matcher matches the given string
    pub fn matches(&self, actual: &str) -> bool {
        match self {
            StringMatcher::Exact(expected) => actual == expected,
            StringMatcher::Contains(substring) => actual.contains(substring),
            StringMatcher::StartsWith(prefix) => actual.starts_with(prefix),
            StringMatcher::EndsWith(suffix) => actual.ends_with(suffix),
            StringMatcher::NonEmpty => !actual.is_empty(),
            StringMatcher::Any => true,
        }
    }

    /// Describe what this matcher expects (for error messages)
    pub fn describe(&self) -> String {
        match self {
            StringMatcher::Exact(s) => format!("exactly {:?}", s),
            StringMatcher::Contains(s) => format!("containing {:?}", s),
            StringMatcher::StartsWith(s) => format!("starting with {:?}", s),
            StringMatcher::EndsWith(s) => format!("ending with {:?}", s),
            StringMatcher::NonEmpty => "non-empty string".to_string(),
            StringMatcher::Any => "any string".to_string(),
        }
    }
}

impl From<&str> for StringMatcher {
    fn from(s: &str) -> Self {
        StringMatcher::Exact(s.to_string())
    }
}

impl From<String> for StringMatcher {
    fn from(s: String) -> Self {
        StringMatcher::Exact(s)
    }
}

/// Matcher for NAG (Numeric Annotation Glyph) values
#[derive(Debug, Clone)]
pub enum NagMatcher {
    /// Exact NAG match
    Exact(Nag),
    /// NAGs must include this value (among others)
    Includes(Nag),
    /// NAGs must include all of these
    IncludesAll(Vec<Nag>),
    /// NAGs must include any of these
    IncludesAny(Vec<Nag>),
    /// Must have at least one NAG
    HasAny,
    /// Must have no NAGs
    None,
    /// Match any NAG state (always succeeds)
    Any,
}

impl NagMatcher {
    /// Match a specific NAG exactly (node must have only this NAG)
    pub fn exact(nag: Nag) -> Self {
        NagMatcher::Exact(nag)
    }

    /// NAGs must include this value
    pub fn includes(nag: Nag) -> Self {
        NagMatcher::Includes(nag)
    }

    /// NAGs must include all of these
    pub fn includes_all(nags: Vec<Nag>) -> Self {
        NagMatcher::IncludesAll(nags)
    }

    /// NAGs must include any of these
    pub fn includes_any(nags: Vec<Nag>) -> Self {
        NagMatcher::IncludesAny(nags)
    }

    /// Must have at least one NAG
    pub fn has_any() -> Self {
        NagMatcher::HasAny
    }

    /// Must have no NAGs
    pub fn none() -> Self {
        NagMatcher::None
    }

    /// Match any NAG state
    pub fn any() -> Self {
        NagMatcher::Any
    }

    /// Check if the matcher matches the given NAGs
    pub fn matches(&self, actual: &[Nag]) -> bool {
        match self {
            NagMatcher::Exact(nag) => actual.len() == 1 && actual[0] == *nag,
            NagMatcher::Includes(nag) => actual.contains(nag),
            NagMatcher::IncludesAll(nags) => nags.iter().all(|n| actual.contains(n)),
            NagMatcher::IncludesAny(nags) => nags.iter().any(|n| actual.contains(n)),
            NagMatcher::HasAny => !actual.is_empty(),
            NagMatcher::None => actual.is_empty(),
            NagMatcher::Any => true,
        }
    }

    /// Describe what this matcher expects (for error messages)
    pub fn describe(&self) -> String {
        match self {
            NagMatcher::Exact(nag) => format!("exactly NAG {}", nag),
            NagMatcher::Includes(nag) => format!("NAGs including {}", nag),
            NagMatcher::IncludesAll(nags) => {
                let nag_strs: Vec<_> = nags.iter().map(|n| n.to_string()).collect();
                format!("NAGs including all of [{}]", nag_strs.join(", "))
            }
            NagMatcher::IncludesAny(nags) => {
                let nag_strs: Vec<_> = nags.iter().map(|n| n.to_string()).collect();
                format!("NAGs including any of [{}]", nag_strs.join(", "))
            }
            NagMatcher::HasAny => "at least one NAG".to_string(),
            NagMatcher::None => "no NAGs".to_string(),
            NagMatcher::Any => "any NAGs".to_string(),
        }
    }

    /// Describe actual NAGs (for error messages)
    pub fn describe_actual(actual: &[Nag]) -> String {
        if actual.is_empty() {
            "no NAGs".to_string()
        } else {
            let nag_strs: Vec<_> = actual.iter().map(|n| n.to_string()).collect();
            format!("[{}]", nag_strs.join(", "))
        }
    }
}

impl From<Nag> for NagMatcher {
    fn from(nag: Nag) -> Self {
        NagMatcher::Includes(nag)
    }
}

/// Matcher for child count
#[derive(Debug, Clone)]
pub enum CountMatcher {
    /// Exact count
    Exact(usize),
    /// At least this many
    AtLeast(usize),
    /// At most this many
    AtMost(usize),
    /// Between min and max (inclusive)
    Between(usize, usize),
    /// Any count (always matches)
    Any,
}

impl CountMatcher {
    pub fn exact(n: usize) -> Self {
        CountMatcher::Exact(n)
    }

    pub fn at_least(n: usize) -> Self {
        CountMatcher::AtLeast(n)
    }

    pub fn at_most(n: usize) -> Self {
        CountMatcher::AtMost(n)
    }

    pub fn between(min: usize, max: usize) -> Self {
        CountMatcher::Between(min, max)
    }

    pub fn any() -> Self {
        CountMatcher::Any
    }

    pub fn matches(&self, actual: usize) -> bool {
        match self {
            CountMatcher::Exact(n) => actual == *n,
            CountMatcher::AtLeast(n) => actual >= *n,
            CountMatcher::AtMost(n) => actual <= *n,
            CountMatcher::Between(min, max) => actual >= *min && actual <= *max,
            CountMatcher::Any => true,
        }
    }

    pub fn describe(&self) -> String {
        match self {
            CountMatcher::Exact(n) => format!("exactly {}", n),
            CountMatcher::AtLeast(n) => format!("at least {}", n),
            CountMatcher::AtMost(n) => format!("at most {}", n),
            CountMatcher::Between(min, max) => format!("between {} and {}", min, max),
            CountMatcher::Any => "any count".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_matcher_exact() {
        let matcher = StringMatcher::exact("hello");
        assert!(matcher.matches("hello"));
        assert!(!matcher.matches("Hello"));
        assert!(!matcher.matches("hello world"));
    }

    #[test]
    fn test_string_matcher_contains() {
        let matcher = StringMatcher::contains("world");
        assert!(matcher.matches("hello world"));
        assert!(matcher.matches("world"));
        assert!(!matcher.matches("hello"));
    }

    #[test]
    fn test_string_matcher_starts_with() {
        let matcher = StringMatcher::starts_with("hello");
        assert!(matcher.matches("hello world"));
        assert!(matcher.matches("hello"));
        assert!(!matcher.matches("world hello"));
    }

    #[test]
    fn test_string_matcher_non_empty() {
        let matcher = StringMatcher::non_empty();
        assert!(matcher.matches("hello"));
        assert!(!matcher.matches(""));
    }

    #[test]
    fn test_string_matcher_any() {
        let matcher = StringMatcher::any();
        assert!(matcher.matches("hello"));
        assert!(matcher.matches(""));
    }

    #[test]
    fn test_nag_matcher_includes() {
        let matcher = NagMatcher::includes(Nag::GOOD_MOVE);
        assert!(matcher.matches(&[Nag::GOOD_MOVE]));
        assert!(matcher.matches(&[Nag::GOOD_MOVE, Nag::WHITE_SLIGHT_ADVANTAGE]));
        assert!(!matcher.matches(&[Nag::POOR_MOVE]));
        assert!(!matcher.matches(&[]));
    }

    #[test]
    fn test_nag_matcher_none() {
        let matcher = NagMatcher::none();
        assert!(matcher.matches(&[]));
        assert!(!matcher.matches(&[Nag::GOOD_MOVE]));
    }

    #[test]
    fn test_count_matcher() {
        assert!(CountMatcher::exact(3).matches(3));
        assert!(!CountMatcher::exact(3).matches(4));
        assert!(CountMatcher::at_least(2).matches(3));
        assert!(!CountMatcher::at_least(2).matches(1));
    }
}
