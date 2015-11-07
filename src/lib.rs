
//! **mprovision** is a tool that helps iOS developers to manage mobileprovision
//! files. Main purpose of this crate is to contain functions and types
//! for **mprovision**.

use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};
use std::env;

/// Returns an iterator over the `*.mobileprovision` entries within a given
/// directory.
///
/// # Errors
/// This function will return an error in the following cases:
///
/// - the user lacks the requisite permissions
/// - there is no entry in the filesystem at the provided path
/// - the provided path is not a directory
pub fn files<P>(path: P) -> Result<Box<Iterator<Item=DirEntry>>, String> where P: AsRef<Path> {
    let metadata = try!(fs::metadata(&path).map_err(|err| err.to_string()));
    if !metadata.is_dir() {
        return Err(format!("{:?} is not a directory", path.as_ref()));
    }
    let entries = try!(fs::read_dir(&path).map_err(|err| err.to_string()));
    let filtered = entries
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            if let Some(ext) = entry.path().extension() {
                if ext == "mobileprovision" {
                    return Some(entry);
                }
            }
            None
        });
    Ok(Box::new(filtered))
}

/// Returns the path to the directory that contains installed mobile
/// provisioning profiles.
///
/// Should return `~/Library/MobileDevice/Provisioning Profiles` directory.
///
/// # Errors
/// This function will return an error if 'HOME' environment variable is not set
/// or equal to the empty string.
pub fn directory() -> Result<PathBuf, String> {
    env::home_dir()
        .map(|path| path.join("Library/MobileDevice/Provisioning Profiles"))
        .ok_or("couldn't find home directory".into())
}
