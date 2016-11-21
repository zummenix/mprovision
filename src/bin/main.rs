
extern crate mprovision;
extern crate docopt;
#[macro_use(version)]
extern crate version;
extern crate chrono;
#[cfg(test)]
#[macro_use(expect)]
extern crate expectest;
extern crate clap;

use std::io::{self, Write};
use std::process;
use std::path::Path;
use std::error;
use std::fmt;
use docopt::Docopt;
use mprovision as mp;
use cli::{parse, Error, Result, Command};

mod cli;

fn main() {
    match parse(::std::env::args()) {
        Ok(_) => println!("Ok"),
        Err(e) => e.exit(),
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
// fn remove(args: &::docopt::ArgvMap) -> CliResult {
//    let uuid = args.get_str("<uuid>");
//    if uuid.is_empty() {
//        return Err(CliError::EmptyParameter("<uuid>"));
//    }
//
//    mp::with_dir(directory(args), |dir| mp::remove(dir, uuid))?;
//    println!("'{}' was removed", uuid);
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
//
// fn remove_expired(args: &::docopt::ArgvMap) -> CliResult {
//    use chrono::*;
//
//    let info = mp::with_dir(directory(args), |dir| mp::expired_profiles(dir, UTC::now()))?;
//    if info.profiles.is_empty() {
//        println!("All provisioning profiles are valid");
//    } else {
//        let mut errors_counter = 0;
//        let mut removals_counter = 0;
//        for profile in info.profiles {
//            match std::fs::remove_file(&profile.path) {
//                Ok(_) => {
//                    removals_counter += 1;
//                    println!("'{}' was removed", profile.uuid)
//                }
//                Err(e) => {
//                    errors_counter += 1;
//                    println!("Error while trying to remove '{}'", profile.uuid);
//                    println!("{}", e);
//                }
//            }
//        }
//        println!("Removed {} of {}", removals_counter, info.total);
//        if errors_counter > 0 {
//            println!("Errors: {}", errors_counter);
//        }
//    }
//    Ok(())
// }
//
// fn directory(args: &::docopt::ArgvMap) -> Option<&Path> {
//    let dir_name = args.get_str("<directory>");
//    if dir_name.is_empty() {
//        None
//    } else {
//        Some(dir_name.as_ref())
//    }
// }
