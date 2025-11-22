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
    module: Option<String>,
    schemas: bool,
    _graph: bool,
    json: bool,
) -> Result<()> {
    use crate::error::GenerationError;

    let spec_path = spec.ok_or(GenerationError::SpecPathRequired)?;

    println!("{}", "🔍 Inspecting OpenAPI spec...".bright_cyan());
    println!();

    let parsed = fetch_and_parse_spec(&spec_path).await?;

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
