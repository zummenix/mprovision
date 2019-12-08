use crate::cli::Command;
use mprovision as mp;
use std::env;
use std::io::{self, Write};
use std::path::Path;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

mod cli;

type Result = std::result::Result<(), cli::Error>;

fn main() {
    if let Err(e) = cli::parse(env::args()).and_then(run) {
        e.exit();
    }
}

fn run(command: cli::Command) -> Result {
    match command {
        Command::List(cli::ListParams {
            text,
            expire_in_days,
            directory,
        }) => list(&text, expire_in_days, directory),
        Command::ShowUuid(cli::ShowUuidParams { uuid, directory }) => show_uuid(&uuid, directory),
        Command::ShowFile(cli::ShowFileParams { file }) => show_file(&file),
        Command::Remove(cli::RemoveParams { ids, directory }) => remove(&ids, directory),
        Command::Clean(cli::CleanParams { directory }) => clean(directory),
    }
}

fn list(text: &Option<String>, expires_in_days: Option<u64>, directory: Option<PathBuf>) -> Result {
    let dir = mp::with_directory(directory)?;
    let entries = mp::entries(&dir).map(std::iter::Iterator::collect)?;
    let date =
        expires_in_days.map(|days| SystemTime::now() + Duration::from_secs(days * 24 * 60 * 60));
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
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    for (i, profile) in profiles.iter().enumerate() {
        let separator = if i + 1 == profiles.len() { "" } else { "\n" };
        writeln!(&mut stdout, "{}{}", profile.info.description(), separator)?;
    }
    Ok(())
}

fn show_uuid(uuid: &str, directory: Option<PathBuf>) -> Result {
    let dir = mp::with_directory(directory)?;
    let profile = mp::find_by_uuid(&dir, &uuid)?;
    show_file(&profile.path)
}

fn show_file(path: &Path) -> Result {
    let xml = mp::show(&path)?;
    writeln!(io::stdout(), "{}", xml)?;
    Ok(())
}

fn remove(ids: &[String], directory: Option<PathBuf>) -> Result {
    let dir = mp::with_directory(directory)?;
    let profiles = mp::find_by_ids(&dir, &ids)?;
    remove_profiles(&profiles)
}

fn clean(directory: Option<PathBuf>) -> Result {
    let date = SystemTime::now();
    let dir = mp::with_directory(directory)?;
    let entries = mp::entries(&dir).map(std::iter::Iterator::collect)?;
    let profiles = mp::filter(entries, |profile| profile.info.expiration_date <= date);
    remove_profiles(&profiles)
}

fn remove_profiles(profiles: &[mp::Profile]) -> Result {
    let mut errors_exist = false;
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    for (i, profile) in profiles.iter().enumerate() {
        match mp::remove(&profile.path) {
            Ok(()) => {
                let separator = if i + 1 == profiles.len() { "" } else { "\n" };
                writeln!(&mut stdout, "{}{}", profile.info.description(), separator)?
            }
            Err(err) => {
                errors_exist = true;
                writeln!(io::stderr(), "{}", err)?
            }
        }
    }
    if errors_exist {
        // Don't need to show anything – all errors are already printed.
        Err(cli::Error::Custom(String::new()))
    } else {
        Ok(())
    }
}
