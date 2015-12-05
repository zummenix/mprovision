
/// Represents provisioning profile data.
#[derive(Default, Debug)]
pub struct Profile {
    pub uuid: Option<String>,
    pub name: Option<String>,
    pub app_identifier: Option<String>,
}

impl Profile {
    /// Returns `true` if one or more fields of the profile contain `string`.
    pub fn contains(&self, string: &str) -> bool {
        let s = string.to_lowercase();
        let items = &[self.name.as_ref(), self.app_identifier.as_ref(), self.uuid.as_ref()];
        for item in items {
            if let &Some(ref string) = item {
                if string.to_lowercase().contains(&s) {
                    return true;
                }
            }
        }
        false
    }

    /// Returns profile in a text form.
    pub fn description(&self) -> String {
        let mut desc = String::new();
        desc.push_str(self.uuid.as_ref().unwrap_or(&"<UUID not found>".into()));
        desc.push_str("\n");
        desc.push_str(self.app_identifier.as_ref().unwrap_or(&"<ID not found>".into()));
        desc.push_str("\n");
        desc.push_str(self.name.as_ref().unwrap_or(&"<Name not found>".into()));
        desc
    }
}
