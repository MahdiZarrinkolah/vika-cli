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
        let types_content = format!(
            "{}\n",
            types.iter()
                .map(|t| t.content.clone())
                .collect::<Vec<_>>()
                .join("\n\n")
        );
        
        let types_file = module_dir.join("types.ts");
        std::fs::write(&types_file, types_content)
            .with_context(|| format!("Failed to write types file: {}", types_file.display()))?;
        written_files.push(types_file);
    }

    // Write Zod schemas
    if !zod_schemas.is_empty() {
        let zod_content = format!(
            "import {{ z }} from \"zod\";\n\n{}\n",
            zod_schemas.iter()
                .map(|z| z.content.clone())
                .collect::<Vec<_>>()
                .join("\n\n")
        );
        
        let zod_file = module_dir.join("schemas.ts");
        std::fs::write(&zod_file, zod_content)
            .with_context(|| format!("Failed to write schemas file: {}", zod_file.display()))?;
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
        let index_file = module_dir.join("index.ts");
        std::fs::write(&index_file, index_exports.join("\n") + "\n")
            .with_context(|| format!("Failed to write index file: {}", index_file.display()))?;
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
        let functions_content = format!(
            "{}\n",
            functions.iter()
                .map(|f| f.content.clone())
                .collect::<Vec<_>>()
                .join("\n\n")
        );

        let api_file = module_dir.join("index.ts");
        std::fs::write(&api_file, functions_content)
            .with_context(|| format!("Failed to write API file: {}", api_file.display()))?;
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

    std::fs::write(output_path, http_client_content)
        .with_context(|| format!("Failed to write http client template: {}", output_path.display()))?;

    Ok(())
}

