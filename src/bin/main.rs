extern crate clap;
#[cfg(test)]
#[macro_use(expect)]
extern crate expectest;
extern crate mprovision;
#[macro_use]
extern crate structopt;

use cli::Command;
use mprovision as mp;
use std::env;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

mod cli;

fn main() {
    if let Err(e) = cli::parse(env::args()).and_then(run) {
        e.exit();
    }
}

fn run(command: cli::Command) -> Result<(), cli::Error> {
    match command {
        Command::List(cli::ListParams {
            text,
            expire_in_days,
            directory,
        }) => list(text, expire_in_days, directory),
        Command::ShowUuid(cli::ShowUuidParams { uuid, directory }) => show_uuid(uuid, directory),
        Command::ShowFile(cli::ShowFileParams { file }) => show_file(file),
        Command::Remove(cli::RemoveParams { ids, directory }) => remove(ids, directory),
        Command::Clean(cli::CleanParams { directory }) => clean(directory),
    }
}

fn list(
    text: Option<String>,
    expires_in_days: Option<i64>,
    directory: Option<PathBuf>,
) -> Result<(), cli::Error> {
    mp::with_directory(directory)
        .and_then(|dir| mp::entries(&dir).map(|entries| entries.collect::<Vec<_>>()))
        .map_err(|err| err.into())
        .map(|entries| {
            let total = entries.len();
            let date = expires_in_days
                .map(|days| SystemTime::now() + Duration::from_secs(days as u64 * 24 * 60 * 60));
            let filter_string = text.as_ref();
            let mut profiles = mp::filter(entries, |profile| match (date, filter_string) {
                (Some(date), Some(string)) => {
                    profile.info.expiration_date <= date && profile.info.contains(string)
                }
                (Some(date), _) => profile.info.expiration_date <= date,
                (_, Some(string)) => profile.info.contains(string),
                (_, _) => true,
            });
            profiles.sort_by(|a, b| a.info.creation_date.cmp(&b.info.creation_date));
            (total, profiles)
        }).and_then(|(total, profiles)| {
            if profiles.is_empty() {
                Ok(println!("Nothing found"))
            } else {
                for profile in &profiles {
                    println!("\n{}", profile.info.description());
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
        }).map_err(|err| err.into())
}

fn show_file(path: PathBuf) -> Result<(), cli::Error> {
    mp::show(&path)
        .map(|xml| println!("{}", xml))
        .map_err(|err| err.into())
}

fn remove(ids: Vec<String>, directory: Option<PathBuf>) -> Result<(), cli::Error> {
    mp::with_directory(directory)
        .and_then(|directory| {
            mp::find_by_ids(&directory, ids).and_then(|profiles| {
                for profile in profiles {
                    match mp::remove(&profile.path) {
                        Ok(_) => println!("\nRemoved: {}", profile.info.description()),
                        Err(_) => println!("\nError while removing '{}'", profile.info.uuid),
                    }
                }
                Ok(())
            })
        }).map_err(|err| err.into())
}

fn clean(directory: Option<PathBuf>) -> Result<(), cli::Error> {
    fn concat(mut s1: String, s2: String) -> String {
        s1.push_str(&s2);
        s1
    }

    let date = SystemTime::now();
    mp::with_directory(directory)
        .and_then(|dir| mp::entries(&dir).map(|entries| entries.collect::<Vec<_>>()))
        .map_err(|err| err.into())
        .map(|entries| mp::filter(entries, |profile| profile.info.expiration_date <= date))
        .and_then(|profiles| {
            if profiles.is_empty() {
                Ok(println!("All provisioning profiles are valid"))
            } else {
                profiles
                    .iter()
                    .map(|profile| {
                        std::fs::remove_file(&profile.path)
                            .map(|_| format!("'{}' was removed\n", profile.info.uuid))
                            .map_err(|err| format!("'{}' {}\n", profile.info.uuid, err))
                    }).fold(Ok(String::new()), |acc, s| match (acc, s) {
                        (Ok(acc), Ok(s)) => Ok(concat(acc, s)),
                        (Ok(acc), Err(s)) | (Err(acc), Ok(s)) | (Err(acc), Err(s)) => {
                            Err(concat(acc, s))
                        }
                    }).map(|s| println!("{}", s))
                    .map_err(cli::Error::Custom)
            }
        })
}
