//! Path manipulation utilities.
//!
//! This module provides utility functions for manipulating file paths,
//! such as removing prefixes, normalizing paths, and other common operations.

use std::path::{Path, PathBuf};

/// Removes a prefix from a path if it exists.
///
/// This function checks if `path` starts with the given `prefix` and removes
/// the prefix if it does. If the path doesn't start with the prefix,
/// the original path is returned unchanged.
///
/// # Arguments
///
/// * `path` - The path to process
/// * `prefix` - The prefix to remove
///
/// # Returns
///
/// A new `PathBuf` with the prefix removed if it was present, or the original path otherwise.
///
/// # Examples
///
/// ```
/// use std::path::{Path, PathBuf};
/// use lumin::paths::remove_path_prefix;
///
/// let path = Path::new("/home/user/projects/myrepo/src/main.rs");
/// let prefix = Path::new("/home/user/projects/myrepo");
///
/// let result = remove_path_prefix(path, prefix);
/// assert_eq!(result, PathBuf::from("src/main.rs"));
///
/// // If the prefix doesn't match, the original path is returned
/// let other_prefix = Path::new("/tmp");
/// let unchanged = remove_path_prefix(path, other_prefix);
/// assert_eq!(unchanged, path);
/// ```
pub fn remove_path_prefix<P: AsRef<Path>, Q: AsRef<Path>>(path: P, prefix: Q) -> PathBuf {
    let path = path.as_ref();
    let prefix = prefix.as_ref();
    
    // Try to strip the prefix using the standard library function
    match path.strip_prefix(prefix) {
        Ok(stripped) => stripped.to_path_buf(),
        Err(_) => {
            // If strip_prefix fails (meaning the prefix doesn't match),
            // return the original path
            path.to_path_buf()
        }
    }
}

#[cfg(test)]
mod tests;