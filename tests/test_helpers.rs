use anyhow::Result;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

// The main test directory
pub const TEST_DIR: &str = "tests/test_dir_1";

/// Set up temporary files for testing (files that would be ignored by gitignore)
pub fn setup_ignored_files() -> Result<Vec<PathBuf>> {
    let mut created_files = Vec::new();

    // Create a .hidden directory if it doesn't exist
    let hidden_dir = PathBuf::from(TEST_DIR).join(".hidden");
    if !hidden_dir.exists() {
        fs::create_dir_all(&hidden_dir)?;
    }

    // Create a secret file in the hidden directory with a specific pattern
    let secret_file = hidden_dir.join("secret.txt");
    let mut file = File::create(&secret_file)?;
    writeln!(
        file,
        "This is a hidden file that should be ignored by default when respecting gitignore."
    )?;
    writeln!(file, "")?;
    writeln!(file, "It contains some sensitive information:")?;
    writeln!(file, "API_KEY=test_key_12345")?;
    writeln!(file, "SECRET=test_secret_67890")?;
    created_files.push(secret_file);

    // Create some temporary files (should be ignored by gitignore)
    let temp_file = PathBuf::from(TEST_DIR).join("temp_file.tmp");
    let mut file = File::create(&temp_file)?;
    writeln!(
        file,
        "This is a temporary file that should be ignored by gitignore."
    )?;
    created_files.push(temp_file);

    // Create a log file (should be ignored by gitignore)
    let log_file = PathBuf::from(TEST_DIR).join("test.log");
    let mut file = File::create(&log_file)?;
    writeln!(
        file,
        "DEBUG: This is a log file that should be ignored by gitignore."
    )?;
    writeln!(file, "INFO: Test log entry")?;
    writeln!(file, "ERROR: Test error message")?;
    created_files.push(log_file);

    // Add patterns to .gitignore file if it doesn't already have them
    let gitignore_path = PathBuf::from(TEST_DIR).join(".gitignore");
    if !gitignore_path.exists() || !fs::read_to_string(&gitignore_path)?.contains(".hidden") {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(&gitignore_path)?;

        writeln!(file, "# Test gitignore file")?;
        writeln!(file, "# Ignore temporary files")?;
        writeln!(file, "*.tmp")?;
        writeln!(file, "# Ignore log files")?;
        writeln!(file, "*.log")?;
        writeln!(file, "# Ignore hidden directories")?;
        writeln!(file, ".hidden/")?;
    }

    Ok(created_files)
}

/// Clean up temporary files created for testing
pub fn teardown_ignored_files(created_files: &[PathBuf]) -> Result<()> {
    for file_path in created_files {
        if file_path.exists() {
            fs::remove_file(file_path)?;
        }
    }

    Ok(())
}

/// Creates a test environment for a specific test
pub struct TestEnvironment {
    pub created_files: Vec<PathBuf>,
}

impl TestEnvironment {
    /// Set up a test environment
    pub fn setup() -> Result<Self> {
        let created_files = setup_ignored_files()?;
        Ok(TestEnvironment { created_files })
    }
}

impl Drop for TestEnvironment {
    fn drop(&mut self) {
        if let Err(e) = teardown_ignored_files(&self.created_files) {
            eprintln!("Warning: Failed to clean up test files: {}", e);
        }
    }
}
