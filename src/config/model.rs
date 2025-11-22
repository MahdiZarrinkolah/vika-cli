use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "$schema", default = "default_schema")]
    pub schema: String,
    
    #[serde(default = "default_root_dir")]
    pub root_dir: String,
    
    #[serde(default)]
    pub schemas: SchemasConfig,
    
    #[serde(default)]
    pub apis: ApisConfig,
    
    #[serde(default)]
    pub modules: ModulesConfig,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spec_path: Option<String>,
}

pub fn default_schema() -> String {
    "https://raw.githubusercontent.com/vikarno/vika-cli/main/schema/vika-config.schema.json".to_string()
}

fn default_root_dir() -> String {
    "src".to_string()
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModulesConfig {
    #[serde(default)]
    pub ignore: Vec<String>,
    
    #[serde(default)]
    pub selected: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            schema: default_schema(),
            root_dir: default_root_dir(),
            schemas: SchemasConfig {
                output: default_schemas_output(),
                naming: default_naming(),
            },
            apis: ApisConfig {
                output: default_apis_output(),
                style: default_style(),
                base_url: None,
                header_strategy: default_header_strategy(),
            },
            modules: ModulesConfig {
                ignore: vec![],
                selected: vec![],
            },
            spec_path: None,
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
        }
    }
}

impl Default for ModulesConfig {
    fn default() -> Self {
        Self {
            ignore: vec![],
            selected: vec![],
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
        assert_eq!(config.schemas.output, "src/schemas");
        assert_eq!(config.apis.output, "src/apis");
        assert_eq!(config.apis.style, "fetch");
        assert!(!config.schema.is_empty());
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let json = serde_json::to_string_pretty(&config).unwrap();
        
        assert!(json.contains("\"root_dir\""));
        assert!(json.contains("\"schemas\""));
        assert!(json.contains("\"apis\""));
        assert!(json.contains("\"$schema\""));
    }

    #[test]
    fn test_config_deserialization() {
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
            }
        }
        "#;
        
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.root_dir, "test");
        assert_eq!(config.schemas.output, "test/schemas");
        assert_eq!(config.schemas.naming, "camelCase");
        assert_eq!(config.apis.header_strategy, "bearerToken");
        assert_eq!(config.modules.ignore, vec!["test"]);
    }

    #[test]
    fn test_config_with_base_url() {
        let mut config = Config::default();
        config.apis.base_url = Some("/api/v1".to_string());
        
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
}

