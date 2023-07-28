use clap::Parser;
use std::path::PathBuf;
use std::result;

/// A tool that helps iOS developers to manage mobileprovision files.
#[derive(Debug, PartialEq, Parser)]
#[command(author, about)]
pub enum Command {
    /// Lists provisioning profiles
    #[command(name = "list")]
    List(ListParams),

    /// Shows details of a provisioning profile using its uuid
    #[command(name = "show")]
    ShowUuid(ShowUuidParams),

    /// Shows details of a provisioning profile
    #[command(name = "show-file")]
    ShowFile(ShowFileParams),

    /// Removes provisioning profiles
    #[command(name = "remove")]
    Remove(RemoveParams),

    /// Removes expired provisioning profiles
    #[command(name = "clean")]
    Clean(CleanParams),

    /// Extracts provisioning profiles from ipa file or zip archive
    #[command(name = "extract")]
    Extract(ExtractParams),
}

#[derive(Debug, Default, PartialEq, Parser)]
pub struct ListParams {
    /// Lists provisioning profiles that contain this text
    #[arg(short = 't', long = "text", value_parser = clap::builder::NonEmptyStringValueParser::new())]
    pub text: Option<String>,

    /// Lists provisioning profiles that will expire in days
    #[arg(short = 'd', long = "expire-in-days", value_parser = parse_days)]
    pub expire_in_days: Option<u64>,

    /// A directory where to search provisioning profiles
    #[arg(long = "source")]
    pub directory: Option<PathBuf>,

    /// Output profile details in one line
    #[arg(long = "oneline")]
    pub oneline: bool,
}

#[derive(Debug, Default, PartialEq, Parser)]
pub struct ShowUuidParams {
    /// An uuid of a provisioning profile
    #[arg(value_parser = clap::builder::NonEmptyStringValueParser::new())]
    pub uuid: String,

    /// A directory where to search provisioning profiles
    #[arg(long = "source")]
    pub directory: Option<PathBuf>,
}

#[derive(Debug, Default, PartialEq, Parser)]
pub struct ShowFileParams {
    /// A file path of a provisioning profile
    pub file: PathBuf,
}

#[derive(Debug, Default, PartialEq, Parser)]
pub struct RemoveParams {
    /// uuid(s) or bundle id(s) of provisioning profiles
    #[arg(num_args(1..), value_parser = clap::builder::NonEmptyStringValueParser::new())]
    pub ids: Vec<String>,

    /// A directory where to search provisioning profiles
    #[arg(long = "source")]
    pub directory: Option<PathBuf>,

    /// Whether to remove provisioning profiles permanently
    #[arg(long = "permanently")]
    pub permanently: bool,
}

#[derive(Debug, Default, PartialEq, Parser)]
pub struct CleanParams {
    /// A directory where to clean
    #[arg(long = "source")]
    pub directory: Option<PathBuf>,

    /// Whether to remove provisioning profiles permanently
    #[arg(long = "permanently")]
    pub permanently: bool,
}

#[derive(Debug, Default, PartialEq, Parser)]
pub struct ExtractParams {
    /// File path to an archive
    pub source: PathBuf,
    /// Directory where to place extracted provisioning profiles
    pub destination: PathBuf,
}

/// Runs the cli and returns the `Command`.
pub fn run() -> Command {
    Command::parse()
}

/// Parses and validates days argument.
fn parse_days(s: &str) -> result::Result<u64, String> {
    let days = s.parse::<i64>().map_err(|err| err.to_string())?;
    if !(0..=365).contains(&days) {
        return Err(format!("should be between 0 and 365, got {}", days));
    }
    Ok(days as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parses arguments and returns a `Command`.
    fn parse<'a, I>(args: I) -> result::Result<Command, clap::Error>
    where
        I: IntoIterator<Item = &'a str>,
        ::std::ffi::OsString: From<&'a str>,
    {
        Command::try_parse_from(std::iter::once("mprovision").chain(args))
    }

    #[test]
    fn list() {
        assert_eq!(
            parse(["list"]).unwrap(),
            Command::List(ListParams::default())
        );
    }

    #[test]
    fn list_with_source() {
        assert_eq!(
            parse(["list", "--source", "."]).unwrap(),
            Command::List(ListParams {
                text: None,
                expire_in_days: None,
                directory: Some(".".into()),
                oneline: false,
            })
        );
    }

    #[test]
    fn list_with_empty_source_should_err() {
        assert!(parse(["list", "--source", ""]).is_err());
    }

    #[test]
    fn list_with_text_long() {
        assert_eq!(
            parse(["list", "--text", "abc"]).unwrap(),
            Command::List(ListParams {
                text: Some("abc".to_string()),
                expire_in_days: None,
                directory: None,
                oneline: false,
            })
        );
    }

    #[test]
    fn list_with_text_short() {
        assert_eq!(
            parse(["list", "-t", "abc"]).unwrap(),
            Command::List(ListParams {
                text: Some("abc".to_string()),
                expire_in_days: None,
                directory: None,
                oneline: false,
            })
        );
    }

    #[test]
    fn list_with_empty_text_should_err() {
        assert!(parse(["list", "--text", ""]).is_err());
    }

    #[test]
    fn list_with_expire_long() {
        assert_eq!(
            parse(["list", "--expire-in-days", "3"]).unwrap(),
            Command::List(ListParams {
                text: None,
                expire_in_days: Some(3),
                directory: None,
                oneline: false,
            })
        );
    }

    #[test]
    fn list_with_expire_short() {
        assert_eq!(
            parse(["list", "-d", "3"]).unwrap(),
            Command::List(ListParams {
                text: None,
                expire_in_days: Some(3),
                directory: None,
                oneline: false,
            })
        );
    }

    #[test]
    fn list_with_expire_less_than_0_should_err() {
        assert!(parse(["list", "--expire-in-days", "-3"]).is_err());
    }

    #[test]
    fn list_with_expire_grater_than_365_should_err() {
        assert!(parse(["list", "--expire-in-days", "366"]).is_err());
    }

    #[test]
    fn list_with_all_arguments_long() {
        assert_eq!(
            parse([
                "list",
                "--text",
                "abc",
                "--expire-in-days",
                "3",
                "--source",
                ".",
            ])
            .unwrap(),
            Command::List(ListParams {
                text: Some("abc".to_string()),
                expire_in_days: Some(3),
                directory: Some(".".into()),
                oneline: false,
            })
        );
    }

    #[test]
    fn list_with_all_arguments_short() {
        assert_eq!(
            parse(["list", "-t", "abc", "-d", "3", "--source", ".",]).unwrap(),
            Command::List(ListParams {
                text: Some("abc".to_string()),
                expire_in_days: Some(3),
                directory: Some(".".into()),
                oneline: false,
            })
        );
    }

    #[test]
    fn list_with_oneline() {
        assert_eq!(
            parse(["list", "--oneline"]).unwrap(),
            Command::List(ListParams {
                text: None,
                expire_in_days: None,
                directory: None,
                oneline: true
            })
        );
    }

    #[test]
    fn show_uuid() {
        assert_eq!(
            parse(["show", "abcd"]).unwrap(),
            Command::ShowUuid(ShowUuidParams {
                uuid: "abcd".to_string(),
                directory: None,
            })
        );
    }

    #[test]
    fn show_uuid_without_args_should_err() {
        assert!(parse(["show", ""]).is_err());
    }

    #[test]
    fn show_uuid_with_source() {
        assert_eq!(
            parse(["show", "abcd", "--source", "."]).unwrap(),
            Command::ShowUuid(ShowUuidParams {
                uuid: "abcd".to_string(),
                directory: Some(".".into()),
            })
        );
    }

    #[test]
    fn show_uuid_with_empty_source_should_err() {
        assert!(parse(["show", "abcd", "--source", ""]).is_err());
    }

    #[test]
    fn show_file() {
        assert_eq!(
            parse(["show-file", "file.mprovision"]).unwrap(),
            Command::ShowFile(ShowFileParams {
                file: "file.mprovision".into(),
            })
        );
    }

    #[test]
    fn show_file_with_multiple_paths_should_err() {
        assert!(parse(["show-file", "file.mprovision", "."]).is_err());
    }

    #[test]
    fn show_file_with_empty_path_should_err() {
        assert!(parse(["show-file", ""]).is_err());
    }

    #[test]
    fn remove() {
        assert_eq!(
            parse(["remove", "abcd"]).unwrap(),
            Command::Remove(RemoveParams {
                ids: vec!["abcd".to_string()],
                directory: None,
                permanently: false,
            })
        );
    }

    #[test]
    fn remove_single_permanently() {
        assert_eq!(
            parse(["remove", "abcd", "--permanently"]).unwrap(),
            Command::Remove(RemoveParams {
                ids: vec!["abcd".to_string()],
                directory: None,
                permanently: true,
            })
        );
    }

    #[test]
    fn remove_multiple() {
        assert_eq!(
            parse(["remove", "abcd", "ef"]).unwrap(),
            Command::Remove(RemoveParams {
                ids: vec!["abcd".to_string(), "ef".to_string()],
                directory: None,
                permanently: false,
            })
        );
    }

    #[test]
    fn remove_with_empty_arg_should_err() {
        assert!(parse(["remove", ""]).is_err());
    }

    #[test]
    fn remove_single_with_source() {
        assert_eq!(
            parse(["remove", "abcd", "--source", "."]).unwrap(),
            Command::Remove(RemoveParams {
                ids: vec!["abcd".to_string()],
                directory: Some(".".into()),
                permanently: false,
            })
        );
    }

    #[test]
    fn remove_multiple_with_source() {
        assert_eq!(
            parse(["remove", "abcd", "ef", "--source", ".",]).unwrap(),
            Command::Remove(RemoveParams {
                ids: vec!["abcd".to_string(), "ef".to_string()],
                directory: Some(".".into()),
                permanently: false,
            })
        );
    }

    #[test]
    fn remove_with_permanently_and_source() {
        assert_eq!(
            parse(["remove", "abcd", "ef", "--permanently", "--source", ".",]).unwrap(),
            Command::Remove(RemoveParams {
                ids: vec!["abcd".to_string(), "ef".to_string()],
                directory: Some(".".into()),
                permanently: true,
            })
        );
    }

    #[test]
    fn remove_with_empty_source_should_err() {
        assert!(parse(["remove", "abcd", "--source", ""]).is_err());
    }

    #[test]
    fn clean() {
        assert_eq!(
            parse(["clean"]).unwrap(),
            Command::Clean(CleanParams {
                directory: None,
                permanently: false,
            })
        );
    }

    #[test]
    fn clean_with_permanently() {
        assert_eq!(
            parse(["clean", "--permanently"]).unwrap(),
            Command::Clean(CleanParams {
                directory: None,
                permanently: true,
            })
        );
    }

    #[test]
    fn clean_with_source() {
        assert_eq!(
            parse(["clean", "--source", "."]).unwrap(),
            Command::Clean(CleanParams {
                directory: Some(".".into()),
                permanently: false,
            })
        );
    }

    #[test]
    fn clean_with_permanently_and_source() {
        assert_eq!(
            parse(["clean", "--permanently", "--source", "."]).unwrap(),
            Command::Clean(CleanParams {
                directory: Some(".".into()),
                permanently: true,
            })
        );
    }

    #[test]
    fn clean_with_empty_source_should_err() {
        assert!(parse(["clean", "--source", ""]).is_err());
    }

    #[test]
    fn extract() {
        assert_eq!(
            parse(["extract", "app.ipa", "."]).unwrap(),
            Command::Extract(ExtractParams {
                source: "app.ipa".into(),
                destination: ".".into(),
            })
        );
    }

    #[test]
    fn extract_with_one_arg_should_err() {
        assert!(parse(["extract", "app.ipa"]).is_err());
    }

    #[test]
    fn extract_without_args_should_err() {
        assert!(parse(["extract"]).is_err());
    }
}
