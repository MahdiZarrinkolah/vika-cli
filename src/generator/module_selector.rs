use anyhow::{Context, Result};
use dialoguer::MultiSelect;
use colored::*;

pub fn select_modules(available_modules: &[String], ignored_modules: &[String]) -> Result<Vec<String>> {
    let filtered_modules: Vec<String> = available_modules
        .iter()
        .filter(|m| !ignored_modules.contains(m))
        .cloned()
        .collect();

    if filtered_modules.is_empty() {
        return Err(anyhow::anyhow!("No modules available after filtering"));
    }

    println!("{}", "Which modules do you want to generate?".bright_cyan());
    println!();

    let selections = MultiSelect::new()
        .with_prompt("Select modules (use space to toggle, enter to confirm)")
        .items(&filtered_modules)
        .interact()
        .context("Failed to get user selection")?;

    if selections.is_empty() {
        return Err(anyhow::anyhow!("No modules selected"));
    }

    let selected: Vec<String> = selections
        .iter()
        .map(|&i| filtered_modules[i].clone())
        .collect();

    Ok(selected)
}

