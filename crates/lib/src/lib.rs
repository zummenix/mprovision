//! **mprovision** is a tool that helps `iOS` developers to manage mobileprovision
//! files. Main purpose of this crate is to contain functions and types
//! for **mprovision**.

use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::error::Error;
use crate::profile::Profile;

pub mod error;
pub mod plist_extractor;
pub mod profile;

/// A Result type for this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// A file extension of a povisioning profile.
pub const EXT_MOBILEPROVISION: &str = "mobileprovision";

/// Returns true if the `file_path` is a provisioning profile file.
pub fn is_mobileprovision(file_path: &Path) -> bool {
    file_path.extension().and_then(|ext| ext.to_str()) == Some(EXT_MOBILEPROVISION)
}

/// Returns an iterator over the `*.mobileprovision` file paths within a given
/// directory.
///
/// # Errors
/// This function will return an error in the following cases:
///
/// - the user lacks the requisite permissions
/// - there is no entry in the filesystem at the provided path
/// - the provided path is not a directory
pub fn file_paths(dir: &Path) -> Result<impl Iterator<Item = PathBuf>> {
    let filtered = fs::read_dir(dir)?
        .filter(|entry| {
            entry
                .as_ref()
                .ok()
                .map(|entry| is_mobileprovision(entry.path().as_ref()))
                .unwrap_or(false)
        })
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path());
    Ok(filtered)
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
    dirs::home_dir()
        .map(|path| path.join("Library/MobileDevice/Provisioning Profiles"))
        .ok_or_else(|| {
            Error::Own(
                "'HOME' environment variable is not set or equal to the empty string.".to_owned(),
            )
        })
}

/// Returns `dir` or default [`directory`].
///
/// # Errors
/// The same as for [`directory`].
pub fn dir_or_default(dir: Option<PathBuf>) -> Result<PathBuf> {
    dir.map(Result::Ok).unwrap_or_else(directory)
}

/// Filters files using predicate function `f`.
///
/// The filtering is performed concurrently.
pub fn filter<F>(file_paths: Vec<PathBuf>, f: F) -> Vec<Profile>
where
    F: Fn(&Profile) -> bool + Send + Sync,
{
    use rayon::prelude::*;
    file_paths
        .par_iter()
        .map(|path| Profile::from_file(path))
        .filter_map(Result::ok)
        .filter(f)
        .collect()
}

/// Filters files of a directory using predicate function `f`.
///
/// Conveniently combines [`file_paths`] and [`filter`] functions together.
pub fn filter_dir<F>(dir: &Path, f: F) -> Result<Vec<Profile>>
where
    F: Fn(&Profile) -> bool + Send + Sync,
{
    Ok(filter(file_paths(dir)?.collect(), f))
}

/// Returns internals of a provisioning profile.
pub fn show(file_path: &Path) -> Result<String> {
    let mut buf = Vec::new();
    File::open(file_path)
        .and_then(|mut file| file.read_to_end(&mut buf))
        .map_err(|err| err.into())
        .and_then(|_| {
            plist_extractor::find(&buf)
                .ok_or_else(|| Error::Own(format!("Couldn't parse '{}'", file_path.display())))
        })
        .and_then(|data| String::from_utf8(data.to_owned()).map_err(|err| err.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_mobileprovision_files() {
        use std::fs::File;

        let temp_dir = tempfile::tempdir().unwrap();
        let result = file_paths(temp_dir.path()).map(|iter| iter.count()).unwrap();
        assert_eq!(result, 0);

        File::create(temp_dir.path().join("1.mobileprovision")).unwrap();
        File::create(temp_dir.path().join("2.mobileprovision")).unwrap();
        File::create(temp_dir.path().join("3.txt")).unwrap();
        let result = file_paths(temp_dir.path()).map(|iter| iter.count()).unwrap();
        assert_eq!(result, 2);
    }
}
