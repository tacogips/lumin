//! Tests for the paths module.

use super::*;
use std::path::Path;

#[test]
fn test_remove_path_prefix() {
    // Test removing a prefix that exists
    let path = Path::new("/home/user/projects/myrepo/src/main.rs");
    let prefix = Path::new("/home/user/projects/myrepo");
    let result = remove_path_prefix(path, prefix);
    assert_eq!(result, PathBuf::from("src/main.rs"));

    // Test with a path that doesn't have the prefix
    let path = Path::new("/var/log/syslog");
    let prefix = Path::new("/home/user");
    let result = remove_path_prefix(path, prefix);
    assert_eq!(result, PathBuf::from("/var/log/syslog"));

    // Test with a prefix that's a substring but not a proper path prefix
    let path = Path::new("/home/username/file.txt");
    let prefix = Path::new("/home/user");
    let result = remove_path_prefix(path, prefix);
    assert_eq!(result, PathBuf::from("/home/username/file.txt"));

    // Test with empty path
    let path = Path::new("");
    let prefix = Path::new("/home/user");
    let result = remove_path_prefix(path, prefix);
    assert_eq!(result, PathBuf::from(""));

    // Test with empty prefix
    let path = Path::new("/home/user/file.txt");
    let prefix = Path::new("");
    let result = remove_path_prefix(path, prefix);
    assert_eq!(result, PathBuf::from("/home/user/file.txt"));

    // Test with matching path and prefix (should result in empty path)
    let path = Path::new("/home/user");
    let prefix = Path::new("/home/user");
    let result = remove_path_prefix(path, prefix);
    assert_eq!(result, PathBuf::from(""));
}
