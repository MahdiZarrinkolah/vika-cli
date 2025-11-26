use vika_cli::progress::ProgressReporter;

#[test]
fn test_progress_reporter_new_non_verbose() {
    let reporter = ProgressReporter::new(false);
    // Test passes if reporter is successfully created
    reporter.success("Test");
}

#[test]
fn test_progress_reporter_new_verbose() {
    let reporter = ProgressReporter::new(true);
    // Test passes if reporter is successfully created
    reporter.success("Test");
}

#[test]
fn test_start_and_finish_spinner_verbose() {
    let mut reporter = ProgressReporter::new(true);
    reporter.start_spinner("Test message");
    reporter.finish_spinner("Done");
    // Should not panic
}

#[test]
fn test_start_and_finish_spinner_non_verbose() {
    let mut reporter = ProgressReporter::new(false);
    reporter.start_spinner("Test");
    reporter.finish_spinner("Done");
    // Should not panic
}

#[test]
fn test_info_verbose() {
    let reporter = ProgressReporter::new(true);
    reporter.info("Test info");
    // Should not panic
}

#[test]
fn test_info_non_verbose() {
    let reporter = ProgressReporter::new(false);
    reporter.info("Test info");
    // Should not panic (doesn't print in non-verbose)
}

#[test]
fn test_success() {
    let reporter = ProgressReporter::new(false);
    reporter.success("Test success");
    // Should not panic
}

#[test]
fn test_warning() {
    let reporter = ProgressReporter::new(false);
    reporter.warning("Test warning");
    // Should not panic
}

#[test]
fn test_error() {
    let reporter = ProgressReporter::new(false);
    reporter.error("Test error");
    // Should not panic
}

#[test]
fn test_drop_with_spinner() {
    let mut reporter = ProgressReporter::new(false);
    reporter.start_spinner("Test");
    // Dropping reporter should clean up spinner
    drop(reporter);
    // Test passes if no panic
}

#[test]
fn test_multiple_spinner_operations() {
    let mut reporter = ProgressReporter::new(false);
    reporter.start_spinner("Operation 1");
    reporter.finish_spinner("Done 1");
    reporter.start_spinner("Operation 2");
    reporter.finish_spinner("Done 2");
    // Should handle multiple start/finish cycles
}

#[test]
fn test_all_message_types() {
    let reporter = ProgressReporter::new(true);
    reporter.info("Info message");
    reporter.success("Success message");
    reporter.warning("Warning message");
    reporter.error("Error message");
    // Should handle all message types
}
