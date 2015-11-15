
//! **mprovision** is a tool that helps iOS developers to manage mobileprovision
//! files. Main purpose of this crate is to contain functions and types
//! for **mprovision**.

#[cfg(test)]
#[macro_use(expect)]
extern crate expectest;
#[cfg(test)]
extern crate tempdir;
extern crate plist;

use std::fs::{self, DirEntry, File};
use std::path::{Path, PathBuf};
use std::env;
use std::io::{self, Read};
use std::fmt;
use std::error;
use plist::PlistEvent::*;

/// A Result type for this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// An Error type.
#[derive(Debug)]
pub enum Error {
    /// Denotes I/O error.
    Io(io::Error),
    /// Denotes error that produces this crate.
    Own(String),
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Io(ref e) => e.description(),
            Error::Own(ref e) => e,
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Io(ref e) => Some(e),
            Error::Own(_) => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref e) => e.fmt(f),
            Error::Own(ref e) => e.fmt(f),
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

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
    let mut file = try!(File::open(path));
    buf.clear();
    try!(file.read_to_end(buf));
    profile_from_data(buf).ok_or(Error::Own("Couldn't parse file.".into()))
}

/// Returns instance of the `Profile` parsed from a `data`.
pub fn profile_from_data(data: &[u8]) -> Option<Profile> {
    if let Some(data) = find_plist(data) {
        let mut profile = Profile::default();
        let mut iter = plist::StreamingParser::new(io::Cursor::new(data)).into_iter();
        while let Some(item) = iter.next() {
            if let Ok(StringValue(key)) = item {
                if key == "UUID" {
                    if let Some(Ok(StringValue(value))) = iter.next() {
                        profile.uuid = Some(value);
                    }
                }
                if key == "Name" {
                    if let Some(Ok(StringValue(value))) = iter.next() {
                        profile.name = Some(value);
                    }
                }
                if key == "application-identifier" {
                    if let Some(Ok(StringValue(value))) = iter.next() {
                        profile.app_identifier = Some(value);
                    }
                }
            }
        }
        Some(profile)
    } else {
        None
    }
}

/// Represents provisioning profile data.
#[derive(Default, Debug)]
pub struct Profile {
    pub uuid: Option<String>,
    pub name: Option<String>,
    pub app_identifier: Option<String>,
}

impl Profile {
    /// Returns `true` if one or more fields of the profile contain `string`.
    pub fn contains(&self, string: &str) -> bool {
        let s = string.to_lowercase();
        let items = &[self.name.as_ref(), self.app_identifier.as_ref(), self.uuid.as_ref()];
        for item in items {
            if let &Some(ref string) = item {
                if string.to_lowercase().contains(&s) {
                    return true;
                }
            }
        }
        false
    }

    /// Returns profile in a text form.
    pub fn description(&self) -> String {
        let mut desc = String::new();
        desc.push_str(self.uuid.as_ref().unwrap_or(&"<UUID not found>".into()));
        desc.push_str("\n");
        desc.push_str(self.app_identifier.as_ref().unwrap_or(&"<ID not found>".into()));
        desc.push_str("\n");
        desc.push_str(self.name.as_ref().unwrap_or(&"<Name not found>".into()));
        desc
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
