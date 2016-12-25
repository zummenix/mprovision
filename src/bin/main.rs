
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
            Command::ShowUuid(uuid, directory) => {
                mp::with_directory(directory)
                    .and_then(|directory| {
                        mp::find_by_uuid(&directory, &uuid).and_then(|profile| {
                            mp::show(&profile.path).map(|xml| println!("{}", xml))
                        })
                    })
                    .map_err(|err| err.into())
            }
            Command::ShowPath(file_path) => {
                mp::show(&file_path)
                    .map(|xml| println!("{}", xml))
                    .map_err(|err| err.into())
            }
            Command::RemoveUuid(uuid, directory) => {
                mp::with_directory(directory)
                    .and_then(|directory| {
                        mp::find_by_uuid(&directory, &uuid)
                            .and_then(|profile| mp::remove(&profile.path))
                            .map(|_| println!("'{}' was removed", uuid))
                    })
                    .map_err(|err| err.into())
            }
            Command::RemovePath(file_path) => {
                mp::remove(&file_path)
                    .map(|_| println!("'{}' was removed", file_path.display()))
                    .map_err(|err| err.into())
            }
            Command::Cleanup(directory) => {
                let info = mp::with_directory(directory)
                    .and_then(|directory| mp::expired_profiles(&directory, UTC::now()))?;
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
