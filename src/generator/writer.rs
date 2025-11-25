use crate::error::{FileSystemError, Result};
use crate::generator::api_client::ApiFunction;
use crate::generator::ts_typings::TypeScriptType;
use crate::generator::utils::sanitize_module_name;
use crate::generator::zod_schema::ZodSchema;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub fn ensure_directory(path: &Path) -> Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path).map_err(|e| FileSystemError::CreateDirectoryFailed {
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
    write_schemas_with_options(
        output_dir,
        module_name,
        types,
        zod_schemas,
        None,
        false,
        false,
    )
}

pub fn write_schemas_with_options(
    output_dir: &Path,
    module_name: &str,
    types: &[TypeScriptType],
    zod_schemas: &[ZodSchema],
    spec_name: Option<&str>,
    backup: bool,
    force: bool,
) -> Result<Vec<PathBuf>> {
    write_schemas_with_module_mapping(
        output_dir,
        module_name,
        types,
        zod_schemas,
        spec_name,
        backup,
        force,
        None, // module_schemas - will be added later if needed
        &[],  // common_schemas
    )
}

pub fn write_schemas_with_module_mapping(
    output_dir: &Path,
    module_name: &str,
    types: &[TypeScriptType],
    zod_schemas: &[ZodSchema],
    spec_name: Option<&str>,
    backup: bool,
    force: bool,
    module_schemas: Option<&std::collections::HashMap<String, Vec<String>>>,
    common_schemas: &[String],
) -> Result<Vec<PathBuf>> {
    // Build module directory path: {output_dir}/{module_name}
    // Note: output_dir already includes spec_name if needed (from config)
    // spec_name is only used for import path calculations, not directory structure
    let module_dir = output_dir.join(sanitize_module_name(module_name));
    ensure_directory(&module_dir)?;

    let mut written_files = Vec::new();

    // Write TypeScript types
    if !types.is_empty() {
        // Deduplicate types by name (to avoid duplicate enum/type declarations)
        // Extract type name from content: "export type XEnum = ..." or "export interface X { ... }"
        let mut seen_type_names = std::collections::HashSet::new();
        let mut deduplicated_types = Vec::new();
        for t in types {
            // Extract type name from content
            let type_name = if let Some(start) = t.content.find("export type ") {
                let after_export_type = &t.content[start + 12..];
                if let Some(end) = after_export_type.find([' ', '=', '\n']) {
                    after_export_type[..end].trim().to_string()
                } else {
                    after_export_type.trim().to_string()
                }
            } else if let Some(start) = t.content.find("export interface ") {
                let after_export_interface = &t.content[start + 17..];
                if let Some(end) = after_export_interface.find([' ', '{', '\n']) {
                    after_export_interface[..end].trim().to_string()
                } else {
                    after_export_interface.trim().to_string()
                }
            } else {
                // Fallback: use full content as key
                t.content.clone()
            };

            if !seen_type_names.contains(&type_name) {
                seen_type_names.insert(type_name);
                deduplicated_types.push(t);
            }
        }

        let types_content_raw = deduplicated_types
            .iter()
            .map(|t| t.content.clone())
            .collect::<Vec<_>>()
            .join("\n\n");

        // Check if we need to import Common types
        // In single-spec mode: schemas/<module>/types.ts -> ../common
        // In multi-spec mode: schemas/<spec_name>/<module>/types.ts -> ../common
        let needs_common_import = types_content_raw.contains("Common.");
        let common_import = if needs_common_import {
            // We're at schemas/{spec_name}/{module}/types.ts (multi-spec) or schemas/{module}/types.ts (single-spec)
            // Common is at schemas/{spec_name}/common (multi-spec) or schemas/common (single-spec)
            // So we go up 1 level (module -> spec_name or module -> schemas), then down to common
            let relative_path = "../";
            format!("import * as Common from \"{}common\";\n\n", relative_path)
        } else {
            String::new()
        };

        let types_content =
            format_typescript_code(&format!("{}{}", common_import, types_content_raw));

        let types_file = module_dir.join("types.ts");
        write_file_with_backup(&types_file, &types_content, backup, force)?;
        written_files.push(types_file);
    }

    // Write Zod schemas
    if !zod_schemas.is_empty() {
        let zod_content_raw = zod_schemas
            .iter()
            .map(|z| z.content.clone())
            .collect::<Vec<_>>()
            .join("\n\n");

        // Check if we need to import Common schemas
        // In single-spec mode: schemas/<module>/schemas.ts -> ../common
        // In multi-spec mode: schemas/<spec_name>/<module>/schemas.ts -> ../common
        let needs_common_import = zod_content_raw.contains("Common.");
        let common_import = if needs_common_import {
            // We're at schemas/{spec_name}/{module}/schemas.ts (multi-spec) or schemas/{module}/schemas.ts (single-spec)
            // Common is at schemas/{spec_name}/common (multi-spec) or schemas/common (single-spec)
            // So we go up 1 level (module -> spec_name or module -> schemas), then down to common
            let relative_path = "../";
            format!("import * as Common from \"{}common\";\n\n", relative_path)
        } else {
            String::new()
        };

        // Detect cross-module enum schema references and add imports
        // This handles cases where a module references an enum from another module
        // (e.g., orders module using CodeEnumSchema from currencies module)
        let mut cross_module_imports: std::collections::HashMap<
            String,
            std::collections::HashSet<String>,
        > = std::collections::HashMap::new();
        if let Some(module_schemas_map) = module_schemas {
            let current_module_schemas: std::collections::HashSet<String> = module_schemas_map
                .get(module_name)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .collect();

            // Check which enums are defined locally in this module's zod_schemas
            let locally_defined_enums: std::collections::HashSet<String> = zod_schemas
                .iter()
                .filter_map(|z| {
                    // Extract enum name from "export const XEnumSchema = z.enum([...])"
                    if let Some(start) = z.content.find("export const ") {
                        let after_export = &z.content[start + 13..];
                        if let Some(end) = after_export.find("EnumSchema") {
                            let enum_name = &after_export[..end + "EnumSchema".len()];
                            if enum_name.ends_with("EnumSchema") {
                                return Some(enum_name.to_string());
                            }
                        }
                    }
                    None
                })
                .collect();

            // Find enum schema references in the content (pattern: XEnumSchema where X is not Common)
            // We'll search for patterns like "CodeEnumSchema", "CountryCodeEnumSchema", etc.
            let mut pos = 0;
            while let Some(start) = zod_content_raw[pos..].find("EnumSchema") {
                let actual_start = pos + start;
                // Find the start of the enum name (go backwards to find word boundary)
                let mut name_start = actual_start;
                while name_start > 0 {
                    let ch = zod_content_raw.chars().nth(name_start - 1).unwrap_or(' ');
                    if !ch.is_alphanumeric() && ch != '_' {
                        break;
                    }
                    name_start -= 1;
                }
                let enum_name = &zod_content_raw[name_start..actual_start + "EnumSchema".len()];

                // Skip if it's Common.EnumSchema (already imported)
                if enum_name.starts_with("Common.") {
                    pos = actual_start + "EnumSchema".len();
                    continue;
                }

                // Skip if this enum is defined locally in this module
                if locally_defined_enums.contains(enum_name) {
                    pos = actual_start + "EnumSchema".len();
                    continue;
                }

                // Extract schema name from enum name (e.g., CodeEnumSchema -> Code)
                let schema_name = enum_name.replace("EnumSchema", "");

                // Check if this enum is not defined locally AND not in common
                // If it's not defined locally but IS in common, we should use Common.EnumSchema instead
                if !locally_defined_enums.contains(enum_name)
                    && !common_schemas.contains(&schema_name)
                {
                    // Find which module defines this schema (and thus exports the enum)
                    // Try exact match first
                    let mut found_module: Option<String> = None;
                    for (other_module, other_schemas) in module_schemas_map {
                        if other_module != module_name && other_schemas.contains(&schema_name) {
                            // Found it! But check if it's not in common schemas
                            // If it's in common, the enum should be imported from common, not this module
                            if !common_schemas.contains(&schema_name) {
                                found_module = Some(other_module.clone());
                                break;
                            }
                        }
                    }

                    // If not found with exact match, try case-insensitive and partial matches
                    // This handles cases where schema names might have different casing or prefixes
                    if found_module.is_none() {
                        let schema_name_lower = schema_name.to_lowercase();
                        for (other_module, other_schemas) in module_schemas_map {
                            if other_module != module_name {
                                // Check if any schema name matches (case-insensitive or contains the enum name)
                                for other_schema in other_schemas {
                                    let other_schema_lower = other_schema.to_lowercase();
                                    // Match if schema name equals enum base name (case-insensitive)
                                    // or if enum name is contained in schema name
                                    if (other_schema_lower == schema_name_lower
                                        || other_schema_lower.contains(&schema_name_lower)
                                        || schema_name_lower.contains(&other_schema_lower))
                                        && !common_schemas.contains(other_schema)
                                    {
                                        found_module = Some(other_module.clone());
                                        break;
                                    }
                                }
                                if found_module.is_some() {
                                    break;
                                }
                            }
                        }
                    }

                    // If we found a module, add the import
                    if let Some(module) = found_module {
                        cross_module_imports
                            .entry(module)
                            .or_insert_with(std::collections::HashSet::new)
                            .insert(enum_name.to_string());
                    }
                    // Note: Disabled heuristic matching as it was too aggressive and caused false imports
                    // If an enum is truly needed from another module, it should be found via exact or fuzzy schema name match
                }

                pos = actual_start + "EnumSchema".len();
            }
        }

        // Build cross-module imports (deduplicated)
        let mut cross_module_import_lines = String::new();
        for (other_module, enum_names_set) in &cross_module_imports {
            let mut enum_names: Vec<String> = enum_names_set.iter().cloned().collect();
            enum_names.sort(); // Sort for consistent output
            if !enum_names.is_empty() {
                let relative_path = "../";
                let module_import = format!(
                    "import {{ {} }} from \"{}{}\";\n",
                    enum_names.join(", "),
                    relative_path,
                    sanitize_module_name(other_module)
                );
                cross_module_import_lines.push_str(&module_import);
            }
        }
        if !cross_module_import_lines.is_empty() {
            cross_module_import_lines.push('\n');
        }

        let zod_content = format_typescript_code(&format!(
            "import {{ z }} from \"zod\";\n{}{}{}",
            if !common_import.is_empty() {
                &common_import
            } else {
                ""
            },
            cross_module_import_lines,
            zod_content_raw
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
    write_api_client_with_options(output_dir, module_name, functions, None, false, false)
}

pub fn write_api_client_with_options(
    output_dir: &Path,
    module_name: &str,
    functions: &[ApiFunction],
    spec_name: Option<&str>,
    backup: bool,
    force: bool,
) -> Result<Vec<PathBuf>> {
    // Build module directory path: {output_dir}/{module_name}
    // Note: output_dir already includes spec_name if needed (from config)
    // spec_name is only used for import path calculations, not directory structure
    let module_dir = output_dir.join(sanitize_module_name(module_name));
    ensure_directory(&module_dir)?;

    let mut written_files = Vec::new();

    if !functions.is_empty() {
        // Consolidate imports: extract all imports and merge by module
        // Map: module_path -> (type_imports_set, other_imports_set)
        // We need to separate type imports from other imports to reconstruct them correctly
        let mut imports_by_module: std::collections::HashMap<
            String,
            (std::collections::HashSet<String>, Vec<String>),
        > = std::collections::HashMap::new();
        let mut function_bodies = Vec::new();
        let mut seen_functions: std::collections::HashSet<String> =
            std::collections::HashSet::new();

        for func in functions {
            let lines: Vec<&str> = func.content.lines().collect();
            let mut func_lines = Vec::new();
            let mut in_function = false;
            let mut jsdoc_lines = Vec::new();
            let mut in_jsdoc = false;
            let mut function_name: Option<String> = None;

            for line in lines {
                if line.trim().starts_with("import ") {
                    let import_line = line.trim().trim_end_matches(';').trim();
                    // Parse import statement: "import type { A, B } from 'path'" or "import * as X from 'path'"
                    if let Some(from_pos) = import_line.find(" from ") {
                        let before_from = &import_line[..from_pos];
                        let after_from = &import_line[from_pos + 6..];
                        let module_path = after_from.trim_matches('"').trim_matches('\'').trim();

                        // Extract imported items
                        if before_from.contains("import type {") {
                            // Type import: "import type { A, B }"
                            if let Some(start) = before_from.find('{') {
                                if let Some(end) = before_from.find('}') {
                                    let items_str = &before_from[start + 1..end];
                                    let items: Vec<String> = items_str
                                        .split(',')
                                        .map(|s| s.trim().to_string())
                                        .filter(|s| !s.is_empty())
                                        .collect();

                                    let (type_imports, _) = imports_by_module
                                        .entry(module_path.to_string())
                                        .or_insert_with(|| {
                                            (std::collections::HashSet::new(), Vec::new())
                                        });
                                    type_imports.extend(items);
                                }
                            }
                        } else if before_from.contains("import * as ") {
                            // Namespace import: "import * as X"
                            // Keep as-is, don't merge
                            let (_, other_imports) = imports_by_module
                                .entry(module_path.to_string())
                                .or_insert_with(|| (std::collections::HashSet::new(), Vec::new()));
                            other_imports.push(import_line.to_string());
                        } else {
                            // Default import or other format (e.g., "import { http }")
                            // Keep as-is
                            let (_, other_imports) = imports_by_module
                                .entry(module_path.to_string())
                                .or_insert_with(|| (std::collections::HashSet::new(), Vec::new()));
                            other_imports.push(import_line.to_string());
                        }
                    } else {
                        // Malformed import - keep as-is
                        let (_, other_imports) = imports_by_module
                            .entry("".to_string())
                            .or_insert_with(|| (std::collections::HashSet::new(), Vec::new()));
                        other_imports.push(import_line.to_string());
                    }
                } else if line.trim().starts_with("/**") {
                    // Start of JSDoc comment
                    in_jsdoc = true;
                    jsdoc_lines.push(line);
                } else if in_jsdoc {
                    jsdoc_lines.push(line);
                    if line.trim().ends_with("*/") {
                        // End of JSDoc comment
                        in_jsdoc = false;
                    }
                } else if line.trim().starts_with("export const ") {
                    // Extract function name to check for duplicates
                    // Find the function name after "export const " (13 chars)
                    let trimmed = line.trim();
                    if trimmed.len() > 13 {
                        let after_export_const = &trimmed[13..];
                        // Find the first space or opening parenthesis after function name
                        let name_end = after_export_const
                            .find(' ')
                            .or_else(|| after_export_const.find('('))
                            .unwrap_or(after_export_const.len());
                        let name = after_export_const[..name_end].trim().to_string();
                        if !name.is_empty() {
                            function_name = Some(name.clone());
                            if seen_functions.contains(&name) {
                                // Skip duplicate function
                                jsdoc_lines.clear();
                                break;
                            }
                            seen_functions.insert(name);
                        }
                    }
                    in_function = true;
                    // Add JSDoc comments before the function
                    func_lines.append(&mut jsdoc_lines);
                    func_lines.push(line);
                } else if in_function {
                    func_lines.push(line);
                    // Check if function ends
                    if line.trim() == "};" {
                        break;
                    }
                }
                // Skip type definitions - they're in types.ts now
            }

            if !func_lines.is_empty() && function_name.is_some() {
                function_bodies.push(func_lines.join("\n"));
            }
        }

        // Combine imports and function bodies (no type definitions)
        // Merge imports by module path
        let mut imports_vec = Vec::new();
        for (module_path, (type_import_items, other_imports)) in imports_by_module.iter() {
            if module_path.is_empty() {
                // Malformed imports - add as-is (deduplicate)
                let deduped: std::collections::HashSet<String> =
                    other_imports.iter().cloned().collect();
                imports_vec.extend(deduped.into_iter());
            } else {
                // Deduplicate and separate other imports by type
                let deduped_imports: std::collections::HashSet<String> =
                    other_imports.iter().cloned().collect();
                let mut namespace_imports = Vec::new();
                let mut default_imports = Vec::new();

                for item in deduped_imports.iter() {
                    if item.contains("import * as") {
                        // Namespace import - keep as-is
                        namespace_imports.push(item.clone());
                    } else {
                        // Default import (e.g., "import { http }")
                        default_imports.push(item.clone());
                    }
                }

                // Add namespace imports (sorted for consistency)
                namespace_imports.sort();
                for ns_import in namespace_imports {
                    imports_vec.push(format!("{};", ns_import));
                }

                // Add default imports (sorted for consistency)
                default_imports.sort();
                for default_import in default_imports {
                    imports_vec.push(format!("{};", default_import));
                }

                // Merge and add type imports
                if !type_import_items.is_empty() {
                    let mut sorted_types: Vec<String> = type_import_items.iter().cloned().collect();
                    sorted_types.sort();
                    imports_vec.push(format!(
                        "import type {{ {} }} from \"{}\";",
                        sorted_types.join(", "),
                        module_path
                    ));
                }
            }
        }
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

    let http_client_content = r#"const requestInitIndicators = [
  "method",
  "headers",
  "body",
  "signal",
  "credentials",
  "cache",
  "redirect",
  "referrer",
  "referrerPolicy",
  "integrity",
  "keepalive",
  "mode",
  "priority",
  "window",
];

const isRequestInitLike = (value: unknown): value is RequestInit => {
  if (!value || typeof value !== "object") {
    return false;
  }
  const candidate = value as Record<string, unknown>;
  return requestInitIndicators.some((key) => key in candidate);
};

export const http = {
  // GET helper. Second argument can be either a RequestInit or a JSON body for uncommon GET-with-body endpoints.
  async get<T = any>(url: string, optionsOrBody?: RequestInit | unknown): Promise<T> {
    let init: RequestInit = { method: "GET", body: null };

    if (optionsOrBody !== undefined && optionsOrBody !== null) {
      if (isRequestInitLike(optionsOrBody)) {
        const candidate = optionsOrBody as RequestInit;
        init = {
          ...candidate,
          method: "GET",
          body: candidate.body ?? null,
        };
      } else {
        init = {
          method: "GET",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify(optionsOrBody),
        };
      }
    }

    const response = await fetch(url, {
      ...init,
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
      body: body !== undefined ? JSON.stringify(body) : (options.body ?? null),
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
      body: body !== undefined ? JSON.stringify(body) : (options.body ?? null),
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
      body: options.body ?? null,
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
      body: body !== undefined ? JSON.stringify(body) : (options.body ?? null),
    });
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    return response.json();
  },

  async head(url: string, options: RequestInit = {}): Promise<Response> {
    const response = await fetch(url, {
      ...options,
      method: "HEAD",
      body: options.body ?? null,
    });
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    return response;
  },

  async options<T = any>(url: string, options: RequestInit = {}): Promise<T> {
    const response = await fetch(url, {
      ...options,
      method: "OPTIONS",
      body: options.body ?? null,
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
    // Basic formatting: remove extra blank lines while preserving indentation
    let lines: Vec<&str> = code.lines().collect();
    let mut formatted = Vec::new();
    let mut last_was_empty = false;

    for line in lines {
        if line.trim().is_empty() {
            if !last_was_empty && !formatted.is_empty() {
                formatted.push(String::new());
                last_was_empty = true;
            }
            continue;
        }
        last_was_empty = false;
        formatted.push(line.to_string());
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

pub fn write_file_with_backup(path: &Path, content: &str, backup: bool, force: bool) -> Result<()> {
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

            // If metadata hash doesn't match current or file hash, check if it's just formatting
            if metadata.hash != current_hash && metadata.hash != file_hash {
                // Try to detect formatter by walking up the directory tree
                // This handles the case where file was formatted but spec didn't change
                use crate::formatter::FormatterManager;

                // Find formatter by checking parent directories (where config files are likely located)
                let mut search_dir = path.parent().unwrap_or_else(|| Path::new("."));
                let mut formatter = None;

                // Walk up the directory tree to find formatter config
                while search_dir != Path::new("/") && search_dir != Path::new("") {
                    if let Some(fmt) = FormatterManager::detect_formatter_from_dir(search_dir) {
                        formatter = Some(fmt);
                        break;
                    }
                    if let Some(parent) = search_dir.parent() {
                        search_dir = parent;
                    } else {
                        break;
                    }
                }

                // Also try current directory as fallback
                if formatter.is_none() {
                    formatter = FormatterManager::detect_formatter();
                }

                if let Some(fmt) = formatter {
                    // Format the new content and compare with file
                    match FormatterManager::format_content(content, fmt, path) {
                        Ok(formatted_content) => {
                            let formatted_hash = compute_content_hash(&formatted_content);
                            if formatted_hash == file_hash {
                                // File matches formatted version of new content - it's just formatting, allow overwrite
                                // Continue to write the file
                            } else {
                                // File doesn't match formatted new content
                                // Check if spec changed - if so, differences are expected
                                if current_hash == metadata.hash {
                                    // Spec didn't change, so file should match formatted version if it's just formatting
                                    // Since it doesn't match, it's likely a user modification
                                    return Err(FileSystemError::FileModifiedByUser {
                                        path: path.display().to_string(),
                                    }
                                    .into());
                                }
                                // Spec changed - file differences are expected, allow overwrite
                                // (formatted new content won't match formatted old content when spec changes)
                            }
                        }
                        Err(_) => {
                            // Formatting failed - check if spec changed
                            if current_hash == metadata.hash {
                                // Spec didn't change but formatting failed - can't verify
                                // Since metadata update after formatting should handle this, allow overwrite
                            }
                            // If spec changed, allow overwrite (differences are expected)
                        }
                    }
                } else {
                    // No formatter detected - check if spec changed
                    if current_hash == metadata.hash {
                        // Spec didn't change, but file_hash != metadata.hash
                        // This likely means file was formatted, but we can't verify without formatter
                        // Since metadata update after formatting should handle this, allow overwrite
                    }
                    // If spec changed, allow overwrite (differences are expected)
                }
            }
        }
    }

    // Write the file
    std::fs::write(path, content).map_err(|e| FileSystemError::WriteFileFailed {
        path: path.display().to_string(),
        source: e,
    })?;

    // Save metadata
    save_file_metadata(path, content)?;

    Ok(())
}

fn create_backup(path: &Path) -> Result<()> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::{SystemTime, UNIX_EPOCH};

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let backup_dir = PathBuf::from(format!(".vika-backup/{}", timestamp));
    std::fs::create_dir_all(&backup_dir).map_err(|e| FileSystemError::CreateDirectoryFailed {
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
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("file");
        backup_dir.join(format!("{}_{}", hash, filename))
    } else {
        // For relative paths, preserve directory structure
        let relative_path = path.strip_prefix(".").unwrap_or(path);
        backup_dir.join(relative_path)
    };

    if let Some(parent) = backup_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| FileSystemError::CreateDirectoryFailed {
            path: parent.display().to_string(),
            source: e,
        })?;
    }

    std::fs::copy(path, &backup_path).map_err(|e| FileSystemError::WriteFileFailed {
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
    let content = std::fs::read_to_string(path).map_err(|e| FileSystemError::ReadFileFailed {
        path: path.display().to_string(),
        source: e,
    })?;
    Ok(compute_content_hash(&content))
}

/// Update metadata for a file from its current content on disk
/// Useful after formatting files to update metadata hash
pub fn update_file_metadata_from_disk(path: &Path) -> Result<()> {
    let content = std::fs::read_to_string(path).map_err(|e| FileSystemError::ReadFileFailed {
        path: path.display().to_string(),
        source: e,
    })?;
    save_file_metadata(path, &content)
}

/// Batch update metadata for multiple files from disk
/// Much more efficient than calling update_file_metadata_from_disk for each file
/// Reads metadata JSON once, updates all files, writes once
pub fn batch_update_file_metadata_from_disk(paths: &[PathBuf]) -> Result<()> {
    if paths.is_empty() {
        return Ok(());
    }

    let metadata_dir = PathBuf::from(".vika-cache");
    std::fs::create_dir_all(&metadata_dir).map_err(|e| FileSystemError::CreateDirectoryFailed {
        path: metadata_dir.display().to_string(),
        source: e,
    })?;

    let metadata_file = metadata_dir.join("file-metadata.json");
    let mut metadata_map: std::collections::HashMap<String, FileMetadata> =
        if metadata_file.exists() {
            let content = std::fs::read_to_string(&metadata_file).map_err(|e| {
                FileSystemError::ReadFileFailed {
                    path: metadata_file.display().to_string(),
                    source: e,
                }
            })?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            std::collections::HashMap::new()
        };

    let generated_at = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Update metadata for all files in batch
    for path in paths {
        match std::fs::read_to_string(path) {
            Ok(content) => {
                let hash = compute_content_hash(&content);
                metadata_map.insert(
                    path.display().to_string(),
                    FileMetadata {
                        hash,
                        generated_at,
                        generated_by: "vika-cli".to_string(),
                    },
                );
            }
            Err(e) => {
                // Log but continue with other files
                eprintln!("Warning: Failed to read {}: {}", path.display(), e);
            }
        }
    }

    // Write updated metadata once
    let json = serde_json::to_string_pretty(&metadata_map).map_err(|e| {
        FileSystemError::WriteFileFailed {
            path: metadata_file.display().to_string(),
            source: std::io::Error::new(std::io::ErrorKind::InvalidData, format!("{}", e)),
        }
    })?;

    std::fs::write(&metadata_file, json).map_err(|e| FileSystemError::WriteFileFailed {
        path: metadata_file.display().to_string(),
        source: e,
    })?;

    Ok(())
}

pub fn save_file_metadata(path: &Path, content: &str) -> Result<()> {
    let metadata_dir = PathBuf::from(".vika-cache");
    std::fs::create_dir_all(&metadata_dir).map_err(|e| FileSystemError::CreateDirectoryFailed {
        path: metadata_dir.display().to_string(),
        source: e,
    })?;

    let metadata_file = metadata_dir.join("file-metadata.json");
    let mut metadata_map: std::collections::HashMap<String, FileMetadata> =
        if metadata_file.exists() {
            let content = std::fs::read_to_string(&metadata_file).map_err(|e| {
                FileSystemError::ReadFileFailed {
                    path: metadata_file.display().to_string(),
                    source: e,
                }
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

    let json = serde_json::to_string_pretty(&metadata_map).map_err(|e| {
        FileSystemError::WriteFileFailed {
            path: metadata_file.display().to_string(),
            source: std::io::Error::new(std::io::ErrorKind::InvalidData, format!("{}", e)),
        }
    })?;

    std::fs::write(&metadata_file, json).map_err(|e| FileSystemError::WriteFileFailed {
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
        }
        .into());
    }

    let content =
        std::fs::read_to_string(&metadata_file).map_err(|e| FileSystemError::ReadFileFailed {
            path: metadata_file.display().to_string(),
            source: e,
        })?;

    let metadata_map: std::collections::HashMap<String, FileMetadata> =
        serde_json::from_str(&content).map_err(|e| FileSystemError::ReadFileFailed {
            path: metadata_file.display().to_string(),
            source: std::io::Error::new(std::io::ErrorKind::InvalidData, format!("{}", e)),
        })?;

    metadata_map
        .get(&path.display().to_string())
        .cloned()
        .ok_or_else(|| {
            FileSystemError::FileNotFound {
                path: path.display().to_string(),
            }
            .into()
        })
}
