use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::Arc;

pub struct ProgressReporter {
    verbose: bool,
    spinner: Option<Arc<ProgressBar>>,
}

impl ProgressReporter {
    pub fn new(verbose: bool) -> Self {
        Self {
            verbose,
            spinner: None,
        }
    }

    pub fn start_spinner(&mut self, message: &str) {
        if !self.verbose {
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.cyan} {msg}")
                    .unwrap(),
            );
            pb.set_message(message.to_string());
            pb.enable_steady_tick(std::time::Duration::from_millis(100));
            self.spinner = Some(Arc::new(pb));
        } else {
            println!("{}", format!("⏳ {}", message).bright_cyan());
        }
    }

    pub fn finish_spinner(&mut self, message: &str) {
        if let Some(spinner) = self.spinner.take() {
            spinner.finish_with_message(message.to_string());
        } else if self.verbose {
            println!("{}", format!("✅ {}", message).green());
        }
    }

    pub fn info(&self, message: &str) {
        if self.verbose {
            println!("{}", format!("ℹ️  {}", message).bright_blue());
        }
    }

    pub fn success(&self, message: &str) {
        println!("{}", format!("✅ {}", message).green());
    }

    pub fn warning(&self, message: &str) {
        println!("{}", format!("⚠️  {}", message).yellow());
    }

    pub fn error(&self, message: &str) {
        eprintln!("{}", format!("❌ {}", message).red());
    }
}

impl Drop for ProgressReporter {
    fn drop(&mut self) {
        if let Some(spinner) = self.spinner.take() {
            spinner.finish_and_clear();
        }
    }
}
