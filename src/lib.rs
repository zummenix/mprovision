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
extern crate num_cpus;

use std::fs::{self, DirEntry, File};
use std::path::{Path, PathBuf};
use std::env;
use std::io::{self, Read};
use plist::PlistEvent::*;

use std::thread;
use std::thread::JoinHandle;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;

use crossbeam::sync::chase_lev;
use crossbeam::sync::chase_lev::*;

pub use error::Error;
pub use profile::Profile;

mod error;
mod profile;

fn execute<F, I, J>(iter: I, number_of_threads: usize, f: F) -> JobIter<J::Output> where F: Fn(I::Item) -> J, J: Job + Send + 'static, J::Output: Send + 'static, I: Iterator {
    let (mut worker, stealer) = chase_lev::deque();
    let (tx, rx) = channel();

    for item in iter {
        let job = (f)(item);
        worker.push(job);
    }

    let mut handles = Vec::new();
    for thread_id in 0..number_of_threads {
        let stealer = stealer.clone();
        let tx = tx.clone();

        let handle = thread::spawn(move|| {
            loop {
                match stealer.steal() {
                    Steal::Data(job) => tx.send(Event::Data(thread_id, job.execute())).unwrap_or(()),
                    Steal::Empty => break,
                    _ => (),
                }
            }

            tx.send(Event::Finish(thread_id)).unwrap_or(());
        });

        handles.push((thread_id, handle));
    }

    JobIter {
        handles: handles,
        receiver: rx,
    }
}

trait Job {
    type Output;
    fn execute(self) -> Self::Output;
}

struct ReadProfileJob(DirEntry);

impl Job for ReadProfileJob {
    type Output = Result<Profile>;
    fn execute(self) -> Self::Output {
        profile_from_file(self.0.path().as_path())
    }
}

struct JobIter<T> {
    handles: Vec<(usize, JoinHandle<()>)>,
    receiver: Receiver<Event<T>>,
}

impl<T> Iterator for JobIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.receiver.recv() {
                Ok(event) => {
                    match event {
                        Event::Finish(thread_index) => {
                            let mut index = None;
                            for (i, &(id, _)) in self.handles.iter().enumerate() {
                                if thread_index == id {
                                    index = Some(i);
                                    break;
                                }
                            }
                            if let Some(i) = index {
                                let handle = self.handles.swap_remove(i);
                                match handle.1.join() {
                                    Err(error) => println!("Error: {:?}", error),
                                    Ok(_) => (),
                                }
                            }
                        }
                        Event::Data(_, data) => {
                            return Some(data);
                        }
                    }
                },
                Err(_) => {
                    break;
                },
            }
        }
        None
    }
}

enum Event<T> {
    Finish(usize),
    Data(usize, T),
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
    let iter = execute(files, num_cpus::get(), |entry| ReadProfileJob(entry));
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

pub fn xml(path: &Path, uuid: &str) -> Result<String> {
    for profile in try!(valid_profiles(path)).into_iter() {
        if profile.uuid == uuid {
            let mut file = try!(File::open(&profile.path));
            let mut buf = Vec::new();
            try!(file.read_to_end(&mut buf));
            let data = try!(find_plist(&buf).ok_or(Error::Own("Couldn't parse file.".into())));
            return Ok(try!(String::from_utf8(data.to_owned())));
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
