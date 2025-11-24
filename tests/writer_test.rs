use std::fs;
use tempfile::TempDir;
use vika_cli::generator::writer::{ensure_directory, write_file_safe};

#[test]
fn test_ensure_directory() {
    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path().join("test/sub/dir");

    assert!(ensure_directory(&test_dir).is_ok());
    assert!(test_dir.exists());
    assert!(test_dir.is_dir());
}

#[test]
fn test_write_file_safe() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ts");
    let content = "export const test = 1;";

    assert!(write_file_safe(&test_file, content).is_ok());
    assert!(test_file.exists());

    let read_content = fs::read_to_string(&test_file).unwrap();
    assert_eq!(read_content, content);
}

#[test]
fn test_write_file_safe_skip_unchanged() {
    let temp_dir = TempDir::new().unwrap();
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
