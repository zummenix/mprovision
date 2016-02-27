
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

use std::fs::{self, DirEntry, File};
use std::path::{Path, PathBuf};
use std::env;
use std::io::{self, Read};
use plist::PlistEvent::*;

pub use error::Error;
pub use profile::Profile;

mod error;
mod profile;

fn execute<F, U, I>(iter: I, number_of_threads: usize, f: F) -> ThreadPoolIter<I, F, U> where F: Fn(I::Item) -> U, I: Iterator {
    ThreadPoolIter {
        iter: iter,
        map_op: f,
        pool_size: number_of_threads,
        buffer: Vec::new(),
    }
}

struct ThreadPoolIter<I, F, U> {
    iter: I,
    map_op: F,
    pool_size: usize,
    buffer: Vec<U>,
}

impl<I, F, U> Iterator for ThreadPoolIter<I, F, U> where I: Iterator, F: Fn(I::Item) -> U + Sync, I::Item: Send, U: Send {
    type Item = U;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buffer.is_empty() {
            let mut buf = Vec::with_capacity(self.pool_size);
            let mut n = 0;
            while n < self.pool_size {
                if let Some(item) = self.iter.next() {
                    buf.push(item);
                } else {
                    break;
                }
                n += 1;
            }
            if buf.is_empty() {
                None
            } else {
                self.buffer = crossbeam::scope(|scope| {
                    let ref f = self.map_op;
                    let guards: Vec<_> = buf.into_iter().map(|item| {
                        scope.spawn(move || (f)(item))
                    }).collect();
                    guards.into_iter().map(|guard| guard.join()).collect()
                });
                Some(self.buffer.remove(0))
            }
        } else {
            Some(self.buffer.remove(0))
        }
    }
}

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
pub fn files(path: &Path) -> Result<Box<Iterator<Item = DirEntry>>> {
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

pub fn profiles(path: &Path) -> Result<Box<Iterator<Item = Result<Profile>>>> {
    let files = try!(files(path));
    let iter = execute(files, 8, |entry| profile_from_file(entry.path().as_path()));
    Ok(Box::new(iter))
}

pub fn valid_profiles(path: &Path) -> Result<Box<Iterator<Item = Profile>>> {
    Ok(Box::new(try!(profiles(path)).filter(Result::is_ok).map(|r| r.unwrap())))
}

pub struct SearchInfo {
    pub total: usize,
    pub profiles: Vec<Profile>,
}

pub fn search(path: &Path, s: &str) -> Result<SearchInfo> {
    let mut total = 0;
    let profiles = try!(valid_profiles(path))
                       .filter(|profile| {
                           total += 1;
                           profile.contains(s)
                       })
                       .collect();

    Ok(SearchInfo {
        total: total,
        profiles: profiles,
    })
}

pub fn remove(path: &Path, uuid: &str) -> Result<()> {
    for profile in try!(valid_profiles(path)).into_iter() {
        if profile.uuid == uuid {
            try!(std::fs::remove_file(&profile.path));
            return Ok(());
        }
    }
    return Err(Error::Own(format!("Profile '{}' is not found.", uuid)));
}

/// Returns instance of the `Profile` parsed from a file.
pub fn profile_from_file(path: &Path) -> Result<Profile> {
    let mut file = try!(File::open(path));
    let mut buf = Vec::new();
    try!(file.read_to_end(&mut buf));
    profile_from_data(&buf)
        .map(|mut p| {
            p.path = path.to_owned();
            p
        })
        .ok_or(Error::Own("Couldn't parse file.".into()))
}

/// Returns instance of the `Profile` parsed from a `data`.
pub fn profile_from_data(data: &[u8]) -> Option<Profile> {
    if let Some(data) = find_plist(data) {
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
