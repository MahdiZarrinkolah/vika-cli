use crate::error::Result;
use crate::generator::swagger_parser::fetch_and_parse_spec;
use colored::*;
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct ModuleInfo {
    #[tabled(rename = "Module")]
    module: String,
    #[tabled(rename = "Endpoints")]
    endpoints: usize,
    #[tabled(rename = "Schemas")]
    schemas: usize,
}

pub async fn run(
    spec: Option<String>,
    all_specs: bool,
    spec_name: Option<String>,
    module: Option<String>,
    schemas: bool,
    _graph: bool,
    json: bool,
) -> Result<()> {
    use crate::error::GenerationError;
    use crate::specs::manager::{list_specs, get_spec_by_name};

    // Load config
    let config = crate::config::loader::load_config()?;
    crate::config::validator::validate_config(&config)?;

    // Get specs from config
    let specs = list_specs(&config);
    
    if specs.is_empty() {
        return Err(GenerationError::SpecPathRequired.into());
    }

    // Handle multiple specs or single spec
    if specs.len() > 1 || all_specs {
        if all_specs {
            // Inspect all specs

            if json {
                // JSON output for all specs
                let mut all_specs_data = Vec::new();
                for spec_entry in &specs {
                    let parsed = crate::generator::swagger_parser::fetch_and_parse_spec(&spec_entry.path).await?;
                    all_specs_data.push(serde_json::json!({
                        "spec_name": spec_entry.name,
                        "spec_path": spec_entry.path,
                        "modules": parsed.modules.len(),
                        "total_endpoints": parsed.operations_by_tag.values().map(|v| v.len()).sum::<usize>(),
                        "total_schemas": parsed.schemas.len(),
                        "modules_detail": parsed.modules.iter().map(|m| {
                            let ops = parsed.operations_by_tag.get(m).map(|v| v.len()).unwrap_or(0);
                            let schemas_count = parsed.module_schemas.get(m).map(|v| v.len()).unwrap_or(0);
                            serde_json::json!({
                                "module": m,
                                "endpoints": ops,
                                "schemas": schemas_count
                            })
                        }).collect::<Vec<_>>()
                    }));
                }
                println!("{}", serde_json::to_string_pretty(&serde_json::json!({
                    "specs": all_specs_data
                })).map_err(|e| {
                    GenerationError::InvalidOperation {
                        message: format!("Failed to serialize JSON: {}", e),
                    }
                })?);
            } else {
                // Human-readable output for all specs
                println!("{}", "🔍 Inspecting all OpenAPI specs...".bright_cyan());
                println!();

                for spec_entry in &specs {
                    println!("{}", format!("📋 Spec: {}", spec_entry.name).bright_green());
                    println!("  Path: {}", spec_entry.path);
                    let parsed = crate::generator::swagger_parser::fetch_and_parse_spec(&spec_entry.path).await?;
                    println!("  • Total modules: {}", parsed.modules.len());
                    println!(
                        "  • Total endpoints: {}",
                        parsed.operations_by_tag.values().map(|v| v.len()).sum::<usize>()
                    );
                    println!("  • Total schemas: {}", parsed.schemas.len());
                    println!();
                }
            }
            return Ok(());
        } else if let Some(name) = spec_name {
            // Inspect specific spec by name
            let spec_entry = get_spec_by_name(&config, &name)?;
            let spec_path = spec_entry.path;
            let parsed = crate::generator::swagger_parser::fetch_and_parse_spec(&spec_path).await?;

            if json {
                let output = serde_json::json!({
                    "spec_name": name,
                    "spec_path": spec_path,
                    "modules": parsed.modules.len(),
                    "total_endpoints": parsed.operations_by_tag.values().map(|v| v.len()).sum::<usize>(),
                    "total_schemas": parsed.schemas.len(),
                    "modules_detail": parsed.modules.iter().map(|m| {
                        let ops = parsed.operations_by_tag.get(m).map(|v| v.len()).unwrap_or(0);
                        let schemas_count = parsed.module_schemas.get(m).map(|v| v.len()).unwrap_or(0);
                        serde_json::json!({
                            "module": m,
                            "endpoints": ops,
                            "schemas": schemas_count
                        })
                    }).collect::<Vec<_>>()
                });
                println!("{}", serde_json::to_string_pretty(&output).map_err(|e| {
                    GenerationError::InvalidOperation {
                        message: format!("Failed to serialize JSON: {}", e),
                    }
                })?);
            } else {
                println!("{}", format!("🔍 Inspecting OpenAPI spec: {}", name).bright_cyan());
                println!("  Path: {}", spec_path);
                println!();
                println!("{}", "📊 Spec Summary:".bright_cyan());
                println!("  • Total modules: {}", parsed.modules.len());
                println!(
                    "  • Total endpoints: {}",
                    parsed.operations_by_tag.values().map(|v| v.len()).sum::<usize>()
                );
                println!("  • Total schemas: {}", parsed.schemas.len());
                println!();

                if let Some(module_name) = module {
                    // Show details for specific module
                    if let Some(operations) = parsed.operations_by_tag.get(&module_name) {
                        println!("{}", format!("📦 Module: {}", module_name).bright_green());
                        println!("  • Endpoints: {}", operations.len());
                        if let Some(schema_names) = parsed.module_schemas.get(&module_name) {
                            println!("  • Schemas: {}", schema_names.len());
                            if schemas {
                                println!("  • Schema names:");
                                for schema in schema_names {
                                    println!("    - {}", schema);
                                }
                            }
                        }
                    } else {
                        println!(
                            "{}",
                            format!("⚠️  Module '{}' not found", module_name).yellow()
                        );
                    }
                } else {
                    // Show all modules
                    let table_data: Vec<ModuleInfo> = parsed
                        .modules
                        .iter()
                        .map(|m| {
                            let endpoints = parsed
                                .operations_by_tag
                                .get(m)
                                .map(|v| v.len())
                                .unwrap_or(0);
                            let schemas_count = parsed.module_schemas.get(m).map(|v| v.len()).unwrap_or(0);
                            ModuleInfo {
                                module: m.clone(),
                                endpoints,
                                schemas: schemas_count,
                            }
                        })
                        .collect();

                    let table = Table::new(table_data);
                    println!("{}", "📦 Modules:".bright_cyan());
                    println!("{}", table);
                }
            }
            return Ok(());
        } else {
            // Multi-spec mode but no flag: prompt user or show all
            let specs = list_specs(&config);
            if specs.is_empty() {
                return Err(GenerationError::SpecPathRequired.into());
            }

            // Default to showing all specs summary
            println!("{}", "🔍 Inspecting all OpenAPI specs...".bright_cyan());
            println!();

            for spec_entry in &specs {
                println!("{}", format!("📋 Spec: {}", spec_entry.name).bright_green());
                println!("  Path: {}", spec_entry.path);
                let parsed = crate::generator::swagger_parser::fetch_and_parse_spec(&spec_entry.path).await?;
                println!("  • Total modules: {}", parsed.modules.len());
                println!(
                    "  • Total endpoints: {}",
                    parsed.operations_by_tag.values().map(|v| v.len()).sum::<usize>()
                );
                println!("  • Total schemas: {}", parsed.schemas.len());
                println!();
            }
            return Ok(());
        }
    }

    // Single spec mode - use first spec
    let spec_entry = if let Some(name) = spec_name {
        get_spec_by_name(&config, &name)?
    } else if let Some(cli_spec) = spec {
        // CLI spec argument - try to find by name or use as path
        get_spec_by_name(&config, &cli_spec).unwrap_or_else(|_| {
            // If not found by name, treat as path and find matching spec
            specs.iter()
                .find(|s| s.path == cli_spec)
                .cloned()
                .ok_or_else(|| GenerationError::SpecPathRequired)
                .unwrap()
        })
    } else {
        // Use first spec
        specs[0].clone()
    };

    let spec_path = &spec_entry.path;
    println!("{}", format!("🔍 Inspecting OpenAPI spec: {}", spec_entry.name).bright_cyan());
    println!();

    let parsed = fetch_and_parse_spec(spec_path).await?;

    if json {
        // JSON output
        let output = serde_json::json!({
            "modules": parsed.modules.len(),
            "total_endpoints": parsed.operations_by_tag.values().map(|v| v.len()).sum::<usize>(),
            "total_schemas": parsed.schemas.len(),
            "modules_detail": parsed.modules.iter().map(|m| {
                let ops = parsed.operations_by_tag.get(m).map(|v| v.len()).unwrap_or(0);
                let schemas_count = parsed.module_schemas.get(m).map(|v| v.len()).unwrap_or(0);
                serde_json::json!({
                    "module": m,
                    "endpoints": ops,
                    "schemas": schemas_count
                })
            }).collect::<Vec<_>>()
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&output).map_err(|e| {
                crate::error::GenerationError::InvalidOperation {
                    message: format!("Failed to serialize JSON: {}", e),
                }
            })?
        );
    } else {
        // Human-readable output
        println!("{}", "📊 Spec Summary:".bright_cyan());
        println!("  • Total modules: {}", parsed.modules.len());
        println!(
            "  • Total endpoints: {}",
            parsed
                .operations_by_tag
                .values()
                .map(|v| v.len())
                .sum::<usize>()
        );
        println!("  • Total schemas: {}", parsed.schemas.len());
        println!();

        if let Some(module_name) = module {
            // Show details for specific module
            if let Some(operations) = parsed.operations_by_tag.get(&module_name) {
                println!("{}", format!("📦 Module: {}", module_name).bright_green());
                println!("  • Endpoints: {}", operations.len());
                if let Some(schema_names) = parsed.module_schemas.get(&module_name) {
                    println!("  • Schemas: {}", schema_names.len());
                    if schemas {
                        println!("  • Schema names:");
                        for schema in schema_names {
                            println!("    - {}", schema);
                        }
                    }
                }
            } else {
                println!(
                    "{}",
                    format!("⚠️  Module '{}' not found", module_name).yellow()
                );
            }
        } else {
            // Show all modules
            let table_data: Vec<ModuleInfo> = parsed
                .modules
                .iter()
                .map(|m| {
                    let endpoints = parsed
                        .operations_by_tag
                        .get(m)
                        .map(|v| v.len())
                        .unwrap_or(0);
                    let schemas_count = parsed.module_schemas.get(m).map(|v| v.len()).unwrap_or(0);
                    ModuleInfo {
                        module: m.clone(),
                        endpoints,
                        schemas: schemas_count,
                    }
                })
                .collect();

            let table = Table::new(table_data);
            println!("{}", "📦 Modules:".bright_cyan());
            println!("{}", table);
        }
    }

    Ok(())
}
