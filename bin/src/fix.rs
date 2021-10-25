use std::borrow::Cow;

use rnix::TextRange;

mod all;
pub use all::all;

mod single;
pub use single::single;

type Source<'a> = Cow<'a, str>;

#[derive(Debug)]
pub struct FixResult<'a> {
    pub src: Source<'a>,
    pub fixed: Vec<Fixed>,
}

#[derive(Debug, Clone)]
pub struct Fixed {
    pub at: TextRange,
    pub code: u32,
}

impl<'a> FixResult<'a> {
    fn empty(src: Source<'a>) -> Self {
        Self { src, fixed: Vec::new() }
    }
}
