use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_extract_command() {
    // Create a temporary directory for the test
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let temp_path = temp_dir.path();

    // Path to a sample IPA file (replace with an actual test fixture if available)
    let sample_ipa = "tests/fixtures/sample.ipa";

    // Ensure the sample IPA file exists
    assert!(Path::new(sample_ipa).exists(), "Sample IPA file is missing");

    // Run the `extract` command
    let mut cmd = Command::cargo_bin("mprovision").expect("Binary not found");
    cmd.arg("extract")
        .arg(sample_ipa)
        .arg(temp_path);

    // Assert the command runs successfully
    cmd.assert().success();

    // Check that the extracted files exist in the temp directory
    let extracted_files: Vec<_> = fs::read_dir(temp_path)
        .expect("Failed to read temp dir")
        .map(|entry| entry.expect("Failed to read entry").path())
        .collect();

    assert!(!extracted_files.is_empty(), "No files were extracted");

    // Additional assertions can be added to verify the contents of the extracted files
}