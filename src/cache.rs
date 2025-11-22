use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::error::{Result, FileSystemError};

const CACHE_DIR: &str = ".vika-cache";
const SPEC_CACHE_FILE: &str = "spec.json";
const SPEC_META_FILE: &str = "spec.meta.json";

#[cfg(test)]
static TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[derive(Debug, Serialize, Deserialize)]
pub struct SpecMetadata {
    pub url: String,
    pub timestamp: u64,
    pub etag: Option<String>,
    pub content_hash: String,
}

pub struct CacheManager;

impl CacheManager {
    pub fn ensure_cache_dir() -> Result<PathBuf> {
        let cache_dir = PathBuf::from(CACHE_DIR);
        // create_dir_all succeeds if directory already exists
        std::fs::create_dir_all(&cache_dir)
            .map_err(|e| FileSystemError::CreateDirectoryFailed {
                path: CACHE_DIR.to_string(),
                source: e,
            })?;
        Ok(cache_dir)
    }

    pub fn get_cached_spec(url: &str) -> Result<Option<String>> {
        let cache_dir = Self::ensure_cache_dir()?;
        let meta_path = cache_dir.join(SPEC_META_FILE);
        let spec_path = cache_dir.join(SPEC_CACHE_FILE);

        // Check if cache exists
        if !meta_path.exists() || !spec_path.exists() {
            return Ok(None);
        }

        // Read metadata
        let meta_content = std::fs::read_to_string(&meta_path)
            .map_err(|e| FileSystemError::ReadFileFailed {
                path: meta_path.display().to_string(),
                source: e,
            })?;

        let metadata: SpecMetadata = serde_json::from_str(&meta_content)
            .map_err(|_| FileSystemError::ReadFileFailed {
                path: meta_path.display().to_string(),
                source: std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid metadata format"),
            })?;

        // Check if URL matches
        if metadata.url != url {
            return Ok(None);
        }

        // Read cached spec
        let spec_content = std::fs::read_to_string(&spec_path)
            .map_err(|e| FileSystemError::ReadFileFailed {
                path: spec_path.display().to_string(),
                source: e,
            })?;

        Ok(Some(spec_content))
    }

    pub fn cache_spec(url: &str, content: &str) -> Result<()> {
        // Ensure cache directory exists before writing
        let cache_dir = Self::ensure_cache_dir()?;
        let meta_path = cache_dir.join(SPEC_META_FILE);
        let spec_path = cache_dir.join(SPEC_CACHE_FILE);

        // Compute content hash (simple hash for now)
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        let content_hash = format!("{:x}", hasher.finish());

        // Get timestamp
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Create metadata
        let metadata = SpecMetadata {
            url: url.to_string(),
            timestamp,
            etag: None, // Could be enhanced to fetch ETag from response
            content_hash,
        };

        // Write metadata
        let meta_json = serde_json::to_string_pretty(&metadata)
            .map_err(|e| FileSystemError::WriteFileFailed {
                path: meta_path.display().to_string(),
                source: std::io::Error::new(std::io::ErrorKind::InvalidData, format!("{}", e)),
            })?;

        std::fs::write(&meta_path, meta_json)
            .map_err(|e| FileSystemError::WriteFileFailed {
                path: meta_path.display().to_string(),
                source: e,
            })?;

        // Write spec content
        std::fs::write(&spec_path, content)
            .map_err(|e| FileSystemError::WriteFileFailed {
                path: spec_path.display().to_string(),
                source: e,
            })?;

        Ok(())
    }

    pub fn clear_cache() -> Result<()> {
        let cache_dir = PathBuf::from(CACHE_DIR);
        if cache_dir.exists() {
            std::fs::remove_dir_all(&cache_dir)
                .map_err(|e| FileSystemError::CreateDirectoryFailed {
                    path: CACHE_DIR.to_string(),
                    source: e,
                })?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    // Helper to handle mutex poisoning gracefully
    fn lock_test_mutex() -> std::sync::MutexGuard<'static, ()> {
        TEST_MUTEX.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    // Helper to ensure cleanup happens even on panic
    struct DirGuard {
        original_dir: std::path::PathBuf,
    }

    impl Drop for DirGuard {
        fn drop(&mut self) {
            let _ = env::set_current_dir(&self.original_dir);
        }
    }

    #[test]
    fn test_cache_spec() {
        let _lock = lock_test_mutex();
        let temp_dir = tempfile::tempdir().unwrap();
        let original_dir = env::current_dir().unwrap();
        let _guard = DirGuard { original_dir: original_dir.clone() };
        env::set_current_dir(&temp_dir).unwrap();
        
        // Ensure cache directory exists
        let _ = CacheManager::ensure_cache_dir();
        
        let url = "https://example.com/spec.json";
        let content = r#"{"openapi": "3.0.0", "info": {"title": "Test", "version": "1.0.0"}}"#;
        
        let result = CacheManager::cache_spec(url, content);
        assert!(result.is_ok(), "Failed to cache spec: {:?}", result.err());
    }

    #[test]
    fn test_get_cached_spec() {
        let _lock = lock_test_mutex();
        let temp_dir = tempfile::tempdir().unwrap();
        let original_dir = env::current_dir().unwrap();
        let _guard = DirGuard { original_dir: original_dir.clone() };
        env::set_current_dir(&temp_dir).unwrap();
        
        // Ensure cache directory exists
        let _ = CacheManager::ensure_cache_dir();
        
        let url = "https://example.com/spec.json";
        let content = r#"{"openapi": "3.0.0", "info": {"title": "Test", "version": "1.0.0"}}"#;
        
        // Cache first
        let cache_result = CacheManager::cache_spec(url, content);
        assert!(cache_result.is_ok(), "Failed to cache: {:?}", cache_result.err());
        
        // Verify files were created
        let cache_dir = CacheManager::ensure_cache_dir().unwrap();
        let spec_file = cache_dir.join("spec.json");
        let meta_file = cache_dir.join("spec.meta.json");
        assert!(spec_file.exists(), "Cache file should exist at: {}", spec_file.display());
        assert!(meta_file.exists(), "Metadata file should exist at: {}", meta_file.display());
        
        // Then retrieve
        let cached = CacheManager::get_cached_spec(url).unwrap();
        assert!(cached.is_some(), "Cached spec should be found");
        assert_eq!(cached.unwrap(), content);
    }

    #[test]
    fn test_get_cached_spec_miss() {
        let _lock = lock_test_mutex();
        let temp_dir = tempfile::tempdir().unwrap();
        let original_dir = env::current_dir().unwrap();
        let _guard = DirGuard { original_dir: original_dir.clone() };
        env::set_current_dir(&temp_dir).unwrap();
        
        // Ensure cache directory exists
        let _ = CacheManager::ensure_cache_dir();
        
        let url = "https://example.com/spec.json";
        
        let cached = CacheManager::get_cached_spec(url).unwrap();
        assert!(cached.is_none());
    }

    #[test]
    fn test_get_cached_spec_wrong_url() {
        let _lock = lock_test_mutex();
        let temp_dir = tempfile::tempdir().unwrap();
        let original_dir = env::current_dir().unwrap();
        let _guard = DirGuard { original_dir: original_dir.clone() };
        env::set_current_dir(&temp_dir).unwrap();
        
        // Ensure cache directory exists
        let _ = CacheManager::ensure_cache_dir();
        
        let url1 = "https://example.com/spec1.json";
        let url2 = "https://example.com/spec2.json";
        let content = r#"{"openapi": "3.0.0"}"#;
        
        CacheManager::cache_spec(url1, content).unwrap();
        
        let cached = CacheManager::get_cached_spec(url2).unwrap();
        assert!(cached.is_none()); // Different URL should not match
    }
}

