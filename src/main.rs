
extern crate mprovision;
extern crate docopt;

use std::io::{self, Write};
use std::process;
use docopt::Docopt;

const USAGE: &'static str = "
mprovision
A tool that helps iOS developers to manage mobileprovision files.

Usage:
  mprovision search <text> [<directory>]
  mprovision (-h | --help)
  mprovision --version

Options:
  -h --help     Show this help message.
  --version     Show version.
";

fn main() {
    let args = Docopt::new(USAGE)
                            .and_then(|d| d.options_first(true)
                                           .version(Some(version()))
                                           .parse())
                            .unwrap_or_else(|e| e.exit());

    if args.get_bool("search") {
        let text = args.get_str("<text>");
        if text.is_empty() {
            writeln!(&mut io::stderr(), "{}", "<text> should not be empty.").unwrap();
            process::exit(1);
        }
        let dir_name = args.get_str("<directory>");
        let dir_name = if dir_name.is_empty() { None } else { Some(dir_name) };
        match mprovision::search(dir_name, text) {
            Ok(results) => {
                for result in results {
                    match result {
                        Ok(profile) => println!("{}\n", profile.description()),
                        Err(err) => println!("Error: {}", err),
                    }
                }
            },
            Err(err) => {
                writeln!(&mut io::stderr(), "Error: {}", err).unwrap();
                process::exit(1);
            },
        }
    }
}

fn version() -> String {
    let v = (
        option_env!("CARGO_PKG_VERSION_MAJOR"),
        option_env!("CARGO_PKG_VERSION_MINOR"),
        option_env!("CARGO_PKG_VERSION_PATCH"),
    );
    if let (Some(major), Some(minor), Some(patch)) = v {
        format!("mprovision {}.{}.{}", major, minor, patch)
    } else {
        "N/A".to_owned()
    }
}
