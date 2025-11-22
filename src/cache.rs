use crate::error::{FileSystemError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const CACHE_DIR: &str = ".vika-cache";
const SPEC_CACHE_FILE: &str = "spec.json";
const SPEC_META_FILE: &str = "spec.meta.json";

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
        std::fs::create_dir_all(&cache_dir).map_err(|e| {
            FileSystemError::CreateDirectoryFailed {
                path: CACHE_DIR.to_string(),
                source: e,
            }
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
        let meta_content =
            std::fs::read_to_string(&meta_path).map_err(|e| FileSystemError::ReadFileFailed {
                path: meta_path.display().to_string(),
                source: e,
            })?;

        let metadata: SpecMetadata =
            serde_json::from_str(&meta_content).map_err(|_| FileSystemError::ReadFileFailed {
                path: meta_path.display().to_string(),
                source: std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid metadata format",
                ),
            })?;

        // Check if URL matches
        if metadata.url != url {
            return Ok(None);
        }

        // Read cached spec
        let spec_content =
            std::fs::read_to_string(&spec_path).map_err(|e| FileSystemError::ReadFileFailed {
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
        let meta_json = serde_json::to_string_pretty(&metadata).map_err(|e| {
            FileSystemError::WriteFileFailed {
                path: meta_path.display().to_string(),
                source: std::io::Error::new(std::io::ErrorKind::InvalidData, format!("{}", e)),
            }
        })?;

        std::fs::write(&meta_path, meta_json).map_err(|e| FileSystemError::WriteFileFailed {
            path: meta_path.display().to_string(),
            source: e,
        })?;

        // Write spec content
        std::fs::write(&spec_path, content).map_err(|e| FileSystemError::WriteFileFailed {
            path: spec_path.display().to_string(),
            source: e,
        })?;

        Ok(())
    }

    pub fn clear_cache() -> Result<()> {
        let cache_dir = PathBuf::from(CACHE_DIR);
        if cache_dir.exists() {
            std::fs::remove_dir_all(&cache_dir).map_err(|e| {
                FileSystemError::CreateDirectoryFailed {
                    path: CACHE_DIR.to_string(),
                    source: e,
                }
            })?;
        }
        Ok(())
    }
}
