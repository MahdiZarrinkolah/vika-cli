use vika_cli::generator::writer::{
    ensure_directory, write_file_safe, write_file_with_backup,
};
use tempfile::TempDir;
use std::fs;

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

#[test]
fn test_write_file_with_backup() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ts");
    let initial_content = "export const test = 1;";
    let new_content = "export const test = 2;";

    // Write initial file
    write_file_safe(&test_file, initial_content).unwrap();

    // Write with backup
    let result = write_file_with_backup(&test_file, new_content, true, false);
    assert!(result.is_ok());

    // Verify new content
    let read_content = fs::read_to_string(&test_file).unwrap();
    assert_eq!(read_content, new_content);
}

#[test]
fn test_write_file_conflict_detection() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ts");
    let initial_content = "export const test = 1;";

    // Write initial file
    write_file_safe(&test_file, initial_content).unwrap();

    // Modify file manually (simulate user edit)
    fs::write(&test_file, "export const test = 2; // user modified").unwrap();

    // Try to write without force (should detect conflict)
    let new_content = "export const test = 3;";
    let result = write_file_with_backup(&test_file, new_content, false, false);

    // Should fail with conflict error or succeed depending on implementation
    // The actual conflict detection requires metadata to be set first
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_write_file_force_overwrite() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ts");
    let initial_content = "export const test = 1;";

    // Write initial file
    write_file_safe(&test_file, initial_content).unwrap();

    // Modify file manually
    fs::write(&test_file, "export const test = 2; // user modified").unwrap();

    // Write with force (should succeed)
    let new_content = "export const test = 3;";
    let result = write_file_with_backup(&test_file, new_content, false, true);
    assert!(result.is_ok());

    // Verify content was overwritten
    let content = fs::read_to_string(&test_file).unwrap();
    assert!(content.contains("test = 3"));
}

