use crate::{Error, Result};
use serde::Deserialize;
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Represents a file with a provisioning profile info.
#[derive(Debug, Clone)]
pub struct Profile {
    pub path: PathBuf,
    pub info: Info,
}

impl Profile {
    /// Returns instance of the `Profile` parsed from a file.
    pub fn from_file(path: &Path) -> Result<Self> {
        let mut buf = Vec::new();
        File::open(path)?.read_to_end(&mut buf)?;
        let info =
            Info::from_xml_data(&buf).ok_or_else(|| Error::Own("Couldn't parse file.".into()))?;
        Ok(Self {
            path: path.to_owned(),
            info,
        })
    }
}

/// Represents provisioning profile info.
#[derive(Debug, PartialEq, Clone)]
pub struct Info {
    pub uuid: String,
    pub name: String,
    pub app_identifier: String,
    pub creation_date: SystemTime,
    pub expiration_date: SystemTime,
}

#[derive(Debug, Deserialize)]
struct InfoDef {
    #[serde(rename = "UUID")]
    pub uuid: String,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Entitlements")]
    pub entitlements: Entitlements,
    #[serde(rename = "CreationDate")]
    pub creation_date: plist::Date,
    #[serde(rename = "ExpirationDate")]
    pub expiration_date: plist::Date,
}

#[derive(Debug, Deserialize)]
struct Entitlements {
    #[serde(rename = "application-identifier")]
    pub app_identifier: String,
}

impl Info {
    /// Returns instance of the `Info` parsed from a `data`.
    pub fn from_xml_data(data: &[u8]) -> Option<Self> {
        crate::plist_extractor::find(data).and_then(|xml| {
            plist::from_reader_xml(io::Cursor::new(xml))
                .ok()
                .map(|info: InfoDef| Self {
                    uuid: info.uuid,
                    name: info.name,
                    app_identifier: info.entitlements.app_identifier,
                    creation_date: info.creation_date.into(),
                    expiration_date: info.expiration_date.into(),
                })
        })
    }

    /// Returns `true` if one or more fields of the profile contain `string`.
    pub fn contains(&self, string: &str) -> bool {
        let s = string.to_lowercase();
        let items = &[&self.name, &self.app_identifier, &self.uuid];
        for item in items {
            if item.to_lowercase().contains(&s) {
                return true;
            }
        }
        false
    }

    /// Returns `true` if the profile has any of `ids` as `uuid` or `bundle_id`.
    pub fn has_ids(&self, ids: impl IntoIterator<Item = impl AsRef<str>>) -> bool {
        let bundle_id = self.bundle_id();
        ids.into_iter()
            .any(|id| self.uuid == id.as_ref() || bundle_id == Some(id.as_ref()))
    }

    /// Returns a bundle id of a profile.
    pub fn bundle_id(&self) -> Option<&str> {
        self.app_identifier
            .find(|ch| ch == '.')
            .map(|i| &self.app_identifier[(i + 1)..])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use expectest::expect;
    use expectest::prelude::*;

    impl Info {
        /// Returns an empty profile info.
        fn empty() -> Self {
            Self {
                uuid: "".into(),
                name: "".into(),
                app_identifier: "".into(),
                creation_date: SystemTime::UNIX_EPOCH,
                expiration_date: SystemTime::UNIX_EPOCH,
            }
        }
    }

    #[test]
    fn contains() {
        let profile = Info {
            uuid: "123".into(),
            name: "name".into(),
            app_identifier: "id".into(),
            creation_date: SystemTime::UNIX_EPOCH,
            expiration_date: SystemTime::UNIX_EPOCH,
        };
        expect!(profile.contains("12")).to(be_true());
        expect!(profile.contains("me")).to(be_true());
        expect!(profile.contains("id")).to(be_true());
    }

    #[test]
    fn has_id_in_bundle_id() {
        let mut profile = Info::empty();
        profile.app_identifier = "12345ABCDE.com.example.app".to_owned();
        assert!(profile.has_ids(&["com.example.app"]));
    }

    #[test]
    fn has_id_in_uuid() {
        let mut profile = Info::empty();
        profile.uuid = String::from("123");
        assert!(profile.has_ids(&["123"]));
    }

    #[test]
    fn does_not_have_ids() {
        let profile = Info::empty();
        assert!(!profile.has_ids(&["a", "b", "c"]));
    }

    #[test]
    fn correct_bundle_id() {
        let mut profile = Info::empty();
        profile.app_identifier = "12345ABCDE.com.example.app".to_owned();
        expect!(profile.bundle_id()).to(be_some().value("com.example.app"));
    }

    #[test]
    fn incorrect_bundle_id() {
        let mut profile = Info::empty();
        profile.app_identifier = "12345ABCDE".to_owned();
        expect!(profile.bundle_id()).to(be_none());
    }

    #[test]
    fn wildcard_bundle_id() {
        let mut profile = Info::empty();
        profile.app_identifier = "12345ABCDE.*".to_owned();
        expect!(profile.bundle_id()).to(be_some().value("*"));
    }
}
