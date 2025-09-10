//! Integration tests for dbfast
//! Generated on 2025-09-10

use dbfast::{Connection, QueryBuilder};

#[tokio::test]
async fn test_connection_creation() {
    let conn = Connection::new("postgresql://localhost:5432/test".to_string());
    assert_eq!(conn.url(), "postgresql://localhost:5432/test");
}

#[tokio::test]
async fn test_query_builder() {
    let query = QueryBuilder::new()
        .where_clause("id = 1")
        .param("test".to_string())
        .build();
    
    assert_eq!(query, "WHERE id = 1");
}

#[tokio::test]
async fn test_query_builder_multiple_conditions() {
    let query = QueryBuilder::new()
        .where_clause("id = 1")
        .where_clause("name = 'test'")
        .build();
    
    assert_eq!(query, "WHERE id = 1 AND name = 'test'");
}