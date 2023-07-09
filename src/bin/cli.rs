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
    use expectest::expect;
    use expectest::prelude::*;

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
        expect!(parse(&["mprovision", "list"]))
            .to(be_ok().value(Command::List(ListParams::default())));

        expect!(parse(&["mprovision", "list", "--source", "."])).to(be_ok().value(Command::List(
            ListParams {
                text: None,
                expire_in_days: None,
                directory: Some(".".into()),
                oneline: false,
            },
        )));

        expect!(parse(&["mprovision", "list", "--source", ""])).to(be_err());

        expect!(parse(&["mprovision", "list", "--text", "abc"])).to(be_ok().value(Command::List(
            ListParams {
                text: Some("abc".to_string()),
                expire_in_days: None,
                directory: None,
                oneline: false,
            },
        )));

        expect!(parse(&["mprovision", "list", "-t", "abc"])).to(be_ok().value(Command::List(
            ListParams {
                text: Some("abc".to_string()),
                expire_in_days: None,
                directory: None,
                oneline: false,
            },
        )));

        expect!(parse(&["mprovision", "list", "--text", ""])).to(be_err());

        expect!(parse(&["mprovision", "list", "-t", ""])).to(be_err());

        expect!(parse(&["mprovision", "list", "--expire-in-days", "3"])).to(be_ok().value(
            Command::List(ListParams {
                text: None,
                expire_in_days: Some(3),
                directory: None,
                oneline: false,
            }),
        ));

        expect!(parse(&["mprovision", "list", "-d", "3"])).to(be_ok().value(Command::List(
            ListParams {
                text: None,
                expire_in_days: Some(3),
                directory: None,
                oneline: false,
            },
        )));

        expect!(parse(&["mprovision", "list", "--expire-in-days", "-3"])).to(be_err());
        expect!(parse(&["mprovision", "list", "-d", "-3"])).to(be_err());
        expect!(parse(&["mprovision", "list", "--expire-in-days", "366"])).to(be_err());
        expect!(parse(&["mprovision", "list", "-d", "366"])).to(be_err());

        expect!(parse(&[
            "mprovision",
            "list",
            "--text",
            "abc",
            "--expire-in-days",
            "3",
            "--source",
            ".",
        ]))
        .to(be_ok().value(Command::List(ListParams {
            text: Some("abc".to_string()),
            expire_in_days: Some(3),
            directory: Some(".".into()),
            oneline: false,
        })));

        expect!(parse(&[
            "mprovision",
            "list",
            "-t",
            "abc",
            "-d",
            "3",
            "--source",
            ".",
        ]))
        .to(be_ok().value(Command::List(ListParams {
            text: Some("abc".to_string()),
            expire_in_days: Some(3),
            directory: Some(".".into()),
            oneline: false,
        })));

        expect!(parse(&["mprovision", "list", "--oneline"])).to(be_ok().value(Command::List(
            ListParams {
                text: None,
                expire_in_days: None,
                directory: None,
                oneline: true,
            },
        )));
    }

    #[test]
    fn show_uuid_command() {
        expect!(parse(&["mprovision", "show", "abcd"])).to(be_ok().value(Command::ShowUuid(
            ShowUuidParams {
                uuid: "abcd".to_string(),
                directory: None,
            },
        )));

        expect!(parse(&["mprovision", "show", ""])).to(be_err());

        expect!(parse(&["mprovision", "show", "abcd", "--source", "."])).to(be_ok().value(
            Command::ShowUuid(ShowUuidParams {
                uuid: "abcd".to_string(),
                directory: Some(".".into()),
            }),
        ));

        expect!(parse(&["mprovision", "show", "abcd", "--source", ""])).to(be_err());
    }

    #[test]
    fn show_file_command() {
        expect!(parse(&["mprovision", "show-file", "file.mprovision"])).to(be_ok().value(
            Command::ShowFile(ShowFileParams {
                file: "file.mprovision".into(),
            }),
        ));

        expect!(parse(&["mprovision", "show-file", "file.mprovision", "."])).to(be_err());

        expect!(parse(&["mprovision", "show-file", ""])).to(be_err());
    }

    #[test]
    fn remove_id_command() {
        expect!(parse(&["mprovision", "remove", "abcd"])).to(be_ok().value(Command::Remove(
            RemoveParams {
                ids: vec!["abcd".to_string()],
                directory: None,
                permanently: false,
            },
        )));

        expect!(parse(&["mprovision", "remove", "abcd", "--permanently"])).to(be_ok().value(
            Command::Remove(RemoveParams {
                ids: vec!["abcd".to_string()],
                directory: None,
                permanently: true,
            }),
        ));

        expect!(parse(&["mprovision", "remove", "abcd", "ef"])).to(be_ok().value(Command::Remove(
            RemoveParams {
                ids: vec!["abcd".to_string(), "ef".to_string()],
                directory: None,
                permanently: false,
            },
        )));

        expect!(parse(&["mprovision", "remove", ""])).to(be_err());

        expect!(parse(&["mprovision", "remove", "abcd", "--source", "."])).to(be_ok().value(
            Command::Remove(RemoveParams {
                ids: vec!["abcd".to_string()],
                directory: Some(".".into()),
                permanently: false,
            }),
        ));

        expect!(parse(&[
            "mprovision",
            "remove",
            "abcd",
            "ef",
            "--source",
            ".",
        ]))
        .to(be_ok().value(Command::Remove(RemoveParams {
            ids: vec!["abcd".to_string(), "ef".to_string()],
            directory: Some(".".into()),
            permanently: false,
        })));

        expect!(parse(&[
            "mprovision",
            "remove",
            "abcd",
            "ef",
            "--permanently",
            "--source",
            ".",
        ]))
        .to(be_ok().value(Command::Remove(RemoveParams {
            ids: vec!["abcd".to_string(), "ef".to_string()],
            directory: Some(".".into()),
            permanently: true,
        })));

        expect!(parse(&["mprovision", "remove", "abcd", "--source", ""])).to(be_err());
    }

    #[test]
    fn clean_command() {
        expect!(parse(&["mprovision", "clean"])).to(be_ok().value(Command::Clean(CleanParams {
            directory: None,
            permanently: false,
        })));

        expect!(parse(&["mprovision", "clean", "--permanently"])).to(be_ok().value(
            Command::Clean(CleanParams {
                directory: None,
                permanently: true,
            }),
        ));

        expect!(parse(&["mprovision", "clean", "--source", "."])).to(be_ok().value(
            Command::Clean(CleanParams {
                directory: Some(".".into()),
                permanently: false,
            }),
        ));

        expect!(parse(&[
            "mprovision",
            "clean",
            "--permanently",
            "--source",
            "."
        ]))
        .to(be_ok().value(Command::Clean(CleanParams {
            directory: Some(".".into()),
            permanently: true,
        })));

        expect!(parse(&["mprovision", "clean", "--source", ""])).to(be_err());
    }
}
