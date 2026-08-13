//! Named Query Library parser for `.jpx` files.
//!
//! This module provides parsing support for query library files that contain
//! multiple named, reusable JMESPath expressions. The format is inspired by
//! SQLDelight/HugSQL patterns.
//!
//! # File Format
//!
//! ```text
//! -- :name top-keywords
//! -- :desc Extract top keywords from text field
//! tokens(@) | remove_stopwords(@) | stems(@) | frequencies(@)
//!
//! -- :name clean-html
//! -- :desc Strip HTML tags and normalize whitespace
//! regex_replace(@, `"<[^>]+>"`, `" "`) | collapse_whitespace(@)
//! ```
//!
//! ## Directives
//!
//! - `-- :name <name>` - Starts a new query (required)
//! - `-- :desc <description>` - Adds a description to the current query (optional)
//! - `-- ` - Other comment lines are ignored
//!
//! Everything between `-- :name` directives becomes the query expression.
//! Multi-line expressions are supported.
//!
//! # Example
//!
//! ```rust
//! use jpx_core::query_library::QueryLibrary;
//!
//! let content = r#"
//! -- :name greet
//! -- :desc Simple greeting
//! `"hello"`
//!
//! -- :name count
//! length(@)
//! "#;
//!
//! let library = QueryLibrary::parse(content).unwrap();
//!
//! assert_eq!(library.names(), vec!["greet", "count"]);
//!
//! let greet = library.get("greet").unwrap();
//! assert_eq!(greet.name, "greet");
//! assert_eq!(greet.description, Some("Simple greeting".to_string()));
//! assert_eq!(greet.expression, r#"`"hello"`"#);
//!
//! let count = library.get("count").unwrap();
//! assert_eq!(count.expression, "length(@)");
//! ```
//!
//! # Detection
//!
//! Use [`is_query_library`] to check if content looks like a query library:
//!
//! ```rust
//! use jpx_core::query_library::is_query_library;
//!
//! assert!(is_query_library("-- :name foo\nlength(@)"));
//! assert!(!is_query_library("length(@)"));
//! ```

use std::fmt;

/// Error type for query library parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    /// Error message
    pub message: String,
    /// Line number where the error occurred (1-indexed)
    pub line: Option<usize>,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.line {
            Some(line) => write!(f, "{} at line {}", self.message, line),
            None => write!(f, "{}", self.message),
        }
    }
}

impl std::error::Error for ParseError {}

impl ParseError {
    /// Create a new parse error with a message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            line: None,
        }
    }

    /// Create a new parse error with a message and line number.
    pub fn with_line(message: impl Into<String>, line: usize) -> Self {
        Self {
            message: message.into(),
            line: Some(line),
        }
    }
}

/// A named query with optional description.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedQuery {
    /// Query name (used for lookup)
    pub name: String,
    /// Optional description
    pub description: Option<String>,
    /// The JMESPath expression
    pub expression: String,
    /// Line number where the query starts (1-indexed, for error messages)
    pub line_number: usize,
}

/// A collection of named queries parsed from a `.jpx` file.
#[derive(Debug, Clone, Default)]
pub struct QueryLibrary {
    queries: Vec<NamedQuery>,
}

impl QueryLibrary {
    /// Create an empty query library.
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse a query library from file content.
    ///
    /// # Format
    ///
    /// - `-- :name <name>` starts a new query
    /// - `-- :desc <description>` adds a description to the current query
    /// - `-- ` other comment lines are ignored
    /// - Non-comment lines are appended to the current query's expression
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - A query has an empty name
    /// - A query has no expression
    /// - Duplicate query names are found
    /// - No queries are found in the content
    ///
    /// # Example
    ///
    /// ```rust
    /// use jpx_core::query_library::QueryLibrary;
    ///
    /// let content = r#"
    /// -- :name count
    /// length(@)
    /// "#;
    ///
    /// let library = QueryLibrary::parse(content).unwrap();
    /// assert_eq!(library.len(), 1);
    /// ```
    pub fn parse(content: &str) -> Result<Self, ParseError> {
        let mut queries = Vec::new();
        let mut current_name: Option<String> = None;
        let mut current_desc: Option<String> = None;
        let mut current_expr = String::new();
        let mut current_line_number = 0usize;

        for (line_num, line) in content.lines().enumerate() {
            let line_number = line_num + 1; // 1-indexed for error messages
            let trimmed = line.trim();

            if let Some(rest) = trimmed.strip_prefix("-- :name ").or_else(|| {
                // Handle "-- :name" without trailing space (empty name case)
                if trimmed == "-- :name" {
                    Some("")
                } else {
                    None
                }
            }) {
                // Save previous query if exists
                if let Some(name) = current_name.take() {
                    let expr = current_expr.trim().to_string();
                    if expr.is_empty() {
                        return Err(ParseError::with_line(
                            format!("Query '{}' has no expression", name),
                            current_line_number,
                        ));
                    }
                    queries.push(NamedQuery {
                        name,
                        description: current_desc.take(),
                        expression: expr,
                        line_number: current_line_number,
                    });
                    current_expr.clear();
                }

                // Start new query
                let name = rest.trim().to_string();
                if name.is_empty() {
                    return Err(ParseError::with_line("Empty query name", line_number));
                }

                // Check for duplicates
                if queries.iter().any(|q| q.name == name) {
                    return Err(ParseError::with_line(
                        format!("Duplicate query name '{}'", name),
                        line_number,
                    ));
                }

                current_name = Some(name);
                current_line_number = line_number;
            } else if let Some(rest) = trimmed.strip_prefix("-- :desc ") {
                // Add description to current query
                if current_name.is_some() {
                    current_desc = Some(rest.trim().to_string());
                }
            } else if trimmed.starts_with("-- ") || trimmed == "--" {
                // Skip other comments
            } else if !trimmed.is_empty() {
                // Append to current expression
                if current_name.is_some() {
                    if !current_expr.is_empty() {
                        current_expr.push('\n');
                    }
                    current_expr.push_str(line);
                }
            }
        }

        // Save final query
        if let Some(name) = current_name {
            let expr = current_expr.trim().to_string();
            if expr.is_empty() {
                return Err(ParseError::with_line(
                    format!("Query '{}' has no expression", name),
                    current_line_number,
                ));
            }
            queries.push(NamedQuery {
                name,
                description: current_desc,
                expression: expr,
                line_number: current_line_number,
            });
        }

        if queries.is_empty() {
            return Err(ParseError::new(
                "No queries found. Use '-- :name <query-name>' to define queries.",
            ));
        }

        Ok(QueryLibrary { queries })
    }

    /// Get a query by name.
    ///
    /// # Example
    ///
    /// ```rust
    /// use jpx_core::query_library::QueryLibrary;
    ///
    /// let lib = QueryLibrary::parse("-- :name test\nlength(@)").unwrap();
    /// let query = lib.get("test").unwrap();
    /// assert_eq!(query.expression, "length(@)");
    /// ```
    pub fn get(&self, name: &str) -> Option<&NamedQuery> {
        self.queries.iter().find(|q| q.name == name)
    }

    /// Get all queries.
    ///
    /// # Example
    ///
    /// ```rust
    /// use jpx_core::query_library::QueryLibrary;
    ///
    /// let lib = QueryLibrary::parse("-- :name a\n`1`\n-- :name b\n`2`").unwrap();
    /// assert_eq!(lib.list().len(), 2);
    /// ```
    pub fn list(&self) -> &[NamedQuery] {
        &self.queries
    }

    /// Get query names.
    ///
    /// # Example
    ///
    /// ```rust
    /// use jpx_core::query_library::QueryLibrary;
    ///
    /// let lib = QueryLibrary::parse("-- :name foo\n`1`\n-- :name bar\n`2`").unwrap();
    /// assert_eq!(lib.names(), vec!["foo", "bar"]);
    /// ```
    pub fn names(&self) -> Vec<&str> {
        self.queries.iter().map(|q| q.name.as_str()).collect()
    }

    /// Get the number of queries in the library.
    pub fn len(&self) -> usize {
        self.queries.len()
    }

    /// Check if the library is empty.
    pub fn is_empty(&self) -> bool {
        self.queries.is_empty()
    }

    /// Iterate over queries.
    pub fn iter(&self) -> impl Iterator<Item = &NamedQuery> {
        self.queries.iter()
    }
}

impl<'a> IntoIterator for &'a QueryLibrary {
    type Item = &'a NamedQuery;
    type IntoIter = std::slice::Iter<'a, NamedQuery>;

    fn into_iter(self) -> Self::IntoIter {
        self.queries.iter()
    }
}

/// Check if content looks like a query library (starts with `-- :name`).
///
/// This function checks if the first non-empty line starts with `-- :name `,
/// indicating a query library format.
///
/// # Example
///
/// ```rust
/// use jpx_core::query_library::is_query_library;
///
/// assert!(is_query_library("-- :name foo\nlength(@)"));
/// assert!(is_query_library("  -- :name foo\nlength(@)"));
/// assert!(is_query_library("\n-- :name foo\nlength(@)"));
/// assert!(!is_query_library("length(@)"));
/// assert!(!is_query_library("-- comment\nlength(@)"));
/// ```
pub fn is_query_library(content: &str) -> bool {
    content
        .lines()
        .find(|line| !line.trim().is_empty())
        .map(|line| line.trim().starts_with("-- :name "))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_library() {
        let content = r#"
-- :name greet
-- :desc Simple greeting
`"hello"`

-- :name count
length(@)
"#;
        let lib = QueryLibrary::parse(content).unwrap();
        assert_eq!(lib.len(), 2);

        let greet = lib.get("greet").unwrap();
        assert_eq!(greet.name, "greet");
        assert_eq!(greet.description, Some("Simple greeting".to_string()));
        assert_eq!(greet.expression, "`\"hello\"`");

        let count = lib.get("count").unwrap();
        assert_eq!(count.name, "count");
        assert_eq!(count.description, None);
        assert_eq!(count.expression, "length(@)");
    }

    #[test]
    fn test_parse_multiline_expression() {
        let content = r#"
-- :name complex
-- :desc Multi-line query
{
  total: length(@),
  first: @[0]
}
"#;
        let lib = QueryLibrary::parse(content).unwrap();
        let query = lib.get("complex").unwrap();
        assert!(query.expression.contains("total: length(@)"));
        assert!(query.expression.contains("first: @[0]"));
    }

    #[test]
    fn test_parse_empty_name_error() {
        let content = "-- :name \nlength(@)";
        let result = QueryLibrary::parse(content);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Empty query name"));
        assert_eq!(err.line, Some(1));
    }

    #[test]
    fn test_parse_duplicate_name_error() {
        let content = r#"
-- :name foo
length(@)

-- :name foo
keys(@)
"#;
        let result = QueryLibrary::parse(content);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Duplicate query name"));
    }

    #[test]
    fn test_parse_no_expression_error() {
        let content = "-- :name empty\n-- :name another\nlength(@)";
        let result = QueryLibrary::parse(content);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("has no expression"));
    }

    #[test]
    fn test_parse_no_queries_error() {
        let content = "-- just a comment\nlength(@)";
        let result = QueryLibrary::parse(content);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().message,
            "No queries found. Use '-- :name <query-name>' to define queries."
        );
    }

    #[test]
    fn test_is_query_library() {
        assert!(is_query_library("-- :name foo\nlength(@)"));
        assert!(is_query_library("  -- :name foo\nlength(@)"));
        assert!(is_query_library("\n-- :name foo\nlength(@)"));
        assert!(!is_query_library("length(@)"));
        assert!(!is_query_library("-- comment\nlength(@)"));
    }

    #[test]
    fn test_comments_ignored() {
        let content = r#"
-- :name test
-- :desc Description
-- This is a regular comment
-- Another comment
length(@)
-- Trailing comment
"#;
        let lib = QueryLibrary::parse(content).unwrap();
        let query = lib.get("test").unwrap();
        assert_eq!(query.expression, "length(@)");
    }

    #[test]
    fn test_iter() {
        let content = "-- :name a\n`1`\n-- :name b\n`2`";
        let lib = QueryLibrary::parse(content).unwrap();
        let names: Vec<_> = lib.iter().map(|q| &q.name).collect();
        assert_eq!(names, vec!["a", "b"]);
    }

    #[test]
    fn test_into_iter() {
        let content = "-- :name x\n`1`";
        let lib = QueryLibrary::parse(content).unwrap();
        for query in &lib {
            assert_eq!(query.name, "x");
        }
    }
}
