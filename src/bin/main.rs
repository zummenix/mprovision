
extern crate mprovision;
extern crate docopt;
#[macro_use(version)]
extern crate version;

use std::io::{self, Write};
use std::process;
use docopt::Docopt;

const USAGE: &'static str = "
mprovision
A tool that helps iOS developers to manage mobileprovision files.

Usage:
  mprovision search <text> [<directory>]
  mprovision remove <uuid> [<directory>]
  mprovision (-h | --help)
  mprovision --version

Options:
  -h --help     Show this help message.
  --version     Show version.
";

fn main() {
    let args = Docopt::new(USAGE)
        .and_then(|d| d.options_first(true)
                       .version(Some(format!("mprovision {}", version!())))
                       .parse())
        .unwrap_or_else(|e| e.exit());

    if let Some(cmd) = Command::from_args(&args) {
        let result = match cmd {
            Command::Search => search(&args),
            Command::Remove => remove(&args),
        };
        match result {
            Ok(_) => process::exit(0),
            Err(e) => {
                writeln!(&mut io::stderr(), "{}", e).unwrap();
                process::exit(1);
            }
        }
    }
}

enum Command {
    Search,
    Remove,
}

impl Command {
    fn from_args(args: &::docopt::ArgvMap) -> Option<Command> {
        if args.get_bool("search") {
            Some(Command::Search)
        } else if args.get_bool("remove") {
            Some(Command::Remove)
        } else {
            None
        }
    }
}

fn search(args: &::docopt::ArgvMap) -> Result<(), String> {
    let text = args.get_str("<text>");
    if text.is_empty() {
        return Err("<text> should not be empty.".to_owned());
    }
    let dir_name = args.get_str("<directory>");
    let dir_name = if dir_name.is_empty() { None } else { Some(dir_name.as_ref()) };

    mprovision::with_path(dir_name, |path| mprovision::search(path, text))
        .and_then(|info| {
            if info.profiles.len() == 0 {
                println!("Nothing found for '{}'", text);
            } else {
                println!("Found {} of {} profiles.\n", info.profiles.len(), info.total);
                for profile in &info.profiles {
                    println!("{}\n", profile.description());
                }
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
    let dir_name = args.get_str("<directory>");
    let dir_name = if dir_name.is_empty() { None } else { Some(dir_name.as_ref()) };

    mprovision::with_path(dir_name, |path| mprovision::remove(path, uuid))
        .and_then(|_| Ok(println!("Profile '{}' was removed.", uuid)))
        .map_err(|e| e.to_string())
}
