/// Tests for SqlRepository implementation (GREEN PHASE)
use dbfast::sql_repository::SqlRepository;
use std::fs;
use tempfile::TempDir;

/// Test structured repository file discovery
/// RED PHASE: Should FAIL because SqlRepository doesn't exist
#[tokio::test]
async fn test_structured_repository_discovery() {
    let temp_dir = TempDir::new().unwrap();

    // Create structured repository directories
    let schema_dir = temp_dir.path().join("0_schema");
    let seed_common_dir = temp_dir.path().join("1_seed_common");
    let seed_dev_dir = temp_dir.path().join("2_seed_dev");

    fs::create_dir_all(&schema_dir).unwrap();
    fs::create_dir_all(&seed_common_dir).unwrap();
    fs::create_dir_all(&seed_dev_dir).unwrap();

    // Create SQL files
    fs::write(
        schema_dir.join("001_users.sql"),
        "CREATE TABLE users (id SERIAL PRIMARY KEY);",
    )
    .unwrap();
    fs::write(
        seed_common_dir.join("001_default_users.sql"),
        "INSERT INTO users (id) VALUES (1);",
    )
    .unwrap();
    fs::write(
        seed_dev_dir.join("001_dev_users.sql"),
        "INSERT INTO users (id) VALUES (2);",
    )
    .unwrap();

    let repo = SqlRepository::new(temp_dir.path()).unwrap();

    // Should find all SQL files in correct order
    let files = repo.discover_sql_files(&["dev"]).await.unwrap();

    assert_eq!(files.len(), 3);
    assert!(files[0].to_string_lossy().contains("0_schema"));
    assert!(files[1].to_string_lossy().contains("1_seed_common"));
    assert!(files[2].to_string_lossy().contains("2_seed_dev"));
}

/// Test flat repository file discovery
/// RED PHASE: Should FAIL because SqlRepository doesn't exist
#[tokio::test]
async fn test_flat_repository_discovery() {
    let temp_dir = TempDir::new().unwrap();

    // Create flat repository with SQL files
    fs::write(
        temp_dir.path().join("001_schema.sql"),
        "CREATE TABLE items (id SERIAL);",
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("002_seed.sql"),
        "INSERT INTO items (id) VALUES (1);",
    )
    .unwrap();
    fs::write(temp_dir.path().join("readme.txt"), "Not a SQL file").unwrap();

    let repo = SqlRepository::new(temp_dir.path()).unwrap();

    // Should find only SQL files in alphabetical order
    let files = repo.discover_sql_files(&[]).await.unwrap();

    assert_eq!(files.len(), 2);
    assert!(files[0]
        .file_name()
        .unwrap()
        .to_string_lossy()
        .starts_with("001_"));
    assert!(files[1]
        .file_name()
        .unwrap()
        .to_string_lossy()
        .starts_with("002_"));
}

/// Test environment-based filtering
/// RED PHASE: Should FAIL because SqlRepository doesn't exist
#[tokio::test]
async fn test_environment_filtering() {
    let temp_dir = TempDir::new().unwrap();

    // Create environment-specific directories
    let prod_dir = temp_dir.path().join("3_seed_prod");
    let dev_dir = temp_dir.path().join("3_seed_dev");
    let test_dir = temp_dir.path().join("3_seed_test");

    fs::create_dir_all(&prod_dir).unwrap();
    fs::create_dir_all(&dev_dir).unwrap();
    fs::create_dir_all(&test_dir).unwrap();

    fs::write(
        prod_dir.join("prod_data.sql"),
        "INSERT INTO config VALUES ('prod');",
    )
    .unwrap();
    fs::write(
        dev_dir.join("dev_data.sql"),
        "INSERT INTO config VALUES ('dev');",
    )
    .unwrap();
    fs::write(
        test_dir.join("test_data.sql"),
        "INSERT INTO config VALUES ('test');",
    )
    .unwrap();

    let repo = SqlRepository::new(temp_dir.path()).unwrap();

    // Should only include files from specified environments
    let dev_files = repo.discover_sql_files(&["dev"]).await.unwrap();
    let prod_files = repo.discover_sql_files(&["prod"]).await.unwrap();

    assert_eq!(dev_files.len(), 1);
    assert!(dev_files[0].to_string_lossy().contains("dev_data.sql"));

    assert_eq!(prod_files.len(), 1);
    assert!(prod_files[0].to_string_lossy().contains("prod_data.sql"));
}

/// Test SQL content loading
/// RED PHASE: Should FAIL because SqlRepository doesn't exist
#[tokio::test]
async fn test_sql_content_loading() {
    let temp_dir = TempDir::new().unwrap();
    let sql_file = temp_dir.path().join("test.sql");
    let sql_content = "CREATE TABLE test_load (id SERIAL, data TEXT);";

    fs::write(&sql_file, sql_content).unwrap();

    let repo = SqlRepository::new(temp_dir.path()).unwrap();

    let loaded_content = repo.load_sql_content(&sql_file).await.unwrap();

    assert_eq!(loaded_content.trim(), sql_content);
}
