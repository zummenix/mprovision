
use chrono::{DateTime, UTC, TimeZone};

/// Represents provisioning profile data.
#[derive(Debug)]
pub struct Profile {
    pub uuid: String,
    pub name: String,
    pub app_identifier: String,
    pub creation_date: DateTime<UTC>,
    pub expiration_date: DateTime<UTC>,
}

impl Profile {
    pub fn empty() -> Self {
        Profile {
            uuid: "".into(),
            name: "".into(),
            app_identifier: "".into(),
            creation_date: UTC.timestamp(0, 0),
            expiration_date: UTC.timestamp(0, 0),
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
        desc.push_str(&format!("{} - {}", self.creation_date, self.expiration_date));
        desc
    }
}

#[cfg(test)]
mod tests {
    use expectest::prelude::*;
    use super::*;

    #[test]
    fn contains() {
        let profile = Profile {
            uuid: "123".into(),
            name: "name".into(),
            app_identifier: "id".into(),
        };
        expect!(profile.contains("12")).to(be_true());
        expect!(profile.contains("me")).to(be_true());
        expect!(profile.contains("id")).to(be_true());
    }
}
