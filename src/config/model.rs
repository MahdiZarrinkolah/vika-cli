use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
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

fn default_root_dir() -> String {
    "src".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemasConfig {
    #[serde(default = "default_schemas_output")]
    pub output: String,
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
            root_dir: default_root_dir(),
            schemas: SchemasConfig {
                output: default_schemas_output(),
            },
            apis: ApisConfig {
                output: default_apis_output(),
                style: default_style(),
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
        }
    }
}

impl Default for ApisConfig {
    fn default() -> Self {
        Self {
            output: default_apis_output(),
            style: default_style(),
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

