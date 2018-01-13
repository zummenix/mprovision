use std::io::{self, Read};
use std::fs::File;
use std::path::{Path, PathBuf};
use chrono::{DateTime, TimeZone, Utc};
use plist::PlistEvent::*;
use plist;
use {Error, Result};

/// Represents provisioning profile data.
#[derive(Debug, Clone)]
pub struct Profile {
    pub path: PathBuf,
    pub uuid: String,
    pub name: String,
    pub app_identifier: String,
    pub creation_date: DateTime<Utc>,
    pub expiration_date: DateTime<Utc>,
}

impl Profile {
    /// Returns instance of the `Profile` parsed from a file.
    pub fn from_file(path: &Path) -> Result<Self> {
        let mut buf = Vec::new();
        File::open(path)?.read_to_end(&mut buf)?;
        Profile::from_xml_data(&buf)
            .map(|mut p| {
                p.path = path.to_owned();
                p
            })
            .ok_or_else(|| Error::Own("Couldn't parse file.".into()))
    }

    /// Returns instance of the `Profile` parsed from a `data`.
    pub fn from_xml_data(data: &[u8]) -> Option<Self> {
        if let Some(data) = ::plist_extractor::find(data) {
            let mut profile = Profile::empty();
            let mut iter = plist::xml::EventReader::new(io::Cursor::new(data)).into_iter();
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

    pub fn empty() -> Self {
        Profile {
            path: PathBuf::new(),
            uuid: "".into(),
            name: "".into(),
            app_identifier: "".into(),
            creation_date: Utc.timestamp(0, 0),
            expiration_date: Utc.timestamp(0, 0),
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
            self.creation_date,
            self.expiration_date
        ));
        desc
    }
}

#[cfg(test)]
mod tests {
    use expectest::prelude::*;
    use std::path::PathBuf;
    use chrono::{TimeZone, Utc};
    use super::*;

    #[test]
    fn contains() {
        let profile = Profile {
            path: PathBuf::new(),
            uuid: "123".into(),
            name: "name".into(),
            app_identifier: "id".into(),
            creation_date: Utc.timestamp(0, 0),
            expiration_date: Utc.timestamp(0, 0),
        };
        expect!(profile.contains("12")).to(be_true());
        expect!(profile.contains("me")).to(be_true());
        expect!(profile.contains("id")).to(be_true());
    }
}
