use tempfile::TempDir;
use vika_cli::cache::{CacheManager, SpecMetadata};

#[test]
    fn test_ensure_cache_dir() {
        let temp_dir = tempfile::tempdir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        
        let result = CacheManager::ensure_cache_dir();
        assert!(result.is_ok());
        let cache_dir = result.unwrap();
        assert!(cache_dir.exists());
        assert!(cache_dir.is_dir());
    }

    #[test]
    fn test_get_cached_spec_no_cache() {
        let temp_dir = tempfile::tempdir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        
        let result = CacheManager::get_cached_spec("https://example.com/spec.json");
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_cache_and_get_spec() {
        let temp_dir = tempfile::tempdir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        
        let url = "https://example.com/spec.json";
        let content = r#"{"openapi": "3.0.0", "info": {"title": "Test", "version": "1.0.0"}, "paths": {}}"#;
        
        // Cache the spec
        let result = CacheManager::cache_spec(url, content);
        assert!(result.is_ok());
        
        // Retrieve it
        let result = CacheManager::get_cached_spec(url);
        assert!(result.is_ok());
        let cached = result.unwrap();
        assert!(cached.is_some());
        assert_eq!(cached.unwrap(), content);
    }

    #[test]
    fn test_get_cached_spec_wrong_url() {
        let temp_dir = tempfile::tempdir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        
        let url1 = "https://example.com/spec1.json";
        let url2 = "https://example.com/spec2.json";
        let content = r#"{"openapi": "3.0.0"}"#;
        
        CacheManager::cache_spec(url1, content).unwrap();
        
        let result = CacheManager::get_cached_spec(url2);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_clear_cache() {
        let temp_dir = tempfile::tempdir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        
        let url = "https://example.com/spec.json";
        let content = r#"{"openapi": "3.0.0"}"#;
        
        CacheManager::cache_spec(url, content).unwrap();
        assert!(CacheManager::get_cached_spec(url).unwrap().is_some());
        
        CacheManager::clear_cache().unwrap();
        assert!(CacheManager::get_cached_spec(url).unwrap().is_none());
    }

    #[test]
    fn test_spec_metadata_serialization() {
        let metadata = SpecMetadata {
            url: "https://example.com/spec.json".to_string(),
            timestamp: 1234567890,
            etag: Some("etag123".to_string()),
            content_hash: "abc123".to_string(),
        };
        
        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: SpecMetadata = serde_json::from_str(&json).unwrap();
        
        assert_eq!(metadata.url, deserialized.url);
        assert_eq!(metadata.timestamp, deserialized.timestamp);
        assert_eq!(metadata.etag, deserialized.etag);
        assert_eq!(metadata.content_hash, deserialized.content_hash);
    }

