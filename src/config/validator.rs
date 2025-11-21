use anyhow::Result;
use std::path::PathBuf;
use crate::config::model::Config;

pub fn validate_config(config: &Config) -> Result<()> {
    // Validate root_dir
    let root_dir = PathBuf::from(&config.root_dir);
    if root_dir.is_absolute() && !root_dir.exists() {
        return Err(anyhow::anyhow!(
            "Root directory does not exist: {}",
            config.root_dir
        ));
    }
    
    // Validate schemas output path
    let schemas_output = PathBuf::from(&config.schemas.output);
    if schemas_output.is_absolute() {
        validate_safe_path(&schemas_output)?;
    }
    
    // Validate apis output path
    let apis_output = PathBuf::from(&config.apis.output);
    if apis_output.is_absolute() {
        validate_safe_path(&apis_output)?;
    }
    
    // Validate style
    if config.apis.style != "fetch" {
        return Err(anyhow::anyhow!(
            "Unsupported API style: {}. Only 'fetch' is supported.",
            config.apis.style
        ));
    }
    
    Ok(())
}

fn validate_safe_path(path: &PathBuf) -> Result<()> {
    // Prevent writing to system directories
    let path_str = path.to_string_lossy();
    
    if path_str.contains("/etc/") 
        || path_str.contains("/usr/") 
        || path_str.contains("/bin/")
        || path_str.contains("/sbin/")
        || path_str.contains("/var/")
        || path_str.contains("/opt/")
        || path_str == "/"
        || path_str == "/root"
    {
        return Err(anyhow::anyhow!(
            "Unsafe output path detected: {}. Cannot write to system directories.",
            path_str
        ));
    }
    
    Ok(())
}

