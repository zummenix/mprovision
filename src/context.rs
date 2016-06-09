use memmem::{Searcher, TwoWaySearcher};
use crossbeam::sync::MsQueue;

const XML_PREFIX: &'static [u8] = b"<?xml version=";
const XML_SUFFIX: &'static [u8] = b"</plist>";

pub struct BuffersPool {
    queue: MsQueue<Vec<u8>>,
}

impl BuffersPool {
    pub fn acquire(&self) -> Vec<u8> {
        if let Some(mut vec) = self.queue.try_pop() {
            vec.clear();
            vec
        } else {
            Vec::new()
        }
    }

    pub fn release(&self, vec: Vec<u8>) {
        self.queue.push(vec);
    }
}

impl Default for BuffersPool {
    fn default() -> Self {
        BuffersPool { queue: MsQueue::new() }
    }
}

pub struct Context {
    prefix_searcher: TwoWaySearcher<'static>,
    suffix_searcher: TwoWaySearcher<'static>,
    pub buffers_pool: BuffersPool,
}

impl Context {
    /// Returns a plist content in a `data`.
    pub fn find_plist<'b>(&self, data: &'b [u8]) -> Option<&'b [u8]> {
        let start_i = self.prefix_searcher.search_in(data);
        let end_i = self.suffix_searcher.search_in(data).map(|i| i + XML_SUFFIX.len());

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
            prefix_searcher: TwoWaySearcher::new(XML_PREFIX),
            suffix_searcher: TwoWaySearcher::new(XML_SUFFIX),
            buffers_pool: BuffersPool::default(),
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

    #[test]
    fn test_buffers_pool() {
        let pool = BuffersPool::default();
        let mut buf = pool.acquire();
        expect!(buf.iter()).to(be_empty());
        buf.push(1);
        buf.push(2);
        expect!(buf.iter()).to(have_count(2));
        pool.release(buf);

        let buf = pool.acquire();
        expect!(buf.iter()).to(be_empty());
        expect!(buf.capacity()).to(be_greater_than(0));
    }
}
