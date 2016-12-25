//! **mprovision** is a tool that helps `iOS` developers to manage mobileprovision
//! files. Main purpose of this crate is to contain functions and types
//! for **mprovision**.

#[cfg(test)]
#[macro_use(expect)]
extern crate expectest;
#[cfg(test)]
extern crate tempdir;
extern crate plist;
extern crate chrono;
extern crate memmem;

extern crate futures;
extern crate futures_cpupool;
extern crate num_cpus;

use futures::stream::Stream;
use futures::Future;
use futures_cpupool::CpuPool;

use chrono::*;

use std::fs::{self, DirEntry, File};
use std::path::{Path, PathBuf};
use std::env;
use std::io::Read;

pub use error::Error;
pub use profile::Profile;
pub use context::Context;

mod error;
mod profile;
mod context;

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
pub fn entries(dir: &Path) -> Result<Box<Iterator<Item = DirEntry>>> {
    let entries = fs::read_dir(dir)?;
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
        .ok_or_else(|| {
            Error::Own("'HOME' environment variable is not set or equal to the empty string."
                .to_owned())
        })
}

pub fn with_directory(dir: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(d) = dir {
        Ok(d)
    } else {
        directory()
    }
}

pub struct SearchInfo {
    pub total: usize,
    pub profiles: Vec<Profile>,
}

pub fn search(dir: &Path, s: &str) -> Result<SearchInfo> {
    let entries: Vec<DirEntry> = entries(dir)?.collect();
    Ok(SearchInfo {
        total: entries.len(),
        profiles: parallel(entries, |profile| profile.contains(s)),
    })
}

pub fn remove(file_path: &Path) -> Result<()> {
    file_path.extension()
        .and_then(|extension| if extension == "mobileprovision" {
            Some(file_path)
        } else {
            None
        })
        .ok_or_else(|| {
            Error::Own(format!("'{}' doesn't have 'mobileprovision' extension",
                               file_path.display()))
        })
        .and_then(|file_path| std::fs::remove_file(file_path).map_err(|err| err.into()))
}

pub fn xml(dir: &Path, uuid: &str) -> Result<String> {
    match find_by_uuid(dir, uuid) {
        Ok(profile) => {
            let context = Context::default();
            let mut buf = Vec::new();
            File::open(&profile.path)?.read_to_end(&mut buf)?;
            let data = context.find_plist(&buf).ok_or(Error::Own("Couldn't parse file.".into()))?;
            Ok(String::from_utf8(data.to_owned())?)
        }
        Err(err) => Err(err),
    }
}

pub fn expired_profiles(dir: &Path, date: DateTime<UTC>) -> Result<SearchInfo> {
    let entries: Vec<DirEntry> = entries(dir)?.collect();
    Ok(SearchInfo {
        total: entries.len(),
        profiles: parallel(entries, |profile| profile.expiration_date <= date),
    })
}

fn parallel<F>(entries: Vec<DirEntry>, f: F) -> Vec<Profile>
    where F: Fn(&Profile) -> bool + Sync
{
    let cpu_pool = CpuPool::new(num_cpus::get());

    let stream = futures::stream::iter(entries.into_iter().map(|entry| Ok(entry)))
        .map(|entry| cpu_pool.spawn_fn(move || Profile::from_file(&entry.path())))
        .buffered(num_cpus::get() * 2)
        .filter(f)
        .collect();

    stream.wait().unwrap_or(Vec::new())
}

pub fn find_by_uuid(dir: &Path, uuid: &str) -> Result<Profile> {
    let entries: Vec<DirEntry> = entries(dir)?.collect();
    if let Some(profile) = parallel(entries, |profile| profile.uuid == uuid).pop() {
        Ok(profile)
    } else {
        Err(Error::Own(format!("Profile '{}' is not found.", uuid)))
    }
}

#[cfg(test)]
mod tests {
    use expectest::prelude::*;
    use super::*;

    #[test]
    fn filter_mobileprovision_files() {
        use tempdir::TempDir;
        use std::fs::File;

        let temp_dir = TempDir::new("test").unwrap();
        let result = entries(temp_dir.path()).map(|iter| iter.count());
        expect!(result).to(be_ok().value(0));

        File::create(temp_dir.path().join("1.mobileprovision")).unwrap();
        File::create(temp_dir.path().join("2.mobileprovision")).unwrap();
        File::create(temp_dir.path().join("3.txt")).unwrap();
        let result = entries(temp_dir.path()).map(|iter| iter.count());
        expect!(result).to(be_ok().value(2));
    }
}
