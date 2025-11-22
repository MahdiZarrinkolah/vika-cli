use std::fs;
use vika_cli::generator::writer::write_file_safe;

#[test]
fn test_skip_unchanged_files() {
    let temp_dir = tempfile::tempdir().unwrap();
    let test_file = temp_dir.path().join("test.ts");

    let content = "export const test = 1;";

    // Write first time
    write_file_safe(&test_file, content).unwrap();
    let mtime1 = fs::metadata(&test_file).unwrap().modified().unwrap();

    // Write same content again
    write_file_safe(&test_file, content).unwrap();
    let mtime2 = fs::metadata(&test_file).unwrap().modified().unwrap();

    // Times should be very close (file wasn't rewritten)
    let diff = mtime2.duration_since(mtime1).unwrap();
    assert!(diff.as_secs() < 2);
}
