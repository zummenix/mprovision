
//! **mprovision** is a tool that helps iOS developers to manage mobileprovision
//! files. Main purpose of this crate is to contain functions and types
//! for **mprovision**.

#[cfg(test)]
#[macro_use(expect)]
extern crate expectest;
#[cfg(test)]
extern crate tempdir;

use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};
use std::env;

/// A Result type for this crate.
pub type Result<T> = std::result::Result<T, String>;

/// Returns an iterator over the `*.mobileprovision` entries within a given
/// directory.
///
/// # Errors
/// This function will return an error in the following cases:
///
/// - the user lacks the requisite permissions
/// - there is no entry in the filesystem at the provided path
/// - the provided path is not a directory
pub fn files<P>(path: P) -> Result<Box<Iterator<Item = DirEntry>>>
    where P: AsRef<Path>
{
    let metadata = try!(fs::metadata(&path).map_err(|err| err.to_string()));
    if !metadata.is_dir() {
        return Err(format!("{:?} is not a directory", path.as_ref()));
    }
    let entries = try!(fs::read_dir(&path).map_err(|err| err.to_string()));
    let filtered = entries.filter_map(|entry| entry.ok())
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
pub fn directory() -> Result<PathBuf> {
    env::home_dir()
        .map(|path| path.join("Library/MobileDevice/Provisioning Profiles"))
        .ok_or("couldn't find home directory".into())
}

#[cfg(test)]
mod tests {
    use expectest::prelude::*;
    use tempdir::TempDir;
    use std::fs::File;
    use super::files;

    #[test]
    fn filter_mobileprovision_files() {
        let temp_dir = TempDir::new("test").unwrap();
        let result = files(temp_dir.path()).map(|iter| iter.count());
        expect!(result).to(be_ok().value(0));

        File::create(temp_dir.path().join("1.mobileprovision")).unwrap();
        File::create(temp_dir.path().join("2.mobileprovision")).unwrap();
        File::create(temp_dir.path().join("3.txt")).unwrap();
        let result = files(temp_dir.path()).map(|iter| iter.count());
        expect!(result).to(be_ok().value(2));
    }
}
