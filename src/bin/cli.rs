use docopt::{self, Docopt, ArgvMap};
use std::io::{self, Write};
use std::process;
use std::path::{Path, PathBuf};
use std::error;
use std::result;
use std::fmt;
use mprovision as mp;

const USAGE: &'static str = "
mprovision
A tool that helps iOS developers to manage mobileprovision files.

Usage:
  mprovision list [--filter <text>] [--expires-in-days <days>] [<directory>]
  mprovision (-h | --help)
  mprovision --version

Options:
  -h --help     Show this help message.
  --version     Show version.
";

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
    Docopt(docopt::Error),
    Custom(String),
}

impl Error {
    pub fn exit(&self) -> ! {
        match *self {
            Error::Docopt(ref e) => e.exit(),
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
            Error::Docopt(ref e) => e.description(),
            Error::Custom(ref e) => e,
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Lib(ref e) => Some(e),
            Error::Docopt(ref e) => Some(e),
            Error::Custom(_) => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Lib(ref e) => error::Error::description(e).fmt(f),
            Error::Docopt(ref e) => error::Error::description(e).fmt(f),
            Error::Custom(ref e) => e.fmt(f),
        }
    }
}

impl From<mp::Error> for Error {
    fn from(e: mp::Error) -> Self {
        Error::Lib(e)
    }
}

impl From<docopt::Error> for Error {
    fn from(e: docopt::Error) -> Self {
        Error::Docopt(e)
    }
}

pub fn parse<I, S>(args: I) -> Result
    where I: IntoIterator<Item = S>,
          S: AsRef<str>
{
    let version = format!("mprovision {}", version!());
    let argv_map = Docopt::new(USAGE)?.argv(args).version(Some(version)).parse()?;
    if argv_map.get_bool("list") {
        let mut params = ListParams::default();
        if argv_map.get_bool("--filter") {
            let text = argv_map.get_str("--filter");
            if text.is_empty() {
                return Err(Error::Custom("<text> is empty".to_string()));
            }
            params.filter = Some(text.to_string());
        }
        if argv_map.get_bool("--expires-in-days") {
            if let Some(days) = argv_map.get_str("--expires-in-days").parse::<i64>().ok() {
                if days < 0 || days > 365 {
                    return Err(Error::Custom("<days> should be between 0 and 365".to_string()));
                }
                params.expires_in_days = Some(days);
            }
        }
        params.directory = directory(&argv_map);
        Ok(Command::List(params))
    } else {
        Err(Error::Custom("Command is not implemented".to_string()))
    }
}

fn directory(args: &::docopt::ArgvMap) -> Option<PathBuf> {
    let dir_name = args.get_str("<directory>");
    if dir_name.is_empty() {
        None
    } else {
        Some(dir_name.into())
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

        expect!(parse(&["mprovision", "list", "--filter", ""])).to(be_err());

        expect!(parse(&["mprovision", "list", "."])).to(be_ok().value(Command::List(ListParams {
            filter: None,
            expires_in_days: None,
            directory: Some(".".into()),
        })));
        expect!(parse(&["mprovision", "list", "--filter abc"])).to(be_ok());
        expect!(parse(&["mprovision", "list", "--filter abc", "."])).to(be_ok());
        expect!(parse(&["mprovision", "list", "--expires-in-days 0"])).to(be_ok());
        expect!(parse(&["mprovision", "list", "--expires-in-days 0", "."])).to(be_ok());
    }
}
