use crate::error::{Result, FileSystemError};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use crate::generator::api_client::ApiFunction;
use crate::generator::ts_typings::TypeScriptType;
use crate::generator::zod_schema::ZodSchema;

pub fn ensure_directory(path: &Path) -> Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)
            .map_err(|e| FileSystemError::CreateDirectoryFailed {
                path: path.display().to_string(),
                source: e,
            })?;
    }
    Ok(())
}

pub fn write_schemas(
    output_dir: &Path,
    module_name: &str,
    types: &[TypeScriptType],
    zod_schemas: &[ZodSchema],
) -> Result<Vec<PathBuf>> {
    write_schemas_with_options(output_dir, module_name, types, zod_schemas, false, false)
}

pub fn write_schemas_with_options(
    output_dir: &Path,
    module_name: &str,
    types: &[TypeScriptType],
    zod_schemas: &[ZodSchema],
    backup: bool,
    force: bool,
) -> Result<Vec<PathBuf>> {
    let module_dir = output_dir.join(module_name);
    ensure_directory(&module_dir)?;

    let mut written_files = Vec::new();

    // Write TypeScript types
    if !types.is_empty() {
        let types_content = format_typescript_code(&format!(
            "{}",
            types.iter()
                .map(|t| t.content.clone())
                .collect::<Vec<_>>()
                .join("\n\n")
        ));
        
        let types_file = module_dir.join("types.ts");
        write_file_with_backup(&types_file, &types_content, backup, force)?;
        written_files.push(types_file);
    }

    // Write Zod schemas
    if !zod_schemas.is_empty() {
        let zod_content = format_typescript_code(&format!(
            "import {{ z }} from \"zod\";\n\n{}",
            zod_schemas.iter()
                .map(|z| z.content.clone())
                .collect::<Vec<_>>()
                .join("\n\n")
        ));
        
        let zod_file = module_dir.join("schemas.ts");
        write_file_with_backup(&zod_file, &zod_content, backup, force)?;
        written_files.push(zod_file);
    }

    // Write index file with namespace export for better organization
    let mut index_exports = Vec::new();
    if !types.is_empty() {
        index_exports.push("export * from \"./types\";".to_string());
    }
    if !zod_schemas.is_empty() {
        index_exports.push("export * from \"./schemas\";".to_string());
    }

    if !index_exports.is_empty() {
        // Write index file with regular exports
        // Note: TypeScript namespaces cannot use export *, so we use regular exports
        // and import as namespace in API clients for better organization
        let index_content = format_typescript_code(&(index_exports.join("\n") + "\n"));
        let index_file = module_dir.join("index.ts");
        write_file_with_backup(&index_file, &index_content, backup, force)?;
        written_files.push(index_file);
    }

    Ok(written_files)
}

pub fn write_api_client(
    output_dir: &Path,
    module_name: &str,
    functions: &[ApiFunction],
) -> Result<Vec<PathBuf>> {
    write_api_client_with_options(output_dir, module_name, functions, false, false)
}

pub fn write_api_client_with_options(
    output_dir: &Path,
    module_name: &str,
    functions: &[ApiFunction],
    backup: bool,
    force: bool,
) -> Result<Vec<PathBuf>> {
    let module_dir = output_dir.join(module_name);
    ensure_directory(&module_dir)?;

    let mut written_files = Vec::new();

    if !functions.is_empty() {
        // Consolidate imports: extract all imports and deduplicate them
        let mut all_imports: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut function_bodies = Vec::new();
        
        for func in functions {
            let lines: Vec<&str> = func.content.lines().collect();
            let mut func_lines = Vec::new();
            let mut in_function = false;
            
            for line in lines {
                if line.trim().starts_with("import ") {
                    all_imports.insert(line.trim().to_string());
                } else if line.trim().starts_with("export const ") {
                    in_function = true;
                    func_lines.push(line);
                } else if in_function {
                    func_lines.push(line);
                }
            }
            
            if !func_lines.is_empty() {
                function_bodies.push(func_lines.join("\n"));
            }
        }
        
        // Combine imports and function bodies
        let imports_vec: Vec<String> = all_imports.iter().cloned().collect();
        let imports_str = imports_vec.join("\n");
        let functions_str = function_bodies.join("\n\n");
        let combined_content = if !imports_str.is_empty() {
            format!("{}\n\n{}", imports_str, functions_str)
        } else {
            functions_str
        };
        
        let functions_content = format_typescript_code(&combined_content);

        let api_file = module_dir.join("index.ts");
        write_file_with_backup(&api_file, &functions_content, backup, force)?;
        written_files.push(api_file);
    }

    Ok(written_files)
}

pub fn write_http_client_template(output_path: &Path) -> Result<()> {
    ensure_directory(output_path.parent().unwrap_or(Path::new(".")))?;
    
    let http_client_content = r#"export const http = {
  async get<T = any>(url: string, options: RequestInit = {}): Promise<T> {
    const response = await fetch(url, {
      ...options,
      method: "GET",
    });
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    return response.json();
  },

  async post<T = any>(url: string, body?: any, options: RequestInit = {}): Promise<T> {
    const response = await fetch(url, {
      ...options,
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...(options.headers || {}),
      },
      body: body ? JSON.stringify(body) : undefined,
    });
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    return response.json();
  },

  async put<T = any>(url: string, body?: any, options: RequestInit = {}): Promise<T> {
    const response = await fetch(url, {
      ...options,
      method: "PUT",
      headers: {
        "Content-Type": "application/json",
        ...(options.headers || {}),
      },
      body: body ? JSON.stringify(body) : undefined,
    });
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    return response.json();
  },

  async delete<T = any>(url: string, options: RequestInit = {}): Promise<T> {
    const response = await fetch(url, {
      ...options,
      method: "DELETE",
    });
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    return response.json();
  },

  async patch<T = any>(url: string, body?: any, options: RequestInit = {}): Promise<T> {
    const response = await fetch(url, {
      ...options,
      method: "PATCH",
      headers: {
        "Content-Type": "application/json",
        ...(options.headers || {}),
      },
      body: body ? JSON.stringify(body) : undefined,
    });
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    return response.json();
  },
};
"#;

    write_file_safe(output_path, http_client_content)?;

    Ok(())
}

fn format_typescript_code(code: &str) -> String {
    // Basic formatting: ensure consistent spacing and remove extra blank lines
    let lines: Vec<&str> = code.lines().collect();
    let mut formatted = Vec::new();
    let mut last_was_empty = false;
    
    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if !last_was_empty && !formatted.is_empty() {
                formatted.push(String::new());
                last_was_empty = true;
            }
            continue;
        }
        last_was_empty = false;
        formatted.push(trimmed.to_string());
    }
    
    // Remove trailing empty lines
    while formatted.last().map(|s| s.is_empty()).unwrap_or(false) {
        formatted.pop();
    }
    
    formatted.join("\n")
}

pub fn write_file_safe(path: &Path, content: &str) -> Result<()> {
    write_file_with_backup(path, content, false, false)
}

pub fn write_file_with_backup(
    path: &Path,
    content: &str,
    backup: bool,
    force: bool,
) -> Result<()> {
    // Check if file exists and content is different
    let file_exists = path.exists();
    let should_write = if file_exists {
        if let Ok(existing_content) = std::fs::read_to_string(path) {
            existing_content != content
        } else {
            true
        }
    } else {
        true
    };

    if !should_write {
        // Content is the same, skip writing
        return Ok(());
    }

    // Create backup if requested and file exists
    if backup && file_exists {
        create_backup(path)?;
    }

    // Check for conflicts (user modifications) if not forcing
    if !force && file_exists {
        if let Ok(metadata) = load_file_metadata(path) {
            let current_hash = compute_content_hash(content);
            let file_hash = compute_file_hash(path)?;
            if metadata.hash != current_hash && metadata.hash != file_hash {
                // File was modified by user
                return Err(FileSystemError::FileModifiedByUser {
                    path: path.display().to_string(),
                }.into());
            }
        }
    }

    // Write the file
    std::fs::write(path, content)
        .map_err(|e| FileSystemError::WriteFileFailed {
            path: path.display().to_string(),
            source: e,
        })?;

    // Save metadata
    save_file_metadata(path, content)?;

    Ok(())
}

fn create_backup(path: &Path) -> Result<()> {
    use std::time::{SystemTime, UNIX_EPOCH};
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let backup_dir = PathBuf::from(format!(".vika-backup/{}", timestamp));
    std::fs::create_dir_all(&backup_dir)
        .map_err(|e| FileSystemError::CreateDirectoryFailed {
            path: backup_dir.display().to_string(),
            source: e,
        })?;

    // Determine backup path
    let backup_path = if path.is_absolute() {
        // For absolute paths (e.g., from temp directories in tests),
        // use a hash-based filename to avoid very long paths
        let path_str = path.display().to_string();
        let mut hasher = DefaultHasher::new();
        path_str.hash(&mut hasher);
        let hash = format!("{:x}", hasher.finish());
        let filename = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file");
        backup_dir.join(format!("{}_{}", hash, filename))
    } else {
        // For relative paths, preserve directory structure
        let relative_path = path.strip_prefix(".")
            .unwrap_or(path);
        backup_dir.join(relative_path)
    };
    
    if let Some(parent) = backup_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| FileSystemError::CreateDirectoryFailed {
                path: parent.display().to_string(),
                source: e,
            })?;
    }

    std::fs::copy(path, &backup_path)
        .map_err(|e| FileSystemError::WriteFileFailed {
            path: backup_path.display().to_string(),
            source: e,
        })?;

    Ok(())
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct FileMetadata {
    hash: String,
    generated_at: u64,
    generated_by: String,
}

fn compute_content_hash(content: &str) -> String {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

fn compute_file_hash(path: &Path) -> Result<String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| FileSystemError::ReadFileFailed {
            path: path.display().to_string(),
            source: e,
        })?;
    Ok(compute_content_hash(&content))
}

fn save_file_metadata(path: &Path, content: &str) -> Result<()> {
    let metadata_dir = PathBuf::from(".vika-cache");
    std::fs::create_dir_all(&metadata_dir)
        .map_err(|e| FileSystemError::CreateDirectoryFailed {
            path: metadata_dir.display().to_string(),
            source: e,
        })?;

    let metadata_file = metadata_dir.join("file-metadata.json");
    let mut metadata_map: std::collections::HashMap<String, FileMetadata> = if metadata_file.exists() {
        let content = std::fs::read_to_string(&metadata_file)
            .map_err(|e| FileSystemError::ReadFileFailed {
                path: metadata_file.display().to_string(),
                source: e,
            })?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        std::collections::HashMap::new()
    };

    let hash = compute_content_hash(content);
    let generated_at = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    metadata_map.insert(
        path.display().to_string(),
        FileMetadata {
            hash,
            generated_at,
            generated_by: "vika-cli".to_string(),
        },
    );

    let json = serde_json::to_string_pretty(&metadata_map)
        .map_err(|e| FileSystemError::WriteFileFailed {
            path: metadata_file.display().to_string(),
            source: std::io::Error::new(std::io::ErrorKind::InvalidData, format!("{}", e)),
        })?;

    std::fs::write(&metadata_file, json)
        .map_err(|e| FileSystemError::WriteFileFailed {
            path: metadata_file.display().to_string(),
            source: e,
        })?;

    Ok(())
}

fn load_file_metadata(path: &Path) -> Result<FileMetadata> {
    let metadata_file = PathBuf::from(".vika-cache/file-metadata.json");
    if !metadata_file.exists() {
        return Err(FileSystemError::FileNotFound {
            path: metadata_file.display().to_string(),
        }.into());
    }

    let content = std::fs::read_to_string(&metadata_file)
        .map_err(|e| FileSystemError::ReadFileFailed {
            path: metadata_file.display().to_string(),
            source: e,
        })?;

    let metadata_map: std::collections::HashMap<String, FileMetadata> = serde_json::from_str(&content)
        .map_err(|e| FileSystemError::ReadFileFailed {
            path: metadata_file.display().to_string(),
            source: std::io::Error::new(std::io::ErrorKind::InvalidData, format!("{}", e)),
        })?;

    metadata_map
        .get(&path.display().to_string())
        .cloned()
        .ok_or_else(|| FileSystemError::FileNotFound {
            path: path.display().to_string(),
        }.into())
}

