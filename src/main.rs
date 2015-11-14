
extern crate mprovision;
extern crate clap;

use clap::{Arg, App, AppSettings, SubCommand};

fn main() {
    let subcommands = vec![
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
