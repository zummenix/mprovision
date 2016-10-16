use memmem::{Searcher, TwoWaySearcher};

const PLIST_PREFIX: &'static [u8] = b"<?xml version=";
const PLIST_SUFFIX: &'static [u8] = b"</plist>";

pub struct Context {
    prefix_searcher: TwoWaySearcher<'static>,
    suffix_searcher: TwoWaySearcher<'static>,
}

impl Context {
    /// Returns a plist content in a `data`.
    pub fn find_plist<'b>(&self, data: &'b [u8]) -> Option<&'b [u8]> {
        let start_i = self.prefix_searcher.search_in(data);
        let end_i = self.suffix_searcher.search_in(data).map(|i| i + PLIST_SUFFIX.len());

        if let (Some(start_i), Some(end_i)) = (start_i, end_i) {
            if end_i <= data.len() {
                return Some(&data[start_i..end_i]);
            }
        }

        None
    }
}

impl Default for Context {
    fn default() -> Self {
        Context {
            prefix_searcher: TwoWaySearcher::new(PLIST_PREFIX),
            suffix_searcher: TwoWaySearcher::new(PLIST_SUFFIX),
        }
    }
}

#[cfg(test)]
mod tests {
    use expectest::prelude::*;
    use super::*;

    #[test]
    fn test_find_plist() {
        let context = Context::default();
        let data: &[u8] = b"<?xml version=</plist>";
        expect!(context.find_plist(&data)).to(be_some().value(data));

        let data: &[u8] = b"   <?xml version=</plist>   ";
        expect!(context.find_plist(&data)).to(be_some().value(b"<?xml version=</plist>"));
    }
}
