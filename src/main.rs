
extern crate mprovision;
extern crate clap;

use std::fs;
use clap::{Arg, App, AppSettings, SubCommand};

fn main() {
    let subcommands = vec![
        SubCommand::with_name("count")
            .about("Counts provisioning profiles in a directory.")
            .arg(Arg::with_name("DIRECTORY")
                .help("Directory where to count provisioning profiles.")
                .required(false)),
        SubCommand::with_name("search")
            .about("Searches provisioning profile for provided text.")
            .arg(Arg::with_name("TEXT")
                .help("Text to search in provisioning profile.")
                .required(true))
            .arg(Arg::with_name("DIRECTORY")
                .help("Directory where to search provisioning profiles.")
                .required(false)),
    ];

    let matches = App::new("mprovision")
                      .setting(AppSettings::SubcommandRequired)
                      .version("0.1.0")
                      .about("A tool that helps iOS developers to manage mobileprovision files.")
                      .subcommands(subcommands)
                      .get_matches();

    if let Some(matches) = matches.subcommand_matches("count") {
        handle_count_subcommand(matches)
    }
    if let Some(matches) = matches.subcommand_matches("search") {
        let s = matches.value_of("TEXT").unwrap();
        match mprovision::search(matches.value_of("DIRECTORY"), s) {
            Ok(results) => {
                for result in results {
                    match result {
                        Ok(profile) => println!("{}\n", profile.description()),
                        Err(err) => println!("Error: {}", err),
                    }
                }
            },
            Err(err) => println!("Error: {}", err),
        }
    }
}

fn handle_count_subcommand(matches: &clap::ArgMatches) {

    fn show(result: mprovision::Result<Box<Iterator<Item = fs::DirEntry>>>) {
        match result {
            Ok(files) => println!("Found {} files.", files.count()),
            Err(err) => println!("Error: {}", err),
        }
    }

    if let Some(directory) = matches.value_of("DIRECTORY") {
        show(mprovision::files(directory));
    } else {
        match mprovision::directory() {
            Ok(directory) => show(mprovision::files(directory)),
            Err(err) => println!("Error: {}", err),
        }
    }
}
