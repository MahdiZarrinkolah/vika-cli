use vika_cli::progress::ProgressReporter;

#[test]
    fn test_progress_reporter_new() {
        let reporter = ProgressReporter::new(false);
        assert!(!reporter.verbose);
    }

    #[test]
    fn test_progress_reporter_verbose() {
        let reporter = ProgressReporter::new(true);
        assert!(reporter.verbose);
    }

    #[test]
    fn test_start_spinner_verbose() {
        let mut reporter = ProgressReporter::new(true);
        reporter.start_spinner("Test message");
        // In verbose mode, spinner should be None
        assert!(reporter.spinner.is_none());
    }

    #[test]
    fn test_start_spinner_non_verbose() {
        let mut reporter = ProgressReporter::new(false);
        reporter.start_spinner("Test message");
        // In non-verbose mode, spinner should be Some
        assert!(reporter.spinner.is_some());
    }

    #[test]
    fn test_finish_spinner_verbose() {
        let mut reporter = ProgressReporter::new(true);
        reporter.start_spinner("Test");
        reporter.finish_spinner("Done");
        // Should not panic
    }

    #[test]
    fn test_finish_spinner_non_verbose() {
        let mut reporter = ProgressReporter::new(false);
        reporter.start_spinner("Test");
        reporter.finish_spinner("Done");
        assert!(reporter.spinner.is_none());
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

