
extern crate mprovision;
extern crate docopt;
#[macro_use(version)]
extern crate version;
extern crate chrono;

use std::io::{self, Write};
use std::process;
use std::path::Path;
use std::error;
use std::fmt;
use docopt::Docopt;
use mprovision as mp;

const USAGE: &'static str = "
mprovision
A tool that helps iOS developers to manage mobileprovision files.

Usage:
  mprovision search <text> [<directory>]
  mprovision remove <uuid> [<directory>]
  mprovision show-xml <uuid> [<directory>]
  mprovision show-expired --days <days> [<directory>]
  mprovision remove-expired [<directory>]
  mprovision (-h | --help)
  mprovision --version

Options:
  -h --help     Show this help message.
  --version     Show version.
";

fn main() {
    let args = Docopt::new(USAGE)
        .and_then(|d| d.version(Some(format!("mprovision {}", version!()))).parse())
        .unwrap_or_else(|e| e.exit());

    match Command::from_args(&args).execute() {
        Ok(_) => process::exit(0),
        Err(e) => {
            writeln!(&mut io::stderr(), "{}", e).unwrap();
            process::exit(1);
        }
    }
}

struct Command<'a> {
    args: &'a ::docopt::ArgvMap,
}

impl<'a> Command<'a> {
    fn from_args(args: &'a ::docopt::ArgvMap) -> Self {
        Command { args: args }
    }

    fn execute(&self) -> CliResult {
        if self.args.get_bool("search") {
            search(self.args)
        } else if self.args.get_bool("remove") {
            remove(self.args)
        } else if self.args.get_bool("show-xml") {
            show_xml(self.args)
        } else if self.args.get_bool("show-expired") {
            show_expired(self.args)
        } else if self.args.get_bool("remove-expired") {
            remove_expired(self.args)
        } else {
            Err(CliError::Custom("Command is not implemented".to_string()))
        }
    }
}

type CliResult = std::result::Result<(), CliError>;

#[derive(Debug)]
enum CliError {
    Lib(mp::Error),
    Custom(String),
    EmptyParameter(&'static str),
}

impl error::Error for CliError {
    fn description(&self) -> &str {
        match *self {
            CliError::Lib(ref e) => e.description(),
            CliError::Custom(ref e) => e,
            CliError::EmptyParameter(_) => "parameter is empty",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        if let CliError::Lib(ref e) = *self {
            Some(e)
        } else {
            None
        }
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CliError::Lib(ref e) => error::Error::description(e).fmt(f),
            CliError::Custom(ref e) => e.fmt(f),
            CliError::EmptyParameter(p) => write!(f, "{} should not be empty.", p),
        }
    }
}

impl From<mp::Error> for CliError {
    fn from(e: mp::Error) -> Self {
        CliError::Lib(e)
    }
}

fn search(args: &::docopt::ArgvMap) -> CliResult {
    let text = args.get_str("<text>");
    if text.is_empty() {
        return Err(CliError::EmptyParameter("<text>"));
    }

    let info = try!(mp::with_dir(directory(args), |dir| mp::search(dir, text)));
    if info.profiles.is_empty() {
        println!("Nothing found for '{}'", text);
    } else {
        for profile in &info.profiles {
            println!("\n{}", profile.description());
        }
        println!("\nFound {} of {}", info.profiles.len(), info.total);
    }
    Ok(())
}

fn remove(args: &::docopt::ArgvMap) -> CliResult {
    let uuid = args.get_str("<uuid>");
    if uuid.is_empty() {
        return Err(CliError::EmptyParameter("<uuid>"));
    }

    let _ = try!(mp::with_dir(directory(args), |dir| mp::remove(dir, uuid)));
    println!("'{}' was removed", uuid);
    Ok(())
}

fn show_xml(args: &::docopt::ArgvMap) -> CliResult {
    let uuid = args.get_str("<uuid>");
    if uuid.is_empty() {
        return Err(CliError::EmptyParameter("<uuid>"));
    }

    let xml = try!(mp::with_dir(directory(args), |dir| mp::xml(dir, uuid)));
    println!("{}", xml);
    Ok(())
}

fn show_expired(args: &::docopt::ArgvMap) -> CliResult {
    use chrono::*;

    let mut date = UTC::now();
    if let Some(days) = args.get_str("<days>").parse::<i64>().ok() {
        if days < 0 || days > 365 {
            return Err(CliError::Custom("<days> should be between 0 and 365".to_string()));
        }
        date = date + Duration::days(days);
    }

    let info = try!(mp::with_dir(directory(args), |dir| mp::expired_profiles(dir, date)));
    if info.profiles.is_empty() {
        println!("All provisioning profiles are valid");
    } else {
        for profile in &info.profiles {
            println!("\n{}", profile.description());
        }
        println!("\nFound {} of {}", info.profiles.len(), info.total);
    }
    Ok(())
}

fn remove_expired(args: &::docopt::ArgvMap) -> CliResult {
    use chrono::*;

    let info = try!(mp::with_dir(directory(args), |dir| mp::expired_profiles(dir, UTC::now())));
    if info.profiles.is_empty() {
        println!("All provisioning profiles are valid");
    } else {
        let mut errors_counter = 0;
        let mut removals_counter = 0;
        for profile in info.profiles {
            match std::fs::remove_file(&profile.path) {
                Ok(_) => {
                    removals_counter += 1;
                    println!("'{}' was removed", profile.uuid)
                }
                Err(e) => {
                    errors_counter += 1;
                    println!("Error while trying to remove '{}'", profile.uuid);
                    println!("{}", e);
                }
            }
        }
        println!("Removed {} of {}", removals_counter, info.total);
        if errors_counter > 0 {
            println!("Errors: {}", errors_counter);
        }
    }
    Ok(())
}

fn directory(args: &::docopt::ArgvMap) -> Option<&Path> {
    let dir_name = args.get_str("<directory>");
    if dir_name.is_empty() {
        None
    } else {
        Some(dir_name.as_ref())
    }
}
