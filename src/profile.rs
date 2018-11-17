use chrono::{DateTime, Utc};
use crate::{Error, Result};
use plist;
use plist::PlistEvent::*;
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Represents a file with a provisioning profile info.
#[derive(Debug, Clone)]
pub struct Profile {
    pub path: PathBuf,
    pub info: ProfileInfo,
}

impl Profile {
    /// Returns instance of the `Profile` parsed from a file.
    pub fn from_file(path: &Path) -> Result<Self> {
        let mut buf = Vec::new();
        File::open(path)?.read_to_end(&mut buf)?;
        let info = ProfileInfo::from_xml_data(&buf)
            .ok_or_else(|| Error::Own("Couldn't parse file.".into()))?;
        Ok(Profile {
            path: path.to_owned(),
            info,
        })
    }
}

/// Represents provisioning profile info.
#[derive(Debug, Clone)]
pub struct ProfileInfo {
    pub uuid: String,
    pub name: String,
    pub app_identifier: String,
    pub creation_date: SystemTime,
    pub expiration_date: SystemTime,
}

impl ProfileInfo {
    /// Returns instance of the `Profile` parsed from a `data`.
    pub fn from_xml_data(data: &[u8]) -> Option<Self> {
        if let Some(data) = crate::plist_extractor::find(data) {
            let mut profile = ProfileInfo::empty();
            let mut iter = plist::xml::EventReader::new(io::Cursor::new(data));
            while let Some(item) = iter.next() {
                if let Ok(StringValue(key)) = item {
                    if key == "UUID" {
                        if let Some(Ok(StringValue(value))) = iter.next() {
                            profile.uuid = value;
                        }
                    }
                    if key == "Name" {
                        if let Some(Ok(StringValue(value))) = iter.next() {
                            profile.name = value;
                        }
                    }
                    if key == "application-identifier" {
                        if let Some(Ok(StringValue(value))) = iter.next() {
                            profile.app_identifier = value;
                        }
                    }
                    if key == "CreationDate" {
                        if let Some(Ok(DateValue(value))) = iter.next() {
                            profile.creation_date = value.into();
                        }
                    }
                    if key == "ExpirationDate" {
                        if let Some(Ok(DateValue(value))) = iter.next() {
                            profile.expiration_date = value.into();
                        }
                    }
                }
            }
            Some(profile)
        } else {
            None
        }
    }

    /// Returns an empty profile info.
    pub fn empty() -> Self {
        ProfileInfo {
            uuid: "".into(),
            name: "".into(),
            app_identifier: "".into(),
            creation_date: SystemTime::UNIX_EPOCH,
            expiration_date: SystemTime::UNIX_EPOCH,
        }
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

    /// Returns a bundle id of a profile.
    pub fn bundle_id(&self) -> Option<&str> {
        self.app_identifier
            .find(|ch| ch == '.')
            .map(|i| &self.app_identifier[(i + 1)..])
    }

    /// Returns profile in a text form.
    pub fn description(&self) -> String {
        let mut desc = String::new();
        desc.push_str(&self.uuid);
        desc.push_str("\n");
        desc.push_str(&self.app_identifier);
        desc.push_str("\n");
        desc.push_str(&self.name);
        desc.push_str("\n");
        desc.push_str(&format!(
            "{} - {}",
            DateTime::<Utc>::from(self.creation_date),
            DateTime::<Utc>::from(self.expiration_date)
        ));
        desc
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use expectest::prelude::*;

    #[test]
    fn contains() {
        let profile = ProfileInfo {
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
    fn correct_bundle_id() {
        let mut profile = ProfileInfo::empty();
        profile.app_identifier = "12345ABCDE.com.exmaple.app".to_owned();
        expect!(profile.bundle_id()).to(be_some().value("com.exmaple.app"));
    }

    #[test]
    fn incorrect_bundle_id() {
        let mut profile = ProfileInfo::empty();
        profile.app_identifier = "12345ABCDE".to_owned();
        expect!(profile.bundle_id()).to(be_none());
    }

    #[test]
    fn wildcard_bundle_id() {
        let mut profile = ProfileInfo::empty();
        profile.app_identifier = "12345ABCDE.*".to_owned();
        expect!(profile.bundle_id()).to(be_some().value("*"));
    }
}
