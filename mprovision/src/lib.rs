
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

use std::fs::{self, DirEntry, File};
use std::path::{Path, PathBuf};
use std::env;
use std::io::{self, Read};
use plist::PlistEvent::*;

pub use error::Error;
pub use profile::Profile;

mod error;
mod profile;

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
pub fn files<P>(path: P) -> Result<Box<Iterator<Item = DirEntry>>>
    where P: AsRef<Path>
{
    let entries = try!(fs::read_dir(&path));
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
        .ok_or(Error::Io(io::Error::new(io::ErrorKind::NotFound, "")))
}

pub fn search<P>(path: Option<P>, s: &str) -> Result<Vec<Result<Profile>>>
    where P: AsRef<Path>
{
    if let Some(path) = path {
        search_dir(path, s)
    } else {
        let path = try!(directory());
        search_dir(path, s)
    }
}

pub fn search_dir<P>(path: P, s: &str) -> Result<Vec<Result<Profile>>>
    where P: AsRef<Path>
{
    let files = try!(files(&path));
    let mut buf = Vec::with_capacity(100 * 1024);
    let results: Vec<_> = files.map(|entry| profile_from_file(entry.path(), &mut buf))
                               .filter(|result| {
                                   if let &Ok(ref profile) = result {
                                       profile.contains(s)
                                   } else {
                                       true
                                   }
                               })
                               .collect();
    Ok(results)
}

/// Returns instance of the `Profile` parsed from a file.
pub fn profile_from_file<P>(path: P, buf: &mut Vec<u8>) -> Result<Profile>
    where P: AsRef<Path>
{
    let mut file = try!(File::open(path.as_ref()));
    buf.clear();
    try!(file.read_to_end(buf));
    profile_from_data(buf).map(|mut p| {
        p.path = path.as_ref().to_owned();
        p
    }).ok_or(Error::Own("Couldn't parse file.".into()))
}

/// Returns instance of the `Profile` parsed from a `data`.
pub fn profile_from_data(data: &[u8]) -> Option<Profile> {
    if let Some(data) = find_plist(data) {
        let mut profile = Profile::empty();
        let mut iter = plist::StreamingParser::new(io::Cursor::new(data)).into_iter();
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

/// Returns an index where the `needle` starts in the `data`.
fn find(data: &[u8], needle: &[u8]) -> Option<usize> {
    let needle_len = needle.len();
    for (i, _) in data.iter().enumerate() {
        if i + needle_len > data.len() {
            return None;
        }
        if &data[i..i + needle_len] == needle {
            return Some(i);
        }
    }
    None
}

/// Returns a plist content in a `data`.
fn find_plist(data: &[u8]) -> Option<&[u8]> {
    let prefix = b"<?xml version=";
    let suffix = b"</plist>";

    let start_i = find(data, prefix);
    let end_i = find(data, suffix).map(|i| i + suffix.len());

    if let (Some(start_i), Some(end_i)) = (start_i, end_i) {
        if end_i < data.len() {
            return Some(&data[start_i..end_i]);
        }
    }

    None
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
