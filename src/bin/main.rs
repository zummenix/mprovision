
extern crate mprovision;
#[macro_use(version)]
extern crate version;
extern crate chrono;
#[cfg(test)]
#[macro_use(expect)]
extern crate expectest;
extern crate clap;

use mprovision as mp;
use cli::Command;
use std::env;
use std::path::PathBuf;
use chrono::*;

mod cli;

fn main() {
    let result = cli::parse(env::args()).and_then(|command| {
        match command {
            Command::List(cli::ListParams { filter, expire_in_days, directory }) => {
                list(filter, expire_in_days, directory)
            }
            Command::ShowUuid(uuid, directory) => show_uuid(uuid, directory),
            Command::ShowFile(path) => show_file(path),
            Command::Remove(uuid, directory) => remove(uuid, directory),
            Command::Clean(directory) => clean(directory),
        }
    });
    if let Err(e) = result {
        e.exit();
    }
}

fn list(filter: Option<String>,
        expires_in_days: Option<i64>,
        directory: Option<PathBuf>)
        -> Result<(), cli::Error> {
    mp::with_directory(directory)
        .and_then(|dir| mp::entries(&dir).map(|entries| entries.collect::<Vec<_>>()))
        .map_err(|err| err.into())
        .map(|entries| {
            let total = entries.len();
            let date = expires_in_days.map(|days| UTC::now() + Duration::days(days));
            let filter_string = filter.as_ref();
            let mut profiles = mp::filter(entries, |profile| {
                match (date, filter_string) {
                    (Some(date), Some(string)) => {
                        profile.expiration_date <= date && profile.contains(&string)
                    }
                    (Some(date), _) => profile.expiration_date <= date,
                    (_, Some(string)) => profile.contains(&string),
                    (_, _) => true,
                }
            });
            profiles.sort_by(|a, b| a.creation_date.cmp(&b.creation_date));
            (total, profiles)
        })
        .and_then(|(total, profiles)| {
            if profiles.is_empty() {
                Ok(println!("Nothing found"))
            } else {
                for profile in &profiles {
                    println!("\n{}", profile.description());
                }
                Ok(println!("\nFound {} of {}", profiles.len(), total))
            }
        })
}

fn show_uuid(uuid: String, directory: Option<PathBuf>) -> Result<(), cli::Error> {
    mp::with_directory(directory)
        .and_then(|directory| {
            mp::find_by_uuid(&directory, &uuid)
                .and_then(|profile| mp::show(&profile.path).map(|xml| println!("{}", xml)))
        })
        .map_err(|err| err.into())
}

fn show_file(path: PathBuf) -> Result<(), cli::Error> {
    mp::show(&path)
        .map(|xml| println!("{}", xml))
        .map_err(|err| err.into())
}

fn remove(uuid: String, directory: Option<PathBuf>) -> Result<(), cli::Error> {
    mp::with_directory(directory)
        .and_then(|directory| {
            mp::find_by_uuid(&directory, &uuid)
                .and_then(|profile| mp::remove(&profile.path))
                .map(|_| println!("'{}' was removed", uuid))
        })
        .map_err(|err| err.into())
}

fn clean(directory: Option<PathBuf>) -> Result<(), cli::Error> {

    fn concat(mut s1: String, s2: String) -> String {
        s1.push_str(&s2);
        s1
    }

    mp::with_directory(directory)
        .and_then(|dir| mp::entries(&dir).map(|entries| entries.collect::<Vec<_>>()))
        .map_err(|err| err.into())
        .map(|entries| {
            let date = UTC::now();
            mp::filter(entries, |profile| profile.expiration_date <= date)
        })
        .and_then(|profiles| {
            if profiles.is_empty() {
                Ok(println!("All provisioning profiles are valid"))
            } else {
                profiles.iter()
                    .map(|profile| {
                        std::fs::remove_file(&profile.path)
                            .map(|_| format!("'{}' was removed\n", profile.uuid))
                            .map_err(|err| format!("'{}' {}\n", profile.uuid, err))
                    })
                    .fold(Ok(String::new()), |acc, s| {
                        match (acc, s) {
                            (Ok(acc), Ok(s)) => Ok(concat(acc, s)),
                            (Err(acc), Ok(s)) => Err(concat(acc, s)),
                            (Err(acc), Err(s)) => Err(concat(acc, s)),
                            (Ok(acc), Err(s)) => Err(concat(acc, s)),
                        }
                    })
                    .map(|s| println!("{}", s))
                    .map_err(|err| cli::Error::Custom(err))
            }
        })
}
