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
    fn parse<I, S>(args: I) -> result::Result<Command, clap::Error>
    where
        I: IntoIterator<Item = S>,
        S: Clone,
        ::std::ffi::OsString: From<S>,
    {
        Command::try_parse_from(args)
    }

    #[test]
    fn list_command() {
        assert_eq!(
            parse(["mprovision", "list"]).unwrap(),
            Command::List(ListParams::default())
        );

        assert_eq!(
            parse(["mprovision", "list", "--source", "."]).unwrap(),
            Command::List(ListParams {
                text: None,
                expire_in_days: None,
                directory: Some(".".into()),
                oneline: false,
            },)
        );

        assert!(parse(["mprovision", "list", "--source", ""]).is_err());

        assert_eq!(
            parse(["mprovision", "list", "--text", "abc"]).unwrap(),
            Command::List(ListParams {
                text: Some("abc".to_string()),
                expire_in_days: None,
                directory: None,
                oneline: false,
            },)
        );

        assert_eq!(
            parse(["mprovision", "list", "-t", "abc"]).unwrap(),
            Command::List(ListParams {
                text: Some("abc".to_string()),
                expire_in_days: None,
                directory: None,
                oneline: false,
            },)
        );

        assert!(parse(["mprovision", "list", "--text", ""]).is_err());

        assert!(parse(["mprovision", "list", "-t", ""]).is_err());

        assert_eq!(
            parse(["mprovision", "list", "--expire-in-days", "3"]).unwrap(),
            Command::List(ListParams {
                text: None,
                expire_in_days: Some(3),
                directory: None,
                oneline: false,
            })
        );

        assert_eq!(
            parse(["mprovision", "list", "-d", "3"]).unwrap(),
            Command::List(ListParams {
                text: None,
                expire_in_days: Some(3),
                directory: None,
                oneline: false,
            },)
        );

        assert!(parse(["mprovision", "list", "--expire-in-days", "-3"]).is_err());
        assert!(parse(["mprovision", "list", "-d", "-3"]).is_err());
        assert!(parse(["mprovision", "list", "--expire-in-days", "366"]).is_err());
        assert!(parse(["mprovision", "list", "-d", "366"]).is_err());

        assert_eq!(
            parse([
                "mprovision",
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

        assert_eq!(
            parse([
                "mprovision",
                "list",
                "-t",
                "abc",
                "-d",
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

        assert_eq!(
            parse(["mprovision", "list", "--oneline"]).unwrap(),
            Command::List(ListParams {
                text: None,
                expire_in_days: None,
                directory: None,
                oneline: true,
            },)
        );
    }

    #[test]
    fn show_uuid_command() {
        assert_eq!(
            parse(["mprovision", "show", "abcd"]).unwrap(),
            Command::ShowUuid(ShowUuidParams {
                uuid: "abcd".to_string(),
                directory: None,
            },)
        );

        assert!(parse(["mprovision", "show", ""]).is_err());

        assert_eq!(
            parse(["mprovision", "show", "abcd", "--source", "."]).unwrap(),
            Command::ShowUuid(ShowUuidParams {
                uuid: "abcd".to_string(),
                directory: Some(".".into()),
            })
        );

        assert!(parse(["mprovision", "show", "abcd", "--source", ""]).is_err());
    }

    #[test]
    fn show_file_command() {
        assert_eq!(
            parse(["mprovision", "show-file", "file.mprovision"]).unwrap(),
            Command::ShowFile(ShowFileParams {
                file: "file.mprovision".into(),
            })
        );

        assert!(parse(["mprovision", "show-file", "file.mprovision", "."]).is_err());

        assert!(parse(["mprovision", "show-file", ""]).is_err());
    }

    #[test]
    fn remove_id_command() {
        assert_eq!(
            parse(["mprovision", "remove", "abcd"]).unwrap(),
            Command::Remove(RemoveParams {
                ids: vec!["abcd".to_string()],
                directory: None,
                permanently: false,
            },)
        );

        assert_eq!(
            parse(["mprovision", "remove", "abcd", "--permanently"]).unwrap(),
            Command::Remove(RemoveParams {
                ids: vec!["abcd".to_string()],
                directory: None,
                permanently: true,
            })
        );

        assert_eq!(
            parse(["mprovision", "remove", "abcd", "ef"]).unwrap(),
            Command::Remove(RemoveParams {
                ids: vec!["abcd".to_string(), "ef".to_string()],
                directory: None,
                permanently: false,
            },)
        );

        assert!(parse(["mprovision", "remove", ""]).is_err());

        assert_eq!(
            parse(["mprovision", "remove", "abcd", "--source", "."]).unwrap(),
            Command::Remove(RemoveParams {
                ids: vec!["abcd".to_string()],
                directory: Some(".".into()),
                permanently: false,
            })
        );

        assert_eq!(
            parse(["mprovision", "remove", "abcd", "ef", "--source", ".",]).unwrap(),
            Command::Remove(RemoveParams {
                ids: vec!["abcd".to_string(), "ef".to_string()],
                directory: Some(".".into()),
                permanently: false,
            })
        );

        assert_eq!(
            parse([
                "mprovision",
                "remove",
                "abcd",
                "ef",
                "--permanently",
                "--source",
                ".",
            ])
            .unwrap(),
            Command::Remove(RemoveParams {
                ids: vec!["abcd".to_string(), "ef".to_string()],
                directory: Some(".".into()),
                permanently: true,
            })
        );

        assert!(parse(["mprovision", "remove", "abcd", "--source", ""]).is_err());
    }

    #[test]
    fn clean_command() {
        assert_eq!(
            parse(["mprovision", "clean"]).unwrap(),
            Command::Clean(CleanParams {
                directory: None,
                permanently: false,
            })
        );

        assert_eq!(
            parse(["mprovision", "clean", "--permanently"]).unwrap(),
            Command::Clean(CleanParams {
                directory: None,
                permanently: true,
            },)
        );

        assert_eq!(
            parse(["mprovision", "clean", "--source", "."]).unwrap(),
            Command::Clean(CleanParams {
                directory: Some(".".into()),
                permanently: false,
            },)
        );

        assert_eq!(
            parse(["mprovision", "clean", "--permanently", "--source", "."]).unwrap(),
            Command::Clean(CleanParams {
                directory: Some(".".into()),
                permanently: true,
            })
        );

        assert!(parse(["mprovision", "clean", "--source", ""]).is_err());
    }

    #[test]
    fn extract_command() {
        assert_eq!(
            parse(["mprovision", "extract", "app.ipa", "."]).unwrap(),
            Command::Extract(ExtractParams {
                source: "app.ipa".into(),
                destination: ".".into(),
            })
        );

        assert!(parse(["mprovision", "extract", "app.ipa"]).is_err());

        assert!(parse(["mprovision", "extract"]).is_err());
    }
}
