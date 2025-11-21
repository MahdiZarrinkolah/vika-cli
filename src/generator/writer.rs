use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use crate::generator::api_client::ApiFunction;
use crate::generator::ts_typings::TypeScriptType;
use crate::generator::zod_schema::ZodSchema;

pub fn ensure_directory(path: &Path) -> Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)
            .with_context(|| format!("Failed to create directory: {}", path.display()))?;
    }
    Ok(())
}

pub fn write_schemas(
    output_dir: &Path,
    module_name: &str,
    types: &[TypeScriptType],
    zod_schemas: &[ZodSchema],
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
        write_file_safe(&types_file, &types_content)?;
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
        write_file_safe(&zod_file, &zod_content)?;
        written_files.push(zod_file);
    }

    // Write index file
    let mut index_exports = Vec::new();
    if !types.is_empty() {
        index_exports.push("export * from \"./types\";".to_string());
    }
    if !zod_schemas.is_empty() {
        index_exports.push("export * from \"./schemas\";".to_string());
    }

    if !index_exports.is_empty() {
        let index_content = format_typescript_code(&(index_exports.join("\n") + "\n"));
        let index_file = module_dir.join("index.ts");
        write_file_safe(&index_file, &index_content)?;
        written_files.push(index_file);
    }

    Ok(written_files)
}

pub fn write_api_client(
    output_dir: &Path,
    module_name: &str,
    functions: &[ApiFunction],
) -> Result<Vec<PathBuf>> {
    let module_dir = output_dir.join(module_name);
    ensure_directory(&module_dir)?;

    let mut written_files = Vec::new();

    if !functions.is_empty() {
        let functions_content = format_typescript_code(&format!(
            "{}",
            functions.iter()
                .map(|f| f.content.clone())
                .collect::<Vec<_>>()
                .join("\n\n")
        ));

        let api_file = module_dir.join("index.ts");
        write_file_safe(&api_file, &functions_content)?;
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

fn write_file_safe(path: &Path, content: &str) -> Result<()> {
    // Check if file exists and content is different
    if path.exists() {
        if let Ok(existing_content) = std::fs::read_to_string(path) {
            if existing_content == content {
                // Content is the same, skip writing
                return Ok(());
            }
        }
    }
    
    std::fs::write(path, content)
        .with_context(|| format!("Failed to write file: {}", path.display()))?;
    
    Ok(())
}

