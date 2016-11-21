use clap::{self, Arg, App, SubCommand};
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
}
