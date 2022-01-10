//! **mprovision** is a tool that helps `iOS` developers to manage mobileprovision
//! files. Main purpose of this crate is to contain functions and types
//! for **mprovision**.

use std::fs::{self, DirEntry, File};
use std::io::Read;
use std::path::{Path, PathBuf};

pub use crate::error::Error;
pub use crate::profile::Profile;

mod error;
mod plist_extractor;
pub mod profile;

/// A Result type for this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// Returns an iterator over the `*.mobileprovision` entries within a given
/// directory.
///
/// # Errors
/// This function will return an error in the following cases:
///
/// - the user lacks the requisite permissions
/// - there is no entry in the filesystem at the provided path
/// - the provided path is not a directory
pub fn entries(dir: &Path) -> Result<impl Iterator<Item = DirEntry>> {
    let entries = fs::read_dir(dir)?;
    let filtered = entries
        .filter(|entry| {
            entry.as_ref().ok().and_then(|v| {
                v.path()
                    .extension()
                    .map(|ext| ext.to_str() == Some("mobileprovision"))
            }) == Some(true)
        })
        .filter_map(std::result::Result::ok);
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

/// Returns `dir` if it is not `None` otherwise returns `directory()`.
pub fn with_directory(dir: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(d) = dir {
        Ok(d)
    } else {
        directory()
    }
}

/// Removes a provisioning profile.
pub fn remove(file_path: &Path) -> Result<()> {
    let path = validate_path(file_path)?;
    std::fs::remove_file(path)?;
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

/// Filters entries of a directory using `f`.
///
/// The filtering is perfomed concurrently.
pub fn filter<F>(entries: Vec<DirEntry>, f: F) -> Vec<Profile>
where
    F: Fn(&Profile) -> bool + Send + Sync,
{
    use rayon::prelude::*;
    entries
        .par_iter()
        .map(|entry| Profile::from_file(&entry.path()))
        .filter_map(Result::ok)
        .filter(f)
        .collect()
}

/// Searches a provisioning profile by its uuid.
///
/// The search is performed concurrently.
pub fn find_by_uuid(dir: &Path, uuid: &str) -> Result<Profile> {
    let entries: Vec<DirEntry> = entries(dir)?.collect();
    if let Some(profile) = filter(entries, |profile| profile.info.uuid == uuid).pop() {
        Ok(profile)
    } else {
        Err(Error::Own(format!("Profile '{}' is not found.", uuid)))
    }
}

/// Searches provisioning profiles by their uuid(s) or bundle id(s).
///
/// The search is performed concurrently.
pub fn find_by_ids(dir: &Path, ids: &[String]) -> Result<Vec<Profile>> {
    let entries: Vec<DirEntry> = entries(dir)?.collect();
    let profiles = filter(entries, |profile| {
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
        let result = entries(temp_dir.path()).map(|iter| iter.count());
        expect!(result).to(be_ok().value(0));

        File::create(temp_dir.path().join("1.mobileprovision")).unwrap();
        File::create(temp_dir.path().join("2.mobileprovision")).unwrap();
        File::create(temp_dir.path().join("3.txt")).unwrap();
        let result = entries(temp_dir.path()).map(|iter| iter.count());
        expect!(result).to(be_ok().value(2));
    }
}
