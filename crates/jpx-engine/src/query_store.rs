//! Session-scoped storage for named JMESPath queries.
//!
//! The query store allows you to save JMESPath expressions with names for
//! reuse during a session. This is particularly useful for:
//!
//! - Building up complex queries iteratively
//! - Reusing common extraction patterns
//! - Organizing queries with descriptions
//!
//! # Example
//!
//! ```rust
//! use jpx_engine::QueryStore;
//! use jpx_engine::StoredQuery;
//!
//! let mut store = QueryStore::new();
//!
//! // Define a query
//! store.define(StoredQuery {
//!     name: "active_users".to_string(),
//!     expression: "users[?active].name".to_string(),
//!     description: Some("Get names of active users".to_string()),
//! });
//!
//! // Retrieve it later
//! let query = store.get("active_users").unwrap();
//! assert_eq!(query.expression, "users[?active].name");
//! ```
//!
//! # Thread Safety
//!
//! The [`QueryStore`] itself is not thread-safe. When used through
//! [`JpxEngine`](crate::JpxEngine), it's wrapped in `Arc<RwLock<...>>`
//! for safe concurrent access.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A named JMESPath query with optional description.
///
/// Stored queries can be defined, retrieved, and executed by name.
///
/// # Example
///
/// ```rust
/// use jpx_engine::StoredQuery;
///
/// let query = StoredQuery {
///     name: "count_items".to_string(),
///     expression: "length(items)".to_string(),
///     description: Some("Count the number of items".to_string()),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredQuery {
    /// Unique identifier for the query
    pub name: String,
    /// The JMESPath expression
    pub expression: String,
    /// Human-readable description of what the query does
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// In-memory storage for named queries.
///
/// Provides CRUD operations for managing named JMESPath queries within a session.
/// Queries are stored by name and can be listed alphabetically.
///
/// # Example
///
/// ```rust
/// use jpx_engine::{QueryStore, StoredQuery};
///
/// let mut store = QueryStore::new();
///
/// // Add some queries
/// store.define(StoredQuery {
///     name: "first".to_string(),
///     expression: "@[0]".to_string(),
///     description: None,
/// });
///
/// store.define(StoredQuery {
///     name: "last".to_string(),
///     expression: "@[-1]".to_string(),
///     description: None,
/// });
///
/// // List them (alphabetically sorted)
/// let queries = store.list();
/// assert_eq!(queries[0].name, "first");
/// assert_eq!(queries[1].name, "last");
///
/// // Delete one
/// store.delete("first");
/// assert_eq!(store.len(), 1);
/// ```
#[derive(Debug, Default)]
pub struct QueryStore {
    queries: HashMap<String, StoredQuery>,
}

impl QueryStore {
    /// Creates a new empty query store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Stores a named query.
    ///
    /// If a query with the same name already exists, it is replaced and
    /// the old query is returned.
    ///
    /// # Returns
    ///
    /// `Some(StoredQuery)` if a query was replaced, `None` if this is a new name.
    pub fn define(&mut self, query: StoredQuery) -> Option<StoredQuery> {
        self.queries.insert(query.name.clone(), query)
    }

    /// Retrieves a query by name.
    ///
    /// # Returns
    ///
    /// `Some(&StoredQuery)` if found, `None` if no query has that name.
    pub fn get(&self, name: &str) -> Option<&StoredQuery> {
        self.queries.get(name)
    }

    /// Removes a query by name.
    ///
    /// # Returns
    ///
    /// `Some(StoredQuery)` containing the removed query, `None` if not found.
    pub fn delete(&mut self, name: &str) -> Option<StoredQuery> {
        self.queries.remove(name)
    }

    /// Lists all stored queries, sorted alphabetically by name.
    pub fn list(&self) -> Vec<&StoredQuery> {
        let mut queries: Vec<_> = self.queries.values().collect();
        queries.sort_by(|a, b| a.name.cmp(&b.name));
        queries
    }

    /// Returns the number of stored queries.
    pub fn len(&self) -> usize {
        self.queries.len()
    }

    /// Returns `true` if no queries are stored.
    pub fn is_empty(&self) -> bool {
        self.queries.is_empty()
    }

    /// Removes all stored queries.
    pub fn clear(&mut self) {
        self.queries.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_define_and_get() {
        let mut store = QueryStore::new();

        let query = StoredQuery {
            name: "count".to_string(),
            expression: "length(@)".to_string(),
            description: Some("Count items".to_string()),
        };

        assert!(store.define(query.clone()).is_none());
        assert_eq!(store.len(), 1);

        let retrieved = store.get("count").unwrap();
        assert_eq!(retrieved.name, "count");
        assert_eq!(retrieved.expression, "length(@)");
        assert_eq!(retrieved.description, Some("Count items".to_string()));
    }

    #[test]
    fn test_define_overwrites() {
        let mut store = QueryStore::new();

        let query1 = StoredQuery {
            name: "test".to_string(),
            expression: "length(@)".to_string(),
            description: None,
        };

        let query2 = StoredQuery {
            name: "test".to_string(),
            expression: "keys(@)".to_string(),
            description: Some("Updated".to_string()),
        };

        assert!(store.define(query1).is_none());
        let old = store.define(query2).unwrap();
        assert_eq!(old.expression, "length(@)");

        let current = store.get("test").unwrap();
        assert_eq!(current.expression, "keys(@)");
    }

    #[test]
    fn test_delete() {
        let mut store = QueryStore::new();

        let query = StoredQuery {
            name: "to_delete".to_string(),
            expression: "`null`".to_string(),
            description: None,
        };

        store.define(query);
        assert_eq!(store.len(), 1);

        let deleted = store.delete("to_delete").unwrap();
        assert_eq!(deleted.name, "to_delete");
        assert_eq!(store.len(), 0);

        assert!(store.delete("nonexistent").is_none());
    }

    #[test]
    fn test_list() {
        let mut store = QueryStore::new();

        store.define(StoredQuery {
            name: "zebra".to_string(),
            expression: "`1`".to_string(),
            description: None,
        });
        store.define(StoredQuery {
            name: "alpha".to_string(),
            expression: "`2`".to_string(),
            description: None,
        });
        store.define(StoredQuery {
            name: "beta".to_string(),
            expression: "`3`".to_string(),
            description: None,
        });

        let list = store.list();
        assert_eq!(list.len(), 3);
        // Should be sorted alphabetically
        assert_eq!(list[0].name, "alpha");
        assert_eq!(list[1].name, "beta");
        assert_eq!(list[2].name, "zebra");
    }

    #[test]
    fn test_clear() {
        let mut store = QueryStore::new();

        store.define(StoredQuery {
            name: "a".to_string(),
            expression: "`1`".to_string(),
            description: None,
        });
        store.define(StoredQuery {
            name: "b".to_string(),
            expression: "`2`".to_string(),
            description: None,
        });

        assert_eq!(store.len(), 2);
        store.clear();
        assert!(store.is_empty());
    }
}
