use serde::{Deserialize, Serialize};

/// Represents a single OpenAPI specification entry in multi-spec mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecEntry {
    /// Unique name for this spec (kebab-case recommended)
    pub name: String,
    /// Path or URL to the OpenAPI specification file
    pub path: String,

    /// Required per-spec schema output directory and naming configuration
    pub schemas: SchemasConfig,

    /// Required per-spec API output directory and configuration
    pub apis: ApisConfig,

    /// Optional per-spec hooks output directory configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hooks: Option<HooksConfig>,

    /// Required per-spec module selection configuration
    pub modules: ModulesConfig,
}

/// Main configuration structure for vika-cli.
///
/// Represents the `.vika.json` configuration file that controls
/// code generation behavior, output directories, and module selection.
///
/// # Example
///
/// ```no_run
/// use vika_cli::Config;
///
/// let config = Config::default();
/// println!("Root directory: {}", config.root_dir);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "$schema", default = "default_schema")]
    pub schema: String,

    #[serde(default = "default_root_dir")]
    pub root_dir: String,

    #[serde(default)]
    pub generation: GenerationConfig,

    /// Specs configuration - always use array, even for single spec
    #[serde(default)]
    pub specs: Vec<SpecEntry>,
}

pub fn default_schema() -> String {
    "https://raw.githubusercontent.com/vikarno/vika-cli/main/schema/vika-config.schema.json"
        .to_string()
}

fn default_root_dir() -> String {
    "src".to_string()
}

/// Configuration for schema generation (TypeScript types and Zod schemas).
///
/// Controls where schemas are generated and how they are named.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemasConfig {
    #[serde(default = "default_schemas_output")]
    pub output: String,

    #[serde(default = "default_naming")]
    pub naming: String,
}

fn default_naming() -> String {
    "PascalCase".to_string()
}

fn default_schemas_output() -> String {
    "src/schemas".to_string()
}

/// Configuration for API client generation.
///
/// Controls API client output location, style, base URL, header strategy, and runtime client options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApisConfig {
    #[serde(default = "default_apis_output")]
    pub output: String,

    #[serde(default = "default_style")]
    pub style: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,

    #[serde(default = "default_header_strategy")]
    pub header_strategy: String,

    /// Runtime client timeout in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u32>,

    /// Number of retries for failed requests
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retries: Option<u32>,

    /// Delay between retries in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_delay: Option<u32>,

    /// Default headers to include in all requests
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<std::collections::HashMap<String, String>>,
}

fn default_header_strategy() -> String {
    "consumerInjected".to_string()
}

fn default_apis_output() -> String {
    "src/apis".to_string()
}

fn default_style() -> String {
    "fetch".to_string()
}

/// Configuration for hooks generation (React Query, SWR, etc.).
///
/// Controls where hooks and query keys are generated, and which hook library to use.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HooksConfig {
    /// Output directory for hooks files (e.g., "src/hooks")
    #[serde(default = "default_hooks_output")]
    pub output: String,

    /// Output directory for query keys files (e.g., "src/query-keys")
    #[serde(default = "default_query_keys_output")]
    pub query_keys_output: String,

    /// Hook library to use for generation
    /// Options: "react-query" or "swr"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub library: Option<String>,
}

pub fn default_hooks_output() -> String {
    "src/hooks".to_string()
}

pub fn default_query_keys_output() -> String {
    "src/query-keys".to_string()
}

/// Configuration for module selection and filtering.
///
/// Controls which OpenAPI tags/modules are included or excluded from generation.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModulesConfig {
    #[serde(default)]
    pub ignore: Vec<String>,

    #[serde(default)]
    pub selected: Vec<String>,
}

/// Configuration for generation behavior and preferences.
///
/// Controls caching, backups, and conflict resolution strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationConfig {
    #[serde(default = "default_enable_cache")]
    pub enable_cache: bool,

    #[serde(default = "default_enable_backup")]
    pub enable_backup: bool,

    #[serde(default = "default_conflict_strategy")]
    pub conflict_strategy: String,
}

fn default_enable_cache() -> bool {
    true
}

fn default_enable_backup() -> bool {
    false
}

fn default_conflict_strategy() -> String {
    "ask".to_string()
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            enable_cache: default_enable_cache(),
            enable_backup: default_enable_backup(),
            conflict_strategy: default_conflict_strategy(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            schema: default_schema(),
            root_dir: default_root_dir(),
            generation: GenerationConfig::default(),
            specs: vec![],
        }
    }
}

impl Default for SchemasConfig {
    fn default() -> Self {
        Self {
            output: default_schemas_output(),
            naming: default_naming(),
        }
    }
}

impl Default for ApisConfig {
    fn default() -> Self {
        Self {
            output: default_apis_output(),
            style: default_style(),
            base_url: None,
            header_strategy: default_header_strategy(),
            timeout: None,
            retries: None,
            retry_delay: None,
            headers: None,
        }
    }
}

impl Default for HooksConfig {
    fn default() -> Self {
        Self {
            output: default_hooks_output(),
            query_keys_output: default_query_keys_output(),
            library: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_default_config() {
        let config = Config::default();
        assert_eq!(config.root_dir, "src");
        assert!(!config.schema.is_empty());
        assert_eq!(config.specs.len(), 0);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let json = serde_json::to_string_pretty(&config).unwrap();

        assert!(json.contains("\"root_dir\""));
        assert!(json.contains("\"specs\""));
        assert!(json.contains("\"$schema\""));
    }

    #[test]
    fn test_config_deserialization() {
        let json = r#"
        {
            "$schema": "https://example.com/schema.json",
            "root_dir": "test",
            "specs": [
                {
                    "name": "test-spec",
                    "path": "test.yaml",
                    "schemas": {
                        "output": "test/schemas",
                        "naming": "camelCase"
                    },
                    "apis": {
                        "output": "test/apis",
                        "style": "fetch",
                        "header_strategy": "bearerToken"
                    },
                    "modules": {
                        "ignore": ["test"],
                        "selected": []
                    }
                }
            ]
        }
        "#;

        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.root_dir, "test");
        assert_eq!(config.specs.len(), 1);
        assert_eq!(config.specs[0].schemas.output, "test/schemas");
        assert_eq!(config.specs[0].schemas.naming, "camelCase");
        assert_eq!(config.specs[0].apis.header_strategy, "bearerToken");
        assert_eq!(config.specs[0].modules.ignore, vec!["test"]);
    }

    #[test]
    fn test_schemas_config_default() {
        let config = SchemasConfig::default();
        assert_eq!(config.output, "src/schemas");
        assert_eq!(config.naming, "PascalCase");
    }

    #[test]
    fn test_apis_config_default() {
        let config = ApisConfig::default();
        assert_eq!(config.output, "src/apis");
        assert_eq!(config.style, "fetch");
        assert_eq!(config.header_strategy, "consumerInjected");
        assert!(config.base_url.is_none());
    }

    #[test]
    fn test_config_with_base_url() {
        let mut config = Config::default();
        config.specs.push(SpecEntry {
            name: "test".to_string(),
            path: "test.yaml".to_string(),
            schemas: SchemasConfig::default(),
            apis: ApisConfig {
                base_url: Some("/api/v1".to_string()),
                ..ApisConfig::default()
            },
            hooks: None,
            modules: ModulesConfig::default(),
        });

        let json = serde_json::to_string_pretty(&config).unwrap();
        assert!(json.contains("\"base_url\""));
        assert!(json.contains("/api/v1"));
    }

    #[test]
    fn test_config_schema_field() {
        let config = Config::default();
        let json = serde_json::to_string_pretty(&config).unwrap();

        // Check that $schema is included
        assert!(json.contains("\"$schema\""));
    }

    #[test]
    fn test_generation_config_defaults() {
        let config = Config::default();
        assert!(config.generation.enable_cache);
        assert!(!config.generation.enable_backup);
        assert_eq!(config.generation.conflict_strategy, "ask");
    }

    #[test]
    fn test_config_with_generation_settings() {
        let json = r#"
        {
            "$schema": "https://example.com/schema.json",
            "root_dir": "test",
            "schemas": {
                "output": "test/schemas",
                "naming": "camelCase"
            },
            "apis": {
                "output": "test/apis",
                "style": "fetch",
                "header_strategy": "bearerToken"
            },
            "modules": {
                "ignore": ["test"],
                "selected": []
            },
            "generation": {
                "enable_cache": false,
                "enable_backup": true,
                "conflict_strategy": "force"
            }
        }
        "#;

        let config: Config = serde_json::from_str(json).unwrap();
        assert!(!config.generation.enable_cache);
        assert!(config.generation.enable_backup);
        assert_eq!(config.generation.conflict_strategy, "force");
    }

    #[test]
    fn test_multi_spec_deserialization() {
        let json = r#"
        {
            "$schema": "https://example.com/schema.json",
            "specs": [
                { 
                    "name": "auth", 
                    "path": "specs/auth.yaml",
                    "schemas": {},
                    "apis": {},
                    "modules": {}
                },
                { 
                    "name": "orders", 
                    "path": "specs/orders.json",
                    "schemas": {},
                    "apis": {},
                    "modules": {}
                },
                { 
                    "name": "products", 
                    "path": "specs/products.yaml",
                    "schemas": {},
                    "apis": {},
                    "modules": {}
                }
            ]
        }
        "#;

        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.specs.len(), 3);
        assert_eq!(config.specs[0].name, "auth");
        assert_eq!(config.specs[0].path, "specs/auth.yaml");
        assert_eq!(config.specs[1].name, "orders");
        assert_eq!(config.specs[1].path, "specs/orders.json");
        assert_eq!(config.specs[2].name, "products");
        assert_eq!(config.specs[2].path, "specs/products.yaml");
    }

    #[test]
    fn test_spec_entry_serialization() {
        let entry = SpecEntry {
            name: "test-spec".to_string(),
            path: "specs/test.yaml".to_string(),
            schemas: SchemasConfig::default(),
            apis: ApisConfig::default(),
            hooks: None,
            modules: ModulesConfig::default(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"name\""));
        assert!(json.contains("\"path\""));
        assert!(json.contains("test-spec"));
        assert!(json.contains("specs/test.yaml"));
    }
}
