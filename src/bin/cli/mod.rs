use std::error;
use std::path::PathBuf;
use std::result;
use structopt::StructOpt;

#[derive(Debug, PartialEq, StructOpt)]
#[structopt(author, about, global_settings(&[clap::AppSettings::DeriveDisplayOrder]))]
/// A tool that helps iOS developers to manage mobileprovision files.
pub enum Command {
    #[structopt(name = "list")]
    /// Lists provisioning profiles
    List(ListParams),
    #[structopt(name = "show")]
    /// Shows details of a provisioning profile using its uuid
    ShowUuid(ShowUuidParams),
    #[structopt(name = "show-file")]
    /// Shows details of a provisioning profile
    ShowFile(ShowFileParams),
    #[structopt(name = "remove")]
    /// Removes provisioning profiles
    Remove(RemoveParams),
    #[structopt(name = "clean")]
    /// Removes expired provisioning profiles
    Clean(CleanParams),
}

#[derive(Debug, Default, PartialEq, StructOpt)]
pub struct ListParams {
    #[structopt(short = "t", long = "text", empty_values(false))]
    /// Lists provisioning profiles that contain this text
    pub text: Option<String>,
    #[structopt(
        short = "d",
        long = "expire-in-days",
        parse(try_from_str = parse_days)
    )]
    /// Lists provisioning profiles that will expire in days
    pub expire_in_days: Option<u64>,
    #[structopt(long = "source", parse(from_os_str), empty_values(false))]
    /// A directory where to search provisioning profiles
    pub directory: Option<PathBuf>,
    #[structopt(long = "oneline")]
    /// Output profile details in one line
    pub oneline: bool,
}

#[derive(Debug, Default, PartialEq, StructOpt)]
pub struct ShowUuidParams {
    #[structopt(empty_values(false))]
    /// An uuid of a provisioning profile
    pub uuid: String,
    #[structopt(long = "source", parse(from_os_str), empty_values(false))]
    /// A directory where to search provisioning profiles
    pub directory: Option<PathBuf>,
}

#[derive(Debug, Default, PartialEq, StructOpt)]
pub struct ShowFileParams {
    #[structopt(parse(from_os_str), empty_values(false))]
    /// A file path of a provisioning profile
    pub file: PathBuf,
}

#[derive(Debug, Default, PartialEq, StructOpt)]
pub struct RemoveParams {
    #[structopt(empty_values(false))]
    /// uuid(s) or bundle id(s) of provisioning profiles
    pub ids: Vec<String>,
    #[structopt(long = "source", parse(from_os_str), empty_values(false))]
    /// A directory where to search provisioning profiles
    pub directory: Option<PathBuf>,
}

#[derive(Debug, Default, PartialEq, StructOpt)]
pub struct CleanParams {
    #[structopt(long = "source", parse(from_os_str), empty_values(false))]
    /// A directory where to clean
    pub directory: Option<PathBuf>,
}

/// Runs the cli and returns the `Command`.
pub fn run() -> Command {
    Command::from_args()
}

/// Parses and validates days argument.
fn parse_days(s: &str) -> result::Result<u64, Box<dyn error::Error>> {
    let days = s.parse::<i64>()?;
    if days < 0 || days > 365 {
        let message = format!("should be between 0 and 365, got {}", days);
        return Err(message.into());
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
        Command::from_iter_safe(args)
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
