use mprovision::profile::Info;
use std::time::{Duration, SystemTime};

fn time(secs: u64) -> SystemTime {
    SystemTime::UNIX_EPOCH
        .checked_add(Duration::new(secs, 0))
        .unwrap()
}

#[test]
fn deserialize() {
    let data = std::fs::read("tests/test.xml").unwrap();
    let info = Info::from_xml_data(&data).unwrap();
    let expected = Info {
        uuid: "fbcdefgl-af78-hal1-lgl1-87jl897lja8e".to_owned(),
        name: "TestApp iOS Development".to_owned(),
        app_identifier: "1234567890.com.testapp".to_owned(),
        creation_date: time(1562926802),
        expiration_date: time(1594462802),
    };
    assert_eq!(info, expected);
}
