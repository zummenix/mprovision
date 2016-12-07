use clap::{self, Arg, App, SubCommand, AppSettings};
use std::io::{self, Write};
use std::process;
use std::path::PathBuf;
use std::error;
use std::result;
use std::fmt;
use mprovision as mp;

#[derive(Debug, PartialEq)]
pub enum Command {
    List(ListParams),
    ShowUuid(String, Option<PathBuf>),
    ShowPath(PathBuf),
    RemoveUuid(String, Option<PathBuf>),
    RemovePath(PathBuf),
    Cleanup(Option<PathBuf>),
}

#[derive(Debug, Default, PartialEq)]
pub struct ListParams {
    pub filter: Option<String>,
    pub expires_in_days: Option<i64>,
    pub directory: Option<PathBuf>,
}

pub type Result = result::Result<Command, Error>;

#[derive(Debug)]
pub enum Error {
    Lib(mp::Error),
    Clap(clap::Error),
    Custom(String),
}

impl Error {
    pub fn exit(&self) -> ! {
        match *self {
            Error::Clap(ref e) => e.exit(),
            _ => {
                writeln!(&mut io::stderr(), "{}", self).unwrap();
                process::exit(1);
            }
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Lib(ref e) => e.description(),
            Error::Clap(ref e) => e.description(),
            Error::Custom(ref e) => e,
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Lib(ref e) => Some(e),
            Error::Clap(ref e) => Some(e),
            Error::Custom(_) => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Lib(ref e) => error::Error::description(e).fmt(f),
            Error::Clap(ref e) => error::Error::description(e).fmt(f),
            Error::Custom(ref e) => e.fmt(f),
        }
    }
}

impl From<mp::Error> for Error {
    fn from(e: mp::Error) -> Self {
        Error::Lib(e)
    }
}

impl From<clap::Error> for Error {
    fn from(e: clap::Error) -> Self {
        Error::Clap(e)
    }
}

pub fn parse<I, S>(args: I) -> Result
    where I: IntoIterator<Item = S>,
          S: Clone,
          ::std::ffi::OsString: From<S>
{
    let app = App::new("mprovision")
        .about("A tool that helps iOS developers to manage mobileprovision files.")
        .subcommand(SubCommand::with_name("list")
            .about("Lists provisioning profiles")
            .display_order(0)
            .setting(AppSettings::DisableVersion)
            .arg(Arg::with_name("TEXT")
                .long("--filter")
                .required(false)
                .empty_values(false)
                .takes_value(true))
            .arg(Arg::with_name("DAYS")
                .long("--expires-in-days")
                .required(false)
                .empty_values(false)
                .takes_value(true))
            .arg(Arg::with_name("DIRECTORY")
                .required(false)
                .empty_values(false)
                .takes_value(true)))
        .subcommand(SubCommand::with_name("show")
            .about("Shows details of a provisioning profile")
            .display_order(1)
            .setting(AppSettings::DisableVersion)
            .arg(Arg::with_name("UUID")
                .long("--uuid")
                .required(true)
                .empty_values(false)
                .takes_value(true)
                .conflicts_with("PATH"))
            .arg(Arg::with_name("DIRECTORY")
                .required(false)
                .empty_values(false)
                .takes_value(true)
                .conflicts_with("PATH"))
            .arg(Arg::with_name("PATH")
                .long("--path")
                .required(true)
                .empty_values(false)
                .takes_value(true)
                .required(true)))
        .subcommand(SubCommand::with_name("remove")
            .about("Removes a provisioning profile")
            .display_order(2)
            .setting(AppSettings::DisableVersion)
            .arg(Arg::with_name("UUID")
                .long("--uuid")
                .required(true)
                .empty_values(false)
                .takes_value(true)
                .conflicts_with("PATH"))
            .arg(Arg::with_name("DIRECTORY")
                .required(false)
                .empty_values(false)
                .takes_value(true)
                .conflicts_with("PATH"))
            .arg(Arg::with_name("PATH")
                .long("--path")
                .required(true)
                .empty_values(false)
                .takes_value(true)
                .required(true)))
        .subcommand(SubCommand::with_name("cleanup")
            .about("Removes expired provisioning profiles")
            .display_order(4)
            .setting(AppSettings::DisableVersion)
            .arg(Arg::with_name("DIRECTORY")
                .required(false)
                .empty_values(false)
                .takes_value(true)));

    let matches = app.get_matches_from_safe(args)?;
    if let Some(list_matches) = matches.subcommand_matches("list") {
        let mut params = ListParams::default();
        params.filter = list_matches.value_of("TEXT").map(|text| text.to_string());
        if let Some(days) = list_matches.value_of("DAYS")
            .and_then(|days| days.parse::<i64>().ok()) {
            if days < 0 || days > 365 {
                return Err(Error::Custom("DAYS should be between 0 and 365".to_string()));
            }
            params.expires_in_days = Some(days);
        }
        params.directory = list_matches.value_of("DIRECTORY").map(|dir| dir.into());
        Ok(Command::List(params))
    } else if let Some(show_matches) = matches.subcommand_matches("show") {
        if let Some(uuid) = show_matches.value_of("UUID").map(|uuid| uuid.to_string()) {
            let directory = show_matches.value_of("DIRECTORY").map(|dir| dir.into());
            Ok(Command::ShowUuid(uuid, directory))
        } else {
            let path = show_matches.value_of("PATH").map(|path| path.into()).unwrap();
            Ok(Command::ShowPath(path))
        }
    } else if let Some(remove_matches) = matches.subcommand_matches("remove") {
        if let Some(uuid) = remove_matches.value_of("UUID").map(|uuid| uuid.to_string()) {
            let directory = remove_matches.value_of("DIRECTORY").map(|dir| dir.into());
            Ok(Command::RemoveUuid(uuid, directory))
        } else {
            let path = remove_matches.value_of("PATH").map(|path| path.into()).unwrap();
            Ok(Command::RemovePath(path))
        }
    } else if let Some(cleanup_matches) = matches.subcommand_matches("cleanup") {
        let directory = cleanup_matches.value_of("DIRECTORY").map(|dir| dir.into());
        Ok(Command::Cleanup(directory))
    } else {
        Err(Error::Custom("Command isn't implemented".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use expectest::prelude::*;

    #[test]
    fn list_command() {
        expect!(parse(&["mprovision", "list"]))
            .to(be_ok().value(Command::List(ListParams::default())));

        expect!(parse(&["mprovision", "list", "."])).to(be_ok().value(Command::List(ListParams {
            filter: None,
            expires_in_days: None,
            directory: Some(".".into()),
        })));

        expect!(parse(&["mprovision", "list", "--filter", "abc"]))
            .to(be_ok().value(Command::List(ListParams {
                filter: Some("abc".to_string()),
                expires_in_days: None,
                directory: None,
            })));

        expect!(parse(&["mprovision", "list", "--filter", ""])).to(be_err());

        expect!(parse(&["mprovision", "list", "--expires-in-days", "3"]))
            .to(be_ok().value(Command::List(ListParams {
                filter: None,
                expires_in_days: Some(3),
                directory: None,
            })));

        expect!(parse(&["mprovision", "list", "--expires-in-days", "-3"])).to(be_err());
        expect!(parse(&["mprovision", "list", "--expires-in-days", "366"])).to(be_err());

        expect!(parse(&["mprovision", "list", "--filter", "abc", "--expires-in-days", "3", "."]))
            .to(be_ok().value(Command::List(ListParams {
                filter: Some("abc".to_string()),
                expires_in_days: Some(3),
                directory: Some(".".into()),
            })));
    }

    #[test]
    fn show_uuid_command() {
        expect!(parse(&["mprovision", "show", "--uuid", "abcd"]))
            .to(be_ok().value(Command::ShowUuid("abcd".to_string(), None)));

        expect!(parse(&["mprovision", "show", "--uuid", "abcd", "."]))
            .to(be_ok().value(Command::ShowUuid("abcd".to_string(), Some(".".into()))));

        expect!(parse(&["mprovision", "show", "."])).to(be_err());
    }

    #[test]
    fn show_path_command() {
        expect!(parse(&["mprovision", "show", "--path", "file.mprovision"]))
            .to(be_ok().value(Command::ShowPath("file.mprovision".into())));

        expect!(parse(&["mprovision", "show", "--path", "file.mprovision", "."])).to(be_err());
    }

    #[test]
    fn remove_uuid_command() {
        expect!(parse(&["mprovision", "remove", "--uuid", "abcd"]))
            .to(be_ok().value(Command::RemoveUuid("abcd".to_string(), None)));

        expect!(parse(&["mprovision", "remove", "--uuid", "abcd", "."]))
            .to(be_ok().value(Command::RemoveUuid("abcd".to_string(), Some(".".into()))));

        expect!(parse(&["mprovision", "remove", "."])).to(be_err());
    }

    #[test]
    fn remove_path_command() {
        expect!(parse(&["mprovision", "remove", "--path", "file.mprovision"]))
            .to(be_ok().value(Command::RemovePath("file.mprovision".into())));

        expect!(parse(&["mprovision", "remove", "--path", "file.mprovision", "."])).to(be_err());
    }

    #[test]
    fn cleanup_command() {
        expect!(parse(&["mprovision", "cleanup"])).to(be_ok().value(Command::Cleanup(None)));

        expect!(parse(&["mprovision", "cleanup", "."]))
            .to(be_ok().value(Command::Cleanup(Some(".".into()))));
    }
}
