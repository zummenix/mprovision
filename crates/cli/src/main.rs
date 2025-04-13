use cli::Command;
use mprovision as mp;
use profile_formatters::{format_multiline, format_oneline};
use std::path::{Path, PathBuf};
use std::result;
use std::time::{Duration, SystemTime};
use std::{
    fs,
    io::{self, Read, Write},
};
use zip::ZipArchive;

mod cli;
mod profile_formatters;

type Result = result::Result<(), main_error::MainError>;

fn main() -> Result {
    match cli::run() {
        Command::List(cli::ListParams {
            text,
            expire_in_days,
            directory,
            oneline,
        }) => list(
            &text,
            expire_in_days,
            mp::dir_or_default(directory)?,
            oneline,
        ),
        Command::ShowUuid(cli::ShowUuidParams { uuid, directory }) => {
            let dir = mp::dir_or_default(directory)?;
            let profile = mp::filter_dir(&dir, |profile| profile.info.uuid == uuid)?
                .into_iter()
                .next()
                .ok_or_else(|| format!("Failed to find provisioning profile for '{}'", uuid))?;
            show_file(&profile.path)
        }
        Command::ShowFile(cli::ShowFileParams { file }) => show_file(&file),
        Command::Remove(cli::RemoveParams {
            ids,
            directory,
            permanently,
        }) => {
            let dir = mp::dir_or_default(directory)?;
            let profiles = mp::filter_dir(&dir, |profile| profile.info.has_ids(&ids))?;
            remove_profiles(&profiles, permanently)
        }
        Command::Clean(cli::CleanParams {
            directory,
            permanently,
        }) => {
            let dir = mp::dir_or_default(directory)?;
            let date = SystemTime::now();
            let profiles = mp::filter_dir(&dir, |profile| profile.info.expiration_date <= date)?;
            remove_profiles(&profiles, permanently)
        }
        Command::Extract(cli::ExtractParams {
            source,
            destination,
        }) => extract(source, destination),
    }
}

fn list(
    text: &Option<String>,
    expires_in_days: Option<u64>,
    dir: PathBuf,
    oneline: bool,
) -> Result {
    let date =
        expires_in_days.map(|days| SystemTime::now() + Duration::from_secs(days * 24 * 60 * 60));
    let filter_string = text.as_ref();
    let mut profiles = mp::filter_dir(&dir, |profile| match (date, filter_string) {
        (Some(date), Some(string)) => {
            profile.info.expiration_date <= date && profile.info.contains(string)
        }
        (Some(date), _) => profile.info.expiration_date <= date,
        (_, Some(string)) => profile.info.contains(string),
        (_, _) => true,
    })?;
    profiles.sort_by(|a, b| a.info.creation_date.cmp(&b.info.creation_date));
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let format = if oneline {
        format_oneline
    } else {
        format_multiline
    };
    for (i, profile) in profiles.iter().enumerate() {
        let separator = if oneline || i + 1 == profiles.len() {
            ""
        } else {
            "\n"
        };
        writeln!(&mut stdout, "{}{}", format(profile)?, separator)?;
    }
    Ok(())
}

fn show_file(path: &Path) -> Result {
    let xml = mp::show(path)?;
    writeln!(io::stdout(), "{}", xml)?;
    Ok(())
}

fn extract(source: PathBuf, destination: PathBuf) -> Result {
    if !destination.exists() {
        fs::create_dir_all(&destination)?;
    }
    if !destination.is_dir() {
        return Err(format!("Destination '{}' is not a directory", destination.display()).into());
    }
    let mut archive = ZipArchive::new(fs::File::open(source)?)?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let Some(path) = file.enclosed_name().map(|name| name.to_path_buf()) else { continue };
        if !mp::is_mobileprovision(&path) {
            continue;
        }
        let mut buf: Vec<u8> = Vec::with_capacity(file.size() as usize);
        file.read_to_end(&mut buf)?;
        let info = mp::profile::Info::from_xml_data(&buf)
            .ok_or_else(|| format!("Failed to decode {}", path.display()))?;
        let file_name = format!("{}.mobileprovision", info.uuid);
        let mut buf_cursor = io::Cursor::new(buf);
        let outpath = destination.join(file_name);
        let mut outfile = fs::File::create(outpath)?;
        io::copy(&mut buf_cursor, &mut outfile)?;
    }
    Ok(())
}

fn remove_profiles(profiles: &[mp::profile::Profile], permanently: bool) -> Result {
    let mut errors_exist = false;
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    for (i, profile) in profiles.iter().enumerate() {
        match remove(&profile.path, permanently) {
            Ok(()) => {
                let separator = if i + 1 == profiles.len() { "" } else { "\n" };
                writeln!(&mut stdout, "{}{}", format_multiline(profile)?, separator)?
            }
            Err(err) => {
                errors_exist = true;
                writeln!(io::stderr(), "{}", err)?
            }
        }
    }
    if errors_exist {
        // Don't need to show anything – all errors are already printed.
        Err(String::new().into())
    } else {
        Ok(())
    }
}

fn remove(file_path: &Path, permanently: bool) -> result::Result<(), Box<dyn std::error::Error>> {
    if permanently {
        std::fs::remove_file(file_path)?;
    } else {
        trash::delete(file_path)?;
    }
    Ok(())
}
