
extern crate mprovision;
extern crate docopt;
#[macro_use(version)]
extern crate version;
extern crate chrono;

use std::io::{self, Write};
use std::process;
use std::path::Path;
use docopt::Docopt;

const USAGE: &'static str = "
mprovision
A tool that helps iOS developers to manage mobileprovision files.

Usage:
  mprovision search <text> [<directory>]
  mprovision remove <uuid> [<directory>]
  mprovision show-xml <uuid> [<directory>]
  mprovision show-expired [<directory>]
  mprovision remove-expired [<directory>]
  mprovision (-h | --help)
  mprovision --version

Options:
  -h --help     Show this help message.
  --version     Show version.
";

fn main() {
    let args = Docopt::new(USAGE)
                   .and_then(|d| {
                       d.options_first(true)
                        .version(Some(format!("mprovision {}", version!())))
                        .parse()
                   })
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

    fn execute(&self) -> Result<(), String> {
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
            Err("Command is not implemented".to_string())
        }
    }
}

fn search(args: &::docopt::ArgvMap) -> Result<(), String> {
    let text = args.get_str("<text>");
    if text.is_empty() {
        return Err("<text> should not be empty.".to_owned());
    }

    mprovision::with_path(directory(args), |path| mprovision::search(path, text))
        .and_then(|info| {
            if info.profiles.is_empty() {
                println!("Nothing found for '{}'", text);
            } else {
                for profile in &info.profiles {
                    println!("{}\n", profile.description());
                }
                println!("Found {} of {}", info.profiles.len(), info.total);
            }
            Ok(())
        })
        .map_err(|e| e.to_string())
}

fn remove(args: &::docopt::ArgvMap) -> Result<(), String> {
    let uuid = args.get_str("<uuid>");
    if uuid.is_empty() {
        return Err("<uuid> should not be empty.".to_owned());
    }

    mprovision::with_path(directory(args), |path| mprovision::remove(path, uuid))
        .and_then(|_| Ok(println!("'{}' was removed", uuid)))
        .map_err(|e| e.to_string())
}

fn show_xml(args: &::docopt::ArgvMap) -> Result<(), String> {
    let uuid = args.get_str("<uuid>");
    if uuid.is_empty() {
        return Err("<uuid> should not be empty.".to_owned());
    }

    mprovision::with_path(directory(args), |path| mprovision::xml(path, uuid))
        .and_then(|xml| Ok(println!("{}", xml)))
        .map_err(|e| e.to_string())
}

fn show_expired(args: &::docopt::ArgvMap) -> Result<(), String> {
    use chrono::*;

    mprovision::with_path(directory(args),
                          |path| mprovision::expired_profiles(path, UTC::now()))
        .and_then(|info| {
            if info.profiles.is_empty() {
                println!("All provisioning profiles are valid");
            } else {
                for profile in &info.profiles {
                    println!("{}\n", profile.description());
                }
                println!("Found {} of {}", info.profiles.len(), info.total);
            }
            Ok(())
        })
        .map_err(|e| e.to_string())
}

fn remove_expired(args: &::docopt::ArgvMap) -> Result<(), String> {
    use chrono::*;

    mprovision::with_path(directory(args),
                          |path| mprovision::expired_profiles(path, UTC::now()))
        .and_then(|info| {
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
        })
        .map_err(|e| e.to_string())
}

fn directory(args: &::docopt::ArgvMap) -> Option<&Path> {
    let dir_name = args.get_str("<directory>");
    if dir_name.is_empty() {
        None
    } else {
        Some(dir_name.as_ref())
    }
}
