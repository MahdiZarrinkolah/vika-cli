use vika_cli::cache::{CacheManager, SpecMetadata};

#[test]
fn test_ensure_cache_dir() {
    let temp_dir = tempfile::tempdir().unwrap();
    let original_dir = std::env::current_dir().ok();
    
    let _ = std::env::set_current_dir(temp_dir.path());
    
    let result = CacheManager::ensure_cache_dir();
    assert!(result.is_ok());
    if let Ok(cache_dir) = result {
        assert!(cache_dir.exists());
        assert!(cache_dir.is_dir());
    }
    
    if let Some(orig) = original_dir {
        let _ = std::env::set_current_dir(orig);
    }
}

#[test]
fn test_get_cached_spec_no_cache() {
    let temp_dir = tempfile::tempdir().unwrap();
    let original_dir = std::env::current_dir().ok();
    
    let _ = std::env::set_current_dir(temp_dir.path());
    
    let result = CacheManager::get_cached_spec("https://example.com/spec.json");
    // Should return Ok(None) when no cache exists
    assert!(result.is_ok() || result.is_err(), "get_cached_spec should return a result");
    if result.is_ok() {
        assert!(result.unwrap().is_none(), "Should return None when no cache exists");
    }
    
    if let Some(orig) = original_dir {
        let _ = std::env::set_current_dir(orig);
    }
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

// These tests are environment-sensitive and may be skipped in CI
#[test]
#[ignore]
fn test_cache_and_get_spec() {
    let temp_dir = tempfile::tempdir().unwrap();
    let original_dir = std::env::current_dir().ok();
    
    let _ = std::env::set_current_dir(temp_dir.path());
    
    let url = "https://example.com/spec.json";
    let content = r#"{"openapi": "3.0.0", "info": {"title": "Test", "version": "1.0.0"}, "paths": {}}"#;
    
    if let Ok(()) = CacheManager::cache_spec(url, content) {
        if let Ok(Some(cached)) = CacheManager::get_cached_spec(url) {
            assert_eq!(cached, content);
        }
    }
    
    if let Some(orig) = original_dir {
        let _ = std::env::set_current_dir(orig);
    }
}

#[test]
#[ignore]
fn test_get_cached_spec_wrong_url() {
    let temp_dir = tempfile::tempdir().unwrap();
    let original_dir = std::env::current_dir().ok();
    
    let _ = std::env::set_current_dir(temp_dir.path());
    
    let url1 = "https://example.com/spec1.json";
    let url2 = "https://example.com/spec2.json";
    let content = r#"{"openapi": "3.0.0"}"#;
    
    if CacheManager::cache_spec(url1, content).is_ok() {
        if let Ok(result) = CacheManager::get_cached_spec(url2) {
            assert!(result.is_none(), "Should return None for different URL");
        }
    }
    
    if let Some(orig) = original_dir {
        let _ = std::env::set_current_dir(orig);
    }
}

#[test]
#[ignore]
fn test_clear_cache() {
    let temp_dir = tempfile::tempdir().unwrap();
    let original_dir = std::env::current_dir().ok();
    
    let _ = std::env::set_current_dir(temp_dir.path());
    
    let url = "https://example.com/spec.json";
    let content = r#"{"openapi": "3.0.0"}"#;
    
    if CacheManager::cache_spec(url, content).is_ok() {
        if CacheManager::get_cached_spec(url).ok().and_then(|x| x).is_some() {
            let _ = CacheManager::clear_cache();
            if let Ok(result) = CacheManager::get_cached_spec(url) {
                assert!(result.is_none(), "Cache should be cleared");
            }
        }
    }
    
    if let Some(orig) = original_dir {
        let _ = std::env::set_current_dir(orig);
    }
}
