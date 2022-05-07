use clap::Parser;
use std::path::PathBuf;
use std::result;

/// A tool that helps iOS developers to manage mobileprovision files.
#[derive(Debug, PartialEq, Parser)]
#[clap(author, about, global_setting(clap::AppSettings::DeriveDisplayOrder))]
pub enum Command {
    /// Lists provisioning profiles
    #[clap(name = "list")]
    List(ListParams),

    /// Shows details of a provisioning profile using its uuid
    #[clap(name = "show")]
    ShowUuid(ShowUuidParams),

    /// Shows details of a provisioning profile
    #[clap(name = "show-file")]
    ShowFile(ShowFileParams),

    /// Removes provisioning profiles
    #[clap(name = "remove")]
    Remove(RemoveParams),

    /// Removes expired provisioning profiles
    #[clap(name = "clean")]
    Clean(CleanParams),
}

#[derive(Debug, Default, PartialEq, Parser)]
pub struct ListParams {
    /// Lists provisioning profiles that contain this text
    #[clap(short = 't', long = "text", forbid_empty_values(true))]
    pub text: Option<String>,

    /// Lists provisioning profiles that will expire in days
    #[clap( short = 'd', long = "expire-in-days", parse(try_from_str = parse_days))]
    pub expire_in_days: Option<u64>,

    /// A directory where to search provisioning profiles
    #[clap(long = "source", parse(from_os_str), forbid_empty_values(true))]
    pub directory: Option<PathBuf>,

    /// Output profile details in one line
    #[clap(long = "oneline")]
    pub oneline: bool,
}

#[derive(Debug, Default, PartialEq, Parser)]
pub struct ShowUuidParams {
    /// An uuid of a provisioning profile
    #[clap(forbid_empty_values(true))]
    pub uuid: String,

    /// A directory where to search provisioning profiles
    #[clap(long = "source", parse(from_os_str), forbid_empty_values(true))]
    pub directory: Option<PathBuf>,
}

#[derive(Debug, Default, PartialEq, Parser)]
pub struct ShowFileParams {
    /// A file path of a provisioning profile
    #[clap(parse(from_os_str), forbid_empty_values(true))]
    pub file: PathBuf,
}

#[derive(Debug, Default, PartialEq, Parser)]
pub struct RemoveParams {
    /// uuid(s) or bundle id(s) of provisioning profiles
    #[clap(forbid_empty_values(true))]
    pub ids: Vec<String>,

    /// A directory where to search provisioning profiles
    #[clap(long = "source", parse(from_os_str), forbid_empty_values(true))]
    pub directory: Option<PathBuf>,
}

#[derive(Debug, Default, PartialEq, Parser)]
pub struct CleanParams {
    /// A directory where to clean
    #[clap(long = "source", parse(from_os_str), forbid_empty_values(true))]
    pub directory: Option<PathBuf>,
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
            },
        )));

        expect!(parse(&["mprovision", "remove", "abcd", "ef"])).to(be_ok().value(Command::Remove(
            RemoveParams {
                ids: vec!["abcd".to_string(), "ef".to_string()],
                directory: None,
            },
        )));

        expect!(parse(&["mprovision", "remove", ""])).to(be_err());

        expect!(parse(&["mprovision", "remove", "abcd", "--source", "."])).to(be_ok().value(
            Command::Remove(RemoveParams {
                ids: vec!["abcd".to_string()],
                directory: Some(".".into()),
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
        })));

        expect!(parse(&["mprovision", "remove", "abcd", "--source", ""])).to(be_err());
    }

    #[test]
    fn clean_command() {
        expect!(parse(&["mprovision", "clean"]))
            .to(be_ok().value(Command::Clean(CleanParams { directory: None })));

        expect!(parse(&["mprovision", "clean", "--source", "."])).to(be_ok().value(
            Command::Clean(CleanParams {
                directory: Some(".".into()),
            }),
        ));

        expect!(parse(&["mprovision", "clean", "--source", ""])).to(be_err());
    }
}
