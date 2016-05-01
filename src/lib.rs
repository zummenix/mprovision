//! **mprovision** is a tool that helps iOS developers to manage mobileprovision
//! files. Main purpose of this crate is to contain functions and types
//! for **mprovision**.

#[cfg(test)]
#[macro_use(expect)]
extern crate expectest;
#[cfg(test)]
extern crate tempdir;
extern crate plist;
extern crate chrono;
extern crate crossbeam;
extern crate rayon;
extern crate memmem;

use plist::PlistEvent::*;
use chrono::*;
use rayon::prelude::*;

use std::fs::{self, DirEntry, File};
use std::path::{Path, PathBuf};
use std::env;
use std::io::{self, Read};

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
pub fn entries(path: &Path) -> Result<Box<Iterator<Item = DirEntry>>> {
    let entries = try!(fs::read_dir(path));
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
        .ok_or(Error::Own("'HOME' environment variable is not set or equal to the empty string."
                              .to_owned()))
}

pub fn with_path<F, T>(path: Option<&Path>, f: F) -> Result<T>
    where F: FnOnce(&Path) -> Result<T>
{
    if let Some(path) = path {
        f(&path)
    } else {
        let path = try!(directory());
        f(&path)
    }
}

pub struct SearchInfo {
    pub total: usize,
    pub profiles: Vec<Profile>,
}

pub fn search(path: &Path, s: &str) -> Result<SearchInfo> {
    let entries: Vec<DirEntry> = try!(entries(path)).collect();
    Ok(SearchInfo {
        total: entries.len(),
        profiles: parallel(entries, |profile| profile.contains(s)),
    })
}

pub fn remove(path: &Path, uuid: &str) -> Result<()> {
    find_by_uuid(path, uuid)
        .and_then(|profile| std::fs::remove_file(&profile.path).map_err(|err| err.into()))
}

pub fn xml(path: &Path, uuid: &str) -> Result<String> {
    match find_by_uuid(path, uuid) {
        Ok(profile) => {
            let context = Context::new();
            let mut file = try!(File::open(&profile.path));
            let mut buf = Vec::new();
            try!(file.read_to_end(&mut buf));
            let data = try!(context.find_plist(&buf)
                                   .ok_or(Error::Own("Couldn't parse file.".into())));
            Ok(try!(String::from_utf8(data.to_owned())))
        }
        Err(err) => Err(err),
    }
}

pub fn expired_profiles(path: &Path, date: DateTime<UTC>) -> Result<SearchInfo> {
    let entries: Vec<DirEntry> = try!(entries(path)).collect();
    Ok(SearchInfo {
        total: entries.len(),
        profiles: parallel(entries, |profile| profile.expiration_date <= date),
    })
}

/// Returns instance of the `Profile` parsed from a file.
pub fn profile_from_file(path: &Path, context: &Context) -> Result<Profile> {
    let mut file = try!(File::open(path));
    let mut buf = context.buffers_pool.acquire();
    try!(file.read_to_end(&mut buf));
    let result = profile_from_data(&buf, context)
        .map(|mut p| {
            p.path = path.to_owned();
            p
        })
        .ok_or(Error::Own("Couldn't parse file.".into()));
    context.buffers_pool.release(buf);
    result
}

/// Returns instance of the `Profile` parsed from a `data`.
pub fn profile_from_data(data: &[u8], context: &Context) -> Option<Profile> {
    if let Some(data) = context.find_plist(data) {
        let mut profile = Profile::empty();
        let mut iter = plist::xml::EventReader::new(io::Cursor::new(data)).into_iter();
        while let Some(item) = iter.next() {
            if let Ok(StringValue(key)) = item {
                if key == "UUID" {
                    if let Some(Ok(StringValue(value))) = iter.next() {
                        profile.uuid = value;
                    }
                }
                if key == "Name" {
                    if let Some(Ok(StringValue(value))) = iter.next() {
                        profile.name = value;
                    }
                }
                if key == "application-identifier" {
                    if let Some(Ok(StringValue(value))) = iter.next() {
                        profile.app_identifier = value;
                    }
                }
                if key == "CreationDate" {
                    if let Some(Ok(DateValue(value))) = iter.next() {
                        profile.creation_date = value;
                    }
                }
                if key == "ExpirationDate" {
                    if let Some(Ok(DateValue(value))) = iter.next() {
                        profile.expiration_date = value;
                    }
                }
            }
        }
        Some(profile)
    } else {
        None
    }
}

fn parallel<F>(entries: Vec<DirEntry>, f: F) -> Vec<Profile>
    where F: Fn(&Profile) -> bool + Sync
{
    let context = Context::new();
    collect(entries.into_par_iter()
                   .weight_max()
                   .filter_map(|entry| profile_from_file(&entry.path(), &context).ok())
                   .filter(f))
}

fn collect<I>(par_iter: I) -> Vec<I::Item>
    where I: ParallelIterator
{
    let queue = crossbeam::sync::MsQueue::new();

    par_iter.for_each(|item| queue.push(item));

    let mut items = Vec::new();
    while let Some(item) = queue.try_pop() {
        items.push(item);
    }
    items
}

fn find_by_uuid(path: &Path, uuid: &str) -> Result<Profile> {
    let entries: Vec<DirEntry> = try!(entries(path)).collect();
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
