use docopt::{self, Docopt, ArgvMap};
use std::io::{self, Write};
use std::process;
use std::path::Path;
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

#[derive(Debug)]
pub enum Command {
    List,
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
    Ok(Command::List)
}

#[cfg(test)]
mod tests {
    use super::*;
    use expectest::prelude::*;

    #[test]
    fn list_command() {
        expect!(parse(&["mprovision", "list"])).to(be_ok());
        expect!(parse(&["mprovision", "list", "."])).to(be_ok());
        expect!(parse(&["mprovision", "list", "--filter abc"])).to(be_ok());
        expect!(parse(&["mprovision", "list", "--filter abc", "."])).to(be_ok());
        expect!(parse(&["mprovision", "list", "--expires-in-days 0"])).to(be_ok());
        expect!(parse(&["mprovision", "list", "--expires-in-days 0", "."])).to(be_ok());
    }
}
