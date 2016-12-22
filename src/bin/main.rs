
extern crate mprovision;
#[macro_use(version)]
extern crate version;
extern crate chrono;
#[cfg(test)]
#[macro_use(expect)]
extern crate expectest;
extern crate clap;

use mprovision as mp;
use cli::Command;
use std::env;
use chrono::*;
use std::error::Error;

mod cli;

fn main() {
    let result = cli::parse(env::args()).and_then(|command| {
        match command {
            Command::List(list_params) => Ok(()),
            Command::ShowUuid(uuid, directory) => Ok(()),
            Command::ShowPath(file_path) => Ok(()),
            Command::RemoveUuid(uuid, directory) => {
                mp::with_dir(directory, |dir| mp::remove(dir, &uuid))?;
                Ok(println!("'{}' was removed", uuid))
            }
            Command::RemovePath(file_path) => {
                if file_path.extension().map(|ext| ext == "mobileprovision").unwrap_or(false) {
                    if file_path.exists() {
                        std::fs::remove_file(&file_path)
                            .map_err(|err| cli::Error::Custom(err.description().to_string()))?;
                        Ok(println!("'{}' was removed", file_path.display()))
                    } else {
                        Err(cli::Error::Custom(format!("'{}' doesn't exist", file_path.display())))
                    }
                } else {
                    Err(cli::Error::Custom(format!("'{}' doesn't have 'mobileprovision' \
                                                    extension",
                                                   file_path.display())))
                }
            }
            Command::Cleanup(directory) => {
                let info = mp::with_dir(directory, |dir| mp::expired_profiles(dir, UTC::now()))?;
                if info.profiles.is_empty() {
                    Ok(println!("All provisioning profiles are valid"))
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
                        Err(cli::Error::Custom("There were some errors while removing \
                                                provisioning profiles"
                            .into()))
                    } else {
                        Ok(())
                    }
                }
            }
        }
    });
    if let Err(e) = result {
        e.exit();
    }
}

// fn search(args: &::docopt::ArgvMap) -> CliResult {
//    let text = args.get_str("<text>");
//    if text.is_empty() {
//        return Err(CliError::EmptyParameter("<text>"));
//    }
//
//    let info = mp::with_dir(directory(args), |dir| mp::search(dir, text))?;
//    if info.profiles.is_empty() {
//        println!("Nothing found for '{}'", text);
//    } else {
//        for profile in &info.profiles {
//            println!("\n{}", profile.description());
//        }
//        println!("\nFound {} of {}", info.profiles.len(), info.total);
//    }
//    Ok(())
// }
//
// fn show_xml(args: &::docopt::ArgvMap) -> CliResult {
//    let uuid = args.get_str("<uuid>");
//    if uuid.is_empty() {
//        return Err(CliError::EmptyParameter("<uuid>"));
//    }
//
//    let xml = mp::with_dir(directory(args), |dir| mp::xml(dir, uuid))?;
//    println!("{}", xml);
//    Ok(())
// }
//
// fn show_expired(args: &::docopt::ArgvMap) -> CliResult {
//    use chrono::*;
//
//    let mut date = UTC::now();
//    if let Some(days) = args.get_str("<days>").parse::<i64>().ok() {
//        if days < 0 || days > 365 {
//            return Err(CliError::Custom("<days> should be between 0 and 365".to_string()));
//        }
//        date = date + Duration::days(days);
//    }
//
//    let info = mp::with_dir(directory(args), |dir| mp::expired_profiles(dir, date))?;
//    if info.profiles.is_empty() {
//        println!("All provisioning profiles are valid");
//    } else {
//        for profile in &info.profiles {
//            println!("\n{}", profile.description());
//        }
//        println!("\nFound {} of {}", info.profiles.len(), info.total);
//    }
//    Ok(())
// }
