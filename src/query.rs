//! Query building utilities

use serde::{Deserialize, Serialize};

/// Query builder for constructing database queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryBuilder {
    query: String,
    params: Vec<String>,
}

impl QueryBuilder {
    /// Create a new query builder
    #[must_use]
    pub fn new() -> Self {
        Self {
            query: String::new(),
            params: Vec::new(),
        }
    }

    /// Add a WHERE clause to the query
    #[must_use]
    pub fn where_clause(mut self, condition: &str) -> Self {
        if self.query.is_empty() {
            self.query = format!("WHERE {condition}");
        } else {
            self.query = format!("{} AND {condition}", self.query);
        }
        self
    }

    /// Add a parameter to the query
    #[must_use]
    pub fn param(mut self, param: String) -> Self {
        self.params.push(param);
        self
    }

    /// Build the final query string
    #[must_use]
    pub fn build(self) -> String {
        self.query
    }

    /// Get the parameters
    #[must_use]
    pub fn params(&self) -> &[String] {
        &self.params
    }
}

impl Default for QueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}
