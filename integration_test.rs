use bb_engine::config::Config;
use bb_engine::modules::storage::Storage;
use bb_engine::modules::types::{Endpoint, EndpointSource};
use tempfile::TempDir;

#[test]
fn test_config_default() {
    let config = Config::default();
    assert_eq!(config.http.max_concurrent, 10);
    assert_eq!(config.http.timeout, 30);
}

#[test]
fn test_storage_creation() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let storage = Storage::new(&db_path).unwrap();

    // Verify database was created
    assert!(db_path.exists());
}

#[test]
fn test_endpoint_storage() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let storage = Storage::new(&db_path).unwrap();

    let endpoint = Endpoint::new(
        "https://example.com/api/users".to_string(),
        "GET".to_string(),
        EndpointSource::Crawled,
    );

    storage.save_endpoint(&endpoint).unwrap();

    let count = storage.count_endpoints().unwrap();
    assert_eq!(count, 1);

    let endpoints = storage.get_all_endpoints().unwrap();
    assert_eq!(endpoints.len(), 1);
    assert_eq!(endpoints[0].url, "https://example.com/api/users");
}
