use memchr::memmem;

const PLIST_PREFIX: &[u8] = b"<?xml version=";
const PLIST_SUFFIX: &[u8] = b"</plist>";

/// Attempts to find a plist content in a `data` and return it as a slice.
///
/// Since mobileprovision files contain "garbage" at the start and the end you need to extract
/// a plist content before the xml parsing.
pub fn find(data: &[u8]) -> Option<&[u8]> {
    let start_i = memmem::find(data, PLIST_PREFIX);
    let end_i = memmem::rfind(data, PLIST_SUFFIX).map(|i| i + PLIST_SUFFIX.len());

    if let (Some(start_i), Some(end_i)) = (start_i, end_i) {
        if end_i <= data.len() {
            return Some(&data[start_i..end_i]);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use expectest::expect;
    use expectest::prelude::*;

    #[test]
    fn test_find_plist() {
        let data: &[u8] = b"<?xml version=</plist>";
        expect!(find(data)).to(be_some().value(data));

        let data: &[u8] = b"   <?xml version=abcd</plist>   ";
        expect!(find(data)).to(be_some().value(b"<?xml version=abcd</plist>"));
    }
}
