use insta_cmd::{assert_cmd_snapshot, get_cargo_bin};
use std::process::Command;

#[test]
fn extract() {
    let output = tmp_dir();
    let mut cmd = cli();
    cmd.arg("extract")
        .arg("tests/fixtures/ipa/bitbar-ios-sample.ipa")
        .arg(output.path());

    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");
}

#[test]
fn extract_with_list() {
    let output = tmp_dir();
    let mut cmd = cli();
    cmd.arg("extract")
        .arg("tests/fixtures/ipa/bitbar-ios-sample.ipa")
        .arg(output.path());

    assert!(cmd
        .status()
        .expect("extract should be successful")
        .success());

    cmd = cli();
    cmd.arg("list").arg("--source").arg(output.path());

    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    85bef95d-eeb7-4384-b909-86fece8c67fa
    N9HW7DB6H4.*
    XC Ad Hoc: *
    2015-12-08 07:56:54 UTC - 2016-12-07 07:46:52 UTC

    ----- stderr -----
    ");
}

fn cli() -> Command {
    Command::new(get_cargo_bin("mprovision"))
}

fn tmp_dir() -> temp_dir::TempDir {
    temp_dir::TempDir::with_prefix("mprovision-").unwrap()
}
