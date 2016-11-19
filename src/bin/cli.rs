use docopt::{self, Docopt, ArgvMap};

const USAGE: &'static str = "
mprovision
A tool that helps iOS developers to manage mobileprovision files.

Usage:
  mprovision list [--filter <text>] [--expires-in-days <days>] [<directory>]

Options:
  -h --help     Show this help message.
  --version     Show version.
";

pub fn parse_args(args: Vec<&str>) -> Result<ArgvMap, docopt::Error> {
    Docopt::new(USAGE).and_then(|docopt| {
        docopt.argv(args).version(Some(format!("mprovision {}", version!()))).parse()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use expectest::prelude::*;

    #[test]
    fn list_command() {
        expect!(parse_args(vec!["mprovision", "list"])).to(be_ok());
        expect!(parse_args(vec!["mprovision", "list", "."])).to(be_ok());
        expect!(parse_args(vec!["mprovision", "list", "--filter abc"])).to(be_ok());
        expect!(parse_args(vec!["mprovision", "list", "--filter abc", "."])).to(be_ok());
        expect!(parse_args(vec!["mprovision", "list", "--expires-in-days 0"])).to(be_ok());
        expect!(parse_args(vec!["mprovision", "list", "--expires-in-days 0", "."])).to(be_ok());
    }
}
