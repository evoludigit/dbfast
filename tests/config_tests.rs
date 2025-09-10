use dbfast::Config;

#[test]
fn test_config_loading() {
    let config = Config::from_file("tests/fixtures/dbfast.toml").unwrap();
    assert_eq!(config.database.host, "localhost");
    assert_eq!(config.database.port, 5432);
    assert_eq!(config.database.user, "postgres");
    assert_eq!(config.repository.path, "./db");
}

#[test] 
fn test_config_missing_file() {
    let result = Config::from_file("nonexistent.toml");
    assert!(result.is_err());
}