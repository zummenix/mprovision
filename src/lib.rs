//! **mprovision** is a tool that helps `iOS` developers to manage mobileprovision
//! files. Main purpose of this crate is to contain functions and types
//! for **mprovision**.

use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

pub use crate::error::Error;
pub use crate::profile::Profile;

mod error;
mod plist_extractor;
pub mod profile;

/// A Result type for this crate.
pub type Result<T> = std::result::Result<T, Error>;

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
            entry.as_ref().ok().and_then(|v| {
                v.path()
                    .extension()
                    .map(|ext| ext.to_str() == Some("mobileprovision"))
            }) == Some(true)
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

/// Removes a provisioning profile.
pub fn remove(file_path: &Path, permanently: bool) -> Result<()> {
    let path = validate_path(file_path)?;
    if permanently {
        std::fs::remove_file(path)?;
    } else {
        trash::delete(path)?;
    }
    Ok(())
}

/// Returns internals of a provisioning profile.
pub fn show(file_path: &Path) -> Result<String> {
    validate_path(file_path).and_then(|file_path| {
        let mut buf = Vec::new();
        File::open(file_path)
            .and_then(|mut file| file.read_to_end(&mut buf))
            .map_err(|err| err.into())
            .and_then(|_| {
                plist_extractor::find(&buf)
                    .ok_or_else(|| Error::Own(format!("Couldn't parse '{}'", file_path.display())))
            })
            .and_then(|data| String::from_utf8(data.to_owned()).map_err(|err| err.into()))
    })
}

/// Validates that `file_path` has a `mobileprovision` extension.
fn validate_path(file_path: &Path) -> Result<&Path> {
    match file_path.extension() {
        Some(extension) if extension == "mobileprovision" => Ok(file_path),
        _ => Err(Error::Own(format!(
            "'{}' doesn't have 'mobileprovision' extension",
            file_path.display()
        ))),
    }
}

/// Filters files of a directory using `f`.
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

/// Searches a provisioning profile by its uuid.
///
/// The search is performed concurrently.
pub fn find_by_uuid(dir: &Path, uuid: &str) -> Result<Profile> {
    let paths: Vec<PathBuf> = file_paths(dir)?.collect();
    if let Some(profile) = filter(paths, |profile| profile.info.uuid == uuid).pop() {
        Ok(profile)
    } else {
        Err(Error::Own(format!("Profile '{}' is not found.", uuid)))
    }
}

/// Searches provisioning profiles by their uuid(s) or bundle id(s).
///
/// The search is performed concurrently.
pub fn find_by_ids(dir: &Path, ids: &[String]) -> Result<Vec<Profile>> {
    let paths: Vec<PathBuf> = file_paths(dir)?.collect();
    let profiles = filter(paths, |profile| {
        ids.iter()
            .any(|id| id == &profile.info.uuid || profile.info.bundle_id() == Some(id))
    });
    Ok(profiles)
}

#[cfg(test)]
mod tests {
    use super::*;
    use expectest::expect;
    use expectest::prelude::*;

    #[test]
    fn filter_mobileprovision_files() {
        use std::fs::File;

        let temp_dir = tempfile::tempdir().unwrap();
        let result = file_paths(temp_dir.path()).map(|iter| iter.count());
        expect!(result).to(be_ok().value(0));

        File::create(temp_dir.path().join("1.mobileprovision")).unwrap();
        File::create(temp_dir.path().join("2.mobileprovision")).unwrap();
        File::create(temp_dir.path().join("3.txt")).unwrap();
        let result = file_paths(temp_dir.path()).map(|iter| iter.count());
        expect!(result).to(be_ok().value(2));
    }
}
